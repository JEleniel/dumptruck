//! Types used by the NPI/PII detection module.

use serde::{Deserialize, Serialize};

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
	pub column_index: usize,
	pub column_name: Option<String>,
	pub pii_type: PiiType,
	pub confidence: f32,
}
