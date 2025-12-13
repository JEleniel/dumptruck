// Simple in-memory storage for tests
use std::{collections::BTreeSet, fs};

use dumptruck::{
	adapters::{CsvAdapter, FormatAdapter},
	enrichment::ChecksumEnricher,
	pipeline::Pipeline,
	storage::StorageAdapter,
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
fn parses_fixture_file() {
	let path = "tests/fixtures/test_creds_small.csv";
	let content = fs::read_to_string(path).expect("read fixture");
	let adapter = CsvAdapter::new();
	let rows = adapter.parse(&content);
	// expect header + 10 rows = 11
	assert_eq!(rows.len(), 11);
	// first row should be header with ID and credential
	assert_eq!(rows[0][0].to_lowercase(), "id");
	assert_eq!(rows[0][1].to_lowercase(), "credential");
}

#[test]
fn detects_duplicate_hash_entries_when_ingesting_same_file_twice() {
	let path = "tests/fixtures/test_creds_mixed.csv";
	let content = fs::read_to_string(path).expect("read fixture");

	let adapter = CsvAdapter::new();
	let enr = ChecksumEnricher::new();
	let store = InMemoryStorage::new();

	// First ingest
	let p = Pipeline::new(adapter, enr, store);
	let storage_after_first = p.ingest(&content).expect("first ingest");

	let addr_count_first = storage_after_first
		.rows
		.iter()
		.filter(|r| r.get(0).map(|s| s.as_str()) == Some("__address_hash__"))
		.count();
	let cred_count_first = storage_after_first
		.rows
		.iter()
		.filter(|r| r.get(0).map(|s| s.as_str()) == Some("__credential_hash__"))
		.count();

	// Second ingest using returned storage
	let adapter2 = CsvAdapter::new();
	let enr2 = ChecksumEnricher::new();
	let p2 = Pipeline::new(adapter2, enr2, storage_after_first);
	let storage_after_second = p2.ingest(&content).expect("second ingest");

	let addr_count_second = storage_after_second
		.rows
		.iter()
		.filter(|r| r.get(0).map(|s| s.as_str()) == Some("__address_hash__"))
		.count();
	let cred_count_second = storage_after_second
		.rows
		.iter()
		.filter(|r| r.get(0).map(|s| s.as_str()) == Some("__credential_hash__"))
		.count();

	// counts should be equal (no duplicated hash entries)
	assert_eq!(addr_count_first, addr_count_second);
	assert_eq!(cred_count_first, cred_count_second);
}
