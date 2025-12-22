//! Row storage and event type parsing.

use std::io;

use rusqlite::Connection;

/// Parse event type and extract typed columns from row data.
pub fn parse_event_and_columns(
	row: &[String],
) -> (
	Option<String>,
	Option<String>,
	Option<String>,
	Option<String>,
	Option<String>,
) {
	let mut event_type = None;
	let mut address_hash = None;
	let mut credential_hash = None;
	let mut row_hash = None;
	let mut file_id = None;

	for (i, f) in row.iter().enumerate() {
		if f.starts_with("file_id:") {
			file_id = Some(f[8..].to_string());
		} else if f.starts_with("row_hash:") {
			row_hash = Some(f[9..].to_string());
		} else if i == 0 && f.starts_with("__") {
			event_type = Some(f.clone());
			extract_addresses_from_event(f, row, &mut address_hash, &mut credential_hash);
		}
	}

	(event_type, address_hash, credential_hash, row_hash, file_id)
}

/// Extract address and credential hashes from event type and row.
fn extract_addresses_from_event(
	event: &str,
	row: &[String],
	addr_hash: &mut Option<String>,
	cred_hash: &mut Option<String>,
) {
	match event {
		"__address_hash__" if row.len() > 1 => {
			*addr_hash = Some(row[1].clone());
		}
		"__credential_hash__" if row.len() > 1 => {
			*cred_hash = Some(row[1].clone());
		}
		"__addr_cred__" if row.len() > 2 => {
			*addr_hash = Some(row[1].clone());
			*cred_hash = Some(row[2].clone());
		}
		"__known_address_new_credential__" if row.len() > 2 => {
			*addr_hash = Some(row[1].clone());
			*cred_hash = Some(row[2].clone());
		}
		"__new_address__" if row.len() > 1 => {
			*addr_hash = Some(row[1].clone());
		}
		_ => {}
	}
}

/// Build fields JSON from row, excluding special reserved fields.
pub fn build_fields_json(row: &[String]) -> io::Result<String> {
	let mut remaining: Vec<serde_json::Value> = Vec::new();

	for (i, f) in row.iter().enumerate() {
		if f.starts_with("file_id:") || f.starts_with("source_file:") || f.starts_with("row_hash:")
		{
			continue;
		}

		if i == 0 && f.starts_with("__") {
			remaining.push(serde_json::Value::String(f.clone()));
		} else {
			remaining.push(serde_json::Value::String(f.clone()));
		}
	}

	let fields = serde_json::Value::Array(remaining);
	serde_json::to_string(&fields).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Store a normalized row in the database.
pub fn store_row(
	conn: &Connection,
	dataset: Option<&str>,
	event_type: &Option<String>,
	address_hash: &Option<String>,
	credential_hash: &Option<String>,
	row_hash: &Option<String>,
	file_id: &Option<String>,
	source_file: &Option<String>,
	fields_text: &str,
) -> io::Result<()> {
	conn.execute(
		"INSERT INTO normalized_rows (dataset, event_type, address_hash, credential_hash, \
		 row_hash, file_id, source_file, fields) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
		rusqlite::params![
			dataset,
			event_type,
			address_hash,
			credential_hash,
			row_hash,
			file_id,
			source_file,
			fields_text,
		],
	)
	.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
	Ok(())
}
