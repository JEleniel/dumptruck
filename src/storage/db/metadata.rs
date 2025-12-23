//! File metadata and processing tracking.

use std::io;
use rusqlite::Connection;

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
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(rows > 0)
}

/// Insert chain of custody record.
pub fn insert_custody_record(
	conn: &Connection,
	file_id: &str,
	record_id: &str,
	custody_action: &str,
	operator: &str,
	file_hash: &str,
	signature: &[u8],
	public_key: &[u8],
) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO chain_of_custody \
			 (file_id, record_id, custody_action, operator, file_hash, signature, public_key) \
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
			rusqlite::params![
				file_id,
				record_id,
				custody_action,
				operator,
				file_hash,
				signature,
				public_key,
			],
		)
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(rows > 0)
}

/// Get anomalies for a file.
pub fn get_anomalies_for_file(
	conn: &Connection,
	file_id: &str,
) -> io::Result<Vec<(String, i32)>> {
	let mut stmt = conn
		.prepare("SELECT subject_hash, risk_score FROM anomaly_scores WHERE file_id = ?1")
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let anomalies = stmt
		.query_map(rusqlite::params![file_id], |row| {
			Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
		})
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(anomalies)
}

/// Get high-risk anomalies above threshold.
pub fn get_high_risk_anomalies(
	conn: &Connection,
	threshold: i32,
) -> io::Result<Vec<(String, String, i32)>> {
	let mut stmt = conn
		.prepare("SELECT subject_hash, anomaly_type, risk_score FROM anomaly_scores WHERE risk_score > ?1")
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let high_risk = stmt
		.query_map(rusqlite::params![threshold], |row| {
			Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i32>(2)?))
		})
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(high_risk)
}
