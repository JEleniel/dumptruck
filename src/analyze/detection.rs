//! Detection and analysis of sensitive data
//!
//! This module identifies and flags sensitive information:
//! - PII/NPI detection (emails, phones, SSNs, credit cards, crypto addresses)
//! - Weak password detection via rainbow tables
//! - Anomaly and novelty detection in credential data
//! - Outlier identification for risk scoring

use thiserror::Error;

pub mod npi_detection;

#[derive(Debug, Error)]
pub enum DetectionError {}
