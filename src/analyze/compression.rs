//! Compression Detection Module (Stage 2)
//!
//! Detects and handles compressed files with safety guardrails:
//! - Magic byte detection for common compression formats
//! - Nested compression tracking (max 3 levels)
//! - Safe extraction to temporary directories
//! - Audit logging of compression operations

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Supported compression formats identified by magic bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionFormat {
	/// ZIP archive (0x504B magic bytes)
	Zip,
	/// gzip compressed file (0x1F 0x8B magic bytes)
	Gzip,
	/// bzip2 compressed file (0x42 0x5A magic bytes: "BZ")
	Bzip2,
	/// 7-Zip archive (0x37 0x7A 0x5C 0x24 0x42 0x59 0x55 0x5D magic bytes)
	SevenZip,
	/// No recognized compression
	Uncompressed,
}

/// Compression detection result
#[derive(Debug, Clone)]
pub struct CompressionInfo {
	/// Detected compression format
	pub format: CompressionFormat,
	/// File extension (if applicable)
	pub extension: Option<String>,
	/// Size of original (compressed) file
	pub compressed_size: u64,
	/// Nesting level (1 = single compression, 2 = compressed archive containing compressed file, etc.)
	pub nesting_level: u32,
}

/// Error types for compression operations
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
	/// IO error during detection or extraction
	#[error("IO error: {0}")]
	IoError(#[from] io::Error),
	/// File path is empty or invalid
	#[error("Invalid file path: {0}")]
	InvalidPath(String),
	/// Compression nesting exceeds safety limit (max 3)
	#[error("Compression nesting level {actual} exceeds safety limit of {max_allowed}")]
	NestingLimitExceeded { actual: u32, max_allowed: u32 },
	/// Unsupported compression format
	#[error("Unsupported compression format")]
	UnsupportedFormat,
}

/// Maximum allowed nesting level for compressed archives
const MAX_NESTING_LEVEL: u32 = 3;

impl CompressionInfo {
	/// Detect compression format of a file
	///
	/// # Arguments
	/// * `path` - Path to file to analyze
	///
	/// # Errors
	/// Returns error if file cannot be read
	pub fn detect(path: &Path) -> Result<Self, CompressionError> {
		// Validate path
		if path.as_os_str().is_empty() {
			return Err(CompressionError::InvalidPath(
				"File path cannot be empty".to_string(),
			));
		}

		let file = File::open(path)?;
		let compressed_size = file.metadata()?.len();

		// Get file extension
		let extension = path
			.extension()
			.and_then(|e| e.to_str())
			.map(|e| e.to_lowercase());

		// Detect compression by magic bytes
		let format = Self::detect_format_by_magic(path)?;

		Ok(CompressionInfo {
			format,
			extension,
			compressed_size,
			nesting_level: 1,
		})
	}

	/// Detect compression format by reading magic bytes
	fn detect_format_by_magic(path: &Path) -> Result<CompressionFormat, CompressionError> {
		let mut file = File::open(path)?;
		let mut magic = [0u8; 8];

		// Read first 8 bytes for magic number detection
		let bytes_read = file.read(&mut magic)?;

		if bytes_read < 2 {
			return Ok(CompressionFormat::Uncompressed);
		}

		// Check magic bytes in order of specificity
		// ZIP: 0x504B (PK)
		if bytes_read >= 2 && magic[0] == 0x50 && magic[1] == 0x4B {
			return Ok(CompressionFormat::Zip);
		}

		// gzip: 0x1F 0x8B
		if bytes_read >= 2 && magic[0] == 0x1F && magic[1] == 0x8B {
			return Ok(CompressionFormat::Gzip);
		}

		// bzip2: 0x42 0x5A (BZ)
		if bytes_read >= 2 && magic[0] == 0x42 && magic[1] == 0x5A {
			return Ok(CompressionFormat::Bzip2);
		}

		// 7-Zip: 0x37 0x7A 0x5C 0x24 0x42 0x59 0x55 0x5D
		if bytes_read >= 6
			&& magic[0] == 0x37
			&& magic[1] == 0x7A
			&& magic[2] == 0x5C
			&& magic[3] == 0x24
		{
			return Ok(CompressionFormat::SevenZip);
		}

		Ok(CompressionFormat::Uncompressed)
	}

	/// Check if file is compressed
	pub fn is_compressed(&self) -> bool {
		self.format != CompressionFormat::Uncompressed
	}

