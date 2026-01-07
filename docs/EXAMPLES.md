# Dumptruck Usage Examples

Practical examples for running Dumptruck with the stable CLI command surface: `analyze`, `status`, `export`, `import`, and `serve`.

Prefer `dumptruck --help` and `dumptruck <command> --help` for the authoritative set of flags supported by your build.

## Run Dumptruck

```bash
# If you have an installed binary
dumptruck --help
```

```bash
# From the repository (development)
# Note: global flags go before the subcommand
cargo run -- --help
```

## Analyze

```bash
# Analyze a single file
dumptruck analyze ./breach.csv
```

```bash
# Analyze a directory recursively
dumptruck analyze ./breaches --recursive
```

```bash
# Add identification metadata and write JSON output
dumptruck analyze ./breach.csv --date 20260107 --target ExampleCorp --output ./report.json
```

```bash
# Development form of the same command
cargo run -- analyze ./breach.csv --date 20260107 --target ExampleCorp --output ./report.json
```

## Database snapshots

```bash
# Export the current SQLite database to a snapshot file
dumptruck export --output-path ./dumptruck-snapshot.sqlite
```

```bash
# Merge a snapshot back into the main database
dumptruck import --input-path ./dumptruck-snapshot.sqlite
```

## Serve

```bash
# Start the HTTPS server with defaults
dumptruck serve
```

```bash
# Bind on a non-default port with explicit TLS paths
dumptruck serve --port 8443 --cert /etc/tls/tls.crt --key /etc/tls/tls.key
```

## Status

```bash
# Query a running server instance
dumptruck status --url https://localhost:443
```

## Troubleshooting

- If `serve` fails TLS startup, verify the certificate/key paths and permissions.
- If `import` fails, ensure you are not importing from the same database path used by the main database.
