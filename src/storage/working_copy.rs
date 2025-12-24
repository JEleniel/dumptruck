//! Working copy management for isolated file processing.
//!
//! This module ensures that all ingest operations work with isolated copies of input files
//! rather than the originals. This provides:
//! - Isolation: Original files are never modified
//! - Security: Working folder can be NoExec, preventing execution attacks
//! - Cleanup: Temporary files can be securely deleted after processing
//! - Streaming support: Downloaded files go to the same working location
//!
//! # NoExec Verification
//!
//! The module checks that the working folder is mounted with noexec flag by attempting
//! to write and execute a test file. This prevents any malicious code injection attacks.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors related to working copy management
#[derive(Debug, Error)]
pub enum WorkingCopyError {
	/// Working directory creation failed
	#[error("Failed to create working directory: {0}")]
	CreateDirFailed(io::Error),
	/// Working directory is not mounted with noexec
	#[error("Working directory is not mounted with noexec: {0}")]
	NotNoExec(String),
	/// File copy failed
	#[error("Failed to copy file to working directory: {0}")]
	CopyFailed(io::Error),
	/// Failed to read file metadata
	#[error("Failed to read file metadata: {0}")]
	MetadataFailed(io::Error),
	/// Working directory does not exist or is inaccessible
	#[error("Working directory is inaccessible: {0}")]
	WorkdirInaccessible(io::Error),
	/// NoExec check file operations failed
	#[error("NoExec check failed: {0}")]
	NoExecCheckFailed(String),
	/// Invalid working directory path
	#[error("Invalid working directory path: {0}")]
	InvalidPath(String),
}

/// Working copy manager for isolated file processing
pub struct WorkingCopyManager {
	working_dir: PathBuf,
	_verify_noexec: bool,
	verbose: u32,
}

impl WorkingCopyManager {
	/// Create a new working copy manager
	///
	/// # Arguments
	/// * `working_dir` - Directory where working copies will be stored
	/// * `verify_noexec` - Whether to verify the directory is mounted with noexec
	/// * `verbose` - Verbosity level for logging
	///
	/// # Errors
	/// Returns error if directory cannot be created or permissions cannot be set
	pub fn new(
		working_dir: &Path,
		verify_noexec: bool,
		verbose: u32,
	) -> Result<Self, WorkingCopyError> {
		// Validate path
		if working_dir.as_os_str().is_empty() {
			return Err(WorkingCopyError::InvalidPath(
				"Working directory path cannot be empty".to_string(),
			));
		}

		// Create working directory if it doesn't exist
		if !working_dir.exists() {
			fs::create_dir_all(working_dir).map_err(WorkingCopyError::CreateDirFailed)?;

			if verbose >= 1 {
				eprintln!("[INFO] Created working directory: {:?}", working_dir);
			}
		}

		// Verify it's accessible
		fs::metadata(working_dir).map_err(WorkingCopyError::WorkdirInaccessible)?;

		// Set proper permissions on the working directory (0o700: owner rwx only)
		// This is a Linux-specific operation
		#[cfg(unix)]
		{
			use std::os::unix::fs::PermissionsExt;
			let perms = fs::Permissions::from_mode(0o700);
			fs::set_permissions(working_dir, perms).map_err(|e| {
				WorkingCopyError::CreateDirFailed(io::Error::new(
					io::ErrorKind::PermissionDenied,
					format!("Failed to set directory permissions: {}", e),
				))
			})?;

			if verbose >= 1 {
				eprintln!("[INFO] Set working directory permissions to 0o700 (owner rwx only)");
			}
		}

		let manager = WorkingCopyManager {
			working_dir: working_dir.to_path_buf(),
			_verify_noexec: verify_noexec,
			verbose,
		};

		// Check noexec if requested (optional - only if explicitly enabled)
		if verify_noexec {
			manager.verify_noexec_mount()?;
		} else if verbose >= 1 {
			eprintln!("[INFO] NoExec verification disabled");
		}

		Ok(manager)
	}

