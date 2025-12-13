use std::{collections::BTreeSet, fs};

use dumptruck::{
	adapters::CsvAdapter, enrichment::SimpleEnricher, pipeline::Pipeline, storage::StorageAdapter,
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
fn simple_enricher_appends_len_and_checksum() {
	let path = "tests/fixtures/test_creds_small.csv";
	let content = fs::read_to_string(path).expect("read fixture");

	let adapter = CsvAdapter::new();
	let enr = SimpleEnricher::new();
	let store = InMemoryStorage::new();

	let p = Pipeline::new(adapter, enr, store);
	let storage = p.ingest(&content).expect("ingest");

	// find at least one data row with appended len and checksum
	let found = storage.rows.iter().any(|r| {
		r.iter().any(|s| s.starts_with("len:")) && r.iter().any(|s| s.starts_with("checksum:"))
	});
	assert!(
		found,
		"expected at least one enriched row with len and checksum"
	);
}
