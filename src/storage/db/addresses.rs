//! Canonical address and credential tracking.

use std::io;

use rusqlite::{Connection, OptionalExtension};

/// Insert a canonical address.
pub fn insert_canonical_address(
	conn: &Connection,
	canonical_hash: &str,
	address_text: &str,
	normalized_form: &str,
) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO canonical_addresses (canonical_hash, address_text, \
			 normalized_form) VALUES (?1, ?2, ?3)",
			rusqlite::params![canonical_hash, address_text, normalized_form],
		)
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(rows > 0)
}

/// Insert an address alternate mapping.
pub fn insert_address_alternate(
	conn: &Connection,
	canonical_hash: &str,
	alternate_hash: &str,
	alternate_form: &str,
) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO address_alternates (canonical_hash, alternate_hash, \
			 alternate_form) VALUES (?1, ?2, ?3)",
			rusqlite::params![canonical_hash, alternate_hash, alternate_form],
		)
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(rows > 0)
}

/// Lookup canonical address by alternate hash.
pub fn lookup_canonical_by_alternate(
	conn: &Connection,
	alternate_hash: &str,
) -> io::Result<Option<String>> {
	let mut stmt = conn
		.prepare("SELECT canonical_hash FROM address_alternates WHERE alternate_hash = ?1")
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	stmt.query_row(rusqlite::params![alternate_hash], |row| row.get(0))
		.optional()
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Insert credential association for canonical address.
pub fn insert_address_credential_canonical(
	conn: &Connection,
	canonical_hash: &str,
	credential_hash: &str,
) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO address_credentials (canonical_hash, credential_hash) VALUES \
			 (?1, ?2)",
			rusqlite::params![canonical_hash, credential_hash],
		)
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(rows > 0)
}

/// Get all credentials for a canonical address.
pub fn get_credentials_for_address(
	conn: &Connection,
	canonical_hash: &str,
) -> io::Result<Vec<String>> {
	let mut stmt = conn
		.prepare("SELECT credential_hash FROM address_credentials WHERE canonical_hash = ?1")
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let creds = stmt
		.query_map(rusqlite::params![canonical_hash], |row| row.get(0))
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(creds)
}
