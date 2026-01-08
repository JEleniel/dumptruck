# driver:dumptruck-root

## Name

Dumptruck: Privacy-Preserving Data Dump Analysis

## Type

driver

## Status

approved

## Description

Dumptruck exists to securely analyze bulk data dumps (e.g., breach/leak datasets) in a privacy-preserving way, producing actionable reports and building reusable intelligence (deduplication, enrichment, history) while supporting both standalone CLI and HTTPS server operation.

## Sources

- [docs/design/Readme.md](../Readme.md)
- [docs/design/PipelineMap.md](../PipelineMap.md)
- [docs/design/architecture/ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

## Attributes

- Repository: JEleniel/dumptruck
- Primary modes: CLI, Server

## Navigation

Stakeholders:

- [actor:analyst](actor-analyst.md)
- [actor:operator](actor-operator.md)
- [actor:maintainers](actor-maintainers.md)

Derived requirements:

- [requirement:command-surface](requirement-command-surface.md)
- [requirement:stage-based-processing-pipeline](requirement-stage-based-processing-pipeline.md)
- [requirement:sqlite-primary-storage](requirement-sqlite-primary-storage.md)
- [requirement:export-import-sqlite-snapshots](requirement-export-import-sqlite-snapshots.md)
- [requirement:dual-mode-cli-server](requirement-dual-mode-cli-server.md)
- [requirement:rainbow-table-folder-import](requirement-rainbow-table-folder-import.md)

## Canonical JSON

Source of truth: [docs/design/AURORA/cards/driver-dumptruck-root.json](../AURORA/cards/driver-dumptruck-root.json)
