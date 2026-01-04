use regex::Regex;

use crate::{datafile::DataFieldType, detection::DetectionError};

pub struct IMEINumber {}

impl IMEINumber {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// IMEI numbers: set [0-9], 15 digits, checksum Luhn algorithm
		let regex = Regex::new(r"^\d{15}$")?;

		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI {
			confidence += 0.2;
		}

		let haystack = value.replace(&[' ', '-'][..], "");
		if regex.is_match(&haystack) {
			confidence += 0.2;

			// Verify Luhn checksum
			if Self::validate_luhn(&haystack) {
				confidence += 0.6;
			}
		}

		Ok(confidence)
	}

	fn validate_luhn(value: &str) -> bool {
		let digits: Vec<u32> = value.chars().filter_map(|c| c.to_digit(10)).collect();

		if digits.len() != 15 {
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
