# requirement:command-surface

## Name

Stable CLI command surface

## Type

requirement

## Status

approved

## Description

Dumptruck must expose a stable command surface consisting of:

- analyze (process files through the pipeline)
- status (query a running server instance)
- export (copy/export the SQLite database)
- import (merge another SQLite database into the main database)
- serve (run the HTTP server)

## Sources

- [src/cli.rs](../../../src/cli.rs)

## Navigation

- Root driver: [driver:dumptruck-root](driver-dumptruck-root.md)

## Canonical JSON

Source of truth: [docs/design/AURORA/cards/requirement-command-surface.json](../AURORA/cards/requirement-command-surface.json)
