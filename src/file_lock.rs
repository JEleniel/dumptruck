//! File locking utilities for single-writer synchronization across multiple instances.
//!
//! This module provides mechanisms to ensure only one application instance
//! writes a received file at a time, preventing concurrent file corruption.

use std::fs::OpenOptions;
use std::io;
use std::path::Path;

/// A file-based lock that ensures exclusive write access
/// Uses a marker file approach compatible across Unix and Windows
pub struct FileLock {
	lock_path: std::path::PathBuf,
}

impl FileLock {
	/// Acquire an exclusive lock for a file
	/// Creates a lock file and removes it on drop
	pub fn acquire(file_path: &Path) -> io::Result<Self> {
		let lock_path = file_path.with_extension(format!(
			"{}.lock",
			file_path
				.extension()
				.map(|e| e.to_string_lossy().to_string())
				.unwrap_or_default()
		));

		// Try to create the lock file exclusively
		// This operation is atomic on most filesystems
		OpenOptions::new()
			.write(true)
			.create_new(true)
			.open(&lock_path)?;

		Ok(FileLock { lock_path })
	}

	/// Check if a lock exists for the given file
	pub fn is_locked(file_path: &Path) -> bool {
		let lock_path = file_path.with_extension(format!(
			"{}.lock",
			file_path
				.extension()
				.map(|e| e.to_string_lossy().to_string())
				.unwrap_or_default()
		));
		lock_path.exists()
	}
}

impl Drop for FileLock {
	fn drop(&mut self) {
		// Clean up the lock file
		let _ = std::fs::remove_file(&self.lock_path);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_file_lock_creation() {
		// Basic test that FileLock compiles and type-checks correctly
		let _lock_type = std::any::type_name::<FileLock>();
		assert_eq!(_lock_type, "dumptruck::file_lock::FileLock");
	}
}
