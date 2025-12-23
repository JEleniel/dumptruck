//! SQLite schema initialization and management.

use std::io;

use rusqlite::Connection;

/// Initialize the SQLite schema with all required tables and indexes.
pub fn create_schema(conn: &Connection) -> io::Result<()> {
	for statement in sql_statements() {
		conn.execute(&statement, [])
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
	}
	Ok(())
}

/// Get SQL statements for schema creation.
fn sql_statements() -> Vec<&'static str> {
	vec![
		"CREATE TABLE IF NOT EXISTS normalized_rows (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			dataset TEXT,
			event_type TEXT,
			address_hash TEXT,
			credential_hash TEXT,
			row_hash TEXT,
			file_id TEXT,
			source_file TEXT,
			fields TEXT,
			created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
		)",
		"CREATE TABLE IF NOT EXISTS canonical_addresses (
			canonical_hash TEXT PRIMARY KEY,
			address_text TEXT NOT NULL,
			normalized_form TEXT NOT NULL,
			embedding BLOB,
			created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
		)",
		"CREATE TABLE IF NOT EXISTS address_alternates (
			canonical_hash TEXT NOT NULL,
			alternate_hash TEXT NOT NULL,
			alternate_form TEXT NOT NULL,
			created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
			PRIMARY KEY (canonical_hash, alternate_hash),
			FOREIGN KEY (canonical_hash) REFERENCES canonical_addresses(canonical_hash)
		)",
		"CREATE TABLE IF NOT EXISTS address_credentials (
			canonical_hash TEXT NOT NULL,
			credential_hash TEXT NOT NULL,
			occurrence_count INTEGER DEFAULT 1,
			first_seen_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
			last_seen_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
			PRIMARY KEY (canonical_hash, credential_hash),
			FOREIGN KEY (canonical_hash) REFERENCES canonical_addresses(canonical_hash)
		)",
		"CREATE TABLE IF NOT EXISTS address_cooccurrence (
			canonical_hash_1 TEXT NOT NULL,
			canonical_hash_2 TEXT NOT NULL,
			cooccurrence_count INTEGER DEFAULT 1,
			first_seen_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
			last_seen_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
			PRIMARY KEY (canonical_hash_1, canonical_hash_2),
			FOREIGN KEY (canonical_hash_1) REFERENCES canonical_addresses(canonical_hash),
			FOREIGN KEY (canonical_hash_2) REFERENCES canonical_addresses(canonical_hash)
		)",
		"CREATE TABLE IF NOT EXISTS address_breaches (
			canonical_hash TEXT NOT NULL,
			breach_name TEXT NOT NULL,
			breach_title TEXT,
			breach_domain TEXT,
			breach_date TEXT,
			pwn_count INTEGER,
			description TEXT,
			is_verified BOOLEAN DEFAULT 0,
			is_fabricated BOOLEAN DEFAULT 0,
			is_sensitive BOOLEAN DEFAULT 0,
			is_retired BOOLEAN DEFAULT 0,
			created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
			FOREIGN KEY (canonical_hash) REFERENCES canonical_addresses(canonical_hash)
		)",
		"CREATE TABLE IF NOT EXISTS file_metadata (
			file_id TEXT PRIMARY KEY,
			original_filename TEXT NOT NULL,
			sha256_hash TEXT NOT NULL UNIQUE,
			blake3_hash TEXT,
			file_size INTEGER NOT NULL,
			alternate_names TEXT,
			processing_status TEXT,
			created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
		)",
		"CREATE TABLE IF NOT EXISTS chain_of_custody (
			file_id TEXT NOT NULL,
			record_id TEXT PRIMARY KEY,
			custody_action TEXT NOT NULL,
			operator TEXT NOT NULL,
			file_hash TEXT NOT NULL,
			signature BLOB NOT NULL,
			public_key BLOB NOT NULL,
			timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
			FOREIGN KEY (file_id) REFERENCES file_metadata(file_id)
		)",
		"CREATE TABLE IF NOT EXISTS alias_relationships (
			canonical_hash TEXT NOT NULL,
			variant_hash TEXT NOT NULL,
			alias_type TEXT NOT NULL,
			confidence INTEGER NOT NULL,
			created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
			PRIMARY KEY (canonical_hash, variant_hash),
			FOREIGN KEY (canonical_hash) REFERENCES canonical_addresses(canonical_hash)
		)",
		"CREATE TABLE IF NOT EXISTS anomaly_scores (
			file_id TEXT NOT NULL,
			subject_hash TEXT NOT NULL,
			anomaly_type TEXT NOT NULL,
			risk_score INTEGER NOT NULL,
			created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
			PRIMARY KEY (file_id, subject_hash, anomaly_type),
			FOREIGN KEY (file_id) REFERENCES file_metadata(file_id)
		)",
		"CREATE INDEX IF NOT EXISTS idx_normalized_address_hash ON normalized_rows(address_hash)",
		"CREATE INDEX IF NOT EXISTS idx_normalized_credential_hash ON \
		 normalized_rows(credential_hash)",
		"CREATE INDEX IF NOT EXISTS idx_address_alternates_hash ON \
		 address_alternates(alternate_hash)",
		"CREATE INDEX IF NOT EXISTS idx_breaches_canonical ON address_breaches(canonical_hash)",
		"CREATE INDEX IF NOT EXISTS idx_cooccurrence_both ON \
		 address_cooccurrence(canonical_hash_1, canonical_hash_2)",
	]
}
