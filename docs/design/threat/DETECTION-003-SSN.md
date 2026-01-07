# Threat Card: Social Security Number (SSN) Detection

## Overview

**Threat ID:** `DETECTION-003-SSN`

**Category:** PII Detection

**Severity:** Critical

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects US Social Security Numbers (9-digit sequences matching XXX-XX-XXXX or XXXXXXXXX format). SSNs are validated to exclude invalid ranges (000-00-0000, 666-XX-XXXX, 900-999-XXXX).

### Why It Matters

SSNs are critical PII that directly enables:

+ Identity theft and impersonation
+ Credit fraud and account opening
+ Tax fraud (filing false returns)
+ Employment fraud
+ Government benefit fraud
+ Complete identity compromise

An SSN in a breach is among the most serious exposures, as it cannot be changed like a password.

---

## Attack Scenarios

### Scenario 1: Credit Fraud and Account Opening

**Attacker Goal:** Open fraudulent accounts using victim's identity.

**Attack Flow:**

1. Attacker obtains SSN, name, and address from breach data
2. Attacker applies for credit cards, loans, phone service using victim's SSN
3. Attacker uses accounts for fraudulent purchases
4. Victim's credit rating tanks
5. Victim must spend months/years resolving fraud

**Impact:** Financial loss, damaged credit, years of recovery work.

### Scenario 2: Tax Fraud (IRS Impersonation)

**Attacker Goal:** File false tax return and intercept refund.

**Attack Flow:**

1. Attacker obtains SSN, name, address, and income info
2. Attacker files tax return to IRS claiming larger refund
3. Attacker receives refund before victim files legitimate return
4. Victim must prove fraud to IRS and reclaim funds

**Impact:** Financial loss, tax complications, years of IRS involvement.

### Scenario 3: Employment Fraud

**Attacker Goal:** Obtain employment using victim's identity.

**Attack Flow:**

1. Attacker uses SSN and name to apply for jobs
2. Attacker works and earns income under victim's name/SSN
3. Victim receives W-2 forms for jobs they didn't work
4. Tax filing becomes complicated and suspicious

**Impact:** Earnings attribution errors, tax complications, employment record pollution.

---

## Technical Details

### Detection Method

SSN detection uses format validation and range exclusion.

**Pattern/Algorithm:**

+ Matches 9 consecutive digits or XXX-XX-XXXX format
+ Excludes known invalid ranges:
    * 000-00-0000 (test number)
    * 666-XX-XXXX (never assigned)
    * 900-999-XXXX (never assigned, reserved for ITINs)
+ Does not validate actual SSA assignment (that requires SSA database)

**Accuracy:** High for format matching. Cannot determine if SSN is actually valid without SSA database access.

**False Positive Rate:** Low if invalid ranges properly excluded. May match random 9-digit sequences or product codes.

### Data at Risk

+ **Type of Data:** Social Security Numbers, government-issued identifier
+ **Sensitivity Level:** Critical (non-revocable identity compromise)
+ **Regulatory Impact:** GDPR, CCPA, HIPAA, FCRA (Fair Credit Reporting Act), Privacy Act

---

## Mitigation Strategies

### Prevention

+ Never store SSNs if possible; use tokenization instead
+ If SSNs collected, encrypt at rest with strong encryption
+ Limit SSN access to absolutely necessary personnel
+ Implement strict access controls and audit logging
+ Use 6-digit verification instead of full SSN where possible

### Detection & Response

1. Monitor and Alert
   * Critical alert on any SSN detection
   * Immediate escalation to security team
   * Track SSN breach scope (count of unique SSNs)

2. Incident Response
   * Immediate notification to affected individuals (breach notification laws)
   * Offer credit monitoring and fraud protection services (3-5 years minimum)
   * Consider offering identity theft insurance
   * Notify credit agencies (Equifax, Experian, TransUnion)
   * Consider offering credit freeze assistance

### User Controls

+ Flag all SSN detections for review
+ Prevent export/sharing of SSN data
+ Maintain detailed audit log of SSN access

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Valid SSN formats

   ```text
   123-45-6789
   123456789
   ```

   Expected: Detected

2. **Negative Test:** Invalid or excluded SSNs

   ```text
   000-00-0000
   666-12-3456
   900-00-0000
   123-45-678
   ```

   Expected: Not detected (excluded ranges)

3. **Edge Case:** Boundary conditions

   ```text
   899-99-9999
   001-01-0001
   ```

   Expected: Detected

---

## Implementation Notes

+ Source code: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs#L192)
+ SSN validation: Lines 192-220
+ Invalid range exclusions documented in code

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
