use std::env::VarError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataFileError {
	#[error("Invalid UTF-8 encoding")]
	InvalidUtf8(#[from] std::str::Utf8Error),
	#[error("Rainbow Table files do not contain headers to analyze")]
	IsRainbowTableFile,
	#[error("Conversion error: {0}")]
	ConversionError(#[from] VarError),
	#[error("I/O error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Error calculating hash: {0}")]
	HashError(#[from] crate::HashError),
	#[error("Unsupported file type: {0}")]
	UnsupportedFileType(String),
	#[error("CSV error: {0}")]
	CsvError(#[from] csv::Error),
}
