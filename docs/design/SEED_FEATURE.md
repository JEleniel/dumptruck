# Seed Feature Design

## Overview

The seed feature enables bulk initialization of Dumptruck's database with predefined data. A seed operation:

1. **CLI**: `dumptruck seed <folder>` - Recursively scans a folder for data files (CSV/JSON/XML)
2. **Database**: Creates `data/seed.db` with all data normalized and processed (SQLite)
3. **Signature**: Computes SHA-256 signature of the **seed.db file itself** and stores in main database's `seed_metadata` table
4. **Deployment**: Deploys `seed.db` artifact to multiple instances
5. **Startup Verification**: Each instance verifies if its local `seed.db` has changed
6. **Auto-Import**: If `seed.db` signature differs from stored metadata, automatically imports all data into main database

## Key Design Changes

**Previous Design**: Tracked signatures of input data files (CSV/JSON/XML)

- Problem: Signature changed whenever source files changed, requiring rebuilds
- Issue: Not suitable for artifact deployment (what matters is the built seed.db)

**Current Design**: Tracks signature of the **output seed.db file itself**

- Benefit: Signature represents the exact artifact that was built and deployed
- Benefit: Multiple instances can share the same seed.db, each checks locally if already imported
- Benefit: No need to redeploy source files, only the built artifact

## Use Cases

- **Standard Datasets**: Organizations pre-populate known breach data, common weak passwords, historical archives
- **Multi-Instance Deployment**: Seed.db is created once, deployed to all instances via artifact storage (S3/GCS/blob)
- **Quick Setup**: New instances download seed.db, verify it's different from local copy, import if needed
- **Disaster Recovery**: Seed.db is backed up separately, restored by copying the file to data/ directory
- **Compliance**: Audit trail of when seed was created, signature of exact artifact, import logs on each instance

## Architecture

### Seed File Structure

```text
seed_folder/
├── data_set_1.csv
├── subfolder/
│   ├── data_set_2.json
│   └── deep_subfolder/
│       ├── data_set_3.csv
│       └── data_set_4.xml
└── archive.json
```

All files are recursively discovered and processed in order.

### Seed Database (`data/seed.db`)

SQLite database created during `dumptruck seed` command:

**Tables**:

- `canonical_addresses` - Deduplicated addresses from all files
- `address_breaches` - HIBP enrichment data (if enabled)
- `address_cooccurrence` - Relationships between addresses
- `seed_metadata` - Seed creation info and file signatures

### Metadata Tracking

The `seed_metadata` table stores information about created seeds:

```sql
CREATE TABLE seed_metadata (
  id INTEGER PRIMARY KEY,
  seed_path TEXT NOT NULL UNIQUE,     -- Path the seed was created from
  created_at INTEGER NOT NULL,        -- Unix timestamp of seed creation
  file_signature BLOB NOT NULL,       -- SHA-256 signature of seed.db file
  file_manifest TEXT NOT NULL,        -- JSON list of files that were ingested
  total_rows INTEGER NOT NULL,        -- Total rows processed from all files
  unique_addresses INTEGER NOT NULL,  -- Unique addresses in the seed
  verification_count INTEGER NOT NULL,-- Times this seed was verified at startup
  last_verified_at INTEGER            -- Unix timestamp of last verification
);
```

### Seed.db Signature Calculation

The signature represents the exact **output artifact**, not the input files:

1. **After seed.db is created**: Process all data files through ingest pipeline
2. **Compute seed.db signature**:
   + Read `seed.db` file in 4KB chunks
   + Compute SHA-256 hash of all file contents
   + Result: Single signature for the entire built database
3. **Store signature**: Persist in `seed_metadata.file_signature` in main database
4. **Deployment**: Copy `seed.db` artifact to other instances
5. **Verification on startup**:
   + Recompute SHA-256 of local `seed.db` file
   + Compare with stored `file_signature` from seed_metadata
   + If **different** → seed has changed → import new data
   + If **same** → seed unchanged → skip import (data already in main db)

**Benefits**:

- Signature represents the exact artifact that was built and deployed
- Works seamlessly with artifact storage (S3, GCS, Artifact Registry)
- Multiple instances can share the same `seed.db`, each verifies locally
- No need to redeploy source files, only the built artifact matters

## Command Specification

### `dumptruck seed <folder_path> [options]`

**Arguments**:

- `<folder_path>` - Path to folder containing data files (required)

**Options**:

- `--output <path>` - Custom path for seed database (default: `data/seed.db`)
- `--enrichment <yes|no>` - Include HIBP enrichment (default: `no`)
- `--embeddings <yes|no>` - Include vector embeddings (default: `no`)
- `--workers <n>` - Parallel processing threads (default: 4)
- `-v, --verbose` - Debug logging

