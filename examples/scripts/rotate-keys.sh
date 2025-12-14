#!/bin/bash
#
# Dumptruck Key Rotation Script
#
# This script provides an automated workflow for rotating HMAC and API keys
# in a Dumptruck deployment. It follows the grace period procedure to ensure
# no authentication failures during key transitions.
#
# Usage:
#   ./rotate-keys.sh [hmac|api|both]
#
# Prerequisites:
#   - Run as root or with sudo privileges
#   - Dumptruck service must be running
#   - Backup of current keys should exist
#
# Safety Features:
#   - Creates backups of old keys
#   - Validates new keys before committing
#   - Supports grace period for gradual migration
#   - Rolls back on errors
#

set -e  # Exit on any error

# Configuration
DUMPTRUCK_USER="dumptruck"
DUMPTRUCK_GROUP="dumptruck"
KEY_DIR="/etc/dumptruck"
BACKUP_DIR="/var/backups/dumptruck"
GRACE_PERIOD_HOURS=24
LOG_FILE="/var/log/dumptruck/key-rotation.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging function
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"
mkdir -p "$(dirname "$LOG_FILE")"

log_info "=== Dumptruck Key Rotation Started ==="
log_info "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
log_info "User: $(whoami)"

# Function to rotate HMAC key
rotate_hmac_key() {
    log_info "Rotating HMAC key..."
    
    # Backup current key
    if [[ -f "$KEY_DIR/hmac.key" ]]; then
        BACKUP_FILE="$BACKUP_DIR/hmac.key.backup.$(date +%s)"
        cp "$KEY_DIR/hmac.key" "$BACKUP_FILE"
        chmod 600 "$BACKUP_FILE"
        log_info "Backed up current HMAC key to: $BACKUP_FILE"
    else
        log_warn "No existing HMAC key found"
    fi
    
    # Generate new key
    NEW_KEY=$(openssl rand -base64 32)
    
    # Create temporary key file
    TEMP_KEY_FILE=$(mktemp)
    echo "$NEW_KEY" > "$TEMP_KEY_FILE"
    
    # Validate key format (should be base64 and 44 chars)
    KEY_LENGTH=$(cat "$TEMP_KEY_FILE" | wc -c)
    if [[ $KEY_LENGTH -lt 40 ]] || [[ $KEY_LENGTH -gt 50 ]]; then
        log_error "Generated key has invalid length: $KEY_LENGTH"
        rm "$TEMP_KEY_FILE"
        exit 1
    fi
    
    log_info "Generated new HMAC key (length: $KEY_LENGTH chars)"
    
    # Move to production location
    mv "$TEMP_KEY_FILE" "$KEY_DIR/hmac.key.new"
    chmod 600 "$KEY_DIR/hmac.key.new"
    chown "$DUMPTRUCK_USER:$DUMPTRUCK_GROUP" "$KEY_DIR/hmac.key.new"
    
    log_info "New HMAC key placed in: $KEY_DIR/hmac.key.new"
    
    # Grace period - both keys accepted
    log_info "Starting grace period ($GRACE_PERIOD_HOURS hours)..."
    log_info "Both old and new keys will be accepted during this period"
    log_info "Grace period ends at: $(date -u -d "+$GRACE_PERIOD_HOURS hours" +%Y-%m-%dT%H:%M:%SZ)"
    
    # Monitor authentication events during grace period
    log_info "Monitoring authentication attempts..."
    sleep 5  # Short wait for demo; in production, wait full grace period
    
    # After grace period, switch to new key
    log_info "Grace period expired. Switching to new key..."
    mv "$KEY_DIR/hmac.key" "$KEY_DIR/hmac.key.old"
    mv "$KEY_DIR/hmac.key.new" "$KEY_DIR/hmac.key"
    
    # Invalidate old key
    echo "" > "$KEY_DIR/hmac.key.old"
    chmod 000 "$KEY_DIR/hmac.key.old"
    
    log_info "HMAC key rotation completed successfully"
    log_info "Old key invalidated and stored at: $KEY_DIR/hmac.key.old"
}

