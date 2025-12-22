# Threat Card: Anomaly and Novelty Detection

## Overview

**Threat ID:** `DETECTION-014-ANOMALY`

**Category:** Anomaly Detection

**Severity:** Medium

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects multiple types of anomalies in credential data:

+ **Unseen Field Combinations:** Unusual combinations of fields not seen in dataset baseline
+ **Rare Domains:** Email domains appearing rarely in dataset (possible typos, spoofing)
+ **Unexpected Credential Formats:** Passwords with unusual structure or patterns
+ **Baseline Deviation:** Statistical outliers from dataset baseline patterns
+ **Length Outliers:** Unusually long/short values in specific fields

### Why It Matters

Anomaly detection identifies:

+ Potential data corruption or injection
+ Synthetic or test data mixed with real data
+ Malicious modifications to dataset
+ Data quality issues for investigation
+ Unusual patterns suggesting targeted attacks or compromises
+ Potential indicators of compromise (IoCs)

---

## Attack Scenarios

### Scenario 1: Synthetic Data Detection in Mixed Dataset

**Attacker Goal:** Identify which portion of breached dataset is authentic.

**Attack Flow:**

1. Attacker obtains mixed dataset with both real and test data
2. Attacker runs anomaly detection on field combinations
3. Attacker identifies records with unusual field combinations (test data marker)
4. Attacker separates authentic records from test data
5. Attacker focuses attacks on authentic records only

**Impact:** Improved targeting of authentic compromised accounts.

### Scenario 2: Email Typo Exploitation

**Attacker Goal:** Target users with typo-prone email domains.

**Attack Flow:**

1. Attacker analyzes leaked dataset for rare email domains
2. Attacker identifies outlier domains (gmai.com instead of gmail.com)
3. Attacker registers similar domains and sends phishing emails
4. Users may be confused by similar domain to expected sender
5. Attacker harvests credentials through typo domain confusion

**Impact:** Targeted phishing, credential theft.

### Scenario 3: Credential Format Anomaly Analysis

**Attacker Goal:** Identify accounts with unusual passwords suggesting high-value targets.

**Attack Flow:**

1. Attacker analyzes breach data for unusual password patterns
2. Attacker identifies anomalously formatted passwords (company codes, sequences)
3. Attacker infers that unusual format indicates high-value system credentials
4. Attacker prioritizes cracking unusual format passwords
5. Attacker gains access to high-value systems first

**Impact:** Prioritized compromise of critical infrastructure/systems.

---

## Technical Details

### Detection Method

Multi-dimensional anomaly detection across credential data.

**Algorithms:**

1. **Unseen Field Combination Detection**
   * Catalog all observed field combinations in dataset
   * Flag new combinations not in baseline
   * Calculate combination frequency for statistical significance
   * Example: Email domain never seen with specific username pattern

2. **Rare Domain Detection**
   * Calculate domain frequency distribution
   * Identify domains appearing < N times in dataset
   * Flag as potential typos or spoofed domains
   * Domain reputation checks if available

3. **Credential Format Anomaly**
   * Analyze password structure (character classes, length, patterns)
   * Flag passwords with unusual structure vs. baseline
   * Identify potentially machine-generated vs. human passwords
   * Pattern matching for known credential formats (company codes, etc.)

4. **Baseline Deviation**
   * Calculate statistical baseline for each field
   * Mean, median, standard deviation per field
   * Z-score analysis for outliers
   * Track changes in distribution over time

**Accuracy:** Medium to high depending on baseline quality. Requires sufficient sample size for statistical validity.

**False Positive Rate:** Medium. Legitimate unusual values can trigger alerts. Context-dependent detection helps reduce false positives.

### Data at Risk

+ **Type of Data:** Anomaly indicators, data quality metrics, credential patterns
+ **Sensitivity Level:** Low-Medium (indicates potential issues, not direct PII)
+ **Regulatory Impact:** Data integrity assessment, breach investigation

---

## Mitigation Strategies

### Prevention

+ Implement data validation rules at ingestion
+ Use consistent credential format enforcement
+ Regular baseline statistics review
+ Anomaly detection in security operations
+ Alerting on unexpected field combinations

### Detection & Response

1. Monitor and Alert
   * Alert on multiple anomaly types in single record (higher confidence)
   * Correlate anomalies with breach timeline
   * Monitor for patterns suggesting systematic tampering
   * Track anomaly rate changes in dataset

2. Incident Response
   * Investigate combination anomalies for data corruption
   * Verify rare domains for data quality issues
   * Analyze unusual credential formats for patterns
   * Use anomalies for forensic breach timeline analysis
   * Alert systems teams to unusual authentication patterns

### User Controls

+ Configure anomaly detection sensitivity thresholds
+ Enable/disable specific anomaly types
+ Baseline recalculation and reset
+ Manual review queue for high-confidence anomalies
+ Correlation with other detection types

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Clear anomalies

   ```text
   Record: email=test_email@typo.com, username=admin_prod, role=test
   Baseline: Never seen [typo.com + admin_prod + test] combination
   ```

   Expected: Detected as unseen combination anomaly

2. **Negative Test:** Normal baseline patterns

   ```text
   Record: email=user@gmail.com, username=user, role=customer
   Baseline: [gmail.com + customer role] seen 5000+ times
   ```

   Expected: Not detected

3. **Edge Case:** Statistically marginal

   ```text
   Record: email=user@rare.com
   Baseline: Domain appears 1 time out of 100,000 records
   ```

   Expected: May be detected depending on rarity threshold

---

## Implementation Notes

+ Source code: [src/detection/anomaly_detection.rs](../../src/detection/anomaly_detection.rs)
+ Combination analysis and frequency tracking
+ Domain rarity calculation
+ Credential format pattern matching
+ Statistical baseline management
+ Z-score anomaly detection

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
