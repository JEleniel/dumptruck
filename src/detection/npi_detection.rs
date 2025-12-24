//! Detection and normalization of Personally Identifiable Information (PII) and Non-Public
//! Information (NPI).
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
//! Each detection function includes hashing capabilities for duplicate identification while
//! preserving privacy. Other potential NPI/PII fields for analyst review

use serde::{Deserialize, Serialize};

use crate::regexes::{IPV4, IPV6};

/// Types of PII/NPI that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Check if an IPv4 address is in a private/internal range
///
/// Private ranges per RFC 1918:
/// - 10.0.0.0/8 (10.0.0.0 - 10.255.255.255)
/// - 172.16.0.0/12 (172.16.0.0 - 172.31.255.255)
/// - 192.168.0.0/16 (192.168.0.0 - 192.168.255.255)
///
/// Also excludes:
/// - 127.0.0.0/8 (Loopback)
/// - 169.254.0.0/16 (Link-local)
/// - 0.0.0.0/8 (This network)
/// - 255.255.255.255 (Broadcast)
fn is_private_ipv4(octets: &[u8; 4]) -> bool {
	match octets[0] {
		0 => true,                                         // 0.0.0.0/8 - This network
		10 => true,                                        // 10.0.0.0/8 - Private
		127 => true,                                       // 127.0.0.0/8 - Loopback
		169 if octets[1] == 254 => true,                   // 169.254.0.0/16 - Link-local
		172 if octets[1] >= 16 && octets[1] <= 31 => true, // 172.16.0.0/12 - Private
		192 if octets[1] == 168 => true,                   // 192.168.0.0/16 - Private
		255 if octets[1] == 255 && octets[2] == 255 && octets[3] == 255 => true, // 255.255.255.
		// 255 - Broadcast
		_ => false,
	}
}

/// Detect if a value is a valid IPv4 address
fn is_ipv4(value: &str) -> bool {
	if !IPV4.is_match(value) {
		return false;
	}
	// Extract the octets and validate they are in range [0, 255]
	let parts: Vec<&str> = value
		.split('/')
		.next()
		.unwrap_or(value)
		.split('.')
		.collect();
	if parts.len() != 4 {
		return false;
	}
	let octets: Result<Vec<u8>, _> = parts.iter().map(|p| p.parse::<u8>()).collect();
	match octets {
		Ok(bytes) if bytes.len() == 4 => {
			let octets_array = [bytes[0], bytes[1], bytes[2], bytes[3]];
			// Only flag public IPs as PII, not private ones
			!is_private_ipv4(&octets_array)
		}
		_ => false,
	}
}

/// Check if an IPv6 address is in a private/internal range
///
/// Private/internal ranges:
/// - fc00::/7 (Unique local addresses)
/// - fe80::/10 (Link-local addresses)
/// - ::1 (Loopback)
fn is_private_ipv6(value: &str) -> bool {
	let lower = value.to_lowercase();

	// Loopback
	if lower == "::1" {
		return true;
	}

	// Link-local (fe80::/10)
	if lower.starts_with("fe80:") {
		return true;
	}

	// Unique local addresses (fc00::/7)
	// This includes fc00:: and fd00:: ranges
	if lower.starts_with("fc") || lower.starts_with("fd") {
		// Must be followed by hex digits to be valid IPv6
		if lower.len() > 2 {
			let next_char = lower.chars().nth(2).unwrap_or('x');
			if next_char.is_ascii_hexdigit() || next_char == ':' {
				return true;
			}
		}
	}

	false
}

/// Detect if a value is a valid IPv6 address
fn is_ipv6(value: &str) -> bool {
	if !IPV6.is_match(value) {
		return false;
	}
	// Only flag public IPs as PII, not private ones
	!is_private_ipv6(value)
}