**Output**:

```json
{
  "status": "created",
  "seed_db_path": "data/seed.db",
  "folder_path": "/path/to/seed/data",
  "files_discovered": 4,
  "rows_processed": 1250,
  "unique_addresses": 450,
  "file_signature": "a1b2c3d4...",
  "created_at": 1703520000,
  "estimated_import_time_minutes": 2
}
```

## Startup Verification Flow

On server startup, Dumptruck automatically checks if seed.db needs to be imported:

1. **Check seed database existence**: Does `data/seed.db` exist locally?
   + NO: No seed to import, continue normally
   + YES: Continue to step 2

2. **Load seed metadata from main database**: Query `seed_metadata` table
   + Get: `file_signature` (SHA-256 of the seed.db that was deployed)
   + Get: `created_at` (when the seed was originally created)
   + If no metadata row exists: First time seeing this seed, proceed with import

3. **Compute current seed.db signature**: Hash the local `data/seed.db` file
   + Read file in 4KB chunks
   + Compute SHA-256 of entire file contents
   + Result: Current signature

4. **Compare signatures**:
   + **Match**: Seed.db is identical to deployed version → data already imported
     - Update `verification_count` and `last_verified_at`
     - Log: `[INFO] Seed verified (identical to deployed version)`
     - Continue startup normally
   + **Differ**: Seed.db is new or changed → requires import
     - Proceed to step 5

5. **Import seed data**:
   + Load all records from `data/seed.db`
   + Ingest through full pipeline (deduplication, enrichment, etc.)
   + Merge into main database
   + Update `seed_metadata` with new signature and stats
   + Log: `[INFO] Seed imported: <X> rows, <Y> unique addresses merged`

6. **Handle errors**:
   + Seed database corrupted → Log error, skip import, continue (non-blocking)
   + Seed folder unreachable → Not checked in this flow (only seed.db checked)
   + Duplicate constraint violations → Deduplication handles, merge completes

## Implementation Modules

### `src/seed/builder.rs` (SeedBuilder)

- `new(folder_path, output_path)` - Initialize builder
- `discover_files()` -> Result<Vec<PathBuf>> - Find all data files recursively
- `compute_seed_signature(path)` -> Result<String> - **Static method**: SHA-256 of seed.db file
- `build()` -> Result<SeedInfo, SeedError> - Prepare seed metadata
- `SeedInfo`: Seed path, file count, signature (computed after DB creation), stats

### `src/seed/manager.rs` (SeedManager)

- `new(seed_db_path)` - Initialize with path to seed.db
- `verify_and_import()` -> Result<SeedImportStats, SeedError> - Async: compute current signature, compare with stored, import if different
- `get_metadata()` -> Result<SeedMetadata, SeedError> - Query seed_metadata table (requires DB connection)
- `update_verification_timestamp()` - Increment verification count
- `SeedImportStats`: signature_matched, rows_imported, addresses_merged, import_attempted

### `src/seed.rs` (Module root)

- Public enum: `SeedError` - IO, FolderNotFound, NoDataFiles, SignatureError, ImportFailed
- Public struct: `SeedInfo` - Seed creation metadata
- Public struct: `SeedMetadata` - Stored in seed_metadata table
- Public struct: `FileManifest` - List of ingested files
- Re-exports builder and manager

### CLI Integration (`src/cli.rs`, `src/handlers.rs`)

- `CLI::Seed(SeedArgs)` - Command variant with folder, output, options
- `handlers::seed()` - Async handler:
  1. Delete existing seed.db (fresh creation)
  2. Create parent directories
  3. Discover files in source folder
  4. Create empty seed.db file
  5. Compute seed.db signature via SeedBuilder::compute_seed_signature()
  6. Output JSON with signature and metadata

### Database Schema (`src/storage/db/schema.rs`)

- `seed_metadata` table: seed_path, created_at, file_signature, file_manifest, total_rows, unique_addresses, verification_count, last_verified_at
- Created automatically on first `dumptruck seed` command or server startup

## Error Handling

**SeedError enum**:

- `FolderNotFound(path)` - Seed source folder missing
- `NoDataFiles(path)` - Folder has no recognizable data files (CSV/JSON/XML/etc.)
- `DatabaseError(msg)` - seed.db cannot be accessed or is corrupted
- `SignatureError(msg)` - Failed to compute SHA-256 signature
- `ImportFailed(msg)` - Failed to import seed data into main database
- `IoError(io::Error)` - File system I/O error
- `SerializationError(json::Error)` - JSON parsing/writing error

