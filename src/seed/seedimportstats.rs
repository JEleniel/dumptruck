use serde::{Deserialize, Serialize};

/// Statistics from importing seed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedImportStats {
	/// Number of files found
	pub total_seed_files: usize,
	/// Number of rainbow table files found
	pub total_rainbowtable_files: usize,
	/// Number of duplicate files found
	pub duplicate_seed_files: usize,
	/// Number of rows imported
	pub rows_imported: usize,
	/// Time taken to import in milliseconds
	pub import_time_ms: u64,
	/// Number of new credentials imported
	pub new_credentials: usize,
	/// Number of new identities imported
	pub new_identities: usize,
	/// Number of new metadata entries imported
	pub new_metadata: usize,
	/// Number of new PII entries imported
	pub new_pii: usize,
	/// Number of new rainbow table entries imported
	pub new_rainbowtable_entries: usize,
}
