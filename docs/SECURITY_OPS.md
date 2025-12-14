# Security Operations Guide

This document describes operational security procedures for Dumptruck deployment and management.

## Overview

Security operations covers:

- Authentication and authorization
- Key and credential management
- Audit logging and monitoring
- Incident response
- Vulnerability management
- Data protection

## Authentication and Authorization

### OAuth 2.0 Client Credentials Flow

Dumptruck uses OAuth 2.0 client credentials for API authentication.

**Configuration:**

```json
{
  "oauth": {
    "enabled": true,
    "client_id_env": "OAUTH_CLIENT_ID",
    "client_secret_env": "OAUTH_CLIENT_SECRET"
  }
}
```

**Environment Variables:**

```bash
export OAUTH_CLIENT_ID="your-client-id"
export OAUTH_CLIENT_SECRET="your-client-secret"  # Keep this secret!
```

**Obtaining an Access Token:**

```bash
curl -X POST https://your-auth-provider/oauth/token \
  -u "$OAUTH_CLIENT_ID:$OAUTH_CLIENT_SECRET" \
  -d "grant_type=client_credentials" \
  -d "scope=dumptruck:ingest"
```

**Using the Token:**

```bash
curl -X POST https://dumptruck-server/ingest \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d @data.json
```

### TLS Certificate Management

Dumptruck requires TLS 1.3+ with valid certificates.

**Configuration:**

```json
{
  "server": {
    "tls": {
      "enabled": true,
      "cert_path": "/etc/dumptruck/tls/tls.crt",
      "key_path": "/etc/dumptruck/tls/tls.key",
      "min_version": "1.3"
    }
  }
}
```

**Creating Self-Signed Certificates (Development Only):**

```bash
# Generate private key
openssl genrsa -out tls.key 2048

# Generate certificate (valid for 365 days)
openssl req -new -x509 -key tls.key -out tls.crt -days 365 \
  -subj "/C=US/ST=State/L=City/O=Org/CN=dumptruck.example.com"

# Set appropriate permissions
chmod 600 tls.key
chmod 644 tls.crt
```

**Certificate Rotation Procedure:**

1. Generate new certificate and key
2. Test with staging environment
3. Update `tls.crt` and `tls.key` files
4. Restart Dumptruck server gracefully (existing connections complete, new connections use new cert)
5. Verify new certificate is active: `openssl s_client -connect dumptruck:8443`

**Monitoring Certificate Expiry:**

```bash
# Check certificate expiration date
openssl x509 -in /etc/dumptruck/tls/tls.crt -noout -dates

# Set up monitoring (example with 30-day warning)
#!/bin/bash
CERT_FILE="/etc/dumptruck/tls/tls.crt"
EXPIRY_DATE=$(openssl x509 -in "$CERT_FILE" -noout -dates | grep notAfter | cut -d= -f2)
DAYS_UNTIL_EXPIRY=$(( ($(date -d "$EXPIRY_DATE" +%s) - $(date +%s)) / 86400 ))

if [ "$DAYS_UNTIL_EXPIRY" -lt 30 ]; then
  echo "WARNING: Certificate expires in $DAYS_UNTIL_EXPIRY days"
  # Send alert via email/Slack/PagerDuty
fi
```

## Key Rotation and Management

### Automated Key Rotation and Backup

Dumptruck provides automated scripts for key rotation and backup operations. These scripts implement industry-standard key management practices with grace periods, secure encryption, and comprehensive logging.

**Available Scripts:**

- [rotate-keys.sh](../examples/scripts/rotate-keys.sh) - Automated HMAC and API key rotation
- [backup-keys.sh](../examples/scripts/backup-keys.sh) - Secure key backup and recovery

### HMAC Key Rotation

Dumptruck uses HMAC-SHA256 for certain internal operations.

**Key Generation:**

```bash
# Generate a 32-byte (256-bit) HMAC key
openssl rand -base64 32 > /etc/dumptruck/hmac.key

# Set secure permissions
chmod 600 /etc/dumptruck/hmac.key
chown dumptruck:dumptruck /etc/dumptruck/hmac.key
```

**Automated Key Rotation Procedure:**

Use the provided rotation script for automated key rotation:

