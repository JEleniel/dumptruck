//! Storage adapter for filesystem and SQLite backends.
//!
//! Provides trait-based storage interface with implementations for both
//! filesystem CSV storage and structured SQLite database storage.

mod addresses;
mod aliases;
mod breaches;
mod metadata;
mod rows;
mod schema;
mod similarity;

pub use addresses::{
	get_credentials_for_address, insert_address_alternate, insert_address_credential_canonical,
	insert_canonical_address, lookup_canonical_by_alternate,
};
pub use aliases::{get_alias_relationships, insert_alias_relationship};
pub use breaches::{get_address_neighbors, insert_address_breach, record_address_cooccurrence};
pub use metadata::{
	get_anomalies_for_file, get_high_risk_anomalies, insert_anomaly_score, insert_custody_record,
	insert_file_metadata,
};
pub use schema::create_schema;
pub use similarity::{
	cosine_similarity, find_duplicate_address, find_similar_addresses, update_address_embedding,
};

use std::{
	fs::{File, OpenOptions},
	io::{self, BufRead, BufReader, Read, Write},
	path::PathBuf,
};

use rusqlite::Connection;

/// Trait for storage backends used in examples.
pub trait StorageAdapter {
	fn store_row(&mut self, row: &[String]) -> std::io::Result<()>;
	fn contains_hash(&mut self, hash: &str) -> std::io::Result<bool>;
	/// Check whether an address (by its hash) has been seen before.
	fn address_exists(&mut self, addr_hash: &str) -> std::io::Result<bool>;
	/// Check whether a specific credential hash is associated with an address.
	fn address_has_credential(&mut self, addr_hash: &str, cred_hash: &str)
	-> std::io::Result<bool>;
	/// Record an address -> credential mapping in the history store.
	fn add_address_credential(&mut self, addr_hash: &str, cred_hash: &str) -> std::io::Result<()>;

	// Canonical address tracking (production schema only; default no-op for filesystem):

	/// Store a canonical email address (SHA256 hash of normalized form).
	fn insert_canonical_address(
		&mut self,
		canonical_hash: &str,
		address_text: &str,
		normalized_form: &str,
	) -> std::io::Result<bool> {
		let _ = (canonical_hash, address_text, normalized_form);
		Ok(false)
	}

	/// Record a Unicode alternate representation mapping to a canonical address.
	fn insert_address_alternate(
		&mut self,
		canonical_hash: &str,
		alternate_hash: &str,
		alternate_form: &str,
	) -> std::io::Result<bool> {
		let _ = (canonical_hash, alternate_hash, alternate_form);
		Ok(false)
	}

	/// Look up a canonical address by an alternate Unicode hash.
	fn lookup_canonical_by_alternate(
		&mut self,
		alternate_hash: &str,
	) -> std::io::Result<Option<String>> {
		let _ = alternate_hash;
		Ok(None)
	}

	/// Record a credential associated with a canonical address.
	fn insert_address_credential_canonical(
		&mut self,
		canonical_hash: &str,
		credential_hash: &str,
	) -> std::io::Result<bool> {
		let _ = (canonical_hash, credential_hash);
		Ok(false)
	}

