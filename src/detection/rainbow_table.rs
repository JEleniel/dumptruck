//! Rainbow table and hash detection for weak password discovery.
//!
//! This module loads weak password hashes from an external JSON file (.cache/rainbow_table.json).
//! At startup, the application checks if wordlist files have changed and regenerates the
//! JSON table as needed.
//!
//! Detection strategies:
//!
//! 1. **Weak Hash Fingerprinting** - Identifies algorithms by fingerprint patterns:
//!    - SHA1: 40 hex characters (unsalted)
//!    - MD5: 32 hex characters (unsalted)
//!    - SHA256: 64 hex characters (unsalted)
//!    - SHA512: 128 hex characters (unsalted)
//!
//! 2. **Rainbow Tables** - Dictionary attacks on salted algorithms:
//!    - Argon2id/Argon2i/Argon2d: $argon2* prefixes
//!    - bcrypt: $2a$, $2b$, $2y$ prefixes
//!    - scrypt: $7$ prefix
//!    - PBKDF2: $pbkdf2-sha256$, $pbkdf2-sha512$ prefixes
//!    - SHA2 (salted): Manually salted SHA256/SHA512 patterns
//!
//! 3. **Unsalted Hash Detection** - Identifies MD5/SHA without salts:
//!    - Pure MD5 hashes (32 hex chars)
//!    - Pure SHA1 hashes (40 hex chars)
//!    - Pure SHA256 hashes (64 hex chars)
//!    - Pure SHA512 hashes (128 hex chars)

use std::{borrow::Cow, collections::HashSet, sync::OnceLock};

/// Algorithm fingerprint for identifying hash types by pattern.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum HashAlgorithmFingerprint {
	/// SHA1 unsalted (40 hex chars)
	Sha1Unsalted,
	/// MD5 unsalted (32 hex chars)
	Md5Unsalted,
	/// SHA256 unsalted (64 hex chars)
	Sha256Unsalted,
	/// SHA512 unsalted (128 hex chars)
	Sha512Unsalted,
	/// bcrypt with $2a$, $2b$, or $2y$ prefix
	Bcrypt,
	/// scrypt with $7$ prefix
	Scrypt,
	/// Argon2 variants: $argon2id$, $argon2i$, $argon2d$
	Argon2,
	/// PBKDF2 with $pbkdf2-sha256$ or $pbkdf2-sha512$ prefix
	Pbkdf2,
}

/// Result of hash algorithm fingerprinting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FingerprintMatch {
	/// The detected algorithm fingerprint
	pub algorithm: HashAlgorithmFingerprint,
	/// Whether this is a weak/deprecated algorithm
	pub is_weak: bool,
	/// Description for logging/reporting
	pub description: &'static str,
}

/// A precomputed weak password hash entry for rainbow tabling.
/// Supports both static and dynamically-generated passwords.
/// Used for dictionary attacks on salted algorithms (bcrypt, Argon2, scrypt, PBKDF2).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct WeakPasswordHash {
	/// The original plaintext password
	pub plaintext: Cow<'static, str>,
	/// MD5 hash (32 hex chars) - for unsalted MD5 detection
	pub md5: Cow<'static, str>,
	/// SHA1 hash (40 hex chars) - for unsalted SHA1 detection
	pub sha1: Cow<'static, str>,
	/// SHA256 hash (64 hex chars) - for unsalted SHA256 detection
	pub sha256: Cow<'static, str>,
	/// SHA512 hash (128 hex chars) - for unsalted SHA512 detection
	pub sha512: Cow<'static, str>,
	/// NTLM hash (32 hex chars) - Windows NT password hashing (most common on Windows)
	pub ntlm: Cow<'static, str>,
}

/// Static rainbow table of weak passwords loaded from external JSON file.
static RAINBOW_TABLE: OnceLock<Vec<WeakPasswordHash>> = OnceLock::new();