/// Detect if a value looks like a phone number
fn is_phone_number(value: &str) -> bool {
	let digits_only: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	let has_country_code = value.starts_with('+');
	let digit_count = digits_only.len();

	// Between 10-15 digits (international standard)
	// With optional country code prefix OR formatting characters
	(10..=15).contains(&digit_count)
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

/// National ID format matcher with country context
/// Per threat model: preserve ambiguity, never force a single country
#[derive(Debug, Clone, PartialEq)]
struct NationalIdMatch {
	/// Country code or name (e.g., "GB" or "UK National Insurance")
	country: String,
	/// Confidence (0.0-1.0): higher = more certain
	confidence: f32,
	/// Whether format checksum passed (if applicable)
	checksum_valid: bool,
}

/// Validate UK National Insurance Number (NI): 2 letters + 6 digits + 1 letter
/// Example: AB12-34-56-C, AB 12 34 56 C, AB123456C
fn check_uk_ni(value: &str) -> Option<NationalIdMatch> {
	// Remove whitespace and hyphens
	let cleaned: String = value
		.chars()
		.filter(|c| !c.is_whitespace() && *c != '-')
		.collect();

	if cleaned.len() < 9 {
		return None;
	}

	// Pattern: [A-Z]{2}[0-9]{6}[A-Z]
	let mut chars = cleaned.chars().peekable();

	// Collect first 2 letters
	let parts: String = chars.by_ref().take(2).collect();
	if parts.len() != 2 || !parts.chars().all(|c| c.is_alphabetic()) {
		return None;
	}

	// Collect 6 digits
	let digits: String = chars.by_ref().take(6).collect();
	if digits.len() != 6 || !digits.chars().all(|c| c.is_ascii_digit()) {
		return None;
	}

	// Collect 1 letter
	let suffix: String = chars.by_ref().take(1).collect();
	if suffix.len() != 1 || !suffix.chars().all(|c| c.is_alphabetic()) {
		return None;
	}

	Some(NationalIdMatch {
		country: "GB (UK National Insurance)".to_string(),
		confidence: 0.95, // Very high confidence for correct format
		checksum_valid: true,
	})
}

/// Validate German ID (Personalausweis): 10 digits
/// Format: 1234567890, 1234-5678-90
fn check_german_id(value: &str) -> Option<NationalIdMatch> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.len() != 10 {
		return None;
	}

	Some(NationalIdMatch {
		country: "DE (German ID)".to_string(),
		confidence: 0.85,      // Moderate confidence (10 digits alone is ambiguous)
		checksum_valid: false, // German ID doesn't have strong checksum
	})
}

/// Validate French ID (Numéro de Sécurité Sociale): 13-15 digits
/// Format: [0-9]{13,15}
fn check_french_id(digits: &str) -> Option<NationalIdMatch> {
	let len = digits.len();
	if !(13..=15).contains(&len) || !digits.chars().all(|c| c.is_ascii_digit()) {
		return None;
	}

	Some(NationalIdMatch {
		country: "FR (French Social Security)".to_string(),
		confidence: 0.80, // Moderate confidence
		checksum_valid: false,
	})
}

/// Validate Chinese ID (Resident Identity Card): 18 digits
/// Format: [0-9]{18}
fn check_chinese_id(digits: &str) -> Option<NationalIdMatch> {
	if digits.len() != 18 || !digits.chars().all(|c| c.is_ascii_digit()) {
		return None;
	}

	Some(NationalIdMatch {
		country: "CN (Chinese ID)".to_string(),
		confidence: 0.90, // High confidence (18 digits is distinctive)
		checksum_valid: false,
	})
}

/// Validate Spanish ID (DNI): 8 digits + 1 letter
/// Format: 12345678X, 12345678-X, 12345678 X
fn check_spanish_id(value: &str) -> Option<NationalIdMatch> {
	let cleaned: String = value
		.chars()
		.filter(|c| !c.is_whitespace() && *c != '-')
		.collect();

	if cleaned.len() != 9 {
		return None;
	}

	let digits = &cleaned[0..8];
	let letter = &cleaned[8..9];

	if !digits.chars().all(|c| c.is_ascii_digit()) || !letter.chars().all(|c| c.is_alphabetic()) {
		return None;
	}

	Some(NationalIdMatch {
		country: "ES (Spanish DNI)".to_string(),
		confidence: 0.92,
		checksum_valid: false,
	})
}

