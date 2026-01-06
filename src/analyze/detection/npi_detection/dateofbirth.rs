use crate::analyze::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};

pub struct DateOfBirth {}

impl DateOfBirth {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Date of Birth, identified primarily by headers/labels
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI(NPIType::DateOfBirth) {
			confidence += 0.5;
		}

		// Check for date-like patterns (YYYY-MM-DD, MM/DD/YYYY, DD-MM-YYYY, etc.)
		if Self::is_date_format(value) {
			confidence += 0.3;
		}

		Ok(confidence)
	}

	fn is_date_format(value: &str) -> bool {
		// Common date formats: YYYY-MM-DD, MM/DD/YYYY, DD-MM-YYYY, YYYY/MM/DD
		let parts: Vec<&str> = value.split(|c| c == '-' || c == '/' || c == '.').collect();

		if parts.len() != 3 {
			return false;
		}

		// Check if all parts are numeric
		parts
			.iter()
			.all(|part| part.chars().all(|c| c.is_ascii_digit()) && (2..=4).contains(&part.len()))
	}
}
