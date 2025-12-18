//! Builds and manages external JSON-based rainbow tables.
//!
//! This module automatically detects changes to wordlist files and updates the rainbow
//! table JSON file accordingly. At startup, the app checks file signatures to determine
//! if regeneration is needed.

use crate::hash_utils::{md5_hex, ntlm_hex, sha1_hex, sha256_hex, sha512_hex};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during rainbow table building.
#[derive(Debug, Error)]
pub enum RainbowTableError {
	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),
	#[error("JSON serialization error: {0}")]
	Json(#[from] serde_json::Error),
	#[error("UTF-8 error: {0}")]
	Utf8(#[from] std::string::FromUtf8Error),
	#[error("No passwords found to hash")]
	NoPasswords,
}

/// JSON structure for a single password entry with all hash variants.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RainbowTableEntry {
	pub plaintext: String,
	pub md5: String,
	pub sha1: String,
	pub sha256: String,
	pub sha512: String,
	pub ntlm: String,
}

/// JSON structure for the complete rainbow table with metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RainbowTableJson {
	/// Version of the JSON format
	pub version: u32,
	/// File signatures (filename -> MD5 hash of content) for change detection
	pub file_signatures: BTreeMap<String, String>,
	/// List of password entries
	pub entries: Vec<RainbowTableEntry>,
}

/// Manages external JSON-based rainbow tables.
pub struct RainbowTableBuilder {
	/// Path to store/load the rainbow table JSON
	pub output_path: String,
	/// Directory containing wordlist files
	pub data_dir: String,
	/// Whether to include NTLM hashes
	pub include_ntlm: bool,
	/// Whether to include SHA512 hashes
	pub include_sha512: bool,
}

impl RainbowTableBuilder {
	/// Create a new builder with default paths.
	pub fn new() -> Self {
		Self {
			output_path: ".cache/rainbow_table.json".to_string(),
			data_dir: "data".to_string(),
			include_ntlm: true,
			include_sha512: true,
		}
	}

	/// Set custom output path for the JSON file.
	pub fn with_output_path(mut self, path: String) -> Self {
		self.output_path = path;
		self
	}

	/// Set custom data directory for wordlist files.
	pub fn with_data_dir(mut self, path: String) -> Self {
		self.data_dir = path;
		self
	}

	/// Disable NTLM hash generation.
	pub fn without_ntlm(mut self) -> Self {
		self.include_ntlm = false;
		self
	}

	/// Disable SHA512 hash generation.
	pub fn without_sha512(mut self) -> Self {
		self.include_sha512 = false;
		self
	}

	/// Load all wordlists from data directory.
	fn load_wordlists(&self) -> Result<Vec<String>, RainbowTableError> {
		let mut passwords = Vec::new();

		if !Path::new(&self.data_dir).exists() {
			// Data directory doesn't exist yet - this is OK on first startup
			return Ok(passwords);
		}

		// Read all .txt files from data directory
		let entries = fs::read_dir(&self.data_dir)?;

		for entry in entries {
			let entry = entry?;
			let path = entry.path();

			if path.extension().map(|e| e == "txt").unwrap_or(false) {
				let content = fs::read_to_string(&path)?;
				for line in content.lines() {
					let trimmed = line.trim();
					// Skip empty lines and comments (lines starting with #)
					if !trimmed.is_empty() && !trimmed.starts_with('#') {
						passwords.push(trimmed.to_string());
					}
				}
			}
		}

		Ok(passwords)
	}

	/// Compute MD5 hash of a file's content for change detection.
	fn compute_file_signature(path: &str) -> Result<String, RainbowTableError> {
		let content = fs::read_to_string(path)?;
		Ok(md5_hex(&content))
	}

	/// Load current file signatures from existing rainbow table.
	fn load_current_signatures(&self) -> Result<BTreeMap<String, String>, RainbowTableError> {
		if !Path::new(&self.output_path).exists() {
			return Ok(BTreeMap::new());
		}

		let content = fs::read_to_string(&self.output_path)?;
		let table: RainbowTableJson = serde_json::from_str(&content)?;

		Ok(table.file_signatures)
	}

