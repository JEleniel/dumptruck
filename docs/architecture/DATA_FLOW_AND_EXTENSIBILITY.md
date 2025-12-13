# Data Flow & Extensibility

Overview

- The pipeline processes records through discrete stages: Ingest -> Normalize -> Enrich -> Analyze -> Store/History.

Logical data flow

1. Ingest

- Detect format (CSV, JSON, Protobuf, etc.) and stream records.

2. Normalize

- Apply canonicalization rules described in `docs/design/FEATURE_CARDS/normalization.md`.

3. Enrich

- Apply enrichment rules: domain parsing, tokenization, derived attributes; refer to `docs/design/FEATURE_CARDS/enrichment.md`.

4. Analyze

- Deduplicate, correlate with historic hashes, flag novel entries. See `docs/design/FEATURE_CARDS/analysis.md`.

5. Store/History

- Persist artifacts and history in hashed form (see `docs/design/FEATURE_CARDS/history_privacy.md` and `storage.md`).

Extensibility points

- Format adapters: add new parser that returns the internal record representation.
- Normalization rules: composable rule sets loaded from config or code.
- Enrichment plugins: rule-based plugins or WASM modules to run language-agnostic code safely.
- Storage adapters: implement the storage trait for any backend.

Plugin model recommendations

- Prefer Rust traits for server-side plugins with a lightweight registration mechanism.
- For third-party plugins, use a sandbox (WASM or isolated worker) and a clearly versioned API.

Mapping to feature cards

- `docs/design/FEATURE_CARDS/ingestion.md` — ingestion requirements and input formats
- `docs/design/FEATURE_CARDS/normalization.md` — canonicalization and rules
- `docs/design/FEATURE_CARDS/enrichment.md` — enrichment strategies
- `docs/design/FEATURE_CARDS/analysis.md` — core analysis capabilities
- `docs/design/FEATURE_CARDS/history_privacy.md` — privacy and history policies
- `docs/design/FEATURE_CARDS/extensibility.md` — extension points and plugin guidance
- `docs/design/FEATURE_CARDS/storage.md` — storage considerations
- `docs/design/FEATURE_CARDS/server_modes.md` — runtime modes and API behavior
- `docs/design/FEATURE_CARDS/security.md` — security considerations

Developer guidance

- Keep adapters small and testable. Add unit tests for parser edge cases and normalization rules.
- Prefer stream-oriented processing to reduce memory use for large dumps.
