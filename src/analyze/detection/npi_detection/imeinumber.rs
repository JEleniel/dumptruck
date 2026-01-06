use regex::Regex;

use crate::analyze::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};
use crate::util::Checksum;

pub struct IMEINumber {}

impl IMEINumber {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// IMEI numbers: set [0-9], 15 digits, checksum Luhn algorithm
		let regex = Regex::new(r"^\d{15}$")?;

		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI(NPIType::IMEI) {
			confidence += 0.5;
		}

		let haystack = value.replace(&[' ', '-'][..], "");
		if regex.is_match(&haystack) {
			confidence += 0.2;

			// Verify Luhn checksum
			if Checksum::validate_luhn(&haystack) {
				confidence += 0.3;
			}
		}

		Ok(confidence)
	}
}
