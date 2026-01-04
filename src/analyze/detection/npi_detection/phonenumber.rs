use regex::Regex;

use crate::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};

pub struct PhoneNumber {}

impl PhoneNumber {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Phone numbers: E.164 format, international and national formats, identified primarily by headers/labels
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI(NPIType::PhoneNumber) {
			confidence += 0.5;
		}

		// E.164 format: +1234567890 (1-15 digits after +)
		if Self::is_e164_format(value) {
			return Ok(confidence + 0.7);
		}

		// Common international format: +1 (555) 123-4567
		if Self::is_formatted_phone(value) {
			return Ok(confidence + 0.6);
		}

		// National format: (555) 123-4567 or 555-123-4567
		if Self::is_national_format(value) {
			return Ok(confidence + 0.5);
		}

		// US format with 1- prefix: 1-555-123-4567
		if Self::is_one_prefix_format(value) {
			return Ok(confidence + 0.5);
		}

		// 10-digit format with dots: 555.123.4567
		if Self::is_dot_separated_format(value) {
			return Ok(confidence + 0.5);
		}

		// Space-separated format: 555 123 4567
		if Self::is_space_separated_format(value) {
			return Ok(confidence + 0.5);
		}

		// 7-digit local format: 123-4567 or 123.4567
		if Self::is_short_local_format(value) {
			return Ok(confidence + 0.3);
		}

		// Plain digits: 10-15 consecutive digits
		if Self::is_plain_digits(value) {
			return Ok(confidence + 0.4);
		}

		Ok(confidence)
	}

	fn is_e164_format(value: &str) -> bool {
		let regex = Regex::new(r"^\+[1-9]\d{1,14}$").ok();
		regex.is_some_and(|re| re.is_match(value))
	}

	fn is_formatted_phone(value: &str) -> bool {
		let regex = Regex::new(r"^\+\d{1,3}\s?(\(\d{1,3}\)|\d{1,3})\s?\d{1,4}[-.\s]?\d{1,4}").ok();
		regex.is_some_and(|re| re.is_match(value))
	}

	fn is_national_format(value: &str) -> bool {
		let regex = Regex::new(r"^(\(\d{3}\)|\d{3})[-.\s]?\d{3}[-.\s]?\d{4}").ok();
		regex.is_some_and(|re| re.is_match(value))
	}

	fn is_dot_separated_format(value: &str) -> bool {
		// 10-digit format with dots: 555.123.4567
		let regex = Regex::new(r"^\d{3}\.\d{3}\.\d{4}$").ok();
		regex.is_some_and(|re| re.is_match(value))
	}

	fn is_short_local_format(value: &str) -> bool {
		// 7-digit local format: 123-4567
		let regex = Regex::new(r"^\d{3}[-.]?\d{4}$").ok();
		regex.is_some_and(|re| re.is_match(value))
	}

	fn is_space_separated_format(value: &str) -> bool {
		// Space-separated 10-digit: 555 123 4567
		let regex = Regex::new(r"^\d{3}\s\d{3}\s\d{4}$").ok();
		regex.is_some_and(|re| re.is_match(value))
	}

	fn is_one_prefix_format(value: &str) -> bool {
		// US format with 1- prefix: 1-555-123-4567
		let regex = Regex::new(r"^1[-.]?\d{3}[-.]?\d{3}[-.]?\d{4}$").ok();
		regex.is_some_and(|re| re.is_match(value))
	}

	fn is_plain_digits(value: &str) -> bool {
		let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
		(10..=15).contains(&digits.len())
	}
}
