use crate::{datafile::DataFieldType, detection::DetectionError};

pub struct OtherRecordNumber {}

impl OtherRecordNumber {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Other records not covered by specific types
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI {
			confidence += 0.5;
		}

		// Check for record-like patterns (numeric/alphanumeric sequences)
		let is_record_pattern = value
			.chars()
			.all(|c| c.is_alphanumeric() || c == '-' || c == '_');
		if is_record_pattern && value.len() >= 5 {
			confidence += 0.2;
		}

		// Check for common record keywords
		let lower_value = value.to_lowercase();
		let record_keywords = ["record", "ref", "reference", "number", "id", "code"];

		for keyword in &record_keywords {
			if lower_value.contains(keyword) {
				confidence += 0.05;
			}
		}

		Ok(confidence)
	}
}
