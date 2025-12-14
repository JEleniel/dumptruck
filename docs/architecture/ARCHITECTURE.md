# Dumptruck Architecture

## Purpose

Provide a single-source architecture reference for Dumptruck's design, trade-offs, and operational characteristics.

## Goals

- Ingest and normalize large, heterogeneous bulk data dumps
- Enrich and analyze data while preserving privacy of historical data
- Be extensible (formats, enrichment rules, storage backends)
- Support both CLI and Server modes with secure access controls

## Scope

This document covers high-level structure, major components, and interactions. Details are in companion documents:

- [COMPONENTS.md](COMPONENTS.md) - Internal component architecture
- [DEPLOYMENT.md](DEPLOYMENT.md) - Production deployment patterns
- [SECURITY.md](SECURITY.md) - Security design and operational procedures
- [DATA_FLOW_AND_EXTENSIBILITY.md](DATA_FLOW_AND_EXTENSIBILITY.md) - Data flow and plugin points
- [INTERFACES.md](INTERFACES.md) - Public API and CLI specifications

## Architecture Diagrams

### Data Pipeline

![Pipeline](diagrams/pipeline.md)

The primary data pipeline moves input files through the following stages:

1. **Ingest**: Format detection and streaming read
2. **Safe Ingest**: Binary detection, UTF-8 validation, size limits
3. **Normalization**: Unicode canonicalization with NFKC + case-folding
4. **Deduplication**: Hash-based exact match + vector similarity detection
5. **Enrichment**: Ollama embeddings + Have I Been Pwned breach data
6. **Analysis**: PII/NPI detection (phones, SSNs, credit cards, crypto, etc.)
7. **Storage**: PostgreSQL with deduplication graph and breach metadata

### Component Architecture

![Components](diagrams/components.md)

The runtime consists of:

- **Server Runtime**: REST API with job queue and worker pool
- **Pipeline Stages**: Composable, testable modules for each processing stage
- **Extensibility Points**: Format adapters, storage adapters, enrichment plugins
- **External Systems**: PostgreSQL, Ollama, Have I Been Pwned API

### Deployment Architecture

![Deployment](diagrams/deployment.md)

Production deployments feature:

- **Horizontal Scaling**: Stateless API replicas behind load balancer
- **Background Workers**: Configurable worker pool consuming from job queue
- **Security**: TLS 1.3+, OAuth 2.0, secrets management
- **Observability**: Prometheus metrics, structured logging, distributed tracing

## High-level System Context

**Input Sources**
- Files: CSV, TSV, JSON, YAML, XML, Protobuf, BSON
- Streaming: HTTP multipart uploads with chunked processing

**Processing Pipeline**
- Ingestion → Normalization → Deduplication → Enrichment → Analysis → Storage/History

**Interfaces**
- Local CLI for ad-hoc analysis
- REST API server endpoint (HTTP/2, TLS 1.3+, OAuth 2.0)
- Optional batch scheduler for recurring jobs

## Key Non-Functional Requirements

| Requirement | Solution |
|---|---|
| **Scalability** | Stream-oriented processing; horizontal scaling via job queue; multi-GB file support |
| **Privacy** | Non-reversible hashes for historical data; optional HIPAA/PII redaction |
| **Performance** | CPU/IO-efficient; 800+ req/sec on Raspberry Pi 5; constant memory for GB/TB files |
| **Extensibility** | Plugin trait interfaces for formats, enrichers, storage backends |
| **Reliability** | Zero-crash guarantee; error resilience; comprehensive audit logging |
| **Security** | TLS 1.3+; OAuth 2.0; secrets isolation; SAST/dependency scanning in CI/CD |

## Primary Runtime Modes

### CLI Mode
- Single-process analysis for ad-hoc runs
- Glob pattern support for batch file processing
- Parallel processing via rayon (configurable workers)
- Multiple output formats (JSON, CSV, Text, JSONL)

### Server Mode
- Long-running service with REST ingestion and analysis endpoints
- OAuth 2.0/OIDC authentication
- TLS 1.3+ encryption
- Background job queue for async processing
- Configurable worker pool (default: CPU cores)
- Horizontal scaling behind load balancer

## Design Principles

1. **Stream-Oriented**: Process records one at a time to minimize memory footprint
2. **Privacy-Preserving**: Store only non-reversible hashes of sensitive data
3. **Error-Resilient**: Malformed rows logged but processing continues (zero-crash guarantee)
4. **Observable**: All stages emit metrics and structured logs for debugging
5. **Extensible**: Plugin points at format, enrichment, and storage layers
6. **Testable**: Each component can be tested independently with fixtures
7. **Safe**: 100% safe Rust (zero unsafe blocks); comprehensive test coverage

## Constraints and Assumptions

