# Configuration Management

Dumptruck uses a JSON configuration file to manage API keys and email domain substitution rules.

## Configuration File Location

By default, Dumptruck looks for configuration at:

- `config.json` in the current working directory
- Or specify a custom path via the `DUMPTRUCK_CONFIG` environment variable

A default configuration template is provided in `config.default.json`.

## Configuration Structure

### API Keys

The `api_keys` section contains credentials for external services:

```json
{
  "api_keys": {
    "hibp": "your-hibp-api-key-here"
  }
}
```

#### HIBP API Key

The `hibp` key is used for Have I Been Pwned API v3 requests. Obtain a key at [haveibeenpwned.com](https://haveibeenpwned.com).

**Environment Override:**
Set `DUMPTRUCK_HIBP_API_KEY` to override the configuration file value:

```bash
export DUMPTRUCK_HIBP_API_KEY="your-api-key"
```

### Email Suffix Substitutions

Email domains that have changed over time or have common aliases can be configured to map to a canonical form. The key is the canonical suffix, and the value is an array of alternate suffixes.

#### Example

```json
{
  "email_suffix_substitutions": {
    "bankofamerica.com": ["bofa.com"],
    "gmail.com": ["googlemail.com"],
    "yahoo.com": ["ymail.com", "rocketmail.com"],
    "microsoft.com": ["outlook.com", "hotmail.com", "live.com"],
    "amazon.com": ["aws.amazon.com"]
  }
}
```

#### How It Works

During email normalization, if an email uses an alternate domain, it is automatically converted to use the canonical domain:

- Input: `user@googlemail.com`
- Canonical: `user@gmail.com` (mapped via config)

This ensures that multiple email addresses referring to the same account (with different domain names) are recognized as duplicates during deduplication.

## Configuration Merging

Configuration values are loaded in priority order (highest priority first):

1. **Command-line arguments** (for CLI mode only) – highest priority
2. **Environment variables** (e.g., `DUMPTRUCK_HIBP_API_KEY`)
3. **Configuration file** (`config.json` or custom path via `DUMPTRUCK_CONFIG`)
4. **Default values** – lowest priority

This means:

- CLI arguments override everything else
- Environment variables override file-based configuration
- Configuration file values override defaults
- Defaults are used only when no other source provides a value

Example precedence resolution:

```bash
# Example 1: Using all sources
export DUMPTRUCK_HIBP_API_KEY="env-key"           # From environment
dumptruck --config config.json ingest data.csv    # From file: config.json

# Result: Environment variable takes precedence → uses "env-key"

# Example 2: CLI overrides environment
export DUMPTRUCK_HIBP_API_KEY="env-key"
dumptruck --hibp-key "cli-key" ingest data.csv

# Result: CLI argument takes precedence → uses "cli-key"
```

## Code Usage

Load configuration in your Rust code:

```rust
use dumptruck::config::Config;

// Load from default location
let config = Config::from_file("config.json")?;

// Load with environment variable overrides
let config = Config::from_file_with_env("config.json")?;

// Access values
let hibp_key = config.hibp_api_key();
let gmail_alternates = config.get_suffix_alternates("gmail.com");

// Check if suffix has rules
if config.has_suffix_alternates("example.com") {
    // Handle alternates
}
```

## Default Configuration

A default configuration is provided in `config.default.json`. For production use, create a `config.json` file with your actual API keys and domain rules.

## Security Notes

- Never commit `config.json` with real API keys to version control
- Use environment variables for sensitive credentials in CI/CD environments
- The default test API key (`00000000000000000000000000000000`) should only be used for development
