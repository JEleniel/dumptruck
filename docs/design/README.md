# Design Documentation Index

Cross-reference guide for DumpTruck design and architecture documentation.

## Root Driver

Dumptruck is a tool designed to work both standalone and as a HTTPS server that ingests, analyzes, enriches, and reports on data dumps (e.g. "lkeaks"), while also learning from each to improve future detection. It is also designed to automatically identify and trade data with other instances running on the same network.

---

## Core Design Documents

### Capabilities.md

**Purpose**: Detailed normalization requirements and validation specifications

**Sections**:

- Overview: 8-layer normalization pipeline
- 1. Data Ingestion & Format Detection
- 1. Structural Normalization (fields, values, encoding, non-semantic exclusion)
- 1. Record Ordering & Determinism
- 1. Deduplication & Identity Resolution
- 1. Enrichment & Intelligence
- 1. Output Serialization & Comparison (5 comparison strategies)
- 1. Normalization Pipeline Summary + Implementation Status

**Use**: Specification for what DumpTruck does and how it validates data

---

### PIPELINE_MAP.md

**Purpose**: Complete ingestion and analysis pipeline synthesis combining all design docs

**Sections**:

- Pipeline Overview (8-stage diagram)
- Stage 1: Ingest & Format Detection
- Stage 2: Safe Ingest & Validation
- Stage 3: Structural Normalization
- Stage 4: Deduplication & Identity Resolution
- Stage 5: Enrichment & Intelligence
- Stage 6: Intelligence & Analysis
- Stage 7: Storage & Persistence
- Stage 8: Output & Reporting
- Comparison Strategies (5 strategies with use cases)
- Execution Modes (CLI vs Server)
- Error Resilience
- Performance Metrics
- Testing Coverage
- Configuration
- Extensibility Points

**Use**: Unified reference showing complete data flow, decisions, and storage

---

## Detailed Design Documents

### DEDUP_ARCHITECTURE.md

**Purpose**: Address deduplication system design

**Coverage**:

- System architecture diagram
- Components (normalization, hash computation, deduplication, storage)
- Database schema for canonical addresses
- Hash matching and vector similarity
- Unicode variants tracking
- Confidence scoring
- Graph-based co-occurrence

**Links to PIPELINE_MAP**: Stage 4 (Deduplication & Identity Resolution)

---

### ENRICHMENT.md

**Purpose**: Address enrichment pipeline design

**Coverage**:

- Enrichment architecture diagram
- Vector embeddings (768-dim Nomic)
- HIBP API integration
- Co-occurrence graph building
- Database schema for breach data
- Performance characteristics
- Rate limiting and caching

**Links to PIPELINE_MAP**: Stage 5 (Enrichment & Intelligence)

---

## Architecture Documents

### ARCHITECTURE.md

**Purpose**: System-level architecture overview

**Sections**:

- Purpose and Goals
- Data Pipeline (7 stages)
- Component Architecture
- Deployment Architecture
- High-level System Context
- Key Non-Functional Requirements
- Primary Runtime Modes (CLI, Server)
- Design Principles
- Constraints and Assumptions
- Key Components (API Server, Pipeline, Storage, Enrichment, Detection)

**Links**: Provides context for all pipeline stages

---

### COMPONENTS.md

**Purpose**: Component-level architecture details

**Coverage**:

- Format Adapters (CSV, JSON, YAML, XML, Protobuf, BSON)
- Safe Ingest Module
- Normalization Module
- Deduplication Module
- Enrichment Module
- Storage Adapters (PostgreSQL, Filesystem)
- Detection Modules (PII, weak passwords)

**Use**: Deep dive into individual components

---

### DATA_FLOW_AND_EXTENSIBILITY.md

**Purpose**: Data flow through pipeline and plugin architecture

**Coverage**:

- Data flow diagram
- Format adapter plugin interface
- Enrichment plugin interface
- Storage adapter plugin interface
- Extension points and patterns

**Use**: Understanding how to extend DumpTruck with custom formats/enrichers/storage

---

### INTERFACES.md

**Purpose**: Public API and CLI specifications

**Coverage**:

- REST API endpoints (POST /api/v1/ingest, GET /api/v1/status)
- Request/response schemas
- CLI commands and flags
- Configuration options
- Error codes and responses

**Use**: Implementing clients or integrations

---

### DEPLOYMENT.md

**Purpose**: Production deployment patterns

**Coverage**:

- Container orchestration
- Database setup (PostgreSQL with pgvector)
- TLS/SSL configuration
- OAuth 2.0 setup
- Horizontal scaling
- Monitoring and logging
- Secrets management
- High availability patterns

**Use**: Deploying DumpTruck to production

---

### SECURITY.md

**Purpose**: Security architecture and operational procedures

**Coverage**:

- Threat model
- Cryptographic standards (TLS 1.3, OAuth 2.0)
- Data protection (hashing, encryption)
- Access control (OAuth scopes)
- Key rotation
- Audit logging
- Incident response runbooks

**Use**: Security planning and operations

---

## Related Documentation

### README.md

**Purpose**: Project overview and quick start

**Coverage**:

- Why DumpTruck (smart normalization, privacy-first)
- Core Features
- Quick Start
- Architecture Summary
- Performance Benchmarks
- Safety & Security
- Building & Testing

