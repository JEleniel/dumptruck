use crate::{datafile::DataFieldType, detection::DetectionError};

pub struct OtherPersonalData {}

impl OtherPersonalData {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Other non-public personal information not covered by specific types
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI {
			confidence += 0.6;
		}

		// Check for personal data keywords
		let lower_value = value.to_lowercase();
		let personal_keywords = [
			"private",
			"personal",
			"secret",
			"intimate",
			"confidential",
			"sensitive",
			"restricted",
		];

		for keyword in &personal_keywords {
			if lower_value.contains(keyword) {
				confidence += 0.1;
			}
		}

		Ok(confidence)
	}
}
