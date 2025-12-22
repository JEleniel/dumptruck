# Detection Endpoint Examples

This document provides usage examples for the `/detect` endpoint of the Dumptruck API.

## Endpoint

**POST** `/detect`

Accepts CSV/TSV/JSON data and returns PII/NPI detection results with detailed findings per row.

## Basic Request

### CSV Data

```bash
curl -X POST http://localhost:8443/detect \
  -H "Content-Type: text/csv" \
  -d "email,password,age
user@example.com,password123,25
john.doe@gmail.com,weak_pwd,30"
```

### JSON Data

```bash
curl -X POST http://localhost:8443/detect \
  -H "Content-Type: application/json" \
  -d '[
    {"email": "user@example.com", "password": "password123", "age": 25},
    {"email": "john.doe@gmail.com", "password": "weak_pwd", "age": 30}
  ]'
```

## Response Example

```json
{
  "rows_processed": 2,
  "unique_addresses": 2,
  "hashed_credentials_detected": 0,
  "weak_passwords_found": 2,
  "breached_addresses": 0,
  "pii_summary": {
    "email_count": 2,
    "phone_count": 0,
    "ssn_count": 0,
    "credit_card_count": 0,
    "rows_with_pii": 2,
    "pii_types_detected": ["email", "weak_password"]
  },
  "detailed_findings": [
    {
      "row_number": 2,
      "detections": [
        {
          "field_name": "email",
          "field_value": "user@example.com",
          "pii_type": "email",
          "confidence": 1.0,
          "risk_level": "info"
        },
        {
          "field_name": "password",
          "field_value": "password123",
          "pii_type": "weak_password",
          "confidence": 1.0,
          "risk_level": "high"
        }
      ]
    },
    {
      "row_number": 3,
      "detections": [
        {
          "field_name": "email",
          "field_value": "john.doe@gmail.com",
          "pii_type": "email",
          "confidence": 1.0,
          "risk_level": "info"
        },
        {
          "field_name": "password",
          "field_value": "weak_pwd",
          "pii_type": "weak_password",
          "confidence": 1.0,
          "risk_level": "high"
        }
      ]
    }
  ],
  "metadata": [],
  "errors": []
}
```

## Request Parameters

The endpoint accepts the following optional query parameters:

| Parameter | Type | Description |
|-----------|------|-------------|
| `hibp` | boolean | Enable HIBP breach enrichment (default: false) |
| `embeddings` | boolean | Enable vector similarity search for near-duplicates (default: false) |
| `verbose` | integer | Verbosity level (0-2, default: 0) |
| `has_headers` | boolean | Treat first row as headers (default: true for CSV/TSV, false for JSON) |

## Examples with Options

### Enable HIBP Enrichment

```bash
curl -X POST "http://localhost:8443/detect?hibp=true" \
  -H "Content-Type: text/csv" \
  -d "email
pwned@example.com"
```

Response will include breach information:

```json
{
  "breached_addresses": 1,
  ...
  "detailed_findings": [
    {
      "row_number": 2,
      "detections": [
        {
          "field_name": "email",
          "field_value": "pwned@example.com",
          "pii_type": "email",
          "confidence": 1.0,
          "risk_level": "critical",
          "breach_data": {
            "breach_count": 3,
            "breaches": ["Adobe", "LinkedIn", "Yahoo"]
          }
        }
      ]
    }
  ]
}
```

### Enable Embeddings for Similarity Search

```bash
curl -X POST "http://localhost:8443/detect?embeddings=true" \
  -H "Content-Type: text/csv" \
  -d "username
john_doe
jon_doe"
```

### Combined Options

```bash
curl -X POST "http://localhost:8443/detect?hibp=true&embeddings=true&verbose=1" \
  -H "Content-Type: text/csv" \
  -d "email,password
user@example.com,SecurePass123!"
```

## Detection Types

The endpoint detects the following PII/NPI types:

### Contact Information

- `email` - Email addresses
- `phone_number` - Phone numbers
- `mailing_address` - Physical addresses

### Identifiers

