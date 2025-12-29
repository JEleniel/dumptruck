//! Social Security Number (SSN) detection and hashing.

/// Detect if a value is a US Social Security Number (XXX-XX-XXXX format).
pub fn is_ssn(value: &str) -> bool {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	// SSNs are exactly 9 digits
	if digits.len() != 9 {
		return false;
	}

	// First 3 digits (area): 000-665, 667-900 (666 and 900-999 invalid)
	let area: &str = &digits[0..3];
	let area_num: u32 = area.parse().unwrap_or(0);
	if area_num == 0 || area_num == 666 || area_num >= 900 {
		return false;
	}

	// Group (middle 2 digits) cannot be 00
	if &digits[3..5] == "00" {
		return false;
	}

	// Serial (last 4 digits) cannot be 0000
	if &digits[5..9] == "0000" {
		return false;
	}

	true
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ssn_valid() {
		assert!(is_ssn("123-45-6789"));
		assert!(is_ssn("123456789"));
	}

	#[test]
	fn test_ssn_invalid() {
		assert!(!is_ssn("000-00-0000"));
		assert!(!is_ssn("666-00-0000"));
		assert!(!is_ssn("900-00-0000"));
		assert!(!is_ssn("123-00-0000"));
		assert!(!is_ssn("123-45-0000"));
	}
}