- **Host Environment**: Provides TLS termination and container orchestration (Kubernetes or Docker)
- **Storage Layer**: Persistent storage behind abstraction layer (PostgreSQL, S3, or filesystem)
- **Secrets Management**: Operators configure secrets and API keys (no built-in secret manager)
- **External APIs**: Have I Been Pwned and Ollama APIs are optional; system degrades gracefully without them
- **Deployment**: Designed for Linux x86_64 and ARM64; Windows support is secondary

## Key Components

### API Server (src/server.rs)
- Axum 0.8 web framework with HTTP/2 support
- TLS 1.3+ via rustls + axum-server
- OAuth 2.0 client credentials flow
- REST endpoints for ingestion and status checks
- Background worker coordination via job queue

### Pipeline (src/pipeline.rs, src/async_pipeline.rs)
- Synchronous pipeline for CLI mode
- Asynchronous pipeline for server mode with tokio integration
- Composable stages: Ingest → Normalize → Dedup → Enrich → Analyze → Store

### Storage (src/storage.rs)
- StorageAdapter trait for multiple backends
- PostgreSQL implementation with pgvector for embeddings
- Filesystem implementation for local development
- Hash-based deduplication with vector similarity search

### Enrichment (src/enrichment.rs)
- Ollama integration for embedding generation (768-dim Nomic vectors)
- Have I Been Pwned API integration for breach data
- Optional plugins for custom enrichment logic

### Detection (src/npi_detection.rs, src/hash_utils.rs)
- PII/NPI detection: phones, SSNs, credit cards, crypto addresses, national IDs
- Hash detection: bcrypt, scrypt, argon2, pbkdf2, MD5/SHA1/SHA256
- Weak password detection via rainbow tables

### Normalization (src/normalization.rs)
- Unicode NFKC normalization
- ICU4X case-folding
- Punctuation and whitespace normalization
- Email domain canonicalization with configurable substitutions

## Data Flow

```
User Input (File/API)
    ↓
[Ingest] Format detection, streaming read
    ↓
[Safe Ingest] UTF-8 validation, binary detection, size checks
    ↓
[Normalization] Unicode normalization, case-folding
    ↓
[Deduplication] Hash lookup + vector similarity search
    ↓
[Enrichment] Ollama embeddings + HIBP lookup
    ↓
[Analysis] PII detection, pattern recognition
    ↓
[Storage] PostgreSQL insert/update with audit logging
    ↓
Output (JSON/CSV/Text)
```

## Extensibility

Dumptruck is designed to be extended at three primary points:

1. **Format Adapters**: Add support for new input formats by implementing the `FormatAdapter` trait
2. **Storage Adapters**: Implement custom backends (S3, MongoDB, etc.) via the `StorageAdapter` trait
3. **Enrichment Plugins**: Custom enrichment logic via WASM-based or native Rust plugins

See [DATA_FLOW_AND_EXTENSIBILITY.md](DATA_FLOW_AND_EXTENSIBILITY.md) for detailed extension patterns.

## Performance Characteristics

- **Throughput**: >800 requests/second (HTTP/2 + TLS on Raspberry Pi 5)
- **Memory**: Constant O(1) memory regardless of file size (streaming)
- **Latency**: Sub-100ms p99 for API requests; parallelizable batch processing
- **Storage**: Deduplication reduces database size by 40-80% on typical leak datasets

See [PARALLEL_PROCESSING_SUMMARY.md](../PARALLEL_PROCESSING_SUMMARY.md) for detailed benchmarks.

## Security Architecture

- **Transport**: TLS 1.3+ with modern cipher suites (AEAD+PFS)
- **Authentication**: OAuth 2.0 client credentials for API access
- **Authorization**: Scope-based access control (to be implemented)
- **Data**: Non-reversible hashing of sensitive fields; optional field-level encryption
- **Audit**: Comprehensive logging of all API calls and data modifications
- **Dependencies**: Automated vulnerability scanning via cargo-audit in CI/CD

See [SECURITY.md](SECURITY.md) for operational security procedures.

## Operational Readiness

✅ **Code Quality**: All tests passing (123 unit + integration tests), zero unsafe blocks

✅ **CI/CD**: Full GitHub Actions pipeline with linting, testing, security scanning, multi-platform builds

✅ **Releases**: Automated semantic versioning, multi-platform binaries, .deb packages, Docker images

✅ **Operations**: Security procedures, incident response, audit logging, certificate management

✅ **Documentation**: Comprehensive guides covering architecture, operations, security, deployment

## Next Steps

For deployment guidance, see [DEPLOYMENT.md](DEPLOYMENT.md).

For security configuration, see [SECURITY.md](SECURITY.md).

For extending Dumptruck, see [DATA_FLOW_AND_EXTENSIBILITY.md](DATA_FLOW_AND_EXTENSIBILITY.md).
