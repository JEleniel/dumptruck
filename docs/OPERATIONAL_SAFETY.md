# Operational Safety: File Handling & Database Concurrency

This document analyzes Dumptruck's behavior under two critical operational scenarios:

1. **Large file handling** - Can it process files at OS limits?
2. **Concurrent database access** - Is it safe when multiple instances share a PostgreSQL database?

## Part 1: File Handling & Size Limits

### Current Behavior

**Maximum File Size Check:**

- Location: `src/safe_ingest.rs` line 11
- Hard limit: **100 MB**
- Behavior: Files larger than 100 MB log a warning but **attempt partial processing**

```rust
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;  // 100 MB

if file_size > MAX_FILE_SIZE {
    warnings.push(format!(
        "File is {} MB (max: {} MB), will attempt partial processing",
        file_size / (1024 * 1024),
        MAX_FILE_SIZE / (1024 * 1024)
    ));
}
```

### Problem: Memory-Based File Reading

**Current Implementation (handlers.rs:49):**

```rust
let content = std::fs::read_to_string(file_path).map_err(|e| {
    format!("Failed to read file {:?}: {}", file_path, e)
})?;
```

**Issue:** `read_to_string()` loads the **entire file into memory** before processing.

**Real-world Impact:**

- A 500 MB CSV file = 500 MB of RAM consumed
- A 5 GB JSON file = **entire process crashes** with OOM
- No streaming/chunked processing = all-or-nothing
- Storage backend (FsStorage) also has same issue at line 229

### Safety Assessment: ⚠️ PARTIALLY SAFE

**What works:**

- ✅ Files up to ~100 MB (depending on available RAM)
- ✅ Safe from crashes on binary/malformed data (`safe_ingest` module handles this)
- ✅ Safe from invalid UTF-8 (lossy conversion implemented)
- ✅ Safe from null bytes and non-text content

**What breaks:**

- ❌ Files larger than available RAM will cause OOM crash
- ❌ `safe_ingest.rs` max check (100 MB) is **ignored** - file is still read entirely
- ❌ No streaming CSV/JSON parsing
- ❌ No chunked writing to storage backend

### Recommendation: Streaming Implementation

To handle files up to OS limits (multiple TB on modern systems), implement:

1. **Streaming CSV/JSON parsing** - process rows one-at-a-time
2. **Async streaming file reads** - tokio::fs with BufReader
3. **Streaming storage writes** - write each row immediately, not buffer all
4. **Remove or increase MAX_FILE_SIZE** - 100 MB is artificial constraint

**Estimated Implementation:**

```rust
// Async streaming approach
pub async fn ingest_stream(&self, path: &Path) -> Result<(), Box<dyn Error>> {
    let file = tokio::fs::File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    while let Some(line) = lines.next_line().await? {
        // Process one row at a time
        let row = self.adapter.parse_row(&line)?;
        self.storage.store_row(&row)?;
    }
    Ok(())
}
```

---

## Part 2: Concurrent Database Access

### Current Architecture

**Storage Adapter Pattern:**

- Location: `src/storage.rs` line 12
- Two implementations: `FsStorage` (single-writer) and `PostgresStorage` (multi-writer capable)

**PostgreSQL Connection Model:**

```rust
pub struct PostgresStorage {
    client: Client,  // Single connection per instance
    dataset: Option<String>,
}
```

### Safety Analysis: ✅ SAFE FOR CONCURRENT ACCESS

#### Why PostgreSQL is Safe

1. **ACID Transactions (Default)**
   + Every INSERT is atomic
   + Serialization level: Read Committed (PostgreSQL default)
   + All writes use parameterized queries (safe from injection)

2. **Row-Level Locking**
   + PostgreSQL automatically locks rows during INSERT
   + Prevents "lost update" race conditions
   + No explicit LOCK needed for reads

3. **Unique Constraints**
   + Tables use PRIMARY KEY and UNIQUE indexes
   + Example: `canonical_addresses(canonical_hash TEXT PRIMARY KEY)`
   + Prevents duplicates even with concurrent writes

4. **Index-Based Queries**
   + All reads use indexed lookups: `address_hash`, `credential_hash`, `file_id`
   + Isolation level protects against dirty reads

#### Specific Concurrent Scenarios

**Scenario 1: Two instances insert same address hash**

```
Instance A: INSERT canonical_addresses(canonical_hash='abc123', address_text='john@example.com')
Instance B: INSERT canonical_addresses(canonical_hash='abc123', address_text='john@example.com')

Result: ✅ SAFE - One succeeds, one gets constraint violation
        Both instances detect duplicate via address_exists() check
        No data corruption
```

**Scenario 2: Two instances insert credential with same address**

```
Instance A: INSERT address_credentials(canonical_hash='xyz', credential_hash='cred1')
Instance B: INSERT address_credentials(canonical_hash='xyz', credential_hash='cred1')

Result: ✅ SAFE - One succeeds, duplicate constraint prevents second
        No lost updates, no race conditions
```

**Scenario 3: Three instances read/write simultaneously**

```
Instance A: SELECT * FROM canonical_addresses WHERE canonical_hash='abc'
Instance B: INSERT address_credentials(canonical_hash='xyz', ...)
Instance C: UPDATE address_credentials SET occurrence_count=2 WHERE id=5

Result: ✅ SAFE - All three execute independently
        No locks needed for SELECT
        Instance B/C's INSERT/UPDATE are serialized by row locks
        Isolation level prevents dirty reads
```

