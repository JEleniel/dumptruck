//! National ID detection for 15+ countries.
/// Information about a detected national ID.
#[derive(Debug, Clone)]
pub struct NationalIdMatch {
	pub id_type: String,
	pub country: String,
}

mod others;
use others::*;

/// Check UK National Insurance Number (2 letters + 6 digits + 1 letter).
fn check_uk_ni(value: &str) -> Option<NationalIdMatch> {
	let normalized = value.to_uppercase();
	let no_spaces = normalized.replace(" ", "").replace("-", "");

	if no_spaces.len() != 9 {
		return None;
	}

	let letters: String = no_spaces.chars().take(2).collect();
	if !letters.chars().all(|c| c.is_ascii_alphabetic()) {
		return None;
	}

	let middle: String = no_spaces.chars().skip(2).take(6).collect();
	if !middle.chars().all(|c| c.is_ascii_digit()) {
		return None;
	}

	let suffix = no_spaces.chars().last();
	if !suffix.is_some_and(|c| c.is_ascii_alphabetic()) {
		return None;
	}

	Some(NationalIdMatch {
		id_type: "UK National Insurance".to_string(),
		country: "GB".to_string(),
	})
}

/// Find all matching national ID formats for a value.
pub fn find_national_id_matches(value: &str) -> Vec<NationalIdMatch> {
	let mut matches = Vec::new();

	if let Some(m) = check_uk_ni(value) {
		matches.push(m);
	}
	if let Some(m) = check_german_id(value) {
		matches.push(m);
	}

	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if let Some(m) = check_french_id(&digits) {
		matches.push(m);
	}
	if let Some(m) = check_chinese_id(&digits) {
		matches.push(m);
	}
	if let Some(m) = check_spanish_id(value) {
		matches.push(m);
	}
	if let Some(m) = check_italian_id(value) {
		matches.push(m);
	}
	if let Some(m) = check_dutch_id(value) {
		matches.push(m);
	}
	if let Some(m) = check_japanese_my_number(value) {
		matches.push(m);
	}
	if let Some(m) = check_indian_aadhaar(value) {
		matches.push(m);
	}

	matches
}

/// Detect if a value is a national ID from any supported country.
pub fn is_national_id(value: &str) -> bool {
	!find_national_id_matches(value).is_empty()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_uk_ni() {
		assert!(is_national_id("AB123456C"));
		assert!(is_national_id("AB 12 34 56 C"));
		assert!(is_national_id("AB-12-34-56-C"));
		assert!(!is_national_id("A1234567C"));
		assert!(!is_national_id("AB12345C"));
	}

	#[test]
	fn test_spanish_dni() {
		assert!(is_national_id("12345678X"));
		assert!(is_national_id("12345678 X"));
		assert!(is_national_id("12345678-X"));
		assert!(!is_national_id("1234567X"));
	}

	#[test]
	fn test_chinese_id() {
		assert!(is_national_id("110101199003072015"));
		assert!(!is_national_id("11010119900307201"));
	}

	#[test]
	fn test_italian_codice() {
		assert!(is_national_id("RSSMRA80A01A123Q"));
		assert!(!is_national_id("RSSMRA80A01A12"));
	}

	#[test]
	fn test_international_ids() {
		assert!(is_national_id("AB123456C")); // UK
		assert!(is_national_id("1234567890")); // German
		assert!(is_national_id("12345678Z")); // Spanish
		assert!(is_national_id("110101199003071011")); // Chinese
	}
}
