use regex::Regex;

use crate::{datafile::DataFieldType, detection::DetectionError};

pub struct PersonalIdentificationNumber {}

impl PersonalIdentificationNumber {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Numeric PINs: set [0-9], typically 4-6 digits
		let regex = Regex::new(r"^\d{4,6}$")?;

		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI {
			confidence += 0.4;
		}

		let haystack = value.replace(&[' ', '-'][..], "");
		if regex.is_match(&haystack) {
			confidence += 0.3;
		}

		// PINs of exact length 4 or 6 are very common
		if haystack.len() == 4 || haystack.len() == 6 {
			confidence += 0.2;
		}

		Ok(confidence)
	}
}
