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
			Self::AccountNumber => write!(f, "{}", Self::ACCOUNT_NUMBER),
			Self::BankIBAN => write!(f, "{}", Self::BANK_IBAN),
			Self::BankRoutingNumber => write!(f, "{}", Self::BANK_ROUTING_NUMBER),
			Self::BankSWIFTCode => write!(f, "{}", Self::BANK_SWIFT_CODE),
			Self::BiometricData => write!(f, "{}", Self::BIOMETRIC_DATA),
			Self::CreditCardNumber => write!(f, "{}", Self::CREDIT_CARD_NUMBER),
			Self::CryptoAddress => write!(f, "{}", Self::CRYPTO_ADDRESS),
			Self::DateOfBirth => write!(f, "{}", Self::DATE_OF_BIRTH),
			Self::EmailAddress => write!(f, "{}", Self::EMAIL_ADDRESS),
			Self::PersonalName => write!(f, "{}", Self::PERSONAL_NAME),
			Self::GenderData => write!(f, "{}", Self::GENDER_DATA),
			Self::GPSLocation => write!(f, "{}", Self::GPS_LOCATION),
			Self::IMEI => write!(f, "{}", Self::IMEI),
			Self::MailingAddress => write!(f, "{}", Self::MAILING_ADDRESS),
			Self::NationalIdentificationNumber => {
				write!(f, "{}", Self::NATIONAL_IDENTIFICATION_NUMBER)
			}
			Self::OtherIdentificationNumber(desc) => {
				write!(f, "{} ({})", Self::OTHER_IDENTIFICATION_NUMBER, desc)
			}
			Self::OtherIdentity(desc) => write!(f, "{} ({})", Self::OTHER_IDENTITY, desc),
			Self::OtherPersonalData(desc) => write!(f, "{} ({})", Self::OTHER_PERSONAL_DATA, desc),
			Self::OtherRecordNumber(desc) => write!(f, "{} ({})", Self::OTHER_RECORD_NUMBER, desc),
			Self::PersonalIdentificationNumber => {
				write!(f, "{}", Self::PERSONAL_IDENTIFICATION_NUMBER)
			}
			Self::PhoneNumber => {
				write!(f, "{}", Self::PHONE_NUMBER)
			}
		}
	}
}

impl NPIType {
	pub const ACCOUNT_NUMBER: &'static str = "Account Number";
	pub const BANK_IBAN: &'static str = "Bank IBAN";
	pub const BANK_ROUTING_NUMBER: &'static str = "Bank Routing Number";
	pub const BANK_SWIFT_CODE: &'static str = "Bank SWIFT/BIC Code";
	pub const BIOMETRIC_DATA: &'static str = "Biometric Data";
	pub const CREDIT_CARD_NUMBER: &'static str = "Credit Card Number";
	pub const CRYPTO_ADDRESS: &'static str = "Crypto Address";
	pub const DATE_OF_BIRTH: &'static str = "Date of Birth";
	pub const EMAIL_ADDRESS: &'static str = "Email Address";
	pub const PERSONAL_NAME: &'static str = "Full Name";
	pub const GENDER_DATA: &'static str = "Gender Data";
	pub const GPS_LOCATION: &'static str = "GPS Location";
	pub const IMEI: &'static str = "IMEI Number";
	pub const MAILING_ADDRESS: &'static str = "Mailing Address";
	pub const NATIONAL_IDENTIFICATION_NUMBER: &'static str = "National Identification Number";
	pub const OTHER_IDENTIFICATION_NUMBER: &'static str = "Other Identification Number";
	pub const OTHER_IDENTITY: &'static str = "Other Identity";
	pub const OTHER_PERSONAL_DATA: &'static str = "Other Personal Data";
	pub const OTHER_RECORD_NUMBER: &'static str = "Other Record Number";
	pub const PERSONAL_IDENTIFICATION_NUMBER: &'static str = "Personal Identification Number";
	pub const PHONE_NUMBER: &'static str = "Phone Number";
}