All errors are logged with context but non-blocking during startup.

## Performance Characteristics

- **Signature Computation**: O(n) where n = total bytes in all files
    + Streaming: 4KB buffer for memory efficiency
    + Example: 100MB seed folder → ~50ms on modern disk

- **Startup Verification**: Sub-100ms (signature comparison only)
    + No import if signature matches (cached)

- **Full Import**: Depends on data size
    + Uses existing ingest pipeline (parallel processing available)
    + Example: 1,000 addresses → ~500ms with 4 workers

## Database Schema Changes

**New table in schema migration**:

```sql
CREATE TABLE IF NOT EXISTS seed_metadata (
  id INTEGER PRIMARY KEY,
  seed_path TEXT NOT NULL UNIQUE,
  created_at INTEGER NOT NULL,
  file_signature BLOB NOT NULL,
  file_manifest TEXT NOT NULL,
  total_rows INTEGER NOT NULL,
  unique_addresses INTEGER NOT NULL,
  verification_count INTEGER NOT NULL DEFAULT 0,
  last_verified_at INTEGER,
  CONSTRAINT seed_path_unique UNIQUE (seed_path)
)
```

## Security Considerations

1. **Signature Authenticity**: Verify with HMAC-SHA256 if seed is untrusted
   + Use app secret key to compute HMAC instead of plain SHA-256
   + Store HMAC in metadata for verification
   + Prevents tampering if seed database is copied

2. **Large Seed Files**: Implement size limits
   + Max 10GB per file in seed folder
   + Max 50GB total seed database
   + Configurable via `SEED_MAX_SIZE` constant

3. **Permissions**: Seed folder should be readable by dumptruck process
   + Check on startup, log warning if not accessible
   + Non-blocking: app continues without seed

4. **Audit Trail**:
   + Log when seed is created (file count, signature)
   + Log when seed is verified/imported (timestamp, change detection)
   + Include in audit logs for compliance

## Testing Strategy

**Unit Tests** (`tests/seed.rs`):

1. `test_seed_builder_creates_database` - Builder creates seed.db
2. `test_seed_signature_deterministic` - Same files → same signature
3. `test_seed_signature_changes_on_file_modification` - Modified file detected
4. `test_seed_manager_imports_on_signature_mismatch` - Import triggered
5. `test_seed_manager_skips_on_signature_match` - Cached seed not reimported
6. `test_seed_error_handling_missing_folder` - Graceful error
7. `test_seed_recursive_folder_discovery` - Finds all files in subdirs
8. `test_seed_multiple_formats` - CSV, JSON, XML files in same seed

**Integration Tests** (`tests/e2e_seed.rs`):

1. Full seed creation + verification cycle
2. Startup import of seed data
3. Detection pipeline runs on seed-imported addresses
4. Concurrent access to seed and main database

**Manual Testing**:

```bash
# Create seed from folder
./dumptruck seed ./data/seed_files --output ./test_seed.db -v

# Verify output
sqlite3 ./test_seed.db "SELECT COUNT(*) FROM canonical_addresses;"

# Start server (should auto-import seed on first run)
./dumptruck server &

# Check logs for seed import message
grep -i "seed" dumptruck.log
```

## Documentation Updates

- **docs/design/SEED_FEATURE.md** - This design document
- **docs/CLI_USAGE.md** - Add `seed` command with examples
- **docs/architecture/ARCHITECTURE.md** - Add seed stage to pipeline
- **README.md** - Update feature list with seed capability
- **PROGRESS.md** - Mark seed feature as implemented

## Future Enhancements

1. **Seed Versioning**: Track seed versions, support seed upgrade
2. **Differential Seeds**: Create incremental seeds instead of full
3. **Seed Encryption**: Encrypt seed.db with app secret key
4. **Remote Seeds**: Support seed URLs (download from S3, git, etc.)
5. **Seed Validation**: Checksum verification for distributed seeds
6. **Seed Merge**: Combine multiple seeds into one database

## Acceptance Criteria

✅ Feature is complete when:

- [X] Design document complete
- [ ] `dumptruck seed` command works with folder path argument
- [ ] Creates `data/seed.db` with all data processed
- [ ] Computes deterministic SHA-256 signature
- [ ] On startup, verifies signature and auto-imports if changed
- [ ] All 15 detection stages run on seed data
- [ ] Unit tests: 8+ covering all code paths
- [ ] Integration tests: 4+ covering seed lifecycle
- [ ] CLI_USAGE.md updated with examples
- [ ] All 228+ existing tests still pass
- [ ] Zero compiler warnings from new code
- [ ] Documentation complete and markdown compliant
