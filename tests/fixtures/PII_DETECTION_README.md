# Comprehensive PII/NPI Detection Test Data

This directory contains test CSV files designed to trigger detection capabilities in Dumptruck's analysis pipeline. Each file targets specific detection scenarios.

## Test Data Files

### `unique_addresses.csv`

**Purpose**: Test email address canonicalization and deduplication

**Contains**:

- Email addresses with variations (gmail.com vs googlemail.com)
- Plus addressing (john+work at example.com)
- Underscore vs dot separators (john.smith vs john_smith)
- Different domain variants (gmail.com, yahoo.com, outlook.com, hotmail.com)

**Expected Detection**: Should identify that many addresses refer to the same person due to canonicalization rules.

### `hashed_credentials_detection.csv`

**Purpose**: Test detection of pre-hashed credentials in datasets

**Contains**:

- MD5 hashes of weak passwords (32 hex characters)
- SHA1 hashes (40 hex characters)
- SHA256 hashes (64 hex characters)
- Real hashes from the rainbow table (e.g., MD5 of "password")

**Expected Detection**: Each password hash should be recognized as a weak credential hash via rainbow table lookup.

### `weak_passwords.csv`

**Purpose**: Test weak password detection

**Contains**:

- Passwords from the rainbow table: "password", "admin", "123456", "qwerty", etc.
- Common keyboard patterns: "asdfgh", "zxcvbn", "1q2w3e4r"
- Simple numeric sequences: "123", "12345"
- Common phrases: "welcome", "letmein", "dragon", "sunshine"

**Expected Detection**: 20 weak passwords that appear in the rainbow table.

### `breached_addresses.csv`

**Purpose**: Test Have I Been Pwned (HIBP) integration for breach history

**Contains**:

- Email addresses known to be in major breaches:
    * Adobe (2013)
    * Equifax (2017)
    * Yahoo (2013)
    * LinkedIn (2012)
    * Twitter, Facebook, Dropbox, LastPass, etc.
- Number of breaches per account
- Breach dates and password change status

**Expected Detection**: This fixture is intended for breach-history enrichment workflows. The current CLI does not expose HIBP lookups.

### `pii_detection.csv`

**Purpose**: Test comprehensive PII/NPI detection

**Contains**:

- **Email addresses**: Various formats for identification
- **SSN**: Social Security Numbers (XXX-XX-XXXX format)
- **Credit Cards**: Valid Luhn algorithm numbers for Visa, Mastercard, Discover, American Express
- **Phone Numbers**: US format (555-XXX-XXXX)
- **National IDs**: Various country-specific ID formats
- **Mailing Addresses**: Street addresses for location analysis

**Expected Detection**:

- 15 rows, each containing multiple PII types
- Should identify all SSNs, credit card numbers, phone numbers, IDs, and addresses

### `crypto_and_financial_pii.csv`

**Purpose**: Test cryptocurrency and advanced financial identifier detection

**Contains**:

- **Bitcoin Addresses**: Multiple formats (P2PKH, P2SH, P2WPKH)
- **Ethereum Addresses**: 0x-prefixed 42-character hex addresses
- **Cryptocurrency Addresses**: XRP/Ripple (r-prefix), and other formats
- **IBAN**: International Bank Account Numbers (15-34 chars, country-specific)
- **SWIFT/BIC Codes**: 8 or 11 character bank codes
- **Phone Numbers**: International formats with country codes

**Expected Detection**:

- All 10 Bitcoin addresses should be detected
- All 10 Ethereum addresses should be detected
- All IBAN numbers should pass validation
- All SWIFT codes should be recognized

### `international_phone_ids.csv`

**Purpose**: Test multi-national phone number and ID detection

**Contains**:

- **Phone Numbers**: International formats from 15+ countries:
    * USA: +1-555-123-4567
    * UK: +44-20-7946-0958
    * Germany: +49-30-88792100
    * France, Spain, Italy, Japan, China, etc.
- **National IDs**: Country-specific formats:
    * USA: SSN (123-45-6789)
    * UK: NI Number (AB 12 34 56 C)
    * Germany: ID Number
    * France: Tax ID
    * Italy: Codice Fiscale (RSSMRA80A01H501X)
    * Japan: My Number
    * China: ID Card (18 digits)
    * India: Aadhaar Number

**Expected Detection**:

- All phone numbers should be recognized despite different formats
- All national IDs should be detected according to country-specific rules

## Usage

### Test single file

```bash
./dumptruck -v analyze "./tests/fixtures/unique_addresses.csv"
./dumptruck -v analyze "./tests/fixtures/pii_detection.csv"
```

### Test all detection files with logging

```bash
./dumptruck -vv analyze "./tests/fixtures/*detection*.csv"
./dumptruck -vv analyze "./tests/fixtures/*pii*.csv"
./dumptruck -vv analyze "./tests/fixtures/crypto_and_financial_pii.csv"
```

### Test breached-addresses fixture

```bash
./dumptruck -v analyze "./tests/fixtures/breached_addresses.csv"
```

### Test with Ollama embeddings

```bash
./dumptruck -v analyze "./tests/fixtures/unique_addresses.csv" --enable-embeddings
```

## Feature Availability Notes

- These fixtures are intended for use with `dumptruck analyze` and automated tests.
- HIBP lookups are not currently exposed via CLI flags.
- Embeddings are controlled via configuration and can be enabled for `analyze` with `--enable-embeddings`.

## Data Integrity Notes

All data in these test files are:

- **Synthetic**: No real credentials or personal information from actual leaks
- **Illustrative**: Examples of what the detection systems should identify
- **Structured**: CSV format for easy pipeline testing
- **Realistic**: Based on actual PII/NPI patterns from real breaches (patterns, not data)

The credit card numbers are valid per Luhn algorithm but are not real cards.
The SSNs follow the XXX-XX-XXXX format but are not real numbers.
The phone numbers use 555-XXXX ranges (reserved for testing in North America).
