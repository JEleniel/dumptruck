//! Credit card, IBAN, bank account detection.

/// Luhn checksum validation for credit card numbers.
fn luhn_checksum(digits: &str) -> bool {
	let reversed: String = digits.chars().rev().collect();
	let mut sum = 0;

	for (idx, c) in reversed.chars().enumerate() {
		if !c.is_ascii_digit() {
			return false;
		}

		let mut digit = c.to_digit(10).unwrap_or(0) as i32;
		if idx % 2 == 1 {
			digit *= 2;
			if digit > 9 {
				digit -= 9;
			}
		}
		sum += digit;
	}

	sum % 10 == 0
}

/// Validate credit card network format (Visa, Mastercard, Amex, etc.).
fn validate_credit_card_network(digits: &str) -> bool {
	if digits.len() < 13 || digits.len() > 19 {
		return false;
	}

	let first = digits.chars().next().unwrap_or('0');

	match first {
		'4' => digits.len() == 16 || digits.len() == 13,
		'5' => {
			let first_two: String = digits.chars().take(2).collect();
			if let Ok(num) = first_two.parse::<u32>() {
				(51..=55).contains(&num) && digits.len() == 16
			} else {
				false
			}
		}
		'3' => {
			let second = digits.chars().nth(1).unwrap_or('0');
			match second {
				'4' | '7' => digits.len() == 15,
				'5' | '6' | '8' => digits.len() == 16,
				_ => false,
			}
		}
		'6' => {
			let first_four: String = digits.chars().take(4).collect();
			(first_four == "6011" || first_four == "6271") && digits.len() == 16
		}
		_ => false,
	}
}

/// Detect if a value is a valid credit card number.
pub fn is_credit_card(value: &str) -> bool {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if !validate_credit_card_network(&digits) {
		return false;
	}

	luhn_checksum(&digits)
}

/// Detect if a value is an IBAN (International Bank Account Number).
pub fn is_iban(value: &str) -> bool {
	let normalized = value.replace(" ", "").replace("-", "").to_uppercase();

	if normalized.len() < 15 || normalized.len() > 34 {
		return false;
	}

	if !normalized.chars().take(2).all(|c| c.is_ascii_alphabetic()) {
		return false;
	}

	normalized.chars().skip(2).all(|c| c.is_alphanumeric())
}

/// Detect if a value is a SWIFT/BIC code.
pub fn is_swift_code(value: &str) -> bool {
	let normalized = value.replace("-", "").to_uppercase();

	if !(normalized.len() == 8 || normalized.len() == 11) {
		return false;
	}

	let bank_code: String = normalized.chars().take(4).collect();
	if !bank_code.chars().all(|c| c.is_ascii_alphabetic()) {
		return false;
	}

	let country_code: String = normalized.chars().skip(4).take(2).collect();
	if !country_code.chars().all(|c| c.is_ascii_alphabetic()) {
		return false;
	}

	true
}

/// Detect if a value is a US routing number (9 digits).
pub fn is_routing_number(value: &str) -> bool {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.len() != 9 {
		return false;
	}

	digits != "000000000"
}

/// Detect if a value is a bank account number (8-17 digits).
pub fn is_bank_account(value: &str) -> bool {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.len() < 8 || digits.len() > 17 {
		return false;
	}

	let first_digit = digits.chars().next().expect("checked non-empty");
	!digits.chars().all(|c| c == first_digit)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_credit_card_valid() {
		assert!(is_credit_card("4111111111111111")); // Visa
		assert!(is_credit_card("5555555555554444")); // Mastercard
		assert!(is_credit_card("378282246310005")); // Amex
		assert!(is_credit_card("6011111111111117")); // Discover
	}

	#[test]
	fn test_credit_card_invalid() {
		assert!(!is_credit_card("411111111111")); // Too short
		assert!(!is_credit_card("4111111111111112")); // Fails Luhn
		assert!(!is_credit_card("9999999999999999")); // Invalid network
	}

	#[test]
	fn test_iban() {
		assert!(is_iban("DE89370400440532013000"));
		assert!(is_iban("GB82WEST12345698765432"));
		assert!(!is_iban("NOT_AN_IBAN"));
	}

	#[test]
	fn test_swift() {
		assert!(is_swift_code("DEUTDEFF"));
		assert!(is_swift_code("HSBKGB2L"));
		assert!(!is_swift_code("SHORT"));
	}

	#[test]
	fn test_routing_number() {
		assert!(is_routing_number("021000021"));
		assert!(!is_routing_number("000000000"));
		assert!(!is_routing_number("12345"));
	}

	#[test]
	fn test_bank_account() {
		assert!(is_bank_account("123456789"));
		assert!(is_bank_account("12345678"));
		assert!(!is_bank_account("000000000"));
		assert!(!is_bank_account("1234567"));
	}
}
