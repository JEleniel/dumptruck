use crate::analyze::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::isocountrycodes::ISO_COUNTRY_CODES},
};
use crate::util::Checksum;

pub struct BankIBAN {}

impl BankIBAN {
	pub fn detect(value: &str, _column_type: DataFieldType) -> Result<f32, DetectionError> {
		// International Bank Account Number (IBAN): 2 letter ISO country code, 2 check digits,
		// 15-34 characters, set [A-Z0-9], optional spaces every 4 characters
		// checksum MOD-97-10
		// column_type is not used for IBAN detection currently

		let mut confidence: f32 = 0.0;

		// Check country code and reject if invalid
		let country_code = &value[0..2];
		if !ISO_COUNTRY_CODES.contains(&country_code) {
			return Ok(0.0);
		} else {
			confidence += 0.05;
		}

		if Checksum::validate_iban_mod_97(value)? {
			confidence += 0.95;
		}

		// If the country code and checksum are valid, 100% confidence
		Ok(confidence)
	}
}
