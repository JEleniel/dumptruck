//! Seed builder - discovers data files and creates seed database with signature tracking.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Information about a created seed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedInfo {
	/// Path to seed database created
	pub seed_db_path: PathBuf,
	/// Path to folder seed was created from
	pub folder_path: PathBuf,
	/// Number of files discovered
	pub files_discovered: usize,
	/// Total rows processed
	pub total_rows: usize,
	/// Unique addresses in seed
	pub unique_addresses: usize,
	/// SHA-256 signature of all file contents
	pub file_signature: String,
	/// Unix timestamp of seed creation
	pub created_at: u64,
	/// Estimated import time in minutes
	pub estimated_import_time_minutes: u64,
}

/// Builder for creating seed databases from folders
pub struct SeedBuilder {
	folder_path: PathBuf,
	output_path: PathBuf,
}

impl SeedBuilder {
	/// Create a new seed builder
	///
	/// # Arguments
	/// * `folder_path` - Path to folder containing data files
	/// * `output_path` - Path for output seed database (default: data/seed.db)
	pub fn new(folder_path: PathBuf, output_path: Option<PathBuf>) -> Self {
		let output = output_path.unwrap_or_else(|| PathBuf::from("data/seed.db"));
		Self {
			folder_path,
			output_path: output,
		}
	}

	/// Discover all data files recursively in folder
	pub fn discover_files(&self) -> Result<Vec<PathBuf>, SeedError> {
		if !self.folder_path.exists() {
			return Err(SeedError::FolderNotFound(
				self.folder_path.display().to_string(),
			));
		}

		let mut files = Vec::new();

		self.walk_directory(&self.folder_path, &mut files)?;

		if files.is_empty() {
			return Err(SeedError::NoDataFiles(
				self.folder_path.display().to_string(),
			));
		}

		// Sort for deterministic ordering
		files.sort();
		Ok(files)
	}

	/// Recursively walk directory and find data files
	fn walk_directory(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), SeedError> {
		for entry in fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();

			if path.is_dir() {
				self.walk_directory(&path, files)?;
			} else if self.is_data_file(&path) {
				files.push(path);
			}
		}
		Ok(())
	}

	/// Check if path is a recognized data file
	fn is_data_file(&self, path: &Path) -> bool {
		if let Some(ext) = path.extension() {
			let ext_str = ext.to_string_lossy().to_lowercase();
			matches!(
				ext_str.as_str(),
				"csv" | "tsv" | "json" | "xml" | "yaml" | "yml" | "jsonl"
			)
		} else {
			false
		}
	}

	/// Compute deterministic SHA-256 signature of seed.db file
	///
	/// This is called AFTER the seed database is created to get its signature.
	/// Returns hex-encoded SHA-256 hash of the seed.db file contents.
	///
	/// # Arguments
	/// * `seed_db_path` - Path to the created seed.db file
	pub fn compute_seed_signature(seed_db_path: &Path) -> Result<String, SeedError> {
		let mut file = fs::File::open(seed_db_path).map_err(|e| {
			SeedError::SignatureError(format!(
				"Failed to read seed database {}: {}",
				seed_db_path.display(),
				e
			))
		})?;

		let mut hasher = Sha256::new();
		let mut buffer = [0u8; 4096];

		loop {
			let bytes_read = file.read(&mut buffer).map_err(|e| {
				SeedError::SignatureError(format!(
					"Failed to read seed database {}: {}",
					seed_db_path.display(),
					e
				))
			})?;

			if bytes_read == 0 {
				break;
			}
			hasher.update(&buffer[..bytes_read]);
		}

		let result = hasher.finalize();
		Ok(format!("{:x}", result))
	}

	/// Build the seed database (placeholder - actual implementation in handlers)
	///
	/// This function prepares the seed builder for use. Actual database
	/// population happens in the handlers where ingest pipeline runs.
	pub fn build(&self) -> Result<SeedInfo, SeedError> {
		let files = self.discover_files()?;
		let file_count = files.len();
		let total_size: u64 = files
			.iter()
			.flat_map(|f| fs::metadata(f).ok())
			.map(|m| m.len())
			.sum();

		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map_err(|e| SeedError::SignatureError(format!("Failed to get current time: {}", e)))?
			.as_secs();

		// Estimate: ~100KB/sec processing on modern hardware
		let estimated_secs = (total_size / (100 * 1024)).max(1);
		let estimated_minutes = (estimated_secs / 60).max(1);

		// Signature will be computed after seed.db is created
		// Using empty string as placeholder - actual signature set in handlers
		Ok(SeedInfo {
			seed_db_path: self.output_path.clone(),
			folder_path: self.folder_path.clone(),
			files_discovered: file_count,
			total_rows: 0,                 // Will be updated after ingest
			unique_addresses: 0,           // Will be updated after ingest
			file_signature: String::new(), // Computed after seed.db created
			created_at: now,
			estimated_import_time_minutes: estimated_minutes,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::TempDir;

	#[test]
	fn test_discover_files_recursive() {
		let temp_dir = TempDir::new().unwrap();
		let root = temp_dir.path();

		// Create test files
		fs::write(root.join("file1.csv"), "test1").unwrap();
		fs::create_dir(root.join("subdir")).unwrap();
		fs::write(root.join("subdir/file2.json"), "test2").unwrap();

		let builder = SeedBuilder::new(root.to_path_buf(), None);
		let files = builder.discover_files().unwrap();

		assert_eq!(files.len(), 2);
	}

	#[test]
	fn test_signature_deterministic() {
		let temp_dir = TempDir::new().unwrap();
		let root = temp_dir.path();

		// Create a seed.db file
		let seed_db_path = root.join("seed.db");
		fs::write(&seed_db_path, "test content").unwrap();

		let sig1 = SeedBuilder::compute_seed_signature(&seed_db_path).unwrap();
		let sig2 = SeedBuilder::compute_seed_signature(&seed_db_path).unwrap();

		assert_eq!(sig1, sig2);
	}

	#[test]
	fn test_signature_changes_on_file_modification() {
		let temp_dir = TempDir::new().unwrap();
		let root = temp_dir.path();

		let seed_db_path = root.join("seed.db");
		fs::write(&seed_db_path, "original content").unwrap();

		let sig1 = SeedBuilder::compute_seed_signature(&seed_db_path).unwrap();

		// Modify file
		fs::write(&seed_db_path, "modified content").unwrap();
		let sig2 = SeedBuilder::compute_seed_signature(&seed_db_path).unwrap();

		assert_ne!(sig1, sig2);
	}

	#[test]
	fn test_folder_not_found() {
		let builder = SeedBuilder::new(PathBuf::from("/nonexistent"), None);
		let result = builder.discover_files();

		assert!(result.is_err());
		match result {
			Err(SeedError::FolderNotFound(_)) => (),
			_ => panic!("Expected FolderNotFound error"),
		}
	}

	#[test]
	fn test_no_data_files() {
		let temp_dir = TempDir::new().unwrap();
		let root = temp_dir.path();

		// Create non-data files
		fs::write(root.join("file.txt"), "text").unwrap();
		fs::write(root.join("file.md"), "markdown").unwrap();

		let builder = SeedBuilder::new(root.to_path_buf(), None);
		let result = builder.discover_files();

		assert!(result.is_err());
		match result {
			Err(SeedError::NoDataFiles(_)) => (),
			_ => panic!("Expected NoDataFiles error"),
		}
	}
}
