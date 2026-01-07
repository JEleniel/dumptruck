# Threat Card: Phone Number Detection

## Overview

**Threat ID:** `DETECTION-002-PHONE`

**Category:** PII Detection

**Severity:** High

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects international phone numbers using country-specific formats. Supports 15+ countries including US, UK, Canada, Germany, France, China, India, Australia, and others with format validation for each country's numbering system.

### Why It Matters

Phone numbers are critical PII that enables:

+ Direct communication targeting (calls, SMS, voicemail phishing)
+ Caller ID spoofing attacks
+ SIM swap attacks (hijacking phone-based 2FA)
+ Location inference from area codes
+ Cross-referencing with other datasets

---

## Attack Scenarios

### Scenario 1: SIM Swap Attack

**Attacker Goal:** Take control of victim's phone to bypass 2FA and access accounts.

**Attack Flow:**

1. Attacker obtains breach data including phone number and email
2. Attacker calls phone provider's customer service with victim's info
3. Attacker claims to have lost phone and requests SIM replacement
4. Provider transfers phone number to attacker's SIM
5. Attacker uses 2FA SMS to reset passwords and access accounts

**Impact:** Complete account takeover, financial theft, identity fraud.

### Scenario 2: Targeted Smishing (SMS Phishing) Campaign

**Attacker Goal:** Trick users into clicking malicious links via SMS.

**Attack Flow:**

1. Attacker extracts phone numbers from breach data
2. Attacker correlates with emails to target specific users
3. Attacker sends SMS claiming account alert or verification needed
4. Users click link and enter credentials on fake login page
5. Attacker harvests credentials

**Impact:** Credential compromise, malware installation, account takeover.

### Scenario 3: Phone-Based Reconnaissance

**Attacker Goal:** Build intelligence for targeted social engineering.

**Attack Flow:**

1. Attacker extracts phone numbers and area codes from breach
2. Attacker maps area codes to geographic locations
3. Attacker uses location info for social engineering (pretending to be local bank, etc.)
4. Attacker gains trust through localized targeting

**Impact:** Social engineering success rate increase, account compromise.

---

## Technical Details

### Detection Method

Phone numbers are detected using country-specific format validation.

**Pattern/Algorithm:**

+ Supports 15+ country formats
+ Validates digit count and formatting requirements
+ Removes common formatting characters (+, -, (, ), space)
+ Matches against country-specific patterns
+ Example: US format validates 10-digit format after +1 or 1 prefix

**Accuracy:** High for standard formats. International formats vary significantly; some modern formats may not match legacy patterns.

**False Positive Rate:** Low. May match non-phone-number sequences that happen to match patterns (e.g., product serial numbers, part numbers).

### Data at Risk

+ **Type of Data:** Telephone numbers, location information (via area codes), communication identifiers
+ **Sensitivity Level:** High (enables direct targeting and account takeover via SIM swap)
+ **Regulatory Impact:** GDPR, CCPA, PIPEDA (Canada), LGPD (Brazil)

---

## Mitigation Strategies

### Prevention

+ Minimize phone number collection
+ Hash phone numbers for comparison (HMAC-based)
+ Implement strong 2FA beyond SMS (hardware keys, TOTP)
+ Use call blocking and verification services

### Detection & Response

1. Monitor and Alert
   * Alert on large volumes of unique phone numbers
   * Monitor for patterns suggesting social engineering campaigns
   * Track phone number reuse across breaches

2. Incident Response
   * Notify affected individuals of breach
   * Recommend SIM swap fraud alert or freeze
   * Suggest SMS 2FA replacement with stronger methods
   * Monitor affected phone numbers for suspicious activity

### User Controls

+ Filter and flag phone number detections
+ Configure alert thresholds
+ Export phone number data for notification campaigns

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Valid international phone formats

   ```text
   +1 (555) 123-4567
   +44 20 7946 0958
   +86 10 1234 5678
   ```

   Expected: Detected

2. **Negative Test:** Invalid formats

   ```text
   123456789
   not-a-phone
   ```

   Expected: Not detected

3. **Edge Case:** Formatting variations

   ```text
   555-123-4567
   (555)123-4567
   ```

   Expected: Detected with format normalization

---

## Implementation Notes

+ Source code: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs#L176)
+ Phone detection validation: Lines 176-191
+ Supported countries documented in code comments

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
