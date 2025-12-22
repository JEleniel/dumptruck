# Threat Card: Weak Password Detection

## Overview

**Threat ID:** `DETECTION-012-WEAKPASS`

**Category:** Credential Quality Detection

**Severity:** High

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects weak passwords through multiple mechanisms:

+ **Rainbow Table Matching:** 40+ common plaintext passwords (password, 123456, admin, etc.)
+ **Hashed Weak Passwords:** SHA-256 hashes matching known weak passwords
+ **Hash Format Detection:** bcrypt, scrypt, argon2, MD5, SHA1, SHA256 hashes
+ **Keyboard Patterns:** 3-5 character sequential patterns (qwerty, asdf, zxcv)

### Why It Matters

Weak password detection alerts to:

+ Passwords easily guessable via brute force or dictionary attacks
+ Credentials already compromised in other breaches
+ Pre-hashed credentials in rainbow tables
+ Insufficient password complexity in dataset
+ High-risk breach correlation with weak passwords

---

## Attack Scenarios

### Scenario 1: Dictionary Attack on Hash Dumps

**Attacker Goal:** Crack password hashes using weak password list.

**Attack Flow:**

1. Attacker obtains breach data with password hashes
2. Attacker uses rainbow table with 40+ common weak passwords
3. Attacker hashes weak passwords and compares to breach hashes
4. Attacker cracks a significant portion of hashes immediately
5. Attacker uses cracked passwords for lateral movement/account takeover

**Impact:** Widespread account compromise, credential reuse across services.

### Scenario 2: Brute Force Attack Using Common Passwords

**Attacker Goal:** Gain access to accounts using common passwords.

**Attack Flow:**

1. Attacker identifies high-value accounts from breach data (admin, CEO, etc.)
2. Attacker knows passwords are commonly weak
3. Attacker attempts login with 40 most common passwords
4. Attacker rapidly gains access to accounts
5. Attacker establishes persistence or exfiltrates data

**Impact:** Account takeover, system compromise, data breach.

### Scenario 3: Cloud Account Takeover via Weak Credentials

**Attacker Goal:** Access victim's cloud accounts using weak passwords.

**Attack Flow:**

1. Attacker obtains email and weak password from breach
2. Attacker attempts login to major cloud providers (AWS, Azure, GCP)
3. Account uses weak password, no 2FA
4. Attacker gains cloud account access
5. Attacker accesses stored data, compute resources, or backups

**Impact:** Cloud infrastructure compromise, data exfiltration, ransomware.

---

## Technical Details

### Detection Method

Multi-layered weak password detection approach.

**Algorithm:**

1. **Plaintext Detection**
   * Compare against 40+ known weak passwords (password, 123456, admin, etc.)
   * Check for keyboard patterns (qwerty, asdf, zxcv)
   * Length checks (warn on passwords < 8 characters)

2. **Hashed Password Detection**
   * SHA-256 hash comparison against weak password rainbow table
   * Detect pre-hashed credentials using format signatures:
     + bcrypt: $2a$, $2b$, $2y$ prefix
     + scrypt: $7$ prefix
     + argon2: $argon2i$, $argon2d$, $argon2id$ prefix
     + MD5: 32 hex characters
     + SHA1: 40 hex characters
     + SHA256: 64 hex characters

3. **Entropy Analysis**
   * Low-entropy passwords flagged
   * Character set diversity analysis
   * Pattern repetition detection

**Accuracy:** Very high for exact matches. False positive rate depends on rainbow table comprehensiveness.

**False Positive Rate:** Very low. Exact matching prevents false positives. Hash format detection may flag legitimate long hex strings.

### Data at Risk

+ **Type of Data:** Password hashes, plaintext passwords, credential strength indicators
+ **Sensitivity Level:** Critical (directly enables account compromise)
+ **Regulatory Impact:** GDPR, password security regulations, HIPAA, PCI DSS

---

## Mitigation Strategies

### Prevention

+ Enforce minimum password length (12+ characters)
+ Require password complexity (uppercase, lowercase, numbers, symbols)
+ Implement password history to prevent reuse
+ Use password managers for random strong passwords
+ Implement rate limiting on login attempts
+ Enforce multi-factor authentication (MFA)
+ Implement passwordless authentication where possible

### Detection & Response

1. Monitor and Alert
   * Alert on weak password detection in breach data
   * Calculate percentage of weak passwords
   * Correlate weak passwords with other high-risk PII
   * Alert on hash format detection (indicates stored credentials)

2. Incident Response
   * If weak passwords detected in dataset:
     + Invalidate those credentials immediately
     + Force password reset on affected accounts
     + Monitor for unauthorized access attempts using weak passwords
   * If hash formats detected:
     + Assess hash strength (bcrypt > scrypt > MD5)
     + Estimate crack time for hashes
   * Recommend credential rotation and security review

### User Controls

+ Enable/disable weak password detection
+ Configure sensitivity levels
+ Set minimum password complexity requirements
+ Integration with password policy enforcement
+ Alert configuration for weak password volume thresholds

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Common weak passwords

   ```text
   password
   123456
   admin
   qwerty
   123456789
   ```

   Expected: Detected as weak

2. **Negative Test:** Strong passwords

   ```text
   Tr0ub4dor&3
   xkcd1701StarTrek
   MyS3cur3P@ssw0rd!
   ```

   Expected: Not detected as weak

3. **Edge Case:** Hash formats

   ```text
   $2b$12$abcdefghijklmnopqrstuvwxyz
   5f4dcc3b5aa765d61d8327deb882cf99
   e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
   ```

   Expected: Detected as hashed password

---

## Implementation Notes

+ Source code: [src/detection/rainbow_table.rs](../../src/detection/rainbow_table.rs)
+ Weak password detection: Detection module
+ Rainbow table loading and validation
+ Hash format detection functions
+ Entropy calculation for password strength

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
