# PostgreSQL + Apache AGE + pgvector

This folder contains Docker configuration for running PostgreSQL with Apache AGE (graph database) and pgvector (vector similarity search) extensions.

## Quick Start

```bash
cd postgres
docker compose up --build -d
```

## Contents

- **`Dockerfile`** - PostgreSQL with Apache AGE and pgvector extensions
- **`docker-compose.yml`** - Service configuration with persistent data volume
- **`init-db.sql`** - Database schema initialization with AGE graph and vector tables

## Features

### Apache AGE

Apache AGE is a PostgreSQL extension that provides graph database functionality. The initialization script:

- Enables the `age` extension
- Creates a graph named `dumptruck` for storing relationships

### pgvector

The pgvector extension enables fast vector similarity search, used for:

- Storing embeddings from the Nomic embedding model (768 dimensions)
- Efficient similarity-based deduplication of addresses
- Cosine similarity indexes (IVFFlat) for near-duplicate detection

## Verification

Connect to the database and verify extensions:

```bash
docker exec -it dumptruck-db psql -U dumptruck -d dumptruck
```

In psql, run:

```sql
-- List all installed extensions
\dx

-- Verify Apache AGE graph was created
SELECT * FROM ag_catalog.ag_graph WHERE name = 'dumptruck';

-- Verify pgvector schema
SELECT * FROM canonical_addresses LIMIT 1;
```

Then exit with `\q`.

## Data Persistence

Database files are stored in a named volume (`postgres-data`), which persists across container restarts.

To completely reset the database:

```bash
docker compose down -v
docker compose up --build -d
```

## Configuration

- **User**: `dumptruck`
- **Password**: `dumptruck`
- **Database**: `dumptruck`
- **Port**: `5432`

## Troubleshooting

### Extension Load Failures

If the pgvector extension doesn't load during build, check that:

1. PostgreSQL version matches the dev headers (`postgresql-server-dev-17`)
2. Build dependencies are installed
3. The pgvector source compiles successfully

### Extensions Not Available

Verify the extensions were created in the database:

```bash
docker exec -it dumptruck-db psql -U dumptruck -d dumptruck -c '\dx'
```

Expected extensions: `age`, `pgvector`, `plpgsql`