/// Validate Italian ID (Codice Fiscale): 16 alphanumeric characters
/// Format: RSSMRA80A01A123Q (6 letters, 2 digits, 1 letter, 2 digits, 1 letter, 3 digits, 1 letter)
fn check_italian_id(value: &str) -> Option<NationalIdMatch> {
	let cleaned: String = value
		.chars()
		.filter(|c| !c.is_whitespace() && *c != '-')
		.collect();

	if cleaned.len() != 16 {
		return None;
	}

	// Very specific pattern (simplified check)
	let mut chars = cleaned.chars();
	let part1: String = chars.by_ref().take(6).collect();
	let part2: String = chars.by_ref().take(2).collect();
	let part3: String = chars.by_ref().take(1).collect();
	let part4: String = chars.by_ref().take(2).collect();
	let part5: String = chars.by_ref().take(1).collect();
	let part6: String = chars.by_ref().take(3).collect();
	let part7: String = chars.collect();

	if part1.chars().all(|c| c.is_alphabetic())
		&& part2.chars().all(|c| c.is_ascii_digit())
		&& part3.chars().all(|c| c.is_alphabetic())
		&& part4.chars().all(|c| c.is_ascii_digit())
		&& part5.chars().all(|c| c.is_alphabetic())
		&& part6.chars().all(|c| c.is_ascii_digit())
		&& part7.chars().all(|c| c.is_alphabetic())
	{
		return Some(NationalIdMatch {
			country: "IT (Italian Codice Fiscale)".to_string(),
			confidence: 0.93,
			checksum_valid: false,
		});
	}

	None
}

/// Validate Dutch ID (Burgerservicenummer): 9 digits
/// Format: 123456789, 123-456-789
fn check_dutch_id(value: &str) -> Option<NationalIdMatch> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.len() != 9 {
		return None;
	}

	Some(NationalIdMatch {
		country: "NL (Dutch BSN)".to_string(),
		confidence: 0.80,
		checksum_valid: false,
	})
}

/// Validate Japanese My Number: 12 digits
/// Format: 012345678901, 0123-4567-890
fn check_japanese_my_number(value: &str) -> Option<NationalIdMatch> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.len() != 12 {
		return None;
	}

	Some(NationalIdMatch {
		country: "JP (Japanese My Number)".to_string(),
		confidence: 0.88,
		checksum_valid: false,
	})
}

/// Validate Indian Aadhaar: 12 digits
/// Format: 123456789012, 1234 5678 9012
fn check_indian_aadhaar(value: &str) -> Option<NationalIdMatch> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.len() != 12 {
		return None;
	}

	Some(NationalIdMatch {
		country: "IN (Indian Aadhaar)".to_string(),
		confidence: 0.86,
		checksum_valid: false,
	})
}

/// Find all possible national IDs for a value
/// Returns multiple matches if value matches multiple country formats
/// Per threat model: preserve ambiguity, never force single country
fn find_national_id_matches(value: &str) -> Vec<NationalIdMatch> {
	let mut matches = Vec::new();

	// Canonicalize: trim and uppercase
	let trimmed = value.trim();

	// Extract digit-only version
	let digits_only: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();

	// Check format-specific validators first (higher confidence)
	if let Some(m) = check_uk_ni(trimmed) {
		matches.push(m);
	}
	if let Some(m) = check_spanish_id(trimmed) {
		matches.push(m);
	}
	if let Some(m) = check_italian_id(trimmed) {
		matches.push(m);
	}

	// Check digit-only validators
	if let Some(m) = check_chinese_id(&digits_only) {
		matches.push(m);
	}
	if let Some(m) = check_german_id(trimmed) {
		matches.push(m);
	}
	if let Some(m) = check_french_id(&digits_only) {
		matches.push(m);
	}
	if let Some(m) = check_dutch_id(trimmed) {
		matches.push(m);
	}
	if let Some(m) = check_japanese_my_number(trimmed) {
		matches.push(m);
	}
	if let Some(m) = check_indian_aadhaar(trimmed) {
		matches.push(m);
	}

	matches
}

