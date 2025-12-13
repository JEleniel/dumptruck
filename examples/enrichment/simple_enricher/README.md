Simple Enricher Example

This example demonstrates a minimal Rust-based enricher for Dumptruck.

Usage

- Read `src/lib.rs` for a straightforward implementation.
- To integrate into the main project, implement `crate::enrichment::EnrichmentPlugin`
  from `src/enrichment.rs` and adapt the `enrich()` logic accordingly.

Rationale

- Simple, compile-time Rust extensions are preferred: they are fast, safe, and
  easy to test. This example intentionally avoids WASM or runtime plugin
  sandboxes to keep development simple.
