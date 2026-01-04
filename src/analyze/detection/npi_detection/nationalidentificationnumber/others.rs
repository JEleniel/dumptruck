//! Additional national ID checks factored out to keep the parent module small.

use super::NationalIdMatch;

/// Check German Personalausweis (ID Card) - 10 digits.
pub fn check_german_id(value: &str) -> Option<NationalIdMatch> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.len() != 10 {
		return None;
	}

	Some(NationalIdMatch {
		id_type: "German Personalausweis".to_string(),
		country: "DE".to_string(),
	})
}

/// Check French ID - 13-15 digits.
pub fn check_french_id(digits: &str) -> Option<NationalIdMatch> {
	if digits.len() < 13 || digits.len() > 15 {
		return None;
	}

	Some(NationalIdMatch {
		id_type: "French ID".to_string(),
		country: "FR".to_string(),
	})
}

/// Check Chinese ID Number - 18 digits.
pub fn check_chinese_id(digits: &str) -> Option<NationalIdMatch> {
	if digits.len() != 18 {
		return None;
	}

	Some(NationalIdMatch {
		id_type: "Chinese Resident ID".to_string(),
		country: "CN".to_string(),
	})
}

/// Check Spanish DNI - 8 digits + 1 letter.
pub fn check_spanish_id(value: &str) -> Option<NationalIdMatch> {
	let normalized = value.to_uppercase();
	let no_spaces = normalized.replace(" ", "").replace("-", "");

	if no_spaces.len() != 9 {
		return None;
	}

	let digits: String = no_spaces.chars().take(8).collect();
	if !digits.chars().all(|c| c.is_ascii_digit()) {
		return None;
	}

	let suffix = no_spaces.chars().last();
	if !suffix.is_some_and(|c| c.is_ascii_alphabetic()) {
		return None;
	}

	Some(NationalIdMatch {
		id_type: "Spanish DNI".to_string(),
		country: "ES".to_string(),
	})
}

/// Check Italian Codice Fiscale - 16 alphanumeric characters.
pub fn check_italian_id(value: &str) -> Option<NationalIdMatch> {
	let normalized = value.to_uppercase();
	let no_spaces = normalized.replace(" ", "").replace("-", "");

	if no_spaces.len() != 16 {
		return None;
	}

	if !no_spaces.chars().all(|c| c.is_ascii_alphanumeric()) {
		return None;
	}

	Some(NationalIdMatch {
		id_type: "Italian Codice Fiscale".to_string(),
		country: "IT".to_string(),
	})
}

/// Check Dutch Burgerservicenummer (BSN) - 9 digits.
pub fn check_dutch_id(value: &str) -> Option<NationalIdMatch> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.len() != 9 {
		return None;
	}

	Some(NationalIdMatch {
		id_type: "Dutch BSN".to_string(),
		country: "NL".to_string(),
	})
}

/// Check Japanese My Number - 12 digits.
pub fn check_japanese_my_number(value: &str) -> Option<NationalIdMatch> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.len() != 12 {
		return None;
	}

	Some(NationalIdMatch {
		id_type: "Japanese My Number".to_string(),
		country: "JP".to_string(),
	})
}

/// Check Indian Aadhaar - 12 digits.
pub fn check_indian_aadhaar(value: &str) -> Option<NationalIdMatch> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

	if digits.len() != 12 {
		return None;
	}

	Some(NationalIdMatch {
		id_type: "Indian Aadhaar".to_string(),
		country: "IN".to_string(),
	})
}
