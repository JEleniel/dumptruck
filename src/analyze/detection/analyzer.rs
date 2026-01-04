//! Unified detection and enrichment pipeline aggregating all detection modules.
//!
//! Orchestrates PII/NPI detection, weak password detection, hashed credential detection,
//! and HIBP breach enrichment into a single row-by-row pipeline.
use crate::{
	common::Hash, data::Database, datafile::DataFieldType, detection::npi_detection::NPIType,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Detection results for a single row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
	/// Row number for reference
	pub row_number: usize,
	/// User ID found in the row
	pub user_identity: bool,
	/// Credentials found in the row
	pub credential: bool,
	/// Detected PII findings in the row
	pub npi_type: Option<NPIType>,
	/// Whether row contains weak password hashes
	pub password_weak_hash: bool,
}

pub struct Analyzer {}

impl Analyzer {
	pub fn analyze(
		db: &Database,
		row_number: &usize,
		column_type: &DataFieldType,
		value: &str,
	) -> Result<DetectionResult, AnalyzeError> {
		match column_type {
			DataFieldType::Credential => {
				let leaked_password = Self::detect_leaked_passwords(db, value)?;
				Ok(DetectionResult {
					row_number: row_number.clone(),
					npi_type: None,
					password_weak_hash: leaked_password,
					user_identity: false,
					credential: true,
				})
			}
			DataFieldType::UserIdentity => Ok(DetectionResult {
				row_number: row_number.clone(),
				npi_type: None,
				password_weak_hash: false,
				user_identity: true,
				credential: false,
			}),
			DataFieldType::NPI(npi_type) => {
				let npi_result =
					crate::detection::npi_detection::detect_npi(value, column_type.clone());
				// High confidence and type match check
				if npi_result.npi_detected {
					if npi_result.confidence > 0.8 {
						if let Some(detected_npi_type) = npi_result.npi_type {
							if npi_type == detected_npi_type {
								return Ok(DetectionResult {
									row_number: row_number.clone(),
									npi_type: Some(npi_type.clone()),
									password_weak_hash: false,
									user_identity: false,
									credential: false,
								});
							}
						}
					}
				}
				Ok(DetectionResult {
					row_number: row_number.clone(),
					npi_type: Some(npi_type.clone()),
					password_weak_hash: false,
					user_identity: false,
					credential: false,
				})
			}
			_ => Ok(DetectionResult {
				row_number: 0,
				npi_type: None,
				password_weak_hash: false,
				user_identity: false,
				credential: false,
			}),
		}
	}

	/// Weak password detection function
	///
	/// Detects weak passwords by comparing hashes against a rainbow table.
	/// Only exact matches are detected; this prevents false positives from substring matches.
	fn detect_leaked_passwords(db: &Database, value: &str) -> Result<bool, AnalyzeError> {
		let rainbow = db.rainbowtable.get_all()?;

		let hash_md5 = Hash::calculate_md5(&mut value)?;
		let hash_sha256 = Hash::calculate_sha256(&mut value)?;
		let hash_ntlm = Hash::calculate_ntlm(&mut value)?;

		if rainbow.contains(&hash_md5)
			|| rainbow.contains(&hash_sha256)
			|| rainbow.contains(&hash_ntlm)
		{
			Ok(true)
		} else {
			Ok(false)
		}
	}
}

#[derive(Debug, Error)]
pub enum AnalyzeError {}
