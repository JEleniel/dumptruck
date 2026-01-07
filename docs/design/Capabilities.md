# DumpTruck Capabilities

## Overview

DumpTruck analyzes, normalizes, deduplicates, and enriches bulk data dumps through a multi-layer normalization and comparison pipeline. This document specifies the normalization and validation requirements for each pipeline stage.

---

## 1. Data Input & Format Detection

### Requirements

- **Format Detection**: Automatically identify file format (CSV, TSV, JSON, YAML, XML, Protocol Buffers, BSON) using fingerprints and headers
- **Multi-Size Support**: Handle files of unknown size without size limits; use streaming for memory efficiency
- **Field Identification**: Extract and catalog field names, types, and constraints from data
- **Integrity Verification**: Compute multi-hash signatures (SHA-256) of analyzed files to detect exact duplicates
- **Duplicate Detection**: Compare hash signatures to previously stored signatures in data store; skip re-analysis if duplicate

---

## 2. Structural Normalization

Normalize raw input data into well-formed, canonical CSV with UTF-8 encoding.

### Field & Column Normalization

- **Naming**: Normalize field names and sort columns alphabetically
- **Encoding**: Use UTF-8 with proper quoting for CSV output
- **Type Inference**: Detect field types (string, integer, float, timestamp, boolean)
- **Type Ambiguity Resolution**: Distinguish `"00123"` (string) from `123` (integer)
- **Field Requirement**: Detect optional vs mandatory fields

### Value-Level Normalization

- **Whitespace**:
    + Trim leading/trailing spaces from all fields _except_ passwords and sensitive data
    + Collapse multiple internal spaces where non-semantic
    + Normalize tabs vs spaces; remove indentation artifacts in structured data
    + Preserve whitespace in free-text fields unless explicitly intended

- **Unicode Canonicalization**:
    + Convert all interchangeable Unicode to NFKC canonical form
    + Support full Unicode character set via UTF-8

- **Numeric Values**:
    + Normalize integers vs floats (`1` vs `1.0`)
    + Remove thousands separators
    + Normalize scientific notation
    + Clamp precision to meaningful digits

- **Boolean Values**:
    + Map all variants to lowercase `true` or `false`
    + Supported variants: `T/F`, `1/0`, `yes/no`, `Y/N`, `true/false`

- **Dates & Times**:
    + Canonicalize to ISO-8601/RFC3339 format
    + Normalize time zones to UTC
    + Resolve locale ambiguity (`MM/DD` vs `DD/MM`)
    + Standardize precision (seconds vs milliseconds)

- **Null & Empty Semantics**:
    + Canonicalize all null/empty representations: `NULL`, `null`, `""`, `"N/A"`, `"none"`, `"-"`
    + Distinguish: missing field vs explicit null vs empty value
    + Domain-specific rules determine if absence equals null

- **Domain Normalization**:
    + Normalize equivalent values by domain (e.g., `NY` → `New York`, `US` → `USA`)
    + Use controlled vocabularies and case-folding with domain rules

### Encoding & Binary Data

- **Line Endings**: Use only Unix LF (`\n`) line endings
- **BOMs & Separators**: Remove byte order marks and invisible separators
- **Binary Data**: Encode as BASE64
- **Endianness**: Normalize for binary formats (protobuf → JSON extraction recommended)

### Non-Semantic Field Exclusion

Exclude or isolate fields that are non-deterministic or system-generated:

- UUIDs and auto-generated IDs
- Timestamps (creation, modification times)
- Checksums and hash values
- Session tokens
- Audit fields (`created_at`, `updated_by`, `row_number`)
- Comments and metadata

---

## 3. Record Ordering & Determinism

### Record Ordering

- **Primary Sort**: Order records by:
    + Primary key (if identifiable: `ID`, `UserID`, etc.)
    + Stable hash of normalized record (fallback)
- **Set/Array Canonicalization**: Enforce deterministic order for array values within fields

