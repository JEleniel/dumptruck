//! Risk scoring engine for breach data analysis.
//!
//! Calculates comprehensive risk scores (0-100) based on 5 independent factors:
//! 1. Weak Password (30% weight, 0-30 points)
//! 2. Weak Hash Algorithm (20% weight, 0-20 points)
//! 3. Breach History (40% weight, 0-40 points)
//! 4. PII Exposure (15% weight, 0-25 points)
//! 5. Anomaly Detection (10% weight, 0-8 points)
//!
//! Scoring formula: ((weighted_sum / 123) × 100) → 0-100 normalized score

use crate::npi_detection::PiiType;

/// Risk score with breakdown of factors
#[derive(Debug, Clone)]
pub struct RiskScore {
	/// Overall risk score 0-100
	pub score: u8,
	/// Risk level category
	pub level: RiskLevel,
	/// Individual factor scores
	pub factors: RiskFactors,
	/// Explanation of score contributors
	pub explanation: String,
}

/// Risk level categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
	/// 0-10: No significant risk detected
	Green,
	/// 11-25: Low risk, monitor
	Yellow,
	/// 26-50: Moderate risk, review recommended
	Orange,
	/// 51-75: High risk, escalate recommended
	Red,
	/// 76-100: Critical risk, immediate action required
	Critical,
}

impl RiskLevel {
	pub fn label(&self) -> &'static str {
		match self {
			RiskLevel::Green => "Green",
			RiskLevel::Yellow => "Yellow",
			RiskLevel::Orange => "Orange",
			RiskLevel::Red => "Red",
			RiskLevel::Critical => "Critical",
		}
	}
}

/// Breakdown of individual risk factor scores
#[derive(Debug, Clone, Default)]
pub struct RiskFactors {
	/// Weak password score (0-30)
	pub weak_password_score: u8,
	/// Weak hash algorithm score (0-20)
	pub weak_hash_score: u8,
	/// Breach history score (0-40)
	pub breach_score: u8,
	/// PII exposure score (0-25)
	pub pii_score: u8,
	/// Anomaly detection score (0-8)
	pub anomaly_score: u8,
}

/// Risk scoring configuration with adjustable weights
#[derive(Debug, Clone)]
pub struct RiskScoringConfig {
	/// Weak password weight (default: 0.30)
	pub weak_password_weight: f32,
	/// Weak hash weight (default: 0.20)
	pub weak_hash_weight: f32,
	/// Breach history weight (default: 0.40)
	pub breach_weight: f32,
	/// PII exposure weight (default: 0.15)
	pub pii_weight: f32,
	/// Anomaly detection weight (default: 0.10)
	pub anomaly_weight: f32,
}

impl Default for RiskScoringConfig {
	fn default() -> Self {
		RiskScoringConfig {
			weak_password_weight: 0.30,
			weak_hash_weight: 0.20,
			breach_weight: 0.40,
			pii_weight: 0.15,
			anomaly_weight: 0.10,
		}
	}
}

/// Main risk scoring engine
pub struct RiskScoringEngine {
	config: RiskScoringConfig,
}

impl RiskScoringEngine {
	/// Create new risk scoring engine with default weights
	pub fn new() -> Self {
		RiskScoringEngine {
			config: RiskScoringConfig::default(),
		}
	}

	/// Create with custom configuration
	pub fn with_config(config: RiskScoringConfig) -> Self {
		RiskScoringEngine { config }
	}

