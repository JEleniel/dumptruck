//! Seed manager - verifies signatures and imports seed data on startup.

use std::{fs::File, path::PathBuf};

use crate::{
	common::Hash,
	data::Database,
	datafile::DataFile,
	seed::{SeedArgs, SeedImportStats},
};

use super::SeedError;

/// Manager of seed databases
pub struct SeedManager {
	seed_db_path: PathBuf,
	seed_db_signature: Option<String>,
	seed_db: Database,
	stats: SeedImportStats,
	files: Vec<DataFile>,
	known_files: Vec<DataFile>,
}

impl SeedManager {
	pub async fn run(args: SeedArgs) -> Result<(), SeedError> {
		let manager = SeedManager::new(args.seed_db_path)?;

		if manager.changed()? {
			manager.generate(args.source_path.clone())?;
		}

		Ok(())
	}

	/// Create a new seed manager
	fn new(seed_db_path: PathBuf) -> Self {
		let seed_db_path = seed_db_path.join("seed.db");

		// Hash an existing seed db
		let hash = if File::exists(seed_db_path) {
			Ok(Hash::calculate_md5(&mut File::open(seed_db_path)?)?)
		} else {
			Ok(None)
		};

		let seed_db = Database::new(seed_db_path.clone());
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
