# Ollama (Nomic Embeddings)

This folder contains Docker configuration for running Ollama with Nomic embeddings.

## Quick Start

```bash
cd ollama
docker compose up -d
```

## Contents

- **`docker-compose.yml`** - Ollama service configuration

## Pulling Models

Once running, pull the nomic-embed-text model:

```bash
docker exec dumptruck-ollama ollama pull nomic-embed-text:latest
```

## Verification

```bash
curl http://localhost:11434/api/tags
```

## Cleanup

```bash
docker compose down -v
```

## Configuration

- **Container**: `dumptruck-ollama`
- **Port**: `11434`
- **Environment**: CPU-only (set `OLLAMA_HOST` for GPU if available)
- **Models**: Store in `ollama-data` volume

## Usage

The Ollama API is available at `http://localhost:11434` for embedding requests.
