# OWASP Threat Card Template

Use this template to document threat cards for each detection capability in the Dumptruck system. Threat cards help identify, understand, and mitigate security risks.

---

## Detection: _(Detection Name)_

### Overview

**Threat ID:** `DETECTION-[NUMBER]`

**Category:** PII Detection | Credential Detection | Anomaly Detection

**Severity:** Critical | High | Medium | Low

**Last Updated:** `YYYY-MM-DD`

---

## Threat Description

### What is Detected?

Brief description of what this detection identifies.

### Why It Matters

Explanation of the security and privacy implications of not detecting this information.

---

## Attack Scenarios

### Scenario 1: _(Specific Attack)_

**Attacker Goal:** What is the attacker trying to achieve?

**Attack Flow:**

1. Step 1
2. Step 2
3. Step 3

**Impact:** What happens if this attack succeeds?

### Scenario 2: _(Alternative Attack)_

Repeat structure as needed.

---

## Technical Details

### Detection Method

How does Dumptruck identify this threat?

**Pattern/Algorithm:** Description of the detection algorithm or regex pattern used.

**Accuracy:** Estimated precision and recall rates.

**False Positive Rate:** Known issues or edge cases that may trigger false positives.

### Data at Risk

- **Type of Data:** What kind of information is at risk?
- **Sensitivity Level:** How sensitive is this data?
- **Regulatory Impact:** GDPR, CCPA, PCI-DSS, HIPAA, etc.

---

## Mitigation Strategies

### Prevention

What measures prevent this threat from occurring?

### Detection & Response

1. Monitor and Alert
   + Alert on detection of this threat
   + Set appropriate thresholds

2. Incident Response
   + Steps to investigate
   + Steps to remediate
   + Communication procedures

### User Controls

- Configuration options
- Sensitivity adjustments
- Allowlist/denylist capabilities

---

## Testing & Validation

### Test Cases

1. **Positive Test:** Example input that should trigger detection

   ```text
   Example here
   ```

   Expected: Detected

2. **Negative Test:** Example input that should NOT trigger detection

   ```text
   Example here
   ```

   Expected: Not detected

3. **Edge Case:** Boundary condition testing

   ```text
   Example here
   ```

   Expected: Result as documented

---

## References

### Related Threats

- Related threat cards if any

### External Resources

- OWASP resources if applicable
- NIST resources if applicable
- CWE/CVE identifiers if applicable

### Implementation Notes

- Link to source code: `src/detection/[module].rs`
- Related documentation: [Links]

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Implementation  |           |      |