/// Detect if a value looks like a national ID number
/// Per threat model: Layered detection with confidence scoring
/// Returns true if ANY plausible match found with confidence >= 0.75
fn is_national_id(value: &str) -> bool {
	// Don't double-count SSNs
	if is_ssn(value) {
		return false;
	}

	let matches = find_national_id_matches(value);

	// Accept if any match has sufficient confidence
	// Threshold 0.75 balances false positives vs missed detection
	matches.iter().any(|m| m.confidence >= 0.75)
}

/// Validate credit card using Luhn algorithm (ISO/IEC 7812-1)
///
/// # Process
/// 1. Starting from the right, double every second digit
/// 2. If doubling yields >9, subtract 9
/// 3. Sum all digits
/// 4. Valid if total mod 10 == 0
fn luhn_checksum(digits: &str) -> bool {
	let mut sum = 0;
	let mut is_second = false;

	for digit_char in digits.chars().rev() {
		let mut digit = digit_char.to_digit(10).unwrap();
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

/// Validate credit card network format based on IIN (Issuer Identification Number)
///
/// Returns (is_valid, network_name) where network_name is for informational purposes
fn validate_credit_card_network(digits: &str) -> bool {
	let len = digits.len();

	// Extract first 2-4 digits for IIN checking
	let first_digit = digits.chars().next().unwrap_or('0');
	let first_two = if len >= 2 { &digits[0..2] } else { "0" };
	let first_four = if len >= 4 { &digits[0..4] } else { "0" };
	let first_six = if len >= 6 { &digits[0..6] } else { "0" };

	// Visa: starts with 4, length 13, 16, or 19
	if first_digit == '4' && (len == 13 || len == 16 || len == 19) {
		return true;
	}

	// Mastercard: 51-55 or 2221-2720, length 16
	if len == 16 {
		if let Ok(first_two_num) = first_two.parse::<u32>()
			&& (51..=55).contains(&first_two_num)
		{
			return true;
		}
		// 2221-2720 range
		if let Ok(first_four_num) = first_four.parse::<u32>()
			&& (2221..=2720).contains(&first_four_num)
		{
			return true;
		}
	}

	// American Express: 34 or 37, length 15
	if (first_two == "34" || first_two == "37") && len == 15 {
		return true;
	}

	// Discover: 6011, 622126-622925, 644-649, 65, length 16-19
	if (16..=19).contains(&len) {
		if first_four == "6011" {
			return true;
		}
		if let Ok(first_six_num) = first_six.parse::<u32>()
			&& (622126..=622925).contains(&first_six_num)
		{
			return true;
		}
		if let Ok(first_three) = first_two.parse::<u32>()
			&& (644..=649).contains(&first_three)
		{
			return true;
		}
		if first_digit == '6' && first_two == "65" {
			return true;
		}
	}

	// JCB: 3528-3589, length 16-19
	if (16..=19).contains(&len)
		&& let Ok(first_four_num) = first_four.parse::<u32>()
		&& (3528..=3589).contains(&first_four_num)
	{
		return true;
	}

	// Diners Club: 300-305, 36, 38, 39, length 14
	if len == 14 {
		if let Ok(first_three) = first_two.parse::<u32>()
			&& (300..=305).contains(&first_three)
		{
			return true;
		}
		if first_two == "36" || first_two == "38" || first_two == "39" {
			return true;
		}
	}

	// If we got here, it doesn't match known card networks
	false
}

/// Detect if a value looks like a credit card number (PAN)
///
/// Validates according to ISO/IEC 7812:
/// - Length: 13-19 digits (most commonly 16)
/// - Luhn algorithm: check digit validation
/// - Network format: matches known card network patterns
fn is_credit_card(value: &str) -> bool {
	let digits_only: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	let len = digits_only.len();

	// Credit cards are 13-19 digits
	if !(13..=19).contains(&len) {
		return false;
	}

	// Must pass Luhn checksum validation
	if !luhn_checksum(&digits_only) {
		return false;
	}

	// Must match a known card network format
	validate_credit_card_network(&digits_only)
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
		.filter(|w| w.chars().next().is_some_and(|c| c.is_uppercase()))
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
	if trimmed.len() >= 26
		&& trimmed.len() <= 62
		&& (trimmed.starts_with('1') || trimmed.starts_with('3') || trimmed.starts_with("bc1"))
	{
		// For bc1 addresses, allow lowercase letters and digits
		if trimmed.starts_with("bc1") {
			return trimmed.chars().skip(3).all(|c| {
				c.is_ascii_digit() || (c.is_ascii_lowercase() && c != 'b' && c != 'i' && c != 'o')
			});
		}
		// For legacy addresses, don't allow 0, O, I, l
		return trimmed
			.chars()
			.all(|c| c.is_ascii_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l');
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
	if trimmed.len() >= 12
		&& trimmed.len() <= 16
		&& trimmed
			.chars()
			.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
	{
		// Likely PayPal merchant ID if all uppercase alphanumeric
		return true;
	}

	// Apple Pay / Google Pay tokens: long alphanumeric with underscores
	if trimmed.len() >= 16
		&& trimmed
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || c == '_')
		&& trimmed.len() <= 64
	{
		return true;
	}

	false
}

/// Analyze a field value and detect any PII/NPI
pub fn detect_pii(value: &str, column_name: Option<&str>) -> Vec<PiiType> {
	let trimmed = value.trim();
	let mut detected = Vec::new();

	// Contact information
	detect_contact_info(trimmed, &mut detected);

	// Network addresses
	detect_network_addresses(trimmed, &mut detected);

	// Financial identifiers
	detect_financial_identifiers(trimmed, &mut detected);

	// Personal identifiers
	detect_personal_identifiers(trimmed, &mut detected);

	// Names and addresses
	detect_names_and_addresses(trimmed, &mut detected);

	// Cryptography and wallets
	detect_crypto_identifiers(trimmed, &mut detected);

	// Column name heuristic
	apply_column_name_heuristic(column_name, &mut detected);

	detected
}

fn detect_contact_info(value: &str, detected: &mut Vec<PiiType>) {
	if value.contains('@') && value.len() > 5 {
		detected.push(PiiType::Email);
	}
	if is_phone_number(value) {
		detected.push(PiiType::PhoneNumber);
	}
}

fn detect_network_addresses(value: &str, detected: &mut Vec<PiiType>) {
	if is_ipv4(value) {
		detected.push(PiiType::IpV4Address);
		detected.push(PiiType::IpAddress);
	}
	if is_ipv6(value) {
		detected.push(PiiType::IpV6Address);
		detected.push(PiiType::IpAddress);
	}
}

fn detect_financial_identifiers(value: &str, detected: &mut Vec<PiiType>) {
	if is_ssn(value) {
		detected.push(PiiType::SocialSecurityNumber);
	}
	if is_credit_card(value) {
		detected.push(PiiType::CreditCardNumber);
	}
	if is_iban(value) {
		detected.push(PiiType::IBAN);
	}
	if is_swift_code(value) {
		detected.push(PiiType::SWIFTCode);
	}
	if is_routing_number(value) {
		detected.push(PiiType::RoutingNumber);
	}
	if is_bank_account(value) {
		detected.push(PiiType::BankAccount);
	}
}

fn detect_personal_identifiers(value: &str, detected: &mut Vec<PiiType>) {
	if is_national_id(value) {
		detected.push(PiiType::NationalId);
	}
}

fn detect_names_and_addresses(value: &str, detected: &mut Vec<PiiType>) {
	if is_name(value) {
		detected.push(PiiType::Name);
	}
	if is_mailing_address(value) {
		detected.push(PiiType::MailingAddress);
	}
}

fn detect_crypto_identifiers(value: &str, detected: &mut Vec<PiiType>) {
	if is_crypto_address(value) {
		detected.push(PiiType::CryptoAddress);
	}
	if is_digital_wallet_token(value) {
		detected.push(PiiType::DigitalWalletToken);
	}
}

fn apply_column_name_heuristic(column_name: Option<&str>, detected: &mut Vec<PiiType>) {
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
	use crate::core::hash_utils;
	let digits_only: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	hash_utils::sha256_hex(&digits_only)
}

/// Hash a credit card number for duplicate detection
/// Hashes the last 4 digits + card length (prevents exposing full PAN)
pub fn hash_credit_card(value: &str) -> String {
	use crate::core::hash_utils;
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
	use crate::core::hash_utils;
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
	use crate::core::hash_utils;
	let normalized = value
		.chars()
		.filter(|c| c.is_ascii_digit())
		.collect::<String>();
	hash_utils::sha256_hex(&normalized)
}

/// Hash an IBAN for duplicate detection (Stage 1: Evidence Preservation)
pub fn hash_iban(value: &str) -> String {
	use crate::core::hash_utils;
	let normalized = value.replace(" ", "").replace("-", "").to_uppercase();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a SWIFT code for duplicate detection (Stage 1: Evidence Preservation)
pub fn hash_swift_code(value: &str) -> String {
	use crate::core::hash_utils;
	let normalized = value.replace("-", "").to_uppercase();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a routing number for duplicate detection (Stage 1: Evidence Preservation)
pub fn hash_routing_number(value: &str) -> String {
	use crate::core::hash_utils;
	let normalized: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a bank account number for duplicate detection (Stage 1: Evidence Preservation)
pub fn hash_bank_account(value: &str) -> String {
	use crate::core::hash_utils;
	let normalized: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a cryptocurrency address for duplicate detection (Stage 1: Evidence Preservation)
pub fn hash_crypto_address(value: &str) -> String {
	use crate::core::hash_utils;
	let normalized = value.trim().to_lowercase();
	hash_utils::sha256_hex(&normalized)
}

/// Hash a digital wallet token for duplicate detection (Stage 1: Evidence Preservation)
pub fn hash_digital_wallet_token(value: &str) -> String {
	use crate::core::hash_utils;
	let normalized = value.trim().to_lowercase();
	hash_utils::sha256_hex(&normalized)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ipv4_detection() {
		// Public IPs should be detected as PII
		assert!(is_ipv4("8.8.8.8"));
		assert!(is_ipv4("1.1.1.1"));

		// Private/internal IPs should NOT be detected as PII
		// RFC 1918 ranges
		assert!(!is_ipv4("10.0.0.1"));
		assert!(!is_ipv4("10.255.255.255"));
		assert!(!is_ipv4("172.16.0.1"));
		assert!(!is_ipv4("172.31.255.255"));
		assert!(!is_ipv4("192.168.0.1"));
		assert!(!is_ipv4("192.168.1.1"));
		assert!(!is_ipv4("192.168.255.255"));

		// Loopback and link-local
		assert!(!is_ipv4("127.0.0.1"));
		assert!(!is_ipv4("127.255.255.255"));
		assert!(!is_ipv4("169.254.0.1"));
		assert!(!is_ipv4("169.254.255.255"));

		// Special ranges
		assert!(!is_ipv4("0.0.0.1"));
		assert!(!is_ipv4("255.255.255.255"));

		// Invalid IPs
		assert!(!is_ipv4("256.1.1.1"));
		assert!(!is_ipv4("192.168.1"));
	}

	#[test]
	fn test_ipv6_detection() {
		// Public IPv6 should be detected as PII
		assert!(is_ipv6("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));

		// Private/internal IPv6 should NOT be detected as PII
		// Loopback
		assert!(!is_ipv6("::1"));
		// Link-local
		assert!(!is_ipv6("fe80::1"));
		assert!(!is_ipv6("fe80::ffff:192.0.2.1"));
		// Unique local addresses (fc00::/7 and fd00::/8)
		assert!(!is_ipv6("fc00::1"));
		assert!(!is_ipv6("fd00::1"));
		assert!(!is_ipv6("fd12:3456:789a::1"));

		// Not IPv6
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

		// Private IP should NOT be flagged as PII
		let pii = detect_pii("192.168.1.1", None);
		assert!(!pii.contains(&PiiType::IpV4Address));
		assert!(!pii.contains(&PiiType::IpAddress));

		// Public IP should be flagged as PII
		let pii = detect_pii("8.8.8.8", None);
		assert!(pii.contains(&PiiType::IpV4Address));
		assert!(pii.contains(&PiiType::IpAddress));

		let pii = detect_pii("John Doe", None);
		assert!(pii.contains(&PiiType::Name));
	}

	#[test]
	fn test_national_id_uk_ni() {
		// UK National Insurance: 2 letters + 6 digits + 1 letter
		assert!(is_national_id("AB123456C"));
		assert!(is_national_id("AB 12 34 56 C"));
		assert!(is_national_id("AB-12-34-56-C"));
		assert!(!is_national_id("A1234567C")); // Only 1 letter at start
		assert!(!is_national_id("AB12345C")); // Only 5 digits
		assert!(!is_national_id("AB1234567")); // No suffix letter
	}

	#[test]
	fn test_national_id_spanish_dni() {
		// Spanish DNI: 8 digits + 1 letter
		assert!(is_national_id("12345678X"));
		assert!(is_national_id("12345678 X"));
		assert!(is_national_id("12345678-X"));
		assert!(!is_national_id("1234567X")); // Only 7 digits
		assert!(!is_national_id("123456789X")); // 9 digits
	}

	#[test]
	fn test_national_id_chinese() {
		// Chinese ID: 18 digits
		assert!(is_national_id("110101199003072015"));
		assert!(!is_national_id("11010119900307201")); // 17 digits
		assert!(!is_national_id("1101011990030720150")); // 19 digits
	}

	#[test]
	fn test_national_id_ambiguity_preservation() {
		// A 13-digit sequence matches both French and could match other formats
		let value = "1234567890123";
		assert!(is_national_id(value)); // Should match French ID pattern
	}

	#[test]
	fn test_national_id_false_positives() {
		// Random sequences without sufficient structure should NOT match
		// Per threat model: formatting or distinctive length required to avoid false positives
		assert!(!is_national_id("123")); // Too short
		assert!(!is_national_id("123-45-6789")); // SSN, excluded
	}

	#[test]
	fn test_national_id_italian_codice() {
		// Italian Codice Fiscale: 16 alphanumeric pattern
		assert!(is_national_id("RSSMRA80A01A123Q"));
		assert!(!is_national_id("RSSMRA80A01A12")); // Too short
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
	fn test_credit_card_detection() {
		// Valid test card numbers (all pass Luhn and network format)
		// Visa - 16 digits
		assert!(is_credit_card("4111111111111111"));
		assert!(is_credit_card("4532015112830366"));

		// Mastercard - 16 digits (51-55 range)
		assert!(is_credit_card("5555555555554444"));
		assert!(is_credit_card("5105105105105100"));

		// American Express - 15 digits
		assert!(is_credit_card("378282246310005"));

		// Discover - 16 digits
		assert!(is_credit_card("6011111111111117"));
		assert!(is_credit_card("6011000990139424"));

		// JCB - 16 digits
		assert!(is_credit_card("3530111333300000"));

		// Formatted versions (spaces and dashes are stripped)
		assert!(is_credit_card("4111-1111-1111-1111"));
		assert!(is_credit_card("4111 1111 1111 1111"));

		// Invalid: wrong length (after stripping non-digits)
		assert!(!is_credit_card("411111111111")); // Too short
		assert!(!is_credit_card("41111111111111111")); // Too long (17 digits)

		// Invalid: fails Luhn check (modified last digit)
		assert!(!is_credit_card("4111111111111112"));
		assert!(!is_credit_card("5555555555554445"));

		// Invalid: matches length and Luhn but wrong network format
		// (This number passes Luhn but starts with invalid IIN)
		assert!(!is_credit_card("9999999999999999"));
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

		// Japanese My Number (12 digits)
		assert!(is_national_id("012345678901"));
		assert!(is_national_id("0123-4567-8901"));

		// Indian Aadhaar
		assert!(is_national_id("123456789012"));
		assert!(is_national_id("1234 5678 9012"));
	}
}
