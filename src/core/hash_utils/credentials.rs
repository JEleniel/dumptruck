//! Credential hash detection and common password hash lookup.

use crate::detection::rainbow_table;

/// Detect if credential value appears to be pre-hashed.
pub fn is_credential_hash(cred_value: &str) -> bool {
	let trimmed = cred_value.trim();

	// Check known common password hashes
	if is_common_password_hash(trimmed) {
		return true;
	}

	// Check for salted hash prefixes
	if check_salted_hash_prefix(trimmed) {
		return true;
	}

	// Check for hex hashes
	if check_hex_hash(trimmed) {
		return true;
	}

	// Check for base64-like encoding
	check_base64_hash(trimmed)
}

fn check_salted_hash_prefix(s: &str) -> bool {
	matches!(s.chars().next(), Some('$'))
		&& (s.starts_with("$2a$")
			|| s.starts_with("$2b$")
			|| s.starts_with("$2y$")
			|| s.starts_with("$7$")
			|| s.starts_with("$argon2id$")
			|| s.starts_with("$argon2i$")
			|| s.starts_with("$argon2d$")
			|| s.starts_with("$pbkdf2-sha256$")
			|| s.starts_with("$pbkdf2-sha512$"))
}

fn check_hex_hash(s: &str) -> bool {
	let is_hex_only = s.chars().all(|c| c.is_ascii_hexdigit());
	let is_lowercase = s
		.chars()
		.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());

	is_hex_only && is_lowercase && matches!(s.len(), 32 | 64 | 128)
}

fn check_base64_hash(s: &str) -> bool {
	if s.len() < 16 || s.len() > 88 {
		return false;
	}

	let is_base64_like = s
		.chars()
		.all(|c| c.is_alphanumeric() || c == '/' || c == '+' || c == '=');

	if !is_base64_like || s.contains(' ') || s.contains('@') || s.contains('-') {
		return false;
	}

	let is_all_lowercase_or_padded = s
		.chars()
		.all(|c| c.is_ascii_lowercase() || c == '/' || c == '+' || c == '=')
		|| (s.len() % 4 == 0 || s.ends_with('='));

	is_all_lowercase_or_padded
}

fn is_common_password_hash(trimmed: &str) -> bool {
	rainbow_table::is_weak_password_hash(trimmed)
}
