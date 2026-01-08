# Configuration

Dumptruck loads configuration from JSON files and applies CLI overrides at runtime.

## Configuration file search

If `--config <CONFIGURATION>` is provided, Dumptruck loads only that file.

Otherwise, Dumptruck searches (in order):

- `./config.json`
- `/etc/dumptruck/config.json`
- A user config path (e.g., `~/.config/dumptruck/config.json` on Linux)

## Schema and template

The authoritative schema is [config.schema.json](../config.schema.json). A starting template is provided in [config.default.json](../config.default.json).

## Common settings

### API keys

`api_keys` is a list of service keys. For example:

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

### Paths

`paths` configures filesystem locations used by Dumptruck.

- `temp_path`: working directory for isolated processing
- `db_path`: base location for the SQLite database (Dumptruck creates `dumptruck.db` under this path)

## Override order

Configuration is loaded from files first, then CLI overrides are applied (CLI wins).

## Security notes

- Do not commit real API keys to version control.
- Prefer OS-level secrets management for production deployments.
