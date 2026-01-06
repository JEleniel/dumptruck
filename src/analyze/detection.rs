//! Detection and analysis of sensitive data
//!
//! This module identifies and flags sensitive information:
//! - PII/NPI detection (emails, phones, SSNs, credit cards, crypto addresses)
//! - Weak password detection via rainbow tables
//! - Anomaly and novelty detection in credential data
//! - Outlier identification for risk scoring

use thiserror::Error;

use crate::ChecksumError;

pub mod npi_detection;

#[derive(Debug, Error)]
pub enum DetectionError {
	#[error("Regex error: {0}")]
	RegexError(#[from] regex::Error),
	#[error("Checksum validation error: {0}")]
	ChecksumError(#[from] ChecksumError),
}
