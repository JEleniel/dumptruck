//! Test fixtures for NPI/PII detection
//! Contains real-world examples and known formats for testing

#[cfg(test)]
pub mod fixtures {
	/// Valid credit card test numbers (from payment processors' test suites)
	pub mod credit_cards {
		pub const VISA_BASIC: &str = "4532015112830366";
		pub const VISA_FORMATTED: &str = "4532-0151-1283-0366";
		pub const VISA_SPACES: &str = "4532 0151 1283 0366";

		pub const MASTERCARD_BASIC: &str = "5425233010103442";
		pub const MASTERCARD_FORMATTED: &str = "5425-2330-1010-3442";

		pub const AMEX_BASIC: &str = "378282246310005";
		pub const AMEX_FORMATTED: &str = "3782-822463-10005";

		pub const DISCOVER_BASIC: &str = "6011111111111117";
		pub const DISCOVER_FORMATTED: &str = "6011-1111-1111-1117";

		pub const DINERS_BASIC: &str = "30569309025904";
		pub const DINERS_FORMATTED: &str = "3056-930902-5904";

		pub const JCB_BASIC: &str = "3530111333300000";
		pub const JCB_FORMATTED: &str = "3530-1113-3330-0000";

		/// Invalid card (doesn't pass Luhn)
		pub const INVALID_CARD: &str = "4532015112830367";
	}

	/// Phone number formats from various countries
	pub mod phone_numbers {
		// US/Canada
		pub const US_BASIC: &str = "5551234567";
		pub const US_FORMATTED: &str = "+1-555-123-4567";
		pub const US_PARENTHESES: &str = "(555) 123-4567";
		pub const US_SPACES: &str = "555 123 4567";

		// UK
		pub const UK_FORMATTED: &str = "+44-20-7946-0958";
		pub const UK_MOBILE: &str = "+44-7911-123456";

		// Germany
		pub const DE_FORMATTED: &str = "+49-30-12345678";
		pub const DE_MOBILE: &str = "+49-170-123456789";

		// France
		pub const FR_FORMATTED: &str = "+33-1-42-68-53-00";
		pub const FR_MOBILE: &str = "+33-6-12-34-56-78";

		// Japan
		pub const JP_FORMATTED: &str = "+81-90-1234-5678";

		// Australia
		pub const AU_FORMATTED: &str = "+61-2-1234-5678";
		pub const AU_MOBILE: &str = "+61-4-1234-5678";

		// China
		pub const CN_MOBILE: &str = "+86-138-0013-8888";

		// Invalid (too short)
		pub const INVALID_SHORT: &str = "123-456";
		// Invalid (too long)
		pub const INVALID_LONG: &str = "+1-555-123-4567-ext-999";
	}

	/// US Social Security Numbers (test format)
	pub mod ssn {
		pub const BASIC: &str = "123-45-6789";
		pub const NO_FORMATTING: &str = "123456789";
		pub const FORMATTED_DASHES: &str = "987-65-4321";
		pub const FORMATTED_SPACES: &str = "555 12 3456";

		// Invalid SSNs
		pub const INVALID_ALL_ZEROS: &str = "000-00-0000";
		pub const INVALID_666: &str = "666-12-3456";
		pub const INVALID_ALL_NINES: &str = "999-99-9999";
	}

	/// National ID formats from various countries
	pub mod national_ids {
		// UK National Insurance Number
		pub const UK_NI_BASIC: &str = "AB123456C";
		pub const UK_NI_FORMATTED: &str = "AB 12 34 56 C";
		pub const UK_NI_DASHES: &str = "AB-12-34-56-C";

		// German Personalausweis (ID Card)
		pub const DE_ID_BASIC: &str = "1234567890";
		pub const DE_ID_FORMATTED: &str = "1234-5678-90";

		// French ID (Carte d'identité)
		pub const FR_ID_BASIC: &str = "1234567890123";
		pub const FR_ID_FORMATTED: &str = "12 345 678 901 23";

		// Spanish DNI (Documento Nacional de Identidad)
		pub const ES_DNI_BASIC: &str = "12345678Z";
		pub const ES_DNI_FORMATTED: &str = "1234-5678-Z";

		// Italian Codice Fiscale
		pub const IT_CF_BASIC: &str = "RSSMRA80A01H501T";
		pub const IT_CF_FORMATTED: &str = "RSS MRA 80A01 H501 T";

