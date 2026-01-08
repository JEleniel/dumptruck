# Design

This directory contains Dumptruck’s design source of truth and supporting design documents.

## AURORA

Dumptruck uses the AURORA card/link model as the canonical design source of truth.

- Canonical cards (JSON): [AURORA/cards/][aurora-cards]
- Canonical links (JSON): [AURORA/links/][aurora-links]
- Human-readable cards (Markdown): [cards/][design-cards]

Conventions:

- Canonical JSON lives only under [AURORA/][aurora-root].
- Human-readable card Markdown files must not embed JSON and must link to the canonical JSON card.
- Human-readable card Markdown files include navigational links to related cards (instead of separate per-link Markdown files).

Root driver:

- Card (Markdown): [cards/driver-dumptruck-root.md][driver-root-md]
- Card (JSON): [AURORA/cards/driver-dumptruck-root.json][driver-root-json]

## Card index

Driver:

- [driver:dumptruck-root][driver-root-md]

Actors:

- [cards/actor-analyst.md](cards/actor-analyst.md)
- [cards/actor-operator.md](cards/actor-operator.md)
- [cards/actor-maintainers.md](cards/actor-maintainers.md)

Requirements:

- [cards/requirement-command-surface.md](cards/requirement-command-surface.md)
- [cards/requirement-stage-based-processing-pipeline.md](cards/requirement-stage-based-processing-pipeline.md)
- [cards/requirement-sqlite-primary-storage.md](cards/requirement-sqlite-primary-storage.md)
- [cards/requirement-export-import-sqlite-snapshots.md](cards/requirement-export-import-sqlite-snapshots.md)
- [cards/requirement-dual-mode-cli-server.md](cards/requirement-dual-mode-cli-server.md)
- [cards/requirement-rainbow-table-folder-import.md](cards/requirement-rainbow-table-folder-import.md)

## Core design docs

- Capabilities: [Capabilities.md][capabilities]
- Pipeline map: [PipelineMap.md][pipeline-map]
- Deduplication design: [DedupArchitecture.md][dedup-architecture]

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
- SQLite is the primary persistent store. JSON may be used for interchange artifacts such as report outputs, API payloads, or optional exports.
- Server API paths may use `/api/v1/ingest` for backwards compatibility, but semantically they submit analysis jobs.

[architecture]: architecture/ARCHITECTURE.md
[aurora-cards]: AURORA/cards/
[aurora-links]: AURORA/links/
[aurora-root]: AURORA/
[capabilities]: Capabilities.md
[components]: architecture/COMPONENTS.md
[data-flow]: architecture/DATA_FLOW_AND_EXTENSIBILITY.md
[dedup-architecture]: DedupArchitecture.md
[deployment]: architecture/DEPLOYMENT.md
[design-cards]: cards/
[diagrams]: architecture/diagrams/
[driver-root-json]: AURORA/cards/driver-dumptruck-root.json
[driver-root-md]: cards/driver-dumptruck-root.md
[interfaces]: architecture/INTERFACES.md
[pipeline-map]: PipelineMap.md
[security]: architecture/SECURITY.md
