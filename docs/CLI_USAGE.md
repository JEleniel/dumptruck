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

## Error Handling

- **File not found**: Ensure glob pattern matches files or path exists
- **Invalid pattern**: Check glob syntax (*, ?, [], {})
- **Connection errors**: Verify database/Ollama/HIBP services running
- **Permission errors**: Ensure read permissions on input files, write on output directory
- **Format mismatch**: Verify `--format` matches actual file format

## Performance Tuning

- **Parallel workers**: Set `--workers` to match CPU count for CPU-bound workloads
- **Batch size**: Adjust based on memory available for large datasets
- **Vector indexing**: Enable embeddings `--embeddings` for faster duplicate detection on large datasets
- **HIBP lookups**: Cache results to avoid API rate limiting
