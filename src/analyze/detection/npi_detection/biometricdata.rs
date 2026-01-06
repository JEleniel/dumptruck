use crate::analyze::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};

pub struct BiometricData {}

impl BiometricData {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Fingerprints, retina scans, voiceprints, facial recognition, and other biometric data
		// Identified primarily by headers/labels
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI(NPIType::BiometricData) {
			confidence += 0.5;
		}

		// Check for common biometric-related patterns in the data
		let lower_value = value.to_lowercase();
		const BIOMETRIC_KEYWORDS: [&str; 9] = [
			"fingerprint",
			"retina",
			"voiceprint",
			"facial",
			"biometric",
			"iris",
			"palm",
			"vein",
			"gait",
		];

		for keyword in &BIOMETRIC_KEYWORDS {
			if lower_value.contains(keyword) {
				confidence += 0.3;
			}
		}

		Ok(confidence)
	}
}
