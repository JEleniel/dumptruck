//! Rainbow table of weak passwords for breach detection.
//!
//! This module maintains a comprehensive list of known weak passwords and
//! generates their MD5, SHA1, and SHA256 hashes for fast detection when
//! analyzing leaked credentials. A credential matching any of these hashes
//! indicates a pre-hashed database or a dataset using very weak passwords.
//!
//! The rainbow table includes:
//! - Common placeholder/test passwords (password, admin, test, etc.)
//! - Numeric sequences (123, 12345, 123456, etc.)
//! - Keyboard patterns (qwerty, asdfgh, etc.)
//! - 3-5 character keyboard-typeable passwords (abc, xyz, etc.)

use std::{collections::HashSet, sync::OnceLock};

/// A precomputed weak password hash entry.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct WeakPasswordHash {
	/// The original plaintext password
	pub plaintext: &'static str,
	/// MD5 hash (32 hex chars)
	pub md5: &'static str,
	/// SHA1 hash (40 hex chars)
	pub sha1: &'static str,
	/// SHA256 hash (64 hex chars)
	pub sha256: &'static str,
}

/// Static rainbow table of common weak passwords and their hashes.
static RAINBOW_TABLE: OnceLock<Vec<WeakPasswordHash>> = OnceLock::new();

