// Peer Discovery for Dumptruck
//
// Implements UDP broadcast-based peer discovery allowing instances on the same
// subnet to discover each other and synchronize deduplication and enrichment data.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Default UDP broadcast port for peer discovery
const DEFAULT_DISCOVERY_PORT: u16 = 49999;

/// Discovery message broadcast interval (seconds)
const DISCOVERY_INTERVAL_SECS: u64 = 30;

/// Peer timeout - remove peers not seen for this duration (seconds)
const PEER_TIMEOUT_SECS: u64 = 120;

/// Maximum peers per subnet to prevent explosion
const MAX_PEERS_PER_SUBNET: usize = 32;

/// Peer discovery message sent via UDP broadcast
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DiscoveryMessage {
	/// Instance identifier (UUID v4)
	pub instance_id: String,
	/// Hostname of the peer
	pub hostname: String,
	/// Dumptruck version
	pub version: String,
	/// Timestamp (Unix epoch seconds)
	pub timestamp: u64,
	/// Sync server port on the peer
	pub sync_port: u16,
	/// Current database version hash
	pub db_version: String,
	/// IPv4 address of the peer
	pub ipv4: String,
}

impl DiscoveryMessage {
	fn new(
		instance_id: String,
		hostname: String,
		version: String,
		sync_port: u16,
		db_version: String,
		ipv4: String,
	) -> Self {
		let timestamp = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		Self {
			instance_id,
			hostname,
			version,
			timestamp,
			sync_port,
			db_version,
			ipv4,
		}
	}
}

/// Peer information tracked in the registry
#[derive(Clone, Debug)]
pub struct Peer {
	/// Instance identifier
	pub instance_id: String,
	/// Hostname
	pub hostname: String,
	/// Sync server address
	pub sync_addr: SocketAddr,
	/// Database version hash
	pub db_version: String,
	/// Last seen timestamp (Unix epoch seconds)
	pub last_seen: u64,
	/// Dumptruck version
	pub version: String,
}

impl Peer {
	fn from_discovery_message(msg: &DiscoveryMessage) -> Result<Self, String> {
		let ip: IpAddr = msg
			.ipv4
			.parse()
			.map_err(|_| "Invalid IPv4 address in discovery message")?;
		let sync_addr = SocketAddr::new(ip, msg.sync_port);

		Ok(Self {
			instance_id: msg.instance_id.clone(),
			hostname: msg.hostname.clone(),
			sync_addr,
			db_version: msg.db_version.clone(),
			last_seen: msg.timestamp,
			version: msg.version.clone(),
		})
	}
}

/// Peer registry with thread-safe access
pub struct PeerRegistry {
	/// Local instance ID
	pub instance_id: String,
	/// Peers indexed by instance_id
	peers: Arc<RwLock<HashMap<String, Peer>>>,
	/// Local IPv4 address
	local_ipv4: Ipv4Addr,
	/// Detected subnet (e.g., 192.168.1.0/24)
	subnet: String,
}

impl PeerRegistry {
	/// Create a new peer registry
	pub fn new(local_ipv4: Ipv4Addr) -> Self {
		let instance_id = Uuid::new_v4().to_string();
		let subnet = Self::calculate_subnet(&local_ipv4);

		Self {
			instance_id,
			peers: Arc::new(RwLock::new(HashMap::new())),
			local_ipv4,
			subnet,
		}
	}

	/// Calculate broadcast address for subnet (assumes /24)
	fn calculate_subnet(ipv4: &Ipv4Addr) -> String {
		let octets = ipv4.octets();
		format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
	}

	/// Get broadcast address for subnet (assumes /24)
	pub fn get_broadcast_addr(&self) -> Ipv4Addr {
		let octets = self.local_ipv4.octets();
		Ipv4Addr::new(octets[0], octets[1], octets[2], 255)
	}

	/// Add or update a peer
	pub async fn add_peer(&self, peer: Peer) {
		let mut peers = self.peers.write().await;

		// Don't track ourselves
		if peer.instance_id == self.instance_id {
			return;
		}

		// Enforce max peers per subnet
		if peers.len() >= MAX_PEERS_PER_SUBNET && !peers.contains_key(&peer.instance_id) {
			return;
		}

		peers.insert(peer.instance_id.clone(), peer);
	}

	/// Get all known peers
	pub async fn get_peers(&self) -> Vec<Peer> {
		let peers = self.peers.read().await;
		peers.values().cloned().collect()
	}

	/// Get a specific peer by instance_id
	pub async fn get_peer(&self, instance_id: &str) -> Option<Peer> {
		let peers = self.peers.read().await;
		peers.get(instance_id).cloned()
	}

