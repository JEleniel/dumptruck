
# Feature: Ingestion

Summary

- Purpose: Accept bulk data dumps in many formats and feed them reliably into the pipeline.

Goals

- Support CSV, TSV, JSON, YAML, XML, Protobuf, BSON, and streamed input.
- Provide clear error reporting and recovery for malformed inputs.

Acceptance Criteria

- CLI and server accept files or streams and produce normalized record batches.
- Parsers provide row/record counts, errors, and sample context for failures.

Implementation Notes

- Use pluggable parser adapters per format; detect format via file magic and heuristics.
- Provide streaming parsing to limit memory usage for very large dumps.
