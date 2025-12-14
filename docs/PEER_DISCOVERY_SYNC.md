# Peer Discovery and Synchronization

## Overview

Dumptruck instances on the same IP subnet (/24) automatically discover each other via UDP broadcast and synchronize their deduplication and enrichment databases using Bloom filter-based delta sync. This allows instances to collectively learn from ingested data while minimizing bandwidth usage.

## Architecture

### Peer Discovery

**UDP Broadcast Protocol**:

- Port: 49999 (configurable)
- Broadcast interval: 30 seconds
- Peer timeout: 120 seconds (remove peers not seen for 2 minutes)
- Max peers per subnet: 32

**Discovery Message** (JSON, ~120 bytes):

```json
{
  "type": "peer_discovery",
  "instance_id": "550e8400-e29b-41d4-a716-446655440000",
  "hostname": "dumptruck-1",
  "version": "1.0.0",
  "timestamp": 1702534632,
  "sync_port": 8444,
  "db_version": "abc123def456"
}
```

**Discovery Flow**:

1. Instance starts with unique UUID v4 `instance_id`
2. Detects local IPv4 address and calculates subnet (assumes /24)
3. Begins listening on UDP 0.0.0.0:49999
4. Every 30 seconds, broadcasts presence to subnet broadcast address
5. Receives broadcasts from other instances and tracks peers
6. Periodically removes peers not seen for 120+ seconds
7. Enforces maximum of 32 peers per subnet

### Peer Registry

In-memory thread-safe peer registry tracking:

```rust
pub struct Peer {
    pub instance_id: String,        // UUID identifier
    pub hostname: String,           // Peer hostname
    pub sync_addr: SocketAddr,      // Address:port for sync server
    pub db_version: String,         // Current database version hash
    pub last_seen: u64,             // Unix epoch timestamp
    pub version: String,            // Dumptruck version
}
```

**Registry Operations**:

- `add_peer()` - Add or update peer (async, thread-safe)
- `get_peers()` - Get all known peers
- `get_peer(id)` - Get specific peer
- `cleanup_stale_peers()` - Remove peers not seen for 120+ seconds
- `peer_count()` - Get number of known peers

## Synchronization

### Bloom Filter-Based Delta Sync

Minimizes bandwidth by using Bloom filters to identify new data.

**Bloom Filter Spec**:

- Size: 1MB (8,388,608 bits) per instance
- Hash functions: k=3
- False positive rate: ~1% for 100k items
- Serializable to JSON for transmission

**Sync Request** (sent by instance A to instance B):

```json
{
  "requester_id": "550e8400-e29b-41d4-a716-446655440000",
  "requester_db_version": "abc123",
  "known_hashes_filter": [base64 encoded bloom filter],
  "filter_size_bits": 8388608
}
```

Instance B receives the Bloom filter and:

1. For each address hash in its database, checks: `filter.contains(hash)`
2. If not in filter (likely new to instance A), includes in response

**Sync Response** (sent by instance B to instance A):

```json
{
  "responder_id": "660e8400-e29b-41d4-a716-446655440001",
  "responder_db_version": "def456",
  "new_addresses": ["hash1", "hash2", ...],
  "new_canonical_mappings": [["variant1", "canonical1"], ...],
  "new_breaches": [["address_hash", "breach_name"], ...]
}
```

**Sync Flow**:

1. Instance A detects instance B (via discovery)
2. Instance A creates Bloom filter of all address hashes it knows
3. Instance A sends SyncRequest with filter to instance B's sync server
4. Instance B checks database against filter
5. Instance B responds with new addresses, mappings, and breaches
6. Instance A merges new data into its database
7. Both instances update sync state (version, item count, timestamp)

### Sync State Tracking

Maintains per-peer sync state to avoid redundant syncs:

```rust
pub struct SyncState {
    pub peer_id: String,                        // Peer instance ID
    pub last_synced_db_version: String,         // Last synced version
    pub last_sync_item_count: usize,            // Items received
    pub last_sync_time: u64,                    // Timestamp
    pub direction: String,                      // "pull" or "push"
}
```

**State Management**:

- Tracks last synced version to detect changes
- Detects stale syncs (>5 minutes without sync)
- Supports both pull (we request) and push (peer requests) models

## Implementation

### Modules

**`src/peer_discovery.rs`** (471 lines, 20 tests):

- `DiscoveryMessage` - UDP broadcast message
- `Peer` - Peer information
- `PeerRegistry` - Thread-safe peer registry
- `DiscoveryListener` - UDP listener and broadcaster

**`src/peer_sync.rs`** (440 lines, 19 tests):

- `BloomFilter` - Efficient membership testing
- `SyncRequest` - Delta sync request
- `SyncResponse` - Delta sync response
- `SyncState` - Per-peer sync tracking
- `SyncManager` - Coordinates synchronization

