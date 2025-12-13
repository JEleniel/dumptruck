# Feature: Server & CLI Modes

Summary

- Purpose: Support both single-run CLI workflows and long-running server analysis with an ingestion API.

Goals

- CLI: run against local files, produce artifacts, and support scripting/CI use.
- Server: expose authenticated HTTP API for upload, status, and retrieval of analysis results.

Acceptance Criteria

- CLI and Server share core processing code and configuration semantics.
- Server supports upload, job status, and result retrieval endpoints with pagination.

Implementation Notes

- Reuse the same pipeline core; provide thin adapters for CLI and HTTP ingestion.
- Design server API with OpenAPI and provide example curl commands.
