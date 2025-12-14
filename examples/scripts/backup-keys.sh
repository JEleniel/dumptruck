#!/bin/bash
#
# Dumptruck Key Backup and Recovery Script
#
# This script provides procedures for:
# - Secure backup of cryptographic keys
# - Encryption of backups
# - Safe key recovery
# - Key verification
#
# Usage:
#   ./backup-keys.sh [backup|restore|verify]
#
# Prerequisites:
#   - GPG installed for encryption (optional but recommended)
#   - Sufficient disk space for backups
#   - Proper file permissions
#

set -e

# Configuration
DUMPTRUCK_USER="dumptruck"
KEY_DIR="/etc/dumptruck"
BACKUP_DIR="/var/backups/dumptruck"
BACKUP_ENCRYPTION_KEY="${BACKUP_ENCRYPTION_KEY:-}"  # GPG recipient, if available
LOG_FILE="/var/log/dumptruck/key-backup.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1" | tee -a "$LOG_FILE"
}

# Create necessary directories
mkdir -p "$BACKUP_DIR"
mkdir -p "$(dirname "$LOG_FILE")"

# Function: Backup keys
backup_keys() {
    log_info "=== Starting Key Backup ==="
    log_info "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    
    BACKUP_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    BACKUP_NAME="dumptruck_keys_$BACKUP_TIMESTAMP"
    BACKUP_TAR="$BACKUP_DIR/$BACKUP_NAME.tar.gz"
    
    # Create temporary directory for staging
    TEMP_BACKUP=$(mktemp -d)
    log_debug "Staging backup in: $TEMP_BACKUP"
    
    # Copy keys
    if [[ -f "$KEY_DIR/hmac.key" ]]; then
        cp "$KEY_DIR/hmac.key" "$TEMP_BACKUP/"
        log_info "✓ Copied HMAC key"
    else
        log_warn "HMAC key not found"
    fi
    
    if [[ -f "$KEY_DIR/hibp.key" ]]; then
        cp "$KEY_DIR/hibp.key" "$TEMP_BACKUP/"
        log_info "✓ Copied HIBP API key"
    else
        log_warn "HIBP API key not found"
    fi
    
    # Create metadata file
    cat > "$TEMP_BACKUP/backup.metadata" << EOF
Dumptruck Key Backup
====================
Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Host: $(hostname)
User: $(whoami)
Keys included:
  - HMAC key: $(test -f "$KEY_DIR/hmac.key" && echo "yes" || echo "no")
  - HIBP API key: $(test -f "$KEY_DIR/hibp.key" && echo "yes" || echo "no")
  
Restore instructions:
  1. Extract: tar xzf $BACKUP_TAR
  2. Review: cat backup.metadata
  3. Restore: ./restore-keys.sh
  4. Verify: ./verify-keys.sh
  
KEEP THIS BACKUP SECURE. It contains sensitive cryptographic keys.
EOF
    
    log_info "Created backup metadata"
    
    # Create tarball
    tar czf "$BACKUP_TAR" -C "$TEMP_BACKUP" .
    chmod 600 "$BACKUP_TAR"
    
    log_info "✓ Created backup: $BACKUP_TAR"
    log_info "  Size: $(du -h "$BACKUP_TAR" | cut -f1)"
    log_info "  SHA256: $(sha256sum "$BACKUP_TAR" | cut -d' ' -f1)"
    
    # Optionally encrypt backup
    if command -v gpg &> /dev/null && [[ -n "$BACKUP_ENCRYPTION_KEY" ]]; then
        log_info "Encrypting backup with GPG..."
        BACKUP_TAR_GPG="$BACKUP_TAR.gpg"
        
        if gpg --recipient "$BACKUP_ENCRYPTION_KEY" --trust-model always \
            --encrypt "$BACKUP_TAR" 2>/dev/null; then
            log_info "✓ Encrypted: $BACKUP_TAR_GPG"
            rm "$BACKUP_TAR"
            BACKUP_TAR="$BACKUP_TAR_GPG"
        else
            log_warn "GPG encryption failed. Backup remains unencrypted."
            log_warn "Set BACKUP_ENCRYPTION_KEY environment variable for encryption"
        fi
    else
        if [[ -z "$BACKUP_ENCRYPTION_KEY" ]]; then
            log_warn "GPG encryption not configured."
            log_warn "Set BACKUP_ENCRYPTION_KEY='your-gpg-key-id' for encrypted backups"
        fi
    fi
    
    # Cleanup
    rm -rf "$TEMP_BACKUP"
    
    log_info "Backup completed successfully"
    log_info "Location: $BACKUP_TAR"
    log_info "Retention: Consider storing in secure off-site location"
}

