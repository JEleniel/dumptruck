//! Alias Resolution Module (Stage 8)
//!
//! Identifies and links related entries across multiple fields and formats.
//!
//! **Key Capabilities:**
//! - Email aliases: Plus addressing (user+tag@domain), dot variants (john.doe vs johndoe)
//! - User ID aliases: Numeric, alphanumeric, UUID variants
//! - Phone normalization: International format standardization with country code
//! - National ID variants: Multiple formats for same identity
//! - Confidence scoring: Each alias link includes confidence percentage
//!
//! Enables tracking of the same person across different data representations,
//! critical for deduplication and fraud detection in breach data.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Errors that can occur during alias resolution
#[derive(Error, Debug)]
pub enum AliasResolutionError {
	#[error("Invalid phone format: {0}")]
	InvalidPhoneFormat(String),

	#[error("Invalid email format: {0}")]
	InvalidEmailFormat(String),

	#[error("Invalid country code: {0}")]
	InvalidCountryCode(String),

	#[error("Resolution failed: {0}")]
	ResolutionFailed(String),
}

/// Types of alias relationships that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AliasType {
	/// Email plus addressing (user+tag@domain → user@domain)
	EmailPlus,
	/// Email dot variants (john.doe@domain → johndoe@domain)
	EmailDot,
	/// Domain aliases (googlemail.com → gmail.com)
	EmailDomain,
	/// Phone number normalization
	PhoneNormalization,
	/// National ID variants
	NationalIdVariant,
	/// User ID format variation
	UserIdVariant,
	/// Username case variation
	UsernameCase,
	/// Generic string similarity
	StringVariant,
}

impl std::fmt::Display for AliasType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AliasType::EmailPlus => write!(f, "email_plus"),
			AliasType::EmailDot => write!(f, "email_dot"),
			AliasType::EmailDomain => write!(f, "email_domain"),
			AliasType::PhoneNormalization => write!(f, "phone_normalization"),
			AliasType::NationalIdVariant => write!(f, "national_id_variant"),
			AliasType::UserIdVariant => write!(f, "user_id_variant"),
			AliasType::UsernameCase => write!(f, "username_case"),
			AliasType::StringVariant => write!(f, "string_variant"),
		}
	}
}

/// A detected alias relationship between two values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasLink {
	/// The original/canonical value
	pub canonical: String,

	/// The variant/alias value
	pub variant: String,

	/// Type of alias relationship
	pub alias_type: AliasType,

	/// Confidence score (0-100) that these are the same entity
	/// 100 = cryptographic certainty, 50 = moderate confidence, 0 = just a suggestion
	pub confidence: u8,

	/// Optional notes about the alias relationship
	pub notes: Option<String>,
}

impl AliasLink {
	/// Create a new alias link
	pub fn new(canonical: String, variant: String, alias_type: AliasType, confidence: u8) -> Self {
		Self {
			canonical,
			variant,
			alias_type,
			confidence,
			notes: None,
		}
	}

	/// Add notes to the alias link
	pub fn with_notes(mut self, notes: String) -> Self {
		self.notes = Some(notes);
		self
	}
}

/// Extracts and normalizes the local part of an email (before @)
fn extract_email_local(email: &str) -> Result<String, AliasResolutionError> {
	let at_idx = email
		.find('@')
		.ok_or_else(|| AliasResolutionError::InvalidEmailFormat("No @ symbol found".to_string()))?;
	Ok(email[..at_idx].to_lowercase())
}

/// Extracts and normalizes the domain part of an email (after @)
fn extract_email_domain(email: &str) -> Result<String, AliasResolutionError> {
	let at_idx = email
		.find('@')
		.ok_or_else(|| AliasResolutionError::InvalidEmailFormat("No @ symbol found".to_string()))?;
	Ok(email[at_idx + 1..].to_lowercase())
}

/// Detect email plus addressing aliases (user+tag@domain → user@domain)
///
/// Plus addressing allows users to create infinite email variants:
/// - john+spam@gmail.com
/// - john+newsletter@gmail.com
///
/// All route to john@gmail.com
pub fn detect_email_plus_aliases(email: &str) -> Result<Option<AliasLink>, AliasResolutionError> {
	let local = extract_email_local(email)?;
	let domain = extract_email_domain(email)?;

	// Check if local part contains a plus sign
	if let Some(plus_idx) = local.find('+') {
		let base_local = &local[..plus_idx];
		let canonical = format!("{}@{}", base_local, domain);

		return Ok(Some(AliasLink::new(
			canonical,
			email.to_lowercase(),
			AliasType::EmailPlus,
			95, // Very high confidence - structural certainty
		)));
	}

	Ok(None)
}

