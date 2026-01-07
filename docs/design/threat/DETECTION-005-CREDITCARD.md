# Threat Card: Credit Card Number Detection

## Overview

**Threat ID:** `DETECTION-005-CREDITCARD`

**Category:** PII Detection

**Severity:** Critical

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects credit card numbers using the Luhn algorithm validation. Supports all major card types (Visa, Mastercard, American Express, Discover, etc.) with 13-19 digit sequences that pass Luhn check and card type validation.

### Why It Matters

Credit card numbers in breaches enable:

+ Immediate financial fraud and unauthorized charges
+ Card cloning and physical card reproduction
+ International fraud and money laundering
+ Rapid financial loss before cardholder notices
+ Damage to cardholder's credit score
+ Recurring fraud risk until card is replaced

---

## Attack Scenarios

### Scenario 1: Immediate Card Fraud

**Attacker Goal:** Make unauthorized purchases before fraud is detected.

**Attack Flow:**

1. Attacker obtains credit card numbers from breach (with expiry and CVV if available)
2. Attacker immediately tests cards with small purchases ($1-5)
3. Attacker discovers which cards are active
4. Attacker makes large fraudulent purchases online
5. Victim's credit card reaches limit before noticing fraudulent charges

**Impact:** Financial loss, fraud cleanup, damaged credit score.

### Scenario 2: Card Cloning

**Attacker Goal:** Create physical clone cards for in-store fraud.

**Attack Flow:**

1. Attacker obtains card number, expiry, and CVV
2. Attacker uses card skimming device to encode information onto blank cards
3. Attacker uses cloned cards at retail locations for fraudulent purchases
4. Attacker exploits time lag before fraud is detected
5. Attacker fences stolen merchandise

**Impact:** Financial loss, inventory loss, victim's credit damage.

### Scenario 3: Recurring Subscription Fraud

**Attacker Goal:** Set up fraudulent recurring charges.

**Attack Flow:**

1. Attacker obtains card number with expiry date
2. Attacker enrolls in subscription services (streaming, dating, etc.)
3. Attacker uses multiple false identities and email addresses
4. Recurring charges appear small ($9.99/month) and may go unnoticed
5. Attacker maintains subscriptions until card is replaced

**Impact:** Ongoing financial loss, subscription fraud, cardholder frustration.

---

## Technical Details

### Detection Method

Credit card detection uses format validation and Luhn algorithm check.

**Pattern/Algorithm:**

+ Match 13-19 digit sequences
+ Apply Luhn algorithm verification (checksum validation)
+ Verify card type based on leading digits (Visa: 4, Mastercard: 51-55, Amex: 34/37, Discover: 6011, etc.)
+ Exclude sequences from non-financial contexts if possible

**Accuracy:** High for actual credit card numbers (Luhn check ensures mathematical validity). Cannot detect invalid cards without real-time BIN lookup.

**False Positive Rate:** Very low. Luhn algorithm prevents false positives from random 16-digit numbers. May match other high-checksum sequences from non-financial domains.

### Data at Risk

+ **Type of Data:** Credit card numbers (full PAN), payment instrument identifiers
+ **Sensitivity Level:** Critical (enables direct financial fraud)
+ **Regulatory Impact:** PCI DSS (strict requirements), GDPR, CCPA, financial regulations

---

## Mitigation Strategies

### Prevention

+ Never store full credit card numbers (use tokenization via payment processor)
+ If card data must be stored, encrypt with government-grade encryption
+ Implement strict access controls on card data
+ Use SSL/TLS for all card data transmission
+ Implement card number masking in logs and audit trails

### Detection & Response

1. Monitor and Alert
   * Critical alert on credit card number detection
   * Immediate escalation to fraud and security teams
   * Determine card quantity and issuer

2. Incident Response
   * Notify card issuers immediately
   * Coordinate with issuer on fraud response
   * Notify affected cardholders of breach
   * Provide guidance on card cancellation and fraud monitoring
   * Monitor for actual fraud on detected cards
   * Coordinate with payment networks (Visa, Mastercard, etc.)

### User Controls

+ Flag all credit card detections for mandatory review
+ Prevent export of unmasked card numbers
+ Implement role-based access control for card data
+ Maintain forensic audit trails with operator identification

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Valid credit card numbers (test cards)

   ```text
   4532015112830366
   5425233010103442
   374245455400126
   ```

   Expected: Detected

2. **Negative Test:** Invalid card numbers or non-card sequences

   ```text
   1234567890123456
   9999999999999999
   ```

   Expected: Not detected (fail Luhn check)

3. **Edge Case:** Card number formats with spaces

   ```text
   4532 0151 1283 0366
   ```

   Expected: Detected after format normalization

---

## Implementation Notes

+ Source code: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs#L596)
+ Luhn algorithm validation: Lines 596-615
+ Card type identification in code comments
+ PCI DSS compliance notes in implementation

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
