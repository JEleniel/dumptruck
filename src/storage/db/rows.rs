//! Row storage and event type parsing.

use std::io;

use rusqlite::Connection;

/// Parsed event and column data from a row.
#[derive(Debug, Clone)]
pub struct RowData<'a> {
	/// Optional dataset name
	pub dataset: Option<&'a str>,
	/// Event type if present
	pub event_type: &'a Option<String>,
	/// Hash of the canonical address
	pub address_hash: &'a Option<String>,
	/// Hash of the credential
	pub credential_hash: &'a Option<String>,
	/// Hash of the entire row
	pub row_hash: &'a Option<String>,
	/// File identifier
	pub file_id: &'a Option<String>,
	/// Source file name
	pub source_file: &'a Option<String>,
	/// Serialized field data
	pub fields_text: &'a str,
}

/// Extracted column values from a row.
pub struct ParsedColumns {
	pub event_type: Option<String>,
	pub address_hash: Option<String>,
	pub credential_hash: Option<String>,
	pub row_hash: Option<String>,
	pub file_id: Option<String>,
}

/// Parse event type and extract typed columns from row data.
pub fn parse_event_and_columns(row: &[String]) -> ParsedColumns {
	let mut event_type = None;
	let mut address_hash = None;
	let mut credential_hash = None;
	let mut row_hash = None;
	let mut file_id = None;

	for (i, f) in row.iter().enumerate() {
		if let Some(stripped) = f.strip_prefix("file_id:") {
			file_id = Some(stripped.to_string());
		} else if let Some(stripped) = f.strip_prefix("row_hash:") {
			row_hash = Some(stripped.to_string());
		} else if i == 0 && f.starts_with("__") {
			event_type = Some(f.clone());
			extract_addresses_from_event(f, row, &mut address_hash, &mut credential_hash);
		}
	}

	ParsedColumns {
		event_type,
		address_hash,
		credential_hash,
		row_hash,
		file_id,
	}
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

	for f in row.iter() {
		if f.starts_with("file_id:") || f.starts_with("source_file:") || f.starts_with("row_hash:")
		{
			continue;
		}

		remaining.push(serde_json::Value::String(f.clone()));
	}

	let fields = serde_json::Value::Array(remaining);
	serde_json::to_string(&fields).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Store a normalized row in the database.
pub fn store_row(conn: &Connection, data: &RowData<'_>) -> io::Result<()> {
	conn.execute(
		"INSERT INTO normalized_rows (dataset, event_type, address_hash, credential_hash, \
		 row_hash, file_id, source_file, fields) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
		rusqlite::params![
			data.dataset,
			data.event_type,
			data.address_hash,
			data.credential_hash,
			data.row_hash,
			data.file_id,
			data.source_file,
			data.fields_text,
		],
	)
	.map_err(io::Error::other)?;
	Ok(())
}
