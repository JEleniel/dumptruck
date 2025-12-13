# PostgreSQL + pgvector + Apache AGE

This folder contains Docker configuration for running PostgreSQL with pgvector and Apache AGE extensions.

## Quick Start

```bash
cd postgres
docker compose up --build -d
```

## Contents

- **`Dockerfile`** - PostgreSQL + pgvector + Apache AGE build
- **`docker-compose.yml`** - Service configuration
- **`init-db.sql`** - Database schema initialization

## Verification

```bash
docker exec -it dumptruck-db psql -U dumptruck -d dumptruck

-- In psql:
\dx                 -- list installed extensions
SELECT * FROM canonical_addresses LIMIT 1;
```

## Cleanup

```bash
docker compose down -v
```

## Configuration

- **User**: `dumptruck`
- **Password**: `dumptruck`
- **Database**: `dumptruck`
- **Port**: `5432`
