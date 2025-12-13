# Components

This document describes Dumptruck's primary components, responsibilities, and public interfaces.

1. Ingestion

- Responsibilities: Accept input files or streams, detect format, provide a uniform input representation to the pipeline.
- Inputs: local files, uploaded payloads via API, streamed dumps.
- Output: tokenized/streamed records handed to Normalization.

2. Normalization

- Responsibilities: Normalize fields, apply canonicalization rules (trim, case normalization, substitution, delimiter handling), validate types.
- Extensibility: format-normalizers for Protobuf/JSON/CSV/etc. can be added as plugins.

3. Enrichment

- Responsibilities: Add derived attributes (e.g., domain extraction, hash forms, inferred types, cross-field enrichment), tag suspicious records.
- Execution: rule-based pipeline; supports chaining and short-circuiting.

4. Analysis

- Responsibilities: Detect patterns, compute statistics, find repeated/novel leaks, generate reports and SIGINT-able streaming results for CLI.
- Includes: deduplication, grouping, correlation with history.

5. Storage & History

- Responsibilities: Persist analysis outputs, history hashes, and metadata.
- Privacy: historic values are stored as keyed non-reversible hashes (HMAC or keyed KDF) rather than plaintext.
- Backends: abstract storage interface — support for local filesystem, object stores, SQL/NoSQL.

6. API & CLI

- CLI: single-node quick-run tool for analysts.
- API Server: REST endpoints for upload, status, results; supports authentication (OAuth2/OIDC) and TLS.

7. Server Runtime

- Responsibilities: orchestration of worker processes, request routing, rate-limiting, configuration loading, metrics and health endpoints.

8. Extensibility/Plugin System

- Well-defined extension points: format adapters, enrichment rules, analyzers, storage adapters.
- Contract: small, language-native adapters (prefer Rust traits) with safe sandboxing in server mode.

9. Observability

- Telemetry: metrics (Prometheus), structured logs, traces.
- Health: liveness/readiness endpoints in server mode.

Interfaces and data contracts

- Use compact, documented schemas for record representation inside the pipeline. Keep raw input separate from normalized/enriched representations.
