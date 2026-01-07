# Threat Card: Email Address Detection

## Overview

**Threat ID:** `DETECTION-001-EMAIL`

**Category:** PII Detection

**Severity:** High

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects email addresses in breach data using pattern matching and structural validation. Emails are identified through detection of `@` symbol combined with valid domain structure (TLD with 2+ alphabetic characters).

### Why It Matters

Email addresses are primary identifiers in credential breaches. They link:

+ Personal identity to compromised credentials
+ User accounts across multiple services
+ Real-world individuals to leaked data
+ Downstream risks including phishing and identity theft

Email detection enables Have I Been Pwned (HIBP) integration to identify which email addresses appear in known breaches, providing critical intelligence for incident response.

---

## Attack Scenarios

### Scenario 1: Phishing Campaign Targeting Breached Users

**Attacker Goal:** Use leaked emails to conduct targeted phishing attacks.

**Attack Flow:**

1. Attacker acquires breach dataset containing emails and passwords
2. Attacker extracts email addresses and matches to passwords
3. Attacker sends personalized phishing emails with correct usernames
4. Users are more likely to trust emails using their actual email/password combo
5. Attacker gains access to email accounts, enabling account takeover

**Impact:** Credential compromise, identity theft, lateral movement to other accounts.

### Scenario 2: Marketing List Compilation

**Attacker Goal:** Build a targeted marketing/spam list from breached data.

**Attack Flow:**

1. Attacker collects multiple breach datasets
2. Attacker extracts and deduplicates email addresses
3. Attacker sells consolidated list to spam operations
4. Spam campaigns reduce email deliverability for legitimate services

**Impact:** Privacy violation, spam proliferation, reputation damage.

### Scenario 3: Account Enumeration and Targeting

**Attacker Goal:** Identify which services specific users are registered with.

**Attack Flow:**

1. Attacker obtains breach data with emails
2. Attacker uses emails to enumerate accounts on popular services
3. Attacker attempts password reuse across discovered accounts
4. Attacker gains unauthorized access

**Impact:** Account takeover, unauthorized access, credential reuse exploitation.

---

## Technical Details

### Detection Method

Email detection uses structural pattern matching combined with domain validation.

**Pattern/Algorithm:**

+ Check for `@` symbol presence
+ Check for `.` in domain portion
+ Validate local part (before `@`) is non-empty and contains no spaces
+ Validate domain:
    * Contains at least one dot
    * Does not start or end with dot
    * TLD (last segment) has 2+ alphabetic characters
    * All segments are non-empty

**Accuracy:** High precision for standard email formats. Standard email RFCs allow complex formats that this simple validator may miss.

**False Positive Rate:** Very low for valid email addresses. Does not validate that email domain actually exists or receives mail. May match email-like strings that are not actually emails.

### Data at Risk

+ **Type of Data:** Email addresses, username identifiers, contact information
+ **Sensitivity Level:** High (directly identifies individuals)
+ **Regulatory Impact:** GDPR (personal data), CCPA (personal information), HIPAA (if health-related context)

---

## Mitigation Strategies

### Prevention

+ Limit email address collection and storage
+ Hash email addresses for duplicate detection (HMAC-based)
+ Use email validation at point of collection
+ Implement data minimization practices

### Detection & Response

1. Monitor and Alert
   * Alert when suspicious volumes of unique emails detected
   * Monitor for correlations with weak passwords
   * Track email appearance in HIBP

2. Incident Response
   * Determine scope of breach (unique email count)
   * Consult HIBP data for known breach history
   * Notify affected users if not previously disclosed
   * Provide guidance on password changes

### User Controls

+ Enable/disable HIBP enrichment for email addresses
+ Configure alert thresholds for email volume
+ Set retention policies for email address data

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Valid email formats should be detected

   ```text
   user@example.com
   test.user@domain.co.uk
   john_doe@company.org
   ```

   Expected: Detected

2. **Negative Test:** Invalid formats should NOT be detected

   ```text
   notanemail
   @nodomain.com
   user@.com
   user name@example.com
   ```

   Expected: Not detected

3. **Edge Case:** Boundary conditions

   ```text
   a@b.co
   user+tag@example.com
   ```

   Expected: <a@b.co> detected; <user+tag@example.com> not detected (contains +)

---

## Implementation Notes

+ Source code: [src/detection/detection.rs](../../src/detection/detection.rs)
+ PII detection: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs)
+ HIBP integration: [docs/HIBP_IMPLEMENTATION.md](../HIBP_IMPLEMENTATION.md)

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
