# Feature: Extensibility & Formats

Summary

- Purpose: Make it straightforward to add new input formats, normalization rules, and enrichment modules.

Goals

- Plugin/adapters model for parsers and enrichers.
- Config-driven transforms so non-developers can adjust normalization rules.

Acceptance Criteria

- New parser or enrichment plugin can be registered without modifying core pipeline code.
- Documentation and example plugin templates are provided.

Implementation Notes

- Design a simple adapter interface and a registry discovered at startup.
- Consider using dynamic loading or a configuration-driven command registration.
