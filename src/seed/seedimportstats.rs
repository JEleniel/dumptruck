use serde::{Deserialize, Serialize};

/// Statistics from importing seed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedImportStats {
	/// Number of duplicate files found
	pub duplicate_files: usize,
	/// Number of unique files processed
	pub new_files: usize,
	/// Number of rows imported
	pub rows_imported: usize,
	/// Time taken to import in milliseconds
	pub import_time_ms: u64,
}
