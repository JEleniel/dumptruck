# Components

This document describes Dumptruck's primary components, responsibilities, and public interfaces.

## Pipeline Stages and Components

The 15-stage pipeline maps to the following components:

### 1. Evidence Preservation (src/evidence.rs)

- Responsibilities: Generate unique file IDs, compute dual hash signatures (SHA-256 + BLAKE3), track alternate file names
- Inputs: Raw files
- Output: File metadata with ID + dual hashes

### 2. Compression Detection (src/compression.rs)

- Responsibilities: Detect compressed archives (ZIP, gzip, bzip2, 7-zip), extract with nested level tracking (max 3)
- Safety: Bailout after 3 nesting levels
- Output: Extracted files or pass-through if uncompressed

### 3. Ingestion (src/safe_ingest.rs, src/adapters.rs)

- Responsibilities: Accept input files or streams, detect format, provide uniform input representation
- Inputs: local files, uploaded payloads via API, streamed dumps
- Output: tokenized/streamed records handed to Normalization
- Supported formats: CSV, TSV, JSON, YAML, XML, Protobuf, BSON

### 4. Chain of Custody (src/chain_of_custody.rs)

- Responsibilities: Create cryptographically signed records (ED25519) with operator, timestamp, action, file hash, record count
- Storage: Immutable audit trail in database
- Compliance: GDPR, HIPAA, PCI-DSS forensic requirements
- Output: Signed Chain of Custody entry with verification capability

### 5. Safe Ingest & Validation (src/safe_ingest.rs)

- Responsibilities: Validate binary file detection, UTF-8 validation with lossy recovery, enforce size limits (100MB), row count verification
- Error Handling: Zero-crash guarantee; log + continue
- Output: Validated, UTF-8 encoded records

### 6. Normalization (src/normalization.rs)

- Responsibilities: Normalize fields (alphabetical ordering, type inference), apply canonicalization rules (Unicode NFKC, case-folding, punctuation, numeric, boolean, date normalization)
- Extensibility: format-normalizers for Protobuf/JSON/CSV/etc. can be added as plugins
- Output: Canonical, deterministic records

### 7. Field Identification (src/field_identification.rs)

- Responsibilities: Scan and classify fields as ID/password/PII/NPI; tag sensitive fields; generate field documentation
- Classifications: ID fields (user_id, account_id, etc.), password fields (password, pwd, secret, token), PII/NPI (SSN, phone, email, national ID, etc.)
- Output: Records with field classifications in metadata

### 8. Alias Resolution (src/alias_resolution.rs)

- Responsibilities: Identify and link aliases across field representations (email variants, user ID variants, phone normalization, national ID formats)
- Graph: Build alias resolution graph with confidence scores
- Storage: `alias_relationships` table with primary/alias/relationship_type/confidence_score
- Output: Resolved alias links

### 9. Deduplication & Identity (src/hash_utils.rs, src/field_hashing.rs, src/ollama.rs, src/peer_discovery.rs)

- Responsibilities: Hash-based exact match (SHA-256) + field hashing for minor variations + vector similarity (Ollama) + peer Bloom filter sync
- Dedup Layers: Hash → Vector similarity → Bloom filter peer sync
- Storage: `canonical_addresses` with dual hashes
- Output: Records classified as new, duplicate, or similar

### 10. Anomaly & Novelty Detection (src/anomaly_detection.rs, src/entropy.rs)

- Responsibilities: Entropy outlier detection, unseen field combination tracking, rare domain/user detection, unexpected credential format identification, statistical baseline deviation analysis
- Baseline: Built from historical records
- Storage: `anomaly_scores` table with (record_id, anomaly_type, score 0-100, details JSONB)
- Output: Anomaly scores and flags per record

### 11. Enrichment (src/enrichment.rs, src/ollama.rs, src/hibp.rs)

- Responsibilities: Generate vector embeddings (768-dim Nomic/Ollama), query HIBP API for breach data, build co-occurrence graph, compare email domain variants
- Background Thread (Server Mode): Continuous HIBP enrichment without blocking ingest
- Storage: `canonical_addresses.embedding`, `address_breaches`, `address_cooccurrence`, `address_alternates`
- Output: Enriched records with embeddings, breach history, graph, domain variants

### 12. Analysis & Detection (src/detection.rs, src/npi_detection.rs, src/risk_scoring.rs)

- Responsibilities: PII/NPI detection (16+ types: SSN, credit card, phone, national ID, crypto, IBAN/SWIFT, IP, digital wallet), weak password detection (plaintext + hashed), Risk Score calculation (0-100)
- Risk Scoring: Weak passwords + hashed credentials + HIBP breaches + anomaly contributions
- Output: Per-row detection results + aggregated statistics + Risk Score

### 13. Storage & History (src/storage.rs)

- Responsibilities: Persist analysis outputs, history hashes, metadata, Chain of Custody records, anomaly scores
- Privacy: historic values stored as non-reversible hashes (HMAC or keyed KDF) rather than plaintext
- Backends: abstract storage interface — PostgreSQL primary, filesystem/S3 extensible
- Tables: `file_metadata`, `chain_of_custody`, `canonical_addresses`, `address_breaches`, `address_alternates`, `address_cooccurrence`, `alias_relationships`, `anomaly_scores`

### 14. Secure Deletion (src/secure_deletion.rs)

- Responsibilities: Shred temporary files (NIST SP 800-88 3-pass overwrite), audit logging, verification
- Scope: Extracted archives, intermediate processing artifacts, cache files
- Error Handling: Log failures but continue
- Purpose: Prevent data ghosting via forensic recovery tools
- Output: Audit log with shredded file list + verification status

### 15. Output & Reporting (src/output.rs, src/handlers.rs)

- Responsibilities: Format results (JSON, CSV, JSONL, text), generate metadata (per-file + aggregate), include evidence ID + Chain of Custody links
- Metadata: Per-file (records, format, timing, detection metrics, evidence ID), Aggregate (unique addresses, hashed credentials, weak passwords, risk distribution, anomaly summary)
- Serialization: Deterministic key ordering, proper CSV quoting, C14N XML
- Output: Results + audit trail

## Supporting Components

### API & CLI (src/cli.rs, src/server.rs)

- CLI: single-node quick-run tool for analysts with glob pattern support and parallel workers
- API Server: REST endpoints for upload, status, results; supports OAuth2/OIDC and TLS 1.3+
- Background Workers: Job queue orchestration for async processing

### Server Runtime (src/server.rs, src/job_queue.rs, src/deploy_manager.rs)

- Orchestration: Worker processes, request routing, rate-limiting, configuration loading
- Health: Liveness/readiness endpoints, service startup management
- Configuration: config.default.json with all parameters
- Metrics and health endpoints for monitoring

### Extensibility/Plugin System (src/adapters.rs, src/enrichment.rs, src/storage.rs)

- Well-defined extension points: format adapters, enrichment rules, storage adapters, anomaly detectors
- Contract: Small, language-native adapters (prefer Rust traits) with safe interfaces
- Trait-based: FormatAdapter, StorageAdapter, AnomalyDetector

### Observability (src/handlers.rs, logging)

- Telemetry: Structured JSON logging with [INFO] prefixes for progress tracking
- Metrics: Per-stage timing, record counts, detection metrics, anomaly scores, risk distribution
- Health: Service startup status, PostgreSQL/Ollama availability checks
- Audit: Chain of Custody event logging, secure deletion verification

## Data Contracts

- Use compact, documented schemas for record representation inside the pipeline
- Keep raw input separate from normalized/enriched representations
- All detections include confidence scores and audit trails