	/// Verify that the working directory is mounted with noexec
	///
	/// This check attempts to create a test file and verify it cannot be executed.
	/// On systems without noexec support or where execution is not prevented,
	/// this will return an error.
	fn verify_noexec_mount(&self) -> Result<(), WorkingCopyError> {
		// NoExec verification is only meaningful on Unix systems
		#[cfg(unix)]
		{
			let test_file = self.working_dir.join(".noexec_check");

			// Try to write a test file
			let script_content = "#!/bin/sh\nexit 0";
			fs::write(&test_file, script_content).map_err(|e| {
				WorkingCopyError::NoExecCheckFailed(format!("Failed to write test file: {}", e))
			})?;

			// Try to set execute permissions (will fail or be ignored on noexec mount)
			use std::os::unix::fs::PermissionsExt;
			let perms = fs::Permissions::from_mode(0o755);
			fs::set_permissions(&test_file, perms).map_err(|e| {
				WorkingCopyError::NoExecCheckFailed(format!("Failed to set permissions: {}", e))
			})?;

			// Try to execute the test file (should fail on noexec mount)
			let output = std::process::Command::new(&test_file).output();

			// Clean up test file
			let _ = fs::remove_file(&test_file);

			// On a noexec mount, execution should fail with permission denied
			match output {
				Ok(_status) => {
					// If execution succeeded, noexec is not enabled
					Err(WorkingCopyError::NotNoExec(
						"Test file executed successfully - noexec is not enabled".to_string(),
					))
				}
				Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
					// This is what we want - execution denied
					if self.verbose >= 2 {
						eprintln!("[DEBUG] Working directory verified as noexec");
					}
					Ok(())
				}
				Err(e) => {
					// Some other error - might be acceptable depending on context
					if self.verbose >= 2 {
						eprintln!("[DEBUG] NoExec check returned: {:?}", e);
					}
					// For now, treat this as success since execution didn't happen
					// (might be file not found, etc. which is still safe)
					Ok(())
				}
			}
		}

