//! Detection and normalization of Personally Identifiable Information (PII) and Non-Public Information (NPI).
//!
//! This module identifies and normalizes:
//! - IP addresses (IPv4 and IPv6)
//! - Names (people's names)
//! - Mailing addresses
//! - Phone numbers
//! - Social Security Numbers
//! - National IDs (international formats from 15+ countries)
//! - Credit card numbers
//! - Bank account numbers and financial identifiers (IBAN, SWIFT, routing numbers)
//! - Cryptocurrency addresses (Bitcoin, Ethereum, XRP, and others)
//! - Digital wallet tokens (Stripe, Square, PayPal, Apple Pay, Google Pay)
//!
//! Each detection function includes hashing capabilities for duplicate identification while preserving privacy.
//! Other potential NPI/PII fields for analyst review

/// Types of PII/NPI that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PiiType {
	Email,
	IpAddress,
	IpV4Address,
	IpV6Address,
	PhoneNumber,
	SocialSecurityNumber,
	CreditCardNumber,
	NationalId,
	Name,
	MailingAddress,
	IBAN,
	SWIFTCode,
	RoutingNumber,
	BankAccount,
	CryptoAddress,
	DigitalWalletToken,
	Unknown,
}

impl std::fmt::Display for PiiType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PiiType::Email => write!(f, "email"),
			PiiType::IpAddress => write!(f, "ip_address"),
			PiiType::IpV4Address => write!(f, "ipv4"),
			PiiType::IpV6Address => write!(f, "ipv6"),
			PiiType::PhoneNumber => write!(f, "phone_number"),
			PiiType::SocialSecurityNumber => write!(f, "ssn"),
			PiiType::CreditCardNumber => write!(f, "credit_card"),
			PiiType::NationalId => write!(f, "national_id"),
			PiiType::Name => write!(f, "name"),
			PiiType::MailingAddress => write!(f, "mailing_address"),
			PiiType::IBAN => write!(f, "iban"),
			PiiType::SWIFTCode => write!(f, "swift_code"),
			PiiType::RoutingNumber => write!(f, "routing_number"),
			PiiType::BankAccount => write!(f, "bank_account"),
			PiiType::CryptoAddress => write!(f, "crypto_address"),
			PiiType::DigitalWalletToken => write!(f, "digital_wallet"),
			PiiType::Unknown => write!(f, "unknown"),
		}
	}
}

/// Information about a detected PII field
#[derive(Debug, Clone)]
pub struct PiiField {
	/// Column index in the row
	pub column_index: usize,
	/// Column name if available
	pub column_name: Option<String>,
	/// Type of PII detected
	pub pii_type: PiiType,
	/// Confidence score (0.0-1.0)
	pub confidence: f32,
}

/// Detect if a value is a valid IPv4 address
fn is_ipv4(value: &str) -> bool {
	let parts: Vec<&str> = value.split('.').collect();
	if parts.len() != 4 {
		return false;
	}
	parts.iter().all(|part| part.parse::<u8>().is_ok())
}

