//! Builds and manages SQLite-based rainbow tables.
//!
//! This module automatically detects changes to wordlist files and updates the rainbow
//! table database accordingly. At startup, the app checks file signatures to determine
//! if regeneration is needed. All rainbow table data is now stored in SQLite instead of JSON.

use crate::core::hash_utils::{md5_hex, ntlm_hex, sha1_hex, sha256_hex, sha512_hex};
use md5::Context;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::path::PathBuf;
use std::{fs, io};
use thiserror::Error;
use tracing::debug;

/// Errors that can occur during rainbow table building.
#[derive(Debug, Error)]
pub enum RainbowTableError {
	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),
	#[error("Database error: {0}")]
	Database(String),
	#[error("JSON serialization error: {0}")]
	Json(#[from] serde_json::Error),
	#[error("UTF-8 error: {0}")]
	Utf8(#[from] std::string::FromUtf8Error),
	#[error("No passwords found to hash")]
	NoPasswords,
}

/// A single password entry with all hash variants.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RainbowTableEntry {
	pub plaintext: String,
	pub md5: String,
	pub sha1: String,
	pub sha256: String,
	pub sha512: String,
	pub ntlm: String,
}

pub struct RainbowTableFile {
	pub path: PathBuf,
	pub signature: String,
	pub processed: bool,
}

/// Manages SQLite-based rainbow tables with automatic change detection.
pub struct RainbowTableBuilder {
	/// Directory containing wordlist files
	pub data_dir: String,
	pub rainbow_table_files: Vec<RainbowTableFile>,
}

impl RainbowTableBuilder {
	/// Create a new builder with default paths.
	pub fn new() -> Self {
		Self {
			data_dir: "data".to_string(),
			rainbow_table_files: Vec::new(),
		}
	}

	/// Set custom data directory for wordlist files.
	pub fn with_data_dir(mut self, path: String) -> Self {
		self.data_dir = path;
		self
	}

	/// Compute MD5 signature of a file for change detection.
	pub fn calculate_md5(file_path: &Path) -> io::Result<String> {
		let file = fs::File::open(file_path)?;
		let mut context = Context::new();
		let mut buffer = [0; 4096]; // buffer size: 4KB
		let mut reader = io::BufReader::new(file);

		loop {
			let bytes_read = reader.read(&mut buffer)?;
			if bytes_read == 0 {
				break;
			}

			context.consume(&buffer[..bytes_read]);
		}

		let result = context.finalize();

		Ok(format!("{:x}", result))
	}

	/// Check if any wordlist files have changed by comparing signatures in DB.
	fn files_changed(&mut self, conn: &Connection) -> Result<bool, RainbowTableError> {
		let data_dir = Path::new(&self.data_dir);

		if !data_dir.exists() {
			return Ok(false);
		}

		// Get all .txt files in data directory
		let entries = fs::read_dir(&self.data_dir)?;

		for entry in entries {
			let entry = entry?;
			let path = entry.path();

			if path.extension().map(|e| e == "txt").unwrap_or(false) {
				let signature = Self::calculate_md5(path.as_path())?;
				self.rainbow_table_files.push(RainbowTableFile {
					path,
					signature,
					processed: false,
				});
			}
		}

		// Check each file against DB signatures
		let mut stmt = conn
			.prepare(
				"SELECT file_md5_signature FROM rainbow_table_file_signatures WHERE filename = ?1",
			)
			.map_err(|e| RainbowTableError::Database(e.to_string()))?;
		for file in &mut self.rainbow_table_files {
			let filename = file.path.file_name().unwrap().to_str().unwrap();
			let new_sig = &file.signature;
			let result: Option<String> = stmt.query_row([filename], |row| row.get(0)).ok();

			if result.as_deref() != Some(new_sig.as_str()) {
				file.processed = true; // ✓ Correct—file is new or changed
			}
		}

		// Check for files in DB that no longer exist
		let mut stmt = conn
			.prepare("SELECT filename FROM rainbow_table_file_signatures")
			.map_err(|e| RainbowTableError::Database(e.to_string()))?;

		let file_names: Vec<String> = stmt
			.query_map([], |row| row.get(0))
			.map_err(|e| RainbowTableError::Database(e.to_string()))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| RainbowTableError::Database(e.to_string()))?;

		// Remove DB entries for files that no longer exist and track if any files changed
		let mut delete_stmt = conn
			.prepare("DELETE FROM rainbow_table_file_signatures WHERE filename = ?1")
			.map_err(|e| RainbowTableError::Database(e.to_string()))?;

