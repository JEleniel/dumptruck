# Design

This directory contains Dumptruck’s design source of truth and supporting design documents.

## AURORA

Dumptruck uses the AURORA card/link model as the canonical design source of truth.

- Canonical cards (JSON): [AURORA/cards/][aurora-cards]
- Canonical links (JSON): [AURORA/links/][aurora-links]
- Human-readable cards (Markdown): [cards/][design-cards]
- Human-readable links (Markdown): [links/][design-links]

Conventions:

- Canonical JSON lives only under [AURORA/][aurora-root].
- Human-readable card/link Markdown files must not embed JSON and must link to the canonical JSON.

Root driver:

- Card (Markdown): [cards/driver-dumptruck-root.md][driver-root-md]
- Card (JSON): [AURORA/cards/driver-dumptruck-root.json][driver-root-json]

## Core design docs

- Capabilities: [Capabilities.md][capabilities]
- Pipeline map: [PIPELINE_MAP.md][pipeline-map]
- Deduplication design: [DEDUP_ARCHITECTURE.md][dedup-architecture]

## Architecture docs

- Architecture: [architecture/ARCHITECTURE.md][architecture]
- Components: [architecture/COMPONENTS.md][components]
- Interfaces: [architecture/INTERFACES.md][interfaces]
- Data flow and extensibility: [architecture/DATA_FLOW_AND_EXTENSIBILITY.md][data-flow]
- Deployment: [architecture/DEPLOYMENT.md][deployment]
- Security: [architecture/SECURITY.md][security]
- Diagrams: [architecture/diagrams/][diagrams]

## Naming and persistence rules

- User-facing commands are: analyze, status, export, import, serve.
- Persistent state is SQLite-only. JSON is used only for report outputs and API payloads.
- Server API paths may use `/api/v1/ingest` for backwards compatibility, but semantically they submit analysis jobs.

[architecture]: architecture/ARCHITECTURE.md
[aurora-cards]: AURORA/cards/
[aurora-links]: AURORA/links/
[aurora-root]: AURORA/
[capabilities]: Capabilities.md
[components]: architecture/COMPONENTS.md
[data-flow]: architecture/DATA_FLOW_AND_EXTENSIBILITY.md
[dedup-architecture]: DEDUP_ARCHITECTURE.md
[deployment]: architecture/DEPLOYMENT.md
[design-cards]: cards/
[design-links]: links/
[diagrams]: architecture/diagrams/
[driver-root-json]: AURORA/cards/driver-dumptruck-root.json
[driver-root-md]: cards/driver-dumptruck-root.md
[interfaces]: architecture/INTERFACES.md
[pipeline-map]: PIPELINE_MAP.md
[security]: architecture/SECURITY.md