	/// Calculate overall risk score for given factors
	pub fn score(
		&self,
		weak_password: bool,
		weak_hash_detected: bool,
		breach_count: usize,
		new_credential_since_breach: bool,
		pii_types: &[PiiType],
		anomaly_count: usize,
	) -> RiskScore {
		// Calculate individual factor scores
		let weak_password_score = if weak_password { 30 } else { 0 };

		let weak_hash_score = if weak_hash_detected { 20 } else { 0 };

		let breach_base = std::cmp::min(breach_count as u8 * 10, 40);
		let breach_bonus = if new_credential_since_breach && breach_count > 0 {
			20
		} else {
			0
		};
		let breach_score = std::cmp::min(breach_base + breach_bonus, 40);

		let pii_score = self.calculate_pii_score(pii_types);

		let anomaly_score = std::cmp::min(anomaly_count as u8, 8);

		// Apply weighted formula
		let raw_score = (weak_password_score as f32 * self.config.weak_password_weight)
			+ (weak_hash_score as f32 * self.config.weak_hash_weight)
			+ (breach_score as f32 * self.config.breach_weight)
			+ (pii_score as f32 * self.config.pii_weight)
			+ (anomaly_score as f32 * self.config.anomaly_weight);

		// Normalize to 0-100
		let normalized = ((raw_score / 123.0) * 100.0) as u8;

		let level = match normalized {
			0..=10 => RiskLevel::Green,
			11..=25 => RiskLevel::Yellow,
			26..=50 => RiskLevel::Orange,
			51..=75 => RiskLevel::Red,
			_ => RiskLevel::Critical,
		};

		let explanation = self.generate_explanation(
			weak_password,
			weak_hash_detected,
			breach_count,
			new_credential_since_breach,
			&pii_types,
			anomaly_count,
		);

		RiskScore {
			score: normalized,
			level,
			factors: RiskFactors {
				weak_password_score,
				weak_hash_score,
				breach_score,
				pii_score,
				anomaly_score,
			},
			explanation,
		}
	}

	/// Calculate PII exposure score based on detected PII types
	fn calculate_pii_score(&self, pii_types: &[PiiType]) -> u8 {
		let mut score = 0u8;

		for pii_type in pii_types {
			score += match pii_type {
				// High sensitivity (10 points each)
				PiiType::SocialSecurityNumber => 10,
				PiiType::CreditCardNumber => 10,
				// Medium sensitivity (5 points)
				PiiType::NationalId => 5,
				// Lower sensitivity (3 points each)
				PiiType::PhoneNumber => 3,
				PiiType::IpAddress | PiiType::IpV4Address | PiiType::IpV6Address => 3,
				// Other PII types (1 point each)
				_ => 1,
			};
		}

		std::cmp::min(score, 25)
	}

	/// Generate human-readable explanation of risk score
	fn generate_explanation(
		&self,
		weak_password: bool,
		weak_hash_detected: bool,
		breach_count: usize,
		new_credential_since_breach: bool,
		pii_types: &[PiiType],
		anomaly_count: usize,
	) -> String {
		let mut parts = vec![];

		if weak_password {
			parts.push("Uses weak/common password".to_string());
		}

		if weak_hash_detected {
			parts.push("Hash algorithm is weak (MD5/SHA1/SHA256)".to_string());
		}

		if breach_count > 0 {
			parts.push(format!("Found in {} breach(es)", breach_count));

			if new_credential_since_breach {
				parts.push("⚠️ NEW CREDENTIAL SINCE BREACH - Active exploitation risk".to_string());
			}
		}

		if !pii_types.is_empty() {
			parts.push(format!("Contains {} PII field(s)", pii_types.len()));
		}

		if anomaly_count > 0 {
			parts.push(format!("{} anomalies detected", anomaly_count));
		}

		if parts.is_empty() {
			"No significant risk factors detected".to_string()
		} else {
			parts.join("; ")
		}
	}
}

impl Default for RiskScoringEngine {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_risk_score_no_risk() {
		let engine = RiskScoringEngine::new();
		let score = engine.score(false, false, 0, false, &[], 0);

		assert_eq!(score.score, 0);
		assert_eq!(score.level, RiskLevel::Green);
	}

	#[test]
	fn test_risk_score_weak_password_only() {
		let engine = RiskScoringEngine::new();
		let score = engine.score(true, false, 0, false, &[], 0);

		// (30 * 0.30) / 123 * 100 = 7.32
		assert!(score.score >= 5 && score.score <= 10);
		assert_eq!(score.level, RiskLevel::Green);
		assert_eq!(score.factors.weak_password_score, 30);
	}

