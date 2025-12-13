# Dumptruck Architecture

Purpose

- Provide a single-source architecture reference for Dumptruck's design, trade-offs, and operational characteristics.

Goals

- Ingest and normalize large, heterogeneous bulk data dumps.
- Enrich and analyze data while preserving privacy of historical data.
- Be extensible (formats, enrichment rules, storage backends).
- Support both CLI and Server modes with secure access controls.

Scope

- This document covers high-level structure, major components, and interactions. Details are in companion documents: `COMPONENTS.md`, `DEPLOYMENT.md`, `SECURITY.md`, and `DATA_FLOW_AND_EXTENSIBILITY.md`.

High-level system context

- Input sources: files (CSV, TSV, JSON, YAML, XML, Protobuf, BSON), streamed dumps.
- Processing pipeline: Ingestion -> Normalization -> Enrichment -> Analysis -> Storage/History.
- Interfaces: Local CLI, REST API server endpoint, optional batch scheduler.

Key non-functional requirements

- Scalability: handle multi-GB dumps; scale horizontally for server mode.
- Privacy: historical data stored as non-reversible hashes; minimal retention.
- Performance: CPU- and IO-efficient pipeline; stream processing where possible.
- Extensibility: plugin points for formats, enrichers, and storage backends.

Primary runtime modes

- CLI: single-process analysis for ad-hoc runs.
- Server: long-running service with REST ingestion and analysis endpoints; supports OAuth2/OIDC and TLS.

Constraints and assumptions

- Host environments provide TLS and container orchestration for server deployments.
- Persistent storage may be object stores or databases behind an abstraction layer.
- Operators will configure secrets and keys (no built-in secret manager).

Where to read more

- Architecture details: `docs/architecture/COMPONENTS.md`
- Deployment patterns: `docs/architecture/DEPLOYMENT.md`
- Security and data handling: `docs/architecture/SECURITY.md`
- Data flows and extensibility: `docs/architecture/DATA_FLOW_AND_EXTENSIBILITY.md`
