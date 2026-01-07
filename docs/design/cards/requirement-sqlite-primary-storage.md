# requirement:sqlite-primary-storage

## Name

SQLite is the primary persistent store

## Type

requirement

## Status

approved

## Description

All operational data for Dumptruck (history, dedup indicators, analysis artifacts, metadata) is persisted in a single SQLite database file. The system does not rely on JSON files for persistent state.

## Sources

- [src/database.rs](../../../src/database.rs)
- [src/database](../../../src/database)

## Constraints

- constraint:no-json-persistence

## Canonical JSON

Source of truth: [docs/design/AURORA/cards/requirement-sqlite-primary-storage.json](../AURORA/cards/requirement-sqlite-primary-storage.json)
