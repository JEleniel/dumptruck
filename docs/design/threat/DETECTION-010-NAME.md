# Threat Card: Name and Identity Detection

## Overview

**Threat ID:** `DETECTION-010-NAME`

**Category:** Identity PII Detection

**Severity:** Medium

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects human names using heuristic pattern matching. Identifies sequences that match typical name characteristics:

+ Capitalized words with appropriate length
+ Common name patterns and structures
+ Names with prefixes (Mr., Dr., Prof., etc.)
+ Multi-part names (first, middle, last)

### Why It Matters

Names in breach data enable:

+ Identity confirmation for social engineering
+ Personalized phishing attacks
+ Impersonation and identity theft
+ Correlated targeting with other PII
+ Building complete identity profiles
+ Fraud and account opening using victim's name

---

## Attack Scenarios

### Scenario 1: Personalized Phishing with Complete Identity

**Attacker Goal:** Increase phishing success rate through personalization.

**Attack Flow:**

1. Attacker obtains names, emails, and companies from breach
2. Attacker crafts highly personalized phishing emails
3. Email references victim's actual name and company (increases credibility)
4. Victim is more likely to trust email from "familiar" source
5. Victim clicks malicious link and enters credentials

**Impact:** Higher phishing success rate, credential compromise.

### Scenario 2: Account Opening Fraud

**Attacker Goal:** Open fraudulent accounts using victim's name and other PII.

**Attack Flow:**

1. Attacker obtains name, address, phone, and ID from breach
2. Attacker creates application for credit card or bank account
3. Attacker uses victim's name with attacker's contact information
4. Account is opened in victim's name but controlled by attacker
5. Attacker uses account for fraud

**Impact:** Account fraud, credit damage, identity theft.

### Scenario 3: Social Engineering and Impersonation

**Attacker Goal:** Impersonate victim or victim's colleague for access.

**Attack Flow:**

1. Attacker obtains name and organizational data from breach
2. Attacker uses name to call IT department claiming to be employee
3. Attacker requests password reset or access credentials
4. IT provides credentials based on matching name
5. Attacker gains system access

**Impact:** Unauthorized system access, credential compromise.

---

## Technical Details

### Detection Method

Name detection uses heuristic pattern matching for common name characteristics.

**Pattern/Algorithm:**

+ Identifies capitalized words meeting name length criteria (3+ characters typically)
+ Checks for known name prefixes and suffixes
+ Validates against common word lists to exclude non-names
+ Examines context (column headers indicating "name" field increase confidence)
+ Multi-word names (first middle last) validation

**Accuracy:** Medium. Heuristic approach means many false positives (common words like "John" vs. brand names like "Apple").

**False Positive Rate:** Moderate to high. Capitalized words, product names, proper nouns all match name pattern. Context matters significantly (column header "name" increases confidence).

### Data at Risk

+ **Type of Data:** Human names, identity identifiers
+ **Sensitivity Level:** Medium-High (confirms identity, enables social engineering)
+ **Regulatory Impact:** GDPR (personal data), privacy laws

---

## Mitigation Strategies

### Prevention

+ Use pseudonyms or placeholder names in test/development data
+ Implement name masking in logs and reports
+ Limit name collection to what's necessary
+ Use data minimization practices

### Detection & Response

1. Monitor and Alert
   * Flag name detections for manual review (high false positive rate)
   * Combine with email/phone for identity confirmation
   * Assess breach scope based on named records

2. Incident Response
   * Use names to calculate breach impact (number of individuals)
   * Correlate with other PII to understand what's exposed
   * Tailor notification messages to individuals
   * Provide ID theft protection services

### User Controls

+ Manual review recommended (validate detections are actual names)
+ Use context (column headers) to improve detection confidence
+ Filter results to reduce false positives
+ Correlation analysis with email/phone for validation

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Common names should be detected

   ```text
   John Smith
   Maria Garcia
   Chen Wei
   François Dupont
   ```

   Expected: Detected

2. **Negative Test:** Non-names should not be detected

   ```text
   Microsoft Corporation
   Product Version
   Error Code 12345
   ```

   Expected: Not detected (or low confidence)

3. **Edge Case:** Names vs. common words

   ```text
   James Bond
   Grace Hopper
   Jack Daniels
   ```

   Expected: Detected (actual names) or uncertain (brand names)

---

## Implementation Notes

+ Source code: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs#L616)
+ Name detection: Lines 616-644
+ Heuristic pattern matching approach
+ Known name prefixes and suffixes in code
+ Common word exclusion list

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