/// Detect if a value is a valid IPv6 address
fn is_ipv6(value: &str) -> bool {
	// Simple heuristic: contains colons and hex characters
	if !value.contains(':') {
		return false;
	}
	// Must have at least 2 colons
	if value.matches(':').count() < 2 {
		return false;
	}
	// Check if it looks like hex
	value
		.split(':')
		.all(|part| part.is_empty() || part.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Detect if a value looks like a phone number
fn is_phone_number(value: &str) -> bool {
	let digits_only: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	let has_country_code = value.starts_with('+');
	let digit_count = digits_only.len();

	// Between 10-15 digits (international standard)
	// With optional country code prefix OR formatting characters
	(digit_count >= 10 && digit_count <= 15)
		&& (has_country_code
			|| value.contains('-')
			|| value.contains(' ')
			|| value.contains('(')
			|| digit_count == 10)
}

/// Detect if a value looks like a Social Security Number (US format)
fn is_ssn(value: &str) -> bool {
	// Format: XXX-XX-XXXX or XXXXXXXXX
	let cleaned: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	if cleaned.len() != 9 {
		return false;
	}
	// First 3 digits should not all be 0, 6, or 9
	let first_three = &cleaned[0..3];
	if first_three == "000" || first_three == "666" || first_three.chars().all(|c| c == '9') {
		return false;
	}
	true
}

/// Detect if a value looks like a national ID number
/// Includes: UK National Insurance, German ID, French ID, etc.
/// Heuristic: 7-15 digit sequence with optional hyphens/spaces/letters, but not SSN
fn is_national_id(value: &str) -> bool {
	// Don't double-count SSNs
	if is_ssn(value) {
		return false;
	}

	let cleaned: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	let digit_count = cleaned.len();

	// National IDs are typically 6-18 digits
	// UK NI: 2 letters + 6 digits + 1 letter
	// German ID: 10 digits
	// French ID: 13-15 digits
	// Chinese ID: 18 digits
	// Most national IDs in digit form: 6-18 digits
	if digit_count < 6 || digit_count > 18 {
		return false;
	}

	// Must have formatting (hyphens, spaces, or mixed letters) to distinguish from random numbers
	let has_formatting = value.contains('-')
		|| value.contains(' ')
		|| value.chars().filter(|c| c.is_alphabetic()).count() > 0;

	// Either has formatting, or is longer sequence of digits (10+)
	has_formatting || digit_count >= 10
}

/// Detect if a value looks like a credit card number
fn is_credit_card(value: &str) -> bool {
	let digits_only: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	let len = digits_only.len();
	// Credit cards are 13-19 digits
	if len < 13 || len > 19 {
		return false;
	}
	// Basic Luhn algorithm check
	let mut sum = 0;
	let mut is_second = false;
	for digit_char in digits_only.chars().rev() {
		let mut digit = digit_char.to_digit(10).unwrap() as u32;
		if is_second {
			digit *= 2;
			if digit > 9 {
				digit -= 9;
			}
		}
		sum += digit;
		is_second = !is_second;
	}
	sum % 10 == 0
}

/// Detect if a value looks like a person's name
/// Heuristic: mixed case, 2+ words or contains common name patterns
fn is_name(value: &str) -> bool {
	let trimmed = value.trim();
	// Must be somewhat reasonable length
	if trimmed.len() < 3 || trimmed.len() > 50 {
		return false;
	}
	// Must not contain digits (names don't have digits)
	if trimmed.chars().any(|c| c.is_ascii_digit()) {
		return false;
	}
	// Should contain at least one space (first and last name)
	if !trimmed.contains(' ') {
		return false;
	}
	// Should have proper capitalization (at least one uppercase per word)
	let words: Vec<&str> = trimmed.split_whitespace().collect();
	if words.len() < 2 {
		return false;
	}
	// Most words should start with uppercase
	let capitalized = words
		.iter()
		.filter(|w| w.chars().next().map_or(false, |c| c.is_uppercase()))
		.count();
	capitalized >= words.len() / 2
}

/// Detect if a value looks like a mailing address
/// Heuristic: contains street indicators and is reasonably long
fn is_mailing_address(value: &str) -> bool {
	let lower = value.to_lowercase();
	let trimmed = value.trim();

	// Must be reasonable length for an address
	if trimmed.len() < 10 || trimmed.len() > 200 {
		return false;
	}

	// Should contain at least one number (street number or zip)
	if !trimmed.chars().any(|c| c.is_ascii_digit()) {
		return false;
	}

	// Look for common address indicators
	let address_keywords = [
		"street",
		"st",
		"avenue",
		"ave",
		"boulevard",
		"blvd",
		"road",
		"rd",
		"lane",
		"ln",
		"drive",
		"dr",
		"court",
		"ct",
		"circle",
		"way",
		"trail",
		"parkway",
		"apartment",
		"apt",
		"suite",
		"ste",
		"floor",
		"zip",
		"postal",
		"city",
		"county",
	];

	address_keywords
		.iter()
		.any(|keyword| lower.contains(keyword))
}

/// Detect if a value is an IBAN (International Bank Account Number)
fn is_iban(value: &str) -> bool {
	let normalized = value.replace(" ", "").replace("-", "").to_uppercase();

	// IBANs are 15-34 characters, start with 2 letters (country code)
	if normalized.len() < 15 || normalized.len() > 34 {
		return false;
	}

	if !normalized.chars().take(2).all(|c| c.is_ascii_alphabetic()) {
		return false;
	}

	// Rest should be alphanumeric
	normalized.chars().skip(2).all(|c| c.is_alphanumeric())
}

/// Detect if a value is a SWIFT/BIC code
fn is_swift_code(value: &str) -> bool {
	let normalized = value.replace("-", "").to_uppercase();

	// SWIFT codes are 8 or 11 characters, all alphanumeric
	if !(normalized.len() == 8 || normalized.len() == 11) {
		return false;
	}

	// First 4 chars are letters (bank code)
	let bank_code: String = normalized.chars().take(4).collect();
	if !bank_code.chars().all(|c| c.is_ascii_alphabetic()) {
		return false;
	}

	// Next 2 are country code (letters)
	let country_code: String = normalized.chars().skip(4).take(2).collect();
	if !country_code.chars().all(|c| c.is_ascii_alphabetic()) {
		return false;
	}

	true
}

/// Detect if a value is a US routing number (9 digits)
fn is_routing_number(value: &str) -> bool {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	// US routing numbers are exactly 9 digits
	if digits.len() != 9 {
		return false;
	}

	// All zeros is invalid
	digits != "000000000"
}

/// Detect if a value is a bank account number (8-17 digits)
fn is_bank_account(value: &str) -> bool {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	// Bank accounts typically 8-17 digits
	if digits.len() < 8 || digits.len() > 17 {
		return false;
	}

	// All zeros or all same digit is invalid
	let first_digit = digits.chars().next().unwrap();
	!digits.chars().all(|c| c == first_digit)
}

/// Detect if a value is a cryptocurrency address (Bitcoin, Ethereum, etc.)
fn is_crypto_address(value: &str) -> bool {
	let trimmed = value.trim();

	// Bitcoin addresses: 26-35 chars, start with 1, 3, or bc1, alphanumeric (no 0, O, I, l)
	if trimmed.len() >= 26 && trimmed.len() <= 62 {
		if trimmed.starts_with('1') || trimmed.starts_with('3') || trimmed.starts_with("bc1") {
			// For bc1 addresses, allow lowercase letters and digits
			if trimmed.starts_with("bc1") {
				return trimmed.chars().skip(3).all(|c| {
					c.is_ascii_digit()
						|| (c.is_ascii_lowercase() && c != 'b' && c != 'i' && c != 'o')
				});
			}
			// For legacy addresses, don't allow 0, O, I, l
			return trimmed.chars().all(|c| {
				c.is_ascii_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l'
			});
		}
	}

	// Ethereum addresses: 42 chars, start with 0x, hex only
	if trimmed.len() == 42 && trimmed.starts_with("0x") {
		return trimmed.chars().skip(2).all(|c| c.is_ascii_hexdigit());
	}

	// XRP (Ripple) addresses: start with 'r', 25-34 chars, alphanumeric
	if trimmed.starts_with('r') && (trimmed.len() >= 25 && trimmed.len() <= 34) {
		return trimmed.chars().all(|c| c.is_ascii_alphanumeric());
	}

	false
}

/// Detect if a value is a digital wallet token or merchant account ID
fn is_digital_wallet_token(value: &str) -> bool {
	let trimmed = value.trim();

	// Stripe account ID: starts with acct_, followed by alphanumeric
	if trimmed.starts_with("acct_") && trimmed.len() > 10 {
		return trimmed
			.chars()
			.skip(5)
			.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
	}

	// Square account ID: starts with sq0asa, followed by alphanumeric
	if trimmed.starts_with("sq0asa-") && trimmed.len() > 15 {
		return trimmed.chars().skip(7).all(|c| c.is_ascii_alphanumeric());
	}

	// PayPal merchant ID: uppercase hex, 12-16 chars
	if trimmed.len() >= 12 && trimmed.len() <= 16 {
		if trimmed
			.chars()
			.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
		{
			// Likely PayPal merchant ID if all uppercase alphanumeric
			return true;
		}
	}

	// Apple Pay / Google Pay tokens: long alphanumeric with underscores
	if trimmed.len() >= 16 {
		if trimmed
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || c == '_')
			&& trimmed.len() <= 64
		{
			return true;
		}
	}

	false
}

/// Analyze a field value and detect any PII/NPI
pub fn detect_pii(value: &str, column_name: Option<&str>) -> Vec<PiiType> {
	let trimmed = value.trim();
	let mut detected = Vec::new();

	// Check email (basic heuristic)
	if trimmed.contains('@') && trimmed.len() > 5 {
		detected.push(PiiType::Email);
	}

	// Check IPv4
	if is_ipv4(trimmed) {
		detected.push(PiiType::IpV4Address);
		detected.push(PiiType::IpAddress);
	}

	// Check IPv6
	if is_ipv6(trimmed) {
		detected.push(PiiType::IpV6Address);
		detected.push(PiiType::IpAddress);
	}

	// Check phone number
	if is_phone_number(trimmed) {
		detected.push(PiiType::PhoneNumber);
	}

	// Check SSN
	if is_ssn(trimmed) {
		detected.push(PiiType::SocialSecurityNumber);
	}

	// Check national ID (including other formats than SSN)
	if is_national_id(trimmed) {
		detected.push(PiiType::NationalId);
	}

	// Check credit card
	if is_credit_card(trimmed) {
		detected.push(PiiType::CreditCardNumber);
	}

	// Check name
	if is_name(trimmed) {
		detected.push(PiiType::Name);
	}

	// Check mailing address
	if is_mailing_address(trimmed) {
		detected.push(PiiType::MailingAddress);
	}

	// Check IBAN
	if is_iban(trimmed) {
		detected.push(PiiType::IBAN);
	}

	// Check SWIFT code
	if is_swift_code(trimmed) {
		detected.push(PiiType::SWIFTCode);
	}

	// Check routing number
	if is_routing_number(trimmed) {
		detected.push(PiiType::RoutingNumber);
	}

	// Check bank account
	if is_bank_account(trimmed) {
		detected.push(PiiType::BankAccount);
	}

	// Check crypto address
	if is_crypto_address(trimmed) {
		detected.push(PiiType::CryptoAddress);
	}

	// Check digital wallet token
	if is_digital_wallet_token(trimmed) {
		detected.push(PiiType::DigitalWalletToken);
	}

	// Heuristic: if column name suggests name field
	if let Some(col) = column_name {
		let col_lower = col.to_lowercase();
		if (col_lower.contains("name")
			|| col_lower.contains("person")
			|| col_lower.contains("user"))
			&& !detected.contains(&PiiType::Name)
		{
			detected.push(PiiType::Unknown);
		}
	}

	detected
}

/// Normalize an IP address to a standard form for hashing
pub fn normalize_ip(value: &str) -> String {
	value.trim().to_lowercase()
}

/// Normalize a name to standard form (trim and normalize case)
pub fn normalize_name(value: &str) -> String {
	use crate::normalization;
	normalization::normalize_field(value)
}

/// Normalize a mailing address
pub fn normalize_address(value: &str) -> String {
	use crate::normalization;
	normalization::normalize_field(value)
}

/// Hash a phone number for duplicate detection
/// Removes all non-digit characters, then hashes the digits
pub fn hash_phone_number(value: &str) -> String {
	use crate::hash_utils;
	let digits_only: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	hash_utils::sha256_hex(&digits_only)
}

/// Hash a credit card number for duplicate detection
/// Hashes the last 4 digits + card length (prevents exposing full PAN)
pub fn hash_credit_card(value: &str) -> String {
	use crate::hash_utils;
	let digits_only: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	if digits_only.len() < 4 {
		// Card too short, hash as-is
		return hash_utils::sha256_hex(&digits_only);
	}
	// Only use last 4 digits and length for hashing (PCI compliance)
	let last_four = &digits_only[digits_only.len() - 4..];
	let masked = format!("{}_{}", last_four, digits_only.len());
	hash_utils::sha256_hex(&masked)
}

/// Hash a national ID number for duplicate detection
/// Normalizes formatting, then hashes
pub fn hash_national_id(value: &str) -> String {
	use crate::hash_utils;
	// Remove common formatting characters
	let normalized = value
		.chars()
		.filter(|c| !c.is_whitespace() && c != &'-' && c != &'/')
		.collect::<String>()
		.to_uppercase();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a Social Security Number for duplicate detection
/// Normalizes formatting, then hashes
pub fn hash_ssn(value: &str) -> String {
	use crate::hash_utils;
	let normalized = value
		.chars()
		.filter(|c| c.is_ascii_digit())
		.collect::<String>();
	hash_utils::sha256_hex(&normalized)
}

/// Hash an IBAN for duplicate detection (Stage 1: Evidence Preservation)
#[allow(dead_code)]
fn hash_iban(value: &str) -> String {
	use crate::hash_utils;
	let normalized = value.replace(" ", "").replace("-", "").to_uppercase();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a SWIFT code for duplicate detection (Stage 1: Evidence Preservation)
#[allow(dead_code)]
fn hash_swift_code(value: &str) -> String {
	use crate::hash_utils;
	let normalized = value.replace("-", "").to_uppercase();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a routing number for duplicate detection (Stage 1: Evidence Preservation)
#[allow(dead_code)]
fn hash_routing_number(value: &str) -> String {
	use crate::hash_utils;
	let normalized: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a bank account number for duplicate detection (Stage 1: Evidence Preservation)
#[allow(dead_code)]
fn hash_bank_account(value: &str) -> String {
	use crate::hash_utils;
	let normalized: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a cryptocurrency address for duplicate detection (Stage 1: Evidence Preservation)
#[allow(dead_code)]
fn hash_crypto_address(value: &str) -> String {
	use crate::hash_utils;
	let normalized = value.trim().to_lowercase();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a digital wallet token for duplicate detection (Stage 1: Evidence Preservation)
#[allow(dead_code)]
fn hash_digital_wallet_token(value: &str) -> String {
	use crate::hash_utils;
	let normalized = value.trim().to_lowercase();
	hash_utils::sha256_hex(&normalized)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ipv4_detection() {
		assert!(is_ipv4("192.168.1.1"));
		assert!(is_ipv4("8.8.8.8"));
		assert!(!is_ipv4("256.1.1.1"));
		assert!(!is_ipv4("192.168.1"));
	}

	#[test]
	fn test_ipv6_detection() {
		assert!(is_ipv6("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
		assert!(is_ipv6("::1"));
		assert!(!is_ipv6("192.168.1.1"));
	}

	#[test]
	fn test_phone_number_detection() {
		assert!(is_phone_number("+1-555-123-4567"));
		assert!(is_phone_number("555-123-4567"));
		assert!(is_phone_number("(555) 123-4567"));
		assert!(!is_phone_number("123"));
	}

	#[test]
	fn test_ssn_detection() {
		assert!(is_ssn("123-45-6789"));
		assert!(is_ssn("123456789"));
		assert!(!is_ssn("000-00-0000"));
		assert!(!is_ssn("666-00-0000"));
	}

	#[test]
	fn test_name_detection() {
		assert!(is_name("John Doe"));
		assert!(is_name("Jane M Smith"));
		assert!(!is_name("john doe")); // lowercase
		assert!(!is_name("John123")); // contains digits
	}

	#[test]
	fn test_mailing_address_detection() {
		assert!(is_mailing_address("123 Main Street, New York, NY 10001"));
		assert!(is_mailing_address("456 Oak Avenue, Suite 200, Boston, MA"));
		assert!(!is_mailing_address("hello world")); // no address keywords
	}

	#[test]
	fn test_detect_pii() {
		let pii = detect_pii("john@example.com", None);
		assert!(pii.contains(&PiiType::Email));

		let pii = detect_pii("192.168.1.1", None);
		assert!(pii.contains(&PiiType::IpV4Address));

		let pii = detect_pii("John Doe", None);
		assert!(pii.contains(&PiiType::Name));
	}

	#[test]
	fn test_national_id_detection() {
		// UK National Insurance style
		assert!(is_national_id("AB12-34-56-C"));
		// German ID style
		assert!(is_national_id("1234567890"));
		// French ID style
		assert!(is_national_id("1234567890123"));
		// Not national ID
		assert!(!is_national_id("123")); // too short
		assert!(!is_national_id("123-45-6789")); // SSN, not national ID
	}

	#[test]
	fn test_hash_phone_number() {
		let hash1 = hash_phone_number("+1-555-123-4567");
		let hash2 = hash_phone_number("555-123-4567");
		// Different digits, different hashes
		assert_ne!(hash1, hash2);
		// Same number, same hash
		let hash3 = hash_phone_number("+1-555-123-4567");
		assert_eq!(hash1, hash3);
	}

	#[test]
	fn test_hash_credit_card() {
		let hash1 = hash_credit_card("4532015112830366");
		let hash2 = hash_credit_card("4532015112830366");
		// Same card, same hash
		assert_eq!(hash1, hash2);

		// Different cards with same last 4 should have same hash
		// (this is intentional for privacy - we only hash last 4)
		let hash3 = hash_credit_card("4111111111110366");
		assert_eq!(hash1, hash3);
	}

	#[test]
	fn test_hash_national_id() {
		let hash1 = hash_national_id("AB12-34-56-C");
		let hash2 = hash_national_id("AB 12 34 56 C");
		// Different formatting, same hash
		assert_eq!(hash1, hash2);

		let hash3 = hash_national_id("AB12-34-56-D");
		// Different ID, different hash
		assert_ne!(hash1, hash3);
	}

	#[test]
	fn test_hash_ssn() {
		let hash1 = hash_ssn("123-45-6789");
		let hash2 = hash_ssn("123456789");
		// Same SSN, same hash (formatting removed)
		assert_eq!(hash1, hash2);

		let hash3 = hash_ssn("123-45-6790");
		// Different SSN, different hash
		assert_ne!(hash1, hash3);
	}

	#[test]
	fn test_iban_detection() {
		// Valid IBANs from various countries
		assert!(is_iban("DE89370400440532013000"));
		assert!(is_iban("GB82WEST12345698765432"));
		assert!(is_iban("FR1420041010050500013M02606"));
		assert!(is_iban("ES9121000418450200051332"));
		assert!(is_iban("IT60X0542811101000000123456"));
		assert!(is_iban("NL91ABNA0417164300"));

		// With spacing
		assert!(is_iban("DE89 3704 0044 0532 0130 00"));

		// Invalid
		assert!(!is_iban("NOT_AN_IBAN")); // Too short
		assert!(!is_iban("1234567890123456")); // Starts with digit
	}

	#[test]
	fn test_swift_detection() {
		// Valid SWIFT codes (8 or 11 characters)
		assert!(is_swift_code("DEUTDEFF")); // 8 chars
		assert!(is_swift_code("HSBKGB2L")); // 8 chars
		assert!(is_swift_code("BARCDEFF")); // 8 chars

		// With dashes
		assert!(is_swift_code("DEUT-DEFF"));

		// Invalid
		assert!(!is_swift_code("SHORT")); // Too short
		assert!(!is_swift_code("1234DEFF")); // Starts with digit
	}

	#[test]
	fn test_routing_number_detection() {
		// Valid US routing numbers (9 digits)
		assert!(is_routing_number("021000021"));
		assert!(is_routing_number("021-000-021"));
		assert!(is_routing_number("111000025"));

		// Invalid
		assert!(!is_routing_number("000000000")); // All zeros
		assert!(!is_routing_number("12345")); // Too short
		assert!(!is_routing_number("12345678901")); // Too long
	}

	#[test]
	fn test_bank_account_detection() {
		// Valid bank accounts (8-17 digits)
		assert!(is_bank_account("123456789")); // 9 digits
		assert!(is_bank_account("1234-5678-9")); // 9 digits with formatting
		assert!(is_bank_account("12345678")); // 8 digits minimum
		assert!(is_bank_account("12345678901234567")); // 17 digits

		// Invalid
		assert!(!is_bank_account("1234567")); // Too short
		assert!(!is_bank_account("000000000")); // All same digit
		assert!(!is_bank_account("12345678901234567890")); // Too long
	}

	#[test]
	fn test_crypto_address_detection() {
		// Valid Bitcoin addresses (26-35 chars, starts with 1, 3, or bc1)
		assert!(is_crypto_address("1A1z7agoat5GkjM7E6vfj4FPVNwvH8K7p")); // 34 chars

		// Valid Bitcoin P2SH address (starts with 3)
		assert!(is_crypto_address("3J98t1WpEZ73CNmYviecrnyiWrnqRhWNLy")); // 34 chars

		// Valid Bitcoin segwit address (starts with bc1)
		assert!(is_crypto_address(
			"bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
		)); // bech32

		// Valid Ethereum addresses (42 chars, starts with 0x)
		assert!(is_crypto_address(
			"0x742d35Cc6634C0532925a3b844Bc9e7595f42e0e"
		));

		// Valid XRP (Ripple) addresses (starts with 'r', 25-34 chars)
		assert!(is_crypto_address("rN7n7otQDd6FczFgLdlqtyMVrXe3JxqXeK")); // 34 chars

		// Invalid
		assert!(!is_crypto_address("0x123")); // Ethereum too short
		assert!(!is_crypto_address("invalid_address")); // Not a crypto address
	}

	#[test]
	fn test_digital_wallet_detection() {
		// Stripe account ID
		assert!(is_digital_wallet_token("acct_1234567890abcdef"));

		// Square account ID
		assert!(is_digital_wallet_token("sq0asa-1234567890abcdef"));

		// PayPal merchant ID (12-16 uppercase alphanumeric)
		assert!(is_digital_wallet_token("XXXXXXXXXXXX")); // 12 chars

		// Apple Pay / Google Pay tokens
		assert!(is_digital_wallet_token("9876543210987654"));
		assert!(is_digital_wallet_token("GPAY123456789ABCDEF_TOKEN_123456"));

		// Invalid
		assert!(!is_digital_wallet_token("short")); // Too short
	}

	#[test]
	fn test_hash_iban() {
		let hash1 = hash_iban("DE89370400440532013000");
		let hash2 = hash_iban("DE89 3704 0044 0532 0130 00");
		// Same IBAN with different formatting should hash to same value
		assert_eq!(hash1, hash2);

		let hash3 = hash_iban("GB82WEST12345698765432");
		// Different IBAN should hash to different value
		assert_ne!(hash1, hash3);
	}

	#[test]
	fn test_hash_swift_code() {
		let hash1 = hash_swift_code("DEUTDEFF");
		let hash2 = hash_swift_code("DEUT-DEFF");
		// Same SWIFT with different formatting should hash to same value
		assert_eq!(hash1, hash2);

		let hash3 = hash_swift_code("HSBKGB2L");
		// Different SWIFT should hash to different value
		assert_ne!(hash1, hash3);
	}

	#[test]
	fn test_hash_routing_number() {
		let hash1 = hash_routing_number("021000021");
		let hash2 = hash_routing_number("021-000-021");
		// Same routing number with different formatting should hash to same value
		assert_eq!(hash1, hash2);

		let hash3 = hash_routing_number("111000025");
		// Different routing number should hash to different value
		assert_ne!(hash1, hash3);
	}

	#[test]
	fn test_hash_bank_account() {
		let hash1 = hash_bank_account("123456789");
		let hash2 = hash_bank_account("1234-5678-9");
		// Same account with different formatting should hash to same value
		assert_eq!(hash1, hash2);

		let hash3 = hash_bank_account("987654321");
		// Different account should hash to different value
		assert_ne!(hash1, hash3);
	}

	#[test]
	fn test_hash_crypto_address() {
		let hash1 = hash_crypto_address("0x742d35Cc6634C0532925a3b844Bc9e7595f42e0e");
		let hash2 = hash_crypto_address("0x742d35Cc6634C0532925a3b844Bc9e7595f42e0e");
		// Same address, same hash
		assert_eq!(hash1, hash2);

		let hash3 = hash_crypto_address("0x1234567890abcdef1234567890abcdef12345678");
		// Different address, different hash
		assert_ne!(hash1, hash3);
	}

	#[test]
	fn test_hash_digital_wallet_token() {
		let hash1 = hash_digital_wallet_token("acct_1234567890abcdef");
		let hash2 = hash_digital_wallet_token("acct_1234567890abcdef");
		// Same token, same hash
		assert_eq!(hash1, hash2);

		let hash3 = hash_digital_wallet_token("acct_abcdef1234567890");
		// Different token, different hash
		assert_ne!(hash1, hash3);
	}

	#[test]
	fn test_comprehensive_pii_detection() {
		// Test that all PII types are properly detected
		let pii = detect_pii("4532015112830366", None);
		assert!(pii.contains(&PiiType::CreditCardNumber));

		let pii = detect_pii("5551234567", None);
		assert!(pii.contains(&PiiType::PhoneNumber));

		let pii = detect_pii("DE89370400440532013000", None);
		assert!(pii.contains(&PiiType::IBAN));

		let pii = detect_pii("DEUTDEFF", None);
		assert!(pii.contains(&PiiType::SWIFTCode));

		let pii = detect_pii("021000021", None);
		assert!(pii.contains(&PiiType::RoutingNumber));

		let pii = detect_pii("123456789", None);
		assert!(pii.contains(&PiiType::BankAccount));

		let pii = detect_pii("0x742d35Cc6634C0532925a3b844Bc9e7595f42e0e", None);
		assert!(pii.contains(&PiiType::CryptoAddress));

		let pii = detect_pii("acct_1234567890abcdef", None);
		assert!(pii.contains(&PiiType::DigitalWalletToken));
	}

	#[test]
	fn test_international_national_ids() {
		// UK National Insurance Number
		assert!(is_national_id("AB123456C"));
		assert!(is_national_id("AB 12 34 56 C"));
		assert!(is_national_id("AB-12-34-56-C"));

		// German Personalausweis (ID Card)
		assert!(is_national_id("1234567890"));
		assert!(is_national_id("1234-5678-90"));

		// Spanish DNI
		assert!(is_national_id("12345678Z"));
		assert!(is_national_id("1234-5678-Z"));

		// Italian Codice Fiscale
		assert!(is_national_id("RSSMRA80A01H501T"));
		assert!(is_national_id("RSS MRA 80A01 H501 T"));

		// Chinese ID (18 digits)
		assert!(is_national_id("110101199003071011"));
		assert!(is_national_id("110101 1990 0307 1011"));

		// Japanese My Number
		assert!(is_national_id("01234567890"));
		assert!(is_national_id("0123-4567-890"));

		// Indian Aadhaar
		assert!(is_national_id("123456789012"));
		assert!(is_national_id("1234 5678 9012"));
	}
}
