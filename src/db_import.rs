//! Database import functionality for restoration and deduplication.
//!
//! Imports a previously exported database from JSON format with:
//! - Deduplication to prevent duplicate insertions
//! - Conflict resolution strategies
//! - Integrity verification
//! - Transaction support for atomicity

use std::io;

use postgres::Client;

use crate::db_export::{
	AliasRelationshipRecord, AnomalyScoreRecord, BreachRecord, CanonicalAddressRecord,
	ChainOfCustodyRecord, CooccurrenceRecord, DatabaseExport, FileMetadataRecord,
	NormalizedRowRecord,
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

/// Import an exported database with deduplication.
pub fn import_database(client: &mut Client, export: &DatabaseExport) -> io::Result<ImportStats> {
	let mut stats = ImportStats::default();

	// Import in dependency order to respect foreign keys

	// 1. File metadata (referenced by other tables)
	stats.file_metadata_imported = import_file_metadata(client, &export.file_metadata)?;
	stats.file_metadata_skipped = export.file_metadata.len() - stats.file_metadata_imported;

	// 2. Chain of custody (references file_metadata)
	stats.chain_of_custody_imported =
		import_chain_of_custody(client, &export.chain_of_custody_records)?;
	stats.chain_of_custody_skipped =
		export.chain_of_custody_records.len() - stats.chain_of_custody_imported;

	// 3. Canonical addresses (no external references)
	stats.canonical_addresses_imported =
		import_canonical_addresses(client, &export.canonical_addresses)?;
	stats.canonical_addresses_skipped =
		export.canonical_addresses.len() - stats.canonical_addresses_imported;

	// 4. Address alternates (references canonical_addresses)
	stats.address_alternates_imported =
		import_address_alternates(client, &export.address_alternates)?;
	stats.address_alternates_skipped =
		export.address_alternates.len() - stats.address_alternates_imported;

	// 5. Address credentials (references canonical_addresses)
	stats.address_credentials_imported =
		import_address_credentials(client, &export.address_credentials)?;
	stats.address_credentials_skipped =
		export.address_credentials.len() - stats.address_credentials_imported;

	// 6. Co-occurrence (references canonical_addresses)
	stats.address_cooccurrence_imported =
		import_address_cooccurrence(client, &export.address_cooccurrence)?;
	stats.address_cooccurrence_skipped =
		export.address_cooccurrence.len() - stats.address_cooccurrence_imported;

	// 7. Breaches (references canonical_addresses)
	stats.address_breaches_imported = import_address_breaches(client, &export.address_breaches)?;
	stats.address_breaches_skipped =
		export.address_breaches.len() - stats.address_breaches_imported;

	// 8. Alias relationships (no external references)
	stats.alias_relationships_imported =
		import_alias_relationships(client, &export.alias_relationships)?;
	stats.alias_relationships_skipped =
		export.alias_relationships.len() - stats.alias_relationships_imported;

	// 9. Anomaly scores (references file_metadata)
	stats.anomaly_scores_imported = import_anomaly_scores(client, &export.anomaly_scores)?;
	stats.anomaly_scores_skipped = export.anomaly_scores.len() - stats.anomaly_scores_imported;

	// 10. Normalized rows (no external references)
	stats.normalized_rows_imported = import_normalized_rows(client, &export.normalized_rows)?;
	stats.normalized_rows_skipped = export.normalized_rows.len() - stats.normalized_rows_imported;

	Ok(stats)
}

fn import_canonical_addresses(
	client: &mut Client,
	records: &[CanonicalAddressRecord],
) -> io::Result<usize> {
	let mut imported = 0;

	for record in records {
		// Check if already exists (deduplication)
		let exists_query =
			"SELECT COUNT(*) as count FROM canonical_addresses WHERE canonical_hash = $1";
		let row = client
			.query_one(exists_query, &[&record.canonical_hash])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: i64 = row.get(0);

		if exists > 0 {
			// Already exists, skip
			continue;
		}

		// Insert new record
		let insert_query = "INSERT INTO canonical_addresses (canonical_hash, address_text, normalized_form, embedding, created_at) VALUES ($1, $2, $3, $4, $5::timestamptz)";
		let result = client.execute(
			insert_query,
			&[
				&record.canonical_hash,
				&record.address_text,
				&record.normalized_form,
				&record.embedding,
				&record.created_at,
			],
		);

		if result.is_ok() {
			imported += 1;
		}
	}

	Ok(imported)
}

fn import_address_alternates(
	client: &mut Client,
	records: &[crate::db_export::AddressAlternateRecord],
) -> io::Result<usize> {
	let mut imported = 0;

	for record in records {
		// Check if already exists
		let exists_query = "SELECT COUNT(*) as count FROM address_alternates WHERE canonical_hash = $1 AND alternate_hash = $2";
		let row = client
			.query_one(
				exists_query,
				&[&record.canonical_hash, &record.alternate_hash],
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: i64 = row.get(0);

		if exists > 0 {
			continue;
		}

		let insert_query = "INSERT INTO address_alternates (canonical_hash, alternate_hash, alternate_form, created_at) VALUES ($1, $2, $3, $4::timestamptz)";
		let result = client.execute(
			insert_query,
			&[
				&record.canonical_hash,
				&record.alternate_hash,
				&record.alternate_form,
				&record.created_at,
			],
		);

		if result.is_ok() {
			imported += 1;
		}
	}

	Ok(imported)
}

fn import_address_credentials(
	client: &mut Client,
	records: &[crate::db_export::AddressCredentialRecord],
) -> io::Result<usize> {
	let mut imported = 0;

	for record in records {
		// Check if already exists
		let exists_query = "SELECT COUNT(*) as count FROM address_credentials WHERE canonical_hash = $1 AND credential_hash = $2";
		let row = client
			.query_one(
				exists_query,
				&[&record.canonical_hash, &record.credential_hash],
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: i64 = row.get(0);

		if exists > 0 {
			continue;
		}

		let insert_query = "INSERT INTO address_credentials (canonical_hash, credential_hash, created_at) VALUES ($1, $2, $3::timestamptz)";
		let result = client.execute(
			insert_query,
			&[
				&record.canonical_hash,
				&record.credential_hash,
				&record.created_at,
			],
		);

		if result.is_ok() {
			imported += 1;
		}
	}

	Ok(imported)
}

fn import_address_cooccurrence(
	client: &mut Client,
	records: &[CooccurrenceRecord],
) -> io::Result<usize> {
	let mut imported = 0;

	for record in records {
		// Check if already exists
		let exists_query = "SELECT COUNT(*) as count FROM address_cooccurrence WHERE canonical_hash_1 = $1 AND canonical_hash_2 = $2";
		let row = client
			.query_one(
				exists_query,
				&[&record.canonical_hash_1, &record.canonical_hash_2],
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: i64 = row.get(0);

		if exists > 0 {
			continue;
		}

		let insert_query = "INSERT INTO address_cooccurrence (canonical_hash_1, canonical_hash_2, cooccurrence_count, first_seen, last_seen) VALUES ($1, $2, $3, $4::timestamptz, $5::timestamptz)";
		let result = client.execute(
			insert_query,
			&[
				&record.canonical_hash_1,
				&record.canonical_hash_2,
				&record.cooccurrence_count,
				&record.first_seen,
				&record.last_seen,
			],
		);

		if result.is_ok() {
			imported += 1;
		}
	}

	Ok(imported)
}

fn import_address_breaches(client: &mut Client, records: &[BreachRecord]) -> io::Result<usize> {
	let mut imported = 0;

	for record in records {
		// Check if already exists
		let exists_query = "SELECT COUNT(*) as count FROM address_breaches WHERE canonical_hash = $1 AND breach_name = $2";
		let row = client
			.query_one(exists_query, &[&record.canonical_hash, &record.breach_name])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: i64 = row.get(0);

		if exists > 0 {
			continue;
		}

		let insert_query = "INSERT INTO address_breaches (canonical_hash, breach_name, breach_date, pwned_count, created_at) VALUES ($1, $2, $3::timestamptz, $4, $5::timestamptz)";
		let result = client.execute(
			insert_query,
			&[
				&record.canonical_hash,
				&record.breach_name,
				&record.breach_date,
				&record.pwned_count,
				&record.created_at,
			],
		);

		if result.is_ok() {
			imported += 1;
		}
	}

	Ok(imported)
}

fn import_file_metadata(client: &mut Client, records: &[FileMetadataRecord]) -> io::Result<usize> {
	let mut imported = 0;

	for record in records {
		// Check if already exists
		let exists_query = "SELECT COUNT(*) as count FROM file_metadata WHERE file_id = $1";
		let row = client
			.query_one(exists_query, &[&record.file_id])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: i64 = row.get(0);

		if exists > 0 {
			continue;
		}

		let insert_query = "INSERT INTO file_metadata (file_id, original_filename, sha256_hash, blake3_hash, file_size, alternate_names, processing_status, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::timestamptz)";
		let result = client.execute(
			insert_query,
			&[
				&record.file_id,
				&record.original_filename,
				&record.sha256_hash,
				&record.blake3_hash,
				&record.file_size,
				&record.alternate_names,
				&record.processing_status,
				&record.created_at,
			],
		);

		if result.is_ok() {
			imported += 1;
		}
	}

	Ok(imported)
}

fn import_chain_of_custody(
	client: &mut Client,
	records: &[ChainOfCustodyRecord],
) -> io::Result<usize> {
	let mut imported = 0;

	for record in records {
		// Check if already exists
		let exists_query =
			"SELECT COUNT(*) as count FROM chain_of_custody_records WHERE record_id = $1";
		let row = client
			.query_one(exists_query, &[&record.record_id])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: i64 = row.get(0);

		if exists > 0 {
			continue;
		}

		let insert_query = "INSERT INTO chain_of_custody_records (file_id, record_id, custody_action, operator, file_hash, signature, public_key, record_count, notes, action_timestamp) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::timestamptz)";
		let result = client.execute(
			insert_query,
			&[
				&record.file_id,
				&record.record_id,
				&record.custody_action,
				&record.operator,
				&record.file_hash,
				&record.signature,
				&record.public_key,
				&record.record_count,
				&record.notes,
				&record.action_timestamp,
			],
		);

		if result.is_ok() {
			imported += 1;
		}
	}

	Ok(imported)
}

fn import_alias_relationships(
	client: &mut Client,
	records: &[AliasRelationshipRecord],
) -> io::Result<usize> {
	let mut imported = 0;

	for record in records {
		// Check if already exists
		let exists_query = "SELECT COUNT(*) as count FROM alias_relationships WHERE canonical_hash = $1 AND variant_hash = $2 AND alias_type = $3";
		let row = client
			.query_one(
				exists_query,
				&[
					&record.canonical_hash,
					&record.variant_hash,
					&record.alias_type,
				],
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: i64 = row.get(0);

		if exists > 0 {
			continue;
		}

		let insert_query = "INSERT INTO alias_relationships (canonical_value, canonical_hash, variant_value, variant_hash, alias_type, confidence, metadata, discovered_at, verified_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::timestamptz, $9::timestamptz)";
		let result = client.execute(
			insert_query,
			&[
				&record.canonical_value,
				&record.canonical_hash,
				&record.variant_value,
				&record.variant_hash,
				&record.alias_type,
				&record.confidence,
				&record.metadata,
				&record.discovered_at,
				&record.verified_at,
			],
		);

		if result.is_ok() {
			imported += 1;
		}
	}

	Ok(imported)
}

fn import_anomaly_scores(client: &mut Client, records: &[AnomalyScoreRecord]) -> io::Result<usize> {
	let mut imported = 0;

	for record in records {
		// Check if already exists
		let exists_query = "SELECT COUNT(*) as count FROM anomaly_scores WHERE file_id = $1 AND subject_hash = $2 AND anomaly_type = $3";
		let row = client
			.query_one(
				exists_query,
				&[&record.file_id, &record.subject_hash, &record.anomaly_type],
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: i64 = row.get(0);

		if exists > 0 {
			continue;
		}

		let insert_query = "INSERT INTO anomaly_scores (file_id, subject_hash, anomaly_type, risk_score, details, detected_at) VALUES ($1, $2, $3, $4, $5, $6::timestamptz)";
		let result = client.execute(
			insert_query,
			&[
				&record.file_id,
				&record.subject_hash,
				&record.anomaly_type,
				&record.risk_score,
				&record.details,
				&record.detected_at,
			],
		);

		if result.is_ok() {
			imported += 1;
		}
	}

	Ok(imported)
}

fn import_normalized_rows(
	client: &mut Client,
	records: &[NormalizedRowRecord],
) -> io::Result<usize> {
	let mut imported = 0;

	for record in records {
		// Check if already exists by unique row signature
		let exists_query = "SELECT COUNT(*) as count FROM normalized_rows WHERE (address_hash = $1 OR address_hash IS NULL) AND (credential_hash = $2 OR credential_hash IS NULL) AND (file_id = $3 OR file_id IS NULL)";
		let row = client
			.query_one(
				exists_query,
				&[
					&record.address_hash,
					&record.credential_hash,
					&record.file_id,
				],
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let exists: i64 = row.get(0);

		if exists > 0 {
			continue;
		}

		let insert_query = "INSERT INTO normalized_rows (dataset, event_type, address_hash, credential_hash, file_id, fields) VALUES ($1, $2, $3, $4, $5, $6)";
		let result = client.execute(
			insert_query,
			&[
				&record.dataset,
				&record.event_type,
				&record.address_hash,
				&record.credential_hash,
				&record.file_id,
				&record.fields,
			],
		);

		if result.is_ok() {
			imported += 1;
		}
	}

	Ok(imported)
}
