# TLS Test Certificates

This directory contains test fixtures including self-signed TLS certificates for local development and testing.

## Files

- **tls.crt** - Self-signed X.509 certificate (PEM format)
- **tls.key** - RSA 2048-bit private key (PEM format)

## Certificate Details

```text
Subject: CN=localhost
Issuer: CN=localhost (self-signed)
Validity: 365 days from generation date
Key Type: RSA 2048-bit
Format: PEM (Privacy Enhanced Mail)
```

## Usage

Start the Dumptruck server with these certificates:

```bash
./target/debug/dumptruck serve \
  --cert tests/fixtures/tls.crt \
  --key tests/fixtures/tls.key \
  --port 8443 \
  --oauth-id test-client \
  --oauth-secret test-secret \
  --oauth-discovery-url https://oauth.example.com/.well-known/openid-configuration
```

## Security Notes

⚠️ **DEVELOPMENT ONLY**: These are self-signed certificates for local testing only.

- **Not for production**: Self-signed certs should never be used in production
- **No CA trust**: Browsers and clients will reject these as untrusted
- **No domain verification**: Not issued by a Certificate Authority

## For Production

For production use:

1. **Obtain real certificates** from a trusted Certificate Authority (Let's Encrypt, DigiCert, etc.)
2. **Use proper DNS records** and domain validation
3. **Implement certificate rotation** and monitoring
4. **Enable OCSP stapling** for certificate status checking
5. **Use strong cipher suites** (TLS 1.3+ as enforced by Dumptruck)

## Testing with curl

To test the server with these self-signed certs, disable certificate verification:

```bash
# Health check (ignoring cert warnings)
curl -k --http2 https://localhost:8443/api/v1/health \
  -H "Authorization: Bearer test-token-12345"

# Ingest request
curl -k --http2 -X POST https://localhost:8443/api/v1/ingest \
  -H "Authorization: Bearer test-token-12345" \
  -H "Content-Type: application/json" \
  -d '{"filename": "test.csv", "file_size_bytes": 1024}'
```

## Regenerating Test Certificates

If needed, regenerate these certificates with:

```bash
openssl req -x509 -newkey rsa:2048 \
  -keyout tests/fixtures/tls.key \
  -out tests/fixtures/tls.crt \
  -days 365 -nodes \
  -subj "/CN=localhost"
```

---

See [STRESS_TEST.md](../../STRESS_TEST.md) for stress testing with these certificates.
