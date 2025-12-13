# Dumptruck

## Overview

Dumptruck is a bulk data analysis tool for Cyber Threat Identification. It provides a set of functions to ingest, normalize, enrich, and analyze bulk data dumps (e.g. credential leaks) as well as tracking historic information to help identify repeated and new leaked data.

## Features

- Support for common data formats, including CSV, TSV, JSON, YAML, XML, Protocol Buffers, BSON, and more. The ingestion pipeline is designed to be extensible.
- Dumptruck can be run as a command line tool against a file, or as a web server providing an analysis endpoint.
- By normalizing the format of incoming data, applying additional rules (e.g. trim, substitution) any two data sets can be compared on even terms.
- Dumptruck can recognize having seen the same value in the same type of field, the same dump in its entirety, the same credential associated with an ID, and other similar enrichment.
- Historic data is stored as hashes and similar, so even a leak of the complete database provides nothing to a threat actor.
- Server mode supports OAuth 2.0/OIDC for authentication, TLS 1.3+ for encryption, and other common security.
- **High Performance**: Achieves >800 requests/second on a Raspberry Pi 5 with concurrent TLS 1.3 connections.

## Documentation

Our documentation is organized into the following sections:

- **[Pipeline Diagram](docs/PIPELINE_DIAGRAM.md)**: Complete visual overview of the entire data processing pipeline
- **[Configuration](docs/CONFIGURATION.md)**: API keys and email suffix substitution rules
- **[Email Suffix Substitution](docs/EMAIL_SUFFIX_SUBSTITUTION.md)**: How email domain aliases map to canonical forms for deduplication
- **[Architecture](docs/architecture/)**: System design, components, data flow, deployment, and security
- **[Deduplication System](docs/DEDUP_ARCHITECTURE.md)**: Complete address deduplication pipeline with Unicode normalization, canonical tracking, and vector similarity
- **[Enrichment Pipeline](docs/ENRICHMENT.md)**: Address enrichment workflow with Ollama embeddings and HIBP breach data
- **[Ollama Integration](docs/OLLAMA.md)**: Setup and usage of Nomic embeddings for near-duplicate detection
- **[Vector Deduplication](docs/VECTOR_DEDUP.md)**: Implementation details of pgvector similarity search
- **[HIBP Integration](docs/HIBP.md)**: Have I Been Pwned API for breach data enrichment
- **API Reference**: Technical documentation for the ingestion endpoint in Server mode
- **User Guide**: Instructions for users

## Getting Started

### Quick Start with Docker Compose

```bash
# Start PostgreSQL + Ollama stack
docker-compose up -d

# Build and run tests
cargo test

# Run the application
cargo run
```

### Manual Setup

See [Deduplication Architecture](docs/DEDUP_ARCHITECTURE.md) for detailed configuration and workflow documentation.

## Support

[Getting Support](SUPPORT.md)

## Feedback and Contributions

Please be sure to read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

[Reporting Security Issues](SECURITY.md)

[Contributing to the Project](CONTRIBUTING.md)

## Building and Testing

<!-- COPILOT add build and test instructions here -->

## License

This project is released under the [MIT](LICENSE-MIT.md) or [Apache 2.0](LICENSE-Apache.md) license, at the users discretion.
