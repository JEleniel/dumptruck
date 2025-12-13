# Feature: Storage & Hashing

Summary

- Purpose: Define storage model for enriched records, historic indexes, and analysis artifacts.

Goals

- Use an append-friendly store for incoming records and indexes for fast lookups.
- Store only hashed identifiers for sensitive fields; keep indexes efficient for lookups.

Acceptance Criteria

- Storage layout supports efficient incremental ingest and query by hashed keys.
- Retention and compaction processes are documented and implementable.

Implementation Notes

- Consider using PostgreSQL with pgvector or lightweight key-value stores for indexes.
- Provide clear migration and backup guidance for stored artifacts.
