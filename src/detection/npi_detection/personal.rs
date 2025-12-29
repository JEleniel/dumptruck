//! Name and address detection.

// Common address keywords used for heuristic mailing address detection.
static ADDRESS_KEYWORDS: &[&str] = &[
	"street",
	"st",
	"avenue",
	"ave",
	"road",
	"rd",
	"drive",
	"dr",
	"lane",
	"ln",
	"way",
	"blvd",
	"boulevard",
	"court",
	"ct",
	"circle",
	"plaza",
	"square",
	"suite",
	"apt",
	"apartment",
	"floor",
	"fl",
	"building",
	"bldg",
	"zip",
	"postal",
	"city",
	"state",
	"county",
	"province",
	"country",
	"address",
	"po box",
];

/// Detect if a value is a person's name (title case, 5+ chars, no digits).
pub fn is_name(value: &str) -> bool {
	let trimmed = value.trim();

	if trimmed.len() < 5 {
		return false;
	}

	if trimmed.chars().any(|c| c.is_ascii_digit()) {
		return false;
	}

	let words: Vec<&str> = trimmed.split_whitespace().collect();
	if words.is_empty() {
		return false;
	}

	// At least first word should start with uppercase
	if let Some(first_word) = words.first()
		&& let Some(first_char) = first_word.chars().next()
		&& !first_char.is_ascii_uppercase()
	{
		return false;
	}

	// Words should be alphabetic only
	words.iter().all(|w| {
		w.chars()
			.all(|c| c.is_alphabetic() || c == '\'' || c == '-')
	})
}

/// Detect if a value is a mailing address.
pub fn is_mailing_address(value: &str) -> bool {
	let trimmed = value.trim().to_lowercase();

	if trimmed.len() < 10 {
		return false;
	}

	ADDRESS_KEYWORDS.iter().any(|kw| trimmed.contains(kw))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_name_valid() {
		assert!(is_name("John Doe"));
		assert!(is_name("Jane M Smith"));
	}

	#[test]
	fn test_name_invalid() {
		assert!(!is_name("john doe")); // lowercase
		assert!(!is_name("John123")); // digits
		assert!(!is_name("Doe")); // too short
	}

	#[test]
	fn test_address_valid() {
		assert!(is_mailing_address("123 Main Street, New York, NY 10001"));
		assert!(is_mailing_address("456 Oak Avenue, Suite 200, Boston, MA"));
	}

	#[test]
	fn test_address_invalid() {
		assert!(!is_mailing_address("hello world"));
	}
}
