# Ollama/Nomic Integration

This document describes the vector embedding setup for near-duplicate detection using Ollama and Nomic embeddings.

## Overview

Dumptruck uses [Ollama](https://ollama.ai) to generate [Nomic](https://www.nomic.ai/) text embeddings (768-dimensional vectors) for address records. These embeddings enable:

- **Vector Similarity Search**: Find addresses with similar text representations (e.g., slight typos, formatting variations).
- **Near-Duplicate Detection**: Identify addresses that may be the same entity despite Unicode, case, or punctuation variations.
- **Graph-Based Deduplication**: Combined with co-occurrence tracking, identify address clusters that represent the same canonical entity.

## Setup

### Prerequisites

- Docker and Docker Compose
- Enough disk space for the Nomic model (~1.5 GB)

### Starting Ollama

```bash
# Start the Docker Compose stack (includes PostgreSQL + Ollama)
docker-compose up -d

# Verify Ollama is running
curl http://localhost:11434/api/tags

# Pull the Nomic Embed Text model
docker-compose exec ollama ollama pull nomic-embed-text:latest
```

The Ollama container will start at `http://localhost:11434` and expose the embedding API on port 11434.

The model used is **`nomic-embed-text:latest`** which generates 768-dimensional vectors optimized for semantic search.

## Usage in Code

### Generating Embeddings

```rust
use dumptruck::ollama::OllamaClient;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let client = OllamaClient::new(None, None); // Uses defaults

    // Ensure model is available
    client.ensure_model().await?;

    // Generate embedding for an address
    let embedding = client.embed("john.doe@example.com").await?;
    println!("Embedding: {:?}", embedding);

    Ok(())
}
```

### Finding Similar Addresses

```rust
use dumptruck::storage::{PostgresStorage, StorageAdapter};

let mut storage = PostgresStorage::new_from_env()?;

// Find addresses similar to a given embedding
let similar = storage.find_similar_addresses(
    &embedding_vec,
    10,           // limit
    0.85,         // similarity threshold (0-1)
)?;

for (canonical_hash, score) in similar {
    println!("Match: {} (similarity: {:.3})", canonical_hash, score);
}
```

### Deduplication Query

```rust
// Check if an address is a duplicate (by hash or vector similarity)
let maybe_duplicate = storage.find_duplicate_address(
    &new_canonical_hash,
    Some(&embedding_vec),
    0.85, // threshold
)?;

match maybe_duplicate {
    Some(existing_hash) => println!("Duplicate of: {}", existing_hash),
    None => println!("New address"),
}
```

## Endpoints

The Ollama API provides:

- `POST /api/embed` - Generate embeddings for text
    + Request: `{"model": "nomic-embed-text", "input": "text to embed"}`
    + Response: `{"embedding": [f32, f32, ...]}`

- `GET /api/tags` - List available models

- `POST /api/pull` - Download a model

## Configuration

Environment variables:

- `OLLAMA_HOST` - Ollama service endpoint (default: `http://localhost:11434`)
- `DUMPTRUCK_PG_CONN` - PostgreSQL connection (default: `postgresql://dumptruck:dumpturck@dumptruck-db/dumptruck`)

## Performance Notes

- **First embedding call**: ~1-2 seconds (model load time)
- **Subsequent calls**: ~100-200 ms per address
- **Batch operations**: Use concurrent requests to maximize throughput
- **Storage**: Vector index is IVFFlat with `lists=100` for ~1M addresses; consider tuning for your scale

## Similarity Threshold

The cosine similarity threshold (0-1 scale) determines how similar two addresses must be:

- `0.95+`: Very strict (only near-identical addresses)
- `0.85-0.95`: Moderate (catches most duplicates with minor variations)
- `0.75-0.85`: Loose (catches addresses with significant differences)

Start with `0.85` and adjust based on false positive/negative rates.

## Troubleshooting

### Ollama service not responding

```bash
# Check logs
docker-compose logs ollama

# Restart the service
docker-compose restart ollama
```

### Model not available

```bash
# Check available models
curl http://localhost:11434/api/tags

# Pull the model explicitly
docker-compose exec ollama ollama pull nomic-embed-text
```

### Embedding dimensions mismatch

Ensure you're using `nomic-embed-text` which generates 768-dim vectors. The PostgreSQL schema expects this exact dimension:

```sql
SELECT dimension(embedding) FROM canonical_addresses LIMIT 1;
```

## See Also

- [Nomic Documentation](https://www.nomic.ai/)
- [Ollama GitHub](https://github.com/ollama/ollama)
- [pgvector Documentation](https://github.com/pgvector/pgvector)
