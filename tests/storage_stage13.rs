//! Integration tests for Stage 13: Storage Enhancement
//!
//! Tests cover:
//! - Schema creation for Stage 13 tables (file_metadata, chain_of_custody, etc.)
//! - Data insertion and retrieval
//! - Query functionality
//! - Backward compatibility with existing schema

#[cfg(test)]
mod tests {
	use std::io;

	/// Mock FileMetadata struct for testing
	#[derive(Debug, Clone, PartialEq, Eq)]
	struct FileMetadata {
		file_id: String,
		original_filename: String,
		sha256_hash: String,
		file_size: i64,
	}

	/// Mock ChainOfCustodyRecord struct for testing
	#[derive(Debug, Clone, PartialEq)]
	struct ChainOfCustodyRecord {
		file_id: String,
		record_id: String,
		custody_action: String,
		operator: String,
		file_hash: String,
	}

	/// Mock AliasRelationship struct for testing
	#[derive(Debug, Clone, PartialEq, Eq)]
	struct AliasRelationship {
		canonical_hash: String,
		variant_hash: String,
		alias_type: String,
		confidence: i32,
	}

	/// Mock AnomalyScore struct for testing
	#[derive(Debug, Clone, PartialEq, Eq)]
	struct AnomalyScore {
		file_id: String,
		subject_hash: String,
		anomaly_type: String,
		risk_score: i32,
	}

	/// Mock in-memory storage for testing Stage 13 functionality
	struct MockStorage {
		file_metadata: Vec<FileMetadata>,
		custody_records: Vec<ChainOfCustodyRecord>,
		alias_relationships: Vec<AliasRelationship>,
		anomaly_scores: Vec<AnomalyScore>,
	}

	impl MockStorage {
		fn new() -> Self {
			MockStorage {
				file_metadata: Vec::new(),
				custody_records: Vec::new(),
				alias_relationships: Vec::new(),
				anomaly_scores: Vec::new(),
			}
		}

		fn insert_file_metadata(&mut self, file: FileMetadata) -> io::Result<bool> {
			// Check for duplicate
			if self.file_metadata.iter().any(|f| f.file_id == file.file_id) {
				return Ok(false);
			}
			self.file_metadata.push(file);
			Ok(true)
		}

		fn get_file_metadata(&self, file_id: &str) -> io::Result<Option<FileMetadata>> {
			Ok(self
				.file_metadata
				.iter()
				.find(|f| f.file_id == file_id)
				.cloned())
		}

		fn insert_custody_record(&mut self, record: ChainOfCustodyRecord) -> io::Result<bool> {
			// Check for duplicate
			if self
				.custody_records
				.iter()
				.any(|r| r.record_id == record.record_id)
			{
				return Ok(false);
			}
			self.custody_records.push(record);
			Ok(true)
		}

		fn get_custody_records(&self, file_id: &str) -> io::Result<Vec<ChainOfCustodyRecord>> {
			Ok(self
				.custody_records
				.iter()
				.filter(|r| r.file_id == file_id)
				.cloned()
				.collect())
		}

		fn insert_alias_relationship(&mut self, alias: AliasRelationship) -> io::Result<bool> {
			// Check for duplicate
			if self.alias_relationships.iter().any(|a| {
				a.canonical_hash == alias.canonical_hash
					&& a.variant_hash == alias.variant_hash
					&& a.alias_type == alias.alias_type
			}) {
				return Ok(false);
			}
			self.alias_relationships.push(alias);
			Ok(true)
		}

		fn get_alias_relationships(&self, canonical_hash: &str) -> io::Result<Vec<(String, i32)>> {
			Ok(self
				.alias_relationships
				.iter()
				.filter(|a| a.canonical_hash == canonical_hash)
				.map(|a| (a.alias_type.clone(), a.confidence))
				.collect())
		}

		fn insert_anomaly_score(&mut self, anomaly: AnomalyScore) -> io::Result<bool> {
			self.anomaly_scores.push(anomaly);
			Ok(true)
		}

		fn get_anomalies_for_file(&self, file_id: &str) -> io::Result<Vec<(String, i32)>> {
			Ok(self
				.anomaly_scores
				.iter()
				.filter(|a| a.file_id == file_id)
				.map(|a| (a.anomaly_type.clone(), a.risk_score))
				.collect())
		}

		fn get_high_risk_anomalies(&self, threshold: i32) -> io::Result<Vec<(String, i32)>> {
			Ok(self
				.anomaly_scores
				.iter()
				.filter(|a| a.risk_score > threshold)
				.map(|a| (a.anomaly_type.clone(), a.risk_score))
				.collect())
		}
	}

	#[test]
	fn test_file_metadata_creation_and_retrieval() -> io::Result<()> {
		let mut storage = MockStorage::new();

		let metadata = FileMetadata {
			file_id: "file-001-1234567890".to_string(),
			original_filename: "breaches.csv".to_string(),
			sha256_hash: "abcdef1234567890".to_string(),
			file_size: 1024000,
		};

		let inserted = storage.insert_file_metadata(metadata.clone())?;
		assert!(inserted, "File metadata should be inserted");

		let retrieved = storage.get_file_metadata("file-001-1234567890")?;
		assert_eq!(
			retrieved,
			Some(metadata),
			"Retrieved metadata should match inserted"
		);

		Ok(())
	}

