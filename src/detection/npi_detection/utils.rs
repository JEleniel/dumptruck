//! Helper normalization and hashing utilities used by the NPI/PII detection module.

use crate::core::hash_utils;
use crate::normalization;

/// Normalize an IP address
pub fn normalize_ip(value: &str) -> String {
	value.trim().to_lowercase()
}

/// Normalize a name
pub fn normalize_name(value: &str) -> String {
	normalization::normalize_field(value)
}

/// Normalize a mailing address
pub fn normalize_address(value: &str) -> String {
	normalization::normalize_field(value)
}

// --- Private helpers -------------------------------------------------

fn digits_only(value: &str) -> String {
	value.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn remove_whitespace_hyphen_slash_upper(value: &str) -> String {
	value
		.chars()
		.filter(|c| !c.is_whitespace() && *c != '-' && *c != '/')
		.collect::<String>()
		.to_uppercase()
}

fn remove_spaces_and_hyphens_upper(value: &str) -> String {
	value
		.chars()
		.filter(|c| !c.is_whitespace() && *c != '-')
		.collect::<String>()
		.to_uppercase()
}

fn normalize_lower_trim(value: &str) -> String {
	value.trim().to_lowercase()
}

// --- Hash wrappers --------------------------------------------------

/// Hash a phone number
pub fn hash_phone_number(value: &str) -> String {
	let normalized = digits_only(value);
	hash_utils::sha256_hex(&normalized)
}

/// Hash a credit card number
pub fn hash_credit_card(value: &str) -> String {
	let digits = digits_only(value);
	if digits.len() < 4 {
		return hash_utils::sha256_hex(&digits);
	}
	let last_four = &digits[digits.len() - 4..];
	let masked = format!("{}_{}", last_four, digits.len());
	hash_utils::sha256_hex(&masked)
}

/// Hash a national ID number
pub fn hash_national_id(value: &str) -> String {
	let normalized = remove_whitespace_hyphen_slash_upper(value);
	hash_utils::sha256_hex(&normalized)
}

/// Hash a Social Security Number
pub fn hash_ssn(value: &str) -> String {
	let normalized = digits_only(value);
	hash_utils::sha256_hex(&normalized)
}

/// Hash an IBAN
pub fn hash_iban(value: &str) -> String {
	let normalized = remove_spaces_and_hyphens_upper(value);
	hash_utils::sha256_hex(&normalized)
}

/// Hash a SWIFT code
pub fn hash_swift_code(value: &str) -> String {
	let normalized = value
		.chars()
		.filter(|c| *c != '-')
		.collect::<String>()
		.to_uppercase();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a routing number
pub fn hash_routing_number(value: &str) -> String {
	let normalized = digits_only(value);
	hash_utils::sha256_hex(&normalized)
}

/// Hash a bank account number
pub fn hash_bank_account(value: &str) -> String {
	let normalized = digits_only(value);
	hash_utils::sha256_hex(&normalized)
}

/// Hash a cryptocurrency address
pub fn hash_crypto_address(value: &str) -> String {
	let normalized = normalize_lower_trim(value);
	hash_utils::sha256_hex(&normalized)
}

/// Hash a digital wallet token
pub fn hash_digital_wallet_token(value: &str) -> String {
	let normalized = normalize_lower_trim(value);
	hash_utils::sha256_hex(&normalized)
}
