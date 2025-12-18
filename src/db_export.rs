//! Database export functionality for backup and deduplication.
//!
//! Exports all PostgreSQL tables to a structured JSON format that preserves:
//! - All canonical addresses and their relationships
//! - Complete audit trail via chain of custody
//! - Alias mappings and confidence scores
//! - Anomaly scores and risk assessments
//! - File metadata and integrity hashes
//!
//! The export format supports round-trip import with deduplication.

use std::io;

use postgres::Client;
use serde::{Deserialize, Serialize};

/// Comprehensive database export structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseExport {
	/// Schema version for backward compatibility
	pub version: String,
	/// Export timestamp
	pub exported_at: String,
	/// All canonical addresses in the database
	pub canonical_addresses: Vec<CanonicalAddressRecord>,
	/// Address variant relationships
	pub address_alternates: Vec<AddressAlternateRecord>,
	/// Credentials associated with addresses
	pub address_credentials: Vec<AddressCredentialRecord>,
	/// Co-occurrence graph edges
	pub address_cooccurrence: Vec<CooccurrenceRecord>,
	/// Breach/HIBP data
	pub address_breaches: Vec<BreachRecord>,
	/// File metadata from Stage 1
	pub file_metadata: Vec<FileMetadataRecord>,
	/// Chain of custody from Stage 4
	pub chain_of_custody_records: Vec<ChainOfCustodyRecord>,
	/// Alias relationships from Stage 8
	pub alias_relationships: Vec<AliasRelationshipRecord>,
	/// Anomaly scores from Stage 10
	pub anomaly_scores: Vec<AnomalyScoreRecord>,
	/// Normalized rows for bulk import
	pub normalized_rows: Vec<NormalizedRowRecord>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CanonicalAddressRecord {
	pub canonical_hash: String,
	pub address_text: String,
	pub normalized_form: String,
	pub embedding: Option<Vec<f32>>,
	pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddressAlternateRecord {
	pub canonical_hash: String,
	pub alternate_hash: String,
	pub alternate_form: String,
	pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddressCredentialRecord {
	pub canonical_hash: String,
	pub credential_hash: String,
	pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CooccurrenceRecord {
	pub canonical_hash_1: String,
	pub canonical_hash_2: String,
	pub cooccurrence_count: i32,
	pub first_seen: String,
	pub last_seen: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BreachRecord {
	pub canonical_hash: String,
	pub breach_name: String,
	pub breach_date: Option<String>,
	pub pwned_count: Option<i32>,
	pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadataRecord {
	pub file_id: String,
	pub original_filename: String,
	pub sha256_hash: String,
	pub blake3_hash: Option<String>,
	pub file_size: i64,
	pub alternate_names: Option<serde_json::Value>,
	pub processing_status: String,
	pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainOfCustodyRecord {
	pub file_id: String,
	pub record_id: String,
	pub custody_action: String,
	pub operator: String,
	pub file_hash: String,
	pub signature: Vec<u8>,
	pub public_key: Vec<u8>,
	pub record_count: i32,
	pub notes: Option<String>,
	pub action_timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AliasRelationshipRecord {
	pub canonical_value: String,
	pub canonical_hash: String,
	pub variant_value: String,
	pub variant_hash: String,
	pub alias_type: String,
	pub confidence: i32,
	pub metadata: Option<serde_json::Value>,
	pub discovered_at: String,
	pub verified_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnomalyScoreRecord {
	pub file_id: String,
	pub subject_hash: String,
	pub anomaly_type: String,
	pub risk_score: i32,
	pub details: Option<serde_json::Value>,
	pub detected_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NormalizedRowRecord {
	pub id: i64,
	pub dataset: Option<String>,
	pub event_type: Option<String>,
	pub address_hash: Option<String>,
	pub credential_hash: Option<String>,
	pub file_id: Option<String>,
	pub fields: Option<serde_json::Value>,
}

/// Export the entire database to a structured JSON format.
pub fn export_database(client: &mut Client) -> io::Result<DatabaseExport> {
	let now = chrono::Utc::now().to_rfc3339();

	// Export canonical addresses
	let canonical_addresses = export_canonical_addresses(client)?;

	// Export address alternates
	let address_alternates = export_address_alternates(client)?;

	// Export address credentials
	let address_credentials = export_address_credentials(client)?;

	// Export co-occurrence edges
	let address_cooccurrence = export_address_cooccurrence(client)?;

	// Export breach records
	let address_breaches = export_address_breaches(client)?;

	// Export file metadata
	let file_metadata = export_file_metadata(client)?;

	// Export chain of custody
	let chain_of_custody_records = export_chain_of_custody(client)?;

	// Export alias relationships
	let alias_relationships = export_alias_relationships(client)?;

	// Export anomaly scores
	let anomaly_scores = export_anomaly_scores(client)?;

	// Export normalized rows
	let normalized_rows = export_normalized_rows(client)?;

	Ok(DatabaseExport {
		version: "1.0.0".to_string(),
		exported_at: now,
		canonical_addresses,
		address_alternates,
		address_credentials,
		address_cooccurrence,
		address_breaches,
		file_metadata,
		chain_of_custody_records,
		alias_relationships,
		anomaly_scores,
		normalized_rows,
	})
}

fn export_canonical_addresses(client: &mut Client) -> io::Result<Vec<CanonicalAddressRecord>> {
	let query = "SELECT canonical_hash, address_text, normalized_form, embedding, created_at FROM canonical_addresses ORDER BY created_at";
	let rows = client
		.query(query, &[])
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let mut records = Vec::new();
	for row in rows {
		let embedding: Option<Vec<f32>> = row.get(3);
		records.push(CanonicalAddressRecord {
			canonical_hash: row.get(0),
			address_text: row.get(1),
			normalized_form: row.get(2),
			embedding,
			created_at: row.get::<_, chrono::DateTime<chrono::Utc>>(4).to_rfc3339(),
		});
	}
	Ok(records)
}

fn export_address_alternates(client: &mut Client) -> io::Result<Vec<AddressAlternateRecord>> {
	let query = "SELECT canonical_hash, alternate_hash, alternate_form, created_at FROM address_alternates ORDER BY created_at";
	let rows = client
		.query(query, &[])
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let mut records = Vec::new();
	for row in rows {
		records.push(AddressAlternateRecord {
			canonical_hash: row.get(0),
			alternate_hash: row.get(1),
			alternate_form: row.get(2),
			created_at: row.get::<_, chrono::DateTime<chrono::Utc>>(3).to_rfc3339(),
		});
	}
	Ok(records)
}

fn export_address_credentials(client: &mut Client) -> io::Result<Vec<AddressCredentialRecord>> {
	let query = "SELECT DISTINCT canonical_hash, credential_hash, MAX(created_at) as created_at FROM address_credentials GROUP BY canonical_hash, credential_hash ORDER BY created_at";
	let rows = client
		.query(query, &[])
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let mut records = Vec::new();
	for row in rows {
		records.push(AddressCredentialRecord {
			canonical_hash: row.get(0),
			credential_hash: row.get(1),
			created_at: row.get::<_, chrono::DateTime<chrono::Utc>>(2).to_rfc3339(),
		});
	}
	Ok(records)
}

fn export_address_cooccurrence(client: &mut Client) -> io::Result<Vec<CooccurrenceRecord>> {
	let query = "SELECT canonical_hash_1, canonical_hash_2, cooccurrence_count, first_seen, last_seen FROM address_cooccurrence ORDER BY first_seen";
	let rows = client
		.query(query, &[])
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let mut records = Vec::new();
	for row in rows {
		records.push(CooccurrenceRecord {
			canonical_hash_1: row.get(0),
			canonical_hash_2: row.get(1),
			cooccurrence_count: row.get(2),
			first_seen: row.get::<_, chrono::DateTime<chrono::Utc>>(3).to_rfc3339(),
			last_seen: row.get::<_, chrono::DateTime<chrono::Utc>>(4).to_rfc3339(),
		});
	}
	Ok(records)
}

fn export_address_breaches(client: &mut Client) -> io::Result<Vec<BreachRecord>> {
	let query = "SELECT canonical_hash, breach_name, breach_date, pwned_count, created_at FROM address_breaches ORDER BY created_at";
	let rows = client
		.query(query, &[])
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let mut records = Vec::new();
	for row in rows {
		let breach_date: Option<chrono::DateTime<chrono::Utc>> = row.get(2);
		records.push(BreachRecord {
			canonical_hash: row.get(0),
			breach_name: row.get(1),
			breach_date: breach_date.map(|d| d.to_rfc3339()),
			pwned_count: row.get(3),
			created_at: row.get::<_, chrono::DateTime<chrono::Utc>>(4).to_rfc3339(),
		});
	}
	Ok(records)
}

fn export_file_metadata(client: &mut Client) -> io::Result<Vec<FileMetadataRecord>> {
	let query = "SELECT file_id, original_filename, sha256_hash, blake3_hash, file_size, alternate_names, processing_status, created_at FROM file_metadata ORDER BY created_at";
	let rows = client
		.query(query, &[])
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let mut records = Vec::new();
	for row in rows {
		records.push(FileMetadataRecord {
			file_id: row.get(0),
			original_filename: row.get(1),
			sha256_hash: row.get(2),
			blake3_hash: row.get(3),
			file_size: row.get(4),
			alternate_names: row.get(5),
			processing_status: row.get(6),
			created_at: row.get::<_, chrono::DateTime<chrono::Utc>>(7).to_rfc3339(),
		});
	}
	Ok(records)
}

fn export_chain_of_custody(client: &mut Client) -> io::Result<Vec<ChainOfCustodyRecord>> {
	let query = "SELECT file_id, record_id, custody_action, operator, file_hash, signature, public_key, record_count, notes, action_timestamp FROM chain_of_custody_records ORDER BY action_timestamp";
	let rows = client
		.query(query, &[])
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let mut records = Vec::new();
	for row in rows {
		let notes: Option<String> = row.get(8);
		records.push(ChainOfCustodyRecord {
			file_id: row.get(0),
			record_id: row.get(1),
			custody_action: row.get(2),
			operator: row.get(3),
			file_hash: row.get(4),
			signature: row.get(5),
			public_key: row.get(6),
			record_count: row.get(7),
			notes,
			action_timestamp: row.get::<_, chrono::DateTime<chrono::Utc>>(9).to_rfc3339(),
		});
	}
	Ok(records)
}

fn export_alias_relationships(client: &mut Client) -> io::Result<Vec<AliasRelationshipRecord>> {
	let query = "SELECT canonical_value, canonical_hash, variant_value, variant_hash, alias_type, confidence, metadata, discovered_at, verified_at FROM alias_relationships ORDER BY discovered_at";
	let rows = client
		.query(query, &[])
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let mut records = Vec::new();
	for row in rows {
		let metadata: Option<serde_json::Value> = row.get(6);
		records.push(AliasRelationshipRecord {
			canonical_value: row.get(0),
			canonical_hash: row.get(1),
			variant_value: row.get(2),
			variant_hash: row.get(3),
			alias_type: row.get(4),
			confidence: row.get(5),
			metadata,
			discovered_at: row.get::<_, chrono::DateTime<chrono::Utc>>(7).to_rfc3339(),
			verified_at: row.get::<_, chrono::DateTime<chrono::Utc>>(8).to_rfc3339(),
		});
	}
	Ok(records)
}

fn export_anomaly_scores(client: &mut Client) -> io::Result<Vec<AnomalyScoreRecord>> {
	let query = "SELECT file_id, subject_hash, anomaly_type, risk_score, details, detected_at FROM anomaly_scores ORDER BY detected_at";
	let rows = client
		.query(query, &[])
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let mut records = Vec::new();
	for row in rows {
		let details: Option<serde_json::Value> = row.get(4);
		records.push(AnomalyScoreRecord {
			file_id: row.get(0),
			subject_hash: row.get(1),
			anomaly_type: row.get(2),
			risk_score: row.get(3),
			details,
			detected_at: row.get::<_, chrono::DateTime<chrono::Utc>>(5).to_rfc3339(),
		});
	}
	Ok(records)
}

fn export_normalized_rows(client: &mut Client) -> io::Result<Vec<NormalizedRowRecord>> {
	let query = "SELECT id, dataset, event_type, address_hash, credential_hash, file_id, fields FROM normalized_rows ORDER BY id";
	let rows = client
		.query(query, &[])
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let mut records = Vec::new();
	for row in rows {
		let fields: Option<serde_json::Value> = row.get(6);
		records.push(NormalizedRowRecord {
			id: row.get(0),
			dataset: row.get(1),
			event_type: row.get(2),
			address_hash: row.get(3),
			credential_hash: row.get(4),
			file_id: row.get(5),
			fields,
		});
	}
	Ok(records)
}
