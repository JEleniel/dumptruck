# constraint:no-json-persistence

## Name

No JSON persistence

## Type

constraint

## Status

approved

## Description

Dumptruck must not use JSON files as a primary or authoritative persistence mechanism. JSON may be used for report output or API responses, but persistent state lives in SQLite.

## Canonical JSON

Source of truth: [docs/design/AURORA/cards/constraint-no-json-persistence.json](../AURORA/cards/constraint-no-json-persistence.json)
