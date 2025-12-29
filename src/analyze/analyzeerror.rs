use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnalyzeError {
	#[error("Invalid breach date format: {0}")]
	InvalidBreachDate(String),
}
