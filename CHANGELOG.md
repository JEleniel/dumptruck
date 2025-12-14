# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/) and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- GitHub Actions CI/CD workflows (lint, test, security, release, docker, build)
- Versioning and release process documentation
- Kubernetes deployment manifests (in progress)
- Security operations and key rotation guidance (in progress)

### Changed

- Improved error messages for malformed input files
- Enhanced documentation with examples and architecture diagrams
- **Relicensed from MIT/Apache 2.0 to GNU General Public License v3.0 or later (GPL-3.0-or-later)**

### Fixed

- TLS provider initialization for rustls 0.23+ compatibility
- Unicode normalization edge cases

## [0.1.0](https://github.com/jeleniel/dumptruck/releases/tag/v0.1.0) - 2025-12-13

### Added

- Core credential analysis pipeline
    + CSV, TSV, JSON, YAML, XML, Protobuf, BSON input formats
    + Async processing with tokio
    + PostgreSQL storage with pgvector embeddings
    + Configurable normalization rules (trim, substitution, case conversion)
    + Deduplication with Unicode normalization and vector similarity search
    + Hash detection: bcrypt, scrypt, argon2, pbkdf2, MD5, SHA1, SHA256
    + Weak password detection with 120+ pre-computed hashes
    + Have I Been Pwned (HIBP) integration for breach enrichment
    + Ollama integration for 768-dimensional embeddings
    + Email domain alias substitution for canonical deduplication

- CLI Tool
    + Glob pattern support for batch processing
    + Parallel file processing (configurable worker count)
    + Multiple output formats: JSON, CSV, Text, JSONL
    + Comprehensive error handling and reporting
    + Status command for pipeline introspection

- Server Mode
    + HTTP/2 support via Hyper 1.5
    + TLS 1.3+ with rustls (no OpenSSL dependency)
    + OAuth 2.0 client credentials authentication
    + REST API endpoint for data ingestion
    + Configurable concurrency and timeouts
    + Request validation and rate limiting

- Data Enrichment
    + Credential hash detection with multiple algorithms
    + Breach correlation with HIBP API v3
    + Vector similarity search for near-duplicate detection (IVFFlat index)
    + Custom enrichment adapter support
    + Configurable API keys and credentials

- Testing & Quality
    + 82 passing unit and integration tests
    + 18 test fixtures covering edge cases
    + Stress testing suite (>800 requests/sec on Raspberry Pi 5)
    + Continuous integration with GitHub Actions
    + Code coverage reporting with codecov
    + Security scanning (cargo audit, clippy, unsafe code detection)

- Documentation
    + Architecture documentation with component diagrams
    + Configuration reference guide
    + Email suffix substitution guide
    + HIBP integration guide
    + Ollama embeddings guide
    + Vector deduplication details
    + CLI usage examples
    + Server deployment guide
    + Adapter development guide

- Deployment
    + Docker Compose stack with PostgreSQL + Ollama + pgvector
    + Self-signed TLS certificates for testing
    + Configuration schema validation
    + Health check endpoints
    + Graceful shutdown handling

### Known Limitations

- Single-node deployment only (no clustering/sharding)
- No built-in web UI (API-only)
- Limited to 4GB+ RAM systems (Ollama embedding requirement)

---

## Guidance for Future Releases

### Pre-Release Checklist

- [ ] All tests passing (`cargo test --all`)
- [ ] Linting clean (`cargo fmt --check && cargo clippy`)
- [ ] Security audit passes (`cargo audit`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated with [Unreleased] content
- [ ] Version bumped in Cargo.toml

### Release Instructions

See [VERSIONING.md](docs/VERSIONING.md) for detailed release process.

[Unreleased]: https://github.com/jeleniel/dumptruck/compare/v0.1.0...HEAD
