# Threat Card: Digital Wallet Token Detection

## Overview

**Threat ID:** `DETECTION-009-WALLET`

**Category:** Financial Asset Identifier

**Severity:** High

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects digital wallet tokens for major payment platforms:

+ **Stripe** - Platform account identifiers
+ **Square** - Payment processor accounts
+ **PayPal** - Account identifiers
+ **Apple Pay** - Device payment tokens
+ **Google Pay** - Wallet tokens
+ Other digital payment providers

### Why It Matters

Digital wallet tokens in breaches enable:

+ Unauthorized purchases using linked payment methods
+ Wallet account takeover and funding access
+ Impersonation for payment transactions
+ Fraud against merchant relationships
+ Access to financial data stored in wallet
+ Cross-service account compromises via linked payment methods

---

## Attack Scenarios

### Scenario 1: Apple Pay/Google Pay Token Fraud

**Attacker Goal:** Add fraudulent cards to victim's digital wallet.

**Attack Flow:**

1. Attacker obtains digital wallet token and correlated credentials
2. Attacker gains access to Apple ID or Google account (phishing/credential reuse)
3. Attacker adds attacker's card to victim's wallet
4. Attacker changes payment method to their card
5. Victim's future purchases are charged to attacker's card

**Impact:** Fraudulent charges, account compromise, financial loss.

### Scenario 2: Stripe/Square Account Takeover

**Attacker Goal:** Access merchant payment account and steal transaction history.

**Attack Flow:**

1. Attacker obtains Stripe/Square account identifier and correlated data
2. Attacker uses account ID to enumerate connected bank accounts
3. Attacker performs account takeover via phishing or credentials
4. Attacker accesses transaction history and customer data
5. Attacker potentially redirects future payments

**Impact:** Merchant account compromise, customer data exposure, payment redirection.

### Scenario 3: PayPal Account Compromise

**Attacker Goal:** Access victim's PayPal account and linked funding sources.

**Attack Flow:**

1. Attacker obtains PayPal token and email from breach
2. Attacker uses email to trigger password reset
3. Attacker gains access to PayPal account
4. Attacker links new payment method and withdraws funds
5. Attacker accesses buyer/seller transaction history

**Impact:** Account takeover, unauthorized transactions, financial loss.

---

## Technical Details

### Detection Method

Digital wallet token detection uses provider-specific format validation.

**Stripe Algorithm:**

+ Account IDs: `acct_` prefix + alphanumeric
+ API keys: `sk_test_` or `sk_live_` + long alphanumeric
+ Token validation against Stripe's documented formats
+ Length ranges and character set validation

**PayPal Algorithm:**

+ Account identifiers: Numeric or alphanumeric with specific patterns
+ Token format: Specific length and structure
+ Validation against known PayPal token formats

**Apple Pay/Google Pay Algorithm:**

+ Device payment tokens: Specific length and format
+ Wallet identifier formats
+ Token expiration handling

**Accuracy:** Medium to high depending on provider documentation availability. Format validation is reliable; actual token validity requires provider API call.

**False Positive Rate:** Low to medium. Provider-specific prefixes reduce false positives. May match developer credentials accidentally left in code.

### Data at Risk

+ **Type of Data:** Payment wallet identifiers, payment processor tokens, merchant account IDs
+ **Sensitivity Level:** High (access to payment methods and transaction history)
+ **Regulatory Impact:** PCI DSS, payment network compliance, GDPR

---

## Mitigation Strategies

### Prevention

+ Never commit payment tokens to code repositories (use environment variables)
+ Implement secrets scanning in CI/CD pipelines
+ Rotate payment tokens regularly
+ Use separate tokens for test and production environments
+ Implement webhook validation and HMAC signing
+ Do not log or store raw payment tokens

### Detection & Response

1. Monitor and Alert
   * Alert on payment token detection (especially live keys)
   * Differentiate between test and production credentials
   * Assess token scope and associated merchant accounts

2. Incident Response
   * If live keys detected: Immediately revoke keys
   * Contact payment processor to monitor for fraudulent activity
   * Rotate keys and update applications
   * Review transaction logs for suspicious activity
   * Notify merchants if account tokens exposed
   * Recommend security audit of payment integration

### User Controls

+ Flag all wallet token detections for review
+ Separate handling for test vs. live credentials
+ Prevent export of sensitive tokens
+ Maintain audit trail of token access

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Payment processor tokens (examples)

   ```text
   acct_1A2B3C4D5E6F7G8H
   sk_test_4eC39HqLyjWDarhtT657G51K
   pm_1IlqXXC4dIcDu7PK4eRR2WKb
   ```

   Expected: Detected

2. **Negative Test:** Non-token sequences

   ```text
   random_text_1234
   test_key_here
   ```

   Expected: Not detected

3. **Edge Case:** Tokens in various contexts

   ```text
   sk_live_sensitive_key_here
   acct_production_123456789
   ```

   Expected: Detected with live/production flag

---

## Implementation Notes

+ Source code: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs#L798)
+ Digital wallet token detection: Lines 798-839
+ Payment processor specific validation
+ Test vs. live credential differentiation

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
