# Threat Card: IP Address Detection

## Overview

**Threat ID:** `DETECTION-006-IPADDRESS`

**Category:** Network Intelligence

**Severity:** Medium

**Last Updated:** 2025-12-21

---

## Threat Description

### What is Detected?

Dumptruck detects both IPv4 and IPv6 addresses, with intelligent filtering to exclude private/internal addresses. Detects public IP addresses that reveal external network connections and origin locations.

**Excluded Ranges (No Detection):**

+ Private RFC 1918: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
+ Loopback: 127.0.0.0/8
+ Link-local: 169.254.0.0/16
+ Broadcast: 255.255.255.255

### Why It Matters

Public IP addresses in breach data reveal:

+ Geographic location of breach origin or user
+ Internet service provider information
+ Potential for geolocation-based targeting
+ Device identification when correlated with other data
+ Network topology disclosure
+ Exposure of legitimate business infrastructure

---

## Attack Scenarios

### Scenario 1: Targeted Network Attack

**Attacker Goal:** Attack specific organization's infrastructure discovered in breach.

**Attack Flow:**

1. Attacker obtains breach data containing public IP addresses from organization
2. Attacker reverse-DNS looks up IP addresses to identify domain
3. Attacker maps IP ranges to organization's network infrastructure
4. Attacker uses IP information for targeted scanning and reconnaissance
5. Attacker discovers vulnerabilities specific to that infrastructure

**Impact:** Network compromise, data exfiltration, operational disruption.

### Scenario 2: Geolocation-Based Targeting

**Attacker Goal:** Target users or infrastructure in specific geographic regions.

**Attack Flow:**

1. Attacker extracts IP addresses from breach data
2. Attacker uses geolocation API to map IPs to cities/countries
3. Attacker filters for targets in specific region (high-value markets, etc.)
4. Attacker launches region-specific phishing or malware campaigns
5. Attacker targets infrastructure with region-specific vulnerabilities

**Impact:** Targeted attacks, higher success rate, infrastructure compromise.

### Scenario 3: Law Enforcement Evasion (Victim Perspective)

**Attacker Goal:** Track alleged attacker's location from logged IP addresses.

**Attack Flow:**

1. Organization has IP logs from breach or attack
2. IP addresses reveal geolocation of activity
3. Attacker uses VPN/proxy to mask true location
4. Law enforcement must subpoena VPN providers to identify attacker
5. IP address disclosure in breach simplifies attacker prosecution

**Impact:** Positive for security teams: easier law enforcement cooperation.

---

## Technical Details

### Detection Method

IP detection uses regex-based pattern matching with range validation.

**Pattern/Algorithm:**

+ IPv4: Regex matching XXX.XXX.XXX.XXX format with valid octet ranges (0-255)
+ IPv6: Hex notation with validation of format and valid characters
+ Private range exclusion checks
+ Loopback and special-use address filtering
+ Valid CIDR range validation if applicable

**Accuracy:** Very high for valid IP address format matching.

**False Positive Rate:** Very low. May match sequences that look like IPs but aren't (e.g., version numbers like 192.168.1.1 in product names).

### Data at Risk

+ **Type of Data:** Public IP addresses, geolocation, network infrastructure identifiers
+ **Sensitivity Level:** Medium (reveals location and infrastructure, but not directly identifying)
+ **Regulatory Impact:** GDPR (IP = personal data under GDPR), privacy laws

---

## Mitigation Strategies

### Prevention

+ Use VPN for all internet traffic (privacy protection)
+ Implement strict firewall rules on legitimate infrastructure
+ Use proxy services to mask origin IPs
+ Rotate public IPs regularly
+ Minimize exposure of IP addresses in logs or data

### Detection & Response

1. Monitor and Alert
   * Track volumes of public IP addresses in breach data
   * Identify IP ranges owned by organization
   * Map IPs to organizational infrastructure for scope assessment

2. Incident Response
   * Assess if breached IPs belong to organization (internal incident)
   * If internal, may indicate system compromise
   * Reverse-DNS lookup to identify services/systems
   * Check firewall logs for suspicious activity from those IPs
   * Notify ISP of potential infrastructure exposure

### User Controls

+ Filter IP address detection results
+ Identify internal vs. external IP ranges
+ Alert on detection of internal IP ranges in external breach data

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Public IPv4 addresses

   ```text
   8.8.8.8
   1.1.1.1
   208.67.222.222
   ```

   Expected: Detected

2. **Negative Test:** Private addresses (excluded)

   ```text
   192.168.1.1
   10.0.0.1
   127.0.0.1
   169.254.1.1
   ```

   Expected: Not detected (filtered)

3. **Edge Case:** IPv6 addresses

   ```text
   2001:4860:4860::8888
   fe80::1
   ```

   Expected: First detected; second not detected (link-local filtered)

---

## Implementation Notes

+ Source code: [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs#L107)
+ IPv4 validation: Lines 107-137
+ IPv6 validation: Lines 167-175
+ Private range filtering: Lines 92-106 and 138-166
+ Regex patterns: [src/regexes.rs](../../src/regexes.rs)

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
