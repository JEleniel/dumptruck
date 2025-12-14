# NPI/PII Test Fixtures

This document describes the comprehensive test fixtures available in `tests/npi_fixtures.rs` for testing PII/NPI detection and hashing functionality.

## Overview

The test fixtures module contains 100+ real-world examples covering:

- Credit card numbers (all major card types)
- Phone numbers (15+ countries)
- National ID formats (15+ countries)
- Bank account numbers and financial identifiers
- Cryptocurrency addresses
- Mailing addresses
- IP addresses (IPv4 and IPv6)
- Names

All fixtures are organized into namespaced modules for easy access in tests.

## Credit Card Fixtures

Located in `credit_cards` module:

- **Visa**: Basic, formatted with dashes and spaces
- **MasterCard**: Basic and formatted
- **American Express (AMEX)**: 15-digit format
- **Discover**: 16-digit format
- **Diners Club**: 14-digit format
- **JCB**: 16-digit format
- **Invalid card**: Sample card that fails Luhn validation

**Usage:**

```rust
use npi_fixtures::credit_cards::*;

assert!(is_credit_card(VISA_BASIC));
assert!(is_credit_card(AMEX_FORMATTED));
```

## Phone Number Fixtures

Located in `phone_numbers` module. Covers international formats from:

- **US/Canada**: Basic 10-digit, formatted with dashes/parentheses/spaces
- **UK**: 11-12 digits with +44 country code
- **Germany**: Mobile and landline formats
- **France**: Mobile and landline with +33
- **Japan**: 10-digit with +81
- **Australia**: Landline and mobile with +61
- **China**: 11-digit mobile with +86
- **Invalid examples**: Too short, too long, malformed

**Usage:**

```rust
use npi_fixtures::phone_numbers::*;

assert!(is_phone_number(US_FORMATTED));
assert!(is_phone_number(UK_MOBILE));
```

## Social Security Number Fixtures

Located in `ssn` module:

- **Valid formats**: Dashes (123-45-6789), no dashes (123456789), spaces
- **Invalid formats**: All zeros, 666 prefix, all nines

**Usage:**

```rust
use npi_fixtures::ssn::*;

assert!(is_ssn(BASIC));
assert!(!is_ssn(INVALID_ALL_ZEROS));
```

## National ID Fixtures

Located in `national_ids` module. Covers 15+ countries:

- **UK**: National Insurance Number (NI), 9 characters (2 letters + 6 digits + 1 letter)
- **Germany**: Personalausweis, 10 digits
- **France**: Carte d'identité, 13-15 digits
- **Spain**: DNI (Documento Nacional de Identidad), 8 digits + 1 letter
- **Italy**: Codice Fiscale, 16 alphanumeric characters
- **Portugal**: NIF, 9 digits
- **Netherlands**: BSN, 9 digits
- **Belgium**: ID number, 11 digits
- **Sweden**: Personal Number, 10 digits
- **Norway**: ID Number, 11 digits
- **Canada**: SIN (Social Insurance Number), 9 digits
- **Japan**: My Number, 11 digits
- **Australia**: Tax File Number (TFN), 9 digits
- **India**: Aadhaar, 12 digits
- **China**: ID Number, 18 digits

Each includes both unformatted and formatted variants.

**Usage:**

```rust
use npi_fixtures::national_ids::*;

assert!(is_national_id(UK_NI_BASIC));
assert!(is_national_id(UK_NI_FORMATTED));
assert!(is_national_id(CN_ID_BASIC));
```

## Account Number Fixtures

Located in `account_numbers` module:

### IBANs

- Germany, UK, France, Spain, Italy, Netherlands
- Both formatted and unformatted variants

### SWIFT/BIC Codes

- German, HSBC, Barclays
- 8 and 11 character variants

### Routing Numbers

- US routing number format (9 digits)
- Formatted with dashes

### Bank Accounts

- US routing and account numbers
- Credit union accounts
- Various lengths (8-17 digits)

### Cryptocurrency Addresses

- **Bitcoin**: 26-35 chars, legacy (starts with 1 or 3), segwit (bc1)
- **Ethereum**: 42 chars, 0x prefix
- **Short Ethereum**: Alternative format

### Digital Wallets & Merchant IDs

