# Have I Been Pwned (HIBP) Integration

This document describes the integration with the Have I Been Pwned API for breach data enrichment.

## Overview

Dumptruck uses the [HIBP API v3](https://haveibeenpwned.com/API/v3) to enrich canonical address records with real-world breach intelligence. For each canonical email address, Dumptruck queries HIBP to identify:

- **Breaches**: Which public data breaches have included this address
- **Exposure Count**: How many times the address appeared across breaches
- **Breach Metadata**: Dates, domains, verification status, and descriptions

This enables analysts to:

- Prioritize addresses that appeared in high-impact breaches
- Track exposure history across multiple dumps
- Correlate leaked credentials with known breach data

## Setup

### API Key (Recommended)

While HIBP API v3 is free and requires only a User-Agent header, obtaining an API key provides:

- Higher rate limits (multiple requests/second vs. 1 request/second)
- Unrestricted access to sensitive breach data
- Priority support

**Get an API key**: <https://haveibeenpwned.com/API/Key>

**For Testing**: Use the test API key `00000000000000000000000000000000` for development and testing.

### Configuration

Environment variable for API key:

```bash
# Production
export DUMPTRUCK_HIBP_API_KEY="your-api-key-here"

# Testing
export DUMPTRUCK_HIBP_API_KEY="00000000000000000000000000000000"
```

## Usage in Code

### Basic Query

```rust
use dumptruck::hibp::HibpClient;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Create client with optional API key from environment
    let api_key = std::env::var("DUMPTRUCK_HIBP_API_KEY").ok();
    let client = HibpClient::new_default(api_key);

    // Get all breaches for an address
    let breaches = client.get_breaches_for_address("john.doe@example.com").await?;
    for breach in &breaches {
        println!("Breach: {} (affected: {} users)", breach.name, breach.pwn_count);
    }

    // Check if address is breached
    let is_breached = client.is_breached("john.doe@example.com").await?;
    println!("Breached: {}", is_breached);

    // Get total exposed count
    let total = client.pwn_count("john.doe@example.com").await?;
    println!("Total exposures: {}", total);

    Ok(())
}
```

### Storing Breach Data

```rust
use dumptruck::storage::{PostgresStorage, StorageAdapter};
use dumptruck::hibp::HibpClient;

let mut storage = PostgresStorage::new_from_env()?;
let client = HibpClient::new_default(api_key);

// Get breaches for canonical address
let breaches = client.get_breaches_for_address(&address_text).await?;

// Store each breach in the database
for breach in breaches {
    storage.insert_address_breach(
        &canonical_hash,
        &breach.name,
        Some(&breach.title),
        Some(&breach.domain),
        Some(&breach.breach_date),
        Some(breach.pwn_count as i32),
        Some(&breach.description),
        breach.is_verified,
        breach.is_fabricated,
        breach.is_sensitive,
        breach.is_retired,
    )?;
}
```

### Querying Stored Breaches

```rust
// Get all breaches for a canonical address
let breaches = storage.get_breaches_for_address(&canonical_hash)?;
for (breach_name, pwn_count) in breaches {
    println!("Breach: {} (exposures: {})", breach_name, pwn_count);
}

// Get breach statistics
let breach_count = storage.get_breach_count(&canonical_hash)?;
let total_exposed = storage.get_total_pwn_count(&canonical_hash)?;
println!("Breaches: {}, Total exposed: {}", breach_count, total_exposed);
```

## Database Schema

The `address_breaches` table stores:

```sql
CREATE TABLE address_breaches (
    id BIGSERIAL PRIMARY KEY,
    canonical_hash TEXT NOT NULL REFERENCES canonical_addresses,
    breach_name TEXT NOT NULL,
    breach_title TEXT,
    breach_domain TEXT,
    breach_date DATE,
    pwn_count INTEGER,
    description TEXT,
    is_verified BOOLEAN,
    is_fabricated BOOLEAN,
    is_sensitive BOOLEAN,
    is_retired BOOLEAN,
    checked_at TIMESTAMPTZ DEFAULT NOW(),
    first_seen_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(canonical_hash, breach_name)
);
```

**Indexes**:

- `(canonical_hash)`: Fast lookup of all breaches for an address
- `(breach_name)`: Fast lookup of all addresses in a specific breach
- `(checked_at)`: Identify addresses that need HIBP re-check

## API Endpoints

### GET /breachedaccount/{email}

Returns all breaches containing the email address.

**Parameters**:

- `email`: The email address to check (URL-encoded)
- `includeUnverified`: Include unverified breaches (true/false)

**Response**: Array of breach objects

**Status Codes**:

- `200 OK`: Address found in one or more breaches
- `404 NOT FOUND`: Address not in any known breach
- `400 BAD REQUEST`: Invalid email format
- `429 TOO MANY REQUESTS`: Rate limit exceeded

### Rate Limiting

| Configuration | Rate Limit |
|---|---|
| Without API key | 1 request/second |
| With API key | 10+ requests/second |
| Burst limit | Subject to fair use policy |

## Example Response

```json
[
  {
    "name": "LinkedIn",
    "title": "LinkedIn",
    "domain": "linkedin.com",
    "breachDate": "2021-06-22",
    "addedDate": "2021-07-15T16:31:31Z",
    "modifiedDate": "2021-07-15T16:31:31Z",
    "pwnCount": 700000000,
    "description": "LinkedIn suffered a data breach...",
    "isVerified": true,
    "isFabricated": false,
    "isSensitive": false,
    "isRetired": false,
    "logoPath": "https://haveibeenpwned.com/Content/Images/PwnedLogos/LinkedIn.png"
  }
]
```

## Workflow Integration

### Ingestion Pipeline

1. **Normalize Address** → Compute canonical hash
2. **Generate Embedding** → Get vector representation
3. **Check HIBP** → Query breaches for canonical address
4. **Store Breaches** → Save breach data in `address_breaches` table
5. **Dedup Check** → Check for duplicates (hash + vector)
6. **Store Canonical** → Create/link canonical address record

### Example Code

```rust
use dumptruck::normalization::normalize_field;
use dumptruck::storage::PostgresStorage;
use dumptruck::ollama::OllamaClient;
use dumptruck::hibp::HibpClient;

async fn process_address(
    storage: &mut PostgresStorage,
    ollama: &OllamaClient,
    hibp: &HibpClient,
    address_text: &str,
) -> std::io::Result<()> {
    // 1. Normalize
    let normalized = normalize_field(address_text);
    let canonical_hash = sha256(&normalized);

    // 2. Embed
    let embedding = ollama.embed(&address_text).await?;

    // 3. Check HIBP
    let breaches = hibp.get_breaches_for_address(address_text).await?;
    for breach in &breaches {
        storage.insert_address_breach(
            &canonical_hash,
            &breach.name,
            Some(&breach.title),
            Some(&breach.domain),
            Some(&breach.breach_date),
            Some(breach.pwn_count as i32),
            Some(&breach.description),
            breach.is_verified,
            breach.is_fabricated,
            breach.is_sensitive,
            breach.is_retired,
        )?;
    }

    // 4. Dedup check
    if let Some(dup) = storage.find_duplicate_address(&canonical_hash, Some(&embedding), 0.85)? {
        // Link to existing canonical address
        storage.insert_address_alternate(&dup, &canonical_hash, &address_text)?;
    } else {
        // Store new canonical address
        storage.insert_canonical_address(&canonical_hash, address_text, &normalized)?;
        storage.update_address_embedding(&canonical_hash, &embedding)?;
    }

    Ok(())
}
```

## Performance Notes

- **First Query**: ~1-2 seconds (includes API connection establishment)
- **Subsequent Queries**: ~100-500ms (depends on HIBP latency)
- **Caching**: Checked-at timestamp allows efficient re-checking of stale data
- **Batch Operations**: Use concurrent tokio tasks to query multiple addresses in parallel

## Privacy & Security

- **Data Retention**: Breach metadata is public information; no sensitive PII is stored
- **API Key**: Treat HIBP API keys as sensitive credentials (use environment variables)
- **User-Agent**: Required header identifies your application to HIBP
- **Rate Limiting**: Respect rate limits to avoid temporary IP blocks

## HIBP Data Quality

**Limitations**:

- Not all breaches are indexed by HIBP (only those publicly disclosed)
- Some breaches are removed due to takedown requests
- Verification status depends on HIBP manual review
- Sensitive breaches may require API key to access

**Best Practices**:

- Always handle 404 responses (not in breach) gracefully
- Implement exponential backoff for rate-limited requests
- Store breach checked-at timestamp to avoid redundant queries
- Log and monitor API errors for troubleshooting

## References

- [HIBP API Documentation](https://haveibeenpwned.com/API/v3)
- [Get API Key](https://haveibeenpwned.com/API/Key)
- [HIBP Blog](https://www.troyhunt.com/)
