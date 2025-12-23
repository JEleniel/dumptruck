# Docker Configuration

This folder contains Docker and Docker Compose configurations for running Dumptruck's optional services.

## Folder Structure

```text
docker/
├── ollama/            # Ollama (Nomic embeddings) - optional
│   ├── docker-compose.yml
│   ├── Dockerfile
│   ├── nvidia.sh
│   └── README.md
└── README.md
```

## Quick Start

### Start Ollama (Optional)

Ollama is only needed if you want to enable vector embeddings for near-duplicate detection:

```bash
cd docker/ollama
docker compose up -d
```

## Services

- **Ollama** (`docker/ollama/`): Optional embedding service (Nomic) for vector similarity search
    + Port: 11435
    + Model: nomic-embed-text:latest (must be pulled separately)

## Storage

Dumptruck uses **SQLite** as the default storage backend for all data. The database file (`dumptruck.db`) is created automatically on first use.

## Pulling Models

After starting Ollama, pull the embedding model:

```bash
docker exec dumptruck-ollama ollama pull nomic-embed-text:latest
```

## Verification

Verify Ollama:

```bash
curl http://localhost:11435/api/tags
```

## Cleanup

Stop and remove containers:

```bash
cd docker/ollama && docker compose down -v
```

## Production Deployment

For production use, consider:

1. Pre-built images for faster startup
2. Persistent volume configuration
3. Network security policies
4. Backup and recovery procedures

See [docs/architecture/DEPLOYMENT.md](../docs/architecture/DEPLOYMENT.md) for guidance.
