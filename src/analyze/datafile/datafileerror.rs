use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataFileError {
	#[error("Invalid UTF-8 encoding")]
	InvalidUtf8(#[from] std::str::Utf8Error),
	#[error("Rainbow Table files do not contain headers to analyze")]
	IsRainbowTableFile,
}
