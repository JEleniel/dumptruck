use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur during seed operations
#[derive(Error, Debug)]
pub enum SeedError {
	#[error("Folder not found: {0}")]
	FolderNotFound(String),

	#[error("No data files found in folder: {0}")]
	NoDataFiles(String),

	#[error("Database error: {0}")]
	DatabaseError(String),

	#[error("Signature computation failed: {0}")]
	SignatureError(String),

	#[error("Import failed: {0}")]
	ImportFailed(String),

	#[error("IO error: {0}")]
	IoError(#[from] std::io::Error),

	#[error("Serialization error: {0}")]
	SerializationError(#[from] serde_json::error::Error),

	#[error("Database not open: {0}")]
	DatabaseNotOpen(PathBuf),
}
