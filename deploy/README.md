# Postgres + Apache AGE + pgvector for Dumptruck

This folder contains a Docker build and a `docker-compose.yml` to run a Postgres
instance with `pgvector` and Apache `AGE` extensions installed. It is intended
for local development and testing of storage + graph/enrichment features.

Notes

- Building the image will compile native extensions; the build may take several
  minutes depending on your machine and network.
- The image is based on `postgres:15` and uses the system package manager to
  install build dependencies before compiling `pgvector` and Apache AGE.

Start the services

```bash
cd deploy
docker compose up --build -d
```

Verify

```bash
# psql into the running container
docker exec -it dumptruck-db psql -U dumptruck -d dumptruck

-- then in psql:
\dx                 -- list installed extensions
SELECT * FROM normalized_rows LIMIT 1;
```

Cleanup

```bash
docker compose down -v
```

Compatibility

- The Dockerfile performs source builds; if you need prebuilt binaries or a
  lighter image, consider using community images that bundle `pgvector` and AGE.
