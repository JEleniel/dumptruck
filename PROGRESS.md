# Implementation Status

## Project Brief

Dumptruck exists to securely analyze bulk data dumps (breach/leak datasets) in a privacy-preserving way, producing actionable reports and building reusable intelligence (deduplication, enrichment, and hashed history) while supporting both standalone CLI and HTTPS server operation.

### Features

- **AURORA: Root driver card**
    + Status: Completed
    + [AURORA Card](docs/design/AURORA/cards/driver-dumptruck-root.json)
    + [AURORA Card (Human)](docs/design/cards/driver-dumptruck-root.md)

- **AURORA: Stakeholder actor cards**
    + Status: Completed
    + [Actor: Analyst](docs/design/AURORA/cards/actor-analyst.json)
    + [Actor: Operator](docs/design/AURORA/cards/actor-operator.json)
    + [Actor: Maintainers](docs/design/AURORA/cards/actor-maintainers.json)

- **AURORA: CLI command surface**
    + Status: Completed
    + [AURORA Feature Card](docs/design/AURORA/cards/requirement-command-surface.json)
    + [AURORA Feature Card (Human)](docs/design/cards/requirement-command-surface.md)

- **AURORA: Stage-based processing pipeline**
    + Status: Completed
    + [AURORA Feature Card](docs/design/AURORA/cards/requirement-stage-based-processing-pipeline.json)
    + [AURORA Feature Card (Human)](docs/design/cards/requirement-stage-based-processing-pipeline.md)

- **AURORA: SQLite primary storage**
    + Status: Completed
    + [AURORA Feature Card](docs/design/AURORA/cards/requirement-sqlite-primary-storage.json)
    + [AURORA Feature Card (Human)](docs/design/cards/requirement-sqlite-primary-storage.md)

- **AURORA: Export/import SQLite snapshots**
    + Status: Completed
    + [AURORA Feature Card](docs/design/AURORA/cards/requirement-export-import-sqlite-snapshots.json)
    + [AURORA Feature Card (Human)](docs/design/cards/requirement-export-import-sqlite-snapshots.md)

- **AURORA: No JSON persistence constraint**
    + Status: Completed
    + [AURORA Constraint Card](docs/design/AURORA/cards/constraint-no-json-persistence.json)
    + [AURORA Constraint Card (Human)](docs/design/cards/constraint-no-json-persistence.md)

- **AURORA: Dual-mode CLI and HTTPS server**
    + Status: Completed
    + [AURORA Feature Card](docs/design/AURORA/cards/requirement-dual-mode-cli-server.json)
    + [AURORA Feature Card (Human)](docs/design/cards/requirement-dual-mode-cli-server.md)

- **Design: Architecture overview docs**
    + Status: Completed
    + [Architecture index](docs/design/architecture/ARCHITECTURE.md)
    + [Interfaces](docs/design/architecture/INTERFACES.md)
    + [Deployment](docs/design/architecture/DEPLOYMENT.md)
    + [Security](docs/design/architecture/SECURITY.md)

- **Threat modeling: OWASP threat card library**
    + Status: Completed
    + [Threat card index](docs/design/threat/README.md)

## Active Context Summary

- docs/design is being normalized to be AURORA-first: canonical JSON cards and links under docs/design/AURORA, with Markdown “view” files under docs/design/cards and docs/design/links.
- Persistence is SQLite-only by design; export/import is via SQLite snapshots.
- User-facing terminology is “analyze”; the HTTP API keeps /api/v1/ingest paths for backwards compatibility only.
- The legacy “seed” concept is removed from design documentation and replaced by export/import semantics.
- AURORA model validity is now machine-validated (standalone validator crate).
- CLI user docs are being brought into alignment with the stable command surface (analyze/status/export/import/serve) and SQLite-only persistence.

## Patterns

- AURORA architecture model:
    + Canonical source of truth: [docs/design/AURORA/cards](docs/design/AURORA/cards) and [docs/design/AURORA/links](docs/design/AURORA/links).
    + All links point toward the root driver card, forming a DAG.
    + Markdown copies link to canonical JSON and do not embed JSON.

- Pipeline decomposition:
    + Stage-based processing with explicit safety steps (safe read/validation, chain of custody, secure deletion) and reporting.

- Privacy-by-design:
    + Prefer irreversible hashing for historic indicators; avoid retaining raw sensitive values when feasible.

## Technologies

- Language: Rust (edition 2024)
- Storage: SQLite (rusqlite + r2d2_sqlite)
- Server: axum 0.8, hyper 1.x, tokio 1.x
- CLI: clap 4.x
- Crypto and identity primitives: ed25519-dalek, sha2, uuid
- HTTP client: reqwest 0.12

## Master Project Plan and Progress Tracker

1. Documentation and architecture model normalization
    + Status: In Progress
    + Goal: Keep docs/design consistent with AURORA-first modeling, SQLite-only persistence, and analyze terminology.

2. AURORA model validation and hygiene
    + Status: Completed
    + Goal: Add lightweight validation to ensure:
        * Every non-root card links (directly or indirectly) to the root driver.
        * No cycles.
        * All link source and target IDs exist.
    + Implementation: `cargo run -p aurora-validator`

3. User-facing terminology consistency
    + Status: In Progress
    + Goal: Ensure CLI and docs use “analyze” consistently while preserving backwards-compatible API paths.
    + Current: Updated docs/CLI_USAGE.md and docs/EXAMPLES.md to remove legacy “ingest” examples and reflect the stable command surface.

4. Seed deprecation follow-through
    + Status: In Progress
    + Goal: Keep user-facing docs referencing export/import rather than “seed”, and reconcile any remaining code-level naming as needed.
    + Current: Removed the “seed” command documentation from user docs; design/docs now describe SQLite snapshot workflows (export/import).

<memory>
Session handoff summary (compressed):

- Design documentation normalization is focused on AURORA-first modeling (canonical JSON in docs/design/AURORA, Markdown views that link to JSON).
- Persistence direction is SQLite-only with export/import via SQLite snapshots, and no JSON state persistence.
- User-facing terminology is “analyze”; /api/v1/ingest remains only as a backwards-compatible API path.
- User docs alignment progress:
    + Updated docs/CLI_USAGE.md and docs/EXAMPLES.md to reflect analyze/status/export/import/serve and removed legacy “seed” and PostgreSQL content from those docs.
    + Updated docs/design/data/MODULES.md to reflect the actual src/database module layout and current snapshot terminology.
- Design docs added/maintained:
    + Architecture overview set under docs/design/architecture/.
    + OWASP-aligned threat card library under docs/design/threat/.
- Legacy “seed” terminology still exists in code comments and some non-updated docs; reconcile/clean-up remains pending (architectural intent is export/import snapshots).
- Prior implementation milestone: NPI/PII detection detect() implementations were completed across remaining types and phone-number detection was enhanced with additional formats.
- Added AURORA model validation tooling:
    + Standalone crate: tools/aurora-validator (runs without compiling the main dumptruck crate).
    + Integration test: tests/aurora_model_validation.rs (will run once the main crate compiles again).
- Note: The main dumptruck crate currently has pre-existing compile failures which block running its full test suite.
</memory>
