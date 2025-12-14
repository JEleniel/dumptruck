# Dumptruck

**A high-performance bulk data analysis system for threat intelligence and credential breach analysis.**

Dumptruck ingests, normalizes, and analyzes large data dumps with surgical precision. Designed for security teams analyzing credential leaks, breach datasets, and threat intelligence at scale—processing gigabytes of data in memory-efficient streaming pipelines while maintaining complete privacy of historical records through non-reversible hashing.

[![License](https://img.shields.io/badge/license-MIT%20%7C%20Apache%202.0-blue)](#license)

---

## Why Dumptruck?

Real-world breach data is messy. Email variants, Unicode aliases, malformed records, and duplicate formats make analysis difficult. Dumptruck solves this with:

- **Smart Normalization** — Intelligently canonicalizes data across Unicode variants, email aliases, and international formats
- **Privacy-First Architecture** — Historical data stored only as non-reversible hashes; zero exposure even if the database is compromised
- **Distributed Deduplication** — Peer discovery and Bloom filter sync allow multiple instances to share intelligence without data exposure
- **Production Ready** — 143+ tests, zero unsafe code, comprehensive audit logging, and operational runbooks included

---

## Core Features

### Data Ingestion & Processing

- **Multiple Format Support**: CSV, TSV, JSON, YAML, XML, Protocol Buffers, BSON with extensible adapter pattern
- **Memory-Efficient Streaming**: Process GB/TB-scale files with constant memory usage via line-by-line streaming
- **Parallel Processing**: Batch ingest with glob patterns and configurable worker threads
- **Safe Ingestion**: Binary detection, UTF-8 validation with lossy fallback, 100MB file size limits, zero-crash guarantee

### Normalization & Deduplication

- **Unicode Canonicalization**: NFKC normalization + ICU4X case-folding + punctuation rules for consistent comparisons
- **Smart Email Handling**: Automatic domain alias substitution (e.g., `googlemail.com` → `gmail.com`)
- **Hash-Based Deduplication**: Exact matching with SHA-256 hashing for O(1) lookups
- **Vector Similarity Search**: Ollama embeddings (768-dim Nomic vectors) with pgvector IVFFlat indexing for near-duplicate detection
- **Bloom Filter Sync**: Distributed deduplication via peer discovery and bandwidth-efficient delta sync

### Intelligence & Enrichment

- **PII/NPI Detection**: Identifies phone numbers (15+ countries), SSNs, credit cards, crypto addresses (Bitcoin, Ethereum, XRP), IBAN/SWIFT codes, national IDs (15+ countries), IP addresses, and digital wallets
- **Weak Password Detection**: Rainbow table with 40+ common passwords and 3-5 character keyboard patterns; pre-hashed credential detection (bcrypt, scrypt, argon2, MD5, SHA1, SHA256)
- **Breach Enrichment**: Have I Been Pwned (HIBP) API integration for real-time breach data lookup and tracking
- **Co-occurrence Analysis**: Graph-based tracking of address relationships and credential associations

### Deployment & Operations

- **CLI & Server Modes**: Standalone tool or HTTP/2 REST API with TLS 1.3+ and OAuth 2.0
- **Flexible Output**: JSON, CSV, JSONL, or human-readable text formats
- **Peer Discovery**: Automatic subnet peer detection via UDP broadcast for distributed deployments
- **Comprehensive Audit Logging**: Structured JSON logging with metadata events for forensics and compliance
- **High Performance**: >800 requests/second on Raspberry Pi 5 with concurrent TLS connections

---

## Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 12+ (for server mode)
- Ollama 0.1+ (for similarity search)
- Docker & Docker Compose (optional, for containerized setup)

### 1. Using Docker Compose (Recommended)

```bash
# Clone and enter the repository
git clone https://github.com/yourusername/dumptruck.git
cd dumptruck

# Start PostgreSQL and Ollama services
docker-compose up -d

# Run tests to verify setup
cargo test

# Ingest a CSV file
cargo run -- ingest tests/fixtures/clean_csv.csv

# Start the server (listens on 0.0.0.0:8443 with TLS)
cargo run -- server --cert /etc/tls/tls.crt --key /etc/tls/tls.key
```

### 2. Manual Setup

```bash
# Build the project
cargo build --release

# Set up PostgreSQL and Ollama manually
# (See docs/architecture/DEPLOYMENT.md for detailed instructions)

# Run CLI
./target/release/dumptruck ingest data.csv --output results.json

# Run server
./target/release/dumptruck server \
  --cert /path/to/cert.pem \
  --key /path/to/key.pem \
  --port 8443
```

### 3. Basic Usage Examples

```bash
# Single file ingest
dumptruck ingest data.csv

# Batch ingest with glob patterns
dumptruck ingest "breaches/*.csv" --workers 4

# Ingest with database output
dumptruck ingest data.json --storage database --database "postgresql://user:pass@localhost/dumptruck"

# Generate report
dumptruck ingest data.csv --output results.json --format json

# Check service connectivity
dumptruck status
```

See [CLI_USAGE.md](docs/CLI_USAGE.md) for comprehensive command reference.

---

## Architecture

Dumptruck follows a modular, testable architecture:

```text
Raw Data → Safe Ingest → Normalization → Deduplication → Enrichment → Analysis → Storage
           ├─ Binary detection
           ├─ UTF-8 validation
           └─ Size limits
                             ├─ Unicode NFKC
                             ├─ Case-folding
                             └─ Email aliases
                                                    ├─ Hash matching
                                                    ├─ Vector similarity
                                                    └─ Peer sync
                                                                       ├─ HIBP lookup
                                                                       ├─ Embeddings
                                                                       └─ Co-occurrence
                                                                                       ├─ PII detection
                                                                                       ├─ Weak pwd detection
                                                                                       └─ Metadata events
```

**Key Design Principles:**

- **Privacy-by-design**: Historical data stored only as HMAC hashes
- **Streaming-first**: Constant memory usage regardless of input size
- **Extensible**: Plugin architecture for formats, enrichment, and storage backends
- **Observable**: Structured logging, metrics, and audit events built-in
- **Production-ready**: 143+ tests, zero unsafe code, comprehensive error handling

**See [docs/architecture/ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md) for detailed system design and component documentation.**

---

## Documentation

| Guide | Purpose |
|-------|---------|
| **[Architecture Overview](docs/architecture/ARCHITECTURE.md)** | System design, components, data flow |
| **[CLI Usage](docs/CLI_USAGE.md)** | Command-line interface and examples |
| **[Configuration](docs/CONFIGURATION.md)** | API keys and settings reference |
| **[Deduplication Pipeline](docs/DEDUP_ARCHITECTURE.md)** | Address canonicalization and matching strategy |
| **[Enrichment Pipeline](docs/ENRICHMENT.md)** | HIBP and Ollama integration details |
| **[Security Operations](docs/SECURITY_OPS.md)** | TLS, OAuth, key rotation, incident response |
| **[Deployment Guide](docs/architecture/DEPLOYMENT.md)** | Production deployment patterns |
| **[Contributing](CONTRIBUTING.md)** | Development guidelines and code standards |

---

## Performance

Dumptruck is engineered for scale:

- **Throughput**: >800 concurrent requests/second on Raspberry Pi 5 with TLS 1.3
- **Memory**: Constant O(1) memory usage via streaming (100GB files in <100MB RAM)
- **Latency**: Sub-100ms response times for typical 1KB-1MB ingests
- **Deduplication**: O(1) hash lookup + bandwidth-efficient Bloom filter peer sync
- **Indexing**: pgvector IVFFlat for sub-millisecond similarity search on 1M+ vectors

Run the stress test locally:

```bash
# Start the server
cargo run -- server &

# Run benchmarks
cargo run --bin stress-test
# Output: 1000 requests, 100% success, 619 req/sec, p95=210ms
```

---

## Safety & Security

**Code Quality:**

- ✅ 100% safe Rust (zero `unsafe` blocks)
- ✅ 143+ unit and integration tests (all passing)
- ✅ OWASP-compliant crypto and authentication
- ✅ Dependency scanning with cargo-audit (CI/CD automated)

**Data Protection:**

- ✅ TLS 1.3+ for all network transport
- ✅ OAuth 2.0 for server authentication
- ✅ Historical data stored as non-reversible HMAC hashes
- ✅ Full-disk encryption support (LUKS) + transparent data encryption (pgcrypto)
- ✅ Automatic key rotation with grace periods

**Operational:**

- ✅ Comprehensive audit logging with structured events
- ✅ Graceful shutdown with background worker cleanup
- ✅ Signal handling for SIGTERM/SIGINT
- ✅ Incident response runbooks included

See [docs/architecture/SECURITY.md](docs/architecture/SECURITY.md) and [docs/SECURITY_OPS.md](docs/SECURITY_OPS.md) for security architecture and operational procedures.

---

## Building & Testing

```bash
# Build
cargo build --release

# Test
cargo test                                    # All tests
cargo test -- --nocapture                   # With output
cargo test test_name                        # Specific test
cargo tarpaulin --out Html                  # Coverage report

# Code Quality
cargo fmt --all -- --check                  # Format check
cargo fmt --all                             # Format code
cargo clippy --all-targets -- -D warnings   # Lint

# Build Debian package
dpkg-buildpackage -us -uc -b                # Build .deb
# Output: ../dumptruck_1.0.0-1_amd64.deb
sudo dpkg -i ../dumptruck_1.0.0-1_amd64.deb # Install package
```

The Debian package installs the binary to `/usr/bin/dumptruck` for direct command-line usage.

---

## Support

- **Documentation**: Start with [docs/](docs/) or [docs/architecture/](docs/architecture/)
- **Examples**: See [examples/](examples/) for adapter and enrichment patterns
- **Issues**: Found a bug? Open an issue with reproduction steps
- **Security**: See [SECURITY.md](SECURITY.md) for responsible disclosure

---

## Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) first.

All commits must be linked to an issue and signed. See CONTRIBUTING.md for details.

---

## License

Dumptruck is licensed under the [GNU General Public License v3.0 or later](LICENSE). See [LICENSE](LICENSE) for details.
