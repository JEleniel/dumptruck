//! Chain of Custody Module (Stage 4)
//!
//! Provides cryptographically signed audit trail records for forensic compliance.
//!
//! Each record is signed with ED25519 to create an immutable chain of custody,
//! enabling verification that data processing was authorized and unmodified.
//!
//! **Key Components:**
//! - ChainOfCustodyRecord: Signed entry with operator, timestamp, action, and file hash
//! - KeyPair generation and management
//! - Signature verification for audit verification
//! - Serialization/deserialization for storage

use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Errors that can occur during chain of custody operations
#[derive(Error, Debug)]
pub enum ChainOfCustodyError {
	#[error("Invalid signature: {0}")]
	InvalidSignature(String),

	#[error("Key generation failed: {0}")]
	KeyGenerationFailed(String),

	#[error("Serialization failed: {0}")]
	SerializationFailed(#[from] serde_json::Error),

	#[error("Invalid public key: {0}")]
	InvalidPublicKey(String),

	#[error("Signature verification failed: record may be tampered")]
	TamperingDetected,
}

/// Supported chain of custody actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CustodyAction {
	/// File ingested and processing started
	FileIngested,
	/// File validation completed
	FileValidated,
	/// Deduplication check completed
	DuplicationCheck,
	/// Enrichment completed
	EnrichmentComplete,
	/// Data stored to database
	DataStored,
	/// Temporary files securely deleted
	TemporaryFilesDeleted,
	/// Processing completed successfully
	ProcessingComplete,
}

impl fmt::Display for CustodyAction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			CustodyAction::FileIngested => write!(f, "file_ingested"),
			CustodyAction::FileValidated => write!(f, "file_validated"),
			CustodyAction::DuplicationCheck => write!(f, "duplication_check"),
			CustodyAction::EnrichmentComplete => write!(f, "enrichment_complete"),
			CustodyAction::DataStored => write!(f, "data_stored"),
			CustodyAction::TemporaryFilesDeleted => write!(f, "temporary_files_deleted"),
			CustodyAction::ProcessingComplete => write!(f, "processing_complete"),
		}
	}
}

/// A single chain of custody record with cryptographic signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfCustodyRecord {
	/// Unique record identifier (UUID format recommended)
	pub record_id: String,

	/// File identifier being processed (from Stage 1: Evidence Preservation)
	pub file_id: String,

	/// SHA-256 hash of the file for integrity verification
	pub file_hash: String,

	/// Operator/user who initiated this action (required for audit trail)
	pub operator: String,

	/// Action performed in the pipeline
	pub action: CustodyAction,

	/// Timestamp when action was performed (UTC)
	pub timestamp: DateTime<Utc>,

	/// Number of records processed in this action
	pub record_count: u64,

	/// ED25519 signature of the record (hex-encoded)
	pub signature: String,

	/// Public key used to sign this record (hex-encoded for verification)
	pub public_key: String,

	/// Optional metadata or notes about this action
	pub notes: Option<String>,
}

impl ChainOfCustodyRecord {
	/// Create a new unsigned custody record
	pub fn new(
		file_id: String,
		file_hash: String,
		operator: String,
		action: CustodyAction,
		record_count: u64,
	) -> Self {
		Self {
			record_id: uuid::Uuid::new_v4().to_string(),
			file_id,
			file_hash,
			operator,
			action,
			timestamp: Utc::now(),
			record_count,
			signature: String::new(),
			public_key: String::new(),
			notes: None,
		}
	}

	/// Add notes to the record
	pub fn with_notes(mut self, notes: String) -> Self {
		self.notes = Some(notes);
		self
	}

	/// Sign this record with an ED25519 private key
	///
	/// # Arguments
	/// * `private_key_bytes` - Raw 32-byte ED25519 private key
	///
	/// # Returns
	/// Self with signature and public_key populated
	pub fn sign(mut self, private_key_bytes: &[u8; 32]) -> Result<Self, ChainOfCustodyError> {
		// Generate signing key from bytes
		let signing_key = SigningKey::from_bytes(private_key_bytes);

		// Get public key for verification
		let verifying_key = signing_key.verifying_key();
		self.public_key = hex::encode(verifying_key.to_bytes());

		// Serialize record without signature for signing
		let record_to_sign = serde_json::json!({
			"record_id": self.record_id,
			"file_id": self.file_id,
			"file_hash": self.file_hash,
			"operator": self.operator,
			"action": self.action.to_string(),
			"timestamp": self.timestamp.to_rfc3339(),
			"record_count": self.record_count,
			"notes": self.notes,
		});

		let message = serde_json::to_string(&record_to_sign)
			.map_err(|e| ChainOfCustodyError::SerializationFailed(e))?;

		// Sign the message
		let signature = signing_key.sign(message.as_bytes());
		self.signature = hex::encode(signature.to_bytes());

		Ok(self)
	}

