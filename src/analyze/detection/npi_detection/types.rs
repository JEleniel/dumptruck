//! Types used by the NPI/PII detection module.

use serde::{Deserialize, Serialize};

/// Types of PII/NPI that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NPIType {
	/// Account Numbers: set [0-9], 8-17 digits, optional spaces or dashes
	AccountNumber,
	/// International Bank Account Number (IBAN): 2 letter ISO country code, 2 check digits,
	/// 15-34 characters, set [A-Z0-9], optional spaces every 4 characters
	/// checksum MOD-97-10
	BankIBAN,
	/// Bank Routing Number: set [0-9],9 digits, checksum MOD-10
	BankRoutingNumber,
	/// SWIFT/BIC Code: 8 or 11 characters, bank code (set [A-Z], 4 characters), country code
	/// (ISO-3166-1 alpha-2), location code (set [A-Z0-9], 2 characters), optional branch code
	/// (set [A-Z0-9], 3 characters), 8-11 characters total
	BankSWIFTCode,
	/// Fingerprints, retina scans, voiceprints, facial recognition, and other biometric data
	BiometricData,
	/// Credit Card Numbers: set [0-9], issuer identification number (6-8 digits), account number, check digit (1 digit),
	/// optional spaces every 4 digits, checksum Luhn algorithm
	CreditCardNumber,
	/// Crypto wallet addresses (Bitcoin, Ethereum, etc.)
	/// Identified best by checksums:
	/// BTC / LTC / XRP / TRX / SOL / DOT:	Base58Check checksum
	/// ETH:	EIP-55 mixed-case checksum
	/// Bech32 (bc1, ltc1, addr1):	Bech32 checksum
	/// XMR:	Network byte + checksum
	CryptoAddress,
	/// Date of Birth, identified primarily by headers/labels
	DateOfBirth,
	/// Email addresses, identified by "standard" email format and headers/labels
	EmailAddress,
	/// Names, identified primarily by headers/labels
	PersonalName,
	/// Gender information, identified primarily by headers/labels, may also be identified by the
	/// M/F/O convention, Male/Female/Other, or similar
	GenderData,
	/// GPS coordinates: decimal degrees or DMS format
	GPSLocation,
	/// IMEI numbers: set [0-9], 15 digits, checksum Luhn algorithm
	IMEI,
	/// Physical mailing addresses, identified primarily by headers/labels
	MailingAddress,
	/// National identification numbers (SSN, SIN, NIN, etc.)
	/// Identified by country-specific formats and checksums where applicable
	NationalIdentificationNumber,
	/// Other identification numbers not covered by specific types
	OtherIdentificationNumber(&'static str),
	/// Other identitiy information not covered by specific types, e.g. social media handles
	OtherIdentity(&'static str),
	/// Other non-public personal information not covered by specific types
	OtherPersonalData(&'static str),
	/// Other records not covered by specific types
	OtherRecordNumber(&'static str),
	/// Numeric PINs: set [0-9], typically 4-6 digits
	PersonalIdentificationNumber,
	/// Phone numbers: E.164 format, international and national formats, identified primarily by headers/labels
	PhoneNumber,
}

impl std::fmt::Display for NPIType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::AccountNumber => write!(f, "Account Number"),
			Self::BankIBAN => write!(f, "Bank IBAN"),
			Self::BankRoutingNumber => write!(f, "Bank Routing Number"),
			Self::BankSWIFTCode => write!(f, "Bank SWIFT/BIC Code"),
			Self::BiometricData => write!(f, "Biometric Data"),
			Self::CreditCardNumber => write!(f, "Credit Card Number"),
			Self::CryptoAddress => write!(f, "Crypto Address"),
			Self::DateOfBirth => write!(f, "Date of Birth"),
			Self::EmailAddress => write!(f, "Email Address"),
			Self::PersonalName => write!(f, "Full Name"),
			Self::GenderData => write!(f, "Gender Data"),
			Self::GPSLocation => write!(f, "GPS Location"),
			Self::IMEI => write!(f, "IMEI Number"),
			Self::MailingAddress => write!(f, "Mailing Address"),
			Self::NationalIdentificationNumber => write!(f, "National Identification Number"),
			Self::OtherIdentificationNumber(desc) => {
				write!(f, "Other Identification Number ({})", desc)
			}
			Self::OtherIdentity(desc) => write!(f, "Other Identity ({})", desc),
			Self::OtherPersonalData(desc) => write!(f, "Other Personal Data ({})", desc),
			Self::OtherRecordNumber(desc) => write!(f, "Other Record Number ({})", desc),
			Self::PersonalIdentificationNumber => write!(f, "Personal Identification Number"),
			Self::PhoneNumber => write!(f, "Phone Number"),
		}
	}
}