### Numeric Precision & Rounding

- Normalize rounding strategy consistently across all numeric values
- Define tolerance thresholds for approximate equality (epsilon) for floating-point comparisons
- NaN and infinity: canonicalize representation (`NaN`, `Infinity`, `-Infinity`)

---

## 4. Deduplication & Identity Resolution

### Key Identification

- **Natural Keys**: Primary key identifiers inherent to the data (e.g., email, SSN)
- **Composite Keys**: Multiple fields that together uniquely identify a record
- **Surrogate Keys**: System-generated identifiers (often excluded from comparison)

### Duplicate Detection

- **Hash-Based Deduplication**: Compute SHA-256 of normalized records for O(1) lookup
- **Semantic Deduplication**: Detect functionally equivalent records (e.g., "John Smith" vs "J. Smith")
- **Record Alignment**: Align entities before field-by-field comparison; compare unaligned entities yields invalid diffs

### Similarity Matching

- **Vector Embeddings**: Optional 768-dimensional embeddings (Nomic/Ollama) for near-duplicate detection
- **Bloom Filter Sync**: Distributed deduplication via peer discovery for multi-instance deployments

---

## 5. Enrichment & Intelligence

### Identity Detection

- **Email Canonicalization**: Normalize email variants (e.g., `googlemail.com` → `gmail.com`; remove plus-addressing)
- **PII/NPI Detection**: Identify 16+ data types:
    + US: SSN, credit card, phone numbers
    + International: Phone numbers (15+ countries), national IDs (15+ countries)
    + Financial: IBAN, SWIFT codes, cryptocurrency addresses (Bitcoin, Ethereum, XRP)
    + General: IP addresses, digital wallets

- **Weak Password Detection**:
    + Plaintext: Detection via rainbow table (40+ common passwords)
    + Hashed: Detection of weak passwords in bcrypt, scrypt, argon2, MD5, SHA1, SHA256 formats

- **Breach Enrichment**:
    + Have I Been Pwned (HIBP) API integration for real-time breach lookups
    + Co-occurrence analysis: Track relationships between addresses and credential associations

---

## 6. Output Serialization & Comparison

### Deterministic Serialization

After all normalization layers:

- **Stable JSON**: Deterministic key ordering for JSON output
- **Canonical CSV**: Properly quoted, normalized records
- **Canonical XML**: C14N (canonical XML) for structured formats

### Comparison Strategy

Choose comparison semantics explicitly based on use case:

| Strategy   | Use Case                           | Implementation |
|------------|------------------------------------|----|
| Exact      | Integrity verification, checksums | Byte-for-byte hash match |
| Field-wise | ETL validation, schema compliance  | Per-field comparison with rules |
| Semantic   | Business logic, domain rules       | Value normalization + equivalence checks |
| Fuzzy      | Human-entered data, typos          | Levenshtein/Jaro-Winkler distance |
| Tolerant   | Measurements, floating-point       | Epsilon-based threshold comparison |

---

## 7. Normalization Pipeline Summary

Apply normalization in this order to ensure deterministic, valid comparisons:

```text
1. Safe Input        → Format detection, encoding validation, size checks
2. Structure         → Field names, types, ordering, column canonicalization
3. Values            → Numeric, boolean, date, whitespace normalization
4. Semantics         → Domain equivalence, null handling, PII detection
5. Deduplication     → Hash/vector/Bloom filter comparison, record alignment
6. Ordering          → Sort by key, enforce determinism
7. Serialization     → Stable JSON/CSV, checksum generation
8. Comparison        → Strategy-specific diff (exact/semantic/fuzzy/tolerant)
```

**Note**: Skipping any layer produces false positives, false negatives, or meaningless diffs.

---

## 8. Evidence Preservation

### File Identification

Every analyzed file receives:

- **File ID**: UUID v4 (immutable, unique identifier)
- **SHA-256 Hash**: Standard cryptographic hash
- **BLAKE3 Hash**: Modern high-performance hash (redundant verification)
- **Alternate Names**: Track file name variants (same data, different filenames)
- **Timestamps**: Created (receipt) and Processed (pipeline start)

### Purpose

- Forensic verification: Re-analyze same file, compare hashes for integrity
- Chain of evidence: Immutable proof of file identity
- Compliance: GDPR, HIPAA, PCI-DSS audit trail requirements

---

## 9. Compression Handling

### Detection & Extraction

- **Magic Byte Detection**: ZIP (0x504B), gzip (0x1F 0x8B), bzip2 (0x42 0x5A), 7-zip (0x37 0x7A)
- **Nested Extraction**: Unzip/decompress iteratively, track depth
- **Safety Bailout**: Stop after 3 nesting levels to prevent infinite loops
- **Temporary Tracking**: Mark extracted files for secure deletion

### Example

```txt
Input: archive.zip (contains: dataset.tar.gz)
↓ Extract level 1: archive.zip → dataset.tar.gz
↓ Detect: dataset.tar.gz is gzip
↓ Extract level 2: dataset.tar.gz → dataset.tar
↓ Detect: dataset.tar is uncompressed
↓ Process: dataset.tar as streaming records
↓ Cleanup: Shred dataset.tar.gz, dataset.tar after completion
```

---

## 10. Field Identification & Classification

### Field Categories

- **ID Fields**: user_id, account_id, employee_id, member_id, customer_id, transaction_id (numeric/UUID/alphanumeric patterns)
- **Password Fields**: password, passwd, pwd, secret, token, api_key, access_token (length >6, diverse characters)
- **PII Fields**: Social Security numbers, credit card numbers, phone numbers, email addresses, national IDs, IP addresses
- **NPI Fields**: Healthcare records, medical IDs, patient names + DOB, prescription data

### Documentation

Fields tagged with classifications are documented in output metadata:

```json
{
  "field_classifications": {
    "user_id": ["id_field"],
    "email": ["pii_field"],
    "password_hash": ["password_field"],
    "phone": ["pii_field"],
    "medical_id": ["npi_field"]
  }
}
```

---

## 11. Alias Resolution

### Alias Types

- **Email Aliases**: `user@domain.com` vs `user+tag@domain.com`, `user.name@domain.com` vs `username@domain.com`
- **User ID Aliases**: `user_123`, `u123`, `123` (same person, different representations)
- **Phone Normalization**: `555-1234`, `(555) 1234`, `5551234` → canonical form
- **National ID Variants**: With/without hyphens, leading zeros

### Relationship Graph

Aliases linked in graph with confidence scores:

```json
{
  "alias_relationships": [
    {
      "primary_id": "user@example.com",
      "alias_id": "user+spam@example.com",
      "relationship_type": "email_plus_addressing",
      "confidence_score": 0.95
    },
    {
      "primary_id": "john_smith",
      "alias_id": "j_smith",
      "relationship_type": "name_abbreviation",
      "confidence_score": 0.75
    }
  ]
}
```

---

## 12. Anomaly & Novelty Detection

### Entropy Outliers

- **Shannon Entropy**: Compute for each string field
- **Threshold**: Flag values > 3 standard deviations above mean
- **Example**: `aZ9kQ2bM1pLx` (high entropy, possibly encrypted) vs `john.smith@example.com` (normal)

### Unseen Field Combinations

- **Tuple Tracking**: Store `(field1_value, field2_value, field3_value, ...)` tuples
- **Novelty**: Flag new combinations not in baseline
- **Example**: Domain "example.com" + user type "admin" never seen together before

### Rare Domain/User Detection

- **Domain Frequency**: Flag domains appearing < 5 times (potential typos)
- **User Frequency**: Flag users with anomalous breach/credential patterns

### Unexpected Credential Formats

- **Pre-hashed Patterns**: Inconsistent salt patterns, non-standard formats
- **Encoding Anomalies**: Non-UTF8 patterns post-Unicode normalization
- **Delimiter Anomalies**: Multi-part credentials with unusual separators

