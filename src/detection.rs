//! Detection and analysis of sensitive data
//!
//! This module identifies and flags sensitive information:
//! - PII/NPI detection (emails, phones, SSNs, credit cards, crypto addresses)
//! - Weak password detection via rainbow tables
//! - Anomaly and novelty detection in credential data
//! - Outlier identification for risk scoring

pub mod analyzer;
pub mod anomaly_detection;
pub mod npi_detection;
pub mod rainbow_table;

pub use analyzer::DetectionResult;
pub use anomaly_detection::AnomalyScore;
pub use npi_detection::{PiiType, detect_pii};
pub use rainbow_table::WeakPasswordHash;
