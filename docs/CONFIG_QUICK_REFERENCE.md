# Quick Reference: Configuration

Dumptruck loads configuration from JSON files and applies CLI overrides at runtime.

For full details, see [CONFIGURATION.md](CONFIGURATION.md).

## Configuration File Locations

If `--config <CONFIGURATION>` is provided, Dumptruck loads only that file.

Otherwise, Dumptruck searches (in order):

- `./config.json`
- `/etc/dumptruck/config.json`
- A user config path (e.g., `~/.config/dumptruck/config.json` on Linux)

## Schema and Template

- Schema: [config.schema.json](../config.schema.json)
- Template: [config.default.json](../config.default.json)

## Common Settings

### API keys

`api_keys` is a list of service keys.

```json
{
  "api_keys": [
    {
      "name": "haveibeenpwned",
      "api_key": "YOUR_KEY_HERE"
    }
  ]
}
```

You can also supply API keys via CLI (repeatable):

```bash
dumptruck --api-keys haveibeenpwned=YOUR_KEY_HERE analyze ./breach.csv
```

### Email suffix substitutions

`email_suffix_substitutions` describes domain aliasing rules used during normalization.

```json
{
  "email_suffix_substitutions": [
    {
      "original": "gmail.com",
      "substitutes": ["googlemail.com"]
    }
  ]
}
```

### Embeddings (Ollama)

Embeddings are configured under `services`.

```json
{
  "services": {
    "enable_embeddings": true,
    "ollama": {
      "url": "http://localhost:11434",
      "vector_threshold": 0.85
    }
  }
}
```

### Paths

`paths` configures filesystem locations used by Dumptruck.

- `temp_path`: working directory for isolated processing
- `db_path`: base location for the SQLite database (Dumptruck creates `dumptruck.db` under this path)

## Troubleshooting

### "Config file not found"

- Verify the file exists at the expected search path(s).
- Use `--config <CONFIGURATION>` to load an explicit file.

### "Embeddings not working"

- Verify `services.ollama.url` is reachable.
- Confirm embeddings are enabled (`services.enable_embeddings` or `analyze --enable-embeddings`).
