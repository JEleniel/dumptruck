//! Co-occurrence and breach data management.

use rusqlite::Connection;
use std::io;

/// Parameters for inserting a breach record into the database.
#[derive(Debug, Clone)]
pub struct BreachRecord<'a> {
	/// Canonical hash of the breached address
	pub canonical_hash: &'a str,
	/// Name of the breach (e.g., "Collection #1")
	pub breach_name: &'a str,
	/// Optional friendly title for the breach
	pub breach_title: Option<&'a str>,
	/// Optional domain/source of the breach
	pub breach_domain: Option<&'a str>,
	/// Optional date of the breach
	pub breach_date: Option<&'a str>,
	/// Optional count of credentials in breach
	pub pwn_count: Option<i32>,
	/// Optional breach description
	pub description: Option<&'a str>,
	/// Whether the breach is verified
	pub is_verified: bool,
	/// Whether the breach is fabricated/honeypot
	pub is_fabricated: bool,
	/// Whether the breach contains sensitive info
	pub is_sensitive: bool,
	/// Whether the breach is retired/old
	pub is_retired: bool,
}

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
		.map_err(io::Error::other)?;

	Ok(rows > 0)
}

/// Get neighbors in co-occurrence graph.
pub fn get_address_neighbors(
	conn: &Connection,
	canonical_hash: &str,
) -> io::Result<Vec<(String, i32)>> {
	let mut stmt = conn
		.prepare("SELECT canonical_hash_2, cooccurrence_count FROM address_cooccurrence WHERE canonical_hash_1 = ?1")
		.map_err(io::Error::other)?;

	let neighbors = stmt
		.query_map(rusqlite::params![canonical_hash], |row| {
			Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
		})
		.map_err(io::Error::other)?
		.collect::<Result<Vec<_>, _>>()
		.map_err(io::Error::other)?;

	Ok(neighbors)
}

/// Insert breach data for canonical address.
pub fn insert_address_breach(
	conn: &Connection,
	record: &BreachRecord<'_>,
) -> io::Result<bool> {
	let rows = conn
		.execute(
			"INSERT OR IGNORE INTO address_breaches \
			 (canonical_hash, breach_name, breach_title, breach_domain, breach_date, \
			  pwn_count, description, is_verified, is_fabricated, is_sensitive, is_retired) \
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
			rusqlite::params![
				record.canonical_hash,
				record.breach_name,
				record.breach_title,
				record.breach_domain,
				record.breach_date,
				record.pwn_count,
				record.description,
				record.is_verified,
				record.is_fabricated,
				record.is_sensitive,
				record.is_retired,
			],
		)
		.map_err(io::Error::other)?;

	Ok(rows > 0)
}
