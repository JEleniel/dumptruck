# CLI Usage Guide

## Overview

Dumptruck provides a command-line interface for ingesting and analyzing bulk data dumps with support for glob patterns, parallel processing, and flexible output formats.

## Basic Usage

### Single File

```bash
dumptruck ingest data.csv
```

### Multiple Files with Glob Patterns

```bash
# All CSV files in current directory
dumptruck ingest "*.csv"

# All data files in a specific directory
dumptruck ingest "tests/fixtures/*.csv"

# Multiple formats with brace expansion (shell-dependent)
dumptruck ingest "tests/fixtures/*.{csv,json}"

# Recursive search (requires ** support)
dumptruck ingest "data/**/*.csv"
```

### Glob Pattern Support

The CLI supports standard glob patterns:

- `*` - matches any sequence of characters (except `/`)
- `?` - matches any single character
- `[abc]` - matches any character in brackets
- `**` - matches zero or more directories (depends on shell)

## Storage Options

### Database Storage (Default)

```bash
# Uses default Docker Compose PostgreSQL connection
dumptruck ingest data.csv

# With custom connection string
dumptruck ingest data.csv --database "postgres://user:pass@host:5432/db"
```

### Filesystem Storage

```bash
dumptruck ingest data.csv --filesystem --storage-path ./results
```

## Enrichment Options

### Ollama Embeddings

Enable vector embeddings for near-duplicate detection:

```bash
dumptruck ingest data.csv --embeddings

# With custom Ollama server
dumptruck ingest data.csv --embeddings --ollama-url http://localhost:11434
```

### HIBP Breach Lookup

Enable Have I Been Pwned breach data enrichment:

```bash
dumptruck ingest data.csv --hibp --hibp-key YOUR_API_KEY
```

Combining both:

```bash
dumptruck ingest data.csv --embeddings --hibp --hibp-key YOUR_API_KEY
```

## Format Options

### Input Format

Auto-detected by default, or specify explicitly:

```bash
# Explicit format
dumptruck ingest data.csv --format csv
dumptruck ingest data.tsv --format tsv
dumptruck ingest data.json --format json
dumptruck ingest data.yaml --format yaml

# Protobuf (binary)
dumptruck ingest data.pb --format protobuf
```

### Output Format

```bash
# JSON (default)
dumptruck ingest data.csv --output-format json

# CSV for spreadsheets
dumptruck ingest data.csv --output-format csv

# Human-readable text
dumptruck ingest data.csv --output-format text

# Newline-delimited JSON (for streaming)
dumptruck ingest data.csv --output-format jsonl

# Save to file
dumptruck ingest data.csv -o results.json
```

## Parallel Processing

### Automatic Parallelization

Process multiple files in parallel using all available CPU cores:

```bash
dumptruck ingest "tests/fixtures/*.csv" --output results.json
```

### Custom Worker Count

```bash
# Use 4 worker threads
dumptruck ingest "tests/fixtures/*.csv" --workers 4
```

## Configuration

### Config File

Specify a JSON configuration file for API keys and domain mappings:

```bash
dumptruck ingest data.csv -c config.json
```

Example `config.json`:

```json
{
	"hibp_api_key": "YOUR_KEY_HERE",
	"email_domain_substitutions": {
		"googlemail.com": "gmail.com",
		"yahoo.co.uk": "yahoo.com"
	}
}
```

## Verbosity

Control output verbosity:

```bash
# Quiet (errors only)
dumptruck ingest data.csv

# Verbose
dumptruck ingest data.csv -v

# Very verbose
dumptruck ingest data.csv -vv

# Debug
dumptruck ingest data.csv -vvv
```

## Status Check

Verify system connectivity:

```bash
# Check all services
dumptruck status --check-database --check-ollama --check-hibp

# With custom URLs
dumptruck status --check-ollama --ollama-url http://localhost:11434
dumptruck status --check-hibp --hibp-key YOUR_KEY
```

## Examples

### Batch Process All Breach Data

```bash
# Process all CSV files in parallel with full enrichment
dumptruck ingest "data/*.csv" \
	--embeddings \
	--hibp \
	--hibp-key $HIBP_KEY \
	-c config.json \
	-o results/output.jsonl \
	--output-format jsonl \
	--workers 8
```

### Analyze Single File with Filesystem Storage

```bash
dumptruck ingest sample.csv \
	--filesystem \
	--storage-path ./local-results \
	--format csv \
	-vv
```

### Development Workflow

```bash
# Test with fixture files
dumptruck ingest "tests/fixtures/well_formed*.csv" \
	-c config.default.json \
	--output-format text \
	-v
```

## Seed Command

The `seed` command creates a seed database from a folder of data files with automatic startup import verification.

### Overview

A seed database enables:

- **Bulk initialization**: Load predefined datasets into Dumptruck
- **Distributed deployments**: Create seed once, deploy to multiple instances
- **Change detection**: Automatic import if seed data files are modified
- **Disaster recovery**: Backup and restore seed data efficiently

### Basic Seed Creation

```bash
# Create seed database from folder
dumptruck seed /path/to/data/files

# Output: data/seed.db with signature for change detection
```