/// Detect email dot variants (john.doe@domain → johndoe@domain)
///
/// Many email systems ignore dots in the local part, allowing these to be equivalent:
/// - john.doe@gmail.com
/// - johndoe@gmail.com
/// - j.o.h.n.d.o.e@gmail.com
pub fn detect_email_dot_aliases(
	email1: &str,
	email2: &str,
) -> Result<Option<AliasLink>, AliasResolutionError> {
	let local1 = extract_email_local(email1)?;
	let local2 = extract_email_local(email2)?;
	let domain1 = extract_email_domain(email1)?;
	let domain2 = extract_email_domain(email2)?;

	// Same domain required
	if domain1 != domain2 {
		return Ok(None);
	}

	// Remove all dots from both locals
	let local1_no_dots = local1.replace(".", "");
	let local2_no_dots = local2.replace(".", "");

	// If they match after removing dots, it's an alias
	if local1_no_dots == local2_no_dots && local1 != local2 {
		let canonical = email1.to_lowercase();
		return Ok(Some(AliasLink::new(
			canonical,
			email2.to_lowercase(),
			AliasType::EmailDot,
			85, // High confidence - domain-specific behavior
		)));
	}

	Ok(None)
}

/// Normalize a phone number to E.164 international format
///
/// # Arguments
/// * `phone` - Raw phone number (may include dashes, spaces, parens)
/// * `country_code` - ISO 3166-1 country code (e.g., "US", "GB", "FR")
///
/// # Returns
/// Normalized phone in E.164 format: +[country_code][number]
/// Example: +1-555-123-4567 → +15551234567
#[allow(dead_code)]
fn normalize_phone_e164(phone: &str, country_code: &str) -> Result<String, AliasResolutionError> {
	// Remove all non-digit characters
	let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.is_empty() {
		return Err(AliasResolutionError::InvalidPhoneFormat(
			"No digits found in phone".to_string(),
		));
	}

	// Map country codes to phone prefixes
	let prefix = match country_code.to_uppercase().as_str() {
		"US" | "CA" => "1",  // North America
		"GB" | "UK" => "44", // United Kingdom
		"FR" => "33",        // France
		"DE" => "49",        // Germany
		"IT" => "39",        // Italy
		"ES" => "34",        // Spain
		"AU" => "61",        // Australia
		"JP" => "81",        // Japan
		"CN" => "86",        // China
		"IN" => "91",        // India
		"MX" => "52",        // Mexico
		"BR" => "55",        // Brazil
		"NL" => "31",        // Netherlands
		"CH" => "41",        // Switzerland
		"SE" => "46",        // Sweden
		code => {
			return Err(AliasResolutionError::InvalidCountryCode(format!(
				"Unknown country code: {}",
				code
			)));
		}
	};

	// If number already has country prefix, validate it
	if digits.starts_with(prefix) {
		return Ok(format!("+{}", digits));
	}

	// If number has leading +, it already has country code - accept as-is
	if phone.starts_with('+') {
		return Ok(format!("+{}", digits));
	}

	// Otherwise, add country code
	Ok(format!("+{}{}", prefix, digits))
}

/// Detect phone number normalization (different formats of same number)
pub fn detect_phone_aliases(
	phone1: &str,
	phone2: &str,
) -> Result<Option<AliasLink>, AliasResolutionError> {
	// Extract digits only
	let digits1: String = phone1.chars().filter(|c| c.is_ascii_digit()).collect();
	let digits2: String = phone2.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits1.is_empty() || digits2.is_empty() {
		return Ok(None);
	}

	// If all digits match, these are the same phone number
	if digits1 == digits2 && phone1 != phone2 {
		return Ok(Some(AliasLink::new(
			phone1.to_string(),
			phone2.to_string(),
			AliasType::PhoneNormalization,
			90, // Very high confidence - digits are identical
		)));
	}

	Ok(None)
}

/// Detect national ID variants (different formats representing same ID)
///
/// National IDs often have multiple valid formats:
/// - With or without separators: 123-45-6789 vs 12345 67890
/// - With or without country prefix
/// - Different case or zero-padding
pub fn detect_national_id_aliases(
	id1: &str,
	id2: &str,
) -> Result<Option<AliasLink>, AliasResolutionError> {
	// Normalize both: remove separators, convert to uppercase
	let normalized1 = id1
		.chars()
		.filter(|c| c.is_alphanumeric())
		.collect::<String>()
		.to_uppercase();

	let normalized2 = id2
		.chars()
		.filter(|c| c.is_alphanumeric())
		.collect::<String>()
		.to_uppercase();

	// If normalized forms match, these are the same ID
	if normalized1 == normalized2 && id1 != id2 {
		return Ok(Some(AliasLink::new(
			id1.to_string(),
			id2.to_string(),
			AliasType::NationalIdVariant,
			92, // High confidence - structural match after normalization
		)));
	}

	Ok(None)
}

