# Data Flow & Extensibility

Overview

- The pipeline processes records through discrete stages: Analyze -> Normalize -> Enrich -> Detect/Score -> Persist.

Logical data flow

1. Analyze

- Detect format (CSV, JSON, Protobuf, etc.) and stream records through safe input validation.

1. Normalize

- Apply canonicalization rules described in [docs/design/Capabilities.md](../Capabilities.md) and [docs/design/PIPELINE_MAP.md](../PIPELINE_MAP.md).

1. Enrich

- Apply enrichment rules (derived attributes, correlations) as described in [docs/design/PIPELINE_MAP.md](../PIPELINE_MAP.md).

1. Analyze

- Deduplicate, correlate with historic hashed indicators, and flag novel/anomalous entries.

1. Store/History

- Persist artifacts and history in hashed form inside SQLite (see the AURORA storage requirements below).

Extensibility points

- Format adapters: add new parser that returns the internal record representation.
- Normalization rules: composable rule sets loaded from config or code.
- Enrichment plugins: rule-based plugins or WASM modules to run language-agnostic code safely.
- Storage adapters: implement the storage trait for any backend.

Plugin model recommendations

- Prefer Rust traits for server-side plugins with a lightweight registration mechanism.
- For third-party plugins, use a sandbox (WASM or isolated worker) and a clearly versioned API.

Mapping to feature cards

This repository now uses AURORA cards as the source of truth:

- [Stable CLI command surface](../cards/requirement-command-surface.md)
- [Stage-based processing pipeline](../cards/requirement-stage-based-processing-pipeline.md)
- [SQLite is the primary persistent store](../cards/requirement-sqlite-primary-storage.md)
- [Export/import via SQLite database snapshots](../cards/requirement-export-import-sqlite-snapshots.md)
- [Support CLI and HTTPS Server modes](../cards/requirement-dual-mode-cli-server.md)
- [No JSON persistence](../cards/constraint-no-json-persistence.md)

Developer guidance

- Keep adapters small and testable. Add unit tests for parser edge cases and normalization rules.
- Prefer stream-oriented processing to reduce memory use for large dumps.