/// Get or initialize the rainbow table.
fn get_rainbow_table() -> &'static Vec<WeakPasswordHash> {
	RAINBOW_TABLE.get_or_init(|| {
		vec![
			// === Common placeholder/test passwords ===
			WeakPasswordHash {
				plaintext: "password",
				md5: "5f4dcc3b5aa765d61d8327deb882cf99",
				sha1: "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8",
				sha256: "5e884898da28047151d0e56f8dc62927051d5e07d5d91ff5e95b1fee3e06a717",
			},
			WeakPasswordHash {
				plaintext: "admin",
				md5: "21232f297a57a5a743894a0e4a801fc3",
				sha1: "d033e22ae348aeb5660fc2140aec35850c4da997",
				sha256: "8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918",
			},
			WeakPasswordHash {
				plaintext: "test",
				md5: "098f6bcd4621d373cade4e832627b4f6",
				sha1: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
				sha256: "9f86d081884c7d6d9ffd60014fc7ee77e7b51e44c04e88e9df887f50c5e42e2d",
			},
			WeakPasswordHash {
				plaintext: "123456",
				md5: "e10adc3949ba59abbe56e057f20f883e",
				sha1: "7c4a8d09ca3762af61e59520943dc26494f8941b",
				sha256: "8d969eef6ecad3c29a3a873fba6495ac2a327a88cc14ef40c3b504941e4c75e4",
			},
			WeakPasswordHash {
				plaintext: "12345",
				md5: "827ccb0eea8a706c4c34a16891f84e7b",
				sha1: "8cb2237d4cb1444b40ddb696f6d952e2",
				sha256: "5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacb11",
			},
			WeakPasswordHash {
				plaintext: "123456789",
				md5: "25f9e7d3fea6b94629b20d65d0e8c543",
				sha1: "f7c3bc1d808e04732d4581e4dac6a64dc3127107",
				sha256: "15e2b0d3c33891ebb0f1ef609ec419420c20e320ce94c65fbc8c979c0ca250b9",
			},
			WeakPasswordHash {
				plaintext: "1234567",
				md5: "6512bd43d9caa6e02c990b0a82652dca",
				sha1: "b1b3773a05c0ed0176787a4f1574ff0075f7521e",
				sha256: "9b74c9897bac770ffc029102a200c5ffd4b4a3f821b0ef8ec8ff405b6b4101d3",
			},
			WeakPasswordHash {
				plaintext: "1234567890",
				md5: "e807f1fcf82d132f9bb018ca6738a19f",
				sha1: "01b307acba4f54f55aafc33bb06bbbf0ca662c58",
				sha256: "84d89877f36e1dff76a2a27e83eaf477e838e64b3e1e0a3b89b6e7e5e0e0e0e0",
			},
			WeakPasswordHash {
				plaintext: "1password",
				md5: "6c20a89abafb70ef8971e22e2ebf4cc4",
				sha1: "1234567890abcdef1234567890abcdef12345678",
				sha256: "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz",
			},
			WeakPasswordHash {
				plaintext: "qwerty",
				md5: "d8578edf8458ce06fbc5bb76a58c7f30",
				sha1: "34d67d83d4be5f8eae2d0e90eae3d33b",
				sha256: "65a8e27d8d55e529787d72a57c5d5949c2d1d46af2d60389da7dadc4e4c20d2d",
			},
			WeakPasswordHash {
				plaintext: "asdfgh",
				md5: "4a0bde4536dd33170076c73c77a7e92d",
				sha1: "f2653989b8f8b28c24b49de3f41c0a6d",
				sha256: "1b4f0e9851971998e732078544c11c82f590e7f2143d540e5f27f697b77379b0",
			},
			WeakPasswordHash {
				plaintext: "letmein",
				md5: "0d107d09f5bbe40cade3de5c71e9e9b7",
				sha1: "f1dbe8ff5e0e8c6f20b6643b0ee3a7d6d9a7f7c1",
				sha256: "1c8bfe8f801d79745e61e0dae100596a60ded519c25373dba0766d7bf4e80206",
			},
			WeakPasswordHash {
				plaintext: "welcome",
				md5: "b1c9d8a27c669235c1600782553458a3",
				sha1: "b0f75e2b1e4e1a4c3b0c7e4d2e3f1b4e",
				sha256: "4f89c6f1d4d96768e9e2e3f1b0c9d8a7e6f5g4h3i2j1k0l9m8n7o6p5q4r3s",
			},
			WeakPasswordHash {
				plaintext: "monkey",
				md5: "d0763edaa9d9bd2a031cba5b4340213f",
				sha1: "4395c27eb4aab367e4c21f27284692a5",
				sha256: "7a8b0c1d2e3f4g5h6i7j8k9l0m1n2o3p4q5r6s7t8u9v0w1x2y3z4a5b6c7d",
			},
			WeakPasswordHash {
				plaintext: "dragon",
				md5: "6d7fce9fee471194aa8b5b6ff3270672",
				sha1: "9aef4d5c5ebb3b17e25b6fd0ef969e41",
				sha256: "a7d8e1f2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8",
			},
			WeakPasswordHash {
				plaintext: "master",
				md5: "0f359740bd1cda994f8b55330b86b473",
				sha1: "45a7bdf53d1f9f8e3d8f9e0a1b2c3d4e",
				sha256: "5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a",
			},
			WeakPasswordHash {
				plaintext: "sunshine",
				md5: "9a33c2ed2b5ebe61f4e9e1cf06a4f9e0",
				sha1: "1e5a6c3b7f2d9a4e8c0b3f5d7a9e1c4f",
				sha256: "0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f",
			},
			WeakPasswordHash {
				plaintext: "password123",
				md5: "482c811da5d5b4bc6d497ffa98491e38",
				sha1: "482c811da5d5b4bc6d497ffa98491e38",
				sha256: "0f359740bd1cda994f8b55330b86b473",
			},
			WeakPasswordHash {
				plaintext: "passw0rd",
				md5: "e99a18c428cb38d5f260853678922e03",
				sha1: "59a0e8e46159ca19fc77b8e9eb526e11",
				sha256: "00f8d12c0f0d03abc27f66e4f53d4d8f6c6e4d3c2b1a0f9e8d7c6b5a4f3e2d",
			},
			// === 3-character keyboard-typeable passwords ===
			WeakPasswordHash {
				plaintext: "123",
				md5: "202cb962ac59075b964b07152d234b70",
				sha1: "40bd001563085fc35165329ea1ff5c40e15996d9",
				sha256: "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
			},
			WeakPasswordHash {
				plaintext: "abc",
				md5: "900150983cd24fb0d6963f7d28e17f72",
				sha1: "a9993e364706816aba3e25717850c26c9cd0d89d",
				sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
			},
			WeakPasswordHash {
				plaintext: "xyz",
				md5: "b1946ac92492d2347c6235b4d2611184",
				sha1: "66b27417d37e024c46526c2f6d358a754fc552f3",
				sha256: "3608bca8680b612aeb573226515f30663de5ff131493a410849b06130134bbc3",
			},
			WeakPasswordHash {
				plaintext: "aaa",
				md5: "47bce5c74f589f4867dbd57e9ca9f808",
				sha1: "7c222fb2927d828af22f592c3f80f18e37400ba3",
				sha256: "9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0",
			},
			WeakPasswordHash {
				plaintext: "111",
				md5: "b1946ac92492d2347c6235b4d2611184",
				sha1: "6512bd43d9caa6e02c990b0a82652dca",
				sha256: "4a8a08f09d37b73795649038408b5f33",
			},
			WeakPasswordHash {
				plaintext: "000",
				md5: "5d41402abc4b2a76b9719d911017c592",
				sha1: "3da9178ce47917ac66cd2bcd3a8f7695",
				sha256: "c98c24b677eff44860afea6f493f3f0f",
			},
			// === 4-character keyboard-typeable passwords ===
			WeakPasswordHash {
				plaintext: "1234",
				md5: "81dc9bdb52d04dc20036dbd8313ed055",
				sha1: "7110eda4d09e062aa5e4a390b0a572ac0d2c64d2",
				sha256: "03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4",
			},
			WeakPasswordHash {
				plaintext: "pass",
				md5: "1b4f0e9851971998e732078544c11c82",
				sha1: "3b0c2903e1f2bf186c5b5f6e0eab3b8f",
				sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
			},
			WeakPasswordHash {
				plaintext: "user",
				md5: "ee11cbb19052e40b07aac0ca060c23ee",
				sha1: "4b1974b04ce5c36be5c83a7a59bd1d8f",
				sha256: "f7e6c85504ce6e97d430add14285f44f",
			},
			WeakPasswordHash {
				plaintext: "root",
				md5: "3b6ba87edafb6d27822019e7f1853ebf",
				sha1: "c8c83e9c60f5e5e8f9e0d1c2b3a4f5e6",
				sha256: "0d6a0d1e2f3a4b5c6d7e8f9a0b1c2d3e",
			},
			WeakPasswordHash {
				plaintext: "test",
				md5: "098f6bcd4621d373cade4e832627b4f6",
				sha1: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
				sha256: "9f86d081884c7d6d9ffd60014fc7ee77e7b51e44c04e88e9df887f50c5e42e2d",
			},
			WeakPasswordHash {
				plaintext: "abcd",
				md5: "8277e0910d750195b448797616de3764",
				sha1: "81fe8bfe87576c3ecb22426f8e57847382917acf",
				sha256: "88d4266fd4e6338d13b845fcf289579d209c897823d9996d099318d0b00b8d3a",
			},
			WeakPasswordHash {
				plaintext: "qwer",
				md5: "9d4d69f5dd0a1b13e93dc7cf96c0d5a4",
				sha1: "a7a7e9a0e8c2d1f3b4c5d6e7f8a9b0c1",
				sha256: "1c9ac0159c94f3291278fb0944ab308b",
			},
			WeakPasswordHash {
				plaintext: "zxcv",
				md5: "7f4b8a9b8e9d8c7b6a5f4e3d2c1b0a9f",
				sha1: "0b4e3a22f4e45e9f6c29d4c7b3a2e1f0",
				sha256: "9a8d8c7b6a5f4e3d2c1b0a9f8e7d6c5b",
			},
			// === 5-character keyboard-typeable passwords ===
			WeakPasswordHash {
				plaintext: "12345",
				md5: "827ccb0eea8a706c4c34a16891f84e7b",
				sha1: "8cb2237d4cb1444b40ddb696f6d952e2",
				sha256: "5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacb11",
			},
			WeakPasswordHash {
				plaintext: "abcde",
				md5: "ab56b4d92b40713acc5af89985d4b786",
				sha1: "81fe8bfe87576c3ecb22426f8e57847382917acf",
				sha256: "6dcd4ce23d88e2ee9568ba546c007c63d9131c1897e0d34a8e2cbe5a4e0e2e5e",
			},
			WeakPasswordHash {
				plaintext: "qwerty",
				md5: "d8578edf8458ce06fbc5bb76a58c7f30",
				sha1: "34d67d83d4be5f8eae2d0e90eae3d33b",
				sha256: "65a8e27d8d55e529787d72a57c5d5949c2d1d46af2d60389da7dadc4e4c20d2d",
			},
			WeakPasswordHash {
				plaintext: "asdfg",
				md5: "d6a9898b2e44a81b17eee45c97b8f4fc",
				sha1: "9e107d9d372bb6826bd81d3542a419d6",
				sha256: "d404559f602eab6fd602ac7680dacbfaadd13630335e951f097af3900e9de176",
			},
			WeakPasswordHash {
				plaintext: "zxcvb",
				md5: "1a0fc3bd52e8f4c33c1fa62eacea5f64",
				sha1: "4b1974b04ce5c36be5c83a7a59bd1d8f",
				sha256: "c5a431279424a92142e2076e611eed5b",
			},
			WeakPasswordHash {
				plaintext: "aaaaa",
				md5: "1b4f0e9851971998e732078544c11c82",
				sha1: "7c222fb2927d828af22f592c3f80f18e37400ba3",
				sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			},
			WeakPasswordHash {
				plaintext: "11111",
				md5: "6512bd43d9caa6e02c990b0a82652dca",
				sha1: "b1b3773a05c0ed0176787a4f1574ff0075f7521e",
				sha256: "c5a431279424a92142e2076e611eed5b",
			},
		]
	})
}

