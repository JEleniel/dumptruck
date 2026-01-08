# Address Enrichment Pipeline

This document describes how canonical addresses are enriched with threat intelligence from multiple sources.

## Overview

The enrichment pipeline augments canonical address records with:

1. **Vector Embeddings** (Nomic/Ollama): Semantic representation for similarity matching
2. **Breach Data** (HIBP API): Known compromises from public breaches
3. **Co-occurrence Graph**: Address pairs seen together in the same record
4. **Unicode Variants**: Alternative representations of the same address

## Architecture

```text
Input Address
    │
    ├─→ Normalization
    │   └─→ NFKC + ICU4X case-fold + punctuation normalization
    │
    ├─→ Canonical Hash
    │   └─→ SHA256(normalized_form)
    │
    ├─→ Vector Embedding
    │   └─→ Ollama/Nomic API (768-dim vector)
    │
    ├─→ Breach Lookup
    │   └─→ HIBP API v3 (breach data enrichment)
    │
    ├─→ Deduplication Check
    │   └─→ Hash match → Vector similarity
    │
    └─→ Storage
        ├─→ canonical_addresses (with embedding)
        ├─→ address_breaches (breach data)
        ├─→ address_alternates (Unicode variants)
        ├─→ address_credentials (linked credentials)
        └─→ address_cooccurrence (graph edges)
```

## Enrichment Sources

### 1. Vector Embedding (Nomic/Ollama)

**Purpose**: Enable semantic similarity matching for near-duplicate detection

**Data**:

- 768-dimensional vector representation
- Generated from email/address text
- Stored in `canonical_addresses.embedding` column

**Queries**:

```sql
-- Find similar addresses by cosine distance
SELECT canonical_hash, (1.0 - (embedding <-> query_vec)) as similarity
FROM canonical_addresses
WHERE (1.0 - (embedding <-> query_vec)) >= 0.85
ORDER BY similarity DESC
LIMIT 10;
```

**Performance**:

- Generation: ~100-200ms per address
- Search: ~10-50ms (IVFFlat index)
- Typical threshold: 0.85 (85% similarity)

### 2. Breach Data (HIBP API)

**Purpose**: Identify which public breaches have compromised an address

**Data**:

- Breach name (e.g., "LinkedIn", "Adobe")
- Breach date and metadata
- Count of exposed credentials
- Verification status

**Storage**:

```sql
INSERT INTO address_breaches (
    canonical_hash, breach_name, breach_title, breach_domain,
    breach_date, pwn_count, description, is_verified, ...
);
```

**Performance**:

- Typical latency: 200-500ms per address
- Rate limits: 1 req/sec (free), 10+ req/sec (API key)
- Caching: Checked-at timestamp prevents redundant queries

### 3. Unicode Variants (Normalization)

**Purpose**: Track alternative representations of the same address

**Common Variants**:

- Composed/decomposed: "José" vs "Jose\u{0301}"
- Fullwidth: "Ａ<lice@Example.COM>" vs "<alice@example.com>"
- German ß: "Straße" vs "STRASSE"
- Punctuation: "O'Connor" vs "O\u{2019}Connor"

**Storage**:

```sql
INSERT INTO address_alternates (
    canonical_hash, alternate_hash, alternate_form
);
```

**Deduplication**:

- Hash alternate_form with same normalization rules
- Lookup canonical via `address_alternates` table
- No duplicate canonical record created

### 4. Co-occurrence Graph (Relationship Tracking)

**Purpose**: Identify address pairs seen together in the same record

**Storage**:

```sql
INSERT INTO address_cooccurrence (
    canonical_hash_1, canonical_hash_2, cooccurrence_count
);
```

**Queries**:

```sql
-- Find all addresses that appeared with a given address
SELECT canonical_hash_2, cooccurrence_count
FROM address_cooccurrence
WHERE canonical_hash_1 = $1
ORDER BY cooccurrence_count DESC;
```

**Use Cases**:

- Build address clusters (transitive closure)
- Identify credential pairs (e.g., work email + personal email)
- Reconstruct partial dumps

## Processing Workflow

### Ingestion Example

**Input Record**:

```csv
john.doe@example.com,password123,jane.smith@example.com,secret456
```

**Processing Steps**:

1. **Address 1: "<john.doe@example.com>"**
   + Normalize: "<john.doe@example.com>" → "<john.doe@example.com>"
   + Hash: SHA256 → `hash_john_doe`
   + Embed: Ollama → 768-dimensional vector
   + HIBP: Query breaches → "LinkedIn", "Yahoo"
   + Store:
     - `canonical_addresses(hash_john_doe, "john.doe@example.com", ..., embedding, ...)`
     - `address_breaches(hash_john_doe, "LinkedIn", ...)`
     - `address_breaches(hash_john_doe, "Yahoo", ...)`
   + Credential: `address_credentials(hash_john_doe, "password123", count=1)`

2. **Address 2: "<jane.smith@example.com>"**
   + Similar process → `hash_jane_smith`
   + Store:
     - `canonical_addresses(hash_jane_smith, "jane.smith@example.com", ...)`
     - `address_breaches(hash_jane_smith, "LinkedIn", ...)`
   + Credential: `address_credentials(hash_jane_smith, "secret456", count=1)`

