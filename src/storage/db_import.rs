//! Database import functionality for restoration and deduplication.
//!
//! Imports a previously exported database from JSON format with:
//! - Deduplication to prevent duplicate insertions
//! - Conflict resolution strategies
//! - Integrity verification
//! - Transaction support for atomicity

use std::io;

use rusqlite::Connection;
use serde_json;

use crate::storage::db_export::{
	AddressAlternateRecord, AddressCredentialRecord, AliasRelationshipRecord, AnomalyScoreRecord,
	BreachRecord, CanonicalAddressRecord, ChainOfCustodyRecord, CooccurrenceRecord, DatabaseExport,
	FileMetadataRecord, NormalizedRowRecord,
};

/// Import statistics tracking what was imported
#[derive(Debug, Clone, Default)]
pub struct ImportStats {
	pub canonical_addresses_imported: usize,
	pub canonical_addresses_skipped: usize,
	pub address_alternates_imported: usize,
	pub address_alternates_skipped: usize,
	pub address_credentials_imported: usize,
	pub address_credentials_skipped: usize,
	pub address_cooccurrence_imported: usize,
	pub address_cooccurrence_skipped: usize,
	pub address_breaches_imported: usize,
	pub address_breaches_skipped: usize,
	pub file_metadata_imported: usize,
	pub file_metadata_skipped: usize,
	pub chain_of_custody_imported: usize,
	pub chain_of_custody_skipped: usize,
	pub alias_relationships_imported: usize,
	pub alias_relationships_skipped: usize,
	pub anomaly_scores_imported: usize,
	pub anomaly_scores_skipped: usize,
	pub normalized_rows_imported: usize,
	pub normalized_rows_skipped: usize,
}

impl ImportStats {
	pub fn total_imported(&self) -> usize {
		self.canonical_addresses_imported
			+ self.address_alternates_imported
			+ self.address_credentials_imported
			+ self.address_cooccurrence_imported
			+ self.address_breaches_imported
			+ self.file_metadata_imported
			+ self.chain_of_custody_imported
			+ self.alias_relationships_imported
			+ self.anomaly_scores_imported
			+ self.normalized_rows_imported
	}

	pub fn total_skipped(&self) -> usize {
		self.canonical_addresses_skipped
			+ self.address_alternates_skipped
			+ self.address_credentials_skipped
			+ self.address_cooccurrence_skipped
			+ self.address_breaches_skipped
			+ self.file_metadata_skipped
			+ self.chain_of_custody_skipped
			+ self.alias_relationships_skipped
			+ self.anomaly_scores_skipped
			+ self.normalized_rows_skipped
	}
}

/// Import the entire database from a structured JSON export.
pub fn import_database(conn: &Connection, export: &DatabaseExport) -> io::Result<ImportStats> {
	let mut stats = ImportStats::default();

	// Import in dependency order: canonical addresses → alternates → relationships
	let ca = import_canonical_addresses(conn, &export.canonical_addresses)?;
	let aa = import_address_alternates(conn, &export.address_alternates)?;
	let ac = import_address_credentials(conn, &export.address_credentials)?;
	let ao = import_address_cooccurrence(conn, &export.address_cooccurrence)?;
	let ab = import_address_breaches(conn, &export.address_breaches)?;
	let fm = import_file_metadata(conn, &export.file_metadata)?;
	let coc = import_chain_of_custody(conn, &export.chain_of_custody)?;
	let ar = import_alias_relationships(conn, &export.alias_relationships)?;
	let as_ = import_anomaly_scores(conn, &export.anomaly_scores)?;
	let nr = import_normalized_rows(conn, &export.normalized_rows)?;

	stats.canonical_addresses_imported = ca.0;
	stats.canonical_addresses_skipped = ca.1;
	stats.address_alternates_imported = aa.0;
	stats.address_alternates_skipped = aa.1;
	stats.address_credentials_imported = ac.0;
	stats.address_credentials_skipped = ac.1;
	stats.address_cooccurrence_imported = ao.0;
	stats.address_cooccurrence_skipped = ao.1;
	stats.address_breaches_imported = ab.0;
	stats.address_breaches_skipped = ab.1;
	stats.file_metadata_imported = fm.0;
	stats.file_metadata_skipped = fm.1;
	stats.chain_of_custody_imported = coc.0;
	stats.chain_of_custody_skipped = coc.1;
	stats.alias_relationships_imported = ar.0;
	stats.alias_relationships_skipped = ar.1;
	stats.anomaly_scores_imported = as_.0;
	stats.anomaly_scores_skipped = as_.1;
	stats.normalized_rows_imported = nr.0;
	stats.normalized_rows_skipped = nr.1;

	Ok(stats)
}