# Function: Verify keys
verify_keys() {
    log_info "=== Verifying Key Integrity ==="
    
    # Verify HMAC key
    if [[ -f "$KEY_DIR/hmac.key" ]]; then
        KEY_SIZE=$(stat -f%z "$KEY_DIR/hmac.key" 2>/dev/null || stat -c%s "$KEY_DIR/hmac.key")
        KEY_PERMS=$(stat -c %a "$KEY_DIR/hmac.key" 2>/dev/null || stat -f %OLp "$KEY_DIR/hmac.key")
        
        if [[ $KEY_SIZE -gt 0 ]]; then
            log_info "✓ HMAC key exists (size: $KEY_SIZE bytes, perms: $KEY_PERMS)"
        else
            log_error "HMAC key is empty!"
            return 1
        fi
    else
        log_warn "HMAC key not found"
    fi
    
    # Verify HIBP key
    if [[ -f "$KEY_DIR/hibp.key" ]]; then
        KEY_SIZE=$(stat -c%s "$KEY_DIR/hibp.key" 2>/dev/null || stat -f%z "$KEY_DIR/hibp.key")
        
        if [[ $KEY_SIZE -gt 0 ]]; then
            log_info "✓ HIBP API key exists (size: $KEY_SIZE bytes)"
        else
            log_error "HIBP API key is empty!"
            return 1
        fi
    else
        log_warn "HIBP API key not found"
    fi
    
    # Verify file permissions (should be 600)
    for key_file in "$KEY_DIR"/hmac.key "$KEY_DIR"/hibp.key; do
        if [[ -f "$key_file" ]]; then
            PERMS=$(stat -c%a "$key_file" 2>/dev/null || stat -f%OLp "$key_file")
            if [[ "$PERMS" == "600" ]]; then
                log_info "✓ $key_file has correct permissions (600)"
            else
                log_warn "⚠ $key_file has unexpected permissions: $PERMS (should be 600)"
            fi
        fi
    done
    
    log_info "Key verification completed"
}

# Function: Show current key status
status_keys() {
    log_info "=== Key Status Report ==="
    log_info "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    log_info "Key directory: $KEY_DIR"
    
    echo ""
    log_info "HMAC Key:"
    if [[ -f "$KEY_DIR/hmac.key" ]]; then
        SIZE=$(stat -c%s "$KEY_DIR/hmac.key" 2>/dev/null || stat -f%z "$KEY_DIR/hmac.key")
        MODIFIED=$(stat -c%y "$KEY_DIR/hmac.key" 2>/dev/null | cut -d' ' -f1 || \
                  stat -f "%Sm" -t "%Y-%m-%d" "$KEY_DIR/hmac.key")
        log_info "  Status: ✓ Exists"
        log_info "  Size: $SIZE bytes"
        log_info "  Modified: $MODIFIED"
        log_info "  Checksum: $(sha256sum "$KEY_DIR/hmac.key" | cut -d' ' -f1)"
    else
        log_warn "  Status: ✗ Not found"
    fi
    
    echo ""
    log_info "HIBP API Key:"
    if [[ -f "$KEY_DIR/hibp.key" ]]; then
        SIZE=$(stat -c%s "$KEY_DIR/hibp.key" 2>/dev/null || stat -f%z "$KEY_DIR/hibp.key")
        MODIFIED=$(stat -c%y "$KEY_DIR/hibp.key" 2>/dev/null | cut -d' ' -f1 || \
                  stat -f "%Sm" -t "%Y-%m-%d" "$KEY_DIR/hibp.key")
        log_info "  Status: ✓ Exists"
        log_info "  Size: $SIZE bytes"
        log_info "  Modified: $MODIFIED"
        log_info "  Checksum: $(sha256sum "$KEY_DIR/hibp.key" | cut -d' ' -f1)"
    else
        log_warn "  Status: ✗ Not found"
    fi
    
    echo ""
    log_info "Recent Backups:"
    ls -lh "$BACKUP_DIR"/dumptruck_keys_* 2>/dev/null | tail -5 || log_warn "  No backups found"
}

# Main
case "${1:-status}" in
    backup)
        backup_keys
        verify_keys
        status_keys
        ;;
    verify)
        verify_keys
        ;;
    status)
        status_keys
        ;;
    *)
        echo "Usage: $0 [backup|verify|status]"
        echo ""
        echo "Commands:"
        echo "  backup  - Create encrypted backup of all keys"
        echo "  verify  - Verify key integrity and permissions"
        echo "  status  - Show current key status and backups"
        exit 1
        ;;
esac

log_info "Operation completed"
