# AGENT_PROGRESS

## Project Brief

Dumptruck is a bulk data analysis system for threat intelligence and credential breach analysis. It ingests large delimited datasets, normalizes messy real-world fields, detects credentials and NPI/PII, enriches findings, and stores historical intelligence as non-reversible hashes. It supports both standalone CLI operation and HTTPS server operation.

Repository: [JEleniel/dumptruck][repo]

Key deliverables for v2 stabilization:

- Capture the intended architecture (Aurora model + views) and map it to the Rust module layout.
- Finish refactoring/implementation so the app compiles, runs, and core CLI+server workflows work end-to-end.
- Implement missing tests to protect detection, normalization, ingestion safety, database, and API contracts.
- Create user documentation (including an executive summary) and a demo pack (scripts + sample files).

### Feature Map (High-Level)

- [ ] **Architecture Model (Aurora)**: Expand the model from the existing mission into drivers/requirements/features/components/tests and provide standard views.
    + Status: In Progress
    + Aurora Mission Card: [docs/design/aurora/mission-5477c32b-28fb-4a18-b359-89437520fe02.json](docs/design/aurora/mission-5477c32b-28fb-4a18-b359-89437520fe02.json)
    + Aurora Application Card: [docs/design/aurora/Application/dumptruck_application-5c1f4f1d-5cfe-4e46-b117-7102662e40f0.json](docs/design/aurora/Application/dumptruck_application-5c1f4f1d-5cfe-4e46-b117-7102662e40f0.json)
    + Aurora Actor Card: [docs/design/aurora/Actor/user-2f4c0a60-1a18-4e21-9d16-1d1e396f3d4a.json](docs/design/aurora/Actor/user-2f4c0a60-1a18-4e21-9d16-1d1e396f3d4a.json)
    + Human Mission Card: [docs/design/aurora/DumptruckMission.md](docs/design/aurora/DumptruckMission.md)
    + Design Diagrams: [docs/design/diagrams](docs/design/diagrams)
    + Design Index: [docs/design/README.md](docs/design/README.md)

- [ ] **Build + Runtime Baseline**: Make `cargo build` and minimal runs succeed on Linux/macOS/Windows.
    + Status: In Progress
    + Notes: Entry point portability/logging issues were resolved (Linux-only std import removed; `tracing::error` imported in the binary entrypoint).

- [ ] **CLI Analyze (Core Pipeline)**: Ingest, normalize, detect, enrich, and report for CSV/TSV/PSV datasets.
    + Status: In Progress

- [ ] **Server Mode (HTTPS API)**: Implement server startup, routing, authentication, and handlers for ingest/status.
    + Status: Pending

- [ ] **Database Import/Export**: Export seed DB and merge/import to support cross-instance sharing.
    + Status: Pending

- [ ] **Test Suite Coverage**: Unit + integration coverage for core invariants.
    + Status: Pending

- [ ] **User Documentation**: Practical CLI + server docs, config reference, security model.
    + Status: Pending

- [ ] **Demo Pack**: Repeatable scripts and sample data files for common workflows.
    + Status: Pending

- [ ] **Executive Summary**: High-level non-technical overview, risks/constraints, and outcomes.
    + Status: Pending

## Active Context Summary

- Branch: v2.0.0
- State of repo:
    + Rust edition: 2024.
    + CLI command surface exists (analyze/status/export/import/serve) but only analyze is wired in runtime dispatcher.
    + Server and status modules currently appear to expose argument structs only.
    + Aurora model includes a mission card involving a User actor and necessitating a Dumptruck application card.
    + Features satisfy the Breach Data Analysis Tool requirement and are explained by User stories.
    + Requirement imposes constraints on secure deletion, TLS, and data protection.
    + Capability exists for running the tool in a secure, disposable environment.
    + That capability necessitates a setup/use/teardown process involving the User actor.
    + Application comprises component cards mapped to Rust modules (cli/datafile/database/analyze/server/util/api/configuration/status).
    + Component `analyze` generates artifacts for an Analysis Report and Learned Data.
    + Artifact Learned Data persists to the SQLite data store.
    + Component `database` depends on the SQLite data store.
    + Components `cli`, `database`, `analyze`, `server`, `api`, `configuration`, and `status` implement the CLI/HTTPS API feature.
    + Component `datafile` implements safe streaming ingestion.
    + Component `database` implements privacy-first persistence and cross-instance sharing.
    + Component `analyze` implements normalization/deduplication.
    + Component `util` is marked as support.
    + Human-readable cards exist as Markdown siblings under docs/design/aurora/.
    + Standard model views exist under docs/design/diagrams/.

## Patterns

