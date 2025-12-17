# Ingestion & Analysis Pipeline Map

Complete synthesis of DumpTruck's data flow from raw input through storage and output.

---

## Pipeline Overview

```text
Raw Data
   ↓
1. Evidence Preservation          → File ID + dual hashes, alternate names captured
   ↓
2. Compression Detection          → ZIP/gzip detection, nested levels (max 3)
   ↓
3. Ingest & Format Detection      → Identify format, extract fields
   ↓
4. Chain of Custody               → Cryptographically signed record created
   ↓
5. Safe Ingest & Validation       → Binary check, UTF-8, size limits
   ↓
6. Structural Normalization       → Canonicalize fields and values
   ↓
7. Field Identification           → ID/password/PII/NPI fields tagged & documented
   ↓
8. Alias Resolution               → Links between entries identified (email, user ID, etc.)
   ↓
9. Deduplication & Identity       → Hash match, vector similarity, field hashing for duplicates
   ↓
10. Anomaly & Novelty Detection   → Entropy outliers, unseen combos, rare domains, format anomalies
   ↓
11. Enrichment & Intelligence     → Embeddings, breach lookup, graph, HIBP background sync
   ↓
12. Analysis & Detection          → PII/NPI, weak passwords, Risk Score (0-100)
   ↓
13. Storage & Persistence         → PostgreSQL with dedup graph + Chain of Custody
   ↓
14. Secure Deletion               → Temporary files shredded (prevent data ghosting)
   ↓
15. Output & Reporting            → JSON/CSV/JSONL with statistics
```

---

## Stage 1: Evidence Preservation

**Input**: Raw files from unknown sources

**Actions**:

- Generate unique file identifier (UUID + timestamp)
- Compute dual hash signatures (SHA-256 + BLAKE3)
- Capture all file name variants for the same data
- Store as alternate names in metadata

**Purpose**: Establish unique identity and immutable proof of file origin; prevent tampering

**Output**: File ID, dual signatures, alternate names list

**Storage**: `file_metadata` table with (file_id, sha256_hash, blake3_hash, alternate_names ARRAY, created_at)

**Code**: `src/handlers.rs`, new: `src/evidence.rs`

---

## Stage 2: Compression Detection

**Input**: Files from Stage 1

**Detection**:

- Magic byte detection: ZIP (0x504B), gzip (0x1F 0x8B), bzip2 (0x42 0x5A), 7-zip (0x37 0x7A)
- Nested compression tracking: unzip/decompress, track depth (max 3 levels)
- Safety bailout: After 3 levels, log warning and process as-is

**Processing**:

- If compressed: extract to temporary directory, process contents
- Temporary files marked for shredding (Stage 14)

**Output**: List of extracted files (or original if not compressed)

**Code**: `src/compression.rs`, `src/handlers.rs`

---

## Stage 3: Ingest & Format Detection

**Input**: Files (compressed or uncompressed) from Stage 2

**Actions**:

- Fingerprint-based format detection (magic bytes, headers)
- Field name extraction and type cataloging
- Streaming reads for memory efficiency

**Supported Formats**: CSV, TSV, JSON, YAML, XML, Protobuf, BSON

**Output**: Structured records with field metadata

**Code**: `src/safe_ingest.rs`, `src/adapters.rs`

---

## Stage 4: Chain of Custody

**Input**: File metadata from Stage 1 + format info from Stage 3

**Actions**:

- Create Chain of Custody entry in database
- Include: file_id, operator (user/service), timestamp, action (ingest), file_hash, record_count_estimate
- Sign entry with system private key (ED25519)
- Log entry as immutable record

**Purpose**: Forensic trail for regulatory compliance (GDPR, HIPAA, PCI-DSS)

**Output**: Signed Chain of Custody record with signature verification capability

**Storage**: `chain_of_custody` table with (id, file_id, operator, timestamp, action, file_hash, record_count, signature, verified_at)

**Code**: `src/chain_of_custody.rs`

---

## Stage 5: Safe Ingest & Validation

**Input**: Raw bytes + format information

**Validation Steps**:

1. Binary file detection
2. UTF-8 validation with lossy recovery (U+FFFD)
3. File size limits (100MB per file)
4. Row count verification