	/// Get all credentials associated with a canonical address.
	fn get_credentials_for_address(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<String>> {
		let _ = canonical_hash;
		Ok(vec![])
	}

	/// Record a co-occurrence edge between two canonical addresses.
	/// Addresses should be in canonical order (hash_1 < hash_2) to avoid duplicates.
	/// Returns Ok(true) if newly inserted, Ok(false) if already existed (count incremented).
	fn record_address_cooccurrence(
		&mut self,
		canonical_hash_1: &str,
		canonical_hash_2: &str,
	) -> std::io::Result<bool> {
		let _ = (canonical_hash_1, canonical_hash_2);
		Ok(false)
	}

	/// Get all neighbors of a canonical address in the co-occurrence graph.
	/// Returns a vector of (neighbor_canonical_hash, cooccurrence_count) tuples.
	fn get_address_neighbors(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<(String, i32)>> {
		let _ = canonical_hash;
		Ok(vec![])
	}

	/// Update the vector embedding for a canonical address (from Nomic/Ollama).
	/// Embeddings are used for similarity-based near-duplicate detection.
	fn update_address_embedding(
		&mut self,
		canonical_hash: &str,
		embedding: &[f32],
	) -> std::io::Result<()> {
		let _ = (canonical_hash, embedding);
		Ok(())
	}

	/// Find similar canonical addresses by vector similarity (cosine distance).
	/// Returns up to `limit` results as (canonical_hash, similarity_score) tuples,
	/// sorted by descending similarity.
	fn find_similar_addresses(
		&mut self,
		embedding: &[f32],
		limit: usize,
		similarity_threshold: f32,
	) -> std::io::Result<Vec<(String, f32)>> {
		let _ = (embedding, limit, similarity_threshold);
		Ok(vec![])
	}

	/// Check if a canonical address is likely a duplicate of an existing one
	/// based on hash match OR vector similarity above threshold.
	/// Returns Some(canonical_hash) if duplicate found, None otherwise.
	fn find_duplicate_address(
		&mut self,
		canonical_hash: &str,
		embedding: Option<&[f32]>,
		similarity_threshold: f32,
	) -> std::io::Result<Option<String>> {
		let _ = (canonical_hash, embedding, similarity_threshold);
		Ok(None)
	}

	/// Store breach data for a canonical address from HIBP API.
	/// Returns Ok(true) if newly inserted, Ok(false) if already existed.
	fn insert_address_breach(
		&mut self,
		canonical_hash: &str,
		breach_name: &str,
		breach_title: Option<&str>,
		breach_domain: Option<&str>,
		breach_date: Option<&str>,
		pwn_count: Option<i32>,
		description: Option<&str>,
		is_verified: bool,
		is_fabricated: bool,
		is_sensitive: bool,
		is_retired: bool,
	) -> std::io::Result<bool> {
		let _ = (
			canonical_hash,
			breach_name,
			breach_title,
			breach_domain,
			breach_date,
			pwn_count,
			description,
			is_verified,
			is_fabricated,
			is_sensitive,
			is_retired,
		);
		Ok(false)
	}

	// ========== Stage 13: Storage Enhancement Methods ==========

	/// Insert file metadata (Stage 1: Evidence Preservation)
	/// Tracks file ID, hashes, size, and processing status
	fn insert_file_metadata(
		&mut self,
		file_id: &str,
		original_filename: &str,
		sha256_hash: &str,
		file_size: i64,
	) -> std::io::Result<bool> {
		let _ = (file_id, original_filename, sha256_hash, file_size);
		Ok(false)
	}

	/// Insert chain of custody record (Stage 4: Chain of Custody)
	/// Stores cryptographically signed audit trail entry
	fn insert_custody_record(
		&mut self,
		file_id: &str,
		record_id: &str,
		custody_action: &str,
		operator: &str,
		file_hash: &str,
		signature: &[u8],
		public_key: &[u8],
	) -> std::io::Result<bool> {
		let _ = (
			file_id,
			record_id,
			custody_action,
			operator,
			file_hash,
			signature,
			public_key,
		);
		Ok(false)
	}

	/// Insert alias relationship (Stage 8: Alias Resolution)
	/// Links related entries across multiple identity formats
	fn insert_alias_relationship(
		&mut self,
		canonical_hash: &str,
		variant_hash: &str,
		alias_type: &str,
		confidence: i32,
	) -> std::io::Result<bool> {
		let _ = (canonical_hash, variant_hash, alias_type, confidence);
		Ok(false)
	}

	/// Get all alias relationships for a canonical hash
	fn get_alias_relationships(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<(String, String, i32)>> {
		let _ = canonical_hash;
		Ok(vec![])
	}

	/// Insert anomaly score (Stage 10: Anomaly Detection)
	/// Records detected statistical anomalies and outliers
	fn insert_anomaly_score(
		&mut self,
		file_id: &str,
		subject_hash: &str,
		anomaly_type: &str,
		risk_score: i32,
	) -> std::io::Result<bool> {
		let _ = (file_id, subject_hash, anomaly_type, risk_score);
		Ok(false)
	}

	/// Get anomaly scores for a file
	fn get_anomalies_for_file(&mut self, file_id: &str) -> std::io::Result<Vec<(String, i32)>> {
		let _ = file_id;
		Ok(vec![])
	}

	/// Get high-risk anomalies (risk_score > threshold)
	fn get_high_risk_anomalies(
		&mut self,
		threshold: i32,
	) -> std::io::Result<Vec<(String, String, i32)>> {
		let _ = threshold;
		Ok(vec![])
	}
}

/// Filesystem-based storage that appends CSV lines to a file.
pub struct FsStorage {
	path: PathBuf,
	file: File,
}

/// SQLite-backed storage adapter. Uses a local SQLite database
/// with tables for canonical addresses, credentials, and relationships.
pub struct SqliteStorage {
	conn: Connection,
	dataset: Option<String>,
}

impl SqliteStorage {
	pub fn new(db_path: &str, dataset: Option<String>) -> std::io::Result<Self> {
		let conn =
			Connection::open(db_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

		// Create schema if not exists
		schema::create_schema(&conn)?;

		Ok(SqliteStorage { conn, dataset })
	}

	/// Create from environment or default to dumptruck.db
	pub fn new_from_env() -> std::io::Result<Self> {
		let path =
			std::env::var("DUMPTRUCK_DB_PATH").unwrap_or_else(|_| "./dumptruck.db".to_string());
		let dataset = std::env::var("DUMPTRUCK_DATASET").ok();
		SqliteStorage::new(&path, dataset)
	}
}

impl FsStorage {
	pub fn new(path: PathBuf) -> std::io::Result<Self> {
		let file = OpenOptions::new().create(true).append(true).open(&path)?;
		Ok(FsStorage { path, file })
	}

	pub fn read_all(path: &PathBuf) -> std::io::Result<String> {
		let mut s = String::new();
		let mut f = File::open(path)?;
		f.read_to_string(&mut s)?;
		Ok(s)
	}
}

impl StorageAdapter for FsStorage {
	fn store_row(&mut self, row: &[String]) -> std::io::Result<()> {
		// naive CSV escaping: wrap fields with quotes if they contain comma or quote
		let mut parts: Vec<String> = Vec::with_capacity(row.len());
		for f in row {
			if f.contains(',') || f.contains('"') || f.contains('\n') {
				let esc = f.replace('"', "\"\"");
				parts.push(format!("\"{}\"", esc));
			} else {
				parts.push(f.clone());
			}
		}
		let line = parts.join(",") + "\n";
		self.file.write_all(line.as_bytes())?;
		Ok(())
	}

	fn address_exists(&mut self, addr_hash: &str) -> std::io::Result<bool> {
		// look for an __address_hash__ line containing the addr_hash
		let f = File::open(&self.path)?;
		let mut reader = BufReader::new(f);
		let mut line = String::new();
		loop {
			line.clear();
			let bytes = reader.read_line(&mut line)?;
			if bytes == 0 {
				break;
			}
			if line.contains("__address_hash__") && line.contains(addr_hash) {
				return Ok(true);
			}
		}
		Ok(false)
	}

	fn address_has_credential(
		&mut self,
		addr_hash: &str,
		cred_hash: &str,
	) -> std::io::Result<bool> {
		// look for a mapping row we write as __addr_cred__,addr_hash,cred_hash
		let f = File::open(&self.path)?;
		let mut reader = BufReader::new(f);
		let mut line = String::new();
		loop {
			line.clear();
			let bytes = reader.read_line(&mut line)?;
			if bytes == 0 {
				break;
			}
			if line.contains("__addr_cred__")
				&& line.contains(addr_hash)
				&& line.contains(cred_hash)
			{
				return Ok(true);
			}
		}
		Ok(false)
	}

	fn add_address_credential(&mut self, addr_hash: &str, cred_hash: &str) -> std::io::Result<()> {
		let row = vec![
			"__addr_cred__".to_string(),
			addr_hash.to_string(),
			cred_hash.to_string(),
		];
		self.store_row(&row)
	}

	fn contains_hash(&mut self, hash: &str) -> std::io::Result<bool> {
		// Stream the file line-by-line so we never load the entire file into memory.
		// This is important when the storage file is larger than available RAM.
		let f = File::open(&self.path)?;
		let mut reader = BufReader::new(f);
		let mut line = String::new();
		loop {
			line.clear();
			let bytes = reader.read_line(&mut line)?;
			if bytes == 0 {
				break;
			}
			if line.contains(hash) {
				return Ok(true);
			}
		}
		Ok(false)
	}

	// ========== Stage 13: Storage Enhancement (FsStorage no-op implementations) ==========

	fn insert_file_metadata(
		&mut self,
		_file_id: &str,
		_original_filename: &str,
		_sha256_hash: &str,
		_file_size: i64,
	) -> std::io::Result<bool> {
		// Filesystem storage doesn't track metadata; return false (not inserted)
		Ok(false)
	}

	fn insert_custody_record(
		&mut self,
		_file_id: &str,
		_record_id: &str,
		_custody_action: &str,
		_operator: &str,
		_file_hash: &str,
		_signature: &[u8],
		_public_key: &[u8],
	) -> std::io::Result<bool> {
		// Filesystem storage doesn't track custody; return false (not inserted)
		Ok(false)
	}

	fn insert_alias_relationship(
		&mut self,
		_canonical_hash: &str,
		_variant_hash: &str,
		_alias_type: &str,
		_confidence: i32,
	) -> std::io::Result<bool> {
		// Filesystem storage doesn't track aliases; return false (not inserted)
		Ok(false)
	}

	fn get_alias_relationships(
		&mut self,
		_canonical_hash: &str,
	) -> std::io::Result<Vec<(String, String, i32)>> {
		// Filesystem storage doesn't track aliases; return empty
		Ok(vec![])
	}

	fn insert_anomaly_score(
		&mut self,
		_file_id: &str,
		_subject_hash: &str,
		_anomaly_type: &str,
		_risk_score: i32,
	) -> std::io::Result<bool> {
		// Filesystem storage doesn't track anomalies; return false (not inserted)
		Ok(false)
	}

	fn get_anomalies_for_file(&mut self, _file_id: &str) -> std::io::Result<Vec<(String, i32)>> {
		// Filesystem storage doesn't track anomalies; return empty
		Ok(vec![])
	}

	fn get_high_risk_anomalies(
		&mut self,
		_threshold: i32,
	) -> std::io::Result<Vec<(String, String, i32)>> {
		// Filesystem storage doesn't track anomalies; return empty
		Ok(vec![])
	}
}

impl StorageAdapter for SqliteStorage {
	fn store_row(&mut self, row: &[String]) -> std::io::Result<()> {
		let (event_type, address_hash, credential_hash, row_hash, file_id) =
			rows::parse_event_and_columns(row);
		let source_file = extract_source_file(row);
		let fields_text = rows::build_fields_json(row)?;

		rows::store_row(
			&self.conn,
			self.dataset.as_deref(),
			&event_type,
			&address_hash,
			&credential_hash,
			&row_hash,
			&file_id,
			&source_file,
			&fields_text,
		)
	}

	fn contains_hash(&mut self, hash: &str) -> std::io::Result<bool> {
		// Check dedicated columns first for speed, then fall back to fields text
		let mut stmt = self
			.conn
			.prepare(
				"SELECT COUNT(*) FROM normalized_rows WHERE credential_hash = ?1 OR address_hash \
				 = ?1",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let count: i64 = stmt
			.query_row(rusqlite::params![hash], |row| row.get(0))
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		if count > 0 {
			return Ok(true);
		}
		// fallback: search JSON text
		let pattern = format!("%{}%", hash);
		let mut stmt2 = self
			.conn
			.prepare("SELECT COUNT(*) FROM normalized_rows WHERE fields LIKE ?1")
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let count2: i64 = stmt2
			.query_row(rusqlite::params![&pattern], |row| row.get(0))
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(count2 > 0)
	}

	fn address_exists(&mut self, addr_hash: &str) -> std::io::Result<bool> {
		let mut stmt = self
			.conn
			.prepare("SELECT COUNT(*) FROM normalized_rows WHERE address_hash = ?1")
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let count: i64 = stmt
			.query_row(rusqlite::params![addr_hash], |row| row.get(0))
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(count > 0)
	}

	fn address_has_credential(
		&mut self,
		addr_hash: &str,
		cred_hash: &str,
	) -> std::io::Result<bool> {
		let mut stmt = self
			.conn
			.prepare(
				"SELECT COUNT(*) FROM normalized_rows WHERE address_hash = ?1 AND credential_hash \
				 = ?2",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let count: i64 = stmt
			.query_row(rusqlite::params![addr_hash, cred_hash], |row| row.get(0))
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(count > 0)
	}

	fn add_address_credential(&mut self, addr_hash: &str, cred_hash: &str) -> std::io::Result<()> {
		let dataset = self.dataset.as_deref();
		let fields = serde_json::Value::Array(vec![
			serde_json::Value::String("__addr_cred__".to_string()),
			serde_json::Value::String(addr_hash.to_string()),
			serde_json::Value::String(cred_hash.to_string()),
		]);
		let fields_text =
			serde_json::to_string(&fields).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		self.conn
			.execute(
				"INSERT INTO normalized_rows (dataset, event_type, address_hash, credential_hash, \
				 file_id, fields) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
				rusqlite::params![
					dataset,
					"__addr_cred__",
					addr_hash,
					cred_hash,
					None::<String>,
					&fields_text,
				],
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(())
	}

	fn insert_canonical_address(
		&mut self,
		canonical_hash: &str,
		address_text: &str,
		normalized_form: &str,
	) -> std::io::Result<bool> {
		addresses::insert_canonical_address(
			&self.conn,
			canonical_hash,
			address_text,
			normalized_form,
		)
	}

	fn insert_address_alternate(
		&mut self,
		canonical_hash: &str,
		alternate_hash: &str,
		alternate_form: &str,
	) -> std::io::Result<bool> {
		addresses::insert_address_alternate(
			&self.conn,
			canonical_hash,
			alternate_hash,
			alternate_form,
		)
	}

	fn lookup_canonical_by_alternate(
		&mut self,
		alternate_hash: &str,
	) -> std::io::Result<Option<String>> {
		addresses::lookup_canonical_by_alternate(&self.conn, alternate_hash)
	}

	fn insert_address_credential_canonical(
		&mut self,
		canonical_hash: &str,
		credential_hash: &str,
	) -> std::io::Result<bool> {
		addresses::insert_address_credential_canonical(&self.conn, canonical_hash, credential_hash)
	}

	fn get_credentials_for_address(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<String>> {
		addresses::get_credentials_for_address(&self.conn, canonical_hash)
	}

	fn record_address_cooccurrence(
		&mut self,
		canonical_hash_1: &str,
		canonical_hash_2: &str,
	) -> std::io::Result<bool> {
		// Ensure canonical order: smaller hash first
		let (h1, h2) = if canonical_hash_1 < canonical_hash_2 {
			(canonical_hash_1, canonical_hash_2)
		} else {
			(canonical_hash_2, canonical_hash_1)
		};
		breaches::record_address_cooccurrence(&self.conn, h1, h2)
	}

	fn get_address_neighbors(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<(String, i32)>> {
		// Get all neighbors from both directions (since the graph is undirected)
		let mut stmt = self
			.conn
			.prepare(
				"SELECT CASE WHEN canonical_hash_1 = ?1 THEN canonical_hash_2 ELSE \
				 canonical_hash_1 END as neighbor, cooccurrence_count FROM address_cooccurrence \
				 WHERE canonical_hash_1 = ?1 OR canonical_hash_2 = ?1 ORDER BY cooccurrence_count \
				 DESC",
			)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let neighbors = stmt
			.query_map(rusqlite::params![canonical_hash], |row| {
				Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
			})
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(neighbors)
	}

	fn update_address_embedding(
		&mut self,
		canonical_hash: &str,
		embedding: &[f32],
	) -> std::io::Result<()> {
		similarity::update_address_embedding(&self.conn, canonical_hash, embedding)
	}

	fn find_similar_addresses(
		&mut self,
		embedding: &[f32],
		limit: usize,
		similarity_threshold: f32,
	) -> std::io::Result<Vec<(String, f32)>> {
		similarity::find_similar_addresses(&self.conn, embedding, limit, similarity_threshold)
	}

	fn find_duplicate_address(
		&mut self,
		canonical_hash: &str,
		embedding: Option<&[f32]>,
		similarity_threshold: f32,
	) -> std::io::Result<Option<String>> {
		similarity::find_duplicate_address(
			&self.conn,
			canonical_hash,
			embedding,
			similarity_threshold,
		)
	}

	fn insert_address_breach(
		&mut self,
		canonical_hash: &str,
		breach_name: &str,
		breach_title: Option<&str>,
		breach_domain: Option<&str>,
		breach_date: Option<&str>,
		pwn_count: Option<i32>,
		description: Option<&str>,
		is_verified: bool,
		is_fabricated: bool,
		is_sensitive: bool,
		is_retired: bool,
	) -> std::io::Result<bool> {
		breaches::insert_address_breach(
			&self.conn,
			canonical_hash,
			breach_name,
			breach_title,
			breach_domain,
			breach_date,
			pwn_count,
			description,
			is_verified,
			is_fabricated,
			is_sensitive,
			is_retired,
		)
	}

	fn insert_file_metadata(
		&mut self,
		file_id: &str,
		original_filename: &str,
		sha256_hash: &str,
		file_size: i64,
	) -> std::io::Result<bool> {
		metadata::insert_file_metadata(
			&self.conn,
			file_id,
			original_filename,
			sha256_hash,
			file_size,
		)
	}

	fn insert_custody_record(
		&mut self,
		file_id: &str,
		record_id: &str,
		custody_action: &str,
		operator: &str,
		file_hash: &str,
		signature: &[u8],
		public_key: &[u8],
	) -> std::io::Result<bool> {
		metadata::insert_custody_record(
			&self.conn,
			file_id,
			record_id,
			custody_action,
			operator,
			file_hash,
			signature,
			public_key,
		)
	}

	fn insert_alias_relationship(
		&mut self,
		canonical_hash: &str,
		variant_hash: &str,
		alias_type: &str,
		confidence: i32,
	) -> std::io::Result<bool> {
		aliases::insert_alias_relationship(
			&self.conn,
			canonical_hash,
			variant_hash,
			alias_type,
			confidence,
		)
	}

	fn get_alias_relationships(
		&mut self,
		canonical_hash: &str,
	) -> std::io::Result<Vec<(String, String, i32)>> {
		aliases::get_alias_relationships(&self.conn, canonical_hash)
	}

	fn insert_anomaly_score(
		&mut self,
		file_id: &str,
		subject_hash: &str,
		anomaly_type: &str,
		risk_score: i32,
	) -> std::io::Result<bool> {
		metadata::insert_anomaly_score(&self.conn, file_id, subject_hash, anomaly_type, risk_score)
	}

	fn get_anomalies_for_file(&mut self, file_id: &str) -> std::io::Result<Vec<(String, i32)>> {
		metadata::get_anomalies_for_file(&self.conn, file_id)
	}

	fn get_high_risk_anomalies(
		&mut self,
		threshold: i32,
	) -> std::io::Result<Vec<(String, String, i32)>> {
		metadata::get_high_risk_anomalies(&self.conn, threshold)
	}
}

/// Extract source_file from row if present.
fn extract_source_file(row: &[String]) -> Option<String> {
	for f in row {
		if f.starts_with("source_file:") {
			return Some(f[12..].to_string());
		}
	}
	None
}

#[cfg(test)]
mod tests {
	use std::{env, fs};

	use super::*;

	#[test]
	fn fs_storage_writes_and_reads() {
		let mut path = env::temp_dir();
		let ts = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_nanos();
		path.push(format!("dumptruck_storage_test_{}.csv", ts));

		// ensure file removed at end
		let _ = fs::remove_file(&path);

		let mut store = FsStorage::new(path.clone()).expect("create storage");
		let row = vec!["a".to_string(), "b,c".to_string(), "d".to_string()];
		store.store_row(&row).expect("store row");

		let content = FsStorage::read_all(&path).expect("read back");
		assert!(content.contains("a,\"b,c\",d"));

		// test contains_hash
		let sha = "deadbeef";
		let _ = fs::write(&path, format!("{}\n", sha));
		let mut s = FsStorage::new(path.clone()).expect("open");
		assert!(s.contains_hash(sha).expect("contains"));

		// cleanup
		let _ = fs::remove_file(&path);
	}
}
