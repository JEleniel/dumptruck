# Ollama (Nomic Embeddings)

This folder contains Docker configuration for running Ollama with Nomic embeddings.

## Quick Start

```bash
cd ollama
docker compose up -d
```

## Contents

- **`docker-compose.yml`** - Ollama service configuration
- **`Dockerfile`** - Ollama image with pre-loaded models
- **`nvidia.sh`** - Setup script for NVidia GPU support

## Pre-loaded Models

The Dockerfile automatically pulls the following models during build:

- `nomic-embed-text` - Nomic embedding model
- `deepseek-r1` - DeepSeek reasoning model
- `qwen3` - Qwen 3 model

## NVidia GPU Setup

### Prerequisites

- NVidia GPU installed on the Docker host
- NVidia GPU drivers installed (check with `nvidia-smi`)

### Install NVidia Container Toolkit

Before using GPU support, run the setup script on your Docker host:

```bash
chmod +x ./nvidia.sh
./nvidia.sh
```

This script will:

1. Configure the NVidia Container Toolkit repository
2. Install the NVidia Container Toolkit packages
3. Configure Docker to use the NVidia runtime
4. Restart Docker to apply changes

Alternatively, follow the [official installation guide](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html).

### Enable GPU in docker-compose.yml

After running the setup script, uncomment the `deploy` section in `docker-compose.yml`:

```yaml
deploy:
  resources:
    reservations:
      devices:
        - driver: nvidia
          count: 1
          capabilities: [gpu]
```

Adjust `count` to match the number of GPUs you want to expose (default is 1, set to `all` to use all available GPUs).

Then start the container:

```bash
docker compose up -d
```

To verify GPU access inside the container:

```bash
docker exec -it dumptruck-ollama bash
# Inside container:
nvidia-smi
```

## Starting the Container

### Basic Startup

```bash
docker compose up -d
```

### With GPU Support (after nvidia.sh)

Edit `docker-compose.yml` to uncomment the deploy section, then:

```bash
docker compose up -d
```

### View Logs

```bash
docker compose logs -f ollama
```

## Verification

Check that Ollama is running and models are loaded:

```bash
curl http://localhost:11434/api/tags
```

Expected output shows loaded models:

```json
{
  "models": [
    {
      "name": "nomic-embed-text:latest",
      "modified_at": "2024-...",
      "size": ...
    },
    ...
  ]
}
```

## Cleanup

```bash
docker compose down -v
```

## Configuration

- **Container**: `dumptruck-ollama`
- **Port**: `11434`
- **Models**: Stored in `ollama-data` volume
- **Image**: Based on `ollama/ollama:latest`

### Alternative GPU Support

#### AMD (ROCm)

Change image to `ollama/ollama:rocm` and add device mappings:

```yaml
image: ollama/ollama:rocm
devices:
  - /dev/kfd:/dev/kfd
  - /dev/dri:/dev/dri
```

#### Vulkan

Add device mappings and environment variable:

```yaml
devices:
  - /dev/kfd:/dev/kfd
  - /dev/dri:/dev/dri
environment:
  OLLAMA_VULKAN: '1'
```

## Usage

The Ollama API is available at `http://localhost:11434` for embedding requests.
