use std::{collections::BTreeSet, fs};

use dumptruck::{
	adapters::CsvAdapter, enrichment::ChecksumEnricher, pipeline::Pipeline, storage::StorageAdapter,
};

struct InMemoryStorage {
	rows: Vec<Vec<String>>,
	index: BTreeSet<String>,
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

	fn add_address_credential(&mut self, addr_hash: &str, cred_hash: &str) -> std::io::Result<()> {
		self.assoc
			.insert((addr_hash.to_string(), cred_hash.to_string()));
		Ok(())
	}
}

#[test]
fn pipeline_detects_unterminated_quote_as_malformed() {
	let path = "tests/fixtures/malformed_missing_quote.csv";
	let content = fs::read_to_string(path).expect("read fixture");

	let adapter = CsvAdapter::new();
	let enr = ChecksumEnricher::new();
	let store = InMemoryStorage::new();

	let p = Pipeline::new(adapter, enr, store);
	let storage = p.ingest(&content).expect("ingest");

	// Expect at least one __malformed_row__ entry
	assert!(
		storage
			.rows
			.iter()
			.any(|r| r.get(0).map(|s| s.as_str()) == Some("__malformed_row__"))
	);
}

#[test]
fn pipeline_detects_mismatched_columns() {
	let path = "tests/fixtures/malformed_mismatched_columns.csv";
	let content = fs::read_to_string(path).expect("read fixture");

	let adapter = CsvAdapter::new();
	let enr = ChecksumEnricher::new();
	let store = InMemoryStorage::new();

	let p = Pipeline::new(adapter, enr, store);
	let storage = p.ingest(&content).expect("ingest");

	let malformed_count = storage
		.rows
		.iter()
		.filter(|r| r.get(0).map(|s| s.as_str()) == Some("__malformed_row__"))
		.count();
	assert!(malformed_count >= 1);
}
