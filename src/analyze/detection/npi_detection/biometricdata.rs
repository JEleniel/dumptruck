use crate::{datafile::DataFieldType, detection::DetectionError};

pub struct BiometricData {}

impl BiometricData {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Fingerprints, retina scans, voiceprints, facial recognition, and other biometric data
		// Identified primarily by headers/labels
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI {
			confidence += 0.5;
		}

		// Check for common biometric-related patterns in the column name or data
		let lower_value = value.to_lowercase();
		let biometric_keywords = [
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

		for keyword in &biometric_keywords {
			if lower_value.contains(keyword) {
				confidence += 0.1;
			}
		}

		Ok(confidence)
	}
}
