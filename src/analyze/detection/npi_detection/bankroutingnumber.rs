use regex::Regex;

use crate::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};

pub struct BankRoutingNumber {}

impl BankRoutingNumber {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Bank Routing Number: set [0-9], 9 digits, checksum MOD-10
		let regex = Regex::new(r"^\d{9}$")?;

		let mut confidence: f32 = 0.0;

		// Check if the column type is relevant
		if column_type == DataFieldType::NPI(NPIType::OtherRecordNumber("Bank Routing Number")) {
			confidence += 0.5;
		}

		let haystack = value.replace(&[' ', '-'][..], "");
		if regex.is_match(&haystack) {
			confidence += 0.2;

			// Verify MOD-10 checksum
			let digits: Vec<u32> = haystack
				.chars()
				.map(|c| c.to_digit(10).unwrap_or(0))
				.collect();

			let weighted_sum: u32 = digits
				.iter()
				.enumerate()
				.map(|(i, d)| {
					let weights = [3, 7, 1];
					d * weights[i % 3]
				})
				.sum();

			if weighted_sum % 10 == 0 {
				confidence += 0.5;
			}
		}

		Ok(confidence)
	}
}
