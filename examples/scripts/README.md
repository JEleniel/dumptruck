# Dumptruck Operational Scripts

This directory contains production-ready bash scripts for key management and operational security procedures.

## Scripts

### rotate-keys.sh

Automated key rotation script for HMAC and API keys with grace period support.

**Usage:**

```bash
chmod +x rotate-keys.sh

# Rotate HMAC key
./rotate-keys.sh hmac

# Rotate API keys
./rotate-keys.sh api

# Rotate both
./rotate-keys.sh both
```

**Features:**

- Automated 32-byte HMAC key generation (openssl)
- API key validation with provider (HIBP example)
- 24-hour grace period (configurable: `GRACE_PERIOD_HOURS`)
- Both old and new keys accepted during transition
- Comprehensive logging to `/var/log/dumptruck/key-rotation.log`
- Error handling with rollback capability
- Post-rotation validation and service health checks

**Configuration:**

Edit script variables to customize:

```bash
DUMPTRUCK_USER="dumptruck"          # Service user
KEY_DIR="/etc/dumptruck"            # Keys location
GRACE_PERIOD_HOURS=24               # Grace period duration
LOG_FILE="/var/log/dumptruck/key-rotation.log"
```

### backup-keys.sh

Secure key backup and recovery script with encryption support.

**Usage:**

```bash
chmod +x backup-keys.sh

# Create encrypted backup
./backup-keys.sh backup

# Verify key integrity and permissions
./backup-keys.sh verify

# Show current key status
./backup-keys.sh status
```

**Features:**

- Automated tarball creation with all keys and metadata
- Optional GPG encryption (set `BACKUP_ENCRYPTION_KEY` environment variable)
- SHA256 checksums for integrity verification
- Metadata file with restoration instructions
- Key permission validation (600 - owner read/write only)
- Comprehensive logging to `/var/log/dumptruck/key-backup.log`
- Backup rotation and status reporting

**Configuration:**

Edit script variables to customize:

```bash
DUMPTRUCK_USER="dumptruck"
KEY_DIR="/etc/dumptruck"
BACKUP_DIR="/var/backups/dumptruck"
BACKUP_ENCRYPTION_KEY="${BACKUP_ENCRYPTION_KEY:-}"  # GPG recipient
LOG_FILE="/var/log/dumptruck/key-backup.log"
```

**Encryption:**

To enable GPG encryption:

```bash
export BACKUP_ENCRYPTION_KEY="your-gpg-key-id"
./backup-keys.sh backup
```

## Operational Workflows

### Initial Setup

```bash
# Generate HMAC key
openssl rand -base64 32 > /etc/dumptruck/hmac.key
chmod 600 /etc/dumptruck/hmac.key
chown dumptruck:dumptruck /etc/dumptruck/hmac.key

# Create backup
./backup-keys.sh backup
```

### Key Rotation Schedule

```bash
# Weekly key rotation (example cron)
0 2 * * 0 /path/to/rotate-keys.sh both >> /var/log/dumptruck/rotation-cron.log 2>&1

# Daily backup (example cron)
0 3 * * * /path/to/backup-keys.sh backup >> /var/log/dumptruck/backup-cron.log 2>&1
```

### Disaster Recovery

```bash
# Extract backup
cd /tmp
tar xzf /var/backups/dumptruck/dumptruck_keys_YYYYMMDD_HHMMSS.tar.gz

# Review restoration instructions
cat backup.metadata

# Restore keys
sudo cp hmac.key /etc/dumptruck/hmac.key
sudo cp hibp.key /etc/dumptruck/hibp.key
sudo chown dumptruck:dumptruck /etc/dumptruck/*.key
sudo chmod 600 /etc/dumptruck/*.key

# Verify restoration
./backup-keys.sh verify
```

## Security Considerations

1. **File Permissions:** All key files must be readable only by the dumptruck user (600)
2. **Ownership:** Keys should be owned by dumptruck:dumptruck
3. **Off-site Storage:** Keep backup copies in secure off-site locations
4. **Encryption:** Always encrypt backups with GPG for sensitive environments
5. **Testing:** Test restore procedures quarterly
6. **Logging:** Monitor key rotation and backup logs regularly
7. **Backup Integrity:** Verify backup checksums periodically

## Integration with Documentation

For comprehensive operational security procedures, see [SECURITY_OPS.md](../../docs/SECURITY_OPS.md):

- [Key Rotation and Management](../../docs/SECURITY_OPS.md#key-rotation-and-management)
- [Key Backup and Recovery](../../docs/SECURITY_OPS.md#key-backup-and-recovery)
- [Audit Logging](../../docs/SECURITY_OPS.md#audit-logging)
- [Incident Response](../../docs/SECURITY_OPS.md#incident-response)

## Requirements

- **bash 4.0+** - Script uses bash features
- **openssl** - For cryptographic key generation and validation
- **curl** - For API validation (optional, if using API key rotation)
- **gpg** - For backup encryption (optional but recommended)
- **systemctl** - For service status checks (optional)

## Logging

All operations are logged to:

- **Key Rotation:** `/var/log/dumptruck/key-rotation.log`
- **Key Backup:** `/var/log/dumptruck/key-backup.log`

Log format includes timestamps and color-coded severity levels (INFO, WARN, ERROR).

## Error Handling

Scripts implement comprehensive error handling:

- `set -e` - Exit on error
- Error messages with context
- Rollback procedures for partial failures
- Service health validation
- Log file verification

## Support

For additional operational guidance, see:

- [SECURITY_OPS.md](../../docs/SECURITY_OPS.md) - Complete security operations guide
- [CONFIGURATION.md](../../docs/CONFIGURATION.md) - Configuration reference
- [DEPLOYMENT.md](../../docs/architecture/DEPLOYMENT.md) - Deployment procedures
