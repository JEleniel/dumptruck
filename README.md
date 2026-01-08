# Dumptruck

**A high-performance bulk data analysis system for threat intelligence and credential breach analysis.**

Dumptruck safely processes, normalizes, and analyzes large data dumps with surgical precision. Designed for security teams analyzing credential leaks, breach datasets, and threat intelligence at scale—processing gigabytes of data in memory-efficient streaming pipelines while maintaining complete privacy of historical records through non-reversible hashing.

[![License](https://img.shields.io/badge/license-MIT%20%7C%20Apache%202.0-blue)](#license)

---

## Why Dumptruck?

Real-world breach data is messy. Email variants, Unicode aliases, malformed records, and duplicate formats make analysis difficult. Dumptruck solves this with:

- **Smart Normalization** — Intelligently canonicalizes data across Unicode variants, email aliases, and international formats
- **Privacy-First Architecture** — Historical data stored only as non-reversible hashes; zero exposure even if the database is compromised
- **Distributed Deduplication** — Peer discovery and Bloom filter sync allow multiple instances to share intelligence without data exposure
- **Production Ready** — extensive tests, zero unsafe code, comprehensive audit logging, and operational runbooks included

---

## Core Features

### Data Ingestion & Processing

- **Multiple Format Support**: CSV, TSV, PSV with an extensible adapter pattern
- **Memory-Efficient Streaming**: Process GB/TB-scale files with constant memory usage via line-by-line streaming
- **Parallel Processing**: Batch analysis with glob patterns and configurable worker threads
- **Safe Ingestion**: Binary detection, UTF-8 validation with lossy fallback, 100MB file size limits, zero-crash guarantee
- **Evidence Preservation**: Unique file IDs with SHA-256 hash signatures and alternate name tracking
- **Compression Detection**: Automatic ZIP/gzip detection with safe nested level limits (max 3 levels)

### Normalization & Deduplication

- **Unicode Canonicalization**: NFKC normalization + ICU4X case-folding + punctuation rules for consistent comparisons
- **Smart Email Handling**: Automatic domain alias substitution (e.g., `googlemail.com` → `gmail.com`)
- **Alias Resolution**: Links between entries identified across email, user IDs, and phone numbers
- **Hash-Based Deduplication**: Exact matching with SHA-256 + dual field hashing for O(1) lookups and minor-variation detection
- **Vector Similarity Search**: Ollama embeddings (768-dim Nomic vectors) with pgvector IVFFlat indexing for near-duplicate detection
- **Bloom Filter Sync**: Distributed deduplication via peer discovery and bandwidth-efficient delta sync

### Intelligence & Enrichment

- **Field Identification & Documentation**: Automatically tags ID, password, PII, and NPI fields with clear reporting
- **Comprehensive Threat Detection**: 40+ detection types across PII, weak credentials, hash formats, and anomalies
- **Risk Scoring**: Assigns 0-100 risk score based on weak password count, credential compromise potential, breach history, and anomaly contributions
- **Breach Enrichment**: Have I Been Pwned (HIBP) API integration for real-time breach data lookup; background thread continuously enriches corpus in server mode
- **Co-occurrence Analysis**: Graph-based tracking of address relationships and credential associations

#### Detection Capabilities

**PII/NPI (Personally Identifiable Information & Non-Public Information):**

- Email addresses with column name hinting
- IP addresses (IPv4 and IPv6, excluding private ranges)
- Phone numbers (10-15 digits, 15+ countries with formatting support)
- Social Security Numbers (US format: XXX-XX-XXXX)
- National IDs (15+ countries: UK National Insurance, EU formats, and more)
- Credit card numbers (standard formats with Luhn validation)
- Names (person name pattern matching)
- Mailing addresses (physical address detection)
- Bank account information (IBAN, SWIFT codes, routing numbers, account numbers)

**Cryptocurrency & Digital Assets:**

- Crypto addresses (Bitcoin, Ethereum, XRP, and other blockchain networks)
- Digital wallet tokens (Stripe, Square, PayPal, Apple Pay, Google Pay)

**Weak & Compromised Credentials:**

- Weak passwords (dictionary-based plaintext comparison)
- Weak password hashes via rainbow tables:
    + bcrypt ($2a$, $2b$, $2y$)
    + Argon2 variants ($argon2id$, $argon2i$, $argon2d$)
    + scrypt ($7$)
    + PBKDF2 ($pbkdf2-sha256$, $pbkdf2-sha512$)
    + Unsalted hashes: MD5 (32 hex), SHA1 (40 hex), SHA256 (64 hex), SHA512 (128 hex), NTLM
- Hash algorithm fingerprinting by pattern

**Anomaly & Novelty Detection:**

- Entropy outliers (>3σ deviation in character distribution)
- Unseen field combinations (rare or unprecedented pairings)
- Rare domains (infrequent top-level domains)
- Unusual credential formats (non-standard password structures)
- Length outliers (statistical deviation in field length)
- Uniform distribution detection (suspiciously uniform characters)
- Baseline deviation (statistical outliers from dataset baseline)

**Data Quality & Safety:**

- Binary file detection during ingest
- UTF-8 validation with lossy fallback
- Compression detection (ZIP/gzip with safe nesting limits)
- File integrity (SHA-256 hash signatures)
- Vector similarity search (Ollama embeddings with pgvector IVFFlat indexing)

### Chain of Custody & Security

- **Chain of Custody**: Cryptographically signed entry for each file processed (ED25519) with operator, timestamp, and audit trail for compliance
- **Secure Deletion**: Temporary files shredded (3-pass NIST SP 800-88 overwrite) to prevent data ghosting via forensic recovery
- **Privacy-First**: Historical data stored only as non-reversible hashes; zero exposure even if database is compromised

### Deployment & Operations

- **CLI & Server Modes**: Standalone tool or HTTP/2 REST API with TLS 1.3+ and OAuth 2.0
- **Flexible Output**: JSON, CSV, JSONL, or human-readable text formats with field classification documentation
- **Peer Discovery**: Automatic subnet peer detection via UDP broadcast for distributed deployments
- **Comprehensive Audit Logging**: Structured JSON logging with metadata events, Chain of Custody records, and forensic details
- **High Performance**: >800 requests/second on Raspberry Pi 5 with concurrent TLS connections

---

## Quick Start

### Prerequisites

- Rust 1.85+ (edition 2024)
- Ollama 0.1+ (optional, for similarity search)
- Docker & Docker Compose (optional, for containerized setup)

### 1. Quick Start (CLI)

```bash
# Clone and enter the repository
git clone https://github.com/JEleniel/dumptruck.git
cd dumptruck

# Run tests to verify build
cargo test

# Analyze a CSV file
cargo run -- analyze tests/fixtures/clean_csv.csv

# Start the server (defaults to HTTPS 443; use --port for local dev)
cargo run -- serve --cert /etc/tls/tls.crt --key /etc/tls/tls.key --port 8443
```

### 2. With Optional Services (Docker Compose)

```bash
# Start Ollama service (if you want vector embeddings)
cd docker/ollama
docker-compose up -d

# Run tests to verify setup
cargo test

# Analyze with embeddings enabled (requires Ollama running)
cargo run -- --embeddings --ollama-url http://localhost:11434 analyze data.csv --enable-embeddings

# Stop Ollama when done
docker-compose down -v
```

### 3. Manual Build

```bash
# Build the project
cargo build --release

# Run CLI
./target/release/dumptruck analyze data.csv --output results.json

# Run server
./target/release/dumptruck serve \
  --cert /path/to/cert.pem \
  --key /path/to/key.pem \
  --port 8443
```

### 4. Basic Usage Examples

```bash
# Single file analysis
dumptruck analyze data.csv

# Analyze a directory recursively
dumptruck analyze ./breaches --recursive

# Enable optional services
dumptruck --embeddings --ollama-url http://localhost:11434 analyze data.csv --enable-embeddings

# Generate report
dumptruck analyze data.csv --output results.json

# Check service connectivity
dumptruck status --url https://localhost:8443
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
- **Production-ready**: 240 tests, zero unsafe code, comprehensive error handling

**See [docs/architecture/ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md) for detailed system design and component documentation.**

---

## Documentation

| Guide                                                          | Purpose                                        |
| -------------------------------------------------------------- | ---------------------------------------------- |
| **[Architecture Overview](docs/architecture/ARCHITECTURE.md)** | System design, components, data flow           |
| **[CLI Usage](docs/CLI_USAGE.md)**                             | Command-line interface and examples            |
| **[Configuration](docs/CONFIGURATION.md)**                     | API keys and settings reference                |
| **[Deduplication Pipeline](docs/DEDUP_ARCHITECTURE.md)**       | Address canonicalization and matching strategy |
| **[Enrichment Pipeline](docs/ENRICHMENT.md)**                  | HIBP and Ollama integration details            |
| **[Security Operations](docs/SECURITY_OPS.md)**                | TLS, OAuth, key rotation, incident response    |
| **[Deployment Guide](docs/architecture/DEPLOYMENT.md)**        | Production deployment patterns                 |
| **[Contributing](CONTRIBUTING.md)**                            | Development guidelines and code standards      |

---

## Performance

Dumptruck is engineered for scale:

- **Throughput**: >800 concurrent requests/second on Raspberry Pi 5 with TLS 1.3
- **Memory**: Constant O(1) memory usage via streaming (100GB files in <100MB RAM)
- **Latency**: Sub-100ms response times for typical 1KB-1MB analysis jobs
- **Deduplication**: O(1) hash lookup + bandwidth-efficient Bloom filter peer sync
- **Indexing**: pgvector IVFFlat for sub-millisecond similarity search on 1M+ vectors

Run the stress test locally:

```bash
# Start the server
cargo run -- serve --cert /etc/tls/tls.crt --key /etc/tls/tls.key --port 8443 &

# Run benchmarks
cargo run --bin stress-test
# Output: 1000 requests, 100% success, 619 req/sec, p95=210ms
```

---

## Safety & Security

**Code Quality:**

- ✅ 100% safe Rust (zero `unsafe` blocks)
- ✅ Extensive unit and integration tests
- ✅ OWASP-compliant crypto and authentication
- ✅ Dependency scanning with cargo-audit (CI/CD automated)

**Data Protection:**

- ✅ TLS 1.3+ for all network transport
- ✅ OAuth 2.0 for server authentication
- ✅ Historical data stored as non-reversible HMAC hashes
- ✅ Full-disk encryption support (e.g., LUKS)
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

Dumptruck is licensed under the [MIT](LICENSE-MIT.md) or [Apache 2.0](LICENSE-Apache.md) license, at the user's discretion. See [LICENSE.md](LICENSE.md) for details.
