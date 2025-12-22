# Threat Card: Mailing Address Detection

## Overview

**Threat ID:** `DETECTION-011-ADDRESS`

**Category:** Location PII Detection

**Severity:** Medium

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects physical mailing addresses using heuristic pattern matching:

+ Street addresses with numbers and street names
+ City, state/province, postal code patterns
+ Apartment/suite numbers and building designations
+ International address formats

### Why It Matters

Physical addresses in breach data enable:

+ Direct mail targeting (spam, scams)
+ Location-based harassment or physical harm
+ Home visiting by attackers or scammers
+ Correlation with other location-based data
+ Identity theft using victim's address
+ Fraud targeting based on neighborhood/area

---

## Attack Scenarios

### Scenario 1: Physical Harassment and Swatting

**Attacker Goal:** Harass or endanger victim using real address.

**Attack Flow:**

1. Attacker obtains victim's home address from breach
2. Attacker combines with name and other PII
3. Attacker makes false emergency call (swatting) reporting crime at address
4. Police arrive at victim's house with tactical response
5. Victim is endangered by police response to false report

**Impact:** Physical danger, police interaction, trauma.

### Scenario 2: Direct Mail Fraud

**Attacker Goal:** Target victim with fraudulent mail offers.

**Attack Flow:**

1. Attacker obtains addresses from breach
2. Attacker sends fraudulent checks or money transfer requests
3. Victim's address is used on fraud mail
4. Victim receives unsolicited financial requests
5. Scammers attempt to convince victim to send money

**Impact:** Fraud attempts, stress, financial loss if victim complies.

### Scenario 3: Physical Targeting

**Attacker Goal:** Plan physical crime against victim (theft, home invasion).

**Attack Flow:**

1. Attacker obtains home addresses from breach
2. Attacker correlates with income data (e.g., job title suggesting wealth)
3. Attacker uses address for targeted burglary or robbery planning
4. Attacker may conduct surveillance before crime
5. Victim's home is targeted

**Impact:** Physical danger, property loss, trauma.

---

## Technical Details

### Detection Method

Address detection uses heuristic pattern matching for common address characteristics.

**Pattern/Algorithm:**

+ Identifies street address patterns (number + street name)
+ Detects city/state combinations
+ Validates postal code formats (US, international)
+ Checks for apartment/suite numbers
+ Multi-line address recognition
+ Context clues from column headers

**Accuracy:** Medium. Heuristic approach means addresses can be confused with other text containing numbers and place names.

**False Positive Rate:** Moderate. Many text patterns can resemble addresses without being actual mailing addresses. Context (column header "address") helps validation.

### Data at Risk

+ **Type of Data:** Physical addresses, location information
+ **Sensitivity Level:** Medium-High (reveals home location, enables physical targeting)
+ **Regulatory Impact:** GDPR (personal data), privacy laws

---

## Mitigation Strategies

### Prevention

+ Use PO boxes instead of home addresses for mail
+ Implement address masking in public-facing data
+ Limit collection of home addresses to necessary contexts
+ Use mail forwarding services to protect real address
+ Register address with mail privacy services (USPS, etc.)

### Detection & Response

1. Monitor and Alert
   * Flag address detections for review (validate actual addresses)
   * Correlate with names to confirm personal records
   * Assess geographic concentration of exposed addresses

2. Incident Response
   * Notify affected individuals of address exposure
   * Recommend address protection measures
   * Suggest credit freeze to prevent account opening
   * Advise on home security measures if needed
   * Consider offering mail forwarding assistance

### User Controls

+ Manual review of detected addresses (high false positive rate)
+ Use context (column headers) to improve confidence
+ Filter results to residential vs. business addresses
+ Geographic analysis of exposed addresses

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Valid address formats

   ```text
   123 Main Street
   456 Oak Avenue, Apt 789
   789 Park Lane
   New York, NY 10001
   ```

   Expected: Detected

2. **Negative Test:** Non-addresses

   ```text
   Building 5 House 3 Section A
   The address bar in my browser
   Please send this address to John
   ```

   Expected: Not detected (or low confidence)

3. **Edge Case:** Ambiguous formats

   ```text
   Highway 51
   County Road 123
   123 Main
   ```

   Expected: May be detected as partial addresses

---

## Implementation Notes

+ Source code: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs#L645)
+ Address detection: Lines 645-695
+ Heuristic pattern matching for street/city/zip
+ Multi-line address support

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
