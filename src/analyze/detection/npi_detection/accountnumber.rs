use regex::Regex;

use crate::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};

pub struct AccountNumber {}

impl AccountNumber {
	/// Detects account numbers
	/// Confidence is based on:
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		let regex = Regex::new(r"\d{8,17}")?;

		let mut confidence: f32 = 0.0;

		// Check if the column type is relevant
		if column_type == DataFieldType::NPI(NPIType::OtherRecordNumber("Account Number")) {
			confidence += 0.5;
		}

		// Check regex match, stripping spaces and dashes for simplicity
		let haystack = value.replace(&[' ', '-'][..], "");
		if regex.is_match(&haystack) {
			confidence += 0.3;
		}

		// 80% max confidence because Account Numbers can overlap with other numeric types
		Ok(confidence)
	}
}
