# Peer Discovery and Database Synchronization Design

## Overview

Dumptruck instances on the same IP subnet (/24 or larger) automatically discover each other and synchronize their deduplication and enrichment databases. This allows instances to collectively learn from ingested data while minimizing bandwidth usage through delta sync and Bloom filter-based exchange.

## 1. Peer Discovery

### 1.1 UDP Broadcast Discovery

**Protocol**: UDP broadcast on port 49999 (configurable)

**Subnet Detection**:

- On startup, detect local IPv4 address and subnet mask
- Calculate broadcast address (e.g., 192.168.1.255 for 192.168.1.0/24)
- Support static subnet override via config

**Discovery Message** (JSON, ~120 bytes):

**Discovery Message** (JSON, ~120 bytes):

```json
{
  "type": "peer_discovery",
  "instance_id": "uuid-v4",
  "hostname": "dumptruck-1",
  "version": "1.0.0",
  "timestamp": 1702534632,
  "sync_port": 8444,
  "db_version": 42
}
```

**Discovery Cycle**:

1. Listen on UDP port 49999 on all interfaces (0.0.0.0:49999)
2. Every 30 seconds (configurable), send broadcast to detected subnet
3. Keep received peers in memory with last-seen timestamp
4. Remove peers not seen for 120 seconds (configurable)
5. Limit peers per subnet to 32 to prevent explosion

### 1.2 Peer Registry

In-memory structure:

```rust
struct Peer {
    instance_id: String,      // UUID, unique identifier
    hostname: String,
    last_seen: Instant,
    db_version: u32,          // Incremented on each local DB change
    sync_port: u16,
}

struct PeerRegistry {
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    local_instance_id: String,
    local_db_version: Arc<AtomicU32>,
    subnet: IpNetwork,
}
```

## 2. Sync Protocol

### 2.1 Three-Phase Sync

**Phase 1: Announce DB Version** (TCP SYN)

- Peer A → Peer B: "I have db_version=42"
- Peer B checks: if own version < 42, request sync

**Phase 2: Bloom Filter Exchange** (TCP)

- Requesting peer asks: "Send me Bloom filter of your hashes"
- Bloom filter: ~32KB for 1M items (false positive rate 1%)
- Requesting peer compares locally to identify missing records

**Phase 3: Delta Sync** (TCP)

- Request only the missing records (usually small set)
- Response includes records with metadata (timestamp, source)

### 2.2 Data to Sync

**Synchronized**:

- Hash of credentials (SHA-256 format)
- Associated metadata: email, breach source, first-seen timestamp
- Enrichment vectors (from Ollama for near-duplicates)
- HIBP breach associations

**NOT synchronized**:

- Raw credentials (only hashes maintained)
- PII detection records (privacy boundary)
- Job queue and processing state

### 2.3 Bandwidth Optimization

**Bloom Filters**:

- Represent set of 1M items in ~32KB
- Use XXHash64 + 4 hash functions for speed
- False positives acceptable (only cause redundant data requests)

**Compression**:

- Gzip compress delta responses (10-20x typical compression)
- Max response: 100MB per sync (prevents resource exhaustion)

**Rate Limiting**:

- Max 1 sync per peer per 1 minute
- Exponential backoff on failure: 5s, 10s, 20s, 60s
- Stop retrying after 10 failed attempts

**Early Exit**:

- If peer's db_version <= local db_version, skip sync
- Skip if peer not seen for 30s (might be offline)

## 3. Data Consistency

### 3.1 Conflict Resolution

**Strategy**: Last-write-wins with version vectors

Each record includes:

- `peer_id`: originating instance UUID
- `timestamp`: creation time (UTC, wall-clock)
- `vector_clock`: `[peer_a_version, peer_b_version, ...]`

**Resolution**:

1. If timestamps differ by >10 seconds: use newer record
2. If within 10 seconds: use lexicographically smaller peer_id
3. For Ollama vectors: merge both (duplicate vectors are harmless)

### 3.2 Database Version Tracking

- Local `db_version: u32` increments on:
    + New credential hash ingested
    + New enrichment data added
    + Successful peer sync accepted
- Version persisted in PostgreSQL config table
- Prevents sync loops (peer A ← peer B ← peer A)

### 3.3 PostgreSQL Schema Changes

Add to existing schema:

```sql
CREATE TABLE IF NOT EXISTS peer_metadata (
    record_id BIGINT PRIMARY KEY,
    peer_id UUID NOT NULL,
    timestamp BIGINT NOT NULL,
    version_vector TEXT NOT NULL,  -- JSON: {"peer-id": version, ...}
    synced_from_peer UUID          -- Track source for debugging
);

CREATE INDEX idx_peer_metadata_timestamp ON peer_metadata(timestamp);
CREATE INDEX idx_peer_metadata_peer_id ON peer_metadata(peer_id);
```

Add to credential_hashes table:

```sql
ALTER TABLE credential_hashes 
    ADD COLUMN IF NOT EXISTS peer_id UUID DEFAULT '00000000-0000-0000-0000-000000000000',
    ADD COLUMN IF NOT EXISTS sync_timestamp BIGINT DEFAULT 0,
    ADD COLUMN IF NOT EXISTS version_vector TEXT DEFAULT '{}';
```

## 4. Failure Scenarios

### Network Partition

- Peers continue operating independently
- On reconnect, version vectors ensure consistency
- No data loss, eventual consistency guaranteed

### Peer Crash/Restart

- Other peers detect absence (no pings for 120s)
- Remove from registry, stop sync attempts
- On restart, peer re-advertises with same instance_id
- If db_version lower, re-sync from others

### Duplicate/Conflicting Hashes

- Same hash from different sources handled by conflict resolution
- Multiple enrichment vectors merged (not deduplicated—all retained)

### Clock Skew

- Timestamps within 10s treated as equal (pair resolves with peer_id)
- For security: HIBP enrichment uses server timestamps, not local clock

## 5. Configuration

**New config.default.json**:

```json
{
  "peer_sync": {
    "enabled": true,
    "discovery_port": 49999,
    "sync_port": 8444,
    "sync_interval_secs": 60,
    "broadcast_interval_secs": 30,
    "peer_timeout_secs": 120,
    "max_peers_per_subnet": 32,
    "sync_max_records": 100000,
    "bloom_filter_capacity": 1000000,
    "bloom_filter_false_positive_rate": 0.01,
    "exponential_backoff": true,
    "subnet_override": null
  }
}
```

## 6. Implementation Plan

### Phase 1: Foundation (Core Sync)

- [ ] Peer discovery module with UDP broadcast
- [ ] Peer registry with expiration
- [ ] Configuration parsing

### Phase 2: Sync Protocol

- [ ] Bloom filter generation (using `bloom` crate)
- [ ] Version announcement protocol
- [ ] Delta sync responder

### Phase 3: Database Integration

- [ ] PostgreSQL metadata tables
- [ ] Conflict resolution logic
- [ ] Version vector tracking

### Phase 4: Robustness

- [ ] Exponential backoff and retry logic
- [ ] Rate limiting per peer
- [ ] Error recovery and cleanup

### Phase 5: Testing & Deployment

- [ ] Multi-instance local test suite
- [ ] Bandwidth/performance measurements
- [ ] Failure scenario tests

## 7. Bandwidth Analysis

### Example Scenario: 3 instances, 1M credentials per instance

**Initial Sync (worst case)**:

- Instance A has records (1 to 1M)
- Instance B has records (500K to 1.5M)
- Instance C has records (1.5M to 2.5M)

**Bandwidth per sync**:

1. Bloom filter exchange: 3 × 32KB = 96KB
2. Delta request/response: ~20K missing records
   + Per record: ~200 bytes compressed (~2KB uncompressed)
   + Total: 20K × 200 = 4MB (or ~400KB compressed)
3. Total per full sync: ~500KB

**After initial sync**:

- Steady-state: only new hashes (~1K-10K per sync cycle)
- Bandwidth: ~10-50KB per sync cycle

**Optimization payoff**:

- Without Bloom filter: request all 1M records (~200MB)
- With Bloom filter: request only deltas (~4MB)
- **50x bandwidth reduction**

## 8. Security Considerations

- **No authentication**: assumes trusted internal network (/24 subnet)
- **No encryption**: assumes internal network (prod: use IPsec/VPN)
- **Rate limiting**: prevents resource exhaustion via malicious peers
- **Record origin tracking**: know which peer provided each record
- **Audit trail**: sync_timestamp and peer_id provide traceability

## 9. Open Questions for Discussion

1. Should we use TCP or UDP for delta sync? (Currently TCP for reliability)
2. Merkle tree instead of Bloom filter? (Added complexity, similar bandwidth)
3. Should PII detection records be synced? (Currently no—privacy boundary)
4. What's the acceptable clock skew tolerance? (Currently 10s)
5. Should peer_id be persistent across restarts? (Currently yes, via config)