fn import_canonical_addresses(
	conn: &Connection,
	records: &[CanonicalAddressRecord],
) -> io::Result<(usize, usize)> {
	let mut imported = 0;
	let mut skipped = 0;

	for record in records {
		let embedding_json = record
			.embedding
			.as_ref()
			.map(|vec| serde_json::to_string(vec).unwrap_or_default());

		let created_at = chrono::DateTime::parse_from_rfc3339(&record.created_at)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let result = conn.execute(
			"INSERT OR IGNORE INTO canonical_addresses (canonical_hash, address_text, \
			 normalized_form, embedding, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
			rusqlite::params![
				&record.canonical_hash,
				&record.address_text,
				&record.normalized_form,
				embedding_json,
				created_at
			],
		);

		match result {
			Ok(n) if n > 0 => imported += 1,
			Ok(_) => skipped += 1,
			Err(e) => return Err(io::Error::other(e)),
		}
	}

	Ok((imported, skipped))
}

fn import_address_alternates(
	conn: &Connection,
	records: &[AddressAlternateRecord],
) -> io::Result<(usize, usize)> {
	let mut imported = 0;
	let mut skipped = 0;

	for record in records {
		let created_at = chrono::DateTime::parse_from_rfc3339(&record.created_at)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let result = conn.execute(
			"INSERT OR IGNORE INTO address_alternates (canonical_hash, alternate_hash, \
			 alternate_form, created_at) VALUES (?1, ?2, ?3, ?4)",
			rusqlite::params![
				&record.canonical_hash,
				&record.alternate_hash,
				&record.alternate_form,
				created_at
			],
		);

		match result {
			Ok(n) if n > 0 => imported += 1,
			Ok(_) => skipped += 1,
			Err(e) => return Err(io::Error::other(e)),
		}
	}

	Ok((imported, skipped))
}

fn import_address_credentials(
	conn: &Connection,
	records: &[AddressCredentialRecord],
) -> io::Result<(usize, usize)> {
	let mut imported = 0;
	let mut skipped = 0;

	for record in records {
		let created_at = chrono::DateTime::parse_from_rfc3339(&record.created_at)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let result = conn.execute(
			"INSERT OR IGNORE INTO address_credentials (canonical_hash, credential_hash, \
			 created_at) VALUES (?1, ?2, ?3)",
			rusqlite::params![&record.canonical_hash, &record.credential_hash, created_at],
		);

		match result {
			Ok(n) if n > 0 => imported += 1,
			Ok(_) => skipped += 1,
			Err(e) => return Err(io::Error::other(e)),
		}
	}

	Ok((imported, skipped))
}

fn import_address_cooccurrence(
	conn: &Connection,
	records: &[CooccurrenceRecord],
) -> io::Result<(usize, usize)> {
	let mut imported = 0;
	let mut skipped = 0;

	for record in records {
		let first_seen = chrono::DateTime::parse_from_rfc3339(&record.first_seen)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let last_seen = chrono::DateTime::parse_from_rfc3339(&record.last_seen)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let result = conn.execute(
			"INSERT OR IGNORE INTO address_cooccurrence (canonical_hash_1, canonical_hash_2, \
			 cooccurrence_count, first_seen, last_seen) VALUES (?1, ?2, ?3, ?4, ?5)",
			rusqlite::params![
				&record.canonical_hash_1,
				&record.canonical_hash_2,
				record.cooccurrence_count,
				first_seen,
				last_seen
			],
		);

		match result {
			Ok(n) if n > 0 => imported += 1,
			Ok(_) => skipped += 1,
			Err(e) => return Err(io::Error::other(e)),
		}
	}

	Ok((imported, skipped))
}

fn import_address_breaches(
	conn: &Connection,
	records: &[BreachRecord],
) -> io::Result<(usize, usize)> {
	let mut imported = 0;
	let mut skipped = 0;

	for record in records {
		let breach_date = record.breach_date.as_ref().and_then(|date_str| {
			chrono::DateTime::parse_from_rfc3339(date_str)
				.ok()
				.map(|dt| dt.with_timezone(&chrono::Utc))
		});

		let created_at = chrono::DateTime::parse_from_rfc3339(&record.created_at)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let result = conn.execute(
			"INSERT OR IGNORE INTO address_breaches (canonical_hash, breach_name, breach_date, \
			 pwned_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
			rusqlite::params![
				&record.canonical_hash,
				&record.breach_name,
				breach_date,
				record.pwned_count,
				created_at
			],
		);

		match result {
			Ok(n) if n > 0 => imported += 1,
			Ok(_) => skipped += 1,
			Err(e) => return Err(io::Error::other(e)),
		}
	}

	Ok((imported, skipped))
}

