# Email Suffix Substitution System

## Overview

Dumptruck can automatically resolve email domain aliases to canonical forms during normalization. This enables accurate deduplication even when addresses use different domain names that are known aliases of each other.

### Real-World Example

Bank of America changed its domain name over time:

- Old domain: `bankofamerica.com`
- New alias: `bofa.com`

Without substitution rules, these would be treated as different addresses:

- `john.smith@bankofamerica.com` → canonical hash A
- `john.smith@bofa.com` → canonical hash B

With substitution rules, both resolve to the same canonical address:

- `john.smith@bankofamerica.com` → canonical hash A
- `john.smith@bofa.com` → **also canonical hash A** (via substitution)

## Configuration

Email suffix substitutions are defined in the JSON configuration file. The format is:

```json
{
  "email_suffix_substitutions": {
    "canonical-domain.com": [
      "alternate-domain-1.com",
      "alternate-domain-2.com"
    ]
  }
}
```

### Default Configuration

The `config.default.json` file includes common email domain aliases:

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

## How It Works

### Normalization Flow

1. **Input Email**: `user@GoogleMail.COM`
2. **Field Normalization**: Convert to lowercase, collapse whitespace → `user@googlemail.com`
3. **Configuration Lookup**: Check if `googlemail.com` is a known alternate
4. **Domain Substitution**: Map `googlemail.com` → `gmail.com` (canonical form)
5. **Output**: `user@gmail.com`

### Hashing & Deduplication

Once emails are normalized with canonical domains, they are hashed:

```
canonical_hash = SHA256(normalize_email_with_config(address, config))
```

All variants of the same address now produce the same hash:

- `user@gmail.com` → hash X
- `user@googlemail.com` → hash X (after substitution)
- `user@GMAIL.COM` → hash X (after normalization + substitution)

## Integration with Pipeline

The email substitution system integrates with the deduplication pipeline:

```
Raw CSV
  ↓
Extract address field
  ↓
normalize_email_with_config(address, config)
  ↓
Compute canonical_hash
  ↓
Check duplicate (hash lookup)
  ├→ Found: link to canonical address
  └→ Not found: continue to vector embedding
```

## Code Examples

### Load Configuration with Substitutions

```rust
use dumptruck::config::Config;
use dumptruck::normalization::normalize_email_with_config;

// Load configuration from file
let config = Config::from_file("config.json")?;

// Normalize an email with substitution rules applied
let email = "john.smith@googlemail.com";
let normalized = normalize_email_with_config(email, &config);
// Result: "john.smith@gmail.com"
```

### Programmatic Configuration

```rust
use dumptruck::config::Config;

let mut config = Config::default();

// Add substitution rules
config.add_suffix_rule(
    "example.com".to_string(),
    vec!["ex.com".to_string(), "example.net".to_string()]
);

// Query rules
if config.has_suffix_alternates("example.com") {
    let alts = config.get_suffix_alternates("example.com");
    println!("Alternates: {:?}", alts);
}
```

## Design Decisions

### Why Configuration-Based?

1. **Flexibility**: Rules can change without recompiling
2. **Maintenance**: Easy to add/remove domain aliases as they change
3. **Transparency**: Rules are explicit and auditable
4. **Scalability**: Supports hundreds of domain mappings

### Key Properties

- **Unidirectional**: The key (canonical domain) maps to values (alternates)
    + ✅ Correct: `gmail.com` → `[googlemail.com]`
    + ❌ Incorrect: `googlemail.com` → `[gmail.com]`

- **One-to-Many**: A canonical domain can have multiple alternates
    + `microsoft.com` → `[outlook.com, hotmail.com, live.com]`

- **No Circular References**: Alternates should not reference each other
    + ❌ Incorrect: `gmail.com` → `[googlemail.com]` AND `googlemail.com` → `[gmail.com]`

- **Stateless**: Normalization is deterministic and order-independent
    + `normalize_email_with_config(email, config)` always produces the same result

## Adding New Rules

To add new domain substitution rules:

1. **Edit `config.json`** (or `config.default.json` for defaults):

```json
{
  "email_suffix_substitutions": {
    "bankofamerica.com": ["bofa.com"],
    "my-company.com": ["mycompany.com", "myco.com"]  // ← New rule
  }
}
```

2. **Restart the application** to load the new configuration

3. **Test the rule**:

```bash
cargo test normalization::normalize_email_with_config
```

## Security Considerations

- Configuration files may contain sensitive API keys; keep them out of version control
- Use environment variables to override API keys in CI/CD systems
- Domain substitution rules are non-sensitive and can be version-controlled
- The system performs no DNS lookups; rules are purely local string matching
