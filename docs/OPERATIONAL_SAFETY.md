# Operational Safety: File Handling & Database Concurrency

This document focuses on operational safety for:

1. **Large file handling** (disk, memory, and working copies)
2. **Concurrent database access** (single instance vs multi-instance deployments)

Dumptruck's primary persistence model is **SQLite-only**. Multi-instance deployments should be designed around **per-instance databases** and **snapshot export/import** rather than sharing one database file between multiple processes.

## File Handling

### Supported input formats

The stable `analyze` workflow currently supports delimited text inputs:

- CSV
- TSV
- PSV

### Working copies

Dumptruck creates a working copy of the input under a temporary directory. For TSV/PSV inputs, the working copy is converted to CSV while reading records from the source file (row-by-row) to avoid loading the entire file into memory.

### Operational risks

- **Disk pressure**: Large files will require additional disk space for working copies.
- **Temp-path hygiene**: Prefer running with a dedicated `--temp-path` on an encrypted filesystem and with restrictive permissions.
- **Binary / malformed data**: Treat unknown or mixed binary/text datasets as hostile. Validate inputs and isolate processing environments.

### Recommendations

- Run analysis in a dedicated user account with minimal permissions.
- Store both the database directory and the temp path on an encrypted volume.
- Monitor free disk space before and during large runs.

## Database Concurrency (SQLite)

### Within a single instance

Within a single Dumptruck process, SQLite access is managed by a connection pool. Concurrent tasks in the same process can safely read/write through SQLite's locking model.

### Multiple instances

Do not run multiple Dumptruck instances against the same SQLite database directory unless you have a deliberate, tested coordination strategy.

Reasons:

- SQLite supports multiple readers and a single writer, but write contention and long-running transactions can cause timeouts and operational fragility.
- Network filesystems (NFS, SMB, some container volume setups) can break file-locking expectations.

### Safe multi-instance pattern

For multi-instance deployments:

- Use one SQLite database per instance.
- Periodically export snapshots and merge them via `import`.
- Treat snapshot imports as controlled operations (ideally during maintenance windows) to avoid competing writers.

## Testing Recommendations

### Large file smoke test

Use a syntactically valid CSV for size tests.

```bash
printf "email,password\n" > test.csv
for i in $(seq 1 200000); do printf "user%u@example.com,password%u\n" "$i" "$i"; done >> test.csv

dumptruck analyze ./test.csv
```

### Multi-instance smoke test (snapshot merge)

```bash
# Instance A analyzes data into its own database directory
dumptruck --database ./db-a analyze ./file-a.csv
dumptruck --database ./db-a export --output-path ./snapshot-a.sqlite

# Instance B analyzes data into its own database directory
dumptruck --database ./db-b analyze ./file-b.csv
dumptruck --database ./db-b export --output-path ./snapshot-b.sqlite

# Merge B into A
dumptruck --database ./db-a import --input-path ./snapshot-b.sqlite
```