**Use**: Getting started and understanding features

---

### docs/CLI_USAGE.md

**Purpose**: Command-line interface reference

**Coverage**:

- Command syntax
- Global flags
- Subcommands (ingest, server, status)
- Examples
- Configuration file reference

**Use**: CLI command reference

---

### docs/CONFIGURATION.md

**Purpose**: Configuration file specification

**Coverage**:

- Configuration file format (JSON)
- All settings with defaults
- Environment variable overrides
- Secrets management
- API key configuration

**Use**: Configuring DumpTruck

---

### docs/PEER_SYNC_DESIGN.md

**Purpose**: Distributed deduplication via peer discovery

**Coverage**:

- Peer discovery via UDP broadcast
- Bloom filter synchronization
- Delta sync algorithm
- Network topology assumptions
- Failure modes and recovery

**Use**: Multi-instance deployments

---

### docs/HIBP_IMPLEMENTATION.md

**Purpose**: Have I Been Pwned API integration

**Coverage**:

- API v3 specification
- Rate limiting
- Response parsing
- Caching strategy
- Error handling

**Use**: Understanding breach lookup integration

---

### docs/VECTOR_DEDUP.md

**Purpose**: Vector-based similarity detection

**Coverage**:

- Embedding generation (Nomic/Ollama)
- Similarity threshold (0.85)
- pgvector IVFFlat indexing
- Performance characteristics
- Tuning parameters

**Use**: Understanding near-duplicate detection

---

## Design Feature Cards

### docs/design/FEATURE_CARDS/

Individual feature specifications:

- **ingestion.md** — Multi-format ingestion, streaming, safe handling
- **storage.md** — PostgreSQL backend, pgvector, schema design
- **analysis.md** — PII detection, weak password detection, co-occurrence
- **history_privacy.md** — Non-reversible hashing, historical data protection
- **extensibility.md** — Plugin architecture, adapter patterns
- **server_modes.md** — CLI and Server mode specifications

**Use**: Individual feature deep dives

---

## Document Relationships

```
┌─────────────────────┐
│   README.md         │
│  (Overview)         │
└──────────┬──────────┘
           │
    ┌──────┴──────────────────────────────────────┐
    │                                              │
    ▼                                              ▼
┌──────────────────┐                      ┌────────────────────┐
│ ARCHITECTURE.md  │                      │  CLI_USAGE.md      │
│  (System View)   │                      │  (User Guide)      │
└────────┬─────────┘                      └────────────────────┘
         │
    ┌────┴─────────────────────────┐
    │                              │
    ▼                              ▼
┌──────────────────┐       ┌──────────────────┐
│ Capabilities.md  │       │ PIPELINE_MAP.md  │
│  (What & Why)    │       │  (How & Where)   │
└──────────────────┘       └────────┬─────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    ▼               ▼               ▼
            ┌─────────────────┐  ┌──────────────┐ ┌──────────────┐
            │ DEDUP_          │  │ ENRICHMENT.  │ │ COMPONENTS.  │
            │ ARCHITECTURE.md │  │ md           │ │ md           │
            └─────────────────┘  └──────────────┘ └──────────────┘
                    │               │               │
                    └───────────────┼───────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    ▼               ▼               ▼
            ┌──────────────────┐ ┌───────────┐ ┌──────────────┐
            │ DEPLOYMENT.md    │ │ SECURITY. │ │ DATA_FLOW.md │
            │  (Operations)    │ │ md        │ │ (Plugins)    │
            └──────────────────┘ └───────────┘ └──────────────┘
```

---

## Document Selection Guide

**I want to understand**... **read**:

- What DumpTruck does → README.md
- How to use the CLI → CLI_USAGE.md
- How to configure it → CONFIGURATION.md
- Overall system design → ARCHITECTURE.md + COMPONENTS.md
- The complete pipeline → PIPELINE_MAP.md
- What normalization does → Capabilities.md
- How deduplication works → DEDUP_ARCHITECTURE.md
- How enrichment works → ENRICHMENT.md
- How to extend it → DATA_FLOW_AND_EXTENSIBILITY.md, FEATURE_CARDS/
- How to deploy → DEPLOYMENT.md
- Security considerations → SECURITY.md
- Distributed deployments → PEER_SYNC_DESIGN.md
- API integration → INTERFACES.md, HIBP_IMPLEMENTATION.md
- Vector similarity → VECTOR_DEDUP.md

---

## Maintenance Notes

**Keep synchronized**:

- Capabilities.md ↔ PIPELINE_MAP.md (stage descriptions)
- DEDUP_ARCHITECTURE.md ↔ COMPONENTS.md (dedup module)
- ENRICHMENT.md ↔ COMPONENTS.md (enrichment module)
- ARCHITECTURE.md ↔ All docs (component references)

**Update when**:

- Adding new format adapter → COMPONENTS.md, DATA_FLOW_AND_EXTENSIBILITY.md
- Adding new enrichment source → ENRICHMENT.md, COMPONENTS.md
- Changing database schema → DEDUP_ARCHITECTURE.md, ENRICHMENT.md, DEPLOYMENT.md
- Changing API endpoints → INTERFACES.md, CLI_USAGE.md
- Changing configuration → CONFIGURATION.md, COMPONENTS.md
