use crate::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::isocountrycodes::ISO_COUNTRY_CODES},
};

pub struct BankIBAN {}

impl BankIBAN {
	pub fn detect(value: &str, _column_type: DataFieldType) -> Result<f32, DetectionError> {
		// International Bank Account Number (IBAN): 2 letter ISO country code, 2 check digits,
		// 15-34 characters, set [A-Z0-9], optional spaces every 4 characters
		// checksum MOD-97-10

		let mut confidence: f32 = 0.0;

		// Check country code and reject if invalid
		let country_code = &value[0..2];
		if !ISO_COUNTRY_CODES.contains(&country_code) {
			return Ok(0.0);
		} else {
			confidence += 0.05;
		}

		let check_digits: u32 = value[2..4].into()?;
		let bban = &value[4..].replace(" ", "");
		let rearranged = format!("{}{}{}", bban, country_code, "00");
		let numeric_representation = rearranged
			.chars()
			.map(|c| {
				if c.is_digit(10) {
					c.to_string()
				} else if c.is_ascii_uppercase() {
					(c as u32 - 'A' as u32 + 10).to_string()
				} else {
					"".to_string()
				}
			})
			.collect::<String>();
		let mod97 = numeric_representation
			.as_bytes()
			.chunks(7)
			.fold(0u32, |acc, chunk| {
				let part = format!("{}{}", acc, std::str::from_utf8(chunk).unwrap());
				part.parse::<u32>().unwrap() % 97
			});
		if (98 - mod97) as u32 == check_digits {
			confidence += 0.95;
		}

		// If the country code and checksum are valid, 100% confidence
		Ok(confidence)
	}
}
