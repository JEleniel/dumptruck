use crate::{datafile::DataFieldType, detection::DetectionError};

pub struct FullName {}

impl FullName {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Full names, identified primarily by headers/labels
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI {
			confidence += 0.5;
		}

		// Check for common name patterns (at least 2 words or known title patterns)
		let parts: Vec<&str> = value.split_whitespace().collect();
		if parts.len() >= 2 {
			confidence += 0.1;
		}

		// Check for name-like characteristics (all alphabetic with possible apostrophes/hyphens)
		let is_name_chars = value
			.chars()
			.all(|c| c.is_alphabetic() || c.is_whitespace() || c == '\'' || c == '-');
		if is_name_chars && value.len() >= 3 && parts.len() > 0 {
			confidence += 0.2;
		}

		Ok(confidence)
	}
}