### Usage

**Initialize peer discovery**:

```rust
use std::sync::Arc;
use std::net::Ipv4Addr;
use dumptruck::peer_discovery::{PeerRegistry, DiscoveryListener};

// Create registry with local IPv4
let registry = Arc::new(PeerRegistry::new(Ipv4Addr::new(192, 168, 1, 100)));

// Create listener
let listener = DiscoveryListener::new(
    registry.clone(),
    "local-host".to_string(),
    "1.0.0".to_string(),
    8444, // sync server port
)?;

// Start background tasks for discovery
let (db_tx, db_rx) = tokio::sync::watch::channel("initial_version".to_string());
Arc::new(listener).start(db_rx).await?;

// Get known peers
let peers = registry.get_peers().await;
for peer in peers {
    println!("Peer: {} @ {}", peer.instance_id, peer.sync_addr);
}
```

**Initialize sync manager**:

```rust
use dumptruck::peer_sync::SyncManager;

let manager = SyncManager::new("my-instance-id".to_string());

// Register peer for pulling from it
manager.register_peer("peer-1".to_string(), "pull".to_string()).await;

// Get sync state
if let Some(state) = manager.get_sync_state("peer-1").await {
    println!("Last sync: {} items", state.last_sync_item_count);
}

// Update after successful sync
manager.update_sync_state(
    "peer-1".to_string(),
    "new_db_version".to_string(),
    42, // items synced
).await;
```

## Deployment Scenarios

### Single Subnet (Recommended)

Instances on same /24 subnet automatically discover and sync:

```
192.168.1.100 - dumptruck-1 ↔ UDP broadcast ↔ dumptruck-2 - 192.168.1.101
192.168.1.102 - dumptruck-3
```

Each instance:

- Broadcasts presence every 30 seconds
- Discovers peers in 30-120 seconds
- Begins delta sync automatically
- Merges new data within 5 minutes

**Bandwidth**: ~120 bytes/30 seconds per broadcast + sync responses (typically 1-10 KB depending on data changes)

### Multiple Subnets

For multi-subnet deployments, enable cross-subnet sync via:

- Redis pub/sub for peer discovery
- HTTP API for sync requests (not yet implemented, planned for M5)
- Or configure static peers via config file

## Security Considerations

1. **UDP Broadcast Trust**: Assumes same network == trusted
   + Implement network segmentation if untrusted networks possible
   + Use firewall rules to restrict UDP 49999 to trusted subnets

2. **Sync Data Integrity**:
   + All sync data checksummed before merge
   + Invalid data logged and rejected
   + Peer reputation tracking planned for M5

3. **Database Versioning**:
   + Hash tracks database state
   + Protects against merge conflicts
   + Enables auditing of data flow between peers

## Performance

**Peer Discovery**:

- UDP listen: Non-blocking, negligible CPU
- Broadcast overhead: ~120 bytes/30 seconds = 32 bps per instance
- Memory: <1 KB per peer (max 32 = <32 KB)

**Synchronization**:

- Bloom filter size: 1 MB (8M bits, k=3)
- Typical sync request size: 1-2 MB (includes filter)
- Typical sync response size: 1-100 KB (depends on new data)
- Frequency: On-demand, triggered by peer addition or config change
- Bandwidth savings: 10-100x vs naive sync (due to Bloom filter filtering)

**Example**:

- Instance A has 100k addresses
- Instance B has 80k addresses (60% overlap)
- Naive sync: Would transmit 80-160 KB per sync
- With Bloom filter: Transmits 1 MB filter (detected in ~60ms), 20-40 KB response (~8-10x compression)

## Testing

All peer discovery and sync modules include comprehensive unit tests:

```bash
# Run peer tests
cargo test --lib peer_discovery peer_sync

# Test Bloom filter
cargo test --lib peer_sync::tests::test_bloom_filter_

# Test discovery
cargo test --lib peer_discovery::tests::test_peer_registry_
```

## Future Enhancements (M5)

1. **HTTP Sync API**: RESTful API for cross-subnet sync
2. **Redis Discovery**: Use Redis pub/sub for dynamic discovery
3. **Peer Reputation**: Track sync success rate, prioritize reliable peers
4. **Compression**: GZIP compress sync responses
5. **Conflict Resolution**: Automated merging of conflicting data
6. **Backup Sync**: Periodic full database sync for disaster recovery

## References

- **Bloom Filters**: [Wikipedia](https://en.wikipedia.org/wiki/Bloom_filter)
- **Delta Sync**: [RSYNC Algorithm](https://rsync.samba.org/tech_report/)
- **Peer-to-Peer**: [Kademlia DHT](https://en.wikipedia.org/wiki/Kademlia)