		// Portuguese NIF (Número de Identificação Fiscal)
		pub const PT_NIF_BASIC: &str = "123456789";
		pub const PT_NIF_FORMATTED: &str = "123 456 789";

		// Dutch BSN (Burgerservicenummer)
		pub const NL_BSN_BASIC: &str = "123456789";
		pub const NL_BSN_FORMATTED: &str = "123-456-789";

		// Belgian ID Number
		pub const BE_ID_BASIC: &str = "12345678901";
		pub const BE_ID_FORMATTED: &str = "123-456-789-01";

		// Swedish Personal Number
		pub const SE_PN_BASIC: &str = "8001011234";
		pub const SE_PN_FORMATTED: &str = "800101-1234";

		// Norwegian ID Number
		pub const NO_ID_BASIC: &str = "01010050006";
		pub const NO_ID_FORMATTED: &str = "010100 50006";

		// Canadian SIN (Social Insurance Number, 9 digits)
		pub const CA_SIN_BASIC: &str = "123456789";
		pub const CA_SIN_FORMATTED: &str = "123-456-789";

		// Japanese My Number
		pub const JP_MYNUMBER_BASIC: &str = "01234567890";
		pub const JP_MYNUMBER_FORMATTED: &str = "0123-4567-890";

		// Australian Tax File Number
		pub const AU_TFN_BASIC: &str = "123456789";
		pub const AU_TFN_FORMATTED: &str = "123 456 789";

		// Indian Aadhaar
		pub const IN_AADHAAR_BASIC: &str = "123456789012";
		pub const IN_AADHAAR_FORMATTED: &str = "1234 5678 9012";

		// Chinese ID (18 digits)
		pub const CN_ID_BASIC: &str = "110101199003071011";
		pub const CN_ID_FORMATTED: &str = "110101 1990 0307 1011";
	}

	/// Account numbers and financial identifiers
	pub mod account_numbers {
		// IBAN examples
		pub const IBAN_DE: &str = "DE89370400440532013000";
		pub const IBAN_GB: &str = "GB82WEST12345698765432";
		pub const IBAN_FR: &str = "FR1420041010050500013M02606";
		pub const IBAN_ES: &str = "ES9121000418450200051332";
		pub const IBAN_IT: &str = "IT60X0542811101000000123456";
		pub const IBAN_NL: &str = "NL91ABNA0417164300";

		// SWIFT/BIC codes
		pub const SWIFT_GERMAN: &str = "DEUTDEFF";
		pub const SWIFT_HSBC: &str = "HSBKGB2L";
		pub const SWIFT_BARCLAYS: &str = "BARCDEFF";

		// US Routing Number (9 digits)
		pub const US_ROUTING: &str = "021000021";
		pub const US_ROUTING_FORMATTED: &str = "021-000-021";

		// Bank Account Number (US)
		pub const US_ACCOUNT_BASIC: &str = "123456789";
		pub const US_ACCOUNT_FORMATTED: &str = "1234-5678-9";

		// Credit Union Account
		pub const CU_ACCOUNT: &str = "098765432";

		// PayPal/Digital Wallet
		pub const PAYPAL_ACCOUNT: &str = "email+unique123@example.com";

		// Crypto wallet addresses
		pub const BITCOIN_ADDRESS: &str = "1A1z7agoat5GkjM7E6vfj4FPVNwvH8K7p";
		pub const ETHEREUM_ADDRESS: &str = "0x742d35Cc6634C0532925a3b844Bc9e7595f42e0e";
		pub const ETHEREUM_ADDRESS_SHORT: &str = "0x742d35Cc6634C0532925a3b844Bc9e7595f42e0e";

		// Merchant account ID
		pub const STRIPE_ACCT: &str = "acct_1234567890abcdef";
		pub const SQUARE_ACCT: &str = "sq0asa-1234567890abcdef";
		pub const PAYPAL_MERCHANT: &str = "XXXXXXXXXXXX";

		// Apple Pay, Google Pay tokens
		pub const APPLE_PAY_TOKEN: &str = "9876543210987654";
		pub const GOOGLE_PAY_TOKEN: &str = "GPAY123456789ABCDEF";
	}

	/// Names for testing name detection
	pub mod names {
		pub const SIMPLE_NAME: &str = "John Doe";
		pub const THREE_PART_NAME: &str = "Mary Jane Watson";
		pub const HYPHENATED_FIRST: &str = "Jean-Pierre Dupont";
		pub const HYPHENATED_LAST: &str = "Maria Garcia-Lopez";
		pub const MIDDLE_INITIAL: &str = "James L Smith";
		pub const SUFFIX: &str = "William Johnson Jr";

