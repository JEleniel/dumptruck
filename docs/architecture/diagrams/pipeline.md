# Pipeline Diagram

The following mermaid diagram shows the primary data pipeline stages.

```mermaid
flowchart LR
  Sources["📁 Files & Streams"] --> Ingest["Ingest<br/>(Format Detection)"]
  Ingest --> Safe["Safe Ingest<br/>(Validation)"]
  Safe --> Normalization["Normalization<br/>(Canonicalization)"]
  Normalization --> Dedup["Deduplication<br/>(Vector + Hash)"]
  Dedup --> Enrichment["Enrichment<br/>(Embeddings, HIBP)"]
  Enrichment --> Analysis["Analysis<br/>(PII Detection)"]
  Analysis --> Storage["🗄️ Storage<br/>(PostgreSQL)"]

  subgraph Interfaces
    CLI["⌨️ CLI"] 
    API["🌐 REST API"]
  end

  CLI --> Ingest
  API --> Ingest

  subgraph Observability
    Metrics["📊 Metrics"]
    Logs["📝 Logs"]
    Tracing["🔍 Tracing"]
  end

  Ingest -->|events| Metrics
  Analysis -->|events| Metrics
  Ingest --> Logs
  Analysis --> Logs
  Enrichment --> Tracing
```

## Pipeline Stages

1. **Ingest**: Format detection and streaming read
2. **Safe Ingest**: Binary detection, UTF-8 validation, size limits
3. **Normalization**: Unicode NFKC + case-folding + punctuation normalization
4. **Deduplication**: Hash-based exact match + vector similarity (pgvector/IVFFlat)
5. **Enrichment**: Ollama embeddings + HIBP breach data lookup
6. **Analysis**: PII/NPI detection (phones, SSNs, credit cards, crypto addresses, etc.)
7. **Storage**: PostgreSQL with address deduplication graph and breach metadata

## Key Design Principles

- **Stream-oriented**: Operate record-by-record to limit memory use (constant memory for GB/TB files)
- **Privacy-preserving**: Historical data stored as non-reversible hashes
- **Error-resilient**: Malformed rows logged but processing continues (zero-crash guarantee)
- **Observable**: All stages emit metrics and structured logs
