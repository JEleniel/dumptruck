# Quick Reference: Configuration

## Configuration File Locations

1. Default: `config.json` in current working directory
2. Custom: Set `DUMPTRUCK_CONFIG` environment variable
3. Template: `config.default.json` (provided in repo)

## Configuration Structure

```json
{
  "api_keys": {
    "hibp": "your-api-key-here"
  },
  "email_suffix_substitutions": {
    "canonical-domain.com": [
      "alternate-domain-1.com",
      "alternate-domain-2.com"
    ]
  }
}
```

## Environment Variable Overrides

| Variable | Purpose | Example |
|----------|---------|---------|
| `DUMPTRUCK_HIBP_API_KEY` | Override HIBP API key | `export DUMPTRUCK_HIBP_API_KEY="abcd1234..."` |
| `DUMPTRUCK_CONFIG` | Custom config file path | `export DUMPTRUCK_CONFIG="/etc/dumptruck/config.json"` |

## Common Email Domain Substitutions

| Canonical | Alternates |
|-----------|-----------|
| `gmail.com` | `googlemail.com` |
| `yahoo.com` | `ymail.com`, `rocketmail.com` |
| `microsoft.com` | `outlook.com`, `hotmail.com`, `live.com` |
| `bankofamerica.com` | `bofa.com` |
| `amazon.com` | `aws.amazon.com` |

## Example Configuration

```json
{
  "api_keys": {
    "hibp": "00000000000000000000000000000000"
  },
  "email_suffix_substitutions": {
    "bankofamerica.com": ["bofa.com"],
    "gmail.com": ["googlemail.com"],
    "yahoo.com": ["ymail.com", "rocketmail.com"],
    "microsoft.com": ["outlook.com", "hotmail.com", "live.com"],
    "amazon.com": ["aws.amazon.com"]
  }
}
```

## Rust API

### Load Configuration

```rust
use dumptruck::config::Config;

// From file
let config = Config::from_file("config.json")?;

// From file with env overrides
let config = Config::from_file_with_env("config.json")?;
```

### Query Configuration

```rust
// Get API key
let hibp_key = config.hibp_api_key();

// Check if suffix has rules
if config.has_suffix_alternates("gmail.com") {
    // Get all alternates
    let alts = config.get_suffix_alternates("gmail.com");
}

// Get all rules
let rules = config.all_suffix_rules();
```

### Modify Configuration

```rust
// Add new rule
config.add_suffix_rule(
    "example.com".to_string(),
    vec!["ex.com".to_string()]
);
```

## Normalization with Config

```rust
use dumptruck::normalization::normalize_email_with_config;

let email = "user@googlemail.com";
let normalized = normalize_email_with_config(email, &config);
// Result: "user@gmail.com"
```

## Testing

Run configuration tests:

```bash
cargo test --test config
```

Run normalization tests with email substitution:

```bash
cargo test normalization::normalize_email_with_config
```

## Best Practices

1. **Never commit real API keys** to version control
2. **Use environment variables** for production deployments
3. **Keep domain rules version-controlled** (they're non-sensitive)
4. **Test new rules** before deploying:

   ```bash
   cargo test --lib normalization
   ```

5. **Document custom rules** in your deployment docs
6. **Review alternates periodically** as companies change domain names

## Troubleshooting

### "Config file not found"

- Check file exists at specified path
- Check `DUMPTRUCK_CONFIG` environment variable
- Verify current working directory

### "Email not being deduplicated"

- Verify domain alias is in config
- Check canonical domain is the **key**, not the value
- Reload application (config is loaded at startup)

### "API key not being used"

- Check `DUMPTRUCK_HIBP_API_KEY` environment variable
- Verify `api_keys.hibp` in config file
- Env var takes precedence over config file
