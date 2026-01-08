# requirement:rainbow-table-folder-import

## Name

Import rainbow table wordlists from a folder

## Type

requirement

## Status

proposed

## Description

Dumptruck must support importing all `.txt` files in a specified folder as a rainbow table seed source.

The folder path is provided via CLI option or configuration.

- Each file is UTF-8 and contains one value per line.
- For each value, Dumptruck precomputes common unsalted password hash types (at minimum: MD5, SHA-1, SHA-256).
- The resulting indicators are persisted in SQLite so weak-password detection can match hashed credentials without storing raw values long-term.

## Sources

- [src/database/rainbowtable.rs](../../../src/database/rainbowtable.rs)
- [src/detection/rainbow_table.rs](../../../src/detection/rainbow_table.rs)
- [docs/design/data/MODULES.md](../data/MODULES.md)

## Navigation

- Root driver: [driver:dumptruck-root](driver-dumptruck-root.md)

## Canonical JSON

Source of truth: [docs/design/AURORA/cards/requirement-rainbow-table-folder-import.json](../AURORA/cards/requirement-rainbow-table-folder-import.json)
