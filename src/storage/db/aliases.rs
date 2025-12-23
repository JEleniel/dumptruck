//! Alias relationship tracking.

use rusqlite::Connection;
use std::io;

/// Insert alias relationship.
pub fn insert_alias_relationship(
	conn: &Connection,
	canonical_hash: &str,
	variant_hash: &str,
	alias_type: &str,
	confidence: i32,
) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO alias_relationships \
			 (canonical_hash, variant_hash, alias_type, confidence) VALUES (?1, ?2, ?3, ?4)",
			rusqlite::params![canonical_hash, variant_hash, alias_type, confidence],
		)
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(rows > 0)
}

/// Get all alias relationships for a canonical address.
pub fn get_alias_relationships(
	conn: &Connection,
	canonical_hash: &str,
) -> io::Result<Vec<(String, String, i32)>> {
	let mut stmt = conn
		.prepare(
			"SELECT variant_hash, alias_type, confidence FROM alias_relationships \
			 WHERE canonical_hash = ?1",
		)
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	let aliases = stmt
		.query_map(rusqlite::params![canonical_hash], |row| {
			Ok((
				row.get::<_, String>(0)?,
				row.get::<_, String>(1)?,
				row.get::<_, i32>(2)?,
			))
		})
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(aliases)
}