	/// Remove stale peers (not seen for PEER_TIMEOUT_SECS)
	pub async fn cleanup_stale_peers(&self) {
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		let mut peers = self.peers.write().await;
		peers.retain(|_, peer| (now - peer.last_seen) < PEER_TIMEOUT_SECS);
	}

	/// Get peer count
	pub async fn peer_count(&self) -> usize {
		let peers = self.peers.read().await;
		peers.len()
	}

	/// Get local subnet
	pub fn subnet(&self) -> &str {
		&self.subnet
	}

	/// Get local IPv4
	pub fn local_ipv4(&self) -> Ipv4Addr {
		self.local_ipv4
	}
}

/// UDP broadcast discovery listener
pub struct DiscoveryListener {
	socket: UdpSocket,
	registry: Arc<PeerRegistry>,
	local_hostname: String,
	version: String,
	sync_port: u16,
}

impl DiscoveryListener {
	/// Create a new discovery listener
	pub fn new(
		registry: Arc<PeerRegistry>,
		local_hostname: String,
		version: String,
		sync_port: u16,
	) -> Result<Self, String> {
		// Bind to 0.0.0.0:49999 to receive broadcasts
		let addr = format!("0.0.0.0:{}", DEFAULT_DISCOVERY_PORT);
		let socket = UdpSocket::bind(&addr)
			.map_err(|e| format!("Failed to bind discovery socket: {}", e))?;

		socket
			.set_read_timeout(Some(Duration::from_secs(5)))
			.map_err(|e| format!("Failed to set socket timeout: {}", e))?;

		Ok(Self {
			socket,
			registry,
			local_hostname,
			version,
			sync_port,
		})
	}

