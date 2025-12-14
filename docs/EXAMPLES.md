# Dumptruck Usage Examples

Practical examples for using Dumptruck to analyze bulk data dumps, identify credentials, and detect breaches.

## Table of Contents

- [Quick Start](#quick-start)
- [Basic Data Analysis](#basic-data-analysis)
- [Multiple Files & Patterns](#multiple-files--patterns)
- [Output Formats](#output-formats)
- [Enrichment Features](#enrichment-features)
- [Storage Options](#storage-options)
- [Parallel Processing](#parallel-processing)
- [Real-World Workflows](#real-world-workflows)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Analyze a Single CSV File

```bash
# Simplest usage - analyze a CSV and output results to stdout
cargo run -- ingest tests/fixtures/test_creds_small.csv
```

Output (default JSON format):

```json
{
  "rows_processed": 10,
  "unique_addresses": 8,
  "addresses_found": [
    "alice@example.com",
    "bob",
    "charlie@sub.example.org",
    "dave_1990",
    "eve+test@example.co.uk",
    "mallory",
    "oscar@example.com",
    "user.name"
  ],
  "hashed_credentials_detected": 0,
  "weak_passwords_found": 4,
  "weak_password_ids": [
    "alice@example.com",
    "dave_1990",
    "mallory",
    "peggy"
  ],
  "breached_addresses": 0,
  "metadata": [
    "Processed CSV file with 10 rows",
    "Identified 8 unique addresses",
    "Found 4 weak passwords",
    "No breached addresses detected"
  ],
  "errors": []
}
```

### Save Results to File

```bash
# Analyze and save results as JSON
cargo run -- ingest tests/fixtures/test_creds_small.csv -o results.json
```

## Basic Data Analysis

### CSV Analysis

```bash
# Analyze a CSV file with email and credential columns
cargo run -- ingest tests/fixtures/test_creds_100.csv --output-format text
```

Output:

```text
=== Dumptruck Analysis Results ===

Rows Processed: 100
Unique Addresses: 85
Addresses Found:
  - alice@example.com
  - bob@example.org
  - charlie@test.com
  - diana@company.io
  - eve@domain.co.uk
  - frank.smith@mail.com
  - grace.lee@email.org
  ... (78 more addresses)

Hashed Credentials Detected: 12
Hashed Credential IDs:
  - dave_1990
  - oscar@example.com
  - trent
  - user.name
  ... (8 more)

Weak Passwords Found: 28
Weak Password IDs:
  - alice@example.com
  - bob
  - charlie@sub.example.org
  - dave_1990
  - eve+test@example.co.uk
  - mallory
  - oscar@example.com
  - peggy
  ... (20 more)

Breached Addresses: 0

Metadata Events:
  - Processed CSV file with 100 rows
  - Identified 85 unique addresses
  - Found 28 weak passwords
  - Detected 12 hashed credentials
  - No known breaches detected
```

### JSON Analysis

```bash
# Analyze JSON structured credential data
cargo run -- ingest tests/fixtures/json_credentials.json
```

The tool auto-detects JSON format and extracts:

- `email` / `username` fields
- `password` fields
- Any additional metadata

### YAML Analysis

```bash
# Analyze YAML formatted credentials
cargo run -- ingest tests/fixtures/yaml_credentials.yaml --format yaml
```

### TSV Analysis

```bash
# Tab-separated values (common in data exports)
cargo run -- ingest tests/fixtures/tab_separated.tsv --format tsv
```

## Multiple Files & Patterns

### Process All CSV Files in a Directory

```bash
# Use glob pattern to match all CSV files
cargo run -- ingest "tests/fixtures/*.csv"
```

This processes:

- `test_creds_100.csv`
- `test_creds_small.csv`
- `test_creds_mixed.csv`
- `well_formed_credentials.csv`
- And all other `.csv` files

### Process Multiple Formats

```bash
# Combine CSV and JSON files using brace expansion
cargo run -- ingest "tests/fixtures/*.{csv,json}"
```

### Process with Specific Pattern

```bash
# Only process "test_creds" files
cargo run -- ingest "tests/fixtures/test_creds*.csv"

# Process well-formed data for validation
cargo run -- ingest "tests/fixtures/well_formed*.csv"
```

### Recursive Directory Search

```bash
# Find all credential files recursively
cargo run -- ingest "data/**/*.csv"
```

## Output Formats

### JSON (Default)

Pretty-printed JSON for programmatic consumption:

```bash
cargo run -- ingest tests/fixtures/test_creds_100.csv --output-format json -o analysis.json
```

Best for: APIs, automation, downstream processing

### CSV Format

Metric/value pairs for spreadsheet import:

```bash
cargo run -- ingest tests/fixtures/test_creds_100.csv --output-format csv -o results.csv
```

Output file:

```csv
metric,value
rows_processed,100
unique_addresses,85
addresses_found,alice@example.com|bob@example.org|charlie@test.com|diana@company.io|eve@domain.co.uk|frank.smith@mail.com|grace.lee@email.org
hashed_credentials_detected,12
weak_passwords_found,28
weak_password_ids,alice@example.com|bob|charlie@sub.example.org|dave_1990|eve+test@example.co.uk|mallory|oscar@example.com|peggy
breached_addresses,0
```

### Human-Readable Text

Clean, formatted summary for reports:

```bash
cargo run -- ingest tests/fixtures/test_creds_100.csv --output-format text
```

Best for: Reports, manual review, logging

### JSONL (Newline-Delimited JSON)

Streaming format with individual events:

```bash
cargo run -- ingest tests/fixtures/test_creds_100.csv --output-format jsonl -o events.jsonl
```

Output:

```jsonl
{"event":"summary","rows_processed":100,"unique_addresses":85,"hashed_credentials_detected":12,"weak_passwords_found":28,"breached_addresses":0}
{"event":"address_found","type":"email","address":"alice@example.com","first_seen":"2024-12-13T10:30:00Z"}
{"event":"address_found","type":"username","address":"bob","first_seen":"2024-12-13T10:30:00Z"}
{"event":"address_found","type":"email","address":"charlie@test.com","first_seen":"2024-12-13T10:30:00Z"}
{"event":"weak_password","address":"alice@example.com","strength":"weak","pattern":"common_word"}
{"event":"weak_password","address":"bob","strength":"very_weak","pattern":"common_name"}
{"event":"hashed_credential","address":"oscar@example.com","hash_type":"bcrypt","identified":"2024-12-13T10:30:00Z"}
{"event":"metadata","message":"Processed CSV file with 100 rows"}
{"event":"metadata","message":"Identified 85 unique addresses"}
```

Best for: Streaming, real-time processing, large datasets

## Enrichment Features

### Enable Vector Embeddings (Ollama)

Improve duplicate detection with AI embeddings:

```bash
# Requires Ollama running on localhost:11434
cargo run -- ingest tests/fixtures/test_creds_100.csv --embeddings
```

With custom Ollama server:

```bash
cargo run -- ingest tests/fixtures/test_creds_100.csv \
  --embeddings \
  --ollama-url http://ollama.example.com:11434
```

This enables near-duplicate detection using vector similarity (default threshold: 0.85).

### Adjust Similarity Threshold

```bash
# More strict deduplication (require 95% similarity)
cargo run -- ingest tests/fixtures/duplicate_rows.csv \
  --embeddings \
  --similarity-threshold 0.95

# More lenient deduplication (accept 70% similarity)
cargo run -- ingest tests/fixtures/test_creds_100.csv \
  --embeddings \
  --similarity-threshold 0.70
```

### Enable HIBP Breach Lookups

Check addresses against Have I Been Pwned:

```bash
# Set HIBP API key via environment variable
export DUMPTRUCK_HIBP_KEY="your-api-key-here"

cargo run -- ingest tests/fixtures/test_creds_100.csv --hibp
```

Or pass key directly:

```bash
cargo run -- ingest tests/fixtures/test_creds_100.csv \
  --hibp \
  --hibp-key "your-api-key-here"
```

### Combine Embeddings + HIBP

Full enrichment pipeline:

```bash
cargo run -- ingest tests/fixtures/test_creds_100.csv \
  --embeddings \
  --ollama-url http://localhost:11434 \
  --hibp \
  --hibp-key $HIBP_KEY \
  --output-format jsonl \
  -o enriched_results.jsonl
```

## Storage Options

### Database Storage (Default)

Uses PostgreSQL via Docker Compose:

```bash
# Start services first
docker-compose up -d

# Analyze and store in database
cargo run -- ingest tests/fixtures/test_creds_100.csv
```

With custom PostgreSQL connection:

```bash
cargo run -- ingest tests/fixtures/test_creds_100.csv \
  --database "postgresql://user:password@db.example.com:5432/dumptruck"
```

### Filesystem Storage

Store results locally without database:

```bash
# Create local storage directory
mkdir -p ./analysis_results

# Analyze and store as files
cargo run -- ingest tests/fixtures/test_creds_100.csv \
  --filesystem \
  --storage-path ./analysis_results
```

Results organized by:

- Hashes
- Addresses
- Credentials
- Metadata

### Compare Storage Options

```bash
# Database (fast for large datasets)
time cargo run -- ingest tests/fixtures/*.csv

# Filesystem (for disconnected environments)
time cargo run -- ingest tests/fixtures/*.csv \
  --filesystem \
  --storage-path ./offline_storage
```

## Parallel Processing

### Automatic Parallelization

Process multiple files using all CPU cores:

```bash
# Automatically uses all available cores
cargo run -- ingest "tests/fixtures/test_creds*.csv" \
  -o bulk_results.json
```

### Custom Worker Count

Control parallelism:

```bash
# Use 4 worker threads
cargo run -- ingest "tests/fixtures/*.csv" --workers 4

# Use 8 workers for high-performance machine
cargo run -- ingest "data/*.csv" --workers 8

# Single-threaded (useful for debugging)
cargo run -- ingest tests/fixtures/test_creds_100.csv --workers 1
```

### Monitor Parallel Processing

Use verbose output to see processing:

```bash
# Verbose: see each file being processed
cargo run -- ingest "tests/fixtures/*.csv" -v

# Very verbose: detailed processing steps
cargo run -- ingest "tests/fixtures/*.csv" -vv

# Debug: all internal operations
cargo run -- ingest "tests/fixtures/*.csv" -vvv
```

## Real-World Workflows

### Security Team Analysis

Analyze leaked credentials and identify internal exposure:

```bash
# Setup
mkdir -p security_analysis
export HIBP_KEY="your-api-key"

# Analyze breach data with full enrichment
cargo run -- ingest breach_data.csv \
  --embeddings \
  --ollama-url http://internal-ollama:11434 \
  --hibp \
  --hibp-key $HIBP_KEY \
  -c config.json \
  -o security_analysis/results.json \
  --output-format json \
  -vv
```

Results include:

- Unique compromised addresses
- Weak password patterns
- Known breach matches
- Near-duplicate detection

### Batch Processing Large Datasets

Process multiple breach dumps efficiently:

```bash
# Process all breach files with parallelization
cargo run -- ingest "breaches/**/*.csv" \
  --workers 8 \
  --embeddings \
  --output-format jsonl \
  -o batch_results.jsonl
```

### Development Testing

Test with fixture files:

```bash
# Quick smoke test with small dataset
cargo run -- ingest tests/fixtures/test_creds_small.csv \
  --output-format text

# Test various formats
cargo run -- ingest "tests/fixtures/*.{csv,json,yaml}" \
  -c config.default.json \
  --output-format csv \
  -o format_test_results.csv
```

### Format Validation

Test data format handling:

```bash
# Valid data - success path
cargo run -- ingest tests/fixtures/well_formed_credentials.csv \
  -o valid_results.json

# Edge cases - various special characters
cargo run -- ingest tests/fixtures/special_characters.csv \
  -vv

# Unicode handling
cargo run -- ingest tests/fixtures/unicode_addresses.csv \
  -o unicode_results.json

# Malformed data - error handling
cargo run -- ingest tests/fixtures/malformed_missing_quote.csv \
  -vvv 2>&1 | head -20
```

### Deduplication Testing

Focus on duplicate detection:

```bash
# Test without embeddings (basic dedup)
cargo run -- ingest tests/fixtures/duplicate_rows.csv \
  -o dedup_basic.json

# Test with embeddings (advanced dedup)
cargo run -- ingest tests/fixtures/duplicate_rows.csv \
  --embeddings \
  --similarity-threshold 0.85 \
  -o dedup_advanced.json
```

Compare results to see embedding impact.

### Configuration Management

Use config file for repeated operations:

```bash
# Create config file
cat > analysis_config.json << 'EOF'
{
  "hibp_api_key": "your-key-here",
  "email_domain_substitutions": {
    "googlemail.com": "gmail.com",
    "yahoo.co.uk": "yahoo.com",
    "hotmail.co.uk": "hotmail.com"
  }
}
EOF

# Use config in analysis
cargo run -- ingest tests/fixtures/test_creds_100.csv \
  -c analysis_config.json \
  --hibp \
  --embeddings \
  -o configured_results.json
```

## Troubleshooting

### File Not Found

```bash
# Problem: glob pattern doesn't match anything
cargo run -- ingest "data/*.csv"
# Error: No files found matching pattern

# Solution: check the pattern and directory
ls tests/fixtures/*.csv
cargo run -- ingest "tests/fixtures/*.csv"
```

### Invalid Format

```bash
# Problem: auto-detection fails
cargo run -- ingest weird_data.txt

# Solution: specify format explicitly
cargo run -- ingest weird_data.txt --format csv
```

### Database Connection Failed

```bash
# Ensure PostgreSQL is running
docker-compose up -d

# Or use filesystem storage
cargo run -- ingest data.csv --filesystem --storage-path ./results
```

### Ollama Not Available

```bash
# Start Ollama
docker-compose -f docker/ollama/docker-compose.yml up -d

# Or disable embeddings
cargo run -- ingest data.csv
```

### Out of Memory

```bash
# Use fewer workers to reduce memory usage
cargo run -- ingest "data/*.csv" --workers 2

# Process files one at a time
for file in data/*.csv; do
  cargo run -- ingest "$file" -o "results/$(basename $file).json"
done
```

### Permission Denied

```bash
# Check input file permissions
ls -l tests/fixtures/test_creds_100.csv

# Check output directory permissions
mkdir -p results && chmod 755 results

# Run with appropriate permissions
cargo run -- ingest tests/fixtures/test_creds_100.csv -o results/output.json
```

---

**See also**:

- [CLI_USAGE.md](docs/CLI_USAGE.md) - Detailed command reference
- [CONFIGURATION.md](docs/CONFIGURATION.md) - Configuration options
- [HIBP.md](docs/HIBP.md) - Have I Been Pwned integration
- [OLLAMA.md](docs/OLLAMA.md) - Vector embeddings setup