	/// Verify this record's signature
	///
	/// Returns true if signature is valid and was created by the claimed public key
	pub fn verify_signature(&self) -> Result<bool, ChainOfCustodyError> {
		// Decode public key from hex
		let pub_key_bytes = hex::decode(&self.public_key)
			.map_err(|e| ChainOfCustodyError::InvalidPublicKey(e.to_string()))?;

		if pub_key_bytes.len() != 32 {
			return Err(ChainOfCustodyError::InvalidPublicKey(
				"Public key must be 32 bytes".to_string(),
			));
		}

		let mut key_array = [0u8; 32];
		key_array.copy_from_slice(&pub_key_bytes);
		let verifying_key = VerifyingKey::from_bytes(&key_array)
			.map_err(|e| ChainOfCustodyError::InvalidPublicKey(e.to_string()))?;

		// Decode signature from hex
		let sig_bytes = hex::decode(&self.signature)
			.map_err(|e| ChainOfCustodyError::InvalidSignature(e.to_string()))?;

		if sig_bytes.len() != 64 {
			return Err(ChainOfCustodyError::InvalidSignature(
				"Signature must be 64 bytes".to_string(),
			));
		}

		let mut sig_array = [0u8; 64];
		sig_array.copy_from_slice(&sig_bytes);
		let signature = Signature::from_bytes(&sig_array);

		// Reconstruct message to verify
		let record_to_verify = serde_json::json!({
			"record_id": self.record_id,
			"file_id": self.file_id,
			"file_hash": self.file_hash,
			"operator": self.operator,
			"action": self.action.to_string(),
			"timestamp": self.timestamp.to_rfc3339(),
			"record_count": self.record_count,
			"notes": self.notes,
		});

		let message = serde_json::to_string(&record_to_verify)
			.map_err(|e| ChainOfCustodyError::SerializationFailed(e))?;

		verifying_key
			.verify_strict(message.as_bytes(), &signature)
			.map_err(|_| ChainOfCustodyError::TamperingDetected)?;

		Ok(true)
	}
}

/// ED25519 key pair for signing and verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyKeyPair {
	/// Private key (32 bytes, hex-encoded) - Keep secure!
	pub private_key: String,

	/// Public key (32 bytes, hex-encoded) - Safe to distribute
	pub public_key: String,
}

impl CustodyKeyPair {
	/// Generate a new random ED25519 key pair
	pub fn generate() -> Result<Self, ChainOfCustodyError> {
		// Generate random 32 bytes for the secret key
		let mut secret_bytes = [0u8; 32];
		use rand::RngCore;
		let mut rng = OsRng;
		rng.fill_bytes(&mut secret_bytes);

		let signing_key = SigningKey::from_bytes(&secret_bytes);
		let verifying_key = signing_key.verifying_key();

		Ok(CustodyKeyPair {
			private_key: hex::encode(signing_key.to_bytes()),
			public_key: hex::encode(verifying_key.to_bytes()),
		})
	}

	/// Get the private key as bytes (32 bytes)
	pub fn private_key_bytes(&self) -> Result<[u8; 32], ChainOfCustodyError> {
		let bytes = hex::decode(&self.private_key).map_err(|e| {
			ChainOfCustodyError::InvalidPublicKey(format!("Failed to decode private key: {}", e))
		})?;

		if bytes.len() != 32 {
			return Err(ChainOfCustodyError::InvalidPublicKey(
				"Private key must be 32 bytes".to_string(),
			));
		}

		let mut key_array = [0u8; 32];
		key_array.copy_from_slice(&bytes);
		Ok(key_array)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_generate_keypair() {
		let keypair = CustodyKeyPair::generate().expect("Should generate keypair");
		assert_eq!(keypair.private_key.len(), 64); // 32 bytes as hex
		assert_eq!(keypair.public_key.len(), 64); // 32 bytes as hex
	}

	#[test]
	fn test_create_and_sign_record() {
		let keypair = CustodyKeyPair::generate().expect("Should generate keypair");
		let private_key_bytes = keypair
			.private_key_bytes()
			.expect("Should decode private key");

		let record = ChainOfCustodyRecord::new(
			"file-123".to_string(),
			"abc123def456".to_string(),
			"operator@example.com".to_string(),
			CustodyAction::FileIngested,
			42,
		);

		let signed = record.sign(&private_key_bytes).expect("Should sign record");

		assert!(!signed.signature.is_empty());
		assert_eq!(signed.public_key, keypair.public_key);
		assert_eq!(signed.record_count, 42);
	}

	#[test]
	fn test_verify_valid_signature() {
		let keypair = CustodyKeyPair::generate().expect("Should generate keypair");
		let private_key_bytes = keypair
			.private_key_bytes()
			.expect("Should decode private key");

		let record = ChainOfCustodyRecord::new(
			"file-123".to_string(),
			"abc123def456".to_string(),
			"operator@example.com".to_string(),
			CustodyAction::FileValidated,
			100,
		);

		let signed = record.sign(&private_key_bytes).expect("Should sign record");

		let is_valid = signed.verify_signature().expect("Should verify signature");
		assert!(is_valid);
	}

	#[test]
	fn test_verify_fails_on_tampering() {
		let keypair = CustodyKeyPair::generate().expect("Should generate keypair");
		let private_key_bytes = keypair
			.private_key_bytes()
			.expect("Should decode private key");

		let record = ChainOfCustodyRecord::new(
			"file-123".to_string(),
			"abc123def456".to_string(),
			"operator@example.com".to_string(),
			CustodyAction::DataStored,
			50,
		);

		let mut signed = record.sign(&private_key_bytes).expect("Should sign record");

		// Tamper with the record
		signed.record_count = 999;

		let result = signed.verify_signature();
		assert!(result.is_err());
		assert!(matches!(
			result.err(),
			Some(ChainOfCustodyError::TamperingDetected)
		));
	}

	#[test]
	fn test_record_with_notes() {
		let keypair = CustodyKeyPair::generate().expect("Should generate keypair");
		let private_key_bytes = keypair
			.private_key_bytes()
			.expect("Should decode private key");

		let record = ChainOfCustodyRecord::new(
			"file-456".to_string(),
			"xyz789abc".to_string(),
			"admin@example.com".to_string(),
			CustodyAction::EnrichmentComplete,
			250,
		)
		.with_notes("Completed enrichment with HIBP lookup".to_string());

		let signed = record.sign(&private_key_bytes).expect("Should sign record");

		assert_eq!(
			signed.notes,
			Some("Completed enrichment with HIBP lookup".to_string())
		);
		let is_valid = signed.verify_signature().expect("Should verify");
		assert!(is_valid);
	}
}
