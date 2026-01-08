# Test Fixtures Matrix

This directory contains comprehensive test fixtures for validating Dumptruck's analysis pipeline across supported formats and edge cases.

## File Index

### Well-Formed Data (Production-Quality)

**well_formed_credentials.csv** - Standard CSV with clean data, consistent columns, 10 rows

- Use case: Normal operation validation
- Expected events: 10 new addresses, 0 duplicates

### Malformed Data (Error Handling)

**missing_header.csv** - CSV without column headers

- Use case: Header detection and error handling
- Expected error: Validation or graceful degradation

**mismatched_columns.csv** - Rows with inconsistent column counts

- Use case: Column count validation
- Expected events: Column validation errors, selective row acceptance

**empty_fields.csv** - CSV with missing/empty values

- Use case: Null/empty value handling
- Expected behavior: Graceful handling or skip

**whitespace_variations.csv** - Leading/trailing whitespace in fields

- Use case: Normalization and trimming validation
- Expected behavior: Spaces stripped or preserved as configured

### Encoding & Character Tests

**unicode_addresses.csv** - UTF-8 addresses with Cyrillic, CJK, diacritics

- Use case: Unicode normalization testing
- Expected behavior: Canonical form matching, no corruption
- Contains: Russian (москва), Chinese (北京), French (café), Spanish (josé), Greek (αθήνα), Japanese (東京)

**special_characters.csv** - Quoted fields, commas in data, escaped characters

- Use case: CSV escaping and special character handling
- Expected behavior: Correct parsing despite special chars

### Password Format Tests

**hashed_credentials.csv** - Mixed hash types (MD5, SHA1, SHA256)

- Use case: Rainbow table and hash detection
- Expected behavior: Hash recognition, weak password detection
- Hash types: MD5 (32 char hex), SHA1 (40 char hex), SHA256 (64 char hex)

### Deduplication Tests

**duplicate_rows.csv** - Same credentials appearing multiple times

- Use case: Duplicate detection and deduplication
- Expected events: `__duplicate_row__` events for matching entries
- Pattern: alice(at)example.com appears 4 times, bob appears 3 times

### Data Heterogeneity

**mixed_format_data.csv** - Plaintext passwords + hashed passwords, metadata columns

- Use case: Mixed data format handling
- Contains: Plaintext, MD5 hashes, SHA256 hashes, source/date metadata

### Alternate Input Formats

**tab_separated.tsv** - Tab-delimited variant of CSV data

- Use case: Format adapter validation for TSV input
- Expected: Same data interpretation as CSV

**json_credentials.json** - JSON array of objects

- Use case: JSON format adapter testing
- Note: The `dumptruck analyze` input parser does not currently support JSON.

**yaml_credentials.yaml** - YAML format

- Use case: YAML format adapter testing
- Note: The `dumptruck analyze` input parser does not currently support YAML.

### Scale Testing

**large_dataset.csv** - 20 rows with potential duplicates

- Use case: Basic scale testing, deduplication at scale
- Expected: Efficient processing, correct duplicate detection

## Usage

### Running All Fixtures Through Pipeline

```bash
# Test well-formed data
cargo run -- analyze tests/fixtures/well_formed_credentials.csv

# Test error handling with malformed data
cargo run -- analyze tests/fixtures/missing_header.csv
cargo run -- analyze tests/fixtures/mismatched_columns.csv

# Test Unicode handling
cargo run -- analyze tests/fixtures/unicode_addresses.csv

# Test hash detection
cargo run -- analyze tests/fixtures/hashed_credentials.csv

# Test TSV input
cargo run -- analyze tests/fixtures/tab_separated.tsv

# Test duplicate detection
cargo run -- analyze tests/fixtures/duplicate_rows.csv
```

### Testing With Supported CLI Flags

```bash
# Enable embeddings enrichment (requires Ollama configured in config)
cargo run -- analyze tests/fixtures/well_formed_credentials.csv --enable-embeddings

# Write results to a JSON file
cargo run -- analyze tests/fixtures/well_formed_credentials.csv --output ./tmp/results.json
```

## Test Coverage

| Aspect | Coverage | Files |
| ------ | -------- | ----- |
| Format | CSV, TSV | Multiple |
| Data Quality | Well-formed, Malformed | 3 files |
| Encoding | ASCII, UTF-8 Unicode | unicode_addresses.csv |
| Password Type | Plaintext, Hashes (MD5/SHA1/SHA256) | hashed_credentials.csv, mixed_format_data.csv |
| Edge Cases | Empty fields, Whitespace, Special chars, Duplicates | 4 files |
| Scale | Small (5-10 rows), Medium (20 rows) | multiple |
| Enrichment | With/Without metadata | mixed_format_data.csv |

## Expected Pipeline Behavior

For **well_formed_credentials.csv**:

- ✅ All 10 rows parsed
- ✅ Column headers detected: email, username, password
- ✅ 10 `__new_address__` events
- ✅ File metadata computed: `__file_hash__` (MD5 + SHA256)
- ✅ Row hashes computed: `__address_hash__`
- ✅ 0 duplicate detection events

For **duplicate_rows.csv**:

- ✅ All 8 rows parsed
- ✅ 3 unique addresses identified
- ✅ 5 `__duplicate_row__` events
- ✅ Deduplication logic applied

For **unicode_addresses.csv**:

- ✅ All 7 rows parsed without corruption
- ✅ Unicode normalization applied
- ✅ 7 `__new_address__` events (or fewer if canonicalization matches)
- ✅ Canonical forms stored in database

For **hashed_credentials.csv**:

- ✅ All 6 rows parsed
- ✅ Hash types detected: MD5, SHA1, SHA256
- ✅ Rainbow table lookups attempted for weak passwords
- ✅ Hashes stored alongside plaintext indicators

## Notes

- All test data uses realistic breach patterns but is synthetically created
- Email domains are intentionally diverse (example.com, test.org, etc.) to test domain canonicalization
- Password hashes are real hash values for common weak passwords (aiding rainbow table testing)
- Files are designed to be processable independently or as a batch