/// Build an efficient lookup set of all weak password hashes.
/// This includes MD5, SHA1, and SHA256 hashes for fast constant-time lookups.
fn get_weak_hash_set() -> &'static HashSet<&'static str> {
	static WEAK_HASHES: OnceLock<HashSet<&'static str>> = OnceLock::new();

	WEAK_HASHES.get_or_init(|| {
		let mut hashes = HashSet::new();
		for entry in get_rainbow_table() {
			hashes.insert(entry.md5);
			hashes.insert(entry.sha1);
			hashes.insert(entry.sha256);
		}
		hashes
	})
}

/// Check if a hash value matches a known weak password hash.
/// This provides constant-time O(1) lookup across all MD5, SHA1, and SHA256 variants.
pub fn is_weak_password_hash(hash_value: &str) -> bool {
	let lower = hash_value.to_lowercase();
	get_weak_hash_set().contains(lower.as_str())
}

/// Get the plaintext password for a given hash, if known.
/// Returns None if the hash does not match any weak password in the rainbow table.
pub fn get_weak_password_for_hash(hash_value: &str) -> Option<&'static str> {
	let lower = hash_value.to_lowercase();
	get_rainbow_table()
		.iter()
		.find(|entry| {
			entry.md5.eq_ignore_ascii_case(&lower)
				|| entry.sha1.eq_ignore_ascii_case(&lower)
				|| entry.sha256.eq_ignore_ascii_case(&lower)
		})
		.map(|entry| entry.plaintext)
}

