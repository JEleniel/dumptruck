use thiserror::Error;

pub struct Checksum {}

impl Checksum {
	pub fn validate_mod_10(digits: &str) -> Result<bool, ChecksumError> {
		let weighted_sum: u32 = digits
			.chars()
			.enumerate()
			.map(|(i, d)| {
				let weights = [3, 7, 1];
				u8::from_str_radix(&d.to_string(), 10).map(|digit| digit * weights[i % 3])
			})
			.collect::<Result<Vec<u8>, _>>()?
			.iter()
			.map(|&x| x as u32)
			.sum();

		Ok(weighted_sum % 10 == 0)
	}

	pub fn validate_iban_mod_97(iban: &str) -> Result<bool, ChecksumError> {
		let rearranged = format!("{}{}{}", &iban[4..], &iban[0..2], &iban[2..4]);
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

		Ok(mod97 == 1)
	}

	pub fn validate_luhn(value: &str) -> bool {
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

#[derive(Debug, Error)]
pub enum ChecksumError {
	#[error("Invalid digit in input string")]
	InvalidDigit(#[from] std::num::ParseIntError),
}
