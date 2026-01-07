# CLI Usage Guide

## Overview

Dumptruck analyzes bulk data files and stores results in an SQLite database.

The stable CLI command surface is:

- `analyze` — analyze a file or folder
- `status` — query a running server instance
- `export` — export the SQLite database to a snapshot file
- `import` — merge a snapshot database into the main database
- `serve` — run the HTTPS server

For the exact flags supported by your build, prefer `dumptruck --help` and `dumptruck <command> --help`.

## Global options

These flags are available before the subcommand:

- `-c, --config <CONFIGURATION>` — override config search path
- `-a, --api-keys <SERVICE=KEY>` — supply API keys (repeatable)
- `-d, --database <PATH>` — database file or directory (if a directory, Dumptruck uses `dumptruck.db`)
- `--temp-path <PATH>` — working directory for isolated processing
- `--embeddings` — enable embeddings support
- `--ollama-url <URL>` — use an external Ollama server
- `--vector-threshold <FLOAT>` — near-duplicate similarity threshold

## Analyze

Analyze a single file or a directory of supported files.

```bash
dumptruck analyze ./breach.csv
```

### Common flags

- `-r, --recursive` — recurse into subdirectories when input is a directory
- `-d, --date <YYYYMMDD>` — breach date used for identification
- `-t, --target <NAME>` — target entity name used for identification
- `-o, --output <PATH>` — write results as JSON to a file (otherwise prints a human-readable summary)
- `--enable-embeddings` — enable embeddings for this analyze run

### Examples

```bash
# Analyze a directory recursively
dumptruck analyze ./breaches --recursive
```

```bash
# Provide target metadata and write JSON output
dumptruck analyze ./breach.csv --date 20260107 --target ExampleCorp --output ./report.json
```

## Export

Export the current SQLite database to a snapshot file.

```bash
dumptruck export --output-path ./dumptruck-snapshot.sqlite
```

## Import

Merge a snapshot SQLite database into the main database.

```bash
dumptruck import --input-path ./dumptruck-snapshot.sqlite
```

## Serve

Run the HTTPS server (HTTP/2) with TLS configuration.

```bash
dumptruck serve
```

### Common flags

- `-b, --bind-addresses <ADDRESSES>` — bind addresses (comma-separated)
- `-p, --port <PORT>` — port (defaults to 443)
- `--cert <PATH>` — TLS certificate (PEM)
- `--key <PATH>` — TLS private key (PEM)
- `--oauth-id <ID>` / `--oauth-secret <SECRET>` / `--oauth-discovery-url <URL>` — OAuth configuration

## Status

Query a running server instance.

```bash
dumptruck status --url https://localhost:443
```

## Troubleshooting

- If you get file-not-found errors, check the input path and permissions.
- If `import` fails, ensure the snapshot file is not the same path as the main database.
- If `serve` fails TLS startup, verify the `--cert` and `--key` paths are readable by the process.