	/// Check if any wordlist files have changed.
	fn files_changed(&self) -> Result<bool, RainbowTableError> {
		let current_signatures = self.load_current_signatures()?;
		let data_dir = Path::new(&self.data_dir);

		if !data_dir.exists() {
			// If data dir doesn't exist, nothing to compare
			return Ok(false);
		}

		// Get all .txt files in data directory
		let mut new_signatures = BTreeMap::new();
		let entries = fs::read_dir(&self.data_dir)?;

		for entry in entries {
			let entry = entry?;
			let path = entry.path();

			if path.extension().map(|e| e == "txt").unwrap_or(false) {
				if let Some(filename) = path.file_name() {
					if let Some(filename_str) = filename.to_str() {
						let sig = Self::compute_file_signature(path.to_str().unwrap())?;
						new_signatures.insert(filename_str.to_string(), sig);
					}
				}
			}
		}

		// Check if signatures differ
		Ok(current_signatures != new_signatures)
	}

	/// Generate rainbow table from all available wordlists.
	pub fn generate(&self) -> Result<RainbowTableJson, RainbowTableError> {
		let passwords = self.load_wordlists()?;

		if passwords.is_empty() {
			// If no passwords found, create empty table
			return Ok(RainbowTableJson {
				version: 1,
				file_signatures: BTreeMap::new(),
				entries: Vec::new(),
			});
		}

		let mut entries = Vec::new();

		// Generate hash entries for all passwords
		for pwd in &passwords {
			entries.push(RainbowTableEntry {
				plaintext: pwd.clone(),
				md5: md5_hex(pwd),
				sha1: sha1_hex(pwd),
				sha256: sha256_hex(pwd),
				sha512: if self.include_sha512 {
					sha512_hex(pwd)
				} else {
					String::new()
				},
				ntlm: if self.include_ntlm {
					ntlm_hex(pwd)
				} else {
					String::new()
				},
			});
		}

		// Compute file signatures for change detection
		let mut file_signatures = BTreeMap::new();
		let data_dir = Path::new(&self.data_dir);

		if data_dir.exists() {
			let entries = fs::read_dir(&self.data_dir)?;
			for entry in entries {
				let entry = entry?;
				let path = entry.path();

				if path.extension().map(|e| e == "txt").unwrap_or(false) {
					if let Some(filename) = path.file_name() {
						if let Some(filename_str) = filename.to_str() {
							let sig = Self::compute_file_signature(path.to_str().unwrap())?;
							file_signatures.insert(filename_str.to_string(), sig);
						}
					}
				}
			}
		}

		Ok(RainbowTableJson {
			version: 1,
			file_signatures,
			entries,
		})
	}

	/// Update the rainbow table if files have changed.
	/// Returns true if table was regenerated, false if no changes detected.
	pub fn update_if_changed(&self) -> Result<bool, RainbowTableError> {
		// Check if we need to regenerate
		if !self.files_changed()? {
			return Ok(false);
		}

		// Generate new table
		let table = self.generate()?;

		// Ensure output directory exists
		if let Some(parent) = Path::new(&self.output_path).parent() {
			if !parent.as_os_str().is_empty() {
				fs::create_dir_all(parent)?;
			}
		}

		// Write to file
		let json = serde_json::to_string_pretty(&table)?;
		fs::write(&self.output_path, json)?;

		Ok(true)
	}

	/// Load rainbow table from JSON file.
	pub fn load(&self) -> Result<RainbowTableJson, RainbowTableError> {
		if !Path::new(&self.output_path).exists() {
			// If file doesn't exist, generate it
			self.update_if_changed()?;
		}

		let content = fs::read_to_string(&self.output_path)?;
		let table = serde_json::from_str(&content)?;

		Ok(table)
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
	fn test_rainbow_table_json_structure() {
		let table = RainbowTableJson {
			version: 1,
			file_signatures: BTreeMap::new(),
			entries: vec![],
		};

		let json = serde_json::to_string_pretty(&table).expect("serialization failed");
		assert!(json.contains("\"version\": 1"));
		assert!(json.contains("\"file_signatures\""));
		assert!(json.contains("\"entries\""));
	}

	#[test]
	fn test_builder_default() {
		let builder = RainbowTableBuilder::new();
		assert_eq!(builder.output_path, ".cache/rainbow_table.json");
		assert_eq!(builder.data_dir, "data");
		assert!(builder.include_ntlm);
		assert!(builder.include_sha512);
	}

	#[test]
	fn test_builder_configuration() {
		let builder = RainbowTableBuilder::new()
			.with_output_path("/tmp/test.json".to_string())
			.with_data_dir("/tmp/data".to_string())
			.without_ntlm()
			.without_sha512();

		assert_eq!(builder.output_path, "/tmp/test.json");
		assert_eq!(builder.data_dir, "/tmp/data");
		assert!(!builder.include_ntlm);
		assert!(!builder.include_sha512);
	}
}