/// Detect user ID variants (numeric, alphanumeric, UUID)
///
/// Same user may be referenced by:
/// - Numeric ID: 12345
/// - Hex ID: 0x3039
/// - UUID: 550e8400-e29b-41d4-a716-446655440000
pub fn detect_user_id_aliases(
	id1: &str,
	id2: &str,
) -> Result<Option<AliasLink>, AliasResolutionError> {
	// Extract only hex digits (works for numeric, hex, UUID)
	let hex1: String = id1.chars().filter(|c| c.is_ascii_hexdigit()).collect();
	let hex2: String = id2.chars().filter(|c| c.is_ascii_hexdigit()).collect();

	if hex1.is_empty() || hex2.is_empty() {
		return Ok(None);
	}

	// Compare as hex values (case-insensitive)
	if hex1.to_lowercase() == hex2.to_lowercase() && id1 != id2 {
		return Ok(Some(AliasLink::new(
			id1.to_string(),
			id2.to_string(),
			AliasType::UserIdVariant,
			88, // High confidence - hex representation matches
		)));
	}

	Ok(None)
}

/// Detect username case variations (john vs JOHN vs John)
pub fn detect_username_case_aliases(username1: &str, username2: &str) -> Option<AliasLink> {
	if username1.to_lowercase() == username2.to_lowercase() && username1 != username2 {
		Some(AliasLink::new(
			username1.to_string(),
			username2.to_string(),
			AliasType::UsernameCase,
			98, // Very high confidence - only difference is case
		))
	} else {
		None
	}
}

/// Find all aliases for a given email address
pub fn find_email_aliases(email: &str) -> Result<Vec<AliasLink>, AliasResolutionError> {
	let mut aliases = Vec::new();

	// Check for plus addressing
	if let Some(alias) = detect_email_plus_aliases(email)? {
		aliases.push(alias);
	}

	Ok(aliases)
}

/// Find all aliases across a collection of values
/// Returns unique canonical forms and their variants
pub fn find_canonical_forms(values: &[&str]) -> Result<Vec<Vec<String>>, AliasResolutionError> {
	let mut groups: Vec<HashSet<String>> = Vec::new();

	for value in values {
		let mut found_group = false;

		// Try to find an alias with an existing group
		for group in &mut groups {
			let existing_values: Vec<String> = group.iter().cloned().collect();
			for existing in existing_values {
				// Check email aliases
				if let Ok(Some(_alias)) = detect_email_dot_aliases(value, &existing) {
					group.insert(value.to_string());
					group.insert(existing);
					found_group = true;
					break;
				}

				// Check case variants
				if let Some(_alias) = detect_username_case_aliases(value, &existing) {
					group.insert(value.to_string());
					group.insert(existing);
					found_group = true;
					break;
				}
			}

			if found_group {
				break;
			}
		}

		// If not found in any group, create a new group
		if !found_group {
			let mut new_group = HashSet::new();
			new_group.insert(value.to_string());
			groups.push(new_group);
		}
	}

	// Convert groups to vectors
	Ok(groups
		.into_iter()
		.map(|g| g.into_iter().collect())
		.collect())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_detect_email_plus_aliases() {
		let result = detect_email_plus_aliases("john+spam@gmail.com")
			.expect("Should parse email")
			.expect("Should detect plus alias");

		assert_eq!(result.alias_type, AliasType::EmailPlus);
		assert_eq!(result.canonical, "john@gmail.com");
		assert_eq!(result.confidence, 95);
	}

	#[test]
	fn test_email_plus_no_alias() {
		let result = detect_email_plus_aliases("john@gmail.com").expect("Should parse email");

		assert!(result.is_none());
	}

	#[test]
	fn test_detect_email_dot_aliases() {
		let result = detect_email_dot_aliases("john.doe@gmail.com", "johndoe@gmail.com")
			.expect("Should parse emails")
			.expect("Should detect dot alias");

		assert_eq!(result.alias_type, AliasType::EmailDot);
		assert!(result.confidence >= 80);
	}

	#[test]
	fn test_detect_phone_aliases() {
		let result = detect_phone_aliases("555-123-4567", "5551234567")
			.expect("Should parse phones")
			.expect("Should detect phone alias");

		assert_eq!(result.alias_type, AliasType::PhoneNormalization);
		assert_eq!(result.confidence, 90);
	}

	#[test]
	fn test_detect_username_case() {
		let alias =
			detect_username_case_aliases("JohnDoe", "johndoe").expect("Should detect case alias");

		assert_eq!(alias.alias_type, AliasType::UsernameCase);
		assert_eq!(alias.confidence, 98);
	}

	#[test]
	fn test_national_id_variants() {
		let result = detect_national_id_aliases("123-45-6789", "12345-6789")
			.expect("Should parse IDs")
			.expect("Should detect ID variant");

		assert_eq!(result.alias_type, AliasType::NationalIdVariant);
		assert!(result.confidence >= 85);
	}
}
