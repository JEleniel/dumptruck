use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnalyzeError {
	#[error("Invalid breach date format: {0}")]
	InvalidBreachDate(String),
	#[error("I/O error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Regex error: {0}")]
	RegexError(#[from] regex::Error),
}
