//! File metadata and processing tracking.

use rusqlite::Connection;
use std::io;

/// Parameters for a chain of custody audit record.
#[derive(Debug, Clone)]
pub struct CustodyRecord<'a> {
	/// Unique file identifier
	pub file_id: &'a str,
	/// Custody record ID
	pub record_id: &'a str,
	/// Action performed (e.g., "INGEST", "EXPORT")
	pub custody_action: &'a str,
	/// Operator who performed the action
	pub operator: &'a str,
	/// Hash of the file at this point
	pub file_hash: &'a str,
	/// ED25519 signature of the custody record
	pub signature: &'a [u8],
	/// ED25519 public key for verification
	pub public_key: &'a [u8],
}

/// Insert file metadata record.
pub fn insert_file_metadata(
	conn: &Connection,
	file_id: &str,
	original_filename: &str,
	sha256_hash: &str,
	file_size: i64,
) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO file_metadata \
			 (file_id, original_filename, sha256_hash, file_size) VALUES (?1, ?2, ?3, ?4)",
			rusqlite::params![file_id, original_filename, sha256_hash, file_size],
		)
		.map_err(io::Error::other)?;

	Ok(rows > 0)
}

/// Insert chain of custody record.
pub fn insert_custody_record(conn: &Connection, record: &CustodyRecord<'_>) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO chain_of_custody \
			 (file_id, record_id, custody_action, operator, file_hash, signature, public_key) \
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
			rusqlite::params![
				record.file_id,
				record.record_id,
				record.custody_action,
				record.operator,
				record.file_hash,
				record.signature,
				record.public_key,
			],
		)
		.map_err(io::Error::other)?;

	Ok(rows > 0)
}

/// Insert anomaly score for file subject.
pub fn insert_anomaly_score(
	conn: &Connection,
	file_id: &str,
	subject_hash: &str,
	anomaly_type: &str,
	risk_score: i32,
) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO anomaly_scores \
			 (file_id, subject_hash, anomaly_type, risk_score) VALUES (?1, ?2, ?3, ?4)",
			rusqlite::params![file_id, subject_hash, anomaly_type, risk_score],
		)
		.map_err(io::Error::other)?;

	Ok(rows > 0)
}

/// Get anomalies for a file.
pub fn get_anomalies_for_file(conn: &Connection, file_id: &str) -> io::Result<Vec<(String, i32)>> {
	let mut stmt = conn
		.prepare("SELECT subject_hash, risk_score FROM anomaly_scores WHERE file_id = ?1")
		.map_err(io::Error::other)?;

	let anomalies = stmt
		.query_map(rusqlite::params![file_id], |row| {
			Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
		})
		.map_err(io::Error::other)?
		.collect::<Result<Vec<_>, _>>()
		.map_err(io::Error::other)?;

	Ok(anomalies)
}

/// Get high-risk anomalies above threshold.
pub fn get_high_risk_anomalies(
	conn: &Connection,
	threshold: i32,
) -> io::Result<Vec<(String, String, i32)>> {
	let mut stmt = conn
		.prepare(
			"SELECT subject_hash, anomaly_type, risk_score FROM anomaly_scores WHERE risk_score > ?1",
		)
		.map_err(io::Error::other)?;

	let high_risk = stmt
		.query_map(rusqlite::params![threshold], |row| {
			Ok((
				row.get::<_, String>(0)?,
				row.get::<_, String>(1)?,
				row.get::<_, i32>(2)?,
			))
		})
		.map_err(io::Error::other)?
		.collect::<Result<Vec<_>, _>>()
		.map_err(io::Error::other)?;

	Ok(high_risk)
}
