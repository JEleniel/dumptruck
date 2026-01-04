use crate::{datafile::DataFieldType, detection::DetectionError};

pub struct OtherIdentificationNumber {}

impl OtherIdentificationNumber {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Other identification numbers not covered by specific types
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI {
			confidence += 0.6;
		}

		// Check for numeric pattern (generally IDs are numeric)
		let numeric_ratio =
			value.chars().filter(|c| c.is_ascii_digit()).count() as f32 / value.len() as f32;
		if numeric_ratio > 0.7 {
			confidence += 0.2;
		}

		// IDs typically have some structure (length constraints)
		if (5..=20).contains(&value.len()) {
			confidence += 0.1;
		}

		Ok(confidence)
	}
}