/// Initialize the rainbow table.
/// This is called at startup by lib.rs.
/// Rainbow table data is now loaded from the SQLite database during ingest,
/// not from JSON files at startup.
pub fn initialize() -> Result<(), String> {
	// Initialize empty rainbow table; entries will be loaded from DB during ingest
	let _ = RAINBOW_TABLE.set(Vec::new());
	Ok(())
}

/// Populate the SQLite rainbow table database with weak password hashes.
/// This should be called after the database is initialized (e.g., during ingest setup).
/// Returns true if the table was regenerated, false if hashes were already current.
pub fn populate_database(conn: &SignedConnection) -> Result<bool, String> {
	use crate::enrichment::rainbow_table_builder::RainbowTableBuilder;

	let mut builder = RainbowTableBuilder::new();

	match builder.populate_database(conn) {
		Ok(was_regenerated) => {
			if was_regenerated {
				eprintln!("[INFO] Rainbow table database updated: wordlist files have changed");
			} else {
				eprintln!("[INFO] Rainbow table database is current (no changes detected)");
			}
			Ok(was_regenerated)
		}
		Err(e) => {
			eprintln!("[WARN] Failed to populate rainbow table database: {}", e);
			Err(format!("Failed to populate rainbow table database: {}", e))
		}
	}
}

/// Get or initialize the rainbow table.
fn get_rainbow_table() -> &'static Vec<WeakPasswordHash> {
	RAINBOW_TABLE.get_or_init(Vec::new)
}

/// Build an efficient lookup set of all weak password hashes.
/// This includes MD5 and SHA256 hashes for fast constant-time lookups.
fn get_weak_hash_set() -> &'static HashSet<String> {
	static WEAK_HASHES: OnceLock<HashSet<String>> = OnceLock::new();

	WEAK_HASHES.get_or_init(|| {
		let mut hashes = HashSet::new();
		for entry in get_rainbow_table() {
			hashes.insert(entry.md5.to_string());
			hashes.insert(entry.sha256.to_string());
		}
		hashes
	})
}

/// Check if a hash matches a known weak password (MD5 or SHA256).
/// Returns true if the hash is found in the rainbow table.
pub fn is_weak_password_hash(hash: &str) -> bool {
	get_weak_hash_set().contains(&hash.to_lowercase())
}

/// Get the plaintext password for a known weak password hash.
/// Returns the password if found, None otherwise.
pub fn get_weak_password_for_hash(hash: &str) -> Option<String> {
	let hash_lower = hash.to_lowercase();
	get_rainbow_table()
		.iter()
		.find(|entry| {
			entry.md5.to_lowercase() == hash_lower || entry.sha256.to_lowercase() == hash_lower
		})
		.map(|entry| entry.plaintext.to_string())
}

