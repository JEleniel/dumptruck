//! Seed feature for bulk initialization of Dumptruck database.
//!
//! The seed feature enables:
//! - Creating a seed database from a folder of data files
//! - Computing deterministic signatures for change detection
//! - Automatic import on startup if seed data has changed
//!
//! # Architecture
//!
//! ```text
//! seed <folder>
//!   → SeedBuilder discovers files recursively
//!   → Computes SHA-256 signature of all file contents
//!   → Processes through full ingest pipeline
//!   → Creates seed.db with deduplicated data
//!   → Stores metadata with signature for verification
//!
//! On startup:
//!   → SeedManager loads metadata from seed.db
//!   → Recomputes signature from seed folder
//!   → If signature differs: import all seed data into main db
//!   → If signature matches: skip import (cached)
//! ```

pub mod builder;
pub mod manager;

pub use builder::{SeedBuilder, SeedError, SeedInfo};
pub use manager::{SeedImportStats, SeedManager, SeedMetadata};

use serde::{Deserialize, Serialize};

/// Result type for seed operations
pub type SeedResult<T> = Result<T, SeedError>;

/// Manifest of files included in a seed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
	/// List of relative file paths in seed
	pub files: Vec<String>,
	/// Total size of all files in bytes
	pub total_size: u64,
	/// File count
	pub file_count: usize,
}

/// Statistics from seed creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedStats {
	/// Total rows processed from all files
	pub total_rows: usize,
	/// Unique addresses deduplicated
	pub unique_addresses: usize,
	/// Number of files discovered
	pub file_count: usize,
	/// Total size of input data
	pub total_size: u64,
	/// Estimated import time on next startup (seconds)
	pub estimated_import_time_secs: u64,
}