### Statistical Baseline Deviation

- **HIBP Baseline**: Compare credential distribution against known breaches
- **Thresholds**: Flag if weak password % > 2x baseline, specific format > 3x frequency

---

## 13. Risk Scoring (0-100)

### Calculation Formula

```rust
Score = min(100,
  weak_password_count * 3 +           // +30 max (10+ weak passwords)
  hashed_credential_count * 2 +       // +20 max (10+ hashed)
  (has_bcrypt|scrypt|argon2 ? 20:0) + // +20 if salted
  (in_hibp_breach ? 15:0) +           // +15 if breached
  anomaly_score / 5                   // +15 max from anomalies
)
```

### Interpretation

- **0-25**: Low risk (no weak passwords, no breaches)
- **26-50**: Medium risk (some weak passwords, no breaches)
- **51-75**: High risk (many weak passwords OR breached)
- **76-100**: Critical risk (multiple risk factors combined)

---

## 14. Secure Deletion (File Shredding)

### NIST SP 800-88 3-Pass Overwrite

1. **Pass 1**: Fill with 0x00 (zeros)
2. **Pass 2**: Fill with 0xFF (ones)
3. **Pass 3**: Fill with random data

### Files Shredded

- Extracted archives (Stage 2)
- Temporary processing artifacts
- Cache files (if enabled)
- Parsing buffers

### Performance

- Small files (<1MB): <1ms
- Medium files (1-100MB): 10-100ms
- Large files (100MB+): 100-500ms

---

## 15. Chain of Custody & Audit Trail

### Signature Record

Each analysis submission generates:

- **Entry ID**: UUID v4
- **File ID**: Reference to file_metadata
- **Operator**: User/service identity (from OAuth token)
- **Timestamp**: ISO-8601 analysis time
- **Action**: "analyze", "enrich", "export", "import"
- **File Hash**: SHA-256 (immutable proof)
- **Record Count**: Number of records processed
- **ED25519 Signature**: Cryptographic proof of operator + timestamp + data

### Compliance

- **GDPR**: Audit trail requirements
- **HIPAA**: Non-repudiation via cryptographic signatures
- **PCI-DSS**: Chain of evidence for forensic compliance
- **ISO 27001**: Information security management

---

## Implementation Status

| Capability | Status | Location |
|------------|--------|----------|
| Format detection | ✅ Implemented | `src/adapters.rs` |
| Streaming read | ✅ Implemented | `src/safe_ingest.rs`, `src/streaming.rs` |
| Unicode normalization | ✅ Implemented | `src/normalization.rs` |
| Hashing & deduplication | ✅ Implemented | `src/hash_utils.rs` |
| Email canonicalization | ✅ Implemented | `src/npi_detection.rs` |
| PII/NPI detection | ✅ Implemented | `src/npi_detection.rs`, `src/detection.rs` |
| Weak password detection | ✅ Implemented | `src/rainbow_table.rs` |
| Evidence Preservation | ⏳ Planned | `src/evidence.rs` |
| Compression Handling | ⏳ Planned | `src/compression.rs` |
| Field Identification | ⏳ Planned | `src/field_identification.rs` |
| Alias Resolution | ⏳ Planned | `src/alias_resolution.rs` |
| Anomaly Detection | ⏳ Planned | `src/anomaly_detection.rs` |
| Risk Scoring | ⏳ Planned | `src/risk_scoring.rs` |
| Secure Deletion | ⏳ Planned | `src/secure_deletion.rs` |
| Chain of Custody | ⏳ Planned | `src/chain_of_custody.rs` |
| HIBP integration | ⏳ Planned | `src/hibp.rs` |
| Vector embeddings | ⏳ Planned | `src/ollama.rs` |
| Peer discovery & sync | ⏳ Planned | `src/peer_discovery.rs`, `src/peer_sync.rs` |
