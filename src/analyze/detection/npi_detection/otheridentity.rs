use crate::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};

pub struct OtherIdentity {}

impl OtherIdentity {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Other identity information not covered by specific types, e.g. social media handles
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI(NPIType::OtherIdentity("Other Identity")) {
			confidence += 0.5;
		}

		// Check for social media handle patterns (@, or username-like)
		if value.starts_with('@') {
			confidence += 0.2;
		}

		// Social media handles are typically short and alphanumeric with underscores
		let is_handle_pattern = value
			.chars()
			.all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.');
		if is_handle_pattern && value.len() >= 3 && value.len() <= 50 {
			confidence += 0.1;
		}

		// Check for common identity keywords
		let lower_value = value.to_lowercase();
		let identity_keywords = [
			"username",
			"handle",
			"user_id",
			"userid",
			"screen_name",
			"login",
		];

		for keyword in &identity_keywords {
			if lower_value.contains(keyword) {
				confidence += 0.05;
			}
		}

		Ok(1.0)
	}
}