**Error Handling**: Log + continue (zero-crash guarantee)

**Output**: Validated, UTF-8 encoded records

**Code**: `src/safe_ingest.rs`

---

## Stage 6: Structural Normalization

**Input**: Validated records

**Field-Level Changes**:

- Alphabetical column ordering
- Type inference (string, int, float, timestamp, bool)
- Nullable detection

**Value-Level Changes**:

_Unicode_: NFKC decomposition, case-folding, punctuation normalization

_Numeric_: Normalize `1` vs `1.0`, remove separators, handle scientific notation

_Boolean_: Map variants to `true`/`false`

_Dates_: ISO-8601/RFC3339, UTC, locale resolution

_Whitespace_: Trim (except passwords), collapse, normalize tabs

_Domain_: Email aliases (`googlemail.com` → `gmail.com`), state codes

_Non-Semantic_: Exclude UUIDs, timestamps, checksums, audit fields

**Detailed Spec**: See `docs/design/Capabilities.md` (Sections 2-3)

---

## Stage 7: Field Identification

**Input**: Normalized records from Stage 6

**Actions**:

- Scan all fields for ID, password, PII, or NPI patterns
- Tag fields with classification (id_field, password_field, pii_field, npi_field)
- Document field types in report metadata
- Flag sensitive fields for special handling

**Classifications**:

- **ID Fields**: user_id, account_id, employee_id, member_id, customer_id, transaction_id (patterns: numeric, UUID, alphanumeric)
- **Password Fields**: password, passwd, pwd, secret, token, api_key, access_token (patterns: length >6, character diversity)
- **PII Fields**: Social Security numbers, credit card numbers, phone numbers, email addresses, national IDs, IP addresses
- **NPI Fields**: Healthcare records, medical IDs, patient names + DOB combinations, prescription data

**Output**: Records with field classifications + documentation in metadata

**Storage**: Enhanced metadata (field_classifications JSONB)

**Code**: `src/field_identification.rs`, `src/detection.rs`

---

## Stage 8: Alias Resolution

**Input**: Alias-resolved records

**Actions**:

- Identify aliases across different field representations:
    + Email aliases: `user@domain.com` vs `user+tag@domain.com`, `user.name@domain.com` vs `username@domain.com`
    + User ID aliases: `user_123`, `u123`, `123`, same person referenced multiple ways
    + Phone number normalization: `555-1234`, `(555) 1234`, `5551234` → canonical form
    + National ID variants: With/without hyphens, leading zeros
- Create alias resolution graph
- Link aliases in `address_alternates` table

**Output**: Records with resolved alias links + relationship graph

**Storage**: `alias_relationships` table with (primary_id, alias_id, relationship_type, confidence_score)

**Code**: `src/alias_resolution.rs`, `src/normalization.rs`

---

## Stage 9: Deduplication & Identity Resolution

**Input**: Alias-resolved records

**Processing Layers**:

**Layer A — Hash-Based**:

- Compute SHA-256 of normalized record
- Store dual hashes for dedup: SHA-256 (exact) + field hash (each field separately)
- O(1) lookup against `canonical_addresses` table
- Field hashing allows detection of duplicates with minor variations

**Layer B — Vector Similarity**:

- Generate 768-dim Nomic embedding (Ollama)
- Query pgvector IVFFlat index (cosine similarity)
- Score ≥ 0.85 → Link as similar
- Score < 0.85 → Treat as new

**Layer C — Bloom Filter Sync** (distributed):

- Per-instance filters for known hashes
- Peer discovery (UDP broadcast)
- Bandwidth-efficient delta sync

**Key Identification**:

- Natural keys (email, phone, SSN)
- Composite keys
- Surrogate keys (often excluded)

**Output**: Records classified as new, duplicate, or similar

**Code**: `src/hash_utils.rs`, `src/ollama.rs`, `src/peer_discovery.rs`, `src/field_hashing.rs`

**Detailed Design**: See `docs/DEDUP_ARCHITECTURE.md`

---

## Stage 10: Anomaly & Novelty Detection

**Input**: Deduplicated records

**Anomaly Detection Categories**:

**Entropy Outliers**:

- Compute Shannon entropy for each string field
- Flag values with entropy > 3 std devs above mean (indicates randomized/encrypted data)
- Example: `aZ9kQ2bM1pLx` vs `john.smith@example.com`

**Unseen Field Combinations**:

- Track `(field1_value, field2_value, field3_value, ...)` tuples
- Flag new combinations not in baseline (e.g., domain + user type combination never seen)
- Baseline built from historical records

**Rare Domain/User Detection**:

- Domain frequency analysis: flag domains appearing < 5 times (potential typos or targeted)
- User frequency analysis: flag users with anomalous breach/credential patterns

**Unexpected Credential Formats**:

- Pre-hashed credentials with inconsistent salt patterns
- Passwords with unusual encoding (non-UTF8 patterns post-Unicode normalization)
- Multi-part credentials with unusual delimiters

**Statistical Deviation from Known Breach Baselines**:

- Compare credential distribution against HIBP baseline (if available)
- Flag if: weak password percentage > 2x baseline, specific format appears > 3x normal frequency

**Output**: Anomaly scores (0-100 per record), classification, flagged fields

**Storage**: `anomaly_scores` table with (record_id, anomaly_type, score, details JSONB)

**Code**: `src/anomaly_detection.rs`, `src/entropy.rs`

---

## Stage 11: Enrichment & Intelligence

**Input**: Anomaly-scored records

**Enrichment Sources**:

**Vector Embeddings**:

- Source: Ollama/Nomic API (768-dim)
- Use: Semantic similarity, near-duplicate clustering
- Storage: `canonical_addresses.embedding`

**Breach Data (HIBP)**:

- Source: Have I Been Pwned API v3
- Data: Breach names, dates, domains, pwn counts
- Storage: `address_breaches` table
- **Background Thread** (Server Mode Only): Continuously calls HIBP with each new email address in background, enriches corpus without blocking ingest

**Co-occurrence Graph**:

- Source: Address pairs in same record
- Use: Relationship clustering
- Storage: `address_cooccurrence` table

**Email Domain Comparison**:

- Each email address compared against email domain substitution list
- Flag if variant exists (e.g., `user@googlemail.com` matches `user@gmail.com`)

**Orchestration**: For each new record: generate embedding → query HIBP (in background for server) → extract pairs → compare domain variants → store

**Output**: Enriched records with embeddings, breach history, graph, domain variants

**Code**: `src/enrichment.rs`, `src/ollama.rs`, `src/hibp.rs`

**Detailed Design**: See `docs/ENRICHMENT.md`

---

## Stage 12: Intelligence & Analysis

**Input**: Enriched records

**PII/NPI Detection** (16+ types):

_US_: SSN, credit card, phone numbers (555-xxxx reserved)

_International_: Phone (15+ countries), national IDs (15+ countries)

_Financial_: IBAN, SWIFT, crypto addresses (Bitcoin, Ethereum, XRP)

_General_: IP addresses, digital wallets, emails

**Weak Password Detection**:

_Plaintext_: Rainbow table (40+ common passwords), dynamic loading from `data/wordlists/`

_Pre-Hashed_: MD5, SHA1, SHA256 (unsalted); bcrypt, scrypt, argon2 (salted)

**Risk Score (0-100)**:

Calculated from:

- Weak password count: +30 points per weak password (capped at 30)
- Pre-hashed credential count: +20 points (capped at 20)
- Credential compromise potential: +20 points if bcrypt/scrypt/argon2 present (capped at 20)
- Enrichment risk: +15 points if in known HIBP breaches (capped at 15)
- Anomaly score contribution: +(anomaly_score / 5) points (capped at 15)
- Final: Clamp result to `[0, 100]`

**Output**: Per-row detection results (PII types, weak credentials, Risk Score 0-100) + aggregated statistics

**Code**: `src/npi_detection.rs`, `src/detection.rs`, `src/rainbow_table.rs`, `src/risk_scoring.rs`

---

## Stage 13: Storage & Persistence

**Input**: Analyzed records

**Database Tables**:

**file_metadata** (new):