		let mut files_deleted = false;
		for filename in file_names {
			if !self
				.rainbow_table_files
				.iter()
				.any(|f| f.path.file_name().unwrap().to_str().unwrap() == filename)
			{
				delete_stmt
					.execute([&filename])
					.map_err(|e| RainbowTableError::Database(e.to_string()))?;
				debug!("Removed stale file signature from DB: {}", filename);
				files_deleted = true;
			}
		}

		// Check if any files were marked as needing processing or files were deleted
		let any_files_changed =
			self.rainbow_table_files.iter().any(|f| f.processed) || files_deleted;
		Ok(any_files_changed)
	}

	/// Generate and store rainbow table entries in database.
	pub fn populate_database(&mut self, conn: &Connection) -> Result<bool, RainbowTableError> {
		// Check if files have changed
		if !self.files_changed(conn)? {
			return Ok(false);
		}

		// Prepare insert statement once
		let mut stmt = conn
			.prepare(
				"INSERT INTO rainbow_tables (plaintext, md5, sha1, sha256, sha512, ntlm) \
				VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
			)
			.map_err(|e| RainbowTableError::Database(e.to_string()))?;

		// Process files line-by-line, inserting hashes immediately
		let data_dir = Path::new(&self.data_dir);
		if data_dir.exists() {
			let entries = fs::read_dir(&self.data_dir)?;

			for entry in entries {
				let entry = entry?;
				let path = entry.path();

				if path.extension().map(|e| e == "txt").unwrap_or(false) {
					// Process file line-by-line with lossy UTF-8 conversion
					let file = match fs::File::open(&path) {
						Ok(f) => f,
						Err(e) => {
							eprintln!("[WARN] Failed to open file {}: {}", path.display(), e);
							continue;
						}
					};
					let reader = BufReader::new(file);

					for line_result in reader.lines() {
						// Skip lines with UTF-8 errors, converting them lossily
						let line = match line_result {
							Ok(l) => l,
							Err(_) => continue, // Skip lines that fail to read
						};

						let trimmed = line.trim();

						// Skip empty lines and comments
						if trimmed.is_empty() || trimmed.starts_with('#') {
							continue;
						}

						// Insert hash immediately without storing in memory
						stmt.execute(rusqlite::params![
							trimmed,
							md5_hex(trimmed),
							sha1_hex(trimmed),
							sha256_hex(trimmed),
							sha512_hex(trimmed),
							ntlm_hex(trimmed)
						])
						.map_err(|e| RainbowTableError::Database(e.to_string()))?;
					}
				}
			}
		}

		drop(stmt);

		// Update file signatures
		let mut sig_stmt = conn
			.prepare(
				"INSERT OR REPLACE INTO rainbow_table_file_signatures (filename, file_md5_signature) \
				VALUES (?1, ?2)",
			)
			.map_err(|e| RainbowTableError::Database(e.to_string()))?;

		if data_dir.exists() {
			let entries = fs::read_dir(&self.data_dir)?;
			for entry in entries {
				let entry = entry?;
				let path = entry.path();

				if path.extension().map(|e| e == "txt").unwrap_or(false)
					&& let Some(filename) = path.file_name()
					&& let Some(filename_str) = filename.to_str()
				{
					let sig = Self::calculate_md5(path.as_path())?;
					sig_stmt
						.execute(rusqlite::params![filename_str, &sig])
						.map_err(|e| RainbowTableError::Database(e.to_string()))?;
				}
			}
		}

		Ok(true)
	}
}

impl Default for RainbowTableBuilder {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_rainbow_table_entry_serialization() {
		let entry = RainbowTableEntry {
			plaintext: "password".to_string(),
			md5: "5f4dcc3b5aa765d61d8327deb882cf99".to_string(),
			sha1: "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8".to_string(),
			sha256: "5e884898da28047151d0e56f8dc62927051d5e07d5d91ff5e95b1fee3e06a717".to_string(),
			sha512: "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3".to_string(),
			ntlm: "8846F7EAEE8FB117AD06BDD830B7586C".to_string(),
		};

		let json = serde_json::to_string(&entry).expect("serialization failed");
		let deserialized: RainbowTableEntry =
			serde_json::from_str(&json).expect("deserialization failed");

		assert_eq!(entry, deserialized);
	}

	#[test]
	fn test_builder_default() {
		let builder = RainbowTableBuilder::new();
		assert_eq!(builder.data_dir, "data");
	}

	#[test]
	fn test_builder_custom_dir() {
		let builder = RainbowTableBuilder::new().with_data_dir("custom_dir".to_string());
		assert_eq!(builder.data_dir, "custom_dir");
	}
}