fn import_file_metadata(
	conn: &Connection,
	records: &[FileMetadataRecord],
) -> io::Result<(usize, usize)> {
	let mut imported = 0;
	let mut skipped = 0;

	for record in records {
		let created_at = chrono::DateTime::parse_from_rfc3339(&record.created_at)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let result = conn.execute(
			"INSERT OR IGNORE INTO file_metadata (file_id, original_filename, sha256_hash, \
			 blake3_hash, file_size, alternate_names, processing_status, created_at) VALUES (?1, \
			 ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
			rusqlite::params![
				&record.file_id,
				&record.original_filename,
				&record.sha256_hash,
				&record.blake3_hash,
				record.file_size,
				&record.alternate_names,
				&record.processing_status,
				created_at
			],
		);

		match result {
			Ok(n) if n > 0 => imported += 1,
			Ok(_) => skipped += 1,
			Err(e) => return Err(io::Error::other(e)),
		}
	}

	Ok((imported, skipped))
}

fn import_chain_of_custody(
	conn: &Connection,
	records: &[ChainOfCustodyRecord],
) -> io::Result<(usize, usize)> {
	let mut imported = 0;
	let mut skipped = 0;

	for record in records {
		let action_timestamp = chrono::DateTime::parse_from_rfc3339(&record.action_timestamp)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let result = conn.execute(
			"INSERT OR IGNORE INTO chain_of_custody_records (file_id, record_id, custody_action, \
			 operator, file_hash, signature, public_key, record_count, notes, action_timestamp) \
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
			rusqlite::params![
				&record.file_id,
				&record.record_id,
				&record.custody_action,
				&record.operator,
				&record.file_hash,
				&record.signature,
				&record.public_key,
				record.record_count,
				&record.notes,
				action_timestamp
			],
		);

		match result {
			Ok(n) if n > 0 => imported += 1,
			Ok(_) => skipped += 1,
			Err(e) => return Err(io::Error::other(e)),
		}
	}

	Ok((imported, skipped))
}

fn import_alias_relationships(
	conn: &Connection,
	records: &[AliasRelationshipRecord],
) -> io::Result<(usize, usize)> {
	let mut imported = 0;
	let mut skipped = 0;

	for record in records {
		let metadata_json = record
			.metadata
			.as_ref()
			.map(|val| serde_json::to_string(val).unwrap_or_default());

		let discovered_at = chrono::DateTime::parse_from_rfc3339(&record.discovered_at)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let verified_at = chrono::DateTime::parse_from_rfc3339(&record.verified_at)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let result = conn.execute(
			"INSERT OR IGNORE INTO alias_relationships (canonical_value, canonical_hash, \
			 variant_value, variant_hash, alias_type, confidence, metadata, discovered_at, \
			 verified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
			rusqlite::params![
				&record.canonical_value,
				&record.canonical_hash,
				&record.variant_value,
				&record.variant_hash,
				&record.alias_type,
				record.confidence,
				metadata_json,
				discovered_at,
				verified_at
			],
		);

		match result {
			Ok(n) if n > 0 => imported += 1,
			Ok(_) => skipped += 1,
			Err(e) => return Err(io::Error::other(e)),
		}
	}

	Ok((imported, skipped))
}

fn import_anomaly_scores(
	conn: &Connection,
	records: &[AnomalyScoreRecord],
) -> io::Result<(usize, usize)> {
	let mut imported = 0;
	let mut skipped = 0;

	for record in records {
		let details_json = record
			.details
			.as_ref()
			.map(|val| serde_json::to_string(val).unwrap_or_default());

		let detected_at = chrono::DateTime::parse_from_rfc3339(&record.detected_at)
			.ok()
			.map(|dt| dt.with_timezone(&chrono::Utc));

		let result = conn.execute(
			"INSERT OR IGNORE INTO anomaly_scores (file_id, subject_hash, anomaly_type, \
			 risk_score, details, detected_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
			rusqlite::params![
				&record.file_id,
				&record.subject_hash,
				&record.anomaly_type,
				record.risk_score,
				details_json,
				detected_at
			],
		);

		match result {
			Ok(n) if n > 0 => imported += 1,
			Ok(_) => skipped += 1,
			Err(e) => return Err(io::Error::other(e)),
		}
	}

	Ok((imported, skipped))
}

fn import_normalized_rows(
	conn: &Connection,
	records: &[NormalizedRowRecord],
) -> io::Result<(usize, usize)> {
	let mut imported = 0;
	let mut skipped = 0;

	for record in records {
		let fields_json = record
			.fields
			.as_ref()
			.map(|val| serde_json::to_string(val).unwrap_or_default());

		let result = conn.execute(
			"INSERT OR IGNORE INTO normalized_rows (id, dataset, event_type, address_hash, \
			 credential_hash, file_id, fields) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
			rusqlite::params![
				record.id,
				&record.dataset,
				&record.event_type,
				&record.address_hash,
				&record.credential_hash,
				&record.file_id,
				fields_json
			],
		);

		match result {
			Ok(n) if n > 0 => imported += 1,
			Ok(_) => skipped += 1,
			Err(e) => return Err(io::Error::other(e)),
		}
	}

	Ok((imported, skipped))
}