3. **Co-occurrence**: Both appeared in same record
   + `address_cooccurrence(hash_john_doe, hash_jane_smith, count=1)`

### Output

Canonical addresses with enrichment:

```text
john.doe@example.com
  ├─ Embedding: [0.234, -0.456, ...] (768-dim)
  ├─ Breaches: LinkedIn, Yahoo
  ├─ Total Exposed: 700M + 3B = 3.7B
  ├─ Credentials: ["password123"]
  └─ Co-occurrence: [jane.smith@example.com (count=1)]

jane.smith@example.com
  ├─ Embedding: [0.123, 0.789, ...]
  ├─ Breaches: LinkedIn
  ├─ Total Exposed: 700M
  ├─ Credentials: ["secret456"]
  └─ Co-occurrence: [john.doe@example.com (count=1)]
```

## Storage Methods

### Core Enrichment Operations

```rust
// Vectors
storage.update_address_embedding(&hash, &embedding)?;
let similar = storage.find_similar_addresses(&embedding, 10, 0.85)?;

// Breaches
storage.insert_address_breach(&hash, "LinkedIn", Some("LinkedIn"), ...)?;
let breaches = storage.get_breaches_for_address(&hash)?;
let breach_count = storage.get_breach_count(&hash)?;
let total = storage.get_total_pwn_count(&hash)?;

// Co-occurrence
storage.record_address_cooccurrence(&hash1, &hash2)?;
let neighbors = storage.get_address_neighbors(&hash)?;

// Variants
storage.insert_address_alternate(&canonical, &variant_hash, &text)?;
let canonical = storage.lookup_canonical_by_alternate(&variant_hash)?;

// Credentials
storage.insert_address_credential_canonical(&hash, &cred_hash)?;
let creds = storage.get_credentials_for_address(&hash)?;
```

## Performance Optimization

### Parallel Processing

```rust
use futures::future::join_all;

let addresses = vec!["addr1@example.com", "addr2@example.com", ...];
let tasks: Vec<_> = addresses.iter()
    .map(|addr| {
        let ollama = ollama.clone();
        let hibp = hibp.clone();
        async move {
            let embedding = ollama.embed(addr).await?;
            let breaches = hibp.get_breaches_for_address(addr).await?;
            Ok((embedding, breaches))
        }
    })
    .collect();

let results = join_all(tasks).await;
```

**Expected Throughput**:

- Single-threaded: ~3-5 addresses/second (rate-limited by HIBP)
- 10 concurrent tasks: ~50-100 addresses/second
- With API key: Up to 10x faster (higher rate limits)

### Batch Operations

- **Embedding**: Generate for multiple addresses in parallel
- **HIBP Queries**: Queue and batch with exponential backoff
- **Storage**: Use prepared statements and transaction batching

### Caching Strategy

- **Breach Data**: Store `checked_at` timestamp; skip if recent
- **Embeddings**: Cache in memory if processing same dataset multiple times
- **Canonical Hash**: Maintain in-memory lookup table during batch processing

## Configuration

### Configuration file

Dumptruck is configured via JSON files. See [CONFIGURATION.md](CONFIGURATION.md) for file locations and overrides.

```json
{
  "api_keys": [
    {
      "name": "haveibeenpwned",
      "api_key": "YOUR_KEY_HERE"
    }
  ],
  "services": {
    "enable_embeddings": true,
    "ollama": {
      "url": "http://localhost:11434",
      "vector_threshold": 0.85
    }
  }
}
```

### Tuning Parameters

| Parameter | Default | Recommendation |
| --- | --- | --- |
| Vector similarity threshold | 0.85 | 0.80-0.90 |
| HIBP rate limit | 1 req/sec | 10 req/sec (with API key) |
| Embedding batch size | 1 | 10-50 (for throughput) |
| HIBP retry delay | exponential | Start at 1s, max 60s |

## Monitoring & Metrics

### Key Metrics

- **Embedding Generation**: ms per address
- **Vector Search Latency**: ms per query
- **HIBP API Latency**: ms per address
- **Duplicate Detection Rate**: % of records found to be duplicates
- **False Positive Rate**: % of incorrect duplicates
- **Storage Throughput**: records per second

### Logging

```rust
eprintln!("Processing {}", address_text);
eprintln!("  Normalized: {}", normalized);
eprintln!("  Hash: {}", canonical_hash);
eprintln!("  Embedding: {}ms", elapsed_embedding);
eprintln!("  HIBP: {} breaches in {}ms", breach_count, elapsed_hibp);
eprintln!("  Stored: {}", if new { "new" } else { "deduplicated" });
```

## Troubleshooting

### Common Issues

| Issue | Cause | Solution |
| --- | --- | --- |
| Slow embedding generation | Model loading | Increase concurrency, cache embeddings |
| HIBP rate limit errors | Too many requests | Use API key, implement exponential backoff |
| Duplicate false negatives | Low similarity threshold | Increase threshold, review manual cases |
| High similarity false positives | Threshold too low | Decrease threshold, add manual review step |
| Storage bottleneck | Connection pool exhausted | Increase pool size, batch inserts |

## See Also

- [Deduplication Architecture](DEDUP_ARCHITECTURE.md)
- [Ollama Integration](OLLAMA.md)
- [HIBP Integration](HIBP.md)
- [Vector Deduplication](VECTOR_DEDUP.md)