```bash
# Make script executable
chmod +x /path/to/examples/scripts/rotate-keys.sh

# Rotate HMAC key
./examples/scripts/rotate-keys.sh hmac

# Rotate API keys
./examples/scripts/rotate-keys.sh api

# Rotate both
./examples/scripts/rotate-keys.sh both
```

The script automates:

1. New key generation
2. Backup of old keys
3. Grace period implementation (24 hours default, configurable)
4. Both old and new keys accepted during transition
5. Automatic key invalidation after grace period
6. Comprehensive logging to `/var/log/dumptruck/key-rotation.log`
7. Post-rotation validation and service checks

**Manual Key Rotation Procedure** (if not using automated script):

1. Generate new key: `openssl rand -base64 32 > /etc/dumptruck/hmac.key.new`
2. Update configuration to use new key
3. Allow grace period (e.g., 24 hours) where both keys are accepted
4. Log all key validation attempts during grace period
5. After grace period, invalidate old key
6. Monitor for authentication failures

**Secure Storage:**

- Store keys on encrypted filesystem
- Restrict file permissions (600 - owner read/write only)
- Use appropriate ownership (dumptruck user)
- Never commit keys to version control (use `.gitignore`)
- Use secrets management solution (e.g., HashiCorp Vault) in production

### Key Backup and Recovery

**Automated Backup Procedure:**

Use the provided backup script for automated key backup and recovery:

```bash
# Make script executable
chmod +x /path/to/examples/scripts/backup-keys.sh

# Create encrypted backup
./examples/scripts/backup-keys.sh backup

# Verify key integrity and permissions
./examples/scripts/backup-keys.sh verify

# Show current key status
./examples/scripts/backup-keys.sh status
```

**Backup Features:**

- Automated tarball creation with all keys and metadata
- Optional GPG encryption (configure with `BACKUP_ENCRYPTION_KEY` environment variable)
- SHA256 checksum for integrity verification
- Metadata file with backup information and restore instructions
- Automatic backup rotation (keep recent backups)
- Comprehensive logging to `/var/log/dumptruck/key-backup.log`

**Restore Procedure:**

```bash
# Extract backup
cd /tmp
tar xzf /var/backups/dumptruck/dumptruck_keys_YYYYMMDD_HHMMSS.tar.gz

# Review metadata
cat backup.metadata

# Restore keys (manual copy to preserve permissions)
sudo cp hmac.key /etc/dumptruck/hmac.key
sudo cp hibp.key /etc/dumptruck/hibp.key
sudo chown dumptruck:dumptruck /etc/dumptruck/*.key
sudo chmod 600 /etc/dumptruck/*.key

# Verify restoration
./backup-keys.sh verify
```

**Backup Storage Best Practices:**

- Store backups on separate encrypted systems
- Rotate backup media regularly
- Test restore procedures quarterly
- Keep off-site copies for disaster recovery
- Implement automated backup schedules
- Monitor backup completion and integrity
- Use immutable backup storage when possible

### API Key Management

**HIBP API Key:**

```json
{
  "hibp": {
    "api_key_env": "HIBP_API_KEY"
  }
}
```

**API Key Rotation:**

1. Request new API key from provider
2. Configure Dumptruck to use new key
3. Test with sample queries
4. Revoke old key at provider
5. Monitor for authentication failures
6. Verify no tools are still using old key

## Audit Logging

### Server Request Logging

Dumptruck logs all ingest requests:

```log
2025-12-13T10:23:45Z INFO dumptruck: POST /ingest client=10.0.0.5 bytes=2048 status=200 duration=125ms
2025-12-13T10:23:46Z INFO dumptruck: POST /ingest client=10.0.0.6 bytes=1024 status=400 error="invalid_format" duration=45ms
```

**Log Retention:**

- Archive logs for 90 days minimum
- Retain indices for 365 days
- Compress old logs to reduce storage

**Log Monitoring:**

```bash
# Monitor for failed authentication attempts
grep "status=401\|status=403" /var/log/dumptruck/server.log | tail -100

# Count ingest requests by client
grep "POST /ingest" /var/log/dumptruck/server.log | awk '{print $NF}' | sort | uniq -c | sort -rn

# Find slow requests (>1000ms)
grep "duration=[0-9]*[0-9][0-9][0-9][0-9]ms" /var/log/dumptruck/server.log
```

### Database Audit Trail

PostgreSQL audit logging (pgAudit extension recommended):

