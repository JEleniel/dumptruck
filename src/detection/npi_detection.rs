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

mod contact;
mod crypto;
mod financial;
mod national_ids;
mod network;
mod personal;
mod ssn;
mod types;
mod utils;

pub use contact::is_phone_number;
pub use crypto::{is_crypto_address, is_digital_wallet_token};
pub use financial::{is_bank_account, is_credit_card, is_iban, is_routing_number, is_swift_code};
pub use national_ids::{NationalIdMatch, find_national_id_matches, is_national_id};
pub use network::{is_ipv4, is_ipv6};
pub use personal::{is_mailing_address, is_name};
pub use ssn::is_ssn;

pub use types::{PiiField, PiiType};

/// Analyze a field value and detect any PII/NPI
pub fn detect_pii(value: &str, column_name: Option<&str>) -> Vec<PiiType> {
	let trimmed = value.trim();
	let mut detected = Vec::new();

	detect_contact_info(trimmed, &mut detected);
	detect_network_addresses(trimmed, &mut detected);
	detect_financial_identifiers(trimmed, &mut detected);
	detect_personal_identifiers(trimmed, &mut detected);
	detect_names_and_addresses(trimmed, &mut detected);
	detect_crypto_identifiers(trimmed, &mut detected);
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
pub use utils::{
	hash_bank_account, hash_credit_card, hash_crypto_address, hash_digital_wallet_token, hash_iban,
	hash_national_id, hash_phone_number, hash_routing_number, hash_ssn, hash_swift_code,
	normalize_address, normalize_ip, normalize_name,
};

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_detect_pii() {
		let pii = detect_pii("john@example.com", None);
		assert!(pii.contains(&PiiType::Email));

		let pii = detect_pii("192.168.1.1", None);
		assert!(!pii.contains(&PiiType::IpV4Address));

		let pii = detect_pii("8.8.8.8", None);
		assert!(pii.contains(&PiiType::IpV4Address));

		let pii = detect_pii("John Doe", None);
		assert!(pii.contains(&PiiType::Name));
	}

	#[test]
	fn test_comprehensive_pii_detection() {
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
}