- file_id (UUID, PK)
- sha256_hash (TEXT, UNIQUE)
- blake3_hash (TEXT, UNIQUE)
- alternate_names (ARRAY TEXT)
- created_at (TIMESTAMP)
- processed_at (TIMESTAMP)

**chain_of_custody** (new):

- id (UUID, PK)
- file_id (FK)
- operator (TEXT)
- timestamp (TIMESTAMP)
- action (TEXT)
- file_hash (TEXT)
- record_count (INT)
- signature (TEXT, ED25519)
- verified_at (TIMESTAMP)

**canonical_addresses** (primary):

- canonical_hash (TEXT, PK)
- address_text (TEXT, UNIQUE)
- normalized_form (TEXT)
- embedding (vector(768))
- field_classifications (JSONB): PII types, detection results
- risk_score (INT, 0-100)
- anomaly_scores (JSONB)
- metadata (JSONB): PII types, detection results

**address_breaches**:

- canonical_hash (FK)
- breach_name, breach_date, domain
- pwn_count, is_verified
- enrichment_timestamp

**address_cooccurrence**:

- address_a_hash, address_b_hash (FKs)
- relationship_count
- first_seen, updated_at
- metadata (JSONB): context, source

**address_alternates**:

- canonical_hash (FK)
- alternate_text
- similarity_score
- match_type (hash|vector)

**alias_relationships** (new):

- primary_id (FK)
- alias_id (FK)
- relationship_type (TEXT)
- confidence_score (FLOAT)

**anomaly_scores** (new):

- record_id (UUID, PK)
- anomaly_type (TEXT)
- score (INT, 0-100)
- details (JSONB)

**Indexes**: PK, UNIQUE, pgvector IVFFlat, FKs, timestamps

**Privacy**: Plaintext stored in `address_text`; history as non-reversible SHA-256 hashes; optional pgcrypto encryption

**Code**: `src/storage.rs`, `src/chain_of_custody.rs`, `src/evidence.rs`, schema in `docker/postgres/init-db.sql`

---

## Stage 14: Secure Deletion

**Input**: Temporary files created during processing (extracted archives, intermediate results)

**Actions**:

- Mark all temporary files for shredding at pipeline end
- Shred using secure overwrite (NIST SP 800-88): 3-pass overwrite (0x00, 0xFF, random)
- Track which files were shredded in audit log
- Verify file removal from filesystem

**Temporary Files to Shred**:

- Extracted files from compressed archives
- Intermediate processing artifacts
- Cache files if enabled
- Temporary parsing buffers

