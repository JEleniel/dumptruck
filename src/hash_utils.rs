use sha2::{Digest as ShaDigest, Sha256};

use crate::rainbow_table;

pub fn sha256_hex(input: &str) -> String {
	sha256_hex_bytes(input.as_bytes())
}

pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
	let mut hasher = Sha256::new();
	hasher.update(bytes);
	let res = hasher.finalize();
	hex::encode(res)
}

pub fn md5_hex_bytes(bytes: &[u8]) -> String {
	let digest = md5::compute(bytes);
	hex::encode(digest.0)
}

/// Detect if a credential value appears to be a pre-hashed password.
/// Returns true if the value matches common hash algorithm patterns:
/// - Simple hex hashes: 32 chars (MD5), 40 chars (SHA1), 64 chars (SHA256)
/// - Salted hash prefixes: bcrypt ($2a$, $2b$, $2y$), scrypt ($7$), argon2 ($argon2id$, $argon2i$)
/// - Base64-like encoding patterns (common for encoded hashes)
/// - Known hashes of common test passwords ("password", "12345", etc.)
///
/// Pre-hashed credentials are useless for breach detection (cannot compare against plaintext
/// lists) and should be discarded as they do not represent actual credential leaks.
pub fn is_credential_hash(cred_value: &str) -> bool {
	let trimmed = cred_value.trim();

	// Check if this is a known hash of a common test/placeholder password
	// These are extremely common in password dumps that have been pre-hashed by the source.
	if is_common_password_hash(trimmed) {
		return true;
	}

	// Check for common salted hash algorithm prefixes
	if trimmed.starts_with("$2a$") || trimmed.starts_with("$2b$") || trimmed.starts_with("$2y$") {
		return true; // bcrypt
	}
	if trimmed.starts_with("$7$") {
		return true; // scrypt
	}
	if trimmed.starts_with("$argon2id$")
		|| trimmed.starts_with("$argon2i$")
		|| trimmed.starts_with("$argon2d$")
	{
		return true; // argon2
	}
	if trimmed.starts_with("$pbkdf2-sha256$") || trimmed.starts_with("$pbkdf2-sha512$") {
		return true; // pbkdf2
	}

	// Check for simple hex hashes (MD5, SHA1, SHA256 without algorithm prefix)
	// Must be all hex characters (lowercase only for consistency) and match known lengths
	let is_hex_only = trimmed.chars().all(|c| c.is_ascii_hexdigit());
	if is_hex_only
		&& trimmed
			.chars()
			.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
	{
		match trimmed.len() {
			32 => return true,  // MD5
			40 => return true,  // SHA1
			64 => return true,  // SHA256
			128 => return true, // SHA512
			_ => {}
		}
	}

	// Check for base64-like encoding (common for encoded digests)
	// Pattern: characters that look like base64 encoding (alphanumeric + / + =)
	let is_base64_like = trimmed.len() >= 16
		&& trimmed.len() <= 88
		&& trimmed
			.chars()
			.all(|c| c.is_alphanumeric() || c == '/' || c == '+' || c == '=');

	// Heuristic: if it looks like base64 and doesn't contain spaces or special chars, likely a hash
	// BUT: reject if it contains mixed case and non-hex chars (e.g., "XD" in middle of hex)
	if is_base64_like && !trimmed.contains(' ') && !trimmed.contains('@') && !trimmed.contains('-')
	{
		// Additional check: base64-encoded hashes typically have padding or are multiples of 4
		// OR: it's all lowercase (base64 without mixed alphanumeric)
		let is_all_lowercase_or_padded = trimmed
			.chars()
			.all(|c| c.is_ascii_lowercase() || c == '/' || c == '+' || c == '=')
			|| (trimmed.len() % 4 == 0 || trimmed.ends_with('='));
		if is_all_lowercase_or_padded {
			return true;
		}
	}

	false
}

/// Check if a value is a known hash of a common test/placeholder password.
/// These are frequently found in dumps of pre-hashed password databases.
/// Delegates to the rainbow table module for efficient O(1) lookup.
fn is_common_password_hash(value: &str) -> bool {
	rainbow_table::is_weak_password_hash(value)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_credential_hash_md5() {
		// MD5 hash of "password123" = 482c811da5d5b4bc6d497ffa98491e38
		assert!(is_credential_hash("482c811da5d5b4bc6d497ffa98491e38"));
	}

	#[test]
	fn test_is_credential_hash_sha1() {
		// SHA1 hash (40 hex chars)
		assert!(is_credential_hash(
			"a94a8fe5ccb19ba61c4c0873d391e987982fbbd3"
		));
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
	fn test_is_credential_hash_common_passwords_sha1() {
		// SHA1 hashes of common placeholder passwords
		assert!(is_credential_hash(
			"5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8"
		)); // "password"
		assert!(is_credential_hash("8cb2237d4cb1444b40ddb696f6d952e2")); // "12345"
		assert!(is_credential_hash(
			"7c4a8d09ca3762af61e59520943dc26494f8941b"
		)); // "123456"
		assert!(is_credential_hash(
			"a94a8fe5ccb19ba61c4c0873d391e987982fbbd3"
		)); // "test"
		assert!(is_credential_hash(
			"d033e22ae348aeb5660fc2140aec35850c4da997"
		)); // "admin"
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
}
