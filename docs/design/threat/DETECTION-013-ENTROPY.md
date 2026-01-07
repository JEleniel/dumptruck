# Threat Card: Entropy Outlier Detection

## Overview

**Threat ID:** `DETECTION-013-ENTROPY`

**Category:** Anomaly Detection

**Severity:** Medium

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects values with unusual character distribution (entropy outliers). Identifies passwords and strings with >3σ statistical deviation from dataset baseline:

+ Values with unusually high entropy (random-looking strings)
+ Values with unusually low entropy (repetitive characters)
+ Character distribution anomalies

### Why It Matters

Entropy outliers may indicate:

+ Synthetic/test data injected into dataset
+ Unusual passwords or malformed entries
+ Injected malicious data or modified records
+ Data quality issues or corruption
+ Unusual credential formats suggesting non-human-chosen passwords
+ Potential for credential cracking difficulty assessment

---

## Attack Scenarios

### Scenario 1: Test Data Detection and Evasion

**Attacker Goal:** Identify synthetic test data to locate production database.

**Attack Flow:**

1. Attacker obtains large database dump
2. Attacker analyzes entropy of password field
3. Attacker identifies test passwords as statistical outliers
4. Attacker notes patterns in test passwords (TestPass123!, etc.)
5. Attacker uses pattern to identify production database sections

**Impact:** Adversary capability to distinguish production from test data.

### Scenario 2: Data Integrity Verification

**Attacker Goal:** Verify authenticity of leaked data before purchase.

**Attack Flow:**

1. Attacker acquires alleged breach data from dark market
2. Attacker runs entropy analysis on passwords
3. Attacker identifies suspicious entropy patterns suggesting synthetic data
4. Attacker determines data is fake/synthetic (avoiding scam purchase)
5. Attacker continues search for authentic data

**Impact:** Enables dark market transactions by validating data authenticity.

### Scenario 3: Credential Cracking Strategy Optimization

**Attacker Goal:** Prioritize cracking of easier passwords first.

**Attack Flow:**

1. Attacker obtains password hashes from breach
2. Attacker analyzes entropy of password strings (via length patterns)
3. Attacker identifies low-entropy passwords as easier to crack
4. Attacker prioritizes cracking low-entropy hashes first
5. Attacker gains quick access to subset of accounts

**Impact:** Accelerated credential cracking, prioritized account compromise.

---

## Technical Details

### Detection Method

Entropy analysis with statistical baseline and outlier detection.

**Algorithm:**

1. **Entropy Calculation**
   * Shannon entropy: Sum of (-p * log2(p)) for each character
   * Measures information content and randomness
   * Higher entropy = more random/unpredictable
   * Lower entropy = more repetitive/predictable

2. **Baseline Establishment**
   * Calculate mean and standard deviation of entropy across dataset
   * Determine baseline for password field entropy
   * Account for expected password patterns

3. **Outlier Detection**
   * Calculate Z-score for each value: (value - mean) / std_dev
   * Flag values with |Z-score| > 3.0 as outliers
   * 3σ standard deviation = 0.3% probability for normal distribution
   * More conservative threshold (2σ) for high sensitivity

**Accuracy:** High for mathematical detection. Statistical validity depends on sample size (minimum 30-100 samples recommended).

**False Positive Rate:** Low if threshold properly calibrated. May flag legitimately unusual passwords or special characters.

### Data at Risk

+ **Type of Data:** Password entropy/quality indicators, data integrity signals
+ **Sensitivity Level:** Low-Medium (indicates anomaly, not direct PII exposure)
+ **Regulatory Impact:** Data breach assessment, forensic analysis

---

## Mitigation Strategies

### Prevention

+ Use password requirements ensuring minimum entropy
+ Implement password strength meters
+ Educate users on strong password creation
+ Use password managers for random generation
+ Avoid predictable password patterns in test data

### Detection & Response

1. Monitor and Alert
   * Alert on significant entropy outliers in password fields
   * Correlate with other anomaly types for data quality assessment
   * Use entropy analysis to identify test vs. production data
   * Monitor entropy distribution changes (possible data manipulation)

2. Incident Response
   * If synthetic data detected: May indicate test data breach (lower impact)
   * If unusual patterns: Investigate for data corruption or injection
   * Entropy analysis aids forensic assessment of breach timeline
   * Use entropy metrics to prioritize password cracking defense

### User Controls

+ Configure entropy threshold (2σ, 3σ, custom)
+ Visualize entropy distribution in dataset
+ Filter results by entropy range
+ Correlation analysis with field type

---

## Testing & Validation

### Test Cases

1. **Positive Test:** High entropy outlier

   ```text
   xK7#mQ$2pL9@wR4vN8!tY3
   Base dataset average entropy: 4.2
   Outlier entropy: 7.8 (>3σ)
   ```

   Expected: Detected as high-entropy outlier

2. **Negative Test:** Normal entropy within range

   ```text
   Password123
   MyPassword2025
   Base dataset average entropy: 4.5
   Value entropy: 4.7 (within 3σ)
   ```

   Expected: Not detected

3. **Edge Case:** Low entropy outlier

   ```text
   aaaaaaaaaa
   Base dataset average entropy: 4.5
   Outlier entropy: 0.5 (<-3σ)
   ```

   Expected: Detected as low-entropy outlier

---

## Implementation Notes

+ Source code: [src/detection/anomaly_detection.rs](../../src/detection/anomaly_detection.rs)
+ Entropy calculation: Lines with entropy functions
+ Statistical baseline computation
+ Z-score calculation for outlier detection
+ Field-specific anomaly thresholds

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
