# Pipeline Diagram

The following mermaid diagram shows the primary data pipeline stages. Labels use single-line identifiers so the diagram validates and renders reliably.

```mermaid
flowchart LR
  Sources[Files_and_Streams] --> Ingest[Ingest]
  Ingest --> Normalization[Normalization]
  Normalization --> Enrichment[Enrichment]
  Enrichment --> Analysis[Analysis]
  Analysis --> Storage_History[Storage_and_History]

  subgraph Interfaces
    CLI[CLI]
    API[REST_API_Server]
  end

  CLI --> Ingest
  API --> Ingest

  subgraph Observability
    Metrics[Metrics]
    Logs[Logging]
    Tracing[Tracing]
  end

  Ingest -->|events| Metrics
  Analysis -->|events| Metrics
  Ingest --> Logs
  Analysis --> Logs
  Analysis --> Tracing

  
```

Notes

- Stages are stream-oriented; where possible, components should operate record-by-record to limit memory use.
