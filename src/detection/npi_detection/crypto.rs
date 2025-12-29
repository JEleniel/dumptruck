//! Cryptocurrency address and digital wallet token detection.

/// Detect if a value is a cryptocurrency address.
pub fn is_crypto_address(value: &str) -> bool {
	let trimmed = value.trim();

	// Bitcoin: 26-35 chars, starts with 1, 3, or bc1
	if trimmed.len() >= 26 && trimmed.len() <= 62 {
		if trimmed.starts_with('1') || trimmed.starts_with('3') {
			return trimmed.chars().all(|c| {
				c.is_ascii_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l'
			});
		}

		if trimmed.starts_with("bc1") {
			return trimmed.chars().skip(3).all(|c| {
				c.is_ascii_digit() || (c.is_ascii_lowercase() && c != 'b' && c != 'i' && c != 'o')
			});
		}
	}

	// Ethereum: 42 chars, starts with 0x, hex only
	if trimmed.len() == 42 && trimmed.starts_with("0x") {
		return trimmed.chars().skip(2).all(|c| c.is_ascii_hexdigit());
	}

	// XRP (Ripple): starts with 'r', 25-34 chars, alphanumeric
	if trimmed.starts_with('r') && trimmed.len() >= 25 && trimmed.len() <= 34 {
		return trimmed.chars().all(|c| c.is_ascii_alphanumeric());
	}

	false
}

/// Detect if a value is a digital wallet token or merchant account ID.
pub fn is_digital_wallet_token(value: &str) -> bool {
	let trimmed = value.trim();

	// Stripe account ID
	if trimmed.starts_with("acct_") && trimmed.len() > 10 {
		return trimmed
			.chars()
			.skip(5)
			.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
	}

	// Square account ID
	if trimmed.starts_with("sq0asa-") && trimmed.len() > 15 {
		return trimmed.chars().skip(7).all(|c| c.is_ascii_alphanumeric());
	}

	// PayPal merchant ID
	if trimmed.len() >= 12
		&& trimmed.len() <= 16
		&& trimmed
			.chars()
			.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
	{
		return true;
	}

	// Apple Pay / Google Pay tokens
	if trimmed.len() >= 16
		&& trimmed.len() <= 64
		&& trimmed
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || c == '_')
	{
		return true;
	}

	false
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_bitcoin_address() {
		assert!(is_crypto_address("1A1z7agoat5GkjM7E6vfj4FPVNwvH8K7p"));
		assert!(is_crypto_address("3J98t1WpEZ73CNmYviecrnyiWrnqRhWNLy"));
		assert!(is_crypto_address(
			"bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
		));
	}

	#[test]
	fn test_ethereum_address() {
		assert!(is_crypto_address(
			"0x742d35Cc6634C0532925a3b844Bc9e7595f42e0e"
		));
		assert!(!is_crypto_address("0x123")); // Too short
	}

	#[test]
	fn test_ripple_address() {
		assert!(is_crypto_address("rN7n7otQDd6FczFgLdlqtyMVrXe3JxqXeK"));
	}

	#[test]
	fn test_digital_wallet() {
		assert!(is_digital_wallet_token("acct_1234567890abcdef"));
		assert!(is_digital_wallet_token("sq0asa-1234567890abcdef"));
		assert!(is_digital_wallet_token("XXXXXXXXXXXX"));
	}
}
