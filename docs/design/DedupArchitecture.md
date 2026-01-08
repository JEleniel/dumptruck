# Deduplication Architecture

## Overview

Dumptruck deduplicates and correlates identity-like values (for example, email addresses, phone numbers, user IDs, and other identifier fields) to:

- Reduce repeated work during `analyze` runs.
- Build reusable intelligence in a privacy-preserving way.
- Support reporting that highlights novelty, repetition, and risk.

This document is a human-readable overview. The source of truth for design requirements is the AURORA model.

## Design constraints

Deduplication must remain consistent with:

- SQLite as the primary persistent store.
- A stable user-facing command surface (`analyze`, `status`, `export`, `import`, `serve`).

## System sketch

```text
Input records
	|
	v
Safe input handling + normalization
	|
	v
Compute stable, privacy-preserving indicators (hashes)
	|
	v
SQLite lookup + update
	|
	v
Artifacts and reports
```

## Core approach

### Normalize then hash

Deduplication depends on normalization so that equivalent values map to the same indicator.

At a minimum, normalization should include:

- Unicode canonicalization.
- Case folding.
- Punctuation and whitespace normalization.

### Persist indicators in SQLite

Deduplication state lives in the primary SQLite database.

Design intent:

- Canonical identifiers are stored as irreversible hashes (keyed where appropriate).
- Variant-to-canonical relationships are stored as links.
- Observations are recorded so reports can compute first-seen/last-seen and frequency.

This design does not assume any secondary database.

## Optional: embedding-assisted similarity

Embedding similarity can be used as an additional, best-effort signal for linking candidates.

Constraints:

- SQLite remains the persistence layer.
- Embeddings, if stored, are stored in SQLite and are not required.
- Similarity lookup must not assume specialized vector extensions.

If enabled, similarity should be conservative:

- Use tight thresholds.
- Prefer explainable, deterministic links.
- Treat similarity as a hint, not a primary key.

## Privacy model

Deduplication should not require storing raw sensitive values long-term.

- Persist canonical/variant indicators as hashes.
- Keep raw values in memory only for the duration of analysis.
- Output raw values only when explicitly requested and safe.

## Canonical references

- [Dumptruck root driver][driver-dumptruck-root]
- [Stage-based pipeline requirement][requirement-stage-based-processing-pipeline]
- [SQLite primary storage requirement][requirement-sqlite-primary-storage]
- [Export/import snapshots requirement][requirement-export-import-sqlite-snapshots]

[driver-dumptruck-root]: AURORA/cards/driver-dumptruck-root.json
[requirement-export-import-sqlite-snapshots]: AURORA/cards/requirement-export-import-sqlite-snapshots.json
[requirement-sqlite-primary-storage]: AURORA/cards/requirement-sqlite-primary-storage.json
[requirement-stage-based-processing-pipeline]: AURORA/cards/requirement-stage-based-processing-pipeline.json