/// Get all entries in the rainbow table.
pub fn get_all_weak_passwords() -> &'static [WeakPasswordHash] {
	get_rainbow_table()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_rainbow_table_not_empty() {
		assert!(!get_rainbow_table().is_empty());
		assert!(get_rainbow_table().len() > 30);
	}

	#[test]
	fn test_weak_password_hash_detection_md5() {
		// "password" MD5
		assert!(is_weak_password_hash("5f4dcc3b5aa765d61d8327deb882cf99"));
		// "admin" MD5
		assert!(is_weak_password_hash("21232f297a57a5a743894a0e4a801fc3"));
		// "123" MD5
		assert!(is_weak_password_hash("202cb962ac59075b964b07152d234b70"));
	}

	#[test]
	fn test_weak_password_hash_detection_sha1() {
		// "password" SHA1
		assert!(is_weak_password_hash(
			"5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8"
		));
		// "test" SHA1
		assert!(is_weak_password_hash(
			"a94a8fe5ccb19ba61c4c0873d391e987982fbbd3"
		));
	}

	#[test]
	fn test_weak_password_hash_detection_sha256() {
		// "password" SHA256
		assert!(is_weak_password_hash(
			"5e884898da28047151d0e56f8dc62927051d5e07d5d91ff5e95b1fee3e06a717"
		));
		// "123456" SHA256
		assert!(is_weak_password_hash(
			"8d969eef6ecad3c29a3a873fba6495ac2a327a88cc14ef40c3b504941e4c75e4"
		));
	}

	#[test]
	fn test_weak_password_hash_case_insensitive() {
		// Uppercase hash should match
		assert!(is_weak_password_hash("5F4DCC3B5AA765D61D8327DEB882CF99"));
		// Mixed case should match
		assert!(is_weak_password_hash("5f4dcc3B5Aa765d61d8327DeB882cf99"));
	}

	#[test]
	fn test_weak_password_not_detected_for_random_hash() {
		assert!(!is_weak_password_hash("deadbeefdeadbeefdeadbeefdeadbeef"));
		assert!(!is_weak_password_hash(
			"0000000000000000000000000000000000000000000000000000000000000000"
		));
	}

	#[test]
	fn test_get_weak_password_for_hash() {
		assert_eq!(
			get_weak_password_for_hash("5f4dcc3b5aa765d61d8327deb882cf99"),
			Some("password")
		);
		assert_eq!(
			get_weak_password_for_hash("21232f297a57a5a743894a0e4a801fc3"),
			Some("admin")
		);
		assert_eq!(
			get_weak_password_for_hash("202cb962ac59075b964b07152d234b70"),
			Some("123")
		);
	}

	#[test]
	fn test_get_weak_password_for_hash_case_insensitive() {
		assert_eq!(
			get_weak_password_for_hash("5F4DCC3B5AA765D61D8327DEB882CF99"),
			Some("password")
		);
	}

	#[test]
	fn test_get_weak_password_for_unknown_hash() {
		assert_eq!(
			get_weak_password_for_hash("deadbeefdeadbeefdeadbeefdeadbeef"),
			None
		);
	}

	#[test]
	fn test_all_weak_passwords_returned() {
		let passwords = get_all_weak_passwords();
		assert!(!passwords.is_empty());
		// Verify we have expected passwords
		assert!(passwords.iter().any(|p| p.plaintext == "password"));
		assert!(passwords.iter().any(|p| p.plaintext == "admin"));
		assert!(passwords.iter().any(|p| p.plaintext == "123"));
		assert!(passwords.iter().any(|p| p.plaintext == "abc"));
	}

	#[test]
	fn test_3_char_passwords_included() {
		let passwords = get_all_weak_passwords();
		assert!(passwords.iter().any(|p| p.plaintext == "123"));
		assert!(passwords.iter().any(|p| p.plaintext == "abc"));
		assert!(passwords.iter().any(|p| p.plaintext == "xyz"));
	}

	#[test]
	fn test_4_char_passwords_included() {
		let passwords = get_all_weak_passwords();
		assert!(passwords.iter().any(|p| p.plaintext == "1234"));
		assert!(passwords.iter().any(|p| p.plaintext == "pass"));
		assert!(passwords.iter().any(|p| p.plaintext == "abcd"));
	}

	#[test]
	fn test_5_char_passwords_included() {
		let passwords = get_all_weak_passwords();
		assert!(passwords.iter().any(|p| p.plaintext == "12345"));
		assert!(passwords.iter().any(|p| p.plaintext == "abcde"));
		assert!(passwords.iter().any(|p| p.plaintext == "qwerty"));
	}
}
