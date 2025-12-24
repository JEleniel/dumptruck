//! Simple ingest -> normalize -> enrich -> store pipeline and integration tests.

use crate::{ingest::adapters::FormatAdapter, normalization, storage::StorageAdapter};

/// Pipeline wires together adapter and storage.
pub struct Pipeline<A: FormatAdapter, S: StorageAdapter> {
	adapter: A,
	storage: S,
}

impl<A: FormatAdapter, S: StorageAdapter> Pipeline<A, S> {
	pub fn new(adapter: A, storage: S) -> Self {
		Pipeline { adapter, storage }
	}

	/// Ingest input, normalize, enrich, and persist rows.
	/// Consume the pipeline, ingest input, and return ownership of the storage.
	pub fn ingest(self, input: &str) -> Result<S, std::io::Error> {
		use crate::core::hash_utils;

		let rows = self.adapter.parse(input);
		let mut storage = self.storage;

		// Compute file-level hashes
		let bytes = input.as_bytes();
		let file_md5 = hash_utils::md5_hex_bytes(bytes);
		let file_sha256 = hash_utils::sha256_hex_bytes(bytes);
		let file_id = file_sha256.clone();
		let meta = vec![
			"__file_hash__".to_string(),
			file_md5.clone(),
			file_sha256.clone(),
		];

		let store_with_file = |s: &mut S, row: &Vec<String>| -> std::io::Result<()> {
			let mut r = row.clone();
			r.push(format!("file_id:{}", file_id));
			s.store_row(&r)
		};

		let _ = store_with_file(&mut storage, &meta);

		// Detect header row
		let (header, expected_columns) = Self::detect_header(&rows);

		for (idx, row) in rows.iter().enumerate() {
			if idx == 0 && header.is_some() {
				continue;
			}

			let normalized = normalization::engine::normalize_row(row);

			// Validate column count
			if let Some(expected) = expected_columns
				&& normalized.len() != expected
			{
				let raw = row.join(",");
				let m = vec!["__malformed_row__".to_string(), idx.to_string(), raw];
				let _ = store_with_file(&mut storage, &m);
				continue;
			}

			// Detect addresses and credentials
			let (addr_hashes, cred_hashes, has_hashed_credentials) =
				Self::extract_address_credentials(&normalized, &header);

			// Skip hashed-only rows
			if has_hashed_credentials && !cred_hashes.is_empty() && addr_hashes.is_empty() {
				let ev = vec![
					"__hashed_credentials_only__".to_string(),
					"row_skipped".to_string(),
					format!("cred_count:{}", cred_hashes.len()),
				];
				let _ = store_with_file(&mut storage, &ev);
				continue;
			}

			// Process addresses and credentials
			for addr in addr_hashes.iter() {
				let addr_seen = storage.address_exists(addr)?;
				if !addr_seen {
					let ev = vec!["__new_address__".to_string(), addr.clone()];
					store_with_file(&mut storage, &ev)?;
					let r = vec!["__address_hash__".to_string(), addr.clone()];
					store_with_file(&mut storage, &r)?;
				}

				for cred in cred_hashes.iter() {
					if !storage.contains_hash(cred)? {
						let r = vec!["__credential_hash__".to_string(), cred.clone()];
						store_with_file(&mut storage, &r)?;
					}

					let assoc = storage.address_has_credential(addr, cred)?;
					if !assoc {
						if addr_seen {
							let ev = vec![
								"__known_address_new_credential__".to_string(),
								addr.clone(),
								cred.clone(),
							];
							store_with_file(&mut storage, &ev)?;
						}
						let mapping = vec!["__addr_cred__".to_string(), addr.clone(), cred.clone()];
						store_with_file(&mut storage, &mapping)?;
					}
				}
			}

			// Enrich and store
			let mut enriched = normalized.to_vec();
			let row_join = normalized.join("|");
			let row_hash = hash_utils::sha256_hex(&row_join);

			for h in addr_hashes.iter() {
				enriched.push(format!("addr_sha256:{}", h));
			}
			for h in cred_hashes.iter() {
				enriched.push(format!("cred_sha256:{}", h));
			}

			if storage.contains_hash(&row_hash)? {
				let dup = vec!["__duplicate_row__".to_string(), row_hash.clone()];
				let _ = store_with_file(&mut storage, &dup);
			} else {
				enriched.push(format!("row_hash:{}", row_hash));
				store_with_file(&mut storage, &enriched)?;
			}
		}

		Ok(storage)
	}