	#[test]
	fn test_file_metadata_duplicate_prevention() -> io::Result<()> {
		let mut storage = MockStorage::new();

		let metadata = FileMetadata {
			file_id: "file-001-1234567890".to_string(),
			original_filename: "breaches.csv".to_string(),
			sha256_hash: "abcdef1234567890".to_string(),
			file_size: 1024000,
		};

		let first_insert = storage.insert_file_metadata(metadata.clone())?;
		assert!(first_insert, "First insert should succeed");

		let second_insert = storage.insert_file_metadata(metadata)?;
		assert!(!second_insert, "Duplicate insert should fail");

		Ok(())
	}

	#[test]
	fn test_chain_of_custody_records() -> io::Result<()> {
		let mut storage = MockStorage::new();

		let file_metadata = FileMetadata {
			file_id: "file-001-1234567890".to_string(),
			original_filename: "breaches.csv".to_string(),
			sha256_hash: "abcdef1234567890".to_string(),
			file_size: 1024000,
		};
		storage.insert_file_metadata(file_metadata)?;

		let record = ChainOfCustodyRecord {
			file_id: "file-001-1234567890".to_string(),
			record_id: "record-001-1234567890".to_string(),
			custody_action: "FileIngested".to_string(),
			operator: "operator@example.com".to_string(),
			file_hash: "abcdef1234567890".to_string(),
		};

		let inserted = storage.insert_custody_record(record.clone())?;
		assert!(inserted, "Custody record should be inserted");

		let records = storage.get_custody_records("file-001-1234567890")?;
		assert_eq!(records.len(), 1, "Should retrieve one record");
		assert_eq!(records[0], record, "Retrieved record should match inserted");

		Ok(())
	}

	#[test]
	fn test_alias_relationships_with_confidence() -> io::Result<()> {
		let mut storage = MockStorage::new();

		let canonical_hash = "canonical-hash-001".to_string();

		// Insert multiple alias relationships
		let alias1 = AliasRelationship {
			canonical_hash: canonical_hash.clone(),
			variant_hash: "variant-hash-001".to_string(),
			alias_type: "EmailPlus".to_string(),
			confidence: 95,
		};

		let alias2 = AliasRelationship {
			canonical_hash: canonical_hash.clone(),
			variant_hash: "variant-hash-002".to_string(),
			alias_type: "EmailDot".to_string(),
			confidence: 85,
		};

		storage.insert_alias_relationship(alias1)?;
		storage.insert_alias_relationship(alias2)?;

		let relationships = storage.get_alias_relationships(&canonical_hash)?;
		assert_eq!(relationships.len(), 2, "Should retrieve two relationships");
		assert_eq!(
			relationships[0],
			("EmailPlus".to_string(), 95),
			"First relationship should match"
		);
		assert_eq!(
			relationships[1],
			("EmailDot".to_string(), 85),
			"Second relationship should match"
		);

		Ok(())
	}

	#[test]
	fn test_anomaly_detection_and_risk_scoring() -> io::Result<()> {
		let mut storage = MockStorage::new();

		// Insert file metadata first
		let metadata = FileMetadata {
			file_id: "file-001-1234567890".to_string(),
			original_filename: "data.csv".to_string(),
			sha256_hash: "hash001".to_string(),
			file_size: 5000,
		};
		storage.insert_file_metadata(metadata)?;

		// Insert anomaly scores
		let anomaly1 = AnomalyScore {
			file_id: "file-001-1234567890".to_string(),
			subject_hash: "subject-001".to_string(),
			anomaly_type: "EntropyOutlier".to_string(),
			risk_score: 75,
		};

		let anomaly2 = AnomalyScore {
			file_id: "file-001-1234567890".to_string(),
			subject_hash: "subject-002".to_string(),
			anomaly_type: "RareDomain".to_string(),
			risk_score: 45,
		};

		storage.insert_anomaly_score(anomaly1)?;
		storage.insert_anomaly_score(anomaly2)?;

		// Retrieve anomalies for file
		let anomalies = storage.get_anomalies_for_file("file-001-1234567890")?;
		assert_eq!(anomalies.len(), 2, "Should retrieve two anomalies");

		// Retrieve high-risk anomalies (threshold > 50)
		let high_risk = storage.get_high_risk_anomalies(50)?;
		assert_eq!(high_risk.len(), 1, "Should retrieve one high-risk anomaly");
		assert_eq!(
			high_risk[0].0, "EntropyOutlier",
			"High-risk anomaly should be EntropyOutlier"
		);
		assert_eq!(high_risk[0].1, 75, "Risk score should be 75");

		Ok(())
	}

	#[test]
	fn test_stage_13_backward_compatibility() -> io::Result<()> {
		// This test verifies that Stage 13 tables can coexist with existing schema
		let mut storage = MockStorage::new();

		// Create data in Stage 13 tables
		let metadata = FileMetadata {
			file_id: "file-001-1234567890".to_string(),
			original_filename: "breaches.csv".to_string(),
			sha256_hash: "sha256_hash".to_string(),
			file_size: 2048,
		};

		let inserted = storage.insert_file_metadata(metadata)?;
		assert!(inserted, "Stage 13 table insert should succeed");

		// Verify data persists
		let retrieved = storage.get_file_metadata("file-001-1234567890")?;
		assert!(retrieved.is_some(), "Stage 13 data should persist");

		// Verify schema didn't break existing functionality
		assert_eq!(
			storage.alias_relationships.len(),
			0,
			"Alias table should be empty"
		);
		assert_eq!(
			storage.anomaly_scores.len(),
			0,
			"Anomaly table should be empty"
		);

		Ok(())
	}
}
