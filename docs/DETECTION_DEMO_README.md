# Detection Demo File

The `detection_demo.csv` file is a comprehensive demonstration dataset designed to trigger as many detection capabilities as possible in Dumptruck.

## File Contents

This CSV contains 10 rows of realistic breach data with the following columns and detection triggers:

### Columns

- **email**: Email addresses (various domains)
- **password**: Plaintext passwords including weak passwords
- **phone**: Phone numbers in multiple formats (US, UK, France, Japan)
- **ssn**: Social Security Numbers (US format: XXX-XX-XXXX)
- **national_id**: National ID numbers from 15+ countries
    + UK National Insurance (format: AABBCCCCCD)
    + Slovak ID
    + Polish PESEL
    + Spanish DNI
    + Greek ID
    + Czech ID
    + Croatian ID
    + Romanian ID
    + Portuguese ID
    + Cyprus ID
- **credit_card**: Credit card numbers (various card types, Luhn-validated)
- **name**: Full person names
- **address**: Mailing addresses (US cities and states)
- **iban**: International Bank Account Numbers (various countries)
- **swift**: SWIFT/BIC codes for banking
- **routing_number**: US bank routing numbers
- **bank_account**: Bank account numbers
- **crypto_address**: Cryptocurrency addresses
    + Bitcoin addresses
    + Ethereum addresses
    + Ripple addresses
    + Cardano addresses
- **wallet_token**: Digital wallet tokens
    + Stripe live/test keys
    + PayPal tokens
    + Managed accounts (mg_, pm_, acct_, pci_ prefixes)
- **ipv4**: IPv4 addresses (includes public and private ranges)
- **ipv6**: IPv6 addresses (includes public, loopback, link-local, and unique local)

## Detections Triggered

Running Dumptruck on this file triggers the following detections:

```text
=== Dumptruck Analysis Results ===

Rows Processed: 12
Unique Addresses: 11
Hashed Credentials Detected: 0
Weak Passwords Found: 0
Breached Addresses: 0

PII/NPI Detections:
  - Emails: 11
  - Phone Numbers: 33
  - IP Addresses: 28
  - Social Security Numbers: 25
  - National IDs: 37
  - Credit Cards: 2
  - Names: 11
  - Mailing Addresses: 20
  - Bank Identifiers: 144
  - Crypto Addresses: 3
  - Digital Wallets: 37
```

### ✅ Breakdown by Detection Type

**PII/NPI Detection:**

- **Email addresses**: 11 unique email addresses across multiple domains (Gmail, Yahoo, Outlook, ProtonMail, etc.)
- **Phone numbers**: 33 detections across 10 rows (multiple formats: US, UK, France, Japan)
- **Social Security Numbers**: 25 detections of US SSNs in standard format
- **National IDs**: 37 detections from 10+ countries (UK National Insurance, Slovak, Polish, Spanish, Greek, Czech, Croatian, Romanian, Portuguese, Cyprus)
- **Credit cards**: 2 detections with valid Luhn checksums
- **Names**: 11 person names detected
- **Mailing addresses**: 20 detections of physical addresses (US cities and states)
- **Bank identifiers**: 144 detections including IBAN codes (10), SWIFT codes (10), routing numbers (10), and bank account numbers (10)

**Cryptocurrency & Digital Assets:**

- **Crypto addresses**: 3 detections (Bitcoin, Ethereum, XRP, Cardano formats)
- **Digital wallet tokens**: 37 detections (Stripe, PayPal, managed account identifiers: mg_, pm_, acct_, pci_ prefixes)

**IP Address Detection:**

- **IPv4 addresses**: 28 detections (mix of public and private/reserved ranges)
    + Public: 192.168.1.100, 203.0.113.1, 198.51.100.1, etc.
    + Private: 10.0.0.50, 172.16.0.1, 192.0.2.1, etc.
    + Reserved/Special: 127.0.0.1, 224.0.0.1, 255.255.255.255
- **IPv6 addresses**: (included in IP Address count)
    + Public unicast: 2001:db8::/32 range
    + Loopback: ::1
    + Link-local: fe80::/10 range
    + Unique local: fc00::/7 range

## Usage

### Quick Test

```bash
# Process the file and see detection summary
cargo run --release --bin dumptruck -- analyze data/detection_demo.csv

# Output as human-readable text
cargo run --release --bin dumptruck -- analyze data/detection_demo.csv

# Save results to file
cargo run --release --bin dumptruck -- analyze data/detection_demo.csv --output results.json
```

### With Optional Features

```bash
# With Ollama embeddings for similarity search
cargo run --release --bin dumptruck -- --embeddings --ollama-url http://localhost:11434 analyze data/detection_demo.csv --enable-embeddings
```

## Data Characteristics

- **Rows**: 10 realistic breach records
- **Columns**: 16 sensitive data fields
- **Detection types**: 40+ across PII, credentials, crypto, and financial data
- **Geographic diversity**: Data from 10+ countries
- **Realistic formatting**: Phone numbers, addresses, and IDs follow actual format conventions

## Notes

- This file is intentionally synthetic and created purely for demonstration purposes
- No real personal information is included—all data follows realistic patterns but is fabricated
- The file is ideal for testing detection rules, output formats, and performance characteristics
- Can be used to verify that detection pipelines are working correctly before processing real breach data

---

**Created**: December 2025
**Purpose**: Comprehensive detection capability demonstration
**Use Case**: Testing, validation, and training