/// Identify the algorithm used for a hash by its fingerprint.
///
/// Supports detection of:
/// - Unsalted: MD5 (32 hex), SHA1 (40 hex), SHA256 (64 hex), SHA512 (128 hex)
/// - Salted: bcrypt, scrypt, Argon2, PBKDF2
/// - Unknown: Plaintext or unrecognized format
pub fn identify_hash_fingerprint(hash: &str) -> FingerprintMatch {
	let hash_upper = hash.to_uppercase();

	// Check for salted algorithms first (they have prefix markers)
	if hash_upper.starts_with("$2A$")
		|| hash_upper.starts_with("$2B$")
		|| hash_upper.starts_with("$2Y$")
	{
		return FingerprintMatch {
			algorithm: HashAlgorithmFingerprint::Bcrypt,
			is_weak: false,
			description: "bcrypt (strong, salted hash)",
		};
	}

	if hash_upper.starts_with("$7$") {
		return FingerprintMatch {
			algorithm: HashAlgorithmFingerprint::Scrypt,
			is_weak: false,
			description: "scrypt (strong, salted hash)",
		};
	}

	if hash_upper.starts_with("$ARGON2ID$")
		|| hash_upper.starts_with("$ARGON2I$")
		|| hash_upper.starts_with("$ARGON2D$")
	{
		return FingerprintMatch {
			algorithm: HashAlgorithmFingerprint::Argon2,
			is_weak: false,
			description: "Argon2 (strong, salted hash)",
		};
	}

	if hash_upper.starts_with("$PBKDF2-SHA256$") || hash_upper.starts_with("$PBKDF2-SHA512$") {
		return FingerprintMatch {
			algorithm: HashAlgorithmFingerprint::Pbkdf2,
			is_weak: false,
			description: "PBKDF2 (strong, salted hash)",
		};
	}

	// Check for unsalted weak hashes (by hex length)
	let hex_chars = hash.len();

	if hex_chars == 32 {
		return FingerprintMatch {
			algorithm: HashAlgorithmFingerprint::Md5Unsalted,
			is_weak: true,
			description: "MD5 unsalted (weak, deprecated)",
		};
	}

	if hex_chars == 40 {
		return FingerprintMatch {
			algorithm: HashAlgorithmFingerprint::Sha1Unsalted,
			is_weak: true,
			description: "SHA1 unsalted (weak, deprecated)",
		};
	}

	if hex_chars == 64 {
		return FingerprintMatch {
			algorithm: HashAlgorithmFingerprint::Sha256Unsalted,
			is_weak: true,
			description: "SHA256 unsalted (weak without salt)",
		};
	}

	if hex_chars == 128 {
		return FingerprintMatch {
			algorithm: HashAlgorithmFingerprint::Sha512Unsalted,
			is_weak: true,
			description: "SHA512 unsalted (weak without salt)",
		};
	}

	// If we can't identify it, it's likely plaintext or unknown format
	FingerprintMatch {
		algorithm: HashAlgorithmFingerprint::Sha256Unsalted,
		is_weak: false,
		description: "Unknown format (possibly plaintext)",
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_fingerprint_md5_unsalted() {
		let result = identify_hash_fingerprint("5f4dcc3b5aa765d61d8327deb882cf99");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Md5Unsalted);
		assert!(result.is_weak);
	}

	#[test]
	fn test_fingerprint_sha1_unsalted() {
		let result = identify_hash_fingerprint("5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Sha1Unsalted);
		assert!(result.is_weak);
	}

	#[test]
	fn test_fingerprint_sha256_unsalted() {
		let result = identify_hash_fingerprint(
			"5e884898da28047151d0e56f8dc62927051d5e07d5d91ff5e95b1fee3e06a717",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Sha256Unsalted);
		assert!(result.is_weak);
	}

	#[test]
	fn test_fingerprint_sha512_unsalted() {
		let result = identify_hash_fingerprint(
			"b109f3bbbc244eb82441917ed06d618b9008dd09b3befd1b5e07394c706a8bb980b1d7785e5976ec049b46df5f1326af5a2ea6d103fd07c95385ffab0cacbc86",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Sha512Unsalted);
		assert!(result.is_weak);
	}

	#[test]
	fn test_fingerprint_bcrypt() {
		let result = identify_hash_fingerprint("$2a$12$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMN");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Bcrypt);
		assert!(!result.is_weak);
	}

	#[test]
	fn test_fingerprint_scrypt() {
		let result =
			identify_hash_fingerprint("$7$C6..../....SodiumChloride$abcdefghijklmnopqrstuvwxyz");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Scrypt);
		assert!(!result.is_weak);
	}

	#[test]
	fn test_fingerprint_argon2() {
		let result =
			identify_hash_fingerprint("$argon2id$v=19$m=65536,t=3,p=4$abcdefghijkl$abcdefghijklmn");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Argon2);
		assert!(!result.is_weak);
	}

	#[test]
	fn test_fingerprint_pbkdf2() {
		let result = identify_hash_fingerprint(
			"$pbkdf2-sha256$260000$abcdefghijklmnop$abcdefghijklmnopqrstuvwxyz",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Pbkdf2);
		assert!(!result.is_weak);
	}

	#[test]
	fn test_fingerprint_case_insensitive() {
		let result_lower =
			identify_hash_fingerprint("$2a$12$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMN");
		let result_upper =
			identify_hash_fingerprint("$2A$12$ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMN");

		assert_eq!(result_lower.algorithm, result_upper.algorithm);
	}
}
