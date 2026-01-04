//! Anomaly & Novelty Detection Module (Stage 10)
//!
//! Identifies suspicious patterns and unexpected values in breach data.
//!
//! **Key Capabilities:**
//! - Entropy outliers: Values with unusual character distribution (>3σ deviation)
//! - Unseen field combinations: Unusual combinations not seen before
//! - Rare domains: Top-level domains appearing rarely in dataset
//! - Unexpected credential formats: Passwords with unusual structure
//! - Baseline deviation: Statistical outliers from dataset baseline
//!
//! Helps identify:
//! - Injected/synthetic test data
//! - Malicious modifications
//! - Credential quality issues
//! - Data quality problems

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during anomaly detection
#[derive(Error, Debug)]
pub enum AnomalyDetectionError {
	#[error("Insufficient data: {0}")]
	InsufficientData(String),

	#[error("Invalid baseline: {0}")]
	InvalidBaseline(String),

	#[error("Calculation failed: {0}")]
	CalculationFailed(String),
}

/// Types of anomalies that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
	/// Entropy value is statistical outlier (>3σ from mean)
	EntropyOutlier,
	/// Field combination not seen in dataset before
	UnseenCombination,
	/// Domain appears rarely in dataset
	RareDomain,
	/// Credential format is unusual
	UnusualFormat,
	/// Statistical deviation from baseline
	BaselineDeviation,
	/// Value length is statistical outlier
	LengthOutlier,
	/// Suspiciously uniform character distribution
	UniformDistribution,
}

impl std::fmt::Display for AnomalyType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AnomalyType::EntropyOutlier => write!(f, "entropy_outlier"),
			AnomalyType::UnseenCombination => write!(f, "unseen_combination"),
			AnomalyType::RareDomain => write!(f, "rare_domain"),
			AnomalyType::UnusualFormat => write!(f, "unusual_format"),
			AnomalyType::BaselineDeviation => write!(f, "baseline_deviation"),
			AnomalyType::LengthOutlier => write!(f, "length_outlier"),
			AnomalyType::UniformDistribution => write!(f, "uniform_distribution"),
		}
	}
}

/// A detected anomaly in data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyScore {
	/// The anomalous value or field
	pub subject: String,

	/// Type of anomaly detected
	pub anomaly_type: AnomalyType,

	/// Risk score (0-100) indicating severity
	/// 100 = definite anomaly, 50 = moderate suspicion, 0 = normal
	pub risk_score: u8,

	/// Statistical metric used for detection
	pub metric: f64,

	/// Threshold that was exceeded
	pub threshold: f64,

	/// Optional explanation of the anomaly
	pub explanation: Option<String>,
}

impl AnomalyScore {
	/// Create a new anomaly score
	pub fn new(
		subject: String,
		anomaly_type: AnomalyType,
		risk_score: u8,
		metric: f64,
		threshold: f64,
	) -> Self {
		Self {
			subject,
			anomaly_type,
			risk_score,
			metric,
			threshold,
			explanation: None,
		}
	}

	/// Add explanation
	pub fn with_explanation(mut self, explanation: String) -> Self {
		self.explanation = Some(explanation);
		self
	}
}

/// Calculate Shannon entropy of a string
/// Range: 0.0 (all same character) to ~5.0 (very random)
pub fn calculate_entropy(value: &str) -> f64 {
	if value.is_empty() {
		return 0.0;
	}

	// Count character frequencies
	let mut frequencies: HashMap<char, usize> = HashMap::new();
	for ch in value.chars() {
		*frequencies.entry(ch).or_insert(0) += 1;
	}

	// Calculate entropy
	let len = value.len() as f64;
	let mut entropy = 0.0;

	for count in frequencies.values() {
		let probability = *count as f64 / len;
		entropy -= probability * probability.log2();
	}

	entropy
}

/// Detect entropy outliers (values with unusual randomness)
///
/// High entropy (very random) can indicate:
/// - Injected test data
/// - Encrypted/hashed values
/// - Corrupted data
///
/// Low entropy (very uniform) can indicate:
/// - Padding or artificial data
/// - Placeholder values
pub fn detect_entropy_outlier(
	value: &str,
	mean_entropy: f64,
	std_dev: f64,
) -> Option<AnomalyScore> {
	let value_entropy = calculate_entropy(value);

	// Check if entropy is >3σ from mean
	let z_score = (value_entropy - mean_entropy).abs() / std_dev.max(0.1);

	if z_score > 3.0 {
		let risk = if value_entropy > mean_entropy + 3.0 * std_dev {
			// Very high entropy = likely synthetic
			85
		} else {
			// Very low entropy = likely placeholder
			75
		};

		return Some(AnomalyScore::new(
			value.to_string(),
			AnomalyType::EntropyOutlier,
			risk,
			value_entropy,
			mean_entropy + 3.0 * std_dev,
		));
	}

	None
}

/// Detect rare domains in the dataset
///
/// Domains appearing in <1% of records may indicate:
/// - Typos or malformed data
/// - Synthetic/test accounts
/// - Unusual account activity
pub fn detect_rare_domain(
	email: &str,
	domain_frequencies: &HashMap<String, usize>,
	total_records: usize,
) -> Option<AnomalyScore> {
	// Extract domain
	let at_idx = email.find('@')?;
	let domain = &email[at_idx + 1..];

	// Get frequency
	let count = domain_frequencies.get(domain).copied().unwrap_or(0);
	let frequency = count as f64 / total_records as f64;

	// If domain appears in <1% of records, flag as rare
	if frequency < 0.01 && count > 0 {
		let risk = (100.0 * (1.0 - frequency)).min(100.0) as u8;

		return Some(
			AnomalyScore::new(
				domain.to_string(),
				AnomalyType::RareDomain,
				risk,
				frequency,
				0.01,
			)
			.with_explanation(format!(
				"Domain appears in only {:.2}% of records ({} occurrences)",
				frequency * 100.0,
				count
			)),
		);
	}

	None
}