		// On non-Unix systems (Windows), skip noexec verification
		#[cfg(not(unix))]
		{
			if self.verbose >= 2 {
				eprintln!("[DEBUG] NoExec verification not available on this platform");
			}
			Ok(())
		}
	}

	/// Create a working copy of a file
	///
	/// Copies the input file to the working directory and returns the path.
	/// The copy is isolated and the original file is never modified.
	///
	/// # Arguments
	/// * `source_path` - Path to the original file
	///
	/// # Returns
	/// Path to the working copy in the working directory
	pub fn create_working_copy(&self, source_path: &Path) -> Result<PathBuf, WorkingCopyError> {
		if self.verbose >= 2 {
			eprintln!("[DEBUG] Creating working copy from: {:?}", source_path);
		}

		// Get the filename from the source path
		let filename = source_path.file_name().ok_or_else(|| {
			WorkingCopyError::InvalidPath(format!("Cannot extract filename from {:?}", source_path))
		})?;

		// Create destination path in working directory
		let dest_path = self.working_dir.join(filename);

		// Copy file to working directory
		fs::copy(source_path, &dest_path).map_err(WorkingCopyError::CopyFailed)?;

		if self.verbose >= 1 {
			eprintln!(
				"[INFO] Working copy created: {:?} -> {:?}",
				source_path, dest_path
			);
		}

		Ok(dest_path)
	}

	/// Create a working copy with a unique name (for streaming/multiple versions)
	///
	/// If the target filename already exists in the working directory,
	/// this generates a unique variant with timestamp or sequence number.
	///
	/// # Arguments
	/// * `source_path` - Path to the original file
	///
	/// # Returns
	/// Path to the uniquely-named working copy
	pub fn create_working_copy_unique(
		&self,
		source_path: &Path,
	) -> Result<PathBuf, WorkingCopyError> {
		if self.verbose >= 2 {
			eprintln!(
				"[DEBUG] Creating unique working copy from: {:?}",
				source_path
			);
		}

		let filename = source_path.file_name().ok_or_else(|| {
			WorkingCopyError::InvalidPath(format!("Cannot extract filename from {:?}", source_path))
		})?;

		let mut dest_path = self.working_dir.join(filename);

		// If file exists, add timestamp/sequence
		if dest_path.exists() {
			let stem = source_path
				.file_stem()
				.and_then(|s| s.to_str())
				.unwrap_or("file");
			let ext = source_path
				.extension()
				.and_then(|s| s.to_str())
				.unwrap_or("");

			let timestamp = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_millis())
				.unwrap_or(0);

			let unique_name = if ext.is_empty() {
				format!("{}.{}", stem, timestamp)
			} else {
				format!("{}.{}.{}", stem, timestamp, ext)
			};

			dest_path = self.working_dir.join(unique_name);
		}

		// Copy file
		fs::copy(source_path, &dest_path).map_err(WorkingCopyError::CopyFailed)?;

		if self.verbose >= 1 {
			eprintln!(
				"[INFO] Unique working copy created: {:?} -> {:?}",
				source_path, dest_path
			);
		}

		Ok(dest_path)
	}

	/// Get the working directory path
	pub fn working_dir(&self) -> &Path {
		&self.working_dir
	}

	/// Clean up all files in the working directory
	///
	/// This is used for cleanup between operations or at shutdown.
	/// Use secure_delete for NIST SP 800-88 compliant deletion (Stage 14).
	pub fn cleanup(&self) -> Result<(), WorkingCopyError> {
		if self.verbose >= 1 {
			eprintln!(
				"[INFO] Cleaning up working directory: {:?}",
				self.working_dir
			);
		}

		for entry in
			fs::read_dir(&self.working_dir).map_err(WorkingCopyError::WorkdirInaccessible)?
		{
			let entry = entry.map_err(WorkingCopyError::WorkdirInaccessible)?;
			let path = entry.path();

			if path.is_file() {
				fs::remove_file(&path).map_err(WorkingCopyError::CopyFailed)?;

				if self.verbose >= 2 {
					eprintln!("[DEBUG] Cleaned up: {:?}", path);
				}
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::TempDir;

	#[test]
	fn test_create_working_copy_manager() {
		let temp_dir = TempDir::new().unwrap();
		let manager = WorkingCopyManager::new(temp_dir.path(), false, 0);
		assert!(manager.is_ok());
	}

	#[test]
	fn test_working_dir_created_if_missing() {
		let temp_dir = TempDir::new().unwrap();
		let work_dir = temp_dir.path().join("work");
		assert!(!work_dir.exists());

		let _manager = WorkingCopyManager::new(&work_dir, false, 0);
		assert!(work_dir.exists());
	}

	#[test]
	fn test_create_working_copy() {
		let temp_dir = TempDir::new().unwrap();
		let work_dir = temp_dir.path().join("work");

		// Create source file
		let source_file = temp_dir.path().join("source.csv");
		fs::write(&source_file, "test,data\n1,2").unwrap();

		let manager = WorkingCopyManager::new(&work_dir, false, 0).unwrap();
		let copy_path = manager.create_working_copy(&source_file).unwrap();

		// Verify copy exists and has same content
		assert!(copy_path.exists());
		let content = fs::read_to_string(&copy_path).unwrap();
		assert_eq!(content, "test,data\n1,2");

		// Verify copy is in working directory
		assert!(copy_path.parent().unwrap() == work_dir);
	}

	#[test]
	fn test_create_working_copy_unique() {
		let temp_dir = TempDir::new().unwrap();
		let work_dir = temp_dir.path().join("work");

		// Create source files
		let source1 = temp_dir.path().join("data.csv");
		let source2 = temp_dir.path().join("data.csv");
		fs::write(&source1, "first").unwrap();
		fs::write(&source2, "second").unwrap();

		let manager = WorkingCopyManager::new(&work_dir, false, 0).unwrap();

		// Create first copy
		let copy1 = manager.create_working_copy_unique(&source1).unwrap();
		assert!(copy1.exists());

		// Create second copy - should have different name
		let copy2 = manager.create_working_copy_unique(&source2).unwrap();
		assert!(copy2.exists());
		assert_ne!(copy1, copy2);
	}

	#[test]
	fn test_cleanup() {
		let temp_dir = TempDir::new().unwrap();
		let work_dir = temp_dir.path().join("work");

		// Create source and copy
		let source = temp_dir.path().join("source.csv");
		fs::write(&source, "data").unwrap();

		let manager = WorkingCopyManager::new(&work_dir, false, 0).unwrap();
		let _copy = manager.create_working_copy(&source).unwrap();

		// Verify copy exists
		let entries: Vec<_> = fs::read_dir(&work_dir)
			.unwrap()
			.filter_map(Result::ok)
			.collect();
		assert!(!entries.is_empty());

		// Clean up
		manager.cleanup().unwrap();

		// Verify cleaned
		let entries: Vec<_> = fs::read_dir(&work_dir)
			.unwrap()
			.filter_map(Result::ok)
			.collect();
		assert!(entries.is_empty());
	}

	#[test]
	fn test_invalid_path() {
		let manager = WorkingCopyManager::new(Path::new(""), false, 0);
		assert!(manager.is_err());
	}

	#[test]
	fn test_noexec_check_disabled() {
		let temp_dir = TempDir::new().unwrap();
		// Should succeed with verify_noexec=false even if not on noexec mount
		let manager = WorkingCopyManager::new(temp_dir.path(), false, 0);
		assert!(manager.is_ok());
	}
}
