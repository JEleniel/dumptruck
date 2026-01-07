# Threat Card: Cryptocurrency Address Detection

## Overview

**Threat ID:** `DETECTION-008-CRYPTO`

**Category:** Financial Asset Identifier

**Severity:** High

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects cryptocurrency addresses for major blockchain networks:

+ **Bitcoin (BTC)** - P2PKH (1...), P2SH (3...), P2WPKH (bc1...)
+ **Ethereum (ETH)** - 0x... hexadecimal addresses
+ **XRP (Ripple)** - r... addresses
+ Additional altcoins and emerging blockchain networks

### Why It Matters

Cryptocurrency addresses in breaches enable:

+ Direct theft of cryptocurrency holdings
+ Redirection of incoming transfers
+ Correlation with cryptocurrency exchange accounts
+ Money laundering activity tracing
+ Ransomware payment detection
+ Identification of users in darknet markets

---

## Attack Scenarios

### Scenario 1: Cryptocurrency Wallet Takeover

**Attacker Goal:** Steal cryptocurrency from victim's wallet.

**Attack Flow:**

1. Attacker obtains cryptocurrency address from breach data
2. Attacker uses address to access blockchain history (public ledger)
3. Attacker determines wallet balance via blockchain analysis tools
4. Attacker uses social engineering/phishing to obtain private keys
5. Attacker transfers cryptocurrency from victim's wallet to attacker's wallet
6. Transaction is irreversible on blockchain

**Impact:** Complete loss of cryptocurrency, funds unrecoverable.

### Scenario 2: Exchange Account Takeover

**Attacker Goal:** Access victim's cryptocurrency exchange account.

**Attack Flow:**

1. Attacker obtains cryptocurrency address and correlated email from breach
2. Attacker uses email to identify exchange account (many addresses on one exchange)
3. Attacker performs password reset or phishing on exchange
4. Attacker gains access to exchange account
5. Attacker sells cryptocurrency and withdraws to attacker-controlled wallet
6. Victim's cryptocurrency is stolen

**Impact:** Complete account compromise, asset theft.

### Scenario 3: Money Laundering Pattern Identification

**Attacker Goal:** Identify victim's role in cryptocurrency money laundering network.

**Attack Flow:**

1. Law enforcement obtains breach data with cryptocurrency addresses
2. Law enforcement correlates addresses to known darknet markets
3. Law enforcement identifies victim as participant in laundering network
4. Law enforcement subpoenas exchange data to identify victim
5. Victim faces investigation and potential prosecution

**Impact:** Victim facing law enforcement investigation, reputational damage.

---

## Technical Details

### Detection Method

Cryptocurrency address detection uses blockchain-specific format validation.

**Bitcoin Algorithm:**

+ Legacy P2PKH: 26-35 character base58 starting with "1"
+ P2SH: 26-35 character base58 starting with "3"
+ Segwit P2WPKH: 42-62 character bech32 starting with "bc1"
+ Check digit validation using base58 checksum
+ Length and character set validation

**Ethereum Algorithm:**

+ Exactly 42 characters (0x + 40 hex)
+ Starts with "0x" or "0X"
+ Valid hexadecimal characters only
+ Optional Keccak-256 checksum validation (EIP-55)

**XRP Algorithm:**

+ Starts with "r"
+ 25-34 character base58 encoded
+ Check digit validation

**Accuracy:** Very high. Checksum validation ensures validity. Cannot determine if address is in active use without blockchain query.

**False Positive Rate:** Very low for Bitcoin and Ethereum (format is specific). May match other base58 encoded data coincidentally.

### Data at Risk

+ **Type of Data:** Cryptocurrency wallet addresses, blockchain identifiers
+ **Sensitivity Level:** High (links to financial assets and transaction history)
+ **Regulatory Impact:** FinCEN guidelines, KYC/AML compliance, FATF recommendations

---

## Mitigation Strategies

### Prevention

+ Never publicly disclose cryptocurrency addresses
+ Use separate addresses for different purposes (privacy)
+ Use hardware wallets for large holdings (offline security)
+ Implement multi-signature wallet requirements for organizational crypto
+ Monitor blockchain for suspicious activity related to detected addresses

### Detection & Response

1. Monitor and Alert
   * Alert on cryptocurrency address detection
   * Assess if addresses are high-value holders (blockchain analysis)
   * Track for suspicious activity on detected addresses

2. Incident Response
   * Monitor detected addresses on blockchain for suspicious transactions
   * If addresses are organizational, review for unauthorized transfers
   * Consider moving cryptocurrency to new addresses/wallets
   * Notify victims of exposure if personal addresses
   * Recommend exchange account security review

### User Controls

+ Filter and flag cryptocurrency address detections
+ Integration with blockchain monitoring services
+ Track detected addresses for suspicious activity

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Valid cryptocurrency addresses

   ```text
   1A1z7agoat2GPFH7qLcstgyiPs2CJVFJ
   3J98t1WpEZ73CNmYviecrnyiWrnqRhWNLy
   bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
   0x71C7656EC7ab88b098defB751B7401B5f6d8976F
   rN7n7otQDd6FczFgLdlqtyMVrEyd4yF3kD
   ```

   Expected: All detected

2. **Negative Test:** Invalid formats

   ```text
   NotAnAddress
   1234567890
   0xZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ
   ```

   Expected: Not detected

3. **Edge Case:** Similar-looking but invalid addresses

   ```text
   1111111111111111111111111
   0x0000000000000000000000000000000000000000
   ```

   Expected: May be detected if checksum validates

---

## Implementation Notes

+ Source code: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs#L764)
+ Cryptocurrency address validation: Lines 764-797
+ Bitcoin format detection and validation
+ Ethereum format detection and validation
+ XRP and altcoin support documented in comments

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
