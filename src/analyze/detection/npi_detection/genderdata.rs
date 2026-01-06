use crate::analyze::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};

pub struct GenderData {}

impl GenderData {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Gender information, identified primarily by headers/labels, may also be identified by the
		// M/F/O convention, Male/Female/Other, or similar
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI(NPIType::GenderData) {
			confidence += 0.5;
		}

		let normalized = value.to_lowercase();

		// Single-character gender codes
		if normalized == "m" || normalized == "f" || normalized == "x" || normalized == "o" {
			confidence += 0.3;
		}

		// Common gender strings
		const GENDER_KEYWORDS: [&str; 10] = [
			"male",
			"female",
			"man",
			"woman",
			"boy",
			"girl",
			"other",
			"non binary",
			"non-binary",
			"nonbinary",
		];

		for keyword in &GENDER_KEYWORDS {
			if normalized.contains(keyword) {
				confidence += 0.3;
			}
		}

		Ok(confidence)
	}
}