**Error Handling**: Log shred failures but continue (don't crash on permission errors)

**Purpose**: Prevent data ghosting via filesystem recovery tools (forensic anti-forensics)

**Output**: Audit log with shredded file list + verification status

**Code**: `src/secure_deletion.rs`, `src/handlers.rs`

---

## Stage 15: Output & Reporting

**Input**: Storage-persisted records

**Output Formats**: JSON, CSV, JSONL, Text

**Metadata Generated**:

_Per-file_: Records processed/skipped/errors, format, timing, detection metrics, evidence ID, Chain of Custody link

_Aggregate_: Total unique addresses, hashed credentials detected, weak passwords, duplicates, risk distribution, anomaly summary

**Serialization**: Deterministic key ordering, proper CSV quoting, C14N XML

**Code**: `src/output.rs`, `src/handlers.rs`

---

## Comparison Strategies

| Strategy   | Use Case               | Implementation                  | Stage |
| ---------- | ---------------------- | ------------------------------- | ----- |
| Exact      | Integrity verification | SHA-256 + BLAKE3 match          | 1     |
| Field-wise | ETL validation         | Per-field rules                 | 7     |
| Semantic   | Business logic         | Normalization + equivalence     | 6-9   |
| Fuzzy      | Human errors, typos    | Vector similarity               | 9     |
| Anomaly    | Outlier detection      | Entropy + statistical deviation | 10    |
| Tolerant   | Measurements           | Epsilon thresholds              | 6     |

---

## Execution Modes

**CLI Mode** (`src/cli.rs`):

- Glob pattern support
- Parallel workers (rayon)
- Multiple output formats
- Ad-hoc analysis
- Evidence preservation with file IDs + dual hashes
- Chain of Custody logging

**Server Mode** (`src/server.rs`):

- TLS 1.3+ with OAuth 2.0
- REST API (`POST /api/v1/ingest`, `GET /api/v1/status/{id}`)
- Background worker pool
- Job queue for async processing
- Horizontal scaling ready
- **Background HIBP Thread**: Continuously queries Have I Been Pwned with each new email, enriches corpus without blocking ingest
- Chain of Custody signatures for audit trail
- Secure deletion of temporary files (shredding)

---

## Error Resilience

**Handled Errors**:

- Binary files → Skip, log
- Invalid UTF-8 → Lossy recovery
- Malformed rows → Log + skip, continue
- API rate limits → Backoff + retry
- Missing fields → Mark as null
- Compression errors → Log, process as-is
- Nested compression > 3 levels → Log warning, process as-is

**Zero-Crash Guarantee**: All errors caught, logged, processing continues

---

## Performance Metrics

| Metric                | Value          | Notes                                     |
| --------------------- | -------------- | ----------------------------------------- |
| Throughput            | 800+ req/sec   | Raspberry Pi 5 with TLS                   |
| Latency               | <100ms typical | 1KB-1MB ingest                            |
| Memory                | O(1) constant  | Stream-oriented                           |
| File Size             | GB/TB capable  | 100MB per-file limit                      |
| Embedding             | 100-200ms each | Per address                               |
| Vector Search         | 10-50ms        | IVFFlat index                             |
| Hash Lookup           | <1ms           | O(1) database                             |
| Evidence Preservation | <1ms           | UUID + dual hashing                       |
| Compression Detection | 1-50ms         | Archive scanning                          |
| Anomaly Detection     | 10-100ms       | Per-record entropy + combination analysis |
| Risk Scoring          | <1ms           | O(1) calculation                          |

---

## Testing Coverage

**Unit Tests**: 140+ covering format detection, normalization, deduplication, detection, storage, anomaly detection, evidence preservation, secure deletion

**Integration Tests**: Multi-format ingestion, deduplication, enrichment, output generation, compression handling, Chain of Custody

**Fixtures**: 22 CSV files with 348+ rows covering unique addresses, hashed credentials, weak passwords, PII, international formats

**New Test Categories**:

- Compression detection (ZIP, gzip, nested levels)
- Field identification (ID, password, PII, NPI)
- Alias resolution (email, user ID, phone)
- Anomaly detection (entropy, unseen combinations, rare domains)
- Risk scoring calculation
- Chain of Custody signing + verification
- Secure deletion (file shredding)

---

## Configuration

**File**: `config.default.json`

**Key Settings**:

- `storage`: "database" or "filesystem"
- `database`: PostgreSQL connection string
- `ollama_url`: [localhost:11435](http://localhost:11435) (or custom)
- `hibp_api_key`: API key for breach lookup
- `hibp_background_enabled`: Enable background enrichment in server mode (default: true)
- `max_compression_depth`: Maximum nested compression levels (default: 3)
- `workers`: Worker thread count
- `max_file_size_mb`: 100 (default)
- `tls_cert`, `tls_key`: TLS certificate paths
- `enable_secure_deletion`: Shred temporary files (default: true)
- `secure_deletion_passes`: Number of overwrite passes (default: 3)

---

## Extensibility Points

**Format Adapters** (`src/adapters.rs`):

- Implement `FormatAdapter` trait
- Register in factory

**Storage Backends** (`src/storage.rs`):

- Implement `StorageAdapter` trait
- PostgreSQL, Filesystem, S3 (planned)

**Enrichment Plugins** (`src/enrichment.rs`):

- Custom enrichment logic
- Pipeline integration

**Anomaly Detectors** (`src/anomaly_detection.rs`):

- Custom anomaly scoring
- Baseline learning

---

## Related Documentation

- **Capabilities.md** — Detailed normalization requirements (8 stages)
- **DEDUP_ARCHITECTURE.md** — Deduplication design and database schema
- **ENRICHMENT.md** — Enrichment sources and strategies
- **ARCHITECTURE.md** — System architecture overview
- **COMPONENTS.md** — Component-level design details
