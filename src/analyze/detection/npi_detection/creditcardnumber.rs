use regex::Regex;

use crate::analyze::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};
use crate::util::Checksum;

pub struct CreditCardNumber {}

impl CreditCardNumber {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Credit Card Numbers: set [0-9], issuer identification number (6-8 digits), account number,
		// check digit (1 digit), optional spaces every 4 digits, checksum Luhn algorithm
		let regex = Regex::new(r"^\d{13,19}$")?;

		let mut confidence: f32 = 0.0;

		if column_type
			== DataFieldType::NPI(NPIType::OtherRecordNumber(NPIType::CREDIT_CARD_NUMBER))
		{
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
