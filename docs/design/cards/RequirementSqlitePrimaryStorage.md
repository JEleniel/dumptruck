# requirement:sqlite-primary-storage

## Name

SQLite is the primary persistent store

## Type

requirement

## Status

approved

## Description

All operational data for Dumptruck (history, dedup indicators, analysis artifacts, metadata) is persisted in a single SQLite database file. SQLite is the authoritative persistent store; JSON may be used for interchange artifacts such as report outputs, API payloads, or optional exports.

## Sources

- [src/database.rs](../../../src/database.rs)
- [src/database](../../../src/database)

## Navigation

- Root driver: [driver:dumptruck-root](driver-dumptruck-root.md)

## Canonical JSON

Source of truth: [docs/design/AURORA/cards/requirement-sqlite-primary-storage.json](../AURORA/cards/requirement-sqlite-primary-storage.json)
