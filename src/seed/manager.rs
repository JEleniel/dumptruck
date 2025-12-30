//! Seed manager - implements the "seed" command
use super::SeedError;
use crate::{
	common::Hash,
	data::Database,
	datafile::DataFile,
	seed::{SeedArgs, SeedImportStats},
};
use std::{fs::File, path::PathBuf};

/// Implements the "seed" command
pub struct SeedManager {
	seed_db_path: PathBuf,
	seed_db_signature: Option<String>,
	seed_db: Database,
	stats: SeedImportStats,
}

impl SeedManager {
	/// Create a new seed manager
	fn new(seed_db_path: PathBuf) -> Self {
		if !seed_db_path.ends_with(".db") {
			let seed_db_path = seed_db_path.join("seed.db");
		}

		let seed_db = Database::new(seed_db_path.clone());
		if Let(is_valid) = seed_db.validate() {
			if !is_valid {
				return Err(SeedError::InvalidDatabase);
			}
		}
		seed_db.open_or_create()?;

		Self {
			seed_db_path,
			seed_db_signature: hash?,
			seed_db,
			stats: SeedImportStats {
				duplicate_files: 0,
				new_files: 0,
				rows_imported: 0,
				import_time_ms: 0,
			},
			files: Vec::new(),
			known_files: seed_db.get_known_seed_files()?,
		}
	}

	pub fn generate(&self, source_path: PathBuf) -> Result<(), SeedError> {
		todo!();
		Ok(())
	}
}