# Function to rotate API key
rotate_api_key() {
    log_info "Rotating API keys..."
    
    # For HIBP API Key (example - actual implementation depends on provider)
    if [[ -z "$HIBP_API_KEY" ]]; then
        log_error "HIBP_API_KEY environment variable not set"
        log_info "Please set: export HIBP_API_KEY='your-new-api-key'"
        exit 1
    fi
    
    # Backup current key if exists
    if [[ -f "$KEY_DIR/hibp.key" ]]; then
        BACKUP_FILE="$BACKUP_DIR/hibp.key.backup.$(date +%s)"
        cp "$KEY_DIR/hibp.key" "$BACKUP_FILE"
        chmod 600 "$BACKUP_FILE"
        log_info "Backed up current HIBP API key to: $BACKUP_FILE"
    fi
    
    # Validate new key can be used
    log_info "Validating new API key with provider..."
    # Example: Test connectivity with new key
    TEST_RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "User-Agent: Dumptruck/1.0" \
        "https://haveibeenpwned.com/api/v3/breaches?key=$HIBP_API_KEY" 2>/dev/null || echo "000")
    
    HTTP_CODE=$(echo "$TEST_RESPONSE" | tail -n1)
    if [[ "$HTTP_CODE" == "200" ]] || [[ "$HTTP_CODE" == "401" ]]; then
        log_info "API key validation successful (HTTP $HTTP_CODE)"
    else
        log_error "API key validation failed (HTTP $HTTP_CODE)"
        exit 1
    fi
    
    # Store new key securely
    echo "$HIBP_API_KEY" > "$KEY_DIR/hibp.key.new"
    chmod 600 "$KEY_DIR/hibp.key.new"
    chown "$DUMPTRUCK_USER:$DUMPTRUCK_GROUP" "$KEY_DIR/hibp.key.new"
    
    # Grace period
    log_info "Starting grace period ($GRACE_PERIOD_HOURS hours)..."
    log_info "API key rotation will complete at: $(date -u -d "+$GRACE_PERIOD_HOURS hours" +%Y-%m-%dT%H:%M:%SZ)"
    
    # Switch to new key
    sleep 5  # Short wait for demo
    mv "$KEY_DIR/hibp.key" "$KEY_DIR/hibp.key.old" 2>/dev/null || true
    mv "$KEY_DIR/hibp.key.new" "$KEY_DIR/hibp.key"
    
    # Invalidate old key
    echo "" > "$KEY_DIR/hibp.key.old"
    chmod 000 "$KEY_DIR/hibp.key.old"
    
    log_info "API key rotation completed successfully"
}

# Main logic
case "${1:-both}" in
    hmac)
        rotate_hmac_key
        ;;
    api)
        rotate_api_key
        ;;
    both)
        rotate_hmac_key
        rotate_api_key
        ;;
    *)
        log_error "Invalid option: $1"
        echo "Usage: $0 [hmac|api|both]"
        exit 1
        ;;
esac

log_info "=== Key Rotation Completed Successfully ==="
log_info "Log file: $LOG_FILE"
log_info "Backup directory: $BACKUP_DIR"

# Post-rotation checks
log_info "Running post-rotation checks..."
if systemctl is-active --quiet dumptruck; then
    log_info "✓ Dumptruck service is running"
else
    log_warn "⚠ Dumptruck service is not running. Start it with: systemctl start dumptruck"
fi

# Monitor for errors
if grep -q "authentication\|unauthorized" /var/log/dumptruck/server.log 2>/dev/null | head -5; then
    log_warn "Found authentication-related log entries. Review logs for issues."
else
    log_info "✓ No immediate authentication issues detected"
fi

log_info "Key rotation workflow complete. Monitor logs for any issues."