	/// Receive and process discovery messages
	async fn receive_messages(&self) -> Result<(), String> {
		let mut buf = vec![0u8; 512];

		loop {
			match self.socket.recv_from(&mut buf) {
				Ok((n, _)) => {
					let msg_data = &buf[..n];
					if let Ok(msg) = serde_json::from_slice::<DiscoveryMessage>(msg_data) {
						// Process received discovery message
						if let Ok(peer) = Peer::from_discovery_message(&msg) {
							self.registry.add_peer(peer).await;
						}
					}
				}
				Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
					// Timeout - continue
					continue;
				}
				Err(e) => {
					return Err(format!("Error receiving discovery message: {}", e));
				}
			}
		}
	}

	/// Broadcast our presence
	async fn broadcast_presence(&self, db_version: String) -> Result<(), String> {
		let message = DiscoveryMessage::new(
			self.registry.instance_id.clone(),
			self.local_hostname.clone(),
			self.version.clone(),
			self.sync_port,
			db_version,
			self.registry.local_ipv4().to_string(),
		);

		let msg_json = serde_json::to_string(&message)
			.map_err(|e| format!("Failed to serialize discovery message: {}", e))?;

		let broadcast_addr = self.registry.get_broadcast_addr();
		let addr = format!("{}:{}", broadcast_addr, DEFAULT_DISCOVERY_PORT);

		self.socket
			.send_to(msg_json.as_bytes(), &addr)
			.map_err(|e| format!("Failed to broadcast discovery message: {}", e))?;

		Ok(())
	}

	/// Start the discovery service
	/// Spawns background tasks for receiving and broadcasting discovery messages
	pub async fn start(
		self: Arc<Self>,
		db_version_rx: tokio::sync::watch::Receiver<String>,
	) -> Result<(), String> {
		// Task 1: Receive discovery messages
		let self_recv = self.clone();
		tokio::spawn(async move {
			let _ = self_recv.receive_messages().await;
		});

		// Task 2: Broadcast presence periodically
		let self_bcast = self.clone();
		let mut db_version_rx = db_version_rx; // watch receiver needs to be mutable
		tokio::spawn(async move {
			let mut interval = tokio::time::interval(Duration::from_secs(DISCOVERY_INTERVAL_SECS));

			loop {
				interval.tick().await;
				let db_version = db_version_rx.borrow().clone();
				let _ = self_bcast.broadcast_presence(db_version).await;
			}
		});

		// Task 3: Clean up stale peers periodically
		let registry = self.registry.clone();
		tokio::spawn(async move {
			let mut interval = tokio::time::interval(Duration::from_secs(60));

			loop {
				interval.tick().await;
				registry.cleanup_stale_peers().await;
			}
		});

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_discovery_message_creation() {
		let msg = DiscoveryMessage::new(
			"test-id".to_string(),
			"test-host".to_string(),
			"1.0.0".to_string(),
			8444,
			"abc123".to_string(),
			"192.168.1.100".to_string(),
		);

		assert_eq!(msg.instance_id, "test-id");
		assert_eq!(msg.hostname, "test-host");
		assert_eq!(msg.version, "1.0.0");
		assert_eq!(msg.sync_port, 8444);
		assert!(msg.timestamp > 0);
	}

	#[test]
	fn test_peer_from_discovery_message() {
		let msg = DiscoveryMessage::new(
			"test-id".to_string(),
			"test-host".to_string(),
			"1.0.0".to_string(),
			8444,
			"abc123".to_string(),
			"192.168.1.100".to_string(),
		);

		let peer = Peer::from_discovery_message(&msg).unwrap();
		assert_eq!(peer.instance_id, "test-id");
		assert_eq!(peer.hostname, "test-host");
		assert_eq!(peer.sync_addr.port(), 8444);
	}

	#[test]
	fn test_subnet_calculation() {
		let ip = Ipv4Addr::new(192, 168, 1, 100);
		let subnet = PeerRegistry::calculate_subnet(&ip);
		assert_eq!(subnet, "192.168.1.0/24");
	}

	#[test]
	fn test_broadcast_address() {
		let ip = Ipv4Addr::new(192, 168, 1, 100);
		let registry = PeerRegistry::new(ip);
		let broadcast = registry.get_broadcast_addr();
		assert_eq!(broadcast, Ipv4Addr::new(192, 168, 1, 255));
	}

	#[tokio::test]
	async fn test_peer_registry_add_and_get() {
		let registry = PeerRegistry::new(Ipv4Addr::new(192, 168, 1, 100));

		let peer = Peer {
			instance_id: "peer-1".to_string(),
			hostname: "peer-host".to_string(),
			sync_addr: "192.168.1.50:8444".parse().unwrap(),
			db_version: "abc123".to_string(),
			last_seen: 1000,
			version: "1.0.0".to_string(),
		};

		registry.add_peer(peer.clone()).await;
		assert_eq!(registry.peer_count().await, 1);

		let retrieved = registry.get_peer("peer-1").await;
		assert!(retrieved.is_some());
		assert_eq!(retrieved.unwrap().hostname, "peer-host");
	}

	#[tokio::test]
	async fn test_peer_registry_ignores_self() {
		let registry = PeerRegistry::new(Ipv4Addr::new(192, 168, 1, 100));
		let self_id = registry.instance_id.clone();

		let peer = Peer {
			instance_id: self_id,
			hostname: "self-host".to_string(),
			sync_addr: "192.168.1.100:8444".parse().unwrap(),
			db_version: "abc123".to_string(),
			last_seen: 1000,
			version: "1.0.0".to_string(),
		};

		registry.add_peer(peer).await;
		assert_eq!(registry.peer_count().await, 0);
	}

	#[tokio::test]
	async fn test_peer_registry_max_peers() {
		let registry = PeerRegistry::new(Ipv4Addr::new(192, 168, 1, 100));

		// Add MAX_PEERS_PER_SUBNET peers
		for i in 0..MAX_PEERS_PER_SUBNET {
			let peer = Peer {
				instance_id: format!("peer-{}", i),
				hostname: format!("peer-{}", i),
				sync_addr: format!("192.168.1.{}:8444", i + 10).parse().unwrap(),
				db_version: "abc123".to_string(),
				last_seen: 1000,
				version: "1.0.0".to_string(),
			};
			registry.add_peer(peer).await;
		}

		assert_eq!(registry.peer_count().await, MAX_PEERS_PER_SUBNET);

		// Try to add one more - should be rejected
		let extra = Peer {
			instance_id: "extra-peer".to_string(),
			hostname: "extra".to_string(),
			sync_addr: "192.168.1.200:8444".parse().unwrap(),
			db_version: "abc123".to_string(),
			last_seen: 1000,
			version: "1.0.0".to_string(),
		};
		registry.add_peer(extra).await;

		// Count should still be MAX
		assert_eq!(registry.peer_count().await, MAX_PEERS_PER_SUBNET);
	}

	#[test]
	fn test_discovery_message_serialization() {
		let msg = DiscoveryMessage::new(
			"test-id".to_string(),
			"test-host".to_string(),
			"1.0.0".to_string(),
			8444,
			"abc123".to_string(),
			"192.168.1.100".to_string(),
		);

		let json = serde_json::to_string(&msg).unwrap();
		let parsed: DiscoveryMessage = serde_json::from_str(&json).unwrap();

		assert_eq!(parsed.instance_id, msg.instance_id);
		assert_eq!(parsed.hostname, msg.hostname);
	}
}