### Seed with Custom Output Path

```bash
# Specify custom seed database location
dumptruck seed /data/breaches --output /backup/seed.db

# Verify creation
sqlite3 /backup/seed.db "SELECT COUNT(*) FROM canonical_addresses;"
```

### Seed with Services Enabled

```bash
# Include HIBP enrichment in seed
dumptruck seed /data/files \
	--enrichment \
	--hibp-key your_api_key

# Include vector embeddings (requires Ollama)
dumptruck seed /data/files \
	--embeddings \
	--ollama-url http://localhost:11435

# Both enrichments with custom workers
dumptruck seed /data/files \
	--enrichment \
	--embeddings \
	--workers 8 \
	--hibp-key your_api_key
```

### Seed with Custom Database

```bash
# Use custom database path for seed storage
dumptruck seed /data/files \
	--database sqlite:///var/lib/dumptruck/seed.db \
	-c /etc/dumptruck/config.json
```

### Seed with Verbose Output

```bash
# Show detailed progress during seed creation
dumptruck seed /data/files -vv

# Output:
# [INFO] Creating seed database from folder: "/data/files"
# [INFO] Discovered 4 data files
# [INFO] Computed file signature: a1b2c3d4...
# [INFO] Seed database ready at: "data/seed.db"
# [INFO] Files: 4, Estimated import time: 2 minute(s)
```

### Seed File Structure

The seed folder can contain:

```text
seed_folder/
├── breaches.csv              # Any format: CSV, JSON, TSV, XML, YAML
├── weak_passwords.json
├── international_data/
│   ├── data_set_2.csv
│   └── deep_folder/
│       └── archive.xml
└── .gitignore                # Non-data files ignored
```

**Supported formats**: CSV, TSV, JSON, XML, YAML, JSONL

### Seed Output

```json
{
  "status": "created",
  "seed_db_path": "data/seed.db",
  "folder_path": "/path/to/data/files",
  "files_discovered": 4,
  "rows_processed": 1250,
  "unique_addresses": 450,
  "file_signature": "a1b2c3d4e5f6...",
  "created_at": 1703520000,
  "estimated_import_time_minutes": 2,
  "details": "Seed database created. On next startup, signature will be verified and data imported if changed."
}
```

### Startup Verification

Once a seed database exists, Dumptruck automatically verifies it on startup:

1. **Server starts**: Loads seed.db metadata (path, signature, statistics)
2. **Signature check**: Recomputes SHA-256 of all files in seed folder
3. **Change detection**:
   + If signature matches → Cached, no import (fast startup)
   + If signature differs → New/modified data detected
4. **Auto-import**: All seed data imported into main database with deduplication
5. **Logging**: `[INFO] Seed data imported from /path, 450 addresses merged`

### Seed Signature Details

- **Deterministic**: Same files always produce same signature
- **Sensitive**: Any file modification triggers re-import
- **Efficient**: Computed with 4KB streaming buffers
- **Format**: SHA-256 hex-encoded string

### Use Cases

#### Pre-loaded Breach Database

```bash
# Create seed from known breaches
dumptruck seed /archive/breaches-2024 \
	--enrichment \
	--hibp-key $HIBP_KEY

# Every instance starts with same baseline
dumptruck server
# [INFO] Seed data imported from /archive/breaches-2024, 50000 addresses merged
```

#### Disaster Recovery

```bash
# Backup seed database
cp data/seed.db /backup/seed-$(date +%Y%m%d).db

# Restore on new system
cp /backup/seed-20241225.db /path/to/new/system/data/seed.db
./dumptruck server
# Seed automatically re-imported on startup
```

#### Development Testing

```bash
# Create seed from test fixtures
dumptruck seed tests/fixtures \
	--output test_seed.db \
	-c config.default.json

# Test in isolation
dumptruck ingest test_data.csv --database test_seed.db
```

#### Multi-Instance Deployment

```bash
# Create seed once (takes 5 minutes)
dumptruck seed /shared/canonical-data --output /shared/seed.db

# Deploy same seed to 100 instances (1 second per instance)
for i in {1..100}; do
	cp /shared/seed.db /instances/instance$i/data/seed.db
	/instances/instance$i/dumptruck server &
done
```

## Error Handling

- **File not found**: Ensure glob pattern matches files or path exists
- **Invalid pattern**: Check glob syntax (*, ?, [], {})
- **Connection errors**: Verify database/Ollama/HIBP services running
- **Permission errors**: Ensure read permissions on input files, write on output directory
- **Format mismatch**: Verify `--format` matches actual file format
- **Seed folder not found**: Check that seed folder path is accessible and contains data files
- **No data files**: Ensure folder contains CSV/JSON/XML/TSV/YAML files (non-data files ignored)

## Performance Tuning

- **Parallel workers**: Set `--workers` to match CPU count for CPU-bound workloads
- **Batch size**: Adjust based on memory available for large datasets
- **Vector indexing**: Enable embeddings `--embeddings` for faster duplicate detection on large datasets
- **HIBP lookups**: Cache results to avoid API rate limiting
