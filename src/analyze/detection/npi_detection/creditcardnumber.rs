use regex::Regex;

use crate::{datafile::DataFieldType, detection::DetectionError};

pub struct CreditCardNumber {}

impl CreditCardNumber {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Credit Card Numbers: set [0-9], issuer identification number (6-8 digits), account number,
		// check digit (1 digit), optional spaces every 4 digits, checksum Luhn algorithm
		let regex = Regex::new(r"^\d{13,19}$")?;

		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI {
			confidence += 0.3;
		}

		let haystack = value.replace(&[' ', '-'][..], "");
		if regex.is_match(&haystack) {
			confidence += 0.2;

			// Verify Luhn checksum
			if Self::validate_luhn(&haystack) {
				confidence += 0.5;
			}
		}

		Ok(confidence)
	}

	fn validate_luhn(value: &str) -> bool {
		let digits: Vec<u32> = value.chars().filter_map(|c| c.to_digit(10)).collect();

		if digits.is_empty() {
			return false;
		}

		let sum: u32 = digits
			.iter()
			.rev()
			.enumerate()
			.map(|(i, d)| {
				if i % 2 == 1 {
					let doubled = d * 2;
					if doubled > 9 { doubled - 9 } else { doubled }
				} else {
					*d
				}
			})
			.sum();

		sum % 10 == 0
	}
}
