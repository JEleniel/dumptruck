//! Evidence Preservation Module (Stage 1)
//!
//! Establishes unique identity and immutable proof of file origin by:
//! - Generating unique file identifier (UUID v4 + timestamp)
//! - Computing file hash for integrity verification
//! - Capturing alternate file names for the same data
//! - Storing metadata for chain of custody and deduplication
//!
//! Note: Currently uses SHA-256 for hashing. BLAKE3 support to be added.

use sha2::Digest;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// File evidence metadata for chain of custody and deduplication
#[derive(Debug, Clone)]
pub struct FileEvidence {
	/// Unique file identifier (UUID v4 + timestamp)
	pub file_id: String,
	/// SHA-256 hash of file contents
	pub sha256_hash: String,
	/// All known variants of the filename
	pub alternate_names: Vec<String>,
	/// Creation timestamp (Unix seconds)
	pub created_at: u64,
	/// File size in bytes
	pub file_size: u64,
}

/// Error types for evidence preservation operations
#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
	/// IO error while reading file
	#[error("IO error: {0}")]
	IoError(#[from] io::Error),
	/// Invalid file path
	#[error("Invalid file path: {0}")]
	InvalidPath(String),
	/// System time error
	#[error("System time error: {0}")]
	TimeError(String),
}

impl FileEvidence {
	/// Create evidence for a file
	///
	/// # Arguments
	/// * `path` - File path to create evidence for
	/// * `alternate_names` - Optional list of other names for the same file
	///
	/// # Errors
	/// Returns error if file cannot be read or hashes cannot be computed
	pub fn create(
		path: &Path,
		alternate_names: Option<Vec<String>>,
	) -> Result<Self, EvidenceError> {
		// Validate path
		if path.as_os_str().is_empty() {
			return Err(EvidenceError::InvalidPath(
				"File path cannot be empty".to_string(),
			));
		}

		// Generate unique file ID with timestamp
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map_err(|e| EvidenceError::TimeError(format!("System time error: {}", e)))?
			.as_secs();

		let file_id = format!("{}-{}", Uuid::new_v4(), now);

		// Read file contents
		let mut file = File::open(path)?;
		let file_size = file.metadata()?.len();

		// Compute SHA-256
		let mut sha256_hasher = sha2::Sha256::new();
		let mut buffer = [0; 8192];
		file.seek(SeekFrom::Start(0))?;

		loop {
			let bytes_read = file.read(&mut buffer)?;
			if bytes_read == 0 {
				break;
			}
			sha256_hasher.update(&buffer[..bytes_read]);
		}

		let sha256_hash = format!("{:x}", sha256_hasher.finalize());

		// Collect alternate names
		let mut names = alternate_names.unwrap_or_default();
		if let Some(filename) = path.file_name() {
			if let Some(filename_str) = filename.to_str() {
				if !names.contains(&filename_str.to_string()) {
					names.push(filename_str.to_string());
				}
			}
		}

		Ok(FileEvidence {
			file_id,
			sha256_hash,
			alternate_names: names,
			created_at: now,
			file_size,
		})
	}

	/// Verify file evidence hasn't been tampered with
	///
	/// # Arguments
	/// * `path` - File path to verify
	///
	/// # Returns
	/// true if file hash matches, false if tampering detected
	pub fn verify(&self, path: &Path) -> Result<bool, EvidenceError> {
		let mut file = File::open(path)?;

		// Verify SHA-256
		let mut sha256_hasher = sha2::Sha256::new();
		let mut buffer = [0; 8192];

		loop {
			let bytes_read = file.read(&mut buffer)?;
			if bytes_read == 0 {
				break;
			}
			sha256_hasher.update(&buffer[..bytes_read]);
		}

		let computed_sha256 = format!("{:x}", sha256_hasher.finalize());
		Ok(computed_sha256 == self.sha256_hash)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Write;

	#[test]
	fn test_create_evidence() {
		let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
		temp_file
			.write_all(b"test data for evidence")
			.expect("Failed to write to temp file");
		temp_file.flush().expect("Failed to flush temp file");

		let path = temp_file.path();
		let evidence = FileEvidence::create(path, Some(vec!["alternate.csv".to_string()]))
			.expect("Failed to create evidence");

		assert!(!evidence.file_id.is_empty());
		assert!(!evidence.sha256_hash.is_empty());
		assert_eq!(evidence.file_size, 22);
		assert!(
			evidence
				.alternate_names
				.contains(&"alternate.csv".to_string())
		);
		assert!(evidence.created_at > 0);
	}

	#[test]
	fn test_verify_evidence() {
		let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
		temp_file
			.write_all(b"test data for verification")
			.expect("Failed to write to temp file");
		temp_file.flush().expect("Failed to flush temp file");

		let path = temp_file.path();
		let evidence = FileEvidence::create(path, None).expect("Failed to create evidence");

		let is_valid = evidence.verify(path).expect("Failed to verify evidence");
		assert!(is_valid);
	}

	#[test]
	fn test_verify_tampered_evidence() {
		let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
		temp_file
			.write_all(b"original data")
			.expect("Failed to write to temp file");
		temp_file.flush().expect("Failed to flush temp file");

		let path = temp_file.path();
		let evidence = FileEvidence::create(path, None).expect("Failed to create evidence");

		// Tamper with the file
		let mut tampered_file = File::create(path).expect("Failed to open file for writing");
		tampered_file
			.write_all(b"tampered data")
			.expect("Failed to write to file");
		drop(tampered_file);

		let is_valid = evidence.verify(path).expect("Failed to verify evidence");
		assert!(!is_valid, "Should detect tampering");
	}

	#[test]
	fn test_invalid_path() {
		let result = FileEvidence::create(Path::new(""), None);
		assert!(result.is_err());
	}

	#[test]
	fn test_nonexistent_file() {
		let result = FileEvidence::create(Path::new("/nonexistent/file.txt"), None);
		assert!(result.is_err());
	}
}