```sql
-- Enable pgaudit extension
CREATE EXTENSION pgaudit;

-- Log all DML operations
SET pgaudit.log = 'WRITE';

-- Log with user information
SET pgaudit.log_statement = on;

-- View audit logs
SELECT usename, sessionid, statement_timestamp, statement, command_tag 
FROM pg_audit_log 
WHERE username = 'dumptruck' 
ORDER BY statement_timestamp DESC;
```

## Incident Response

### Suspected Breach or Compromise

If you suspect Dumptruck or associated systems have been compromised:

1. **Immediate Actions:**
   + Isolate affected systems from network
   + Preserve logs and memory dumps for forensics
   + Do NOT restart systems (preserves volatile evidence)
   + Notify security team and leadership

2. **Investigation:**
   + Review authentication logs for unauthorized access
   + Check database audit logs for suspicious queries
   + Scan for malware/rootkits on compromised system
   + Review network traffic captures
   + Examine file integrity (e.g., checksums of binaries)

3. **Containment:**
   + Revoke all API keys and credentials
   + Rotate all certificates and keys
   + Block IP addresses of attackers
   + Reset database credentials
   + Patch identified vulnerabilities

4. **Recovery:**
   + Restore from clean backups
   + Verify integrity of restored systems
   + Re-credential with new keys
   + Gradually bring systems back online
   + Monitor for signs of re-compromise

5. **Post-Incident:**
   + Conduct post-mortem analysis
   + Document timeline and findings
   + Update security procedures
   + Notify affected users/customers
   + Report to relevant authorities if required

### Memory/Credential Leak in Logs

If plaintext credentials are exposed in logs:

1. **Immediate:**
   + Revoke compromised credentials
   + Remove affected log files
   + Notify affected systems

2. **Investigation:**
   + Determine exposure timeline
   + Identify all affected systems
   + Check for potential abuse of credentials

3. **Prevention:**
   + Implement log redaction for sensitive fields
   + Use structured logging with field masking
   + Implement secrets scanning in CI/CD
   + Regular log rotation to limit retention

**Example log redaction (Python):**

```python
import re
import logging

class RedactingFormatter(logging.Formatter):
    def format(self, record):
        message = super().format(record)
        # Mask API keys, passwords, tokens
        message = re.sub(r'(?:password|api_key|token)=\S+', 
                        lambda m: m.group(0).split('=')[0] + '=***REDACTED***', 
                        message, flags=re.IGNORECASE)
        return message

handler = logging.StreamHandler()
handler.setFormatter(RedactingFormatter('%(asctime)s %(name)s %(levelname)s %(message)s'))
```

## Vulnerability Management

### Dependency Scanning

Automatically scan for vulnerable dependencies:

```bash
# Run security audit
cargo audit

# Check for outdated dependencies
cargo outdated

# Generate SBOM (Software Bill of Materials)
cargo sbom --format json > sbom.json
```

### Patch Management Procedure

1. **Weekly:**
   + Review security advisories (GitHub Security, RustSec, etc.)
   + Run `cargo audit` to identify vulnerable dependencies

2. **Upon Finding Vulnerability:**
   + Assess severity and exploitability
   + Check if vulnerability affects Dumptruck
   + Determine patch availability

3. **Patching Priority:**
   + **Critical (CVSS 9.0+):** Patch within 24 hours
   + **High (7.0-8.9):** Patch within 7 days
   + **Medium (4.0-6.9):** Patch within 30 days
   + **Low (<4.0):** Patch in next regular release

4. **Release Process:**
   + Update vulnerable dependency
   + Run full test suite
   + Tag security release (e.g., `v1.2.4-security`)
   + Publish release with security advisory

### Security Advisory Release

Example security advisory:

```markdown
# Security Advisory: Remote Code Execution in Dependency X

## Summary
Dumptruck versions prior to v1.2.4 are vulnerable to remote code 
execution through dependency X version < 1.0.2.

## Affected Versions
- Dumptruck < v1.2.4

## Severity
Critical (CVSS 9.8)

## Impact
An unauthenticated attacker can achieve remote code execution by 
sending a specially crafted request.

## Resolution
Upgrade to Dumptruck v1.2.4 or later.

## Workaround
Until upgrade is possible, restrict network access to Dumptruck 
to trusted clients only.

## Timeline
- 2025-12-10: Vulnerability reported
- 2025-12-11: Investigation confirmed
- 2025-12-13: Dumptruck v1.2.4 released
- 2025-12-15: Public disclosure

## References
- https://github.com/jeleniel/dumptruck/releases/tag/v1.2.4
- https://nvd.nist.gov/vuln/detail/CVE-XXXX-XXXXX
```