- `national_id` - National ID numbers (15+ countries)
- `ssn` - Social Security Numbers (US)
- `credit_card` - Credit card numbers

### Financial Information

- `iban` - IBAN bank account numbers
- `swift_code` - SWIFT codes
- `bank_account` - Bank account numbers
- `routing_number` - Routing numbers

### Digital Assets

- `crypto_address` - Cryptocurrency addresses
- `digital_wallet` - Digital wallet tokens

### Network Information

- `ip_address` - IP addresses (IPv4 and IPv6)
- `ipv4` - IPv4 addresses
- `ipv6` - IPv6 addresses

### Personal Information

- `name` - Person's name

### Security Credentials

- `weak_password` - Plaintext weak passwords
- `hashed_weak_password` - Weak password hashes (bcrypt, argon2, scrypt, PBKDF2, MD5, SHA1, SHA256, SHA512, NTLM)

### Anomalies

- `entropy_outlier` - High entropy values suggesting encrypted data or hashes
- `unusual_format` - Non-standard field structures
- `rare_domain` - Infrequent top-level domains

## Risk Levels

Detected findings are assigned risk levels:

| Level | Description |
|-------|-------------|
| `info` | Informational (e.g., email addresses) |
| `warning` | Moderate risk (e.g., weak passwords) |
| `high` | High risk (e.g., credit cards, SSNs) |
| `critical` | Critical risk (e.g., breached credentials) |

## Error Handling

If an error occurs, the response includes error messages:

```json
{
  "rows_processed": 0,
  "unique_addresses": 0,
  "hashed_credentials_detected": 0,
  "weak_passwords_found": 0,
  "breached_addresses": 0,
  "pii_summary": null,
  "detailed_findings": [],
  "metadata": [],
  "errors": [
    "Invalid CSV format: missing closing quote at line 2",
    "File size exceeds 100MB limit"
  ]
}
```

## Integration Examples

### Python

```python
import requests
import json

# Detect PII in CSV data
response = requests.post(
    'http://localhost:8443/detect',
    headers={'Content-Type': 'text/csv'},
    data='email,password\nuser@example.com,weak123'
)

result = response.json()
print(f"Found {result['unique_addresses']} unique addresses")
print(f"Weak passwords: {result['weak_passwords_found']}")

for finding in result['detailed_findings']:
    print(f"\nRow {finding['row_number']}:")
    for detection in finding['detections']:
        print(f"  - {detection['pii_type']}: {detection['risk_level']}")
```

### JavaScript/Node.js

```javascript
const response = await fetch('http://localhost:8443/detect', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json'
  },
  body: JSON.stringify([
    { email: 'user@example.com', password: 'weak123' },
    { email: 'another@example.com', password: 'strong!' }
  ])
});

const result = await response.json();
console.log(`Weak passwords found: ${result.weak_passwords_found}`);

result.detailed_findings.forEach(row => {
  console.log(`\nRow ${row.row_number}:`);
  row.detections.forEach(detection => {
    console.log(`  ${detection.pii_type} (${detection.risk_level})`);
  });
});
```

### cURL with File

```bash
# Detect PII in a CSV file
curl -X POST http://localhost:8443/detect \
  -H "Content-Type: text/csv" \
  -d @data.csv

# Pretty-print JSON response
curl -X POST http://localhost:8443/detect \
  -H "Content-Type: text/csv" \
  -d @data.csv | jq '.'
```

## Performance Notes

- CSV/TSV parsing is streaming and memory-efficient
- Files larger than 100MB are rejected
- Compression (ZIP/gzip) is automatically detected
- Parallel processing with configurable worker threads
- Response time typically sub-100ms for 1KB-1MB data

## Security Considerations

- All detected PII values are included in the response for analyst review
- For production use, consider:
    + Enabling TLS 1.3+ (default in server mode)
    + Restricting IP access to trusted networks
    + Enabling OAuth 2.0 authentication
    + Implementing audit logging
    + Masking sensitive values in logs

See [SECURITY_OPS.md](docs/SECURITY_OPS.md) for security operational procedures.
