// Peer Synchronization for Dumptruck
//
// Implements Bloom filter-based delta sync to minimize bandwidth when
// synchronizing deduplication and enrichment data between peers.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Bloom filter for efficient membership testing
/// Uses k=3 hash functions for good false positive/true negative tradeoff
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BloomFilter {
	/// Bit vector (each u8 = 8 bits)
	bits: Vec<u8>,
	/// Size in bits
	size: usize,
	/// Number of hash functions (k)
	k: usize,
}

impl BloomFilter {
	/// Create a new Bloom filter with given bit size
	/// Typical size: 1MB = 8388608 bits for ~100k items with 1% FP rate
	pub fn new(size_bits: usize) -> Self {
		let num_bytes = size_bits.div_ceil(8);
		Self {
			bits: vec![0u8; num_bytes],
			size: size_bits,
			k: 3, // 3 hash functions
		}
	}

	/// Add an item to the filter
	pub fn insert(&mut self, item: &str) {
		for i in 0..self.k {
			let hash = self.hash(item, i);
			let bit_pos = hash % self.size;
			let byte_pos = bit_pos / 8;
			let bit_offset = (bit_pos % 8) as u32;
			self.bits[byte_pos] |= 1 << bit_offset;
		}
	}

	/// Check if an item might be in the filter (may have false positives)
	pub fn contains(&self, item: &str) -> bool {
		for i in 0..self.k {
			let hash = self.hash(item, i);
			let bit_pos = hash % self.size;
			let byte_pos = bit_pos / 8;
			let bit_offset = (bit_pos % 8) as u32;
			if (self.bits[byte_pos] & (1 << bit_offset)) == 0 {
				return false;
			}
		}
		true
	}

	/// Hash an item with seed i
	fn hash(&self, item: &str, seed: usize) -> usize {
		let mut hasher = DefaultHasher::new();
		item.hash(&mut hasher);
		seed.hash(&mut hasher);

		hasher.finish() as usize
	}

	/// Get the filter as bytes for transmission
	pub fn as_bytes(&self) -> &[u8] {
		&self.bits
	}

	/// Create from bytes (for receiving)
	pub fn from_bytes(bytes: Vec<u8>, size: usize) -> Self {
		Self {
			bits: bytes,
			size,
			k: 3,
		}
	}

	/// Get size in bytes
	pub fn size_bytes(&self) -> usize {
		self.bits.len()
	}
}

/// Sync request - sent from one peer to another
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SyncRequest {
	/// Requesting instance ID
	pub requester_id: String,
	/// Requesting instance's database version
	pub requester_db_version: String,
	/// Bloom filter of hashes known to requester (addresses they've seen)
	pub known_hashes_filter: Vec<u8>,
	/// Number of items in the Bloom filter
	pub filter_size_bits: usize,
}

impl SyncRequest {
	pub fn new(
		requester_id: String,
		requester_db_version: String,
		known_hashes_filter: Vec<u8>,
		filter_size_bits: usize,
	) -> Self {
		Self {
			requester_id,
			requester_db_version,
			known_hashes_filter,
			filter_size_bits,
		}
	}
}

/// Sync response - sent by peer with new data
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SyncResponse {
	/// Responder instance ID
	pub responder_id: String,
	/// Responder's database version after this sync
	pub responder_db_version: String,
	/// New addresses unknown to requester
	pub new_addresses: Vec<String>,
	/// New canonical address mappings
	pub new_canonical_mappings: Vec<(String, String)>, // (variant, canonical)
	/// New breach associations
	pub new_breaches: Vec<(String, String)>, // (address_hash, breach_name)
}

impl SyncResponse {
	pub fn new(responder_id: String, responder_db_version: String) -> Self {
		Self {
			responder_id,
			responder_db_version,
			new_addresses: Vec::new(),
			new_canonical_mappings: Vec::new(),
			new_breaches: Vec::new(),
		}
	}

	/// Get size estimate in bytes
	pub fn size_estimate(&self) -> usize {
		// Rough estimate: 64 bytes per address + 48 bytes per mapping + 64 bytes per breach
		(self.new_addresses.len() * 64)
			+ (self.new_canonical_mappings.len() * 48)
			+ (self.new_breaches.len() * 64)
	}
}

/// Sync state tracker for a peer
#[derive(Clone, Debug)]
pub struct SyncState {
	/// Peer instance ID
	pub peer_id: String,
	/// Last known peer database version
	pub last_synced_db_version: String,
	/// Number of items synced in last exchange
	pub last_sync_item_count: usize,
	/// Timestamp of last sync (Unix epoch seconds)
	pub last_sync_time: u64,
	/// Sync direction: "pull" (we request from peer), "push" (peer requests from us)
	pub direction: String,
}

impl SyncState {
	pub fn new(peer_id: String, direction: String) -> Self {
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		Self {
			peer_id,
			last_synced_db_version: String::new(),
			last_sync_item_count: 0,
			last_sync_time: now,
			direction,
		}
	}

	/// Check if sync is stale (not synced for > 300 seconds / 5 minutes)
	pub fn is_stale(&self) -> bool {
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();
		(now - self.last_sync_time) > 300
	}
}

/// Sync manager - coordinates all peer synchronization
pub struct SyncManager {
	/// Local instance ID
	pub instance_id: String,
	/// Sync states indexed by peer instance_id
	sync_states: Arc<tokio::sync::RwLock<std::collections::HashMap<String, SyncState>>>,
}

impl SyncManager {
	pub fn new(instance_id: String) -> Self {
		Self {
			instance_id,
			sync_states: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
		}
	}

	/// Register a peer for synchronization
	pub async fn register_peer(&self, peer_id: String, direction: String) {
		let mut states = self.sync_states.write().await;
		states.insert(peer_id.clone(), SyncState::new(peer_id, direction));
	}

