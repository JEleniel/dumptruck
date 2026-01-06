use regex::Regex;

use crate::analyze::{
	datafile::DataFieldType,
	detection::{
		DetectionError,
		npi_detection::{NPIType, isocountrycodes::ISO_COUNTRY_CODES},
	},
};

pub struct BankSWIFTCode {}

impl BankSWIFTCode {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// SWIFT/BIC Code: 8 or 11 characters, bank code (set [A-Z], 4 characters), country code
		// (ISO-3166-1 alpha-2), location code (set [A-Z0-9], 2 characters), optional branch code
		// (set [A-Z0-9], 3 characters), 8-11 characters total
		let regex = Regex::new(r"^[A-Z]{4}[A-Z]{2}[A-Z0-9]{2}([A-Z0-9]{3})?$")?;

		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI(NPIType::OtherRecordNumber(NPIType::BANK_SWIFT_CODE)) {
			confidence += 0.5;
		}

		let normalized = value.to_uppercase();
		if regex.is_match(&normalized) && (normalized.len() == 8 || normalized.len() == 11) {
			confidence += 0.2;

			// Validate country code
			let country_code = &normalized[4..6];
			if ISO_COUNTRY_CODES.contains(&country_code) {
				confidence += 0.3;
			}
		}

		Ok(confidence)
	}
}
