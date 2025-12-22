# Threat Card: Bank Account and Financial Identifier Detection

## Overview

**Threat ID:** `DETECTION-007-BANKACCOUNT`

**Category:** Financial PII Detection

**Severity:** Critical

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects multiple financial identifiers:

+ **IBAN** (International Bank Account Number) - 15-34 alphanumeric characters with country-specific validation
+ **SWIFT Code** - 8 or 11 character bank identifier codes (BIC)
+ **Routing Numbers** - 9-digit US bank routing numbers with validation
+ **Bank Account Numbers** - 8-17 digit sequences with bank validation
+ **Credit Card Numbers** - See separate threat card

### Why It Matters

Financial identifiers in breaches enable:

+ Direct bank account access and theft
+ Wire fraud and unauthorized transfers
+ International money laundering
+ Account takeover without card
+ Impersonation for financial accounts
+ Rapid funds theft before detection

---

## Attack Scenarios

### Scenario 1: Direct Bank Account Fraud (IBAN)

**Attacker Goal:** Drain victim's bank account using IBAN and routing information.

**Attack Flow:**

1. Attacker obtains IBAN and victim's name from breach
2. Attacker uses IBAN to set up fraudulent wire transfer
3. Attacker registers bill payment or standing order
4. Attacker authorizes transfer of funds to attacker's account
5. Victim's account is drained before noticing fraud

**Impact:** Direct financial loss, frozen accounts, international complications.

### Scenario 2: SWIFT Payment Fraud (International)

**Attacker Goal:** Execute international wire fraud using SWIFT codes.

**Attack Flow:**

1. Attacker obtains SWIFT code and bank account details from breach
2. Attacker initiates SWIFT transfer from victim's bank
3. Attacker uses social engineering or hacked credentials to authorize
4. Funds transferred internationally (difficult to recover)
5. SWIFT transfer is irreversible; funds may be laundered

**Impact:** Large financial loss, international incident, funds likely unrecoverable.

### Scenario 3: Routing Number Fraud (US Domestic)

**Attacker Goal:** Set up fraudulent ACH (Automated Clearing House) transfers.

**Attack Flow:**

1. Attacker obtains routing number and account number from breach
2. Attacker uses for fraudulent ACH debits (bill payments, transfers)
3. Attacker can set up recurring fraudulent charges
4. Victim's account is repeatedly charged before discovering fraud
5. ACH fraud is difficult to reverse (settlement rules favor bank)

**Impact:** Recurring financial loss, account freezing, fraud cleanup nightmare.

---

## Technical Details

### Detection Method

Multiple detection algorithms for different financial identifier types.

**IBAN Algorithm:**

+ Validate country code (first 2 letters)
+ Validate check digits (3-4 characters)
+ IBAN check digit algorithm (mod-97)
+ Country-specific length validation
+ Alphanumeric validation per country rules

**SWIFT Code Algorithm:**

+ 8 or 11 character alphanumeric sequence
+ First 4 characters = bank code (alphabetic)
+ 2 characters = country code (alphabetic, valid ISO 3166)
+ 2 characters = location code
+ Optional 3 characters = branch code

**Routing Number Algorithm:**

+ Exactly 9 digits
+ Checksum validation (weighted sum)
+ Valid routing number range checks

**Accuracy:** High for format validation. IBAN check digit validation provides mathematical verification. SWIFT validation checks country codes against valid list.

**False Positive Rate:** Very low. Check digit and country code validation minimize false positives. May match random sequences that coincidentally match format.

### Data at Risk

+ **Type of Data:** Bank account numbers, international account identifiers, routing information
+ **Sensitivity Level:** Critical (enables direct account access and theft)
+ **Regulatory Impact:** PCI DSS, GDPR, banking regulations, AML (Anti-Money Laundering), financial compliance

---

## Mitigation Strategies

### Prevention

+ Never transmit banking information via email or unsecured channels
+ Use tokenization for bank account storage
+ Implement strict encryption for financial identifier data
+ Limit access to bank account information (need-to-know basis)
+ Use 1-2 digit masking for account numbers in logs

### Detection & Response

1. Monitor and Alert
   * Critical alert on any banking identifier detection
   * Separate alerts for IBAN, SWIFT, routing numbers
   * Assess volume and affected financial institutions

2. Incident Response
   * Immediate notification to affected financial institutions
   * Advise banks to monitor affected accounts for fraudulent activity
   * Notify customers to place fraud alerts with banks
   * Recommend frozen accounts or new account numbers
   * Consider offering fraud protection services
   * Coordinate with AML teams for suspicious activity monitoring

### User Controls

+ Flag all banking identifier detections for mandatory review
+ Prevent unmasked export of account numbers
+ Implement role-based access control
+ Maintain detailed audit logs with operator identification

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Valid banking identifiers

   ```text
   FR1420041010050500013M026
   DEUTDEFF500
   021000021
   12345678
   ```

   Expected: Detected (IBAN, SWIFT, Routing Number, Account Number)

2. **Negative Test:** Invalid formats

   ```text
   INVALID123
   ABC-DEF-GHI
   123456
   ```

   Expected: Not detected (invalid format)

3. **Edge Case:** Boundary conditions

   ```text
   Shortest valid IBAN
   Longest valid IBAN
   ```

   Expected: Detected

---

## Implementation Notes

+ Source code: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs#L696)
+ IBAN detection: Lines 696-712
+ SWIFT detection: Lines 713-736
+ Routing number detection: Lines 737-749
+ Bank account detection: Lines 750-763

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
