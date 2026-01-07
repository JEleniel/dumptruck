# Interfaces

## External interfaces

### CLI

Dumptruck exposes the following user-facing commands:

- analyze
- status
- export
- import
- serve

### HTTP API (server mode)

Dumptruck server mode currently exposes endpoints that submit analysis jobs and query status:

- POST /api/v1/ingest
- POST /api/v1/ingest/upload
- GET /api/v1/status/{job_id}
- GET /api/v1/jobs
- DELETE /api/v1/jobs/{job_id}

Note: The API uses "ingest" in the path for backwards compatibility; semantically it submits work for analysis.

## Internal trait interfaces

This document defines recommended Rust-style interfaces (traits) and example signatures for implementers. These are design contracts — concrete code should follow project conventions and be kept small and testable.

Format adapter (parsers)

- Purpose: read an input source and produce a stream of raw records in the project's internal representation.

Example trait (conceptual)

```rust
pub trait FormatAdapter {
    type Error;
    type Record; // the internal record struct or enum

    /// Initialize adapter from config or reader
    fn new(config: &Config) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Read next record; returns None at EOF
    fn next_record(&mut self) -> Result<Option<Self::Record>, Self::Error>;
}
```

Normalizer

- Purpose: transform raw records into a canonical normalized record.

Example

```rust
pub trait Normalizer {
    type Error;
    type Raw;
    type Normalized;

    fn normalize(&self, raw: Self::Raw) -> Result<Self::Normalized, Self::Error>;
}
```

Enricher

- Purpose: add derived fields or tags to a normalized record.

Example

```rust
pub trait Enricher {
    type Error;
    type Normalized;

    fn enrich(&self, record: Self::Normalized) -> Result<Self::Normalized, Self::Error>;
}
```

Analyzer

- Purpose: run statistical or pattern detection on enriched records (may be stateful).

Storage adapter

- Purpose: persist artifacts, exports, and hashed history entries.

Example

```rust
pub trait StorageAdapter {
    type Error;

    fn write_artifact(&self, path: &str, payload: &[u8]) -> Result<(), Self::Error>;
    fn append_history_hash(&self, key: &str, hash: &str) -> Result<(), Self::Error>;
}
```

Notes

- Use `Result<_, Error>` for clear error propagation.
- Prefer small, single-responsibility traits; compose where necessary.
- For server-side dynamic plugins, prefer a versioned registration API and/or WASM for safe third-party code.
