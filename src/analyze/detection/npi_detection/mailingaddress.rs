use regex::Regex;

use crate::{
	datafile::DataFieldType,
	detection::DetectionError,
	detection::npi_detection::{isocountrycodes::ISO_COUNTRY_CODES, types::NPIType},
};

pub struct MailingAddress {}

impl MailingAddress {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Physical mailing addresses, identified primarily by headers/labels
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI(NPIType::MailingAddress) {
			confidence += 0.5;
		}

		// Check for common address patterns
		let lower_value = value.to_lowercase();
		let address_keywords = [
			"street",
			"st.",
			"avenue",
			"ave.",
			"road",
			"rd.",
			"boulevard",
			"blvd",
			"drive",
			"dr.",
			"lane",
			"ln.",
			"court",
			"ct.",
			"city",
			"state",
			"zip",
			"postal",
			"country",
			"address",
		];

		for keyword in &address_keywords {
			if lower_value.contains(keyword) {
				confidence += 0.05;
			}
		}

		// Check for common address patterns: "123 Main St" or similar
		if value.chars().next().is_some_and(|c| c.is_ascii_digit()) {
			confidence += 0.2;
		}

		// Addresses typically have some length
		if value.len() > 10 {
			confidence += 0.1;
		}

		// Check for country codes
		if Self::has_country_code(value) {
			confidence += 0.15;
		}

		// Check for postal code patterns
		if Self::has_postal_code(value) {
			confidence += 0.15;
		}

		Ok(confidence)
	}

	fn has_country_code(value: &str) -> bool {
		let upper_value = value.to_uppercase();
		for code in &ISO_COUNTRY_CODES {
			if upper_value.contains(code) {
				return true;
			}
		}
		false
	}

	fn has_postal_code(value: &str) -> bool {
		// US ZIP code: 5 digits or ZIP+4 format
		if let Ok(zip_regex) = Regex::new(r"\d{5}(-\d{4})?") {
			if zip_regex.is_match(value) {
				return true;
			}
		}

		// UK Postcode: format like "SW1A 2AA" or "B33 8TH"
		if let Ok(uk_regex) = Regex::new(r"[A-Z]{1,2}\d[A-Z\d]?\s?\d[A-Z]{2}") {
			if uk_regex.is_match(value) {
				return true;
			}
		}

		// Canadian Postal Code: format like "M5V 3A8"
		if let Ok(ca_regex) = Regex::new(r"[A-Z]\d[A-Z]\s?\d[A-Z]\d") {
			if ca_regex.is_match(value) {
				return true;
			}
		}

		// German PLZ: 5 digits
		if let Ok(de_regex) = Regex::new(r"\b\d{5}\b") {
			if de_regex.is_match(value) {
				return true;
			}
		}

		// Generic postal code pattern: 4-10 character alphanumeric codes
		if let Ok(generic_regex) = Regex::new(r"\b[A-Z0-9]{4,10}\b") {
			if generic_regex.is_match(value) {
				return true;
			}
		}

		false
	}
}
