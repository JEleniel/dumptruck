# Dumptruck

[![License](https://img.shields.io/badge/license-MIT%20%7C%20Apache%202.0-blue)](LICENSE.md)

**A high-performance bulk data analysis system for threat intelligence and credential breach analysis.**

Dumptruck safely processes, normalizes, and analyzes large data dumps with surgical precision. Designed for security teams analyzing credential leaks, breach datasets, and threat intelligence at scale—processing gigabytes of data in memory-efficient streaming pipelines while storing historical records only as non-reversible hashes.

---

## Why Dumptruck?

Real-world breach data is messy. Email variants, Unicode aliases, malformed records, and duplicate formats make analysis difficult. Dumptruck solves this with:

- **Smart Normalization** — Intelligently canonicalizes data across Unicode variants, email aliases, and international formats
- **Privacy-First Architecture** — Historical data is stored only as non-reversible hashes (HMAC), with secure deletion support for working files. See _Safety & Security_ below for details and limitations.
- **Cross-platform and portable** - Runs on Linux, Windows, and macOS. External dependencies are minimal and generally present on _any_ install of the OS.
- **Lightweight** - Distributed as a single executable per OS that can be run without installation.

---

## What Dumptruck Is

- A bulk data analysis tool for large breach and threat-intelligence datasets
- A normalization and tagging engine for messy, real-world records
- A privacy-first system that stores historical data only as non-reversible hashes

## What Dumptruck Is Not

- A forensic erasure tool that can guarantee secure deletion on every storage medium or filesystem
- A substitute for encryption-at-rest, access controls, or an incident response plan
- A promise of perfect detection; it is designed to be conservative and extensible, but false positives and false negatives are possible

---

## Core Features

### Data Ingestion & Processing

- **Multiple Format Support**: Support for Comma (CSV), Tab (TSV), or Pipe (PSV) Separated Value files.
- **Memory-Efficient Streaming**: Streaming processing enables handling GB/TB-scale files with minimal memory usage.
- **Safe Ingestion**: Robust binary detection and rejection, executable bit checking (on Posix systems), UTF-8 validation, and a strong focus on resilient handling of malformed input.
- **Robust Error Handling**: Errors are handled in place or logged. Panics are treated as bugs and should be reported.
- **Duplicate File Detection**: Files identified by SHA-256 hash, breach date, and target name to detect previously processed files or duplicates.
- **Real-time Learning**: As each file is processed, new entries are securely hashed and stored to the database so tagged data can be recognized later.
- **Cross-instance Sharing**: Full export and import with merge enables seeding and sharing learned data between instances.

### Normalization & Deduplication

- **Unicode Canonicalization**: NFKC normalization + ICU4X case-folding + punctuation rules for consistent comparisons
- **Smart Email Handling**: Stripping of plus addressing, canonicalization of addresses, and configurable automatic domain alias substitution (e.g., `googlemail.com` → `gmail.com`)
- **Relationship Tracking**: Automatically remembers data items that have been seen together, for example a login and phone number.
- **Vector Similarity Search**: Optional vector embeddings (768-dim Nomic vectors) for near duplicate detection.

### Intelligence & Enrichment

- **Field Identification & Documentation**: Automatically tags ID, credential (e.g., password), and NPI fields with clear reporting in both human and machine-readable formats
- **Comprehensive Threat Detection**: 40+ detection types across NPI, rainbow table lookups, weak credentials, hash formats, and anomalies
- **Risk Scoring**: Assigns 0-100 risk score based on weak password count, credential compromise potential, breach history, and anomaly contributions
- **Co-occurrence Analysis**: Graph-based tracking of address relationships and credential associations

### Detection Capabilities

**Non-Public Information (NPI) including Personally Identifiable Information (PII):**

- Account Numbers
- IBAN
- Bank Routing Numbers
- Bank SWIFT Codes
- Biometric Data
- Credit Card Numbers
- Crypto Addresses
- Date of Birth
- Email addresses
- Personal Names
- Gender Data
- GPS Locations
- IMEI Numbers
- Mailing Addresses
- National Identification Numbers, including SSN
- Other identifier formats
- Personal Identification Numbers (PIN)
- Phone Numbers

**Weak & Compromised Credentials:**

- Compromised passwords (using Rainbow Tables)
- Weak passwords (entropic analysis)
- Hash algorithm fingerprinting

**Anomaly & Novelty Detection:**

- Entropy outliers (>3σ deviation in character distribution)
- Unseen field combinations (rare or unprecedented pairings)
- Unusual credential formats (non-standard password structures)
- Length outliers (statistical deviation in field length)
- Uniform distribution detection (suspiciously uniform characters)
- Baseline deviation (statistical outliers from dataset baseline)

**Data Quality & Safety:**

- Binary file detection and rejection during ingest
- UTF-8 validation and folding
- File integrity (SHA-256 hash signatures)

### Deployment & Operations

- **CLI & Server Modes**: Standalone tool or an HTTPS API (HTTP/2-only) with TLS 1.3+ and OAuth 2.0
- **API Style**: REST endpoints with non-streaming responses
- **Flexible Output**: Human-readable or JSON
- **Optional Integrations**: Docker and Docker Compose, and Ollama (only for features that require them)

---

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

**Secure deletion notes:**

Dumptruck uses a conservative, three-pass overwrite approach intended to align with NIST SP 800-88 _Clear_ guidance. Effectiveness depends on storage medium and filesystem behavior (e.g., SSD wear leveling and copy-on-write filesystems may prevent overwrites from guaranteeing purge).

**Operational:**

- ✅ Graceful shutdown with background worker cleanup
- ✅ Signal handling for SIGTERM/SIGINT

---

## Support

- **Documentation**: See [docs/](./docs/) (in progress)
- **Design**: See [docs/design/](./docs/design/) (architecture and Aurora model)
- **Issues**: Found a bug? Open an issue with reproduction steps
- **Security**: See [SECURITY.md](SECURITY.md) for responsible disclosure

---

## Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) first.

All commits must be linked to an issue and signed. See CONTRIBUTING.md for details.

---

## License

Dumptruck is licensed under the [MIT](LICENSE-MIT.md) or [Apache 2.0](LICENSE-Apache.md) license, at the user's discretion. See [LICENSE.md](LICENSE.md) for details.
