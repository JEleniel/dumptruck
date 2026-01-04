# Data modules (src/data) — overview (Dec 30, 2025)

This document summarizes the contents and public API of the `src/data` folder and the `src/data/database` submodules, the coding patterns used, and notable observations discovered while scanning the codebase.

## Summary

- Top-level files:
    + `exportargs.rs` — CLI `ExportArgs` (clap Parser) containing an `output: PathBuf` field used by the export command.
    + `importargs.rs` — CLI `ImportArgs` (clap Parser) containing an `input: PathBuf` field used by the import command.
    + `database.rs` — `Database` coordinator struct that composes the DB submodules and exposes `create`, `connect`, `validate`, `export`, and `import` methods.

- Database submodules (in `src/data/database`):
    + `migrationtrait.rs` — `MigrationTrait` (create/upgrade/downgrade signatures).
    + `signedconnection.rs` — `SignedConnection` wrapper around `rusqlite::Connection`; finalizes DB signature (SHA-256) on `Drop` and implements `Deref`/`DerefMut`.
    + `metadata.rs` — `Metadata` struct, `MIGRATION_VERSION` const, and helpers to get/set DB UUID, migration version, and DB hash.
    + `credentials.rs` — `Credentials` struct with `new`, `is_known`, `add`, `seen`, `get_all`, `write_all`.
    + `dumps.rs` — `Dumps` struct tracking ingested dump files; methods include `is_known`, `add`, `seen`, `clear`, `get_all`, `write_all`.
    + `identities.rs` — `Identities` struct with `is_known`, `add`, `seen`, `get_all`, `write_all`.
    + `pii.rs` — `Pii` struct for storing PII types and hashes with `add_pii`, `is_known`, and write/get helpers.
    + `rainbowtable.rs` — `RainbowTable` (hash_type, hash) storage with `is_known`, `add`, `get_all`.
    + `seedfiles.rs` — `SeedFiles` with `is_known`, `add`, `get_all`, and `write_all_to_file`.
    + `migrate.rs` — high-level `create`, `upgrade`, and `downgrade` functions that call each submodule's migration implementation.

## Public API highlights

- `ExportArgs` (clap):

```rust
#[derive(Parser, Debug)]
pub struct ExportArgs {
    #[arg(short, long, value_name = "OUTPUT_FILE")]
    pub output: PathBuf,
}
```

- `ImportArgs` (clap):

```rust
#[derive(Parser, Debug)]
pub struct ImportArgs {
    #[arg(short, long, value_name = "INPUT_FILE")]
    pub input: PathBuf,
}
```

- `Database` (coordinator) — important methods:
    + `pub async fn create(path: &PathBuf) -> Result<Self, DatabaseError>`
    + `pub async fn connect(path: &PathBuf, seed_path: &Option<PathBuf>) -> Result<Self, DatabaseError>`
    + `pub async fn validate(&self) -> Result<bool, DatabaseError>`
    + `pub async fn export(&self, arg: &ExportArgs) -> Result<(), DatabaseError>`
    + `pub fn import(&self, arg: &ImportArgs) -> Result<(), DatabaseError>`

Note: `Database::export` and `import` currently reference `arg.output_path` / `arg.input_path` in the implementation; the corresponding `ExportArgs`/`ImportArgs` structs define `output` / `input` fields. This naming mismatch is documented in the Observations section below.

## Coding patterns used (observed)

- Migration pattern: each DB submodule implements `MigrationTrait` and provides `create`, `upgrade`, and `downgrade` functions that execute SQL schema changes via `conn.execute_batch("CREATE TABLE IF NOT EXISTS ...")`.
- Shared connection pattern: modules accept an `Arc<Mutex<SignedConnection>>` (using `std::sync::Mutex`) and call `self.conn.lock().await` before preparing statements.
- Query pattern: `get_all` uses `prepare` + `query_map(... )?` and collects rows into a Rust `Vec`.
- Bulk write pattern: `write_all` opens a transaction (`tx = self.conn.lock().await.transaction()?`), prepares an `INSERT OR IGNORE ...` statement and iterates items, then `tx.commit()?`.
- Idempotency: `INSERT OR IGNORE` used consistently for deduplication.
- Indexing: indices created where appropriate (e.g., `idx_pii_type`, `idx_pii_hash`).
- Signature integrity: `SignedConnection::finalize_signature()` computes SHA-256 of DB file and stores it in `metadata.hash`; `Drop` on `SignedConnection` triggers finalization.
- Error handling: `DatabaseError` in `database.rs` uses `thiserror::Error` and `#[from]` conversions for `rusqlite::Error` and `std::io::Error`.
- Logging: `tracing::info` used for lifecycle events (create/connect/seed/export/import).

## Observations & recommendations

- Naming mismatch (potential compile error): `ExportArgs` exposes `output` but `Database::export` references `arg.output_path`. Similarly, `ImportArgs` exposes `input` but `Database::import` references `arg.input_path`.
    + Recommendation: align field names (change structs to `pub output_path: PathBuf` / `pub input_path: PathBuf`) or change `database.rs` to use `arg.output` / `arg.input`.

- The `SignedConnection` finalization mechanism updates `metadata.hash` on `Drop`. This is deliberate; ensure callers are aware that closing a connection will cause hash updates.

- Keep using the established patterns: `MigrationTrait`, `Arc<Mutex<SignedConnection>>`, `write_all` transaction pattern, and `rusqlite::params![]`. New modules should conform to these conventions.

## Examples

- Export database (CLI):

```
dumptruck export-db -o /path/to/export.sqlite
```

- Import database (CLI):

```
dumptruck import-db -i /path/to/export.sqlite
```

## Files added/changed during this scan

- Created `docs/data/MODULES.md` (this file) to document `src/data`.

---

End of data-module summary.
