//! Co-occurrence and breach data management.

use rusqlite::Connection;
use std::io;

/// Record address co-occurrence edge.
pub fn record_address_cooccurrence(
	conn: &Connection,
	hash_1: &str,
	hash_2: &str,
) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO address_cooccurrence \
			 (canonical_hash_1, canonical_hash_2) VALUES (?1, ?2)",
			rusqlite::params![hash_1, hash_2],
		)
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(rows > 0)
}

/// Get neighbors in co-occurrence graph.
pub fn get_address_neighbors(
	conn: &Connection,
	canonical_hash: &str,
) -> io::Result<Vec<(String, i32)>> {
	let mut stmt = conn
		.prepare("SELECT canonical_hash_2, cooccurrence_count FROM address_cooccurrence WHERE canonical_hash_1 = ?1")
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

/// Insert breach data for canonical address.
pub fn insert_address_breach(
	conn: &Connection,
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
) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO address_breaches \
			 (canonical_hash, breach_name, breach_title, breach_domain, breach_date, \
			  pwn_count, description, is_verified, is_fabricated, is_sensitive, is_retired) \
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
			rusqlite::params![
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
			],
		)
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	Ok(rows > 0)
}