### Potential Issues & Mitigations

#### Issue 1: No Explicit Transaction Boundaries

**Current Code (storage.rs:325):**

```rust
fn store_row(&mut self, row: &[String]) -> std::io::Result<()> {
    // Single INSERT - implicitly wrapped in transaction by PostgreSQL
    self.client.execute(&stmt, &[...])?;
    Ok(())
}
```

**Status:** ✅ Not a problem - single INSERT is atomic by default
**Reason:** PostgreSQL autocommit mode (default) commits after each statement

#### Issue 2: Connection Pool Not Used

**Current Code (storage.rs:207):**

```rust
pub fn new(conn_str: &str, dataset: Option<String>) -> std::io::Result<Self> {
    let client = Client::connect(conn_str, NoTls)?;  // Single connection!
    Ok(PostgresStorage { client, dataset })
}
```

**Status:** ⚠️ Not ideal for production, but safe
**Why:**

- Each Dumptruck instance has its own connection
- Multiple instances = multiple connections (safe)
- No connection pooling means some overhead

**Recommendation:** Use `tokio-postgres` with `deadpool` for connection pooling in high-concurrency scenarios

#### Issue 3: Filesystem Storage (Single-Writer Problem)

**Current Code (storage.rs:190):**

```rust
pub struct FsStorage {
    path: PathBuf,
    file: File,  // Single file handle
}
```

**Status:** ⚠️ UNSAFE for concurrent access
**Why:**

- File opened in append mode, but no file locking
- Two instances can write simultaneously and corrupt file
- No atomic row writes

**Safe Usage:** Only use FsStorage with single instance (not shared)

### Concurrency Verdict

| Scenario | Storage | Status | Notes |
|----------|---------|--------|-------|
| Single instance, PostgreSQL | DB | ✅ SAFE | Normal operation |
| Two instances, shared PostgreSQL | DB | ✅ SAFE | Concurrent INSERTs use row locks |
| Three+ instances, shared PostgreSQL | DB | ✅ SAFE | All operations atomic & isolated |
| Multiple instances, shared filesystem | FS | ❌ UNSAFE | File corruption likely |
| Single instance, filesystem | FS | ✅ SAFE | No concurrency issues |

---

## Summary & Recommendations

### File Handling

**Current State:** Partially safe - works up to RAM limits

**Limitations:**

- Hard 100 MB limit in code (but ignored during actual read)
- All-in-memory file processing
- No streaming support
- Will OOM on files larger than available memory

**Recommendations:**

1. **Immediate:** Document 100 MB practical limit in README
2. **Short-term:** Implement streaming CSV/JSON parsing
3. **Long-term:** Support TAR/ZIP compression for large datasets

### Database Concurrency

**Current State:** Fully safe for PostgreSQL backend

**What's protected:**

- ✅ Concurrent INSERTs from multiple instances
- ✅ Concurrent reads with writes
- ✅ Data integrity via constraints & transactions
- ✅ No race conditions on duplicate detection

**What to avoid:**

- ❌ Using FsStorage backend with multiple instances
- ❌ Concurrent filesystem writes without external locking

**Recommendations:**

1. Enforce PostgreSQL for multi-instance deployments (document requirement)
2. Add connection pooling for production (deadpool-postgres)
3. Add explicit transaction support for batch operations
4. Document FsStorage as single-instance only

### Configuration Guidance

**Single Instance, Development:**

- Use FsStorage or PostgreSQL
- No special concurrency considerations
- File size: Up to available RAM

**Multi-Instance, Production:**

- **MUST** use PostgreSQL backend
- Connection pooling recommended
- File size: Up to available RAM per instance
- Use load balancer to distribute ingest jobs

**Large File Processing:**

- Maximum current: ~100 MB (available RAM dependent)
- Recommended: Split into smaller files via preprocessing
- Future: Implement streaming pipeline for unlimited size

---

## Testing Recommendations

### File Handling Tests

```bash
# Test 100 MB file
dd if=/dev/zero of=test_100mb.csv bs=1M count=100
cargo run -- ingest test_100mb.csv

# Test 1 GB file (will OOM with current implementation)
dd if=/dev/zero of=test_1gb.csv bs=1M count=1024
cargo run -- ingest test_1gb.csv  # Expected: OOM error
```

### Concurrency Tests

```bash
# Start two instances with shared PostgreSQL
cargo run -- ingest file1.csv --database "postgresql://localhost/dumptruck" &
cargo run -- ingest file2.csv --database "postgresql://localhost/dumptruck" &

# Check for data corruption or missed records
psql dumptruck -c "SELECT COUNT(*) FROM normalized_rows;"
```

---

## Appendix: Transaction Isolation Levels

PostgreSQL's default isolation level (Read Committed) provides:

| Operation | Thread-Safe | Notes |
|-----------|------------|-------|
| INSERT | ✅ Yes | Row-level lock during commit |
| SELECT | ✅ Yes | No locks acquired |
| UPDATE | ✅ Yes | Row-level lock acquired |
| DELETE | ✅ Yes | Row-level lock acquired |
| Constraint violations | ✅ Yes | Handled atomically |

For highest safety in Dumptruck's use case, all queries use:

1. **Parameterized queries** (protection against injection)
2. **Indexed lookups** (consistent query plans)
3. **Explicit constraints** (duplicate prevention)
