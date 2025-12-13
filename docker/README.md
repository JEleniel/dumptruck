# Docker Configuration

This folder contains Docker and Docker Compose configurations for running Dumptruck's infrastructure.

## Folder Structure

```text
docker/
├── postgres/          # PostgreSQL + pgvector + Apache AGE
│   ├── Dockerfile
│   ├── docker-compose.yml
│   ├── init-db.sql
│   └── README.md
├── ollama/            # Ollama (Nomic embeddings)
│   ├── docker-compose.yml
│   └── README.md
└── README.md
```

## Quick Start

### Option 1: Use Root docker-compose.yml (Recommended)

Start both PostgreSQL and Ollama together:

```bash
docker compose up -d
```

### Option 2: Run Services Independently

Start PostgreSQL only:

```bash
cd docker/postgres
docker compose up --build -d
```

Start Ollama only:

```bash
cd docker/ollama
docker compose up -d
```

## Services

- **PostgreSQL** (`docker/postgres/`): Database with pgvector and Apache AGE
    + Port: 5432
    + User: dumptruck
    + Database: dumptruck

- **Ollama** (`docker/ollama/`): Embedding service (Nomic)
    + Port: 11434
    + Model: nomic-embed-text:latest (must be pulled separately)

## Database Schema

The initialization script (`docker/postgres/init-db.sql`) creates:

1. `canonical_addresses` - Primary deduplication key with embeddings
2. `address_alternates` - Unicode variant mappings
3. `address_credentials` - Credential associations
4. `address_cooccurrence` - Graph edges (addresses seen together)
5. `address_breaches` - HIBP breach enrichment data
6. `normalized_rows` - Raw normalized input data

See [docker/postgres/init-db.sql](postgres/init-db.sql) for full schema.

## Pulling Models

After starting Ollama, pull the embedding model:

```bash
docker exec dumptruck-ollama ollama pull nomic-embed-text:latest
```

## Verification

Verify PostgreSQL:

```bash
docker exec -it dumptruck-db psql -U dumptruck -d dumptruck
```

Verify Ollama:

```bash
curl http://localhost:11434/api/tags
```

## Cleanup

Stop and remove all containers and volumes:

```bash
docker compose down -v
```

Or individually:

```bash
cd docker/postgres && docker compose down -v
cd docker/ollama && docker compose down -v
```

## Production Deployment

For production use, consider:

1. Using managed database services (AWS RDS, Google Cloud SQL)
2. Pre-built images for faster startup
3. Persistent volume configuration
4. Network security policies
5. Backup and recovery procedures

See [docs/architecture/DEPLOYMENT.md](../docs/architecture/DEPLOYMENT.md) for guidance.