/// Detect unusual password formats
///
/// Typical patterns:
/// - Mixed case + numbers + symbols
/// - Reasonable length (8-64 chars)
/// - Human-memorable or random
///
/// Unusual patterns:
/// - Only lowercase or only numbers
/// - Very short (<4) or very long (>128)
/// - Suspicious patterns (111111, qwerty, etc.)
pub fn detect_unusual_password_format(password: &str) -> Option<AnomalyScore> {
	if password.is_empty() || password.len() > 256 {
		return Some(
			AnomalyScore::new(
				password.to_string(),
				AnomalyType::UnusualFormat,
				90,
				password.len() as f64,
				128.0,
			)
			.with_explanation("Password length is outside normal range".to_string()),
		);
	}

	// Check for suspicious patterns
	let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
	let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
	let has_digit = password.chars().any(|c| c.is_ascii_digit());
	let has_special = password.chars().any(|c| !c.is_alphanumeric());

	// Suspiciously uniform (all one type)
	let variety_count = [has_uppercase, has_lowercase, has_digit, has_special]
		.iter()
		.filter(|&&x| x)
		.count();

	if variety_count <= 1 && password.len() < 20 {
		return Some(
			AnomalyScore::new(
				password.to_string(),
				AnomalyType::UniformDistribution,
				60,
				variety_count as f64,
				2.0,
			)
			.with_explanation("Password has very low character variety".to_string()),
		);
	}

	None
}

/// Detect field combinations not seen before
///
/// Returns Some if this combination appears for first time
pub fn detect_unseen_combination(
	fields: &[&str],
	seen_combinations: &HashSet<String>,
) -> Option<AnomalyScore> {
	let combination = fields.join("|");

	if !seen_combinations.contains(&combination) {
		return Some(
			AnomalyScore::new(
				combination.clone(),
				AnomalyType::UnseenCombination,
				40, // Moderate risk - could be new but legitimate
				1.0,
				1.0,
			)
			.with_explanation("First occurrence of this field combination".to_string()),
		);
	}

	None
}

/// Detect length outliers
///
/// Values with extreme length (very short or very long) may be:
/// - Truncated data
/// - Corrupted
/// - Injected content
pub fn detect_length_outlier(value: &str, mean_length: f64, std_dev: f64) -> Option<AnomalyScore> {
	if std_dev < 0.1 {
		return None;
	}

	let value_length = value.len() as f64;
	let z_score = (value_length - mean_length).abs() / std_dev;

	if z_score > 3.0 {
		let risk = if value_length > mean_length + 3.0 * std_dev {
			70 // Very long = possibly corrupted or injected
		} else {
			60 // Very short = possibly truncated
		};

		return Some(AnomalyScore::new(
			value.to_string(),
			AnomalyType::LengthOutlier,
			risk,
			value_length,
			mean_length + 3.0 * std_dev,
		));
	}

	None
}

/// Calculate basic statistics for a dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetBaseline {
	/// Mean entropy of all values
	pub mean_entropy: f64,

	/// Standard deviation of entropy
	pub entropy_std_dev: f64,

	/// Mean length of values
	pub mean_length: f64,

	/// Standard deviation of length
	pub length_std_dev: f64,

	/// Most common domains (for email detection)
	pub common_domains: Vec<(String, usize)>,

	/// Total records analyzed
	pub record_count: usize,
}

impl DatasetBaseline {
	/// Create baseline from a sample of values
	pub fn from_sample(values: &[&str]) -> Result<Self, AnomalyDetectionError> {
		if values.is_empty() {
			return Err(AnomalyDetectionError::InsufficientData(
				"Sample is empty".to_string(),
			));
		}

		// Calculate entropy statistics
		let entropies: Vec<f64> = values.iter().map(|v| calculate_entropy(v)).collect();
		let mean_entropy = entropies.iter().sum::<f64>() / entropies.len() as f64;

		let variance = entropies
			.iter()
			.map(|e| (e - mean_entropy).powi(2))
			.sum::<f64>()
			/ entropies.len() as f64;
		let entropy_std_dev = variance.sqrt();

		// Calculate length statistics
		let lengths: Vec<f64> = values.iter().map(|v| v.len() as f64).collect();
		let mean_length = lengths.iter().sum::<f64>() / lengths.len() as f64;

		let var_len = lengths
			.iter()
			.map(|l| (l - mean_length).powi(2))
			.sum::<f64>()
			/ lengths.len() as f64;
		let length_std_dev = var_len.sqrt();

		// Count domain frequencies
		let mut domain_counts: HashMap<String, usize> = HashMap::new();
		for value in values {
			if let Some(at_idx) = value.find('@') {
				let domain = value[at_idx + 1..].to_lowercase();
				*domain_counts.entry(domain).or_insert(0) += 1;
			}
		}

		let mut common_domains: Vec<_> = domain_counts.into_iter().collect();
		common_domains.sort_by(|a, b| b.1.cmp(&a.1));
		common_domains.truncate(10);

		Ok(DatasetBaseline {
			mean_entropy,
			entropy_std_dev,
			mean_length,
			length_std_dev,
			common_domains,
			record_count: values.len(),
		})
	}
}

#[cfg(test)]
#[path = "anomaly_detection_tests.rs"]
mod tests;