- Streaming-first ingestion and processing (GB/TB-scale inputs).
- Privacy-first persistence: store historical intelligence as non-reversible hashes.
- Conservative detection and enrichment: prefer explainable scoring and extensibility.
- Strong operational stance: failures should be recoverable; panics treated as bugs.
- Platform portability: avoid OS-specific APIs unless gated and tested.

## Technologies

- Rust 2024, Tokio, Rayon
- CLI: clap
- HTTP server: axum, hyper, tower, tower-http, axum-server
- TLS/auth: rustls, openssl (vendored), ed25519-dalek
- DB: rusqlite (bundled) + r2d2
- Parsing/normalization: csv, regex, unicode-normalization, ICU4X (icu_casemap)
- HTTP clients: reqwest
- Serialization: serde, serde_json

## Master Project Plan and Progress Tracker

1. Architecture Capture (Architect)
    + Outcome: A complete Aurora model in docs/design/aurora/ describing runtime structure and traceability.
    + Tasks:
        * Create driver and requirement cards that reflect the README promises (safe ingestion, privacy-first hashing, CLI+server, import/export, detection coverage).
        * Create system/application/component/interface cards mapped to the current Rust modules (cli/api/database/analyze/util/network/enrichment/server/status).
        * Create data_store cards for SQLite DB and any on-disk artifacts (working files/temp copies, export seed DB).
        * Add test cards tracing to requirements/features where appropriate.
        * Add deployment/node cards for standalone binary and server deployment.
        * Produce standard views (Component, Requirements, Deployment, Everything) and ensure link-direction invariants (away from mission) are satisfied.
    + Status: In Progress

2. Unblock Build and Establish a Minimal Running Baseline (BackendDeveloper)
    + Outcome: `cargo build` works; `dumptruck --help` runs; `dumptruck analyze --help` runs.
    + Tasks:
        * Fix platform-specific and missing imports/macros in entrypoints.
        * Resolve missing workspace member (either add the tool crate or remove it from workspace config).
        * Ensure logging/tracing is configured and compilation succeeds on Linux; add portability checks for macOS/Windows.
    + Status: In Progress

3. Finish CLI Runtime Dispatcher and Core Workflows (BackendDeveloper)
    + Outcome: All CLI commands declared in the CLI surface are implemented and wired.
    + Tasks:
        * Implement and wire `Commands::Serve`, `Commands::Status`, `Commands::Import`, `Commands::Export` in the main dispatcher.
        * Ensure config loading + CLI overrides apply consistently across commands.
        * Define exit codes and error propagation strategy across CLI and server.
    + Status: Pending

4. Implement HTTPS Server Mode (BackendDeveloper)
    + Outcome: Server starts, authenticates requests, and supports ingestion + status endpoints end-to-end.
    + Tasks:
        * Implement server startup (HTTP/2-only), TLS configuration, OAuth 2.0 handling.
        * Implement handlers and routing in src/api/ and src/server/ with clear response types.
        * Add integration tests that exercise key endpoints.
    + Status: Pending

5. Database Import/Export and Cross-Instance Sharing (BackendDeveloper)
    + Outcome: Export creates a portable seed DB; import merges safely and deterministically.
    + Tasks:
        * Implement export to SQLite file and document expected invariants.
        * Implement import/merge logic with conflict handling and idempotency.
        * Add tests verifying round-trip correctness and merge behavior.
    + Status: Pending

6. Test Coverage Expansion (TestDeveloper)
    + Outcome: A regression suite that protects core invariants and high-risk parsing/detection logic.
    + Priorities:
        * Unit tests: normalization, hashing/HMAC boundaries, file safety checks, individual detectors.
        * Integration tests: analyze pipeline over small fixtures, DB persistence, export/import, server endpoints.
        * Property/edge tests: malformed UTF-8, extreme line lengths, delimiter edge cases, Unicode confusables.
    + Status: Pending

7. User Documentation + Executive Summary (TechnicalWriter)
    + Outcome: A documentation set that supports first-time adoption and safe operation.
    + Tasks:
        * Executive summary: problem, what Dumptruck does/does not do, security model, constraints.
        * CLI documentation: command reference, config usage, workflows.
        * Server documentation: auth, endpoints, operational guidance.
        * Data format guidance: supported delimiters, common pitfalls, examples.
    + Status: Pending

8. Demo Pack (UIDeveloper or TechnicalWriter, as appropriate)
    + Outcome: A repeatable demo folder containing scripts and sample datasets for common scenarios.
    + Tasks:
        * Provide sample CSV/TSV/PSV inputs (small, medium, and malformed examples).
        * Provide scripts for common workflows: analyze, export/import seed, run server, call endpoints.
        * Ensure demos run without external dependencies by default; isolate optional Ollama/vector demo.
    + Status: Pending

---

[repo]: https://github.com/JEleniel/dumptruck