## Data Protection

### Database Encryption

**Transparent Data Encryption (TDE) with PostgreSQL:**

```sql
-- Use pgcrypto extension for column-level encryption
CREATE EXTENSION pgcrypto;

-- Encrypt sensitive columns
ALTER TABLE credentials 
ADD COLUMN hash_encrypted BYTEA,
ADD COLUMN encrypted_key BYTEA;

-- Insert encrypted data
INSERT INTO credentials (hash_encrypted) 
VALUES (encrypt('password_hash'::bytea, 'key'::bytea, 'aes'));

-- Decrypt for queries
SELECT decrypt(hash_encrypted, 'key'::bytea, 'aes')::text 
FROM credentials;
```

**Full Disk Encryption:**

```bash
# Linux: LUKS full disk encryption
sudo cryptsetup luksFormat /dev/sdX
sudo cryptsetup luksOpen /dev/sdX dumptruck-data
sudo mkfs.ext4 /dev/mapper/dumptruck-data

# Set up mount
sudo mkdir -p /mnt/dumptruck-data
echo 'dumptruck-data /dev/sdX /etc/dumptruck/keyfile' | sudo tee -a /etc/crypttab
sudo mount /dev/mapper/dumptruck-data /mnt/dumptruck-data
```

### Secure Deletion

Before decommissioning systems, securely delete sensitive data:

```bash
# Securely overwrite deleted data
shred -vfz -n 5 /etc/dumptruck/hmac.key

# Or use dd
dd if=/dev/zero of=/dev/sdX bs=1M status=progress
```

## Monitoring and Alerting

### Key Metrics to Monitor

1. **Authentication:**
   + Failed login attempts
   + API key usage patterns
   + Certificate expiry (< 30 days warning)

2. **Performance:**
   + Request latency (alert on >1000ms)
   + Error rate (alert on >1%)
   + Database connection pool exhaustion

3. **Security:**
   + Unusual query patterns
   + Large data exports
   + Configuration changes
   + Privilege escalation attempts

### Alert Configuration (Example)

```yaml
alerts:
  - name: FailedAuthAttempts
    query: |
      SELECT COUNT(*) FROM server_logs 
      WHERE status IN (401, 403) 
      AND timestamp > now() - interval '5 minutes'
    threshold: 10
    action: "notify-security-team"

  - name: CertificateExpiringSoon
    query: |
      SELECT days_until_expiry FROM certificate_status 
      WHERE name = 'dumptruck-tls'
    threshold: 30
    action: "notify-ops-team"

  - name: HighErrorRate
    query: |
      SELECT SUM(CASE WHEN status >= 500 THEN 1 ELSE 0 END)::float 
      / COUNT(*) FROM server_logs 
      WHERE timestamp > now() - interval '5 minutes'
    threshold: 0.01
    action: "page-on-call"
```

## Security Checklist

Before deploying to production:

- [ ] TLS 1.3+ enabled with valid certificates
- [ ] OAuth 2.0 client credentials configured
- [ ] HMAC keys generated and securely stored
- [ ] API keys (HIBP) securely configured
- [ ] Database encryption enabled
- [ ] Full disk encryption enabled
- [ ] Audit logging configured and monitored
- [ ] Log retention policies defined
- [ ] Incident response procedures documented
- [ ] Dependency scanning automated (cargo audit in CI)
- [ ] Regular security patching process defined
- [ ] Vulnerability disclosure process documented
- [ ] Data protection procedures documented
- [ ] Security monitoring and alerting configured
- [ ] Disaster recovery plan tested
- [ ] Access controls and permissions reviewed
- [ ] Cryptographic random number generation verified

## References

- [OWASP Security Cheat Sheet](https://cheatsheetseries.owasp.org/)
- [CIS Controls](https://www.cisecurity.org/controls)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [RustSec Advisory Database](https://rustsec.org/)
- [PostgreSQL Security](https://www.postgresql.org/docs/current/sql-security.html)
