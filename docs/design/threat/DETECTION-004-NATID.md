# Threat Card: National ID Detection

## Overview

**Threat ID:** `DETECTION-004-NATID`

**Category:** PII Detection

**Severity:** Critical

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects national identification numbers from 15+ countries including:

+ Germany (Personalausweis - ID card number)
+ France (Numéro de sécurité sociale)
+ Spain (DNI - Documento Nacional de Identidad)
+ Italy (Codice fiscale)
+ Netherlands (Burgerservicenummer)
+ Canada (Social Insurance Number)
+ Australia (Tax File Number)
+ India (Aadhaar)
+ China (Resident ID card)
+ And 6+ additional countries

### Why It Matters

National IDs are government-issued identifiers that enable:

+ Identity impersonation
+ Access to government benefits
+ Fraudulent document issuance
+ International travel fraud
+ Financial account takeover
+ Complete identity compromise in victim's jurisdiction

Unlike passwords, national IDs cannot be changed and are tied to government records.

---

## Attack Scenarios

### Scenario 1: Document Fraud (European Example)

**Attacker Goal:** Create fraudulent identity documents using stolen ID number.

**Attack Flow:**

1. Attacker obtains German Personalausweis number and matching personal data
2. Attacker uses ID to open bank accounts claiming to be victim
3. Attacker obtains credit cards and loans
4. Attacker potentially creates entirely fake identity using victim's ID number
5. Victim may not discover fraud for months or years

**Impact:** Financial fraud, damaged government records, identity crisis.

### Scenario 2: Benefit Fraud (Canadian Example)

**Attacker Goal:** Fraudulently claim government benefits using victim's SIN.

**Attack Flow:**

1. Attacker obtains Canadian SIN (Social Insurance Number) from breach
2. Attacker applies for Employment Insurance or other benefits
3. Attacker claims they worked for companies, earning payments
4. Government sends payments and issues T4 documents to victim's SIN
5. Victim's tax records are polluted; recovery is complex

**Impact:** Benefits fraud, tax complications, lost entitlement records.

### Scenario 3: International Travel Fraud (Aadhaar Example)

**Attacker Goal:** Travel internationally using victim's identity.

**Attack Flow:**

1. Attacker obtains Indian Aadhaar number and matching photo (from data broker)
2. Attacker uses Aadhaar to obtain fake passport
3. Attacker travels internationally, potentially committing crimes in victim's name
4. Victim discovers fraud when stopped at border or contacted by law enforcement

**Impact:** Criminal record in victim's name, international incident, severe identity theft.

---

## Technical Details

### Detection Method

National ID detection uses format validation specific to each country's system.

**Pattern/Algorithm:**

+ Country-specific validators for each format
+ Germany: 10-digit ID card format validation
+ France: 13-digit social security number with check digit validation
+ Spain: 8-digit + letter DNI format
+ Italy: 16-character Codice Fiscale with structure validation
+ Canada: 9-digit SIN with validation algorithm
+ Australia: 8-digit TFN with check digit
+ India: 12-digit Aadhaar with structure
+ China: 18-digit ID card with validation
+ Additional countries with appropriate format rules

**Accuracy:** Medium to high depending on format complexity. Some countries require check digit validation; others are format-only validation.

**False Positive Rate:** Low for countries with strict format requirements and check digits. Higher for countries with simple numeric formats where other numeric data might match.

### Data at Risk

+ **Type of Data:** Government-issued identification numbers
+ **Sensitivity Level:** Critical (non-revocable, tied to government records)
+ **Regulatory Impact:** GDPR (very strict on national ID handling), national privacy laws in each country, international conventions

---

## Mitigation Strategies

### Prevention

+ Avoid collecting national IDs unless absolutely required by law
+ If collected, use tokenization or hashing
+ Implement government-grade encryption for national ID data
+ Maintain audit logs exceeding legal requirements
+ Implement access controls with operator identification

### Detection & Response

1. Monitor and Alert
   * Critical alert on national ID detection
   * Immediate escalation to security leadership
   * Determine which countries affected (regulatory trigger)

2. Incident Response
   * Mandatory breach notification per country (varies by jurisdiction)
   * Coordinate with government authorities (may be required)
   * Offer identity protection services
   * Consider offering credit monitoring and fraud alerts in victim's country
   * Notify affected individuals with guidance on protective actions

### User Controls

+ Flag all national ID detections for mandatory review
+ Prevent export or transfer of national ID data
+ Implement role-based access control with specific authorization
+ Maintain detailed audit trail with operator identification

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Valid national ID formats (examples)

   ```text
   12345678A90
   123 45 67890 12
   ```

   Expected: Detected (country-specific)

2. **Negative Test:** Invalid formats

   ```text
   123456789
   AAAABBBBCC
   ```

   Expected: Not detected

3. **Edge Case:** Format boundary conditions

   ```text
   Last valid number for country
   First valid number for country
   ```

   Expected: Detected

---

## Implementation Notes

+ Source code: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs#L469)
+ National ID validation: Lines 469-595
+ Country-specific validators documented in code comments
+ Validation rules per country in inline documentation

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
