use regex::Regex;

use crate::{datafile::DataFieldType, detection::DetectionError};

pub struct EmailAddress {}

impl EmailAddress {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Email addresses, identified by "standard" email format and headers/labels
		let email_regex = Regex::new(
			r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$",
		)?;

		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI {
			confidence += 0.4;
		}

		if email_regex.is_match(value) {
			confidence += 0.6;
		}

		Ok(confidence)
	}
}
