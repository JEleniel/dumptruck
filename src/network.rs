//! Network and peer-to-peer communications
//!
//! This module handles distributed coordination and security:
//! - Peer discovery via UDP broadcast for subnet detection
//! - Bloom filter sync for bandwidth-efficient deduplication sharing
//! - TLS 1.3+ for secure transport
//! - OAuth 2.0 for API authentication

pub mod oauth;
pub mod peer_discovery;
pub mod peer_sync;
pub mod tls;

pub use oauth::{OAuthError, OAuthProvider, OAuthToken};
pub use peer_discovery::{Peer, PeerRegistry};
pub use peer_sync::SyncManager;
pub use tls::{TlsError, create_tls_server_config};