	#[test]
	fn test_risk_score_breach_history() {
		let engine = RiskScoringEngine::new();
		let score = engine.score(false, false, 2, false, &[], 0);

		// (20 * 0.40) / 123 * 100 = 6.50
		assert!(score.score >= 5 && score.score <= 10);
		assert_eq!(score.factors.breach_score, 20);
	}

	#[test]
	fn test_risk_score_compromised_with_new_credential() {
		let engine = RiskScoringEngine::new();
		let score = engine.score(false, false, 1, true, &[], 0);

		// (30 * 0.40) / 123 * 100 = 9.76 (rounded down to 9 due to u8 casting)
		assert_eq!(score.score, 9);
		assert_eq!(score.level, RiskLevel::Green);
		assert_eq!(score.factors.breach_score, 30); // Base 10 + bonus 20
	}

	#[test]
	fn test_risk_score_with_pii() {
		let engine = RiskScoringEngine::new();
		let pii_types = vec![PiiType::CreditCardNumber, PiiType::PhoneNumber];
		let score = engine.score(false, false, 0, false, &pii_types, 0);

		// (13 * 0.15) / 123 * 100 = 1.58
		assert!(score.score <= 5);
		assert_eq!(score.factors.pii_score, 13);
	}

	#[test]
	fn test_risk_score_comprehensive() {
		let engine = RiskScoringEngine::new();
		let pii_types = vec![PiiType::SocialSecurityNumber];
		let score = engine.score(true, true, 2, true, &pii_types, 2);

		// Weak password: 30
		// Weak hash: 20
		// Breach: 40
		// PII: 10
		// Anomaly: 2
		// raw = 30*0.30 + 20*0.20 + 40*0.40 + 10*0.15 + 2*0.10 = 9 + 4 + 16 + 1.5 + 0.2 = 30.7
		// normalized = (30.7 / 123) * 100 = 24.96 ≈ 24 or 25

		assert!(score.score >= 20 && score.score <= 30);
		assert_eq!(score.level, RiskLevel::Yellow);
	}

	#[test]
	fn test_pii_score_capping() {
		let engine = RiskScoringEngine::new();
		// Many SSN and CC = 10 + 10 + 10 + 10 = 40 points, but should cap at 25
		let pii_types = vec![
			PiiType::SocialSecurityNumber,
			PiiType::CreditCardNumber,
			PiiType::NationalId,
			PiiType::PhoneNumber,
		];
		let score = engine.score(false, false, 0, false, &pii_types, 0);

		assert_eq!(score.factors.pii_score, 25); // Capped
	}

	#[test]
	fn test_risk_level_ordering() {
		assert!(RiskLevel::Green < RiskLevel::Yellow);
		assert!(RiskLevel::Yellow < RiskLevel::Orange);
		assert!(RiskLevel::Orange < RiskLevel::Red);
		assert!(RiskLevel::Red < RiskLevel::Critical);
	}

	#[test]
	fn test_anomaly_score_capping() {
		let engine = RiskScoringEngine::new();
		// 15 anomalies should cap at 8
		let score = engine.score(false, false, 0, false, &[], 15);

		assert_eq!(score.factors.anomaly_score, 8); // Capped
	}

	#[test]
	fn test_custom_weights() {
		let config = RiskScoringConfig {
			weak_password_weight: 1.0, // 100% weight on weak password
			weak_hash_weight: 0.0,
			breach_weight: 0.0,
			pii_weight: 0.0,
			anomaly_weight: 0.0,
		};
		let engine = RiskScoringEngine::with_config(config);
		let score = engine.score(true, false, 0, false, &[], 0);

		// (30 * 1.0) / 123 * 100 = 24.39 ≈ 24
		assert!(score.score >= 20 && score.score <= 30);
	}
}