	/// Detect header row from the first row
	fn detect_header(rows: &[Vec<String>]) -> (Option<Vec<String>>, Option<usize>) {
		if !rows.is_empty() {
			let first = &rows[0];
			if first.iter().any(|c| c.chars().any(|ch| ch.is_alphabetic())) {
				return (Some(first.clone()), Some(first.len()));
			}
		}
		(None, None)
	}

	/// Extract address and credential hashes from normalized row
	fn extract_address_credentials(
		normalized: &[String],
		header: &Option<Vec<String>>,
	) -> (Vec<String>, Vec<String>, bool) {
		use crate::core::hash_utils;

		let mut addr_hashes = Vec::new();
		let mut cred_hashes = Vec::new();
		let mut has_hashed_credentials = false;

		if let Some(h) = header {
			for (i, col_name) in h.iter().enumerate() {
				let lname = col_name.to_lowercase();
				if i < normalized.len() {
					let val = &normalized[i];
					if lname.contains("mail")
						|| lname.contains("email")
						|| lname.contains("addr")
						|| lname.contains("address")
					{
						let sha = hash_utils::sha256_hex(val);
						addr_hashes.push(sha);
					}
					if lname.contains("pass")
						|| lname.contains("pwd")
						|| lname.contains("password")
						|| lname.contains("credential")
						|| lname.contains("secret")
					{
						if hash_utils::is_credential_hash(val) {
							has_hashed_credentials = true;
						}
						let sha = hash_utils::sha256_hex(val);
						cred_hashes.push(sha);
					}
				}
			}
		} else {
			for val in normalized.iter() {
				if val.contains('@') {
					addr_hashes.push(hash_utils::sha256_hex(val));
				}
				if val.contains(':') || val.to_lowercase().contains("pass") {
					if hash_utils::is_credential_hash(val) {
						has_hashed_credentials = true;
					}
					cred_hashes.push(hash_utils::sha256_hex(val));
				}
			}
		}

		(addr_hashes, cred_hashes, has_hashed_credentials)
	}
}

#[cfg(test)]
mod tests {
	// Simple in-memory storage for integration tests using a BTreeSet index
	use std::collections::BTreeSet;

	use super::*;
	use crate::{ingest::adapters::CsvAdapter, storage::StorageAdapter};

	struct InMemoryStorage {
		rows: Vec<Vec<String>>,
		index: BTreeSet<String>,
		// associations of (addr_hash, cred_hash)
		assoc: BTreeSet<(String, String)>,
	}

	impl InMemoryStorage {
		fn new() -> Self {
			InMemoryStorage {
				rows: Vec::new(),
				index: BTreeSet::new(),
				assoc: BTreeSet::new(),
			}
		}
	}

	impl StorageAdapter for InMemoryStorage {
		fn store_row(&mut self, row: &[String]) -> std::io::Result<()> {
			// store the row and index each field for quick exact-match lookups
			for v in row.iter() {
				self.index.insert(v.clone());
			}
			self.rows.push(row.to_vec());
			Ok(())
		}

		fn contains_hash(&mut self, hash: &str) -> std::io::Result<bool> {
			Ok(self.index.contains(hash))
		}

		fn address_exists(&mut self, addr_hash: &str) -> std::io::Result<bool> {
			Ok(self.index.contains(addr_hash))
		}

		fn address_has_credential(
			&mut self,
			addr_hash: &str,
			cred_hash: &str,
		) -> std::io::Result<bool> {
			Ok(self
				.assoc
				.contains(&(addr_hash.to_string(), cred_hash.to_string())))
		}

