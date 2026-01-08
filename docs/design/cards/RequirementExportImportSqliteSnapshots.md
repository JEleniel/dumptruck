# requirement:export-import-sqlite-snapshots

## Name

Export/import via SQLite database snapshots

## Type

requirement

## Status

approved

## Description

Dumptruck must support exporting the current SQLite database to a standalone SQLite file, and importing/merging another exported SQLite database into the primary database. This workflow is the supported mechanism for sharing curated datasets between instances.

## Sources

- [src/cli.rs](../../../src/cli.rs)
- [src/database.rs](../../../src/database.rs)

## Navigation

- Root driver: [driver:dumptruck-root](driver-dumptruck-root.md)

## Canonical JSON

Source of truth: [docs/design/AURORA/cards/requirement-export-import-sqlite-snapshots.json](../AURORA/cards/requirement-export-import-sqlite-snapshots.json)