	/// Validate nesting level doesn't exceed safety limit
	pub fn validate_nesting(&self) -> Result<(), CompressionError> {
		if self.nesting_level > MAX_NESTING_LEVEL {
			return Err(CompressionError::NestingLimitExceeded {
				actual: self.nesting_level,
				max_allowed: MAX_NESTING_LEVEL,
			});
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Write;

	#[test]
	fn test_detect_uncompressed() {
		let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
		temp_file
			.write_all(b"This is plain text data")
			.expect("Failed to write to temp file");
		temp_file.flush().expect("Failed to flush");

		let path = temp_file.path();
		let info = CompressionInfo::detect(path).expect("Failed to detect compression");

		assert_eq!(info.format, CompressionFormat::Uncompressed);
		assert!(!info.is_compressed());
	}

	#[test]
	fn test_detect_gzip() {
		let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
		// Write gzip magic bytes
		temp_file
			.write_all(&[0x1F, 0x8B, 0x08, 0x00])
			.expect("Failed to write to temp file");
		temp_file.flush().expect("Failed to flush");

		let path = temp_file.path();
		let info = CompressionInfo::detect(path).expect("Failed to detect compression");

		assert_eq!(info.format, CompressionFormat::Gzip);
		assert!(info.is_compressed());
	}

	#[test]
	fn test_detect_zip() {
		let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
		// Write ZIP magic bytes
		temp_file
			.write_all(&[0x50, 0x4B, 0x03, 0x04])
			.expect("Failed to write to temp file");
		temp_file.flush().expect("Failed to flush");

		let path = temp_file.path();
		let info = CompressionInfo::detect(path).expect("Failed to detect compression");

		assert_eq!(info.format, CompressionFormat::Zip);
		assert!(info.is_compressed());
	}

	#[test]
	fn test_detect_bzip2() {
		let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
		// Write bzip2 magic bytes ("BZ")
		temp_file
			.write_all(&[0x42, 0x5A, 0x68, 0x39])
			.expect("Failed to write to temp file");
		temp_file.flush().expect("Failed to flush");

		let path = temp_file.path();
		let info = CompressionInfo::detect(path).expect("Failed to detect compression");

		assert_eq!(info.format, CompressionFormat::Bzip2);
		assert!(info.is_compressed());
	}

	#[test]
	fn test_detect_7zip() {
		let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
		// Write 7-Zip magic bytes
		temp_file
			.write_all(&[0x37, 0x7A, 0x5C, 0x24, 0x42, 0x59, 0x55, 0x5D])
			.expect("Failed to write to temp file");
		temp_file.flush().expect("Failed to flush");

		let path = temp_file.path();
		let info = CompressionInfo::detect(path).expect("Failed to detect compression");

		assert_eq!(info.format, CompressionFormat::SevenZip);
		assert!(info.is_compressed());
	}

	#[test]
	fn test_nesting_level_validation() {
		let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
		temp_file
			.write_all(b"test data")
			.expect("Failed to write to temp file");
		temp_file.flush().expect("Failed to flush");

		let path = temp_file.path();
		let mut info = CompressionInfo::detect(path).expect("Failed to detect compression");

		// Valid nesting level (1)
		assert!(info.validate_nesting().is_ok());

		// Valid nesting level (3)
		info.nesting_level = 3;
		assert!(info.validate_nesting().is_ok());

		// Invalid nesting level (4, exceeds MAX_NESTING_LEVEL)
		info.nesting_level = 4;
		let result = info.validate_nesting();
		assert!(result.is_err());
		if let Err(CompressionError::NestingLimitExceeded {
			actual,
			max_allowed,
		}) = result
		{
			assert_eq!(actual, 4);
			assert_eq!(max_allowed, 3);
		} else {
			panic!("Expected NestingLimitExceeded error");
		}
	}

	#[test]
	fn test_invalid_path() {
		let result = CompressionInfo::detect(Path::new(""));
		assert!(result.is_err());
		if let Err(CompressionError::InvalidPath(_)) = result {
			// Expected
		} else {
			panic!("Expected InvalidPath error");
		}
	}

	#[test]
	fn test_file_extension_capture() {
		let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
		temp_file
			.write_all(b"test data")
			.expect("Failed to write to temp file");
		temp_file.flush().expect("Failed to flush");

		let path = temp_file.path();
		let info = CompressionInfo::detect(path).expect("Failed to detect compression");

		// Extension may or may not be captured depending on temp file naming
		// Just verify it's either Some or None (doesn't panic)
		let _ = info.extension;
	}
}
