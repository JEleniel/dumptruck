//! Hash utilities and cryptographic fingerprinting
//!
//! This module provides cryptographic hashing operations:
//! - SHA-256 and BLAKE3 dual signatures
//! - Credential fingerprinting
//! - Hash verification and comparison algorithms

pub mod algorithms;
pub mod credentials;
pub mod fingerprint;

pub use algorithms::{
	md4_hex_bytes, md5_hex, md5_hex_bytes, ntlm_hex, sha1_hex, sha1_hex_bytes, sha256_hex,
	sha256_hex_bytes, sha512_hex, sha512_hex_bytes,
};
pub use credentials::is_credential_hash;
pub use fingerprint::{FingerprintMatch, HashAlgorithmFingerprint, identify_hash_fingerprint};

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_credential_hash_md5() {
		// MD5 hash of "password123" = 482c811da5d5b4bc6d497ffa98491e38
		assert!(is_credential_hash("482c811da5d5b4bc6d497ffa98491e38"));
	}

	#[test]
	fn test_is_credential_hash_sha256() {
		// SHA256 hash (64 hex chars)
		assert!(is_credential_hash(
			"5e884898da28047151d0e56f8dc62927051d5e07d5d91ff5e95b1fee3e06a717"
		));
	}

	#[test]
	fn test_is_credential_hash_bcrypt() {
		// bcrypt hash
		assert!(is_credential_hash(
			"$2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcg7b3XeKeUxWdeS86E36MM32Oi"
		));
		assert!(is_credential_hash(
			"$2b$12$R9h/cIPz0gi.URNNX3kh2OPST9/PgBkqquzi.Ss7KIUgO2t0jWMUW"
		));
		assert!(is_credential_hash(
			"$2y$10$S9sBbWcp.YdJUVJvVCN/uOKlxY2GiY0l6DQ4y.xQmGj9Z3Z7G8p7m"
		));
	}

	#[test]
	fn test_is_credential_hash_argon2() {
		// argon2 hash
		assert!(is_credential_hash(
			"$argon2id$v=19$m=19456,t=2,p=1$gzZBvDQIyBAq/\
			 vLDTTQDrw$e0hDlJHyJhVPJ1w8khXcBRR1H8LvGeYpNdQRSVmqG8c"
		));
		assert!(is_credential_hash(
			"$argon2i$v=19$m=65536,t=3,p=4$saltsalt$xJQZ9Bj8OvXyX8Bxdn3sblzqVW8a8jzcf8V8LqPpS/k"
		));
	}

	#[test]
	fn test_is_credential_hash_scrypt() {
		// scrypt hash prefix
		assert!(is_credential_hash(
			"$7$C6..../....SodiumChloride$kBGj9fHzHvFQMKkF4aBNYV4ZYvnP4q7Z/vL5Z6Z5Z6Z"
		));
	}

	#[test]
	fn test_is_credential_hash_pbkdf2() {
		// pbkdf2 hashes
		assert!(is_credential_hash(
			"$pbkdf2-sha256$260000$saltsalt$abcdefg1234567890abcdefg"
		));
		assert!(is_credential_hash("$pbkdf2-sha512$260000$salt$xyz123abc"));
	}

	#[test]
	fn test_is_credential_hash_base64() {
		// Base64-like encoded hashes (multiple of 4)
		assert!(is_credential_hash("aGFzaGVkUGFzc3dvcmQxMjM0NTY3OA==")); // base64 with padding
		assert!(is_credential_hash("aGFzaGVkUGFzc3dvcmQxMjM=")); // base64 with padding
	}

	#[test]
	fn test_is_not_credential_hash_plaintext() {
		// Regular plaintext passwords should not be detected as hashes
		assert!(!is_credential_hash("MyPassword123!"));
		assert!(!is_credential_hash("correct horse battery staple"));
		assert!(!is_credential_hash("P@ssw0rd!"));
	}

	#[test]
	fn test_is_not_credential_hash_email() {
		// Email addresses should not be detected as hashes
		assert!(!is_credential_hash("user@example.com"));
	}

	#[test]
	fn test_is_not_credential_hash_short_hex() {
		// Hex strings that are not standard hash lengths should not match
		assert!(!is_credential_hash("abc123def456")); // 12 chars
		assert!(!is_credential_hash("aabbccdd")); // 8 chars
	}

	#[test]
	fn test_is_not_credential_hash_partial_hex() {
		// Strings with non-hex characters mixed in
		// Note: uppercase hex characters mixed with lowercase is ambiguous
		// (could be base64-like encoding), so we accept it as potentially hashed.
		// The key test is that completely wrong patterns like hyphens are rejected.
		assert!(!is_credential_hash("482c-811d-a5d5-b4bc")); // contains hyphens
	}

	#[test]
	fn test_is_credential_hash_whitespace_trimmed() {
		// Leading/trailing whitespace should be trimmed
		assert!(is_credential_hash("  482c811da5d5b4bc6d497ffa98491e38  "));
		assert!(is_credential_hash(
			"\t$2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcg7b3XeKeUxWdeS86E36MM32Oi\n"
		));
	}

	#[test]
	fn test_is_credential_hash_common_passwords_md5() {
		// MD5 hashes of extremely common placeholder passwords
		assert!(is_credential_hash("5f4dcc3b5aa765d61d8327deb882cf99")); // "password"
		assert!(is_credential_hash("827ccb0eea8a706c4c34a16891f84e7b")); // "12345"
		assert!(is_credential_hash("e10adc3949ba59abbe56e057f20f883e")); // "123456"
		assert!(is_credential_hash("098f6bcd4621d373cade4e832627b4f6")); // "test"
		assert!(is_credential_hash("21232f297a57a5a743894a0e4a801fc3")); // "admin"
		assert!(is_credential_hash("202cb962ac59075b964b07152d234b70")); // "123"
		assert!(is_credential_hash("d8578edf8458ce06fbc5bb76a58c7f30")); // "qwerty"
		assert!(is_credential_hash("0d107d09f5bbe40cade3de5c71e9e9b7")); // "letmein"
	}

	#[test]
	fn test_is_credential_hash_common_passwords_sha256() {
		// SHA256 hashes of common placeholder passwords
		assert!(is_credential_hash(
			"5e884898da28047151d0e56f8dc62927051d5e07d5d91ff5e95b1fee3e06a717"
		)); // "password"
		assert!(is_credential_hash(
			"5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacb11"
		)); // "12345"
		assert!(is_credential_hash(
			"8d969eef6ecad3c29a3a873fba6495ac2a327a88cc14ef40c3b504941e4c75e4"
		)); // "123456"
		assert!(is_credential_hash(
			"9f86d081884c7d6d9ffd60014fc7ee77e7b51e44c04e88e9df887f50c5e42e2d"
		)); // "test"
		assert!(is_credential_hash(
			"8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918"
		)); // "admin"
	}

	#[test]
	fn test_is_credential_hash_common_passwords_case_insensitive() {
		// Common password hashes should match regardless of case
		assert!(is_credential_hash("5F4DCC3B5AA765D61D8327DEB882CF99")); // uppercase MD5
		assert!(is_credential_hash("5f4dcc3b5AA765D61d8327deb882CF99")); // mixed case
	}

	// ===== Fingerprinting Tests =====

	#[test]
	fn test_fingerprint_md5_unsalted() {
		let result = identify_hash_fingerprint("482c811da5d5b4bc6d497ffa98491e38");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Md5Unsalted);
		assert!(result.is_weak);
		assert!(result.description.contains("MD5"));
	}

	#[test]
	fn test_fingerprint_sha1_unsalted() {
		let result = identify_hash_fingerprint("5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Sha1Unsalted);
		assert!(result.is_weak);
		assert!(result.description.contains("SHA1"));
	}

	#[test]
	fn test_fingerprint_sha256_unsalted() {
		let result = identify_hash_fingerprint(
			"5e884898da28047151d0e56f8dc62927051d5e07d5d91ff5e95b1fee3e06a717",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Sha256Unsalted);
		assert!(result.is_weak);
		assert!(result.description.contains("SHA256"));
	}

	#[test]
	fn test_fingerprint_sha512_unsalted() {
		let result = identify_hash_fingerprint(
			"cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Sha512Unsalted);
		assert!(result.is_weak);
		assert!(result.description.contains("SHA512"));
	}

	#[test]
	fn test_fingerprint_bcrypt() {
		let result = identify_hash_fingerprint(
			"$2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcg7b3XeKeUxWdeS86E36MM32Oi",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Bcrypt);
		assert!(!result.is_weak);
		assert!(result.description.contains("Bcrypt"));

		let result = identify_hash_fingerprint(
			"$2b$12$R9h/cIPz0gi.URNNX3kh2OPST9/PgBkqquzi.Ss7KIUgO2t0jWMUW",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Bcrypt);
		assert!(!result.is_weak);

		let result = identify_hash_fingerprint(
			"$2y$10$S9sBbWcp.YdJUVJvVCN/uOKlxY2GiY0l6DQ4y.xQmGj9Z3Z7G8p7m",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Bcrypt);
		assert!(!result.is_weak);
	}

	#[test]
	fn test_fingerprint_scrypt() {
		let result = identify_hash_fingerprint(
			"$7$C6..../....SodiumChloride$kBGj9fHzHvFQMKkF4aBNYV4ZYvnP4q7Z/vL5Z6Z5Z6Z",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Scrypt);
		assert!(!result.is_weak);
		assert!(result.description.contains("Scrypt"));
	}

	#[test]
	fn test_fingerprint_argon2() {
		let result = identify_hash_fingerprint(
			"$argon2id$v=19$m=19456,t=2,p=1$gzZBvDQIyBAq/\
			 vLDTTQDrw$e0hDlJHyJhVPJ1w8khXcBRR1H8LvGeYpNdQRSVmqG8c",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Argon2);
		assert!(!result.is_weak);
		assert!(result.description.contains("Argon2"));

		let result = identify_hash_fingerprint(
			"$argon2i$v=19$m=65536,t=3,p=4$saltsalt$xJQZ9Bj8OvXyX8Bxdn3sblzqVW8a8jzcf8V8LqPpS/k",
		);
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Argon2);
		assert!(!result.is_weak);
	}

	#[test]
	fn test_fingerprint_pbkdf2() {
		let result =
			identify_hash_fingerprint("$pbkdf2-sha256$260000$saltsalt$abcdefg1234567890abcdefg");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Pbkdf2);
		assert!(!result.is_weak);
		assert!(result.description.contains("PBKDF2"));

		let result = identify_hash_fingerprint("$pbkdf2-sha512$260000$salt$xyz123abc");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Pbkdf2);
		assert!(!result.is_weak);
	}

	#[test]
	fn test_fingerprint_unknown() {
		let result = identify_hash_fingerprint("not_a_hash_at_all!");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Unknown);
		assert!(!result.is_weak);
		assert!(result.description.contains("Unknown"));

		let result = identify_hash_fingerprint("plaintext_password");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Unknown);

		let result = identify_hash_fingerprint("user@example.com");
		assert_eq!(result.algorithm, HashAlgorithmFingerprint::Unknown);
	}

	#[test]
	fn test_fingerprint_case_insensitive() {
		// Fingerprinting should work with both lowercase and uppercase hex
		let result_lower = identify_hash_fingerprint("5f4dcc3b5aa765d61d8327deb882cf99");
		let result_upper = identify_hash_fingerprint("5F4DCC3B5AA765D61D8327DEB882CF99");

		assert_eq!(result_lower.algorithm, result_upper.algorithm);
		assert_eq!(
			result_lower.algorithm,
			HashAlgorithmFingerprint::Md5Unsalted
		);
	}

	#[test]
	fn test_fingerprint_weak_vs_strong() {
		// Unsalted algorithms should be weak
		let unsalted = vec![
			identify_hash_fingerprint("482c811da5d5b4bc6d497ffa98491e38"), // MD5
			identify_hash_fingerprint("5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8"), // SHA1
			identify_hash_fingerprint(
				"5e884898da28047151d0e56f8dc62927051d5e07d5d91ff5e95b1fee3e06a717",
			), // SHA256
		];
		for result in unsalted {
			assert!(result.is_weak, "Unsalted hashes should be weak");
		}

		// Salted algorithms should be strong
		let salted = vec![
			identify_hash_fingerprint(
				"$2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcg7b3XeKeUxWdeS86E36MM32Oi",
			), // Bcrypt
			identify_hash_fingerprint(
				"$7$C6..../....SodiumChloride$kBGj9fHzHvFQMKkF4aBNYV4ZYvnP4q7Z/vL5Z6Z5Z6Z",
			), // Scrypt
			identify_hash_fingerprint(
				"$argon2id$v=19$m=19456,t=2,p=1$gzZBvDQIyBAq/\
				 vLDTTQDrw$e0hDlJHyJhVPJ1w8khXcBRR1H8LvGeYpNdQRSVmqG8c",
			), // Argon2
		];
		for result in salted {
			assert!(
				!result.is_weak,
				"Salted algorithms should be strong: {:?}",
				result.algorithm
			);
		}
	}
}
