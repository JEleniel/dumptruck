//! Phone number and contact detection.

/// Detect if a value is a phone number (10-15 digits, allowing formatting).
pub fn is_phone_number(value: &str) -> bool {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	// Phone numbers typically 10-15 digits
	if digits.len() < 10 || digits.len() > 15 {
		return false;
	}

	// All zeros is invalid
	!digits.chars().all(|c| c == '0')
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_phone_detection() {
		assert!(is_phone_number("+1-555-123-4567"));
		assert!(is_phone_number("555-123-4567"));
		assert!(is_phone_number("(555) 123-4567"));
		assert!(!is_phone_number("123"));
	}
}