	/// Get sync state for a peer
	pub async fn get_sync_state(&self, peer_id: &str) -> Option<SyncState> {
		let states = self.sync_states.read().await;
		states.get(peer_id).cloned()
	}

	/// Update sync state after successful sync
	pub async fn update_sync_state(&self, peer_id: String, db_version: String, item_count: usize) {
		let mut states = self.sync_states.write().await;
		if let Some(state) = states.get_mut(&peer_id) {
			state.last_synced_db_version = db_version;
			state.last_sync_item_count = item_count;
			state.last_sync_time = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs();
		}
	}

	/// Get all sync states
	pub async fn get_all_sync_states(&self) -> Vec<SyncState> {
		let states = self.sync_states.read().await;
		states.values().cloned().collect()
	}

	/// Get count of stale sync states
	pub async fn get_stale_sync_count(&self) -> usize {
		let states = self.sync_states.read().await;
		states.values().filter(|s| s.is_stale()).count()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_bloom_filter_creation() {
		let filter = BloomFilter::new(1000);
		assert_eq!(filter.size, 1000);
		assert_eq!(filter.k, 3);
	}

	#[test]
	fn test_bloom_filter_insert_and_contains() {
		let mut filter = BloomFilter::new(10000);
		filter.insert("test_hash_1");
		filter.insert("test_hash_2");

		assert!(filter.contains("test_hash_1"));
		assert!(filter.contains("test_hash_2"));
		assert!(!filter.contains("test_hash_3")); // Should be false (may have false positive, but unlikely)
	}

	#[test]
	fn test_bloom_filter_serialization() {
		let mut filter = BloomFilter::new(1000);
		filter.insert("item1");
		filter.insert("item2");

		let json = serde_json::to_string(&filter).unwrap();
		let restored: BloomFilter = serde_json::from_str(&json).unwrap();

		assert!(restored.contains("item1"));
		assert!(restored.contains("item2"));
	}

	#[test]
	fn test_bloom_filter_from_bytes() {
		let mut filter = BloomFilter::new(1000);
		filter.insert("test_item");

		let bytes = filter.as_bytes().to_vec();
		let restored = BloomFilter::from_bytes(bytes, 1000);

		assert!(restored.contains("test_item"));
	}

	#[test]
	fn test_sync_request_creation() {
		let request = SyncRequest::new(
			"instance-1".to_string(),
			"version-abc".to_string(),
			vec![0, 1, 2, 3],
			1000,
		);

		assert_eq!(request.requester_id, "instance-1");
		assert_eq!(request.requester_db_version, "version-abc");
		assert_eq!(request.filter_size_bits, 1000);
	}

	#[test]
	fn test_sync_response_creation() {
		let mut response = SyncResponse::new("instance-2".to_string(), "version-def".to_string());
		response.new_addresses.push("hash1".to_string());
		response
			.new_canonical_mappings
			.push(("variant1".to_string(), "canonical1".to_string()));

		assert_eq!(response.responder_id, "instance-2");
		assert_eq!(response.new_addresses.len(), 1);
		assert_eq!(response.new_canonical_mappings.len(), 1);
		assert!(response.size_estimate() > 0);
	}

	#[test]
	fn test_sync_request_serialization() {
		let request = SyncRequest::new(
			"instance-1".to_string(),
			"version-abc".to_string(),
			vec![0, 1, 2, 3],
			1000,
		);

		let json = serde_json::to_string(&request).unwrap();
		let restored: SyncRequest = serde_json::from_str(&json).unwrap();

		assert_eq!(restored.requester_id, request.requester_id);
		assert_eq!(restored.requester_db_version, request.requester_db_version);
	}

	#[test]
	fn test_sync_state_creation() {
		let state = SyncState::new("peer-1".to_string(), "pull".to_string());
		assert_eq!(state.peer_id, "peer-1");
		assert_eq!(state.direction, "pull");
		assert_eq!(state.last_sync_item_count, 0);
	}

	#[tokio::test]
	async fn test_sync_manager_register_and_get() {
		let manager = SyncManager::new("instance-1".to_string());
		manager
			.register_peer("peer-1".to_string(), "pull".to_string())
			.await;

		let state = manager.get_sync_state("peer-1").await;
		assert!(state.is_some());
		assert_eq!(state.unwrap().peer_id, "peer-1");
	}

	#[tokio::test]
	async fn test_sync_manager_update_state() {
		let manager = SyncManager::new("instance-1".to_string());
		manager
			.register_peer("peer-1".to_string(), "pull".to_string())
			.await;

		manager
			.update_sync_state("peer-1".to_string(), "version-xyz".to_string(), 42)
			.await;

		let state = manager.get_sync_state("peer-1").await.unwrap();
		assert_eq!(state.last_synced_db_version, "version-xyz");
		assert_eq!(state.last_sync_item_count, 42);
	}

	#[tokio::test]
	async fn test_sync_manager_stale_detection() {
		let manager = SyncManager::new("instance-1".to_string());
		manager
			.register_peer("peer-1".to_string(), "pull".to_string())
			.await;

		// Immediately after registration, should not be stale
		let state = manager.get_sync_state("peer-1").await.unwrap();
		assert!(!state.is_stale());
	}

	#[test]
	fn test_sync_response_serialization() {
		let mut response = SyncResponse::new("instance-2".to_string(), "version-def".to_string());
		response.new_addresses.push("hash1".to_string());

		let json = serde_json::to_string(&response).unwrap();
		let restored: SyncResponse = serde_json::from_str(&json).unwrap();

		assert_eq!(restored.responder_id, response.responder_id);
		assert_eq!(restored.new_addresses.len(), 1);
	}
}