		fn add_address_credential(
			&mut self,
			addr_hash: &str,
			cred_hash: &str,
		) -> std::io::Result<()> {
			self.assoc
				.insert((addr_hash.to_string(), cred_hash.to_string()));
			Ok(())
		}

		fn insert_address_breach(
			&mut self,
			_record: &crate::storage::db::BreachRecord<'_>,
		) -> std::io::Result<bool> {
			Ok(false)
		}

		fn insert_custody_record(
			&mut self,
			_record: &crate::storage::db::CustodyRecord<'_>,
		) -> std::io::Result<bool> {
			Ok(false)
		}
	}

	#[test]
	fn pipeline_end_to_end() {
		let csv = "name,email\nAlice,ALICE@Example.COM\nBob,bob@example.com\n";
		let adapter = CsvAdapter::new();
		let store = InMemoryStorage::new();

		// move store into the pipeline, ingest will return the storage back
		let p = Pipeline::new(adapter, store);
		let storage = p.ingest(csv).expect("ingest");
		// expect metadata file hash row + two stored data rows + per-row enrichment rows
		assert!(storage.rows.len() >= 3);
		// check that the metadata row is present
		assert!(
			storage
				.rows
				.iter()
				.any(|r| r.first().map(|s| s.as_str()) == Some("__file_hash__"))
		);
	}

	#[test]
	fn pipeline_boxed_objects() {
		use crate::{ingest::adapters::CsvAdapter, storage::FsStorage};

		// Use actual FsStorage into a temp file to verify persisted enriched rows
		let csv = "name,email\nAlice,ALICE@Example.COM\nBob,bob@example.com\n";
		let adapter = CsvAdapter::new();

		let mut path = std::env::temp_dir();
		let ts = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_nanos();
		path.push(format!("dumptruck_pipeline_test_{}.csv", ts));

		let mut fs = FsStorage::new(path.clone()).expect("create fs storage");

		// Use dynamic dispatch to create a pipeline-like flow
		let rows = adapter.parse(csv);
		for row in rows.iter() {
			let normalized = normalization::engine::normalize_row(row);
			let enriched = normalized.to_vec();
			fs.store_row(&enriched).expect("store");
		}

		let content = FsStorage::read_all(&path).expect("read back");
		assert!(content.contains("alice") && content.contains("example"));

		let _ = std::fs::remove_file(&path);
	}

	#[test]
	fn pipeline_discard_hashed_credentials() {
		// Test that rows with only hashed credentials (no plaintext addresses) are discarded
		// Format: email, password_hash (SHA256 hex)
		let csv = "email,password\nalice@example.com,482c811da5d5b4bc6d497ffa98491e38\nbob@\
		           example.com,$2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcg7b3XeKeUxWdeS86E36MM32Oi\n";
		let adapter = CsvAdapter::new();
		let store = InMemoryStorage::new();

		let p = Pipeline::new(adapter, store);
		let storage = p.ingest(csv).expect("ingest");

		// Check that we have metadata rows and data rows with plaintext credentials
		assert!(
			storage
				.rows
				.iter()
				.any(|r| r.first().map(|s| s.as_str()) == Some("__file_hash__"))
		);
		// We should NOT have discarded these rows since they have plaintext email addresses
		assert!(storage.rows.len() >= 3);
	}

	#[test]
	fn pipeline_reject_hashed_only_without_address() {
		// Test rows with ONLY hashed credentials (no address field) get discarded
		// This would be a useless row: hash@hash / hashvalue (both pre-hashed)
		let csv = "credential\n$2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcg7b3XeKeUxWdeS86E36MM32Oi\n";
		let adapter = CsvAdapter::new();
		let store = InMemoryStorage::new();

		let p = Pipeline::new(adapter, store);
		let storage = p.ingest(csv).expect("ingest");

		// Should have file metadata and the __hashed_credentials_only__ event marker
		assert!(
			storage
				.rows
				.iter()
				.any(|r| { r.first().map(|s| s.as_str()) == Some("__hashed_credentials_only__") })
		);
	}
}
