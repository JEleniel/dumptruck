use regex::Regex;

use crate::analyze::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};
use crate::util::Checksum;

pub struct BankRoutingNumber {}

impl BankRoutingNumber {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Bank Routing Number: set [0-9], 9 digits, checksum MOD-10
		let regex = Regex::new(r"^\d{9}$")?;

		let mut confidence: f32 = 0.0;

		// Check if the column type is relevant
		if column_type
			== DataFieldType::NPI(NPIType::OtherRecordNumber(NPIType::BANK_ROUTING_NUMBER))
		{
			confidence += 0.5;
		}

		let haystack = value.replace(&[' ', '-'][..], "");
		if regex.is_match(&haystack) {
			confidence += 0.2;

			let checksum = Checksum::validate_mod_10(&haystack)?;

			if checksum {
				confidence += 0.3;
			}
		}

		Ok(confidence)
	}
}