		// Not names
		pub const LOWERCASE_FAIL: &str = "john doe";
		pub const NUMERIC_FAIL: &str = "John123 Doe456";
		pub const SINGLE_WORD_FAIL: &str = "John";
		pub const TOO_LONG_FAIL: &str =
			"Alexander Maximilian Friedrich Wilhelm Von Humboldt Von Neumann Extra";
	}

	/// Mailing addresses
	pub mod addresses {
		pub const US_STREET: &str = "123 Main Street, New York, NY 10001";
		pub const US_APARTMENT: &str = "456 Oak Avenue, Apt 200, Boston, MA 02108";
		pub const US_SUITE: &str = "789 Corporate Drive, Suite 500, San Francisco, CA 94105";
		pub const UK_ADDRESS: &str = "10 Downing Street, London, SW1A 2AA";
		pub const DE_ADDRESS: &str = "Musterstraße 42, 10115 Berlin, Germany";
		pub const FR_ADDRESS: &str = "123 Rue de la Paix, 75000 Paris, France";
		pub const AU_ADDRESS: &str = "42 Church Street, Sydney NSW 2000, Australia";

		// Not addresses
		pub const SHORT_FAIL: &str = "New York";
		pub const NO_NUMBER_FAIL: &str = "Main Street, Boston, MA";
		pub const NO_KEYWORDS_FAIL: &str = "123 Place Name 456";
	}

	/// IP addresses
	pub mod ips {
		pub const IPV4_PRIVATE: &str = "192.168.1.1";
		pub const IPV4_PUBLIC: &str = "8.8.8.8";
		pub const IPV4_LOCALHOST: &str = "127.0.0.1";
		pub const IPV4_BROADCAST: &str = "255.255.255.255";

		pub const IPV6_LOOPBACK: &str = "::1";
		pub const IPV6_FULL: &str = "2001:0db8:85a3:0000:0000:8a2e:0370:7334";
		pub const IPV6_SHORTENED: &str = "2001:db8:85a3::8a2e:370:7334";
		pub const IPV6_COMPRESSED: &str = "2001:db8::8a2e:370:7334";
		pub const IPV6_PRIVATE: &str = "fd12:3456:789a::1";

		// Not IPs
		pub const INVALID_IPV4: &str = "256.1.1.1";
		pub const INVALID_IPV4_INCOMPLETE: &str = "192.168.1";
		pub const INVALID_IPV6: &str = "gggg::1";
	}
}

#[cfg(test)]
mod fixture_tests {
	use super::fixtures::*;

	#[test]
	fn fixture_credit_cards_exist() {
		assert!(!credit_cards::VISA_BASIC.is_empty());
		assert!(!credit_cards::MASTERCARD_BASIC.is_empty());
	}

	#[test]
	fn fixture_phone_numbers_exist() {
		assert!(!phone_numbers::US_FORMATTED.is_empty());
		assert!(!phone_numbers::UK_FORMATTED.is_empty());
	}

	#[test]
	fn fixture_national_ids_exist() {
		assert!(!national_ids::UK_NI_BASIC.is_empty());
		assert!(!national_ids::DE_ID_BASIC.is_empty());
		assert!(!national_ids::JP_MYNUMBER_BASIC.is_empty());
		assert!(!national_ids::CN_ID_BASIC.is_empty());
	}

	#[test]
	fn fixture_account_numbers_exist() {
		assert!(!account_numbers::IBAN_DE.is_empty());
		assert!(!account_numbers::US_ROUTING.is_empty());
		assert!(!account_numbers::BITCOIN_ADDRESS.is_empty());
	}

	#[test]
	fn fixture_names_exist() {
		assert!(!names::SIMPLE_NAME.is_empty());
		assert!(!names::THREE_PART_NAME.is_empty());
	}

	#[test]
	fn fixture_addresses_exist() {
		assert!(!addresses::US_STREET.is_empty());
		assert!(!addresses::UK_ADDRESS.is_empty());
	}

	#[test]
	fn fixture_ips_exist() {
		assert!(!ips::IPV4_PRIVATE.is_empty());
		assert!(!ips::IPV6_FULL.is_empty());
	}
}