- Stripe account IDs (acct_ prefix)
- Square account IDs (sq0asa- prefix)
- PayPal merchant IDs
- Apple Pay tokens
- Google Pay tokens

**Usage:**

```rust
use npi_fixtures::account_numbers::*;

assert!(is_iban(IBAN_DE));
assert!(is_swift_code(SWIFT_GERMAN));
assert!(is_routing_number(US_ROUTING));
assert!(is_crypto_address(BITCOIN_ADDRESS));
```

## Name Fixtures

Located in `names` module:

- **Valid names**: Simple (John Doe), three-part, hyphenated first/last, middle initials, with suffix
- **Invalid names**: Lowercase, numeric, single word, too long

**Usage:**

```rust
use npi_fixtures::names::*;

assert!(is_name(SIMPLE_NAME));
assert!(!is_name(SINGLE_WORD_FAIL));
```

## Mailing Address Fixtures

Located in `addresses` module:

- **Valid addresses**: US (street, apartment, suite), UK, Germany, France, Australia
- **Invalid addresses**: Too short, no number, no keywords

**Usage:**

```rust
use npi_fixtures::addresses::*;

assert!(is_mailing_address(US_STREET));
assert!(!is_mailing_address(SHORT_FAIL));
```

## IP Address Fixtures

Located in `ips` module:

### IPv4

- Private ranges (192.168.x.x)
- Public addresses (8.8.8.8)
- Localhost (127.0.0.1)
- Broadcast (255.255.255.255)
- Invalid examples

### IPv6

- Loopback (::1)
- Full format
- Shortened format
- Compressed format
- Private addresses (fd00::/8)
- Invalid examples

**Usage:**

```rust
use npi_fixtures::ips::*;

assert!(is_ipv4(IPV4_PRIVATE));
assert!(is_ipv6(IPV6_LOOPBACK));
```

## Integration with Tests

The fixtures are designed to be imported into test files:

```rust
#[cfg(test)]
mod tests {
    use npi_fixtures::*;

    #[test]
    fn test_detection_with_fixtures() {
        let credit_card = detect_pii(credit_cards::VISA_BASIC, None);
        assert!(credit_card.contains(&PiiType::CreditCardNumber));
        
        let phone = detect_pii(phone_numbers::US_FORMATTED, None);
        assert!(phone.contains(&PiiType::PhoneNumber));
        
        let iban = detect_pii(account_numbers::IBAN_DE, None);
        assert!(iban.contains(&PiiType::IBAN));
    }
}
```

## Test Coverage

All fixtures are validated by `fixture_tests` module which ensures:

- Credit card fixtures exist and are non-empty
- Phone number fixtures exist and are non-empty
- National ID fixtures exist and are non-empty
- Account number fixtures exist and are non-empty
- Name fixtures exist and are non-empty
- Address fixtures exist and are non-empty
- IP address fixtures exist and are non-empty

Run fixture tests:

```bash
cargo test --test npi_fixtures
```

## Adding New Fixtures

To add fixtures for a new country or format:

1. Open `tests/npi_fixtures.rs`
2. Add a new constant in the appropriate module
3. Update fixture test to validate the new constant
4. Add unit test in `src/npi_detection.rs` to verify detection works
5. Run `cargo test` to verify

Example:

```rust
// In credit_cards module
pub const NEW_CARD_TYPE: &str = "9999999999999999";

// In fixture_tests
#[test]
fn fixture_credit_cards_exist() {
    assert!(!credit_cards::NEW_CARD_TYPE.is_empty());
}

// In npi_detection.rs tests
#[test]
fn test_new_card_detection() {
    assert!(is_credit_card(npi_fixtures::credit_cards::NEW_CARD_TYPE));
}
```

## Real-World Usage Notes

- All credit card numbers are from official payment processor test suites (not real accounts)
- Phone numbers follow country-specific digit length rules (10-15 digits typically)
- National ID formats vary significantly by country (6-18 digits)
- IBAN must be 15-34 alphanumeric chars, starting with 2-letter country code
- SWIFT codes must be 8 or 11 characters: 4 bank + 2 country + 3 location (+ optional 3 branch)
- Bitcoin addresses vary: legacy (1 or 3), segwit (bc1), all 26-35 characters
- Ethereum addresses are fixed 42 characters (0x + 40 hex digits)
- All values have been tested against real detection heuristics
