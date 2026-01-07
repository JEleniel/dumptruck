# Database modules (`src/database`) — overview (Jan 7, 2026)

This document summarizes the SQLite database layer implemented under `src/database` and the coordinator in `src/database.rs`.

## Summary

+ `src/database.rs` — `Database` coordinator (open/export/import) that owns an `r2d2` SQLite pool and composes the per-table modules
+ `src/database/analyzedfiles.rs` — tracks which files have already been analyzed
+ `src/database/credentials.rs` — credentials table and helpers
+ `src/database/exportargs.rs` — clap `ExportArgs` for the `export` command (`--output-path`)
+ `src/database/identities.rs` — identities table and helpers
+ `src/database/importargs.rs` — clap `ImportArgs` for the `import` command (`--input-path`)
+ `src/database/migrate.rs` — migration coordinator that applies each module's `MigrationTrait` implementation
+ `src/database/migrationtrait.rs` — `MigrationTrait` interface (`create`/`upgrade`/`downgrade`)
+ `src/database/metadata.rs` — metadata table helpers (e.g., migration version, database UUID)
+ `src/database/npi.rs` — storage for detected NPI/PII-derived indicators
+ `src/database/rainbowtable.rs` — storage for precomputed weak-password/rainbow indicators

## Public API highlights

+ `Database::open(path)` — accepts a directory or file path; if a directory, uses `dumptruck.db`; ensures schema/migrations are applied
+ `Database::export(arg)` — copies the SQLite database file to `arg.output_path`, then assigns the exported copy a new UUID
+ `Database::import(arg)` — opens `arg.input_path` and merges its contents into the main database

## Coding patterns used (observed)

+ Connection acquisition: most operations call `pool.get()?` to acquire a `rusqlite` connection
+ Migrations: each table module implements `MigrationTrait`; schema changes use `conn.execute_batch(...)`; the coordinator calls modules in a fixed order
+ Idempotency and deduplication: schema uses `CREATE TABLE IF NOT EXISTS ...`; inserts commonly use `INSERT OR IGNORE`
+ Bulk writes: use explicit transactions (`conn.transaction()?`) and batched `prepare` + `execute` loops

## Notes

+ The legacy “seed database” terminology is deprecated in design docs; operationally, `export`/`import` implement SQLite snapshot workflows
