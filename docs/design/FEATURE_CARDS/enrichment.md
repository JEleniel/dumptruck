# Feature: Enrichment

Summary

- Purpose: Enrich incoming records with metadata, correlations, and identity linking.

Goals

- Detect duplicates across a dump and across historic data.
- Link credentials to identifiers (e.g., email, user ID) and surface correlations.

Acceptance Criteria

- Enrichment outputs additional fields (hashes, link ids, confidence scores) for records.
- Enrichment operates incrementally and can be replayed for backfills.

Implementation Notes

- Design enrichment pipeline stages: tokenization -> matching -> scoring -> attach.
- Keep enrichment side-effects idempotent and logged.
