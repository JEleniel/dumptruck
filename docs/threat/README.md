# Dumptruck OWASP Threat Card Library

This directory contains comprehensive OWASP Threat Cards for all detection capabilities in the Dumptruck threat intelligence and breach analysis system.

## Overview

Threat Cards document security risks, attack scenarios, technical details, and mitigation strategies for each detection mechanism. They follow the OWASP Threat Card methodology for structured threat modeling and risk communication.

## Template

Start with [THREAT_CARD_TEMPLATE.md](./THREAT_CARD_TEMPLATE.md) as a reference for creating new threat cards.

---

## Detection Categories

### Personal Identifiable Information (PII) Detection

| Threat ID | Detection | Severity | Status |
| --- | --- | --- | --- |
| [DETECTION-001](./DETECTION-001-EMAIL.md) | Email Address | High | Complete |
| [DETECTION-002](./DETECTION-002-PHONE.md) | Phone Number | High | Complete |
| [DETECTION-003](./DETECTION-003-SSN.md) | Social Security Number | Critical | Complete |
| [DETECTION-004](./DETECTION-004-NATID.md) | National ID | Critical | Complete |
| [DETECTION-010](./DETECTION-010-NAME.md) | Name | Medium | Complete |
| [DETECTION-011](./DETECTION-011-ADDRESS.md) | Mailing Address | Medium | Complete |

### Financial Identifiers

| Threat ID | Detection | Severity | Status |
| --- | --- | --- | --- |
| [DETECTION-005](./DETECTION-005-CREDITCARD.md) | Credit Card Number | Critical | Complete |
| [DETECTION-006](./DETECTION-006-IPADDRESS.md) | IP Address | Medium | Complete |
| [DETECTION-007](./DETECTION-007-BANKACCOUNT.md) | Bank Account/IBAN/SWIFT | Critical | Complete |
| [DETECTION-008](./DETECTION-008-CRYPTO.md) | Cryptocurrency Address | High | Complete |
| [DETECTION-009](./DETECTION-009-WALLET.md) | Digital Wallet Token | High | Complete |

### Credential Quality Assessment

| Threat ID | Detection | Severity | Status |
| --- | --- | --- | --- |
| [DETECTION-012](./DETECTION-012-WEAKPASS.md) | Weak Password | High | Complete |

### Anomaly Detection

| Threat ID | Detection | Severity | Status |
| --- | --- | --- | --- |
| [DETECTION-013](./DETECTION-013-ENTROPY.md) | Entropy Outlier | Medium | Complete |
| [DETECTION-014](./DETECTION-014-ANOMALY.md) | Anomaly & Novelty | Medium | Complete |

---

## Feature-to-Threat Mapping

This section maps Dumptruck features to the threats they detect and mitigate.

### Ingestion Feature

**Purpose:** Accept bulk data dumps in many formats and feed them reliably into the pipeline.

**Related Threats:**

- [DETECTION-005](./DETECTION-005-CREDITCARD.md) - Detects credit cards in ingested data
- [DETECTION-006](./DETECTION-006-IPADDRESS.md) - Identifies IP addresses in ingested data
- [DETECTION-012](./DETECTION-012-WEAKPASS.md) - Detects weak passwords in ingested records

**Security Implications:** Safe ingestion prevents malformed/malicious data from corrupting the system while preserving evidence chain.

### Normalization Feature

**Purpose:** Convert diverse inputs into consistent internal representation with canonical rules.

**Related Threats:**

- [DETECTION-001](./DETECTION-001-EMAIL.md) - Canonicalizes email addresses for consistent detection
- [DETECTION-002](./DETECTION-002-PHONE.md) - Normalizes phone number formats for comparison
- [DETECTION-004](./DETECTION-004-NATID.md) - Standardizes national ID formats across countries
- [DETECTION-010](./DETECTION-010-NAME.md) - Normalizes name fields for matching

**Security Implications:** Normalization enables detection of duplicates and variants that would be missed with raw format matching, improving breach scope assessment.

### History & Privacy Feature

**Purpose:** Store historic indicators safely using only irreversible hashes, enabling duplicate detection without exposing raw data.

**Related Threats:**

- [DETECTION-001](./DETECTION-001-EMAIL.md) - Hash-based email deduplication
- [DETECTION-003](./DETECTION-003-SSN.md) - Non-reversible SSN storage for privacy
- [DETECTION-005](./DETECTION-005-CREDITCARD.md) - Secure credit card hash storage
- [DETECTION-007](./DETECTION-007-BANKACCOUNT.md) - Bank account tokenization for privacy
- [DETECTION-008](./DETECTION-008-CRYPTO.md) - Cryptocurrency address hashing

**Security Implications:** Irreversible hashing ensures historical data cannot expose raw sensitive information even if database is compromised, meeting privacy-by-design requirements.

### Storage & Hashing Feature

**Purpose:** Store enriched records with hashed identifiers and efficient lookup indexes.

**Related Threats:**

- [DETECTION-001](./DETECTION-001-EMAIL.md) - Email index for O(1) duplicate lookup
- [DETECTION-002](./DETECTION-002-PHONE.md) - Phone number indexing for fast matching
- [DETECTION-003](./DETECTION-003-SSN.md) - SSN hash storage with secure encryption
- [DETECTION-005](./DETECTION-005-CREDITCARD.md) - Credit card hash indexing (PCI DSS compliant)
- [DETECTION-007](./DETECTION-007-BANKACCOUNT.md) - Bank account token storage
- [DETECTION-012](./DETECTION-012-WEAKPASS.md) - Rainbow table index for weak password detection

**Security Implications:** Secure storage with hashing and encryption protects sensitive data at rest while enabling fast duplicate detection.

### Analysis Feature

**Purpose:** Provide bulk-analysis operations to find new, repeated, and anomalous leaked data.

**Related Threats:**

- [DETECTION-012](./DETECTION-012-WEAKPASS.md) - Analyze weak password prevalence in dataset
- [DETECTION-013](./DETECTION-013-ENTROPY.md) - Identify entropy outliers indicating synthetic data
- [DETECTION-014](./DETECTION-014-ANOMALY.md) - Detect unseen combinations and rare domains
- [DETECTION-001](./DETECTION-001-EMAIL.md) - Analyze email domain distribution and new domains
- [DETECTION-006](./DETECTION-006-IPADDRESS.md) - Geographic analysis of IP addresses

**Security Implications:** Anomaly detection reveals data quality issues, synthetic data injection, and unusual patterns suggesting targeted compromises or data manipulation.

### Enrichment Feature

**Purpose:** Enrich incoming records with metadata, correlations, and identity linking.

**Related Threats:**

- [DETECTION-001](./DETECTION-001-EMAIL.md) - HIBP breach lookup for detected emails
- [DETECTION-003](./DETECTION-003-SSN.md) - Historical SSN correlation for impact assessment
- [DETECTION-005](./DETECTION-005-CREDITCARD.md) - Card issuer correlation and fraud indicators
- [DETECTION-007](./DETECTION-007-BANKACCOUNT.md) - Bank account correlation across breaches
- [DETECTION-008](./DETECTION-008-CRYPTO.md) - Blockchain address analysis and transaction correlation
- [DETECTION-010](./DETECTION-010-NAME.md) - Identity linking across multiple records

**Security Implications:** Enrichment correlates fragmented data to understand complete identity exposure and downstream risks.

### Security & Authentication Feature

**Purpose:** Protect data in transit and at rest; authenticate and authorize access.

**Related Threats:**

- All threat cards - TLS 1.3+ protects all sensitive data in transit
- All threat cards - OAuth2/OIDC prevents unauthorized access to breach data
- [DETECTION-003](./DETECTION-003-SSN.md) - Role-based access control for SSN data (critical)
- [DETECTION-005](./DETECTION-005-CREDITCARD.md) - Restricted access to credit card data (PCI DSS requirement)
- [DETECTION-007](./DETECTION-007-BANKACCOUNT.md) - Audit logging for bank account access

**Security Implications:** Strong authentication and authorization prevent unauthorized exposure of sensitive threat data to unvetted users.

### Server & CLI Modes Feature

**Purpose:** Support both single-run CLI workflows and long-running server analysis with ingestion API.

**Related Threats:**

- All threat cards - CLI provides isolated processing without network exposure
- All threat cards - Server API enables centralized threat analysis with authentication
- [DETECTION-001](./DETECTION-001-EMAIL.md) - Server API for automated HIBP enrichment
- [DETECTION-006](./DETECTION-006-IPADDRESS.md) - Geolocation API integration for threat intelligence

**Security Implications:** Server mode enables collaborative threat analysis with audit logging, while CLI mode ensures air-gapped environments can process sensitive data safely.

### Extensibility & Formats Feature

**Purpose:** Plugin architecture for new parsers, normalizers, and enrichment modules.

**Related Threats:**

- [DETECTION-001](./DETECTION-001-EMAIL.md) - Custom email validation rules per organization
- [DETECTION-012](./DETECTION-012-WEAKPASS.md) - Custom weak password lists per organization
- [DETECTION-013](./DETECTION-013-ENTROPY.md) - Custom anomaly detection thresholds
- [DETECTION-014](./DETECTION-014-ANOMALY.md) - Organization-specific baseline definitions

**Security Implications:** Extensibility enables organizations to tailor threat detection to their specific risk profile and data characteristics without forking the codebase.

---

## How to Use These Threat Cards

### For Security Teams

1. **Risk Assessment:** Use threat cards to understand exposure from each detection type
2. **Incident Response:** Reference attack scenarios when responding to breaches
3. **Mitigation Planning:** Review mitigation strategies for security controls
4. **Staff Training:** Use scenarios for security awareness training
5. **Vendor Evaluation:** Use cards to evaluate third-party tools and integrations

### For Compliance

1. **Regulatory Mapping:** Threat cards show regulatory impacts (GDPR, CCPA, HIPAA, PCI DSS)
2. **Breach Notification:** Cards help quantify breach impact and notification requirements
3. **Risk Registers:** Add threat cards to organizational risk register
4. **Audit Support:** Provide evidence of threat understanding to auditors

### For Developers

1. **Implementation Reference:** Cards reference source code locations
2. **Test Cases:** Each card includes positive, negative, and edge case test examples
3. **Algorithm Documentation:** Technical sections document detection algorithms
4. **Integration Points:** Cards identify external system integrations (HIBP, Ollama, etc.)

### For Analysts

1. **Context for Alerts:** Understand what triggered detection alerts
2. **False Positive Assessment:** Cards help evaluate detection accuracy
3. **Priority Triaging:** Severity ratings help triage work
4. **Enrichment Planning:** Cards suggest data enrichment and correlation opportunities

---

## Card Structure

Each threat card includes:

- **Overview:** Threat ID, category, severity, and last update
- **Threat Description:** What is detected and why it matters
- **Attack Scenarios:** Realistic attack flows with impact assessment
- **Technical Details:** Detection algorithms, accuracy, false positive rates
- **Data at Risk:** Type, sensitivity level, regulatory impact
- **Mitigation Strategies:** Prevention, detection, response, and user controls
- **Testing & Validation:** Test cases with positive, negative, and edge cases
- **Implementation Notes:** Links to source code and documentation

---

## Severity Levels

- **Critical:** Enables direct account takeover, identity theft, or significant financial loss
- **High:** Enables targeted attacks, credential compromise, or location-based harm
- **Medium:** Indicates data quality issues, enables social engineering, or reveals infrastructure

---

## Integration with Detection System

These threat cards document the detection capabilities found in:

- [src/detection/npi_detection.rs](../../src/detection/npi_detection.rs) - PII/NPI detection
- [src/detection/rainbow_table.rs](../../src/detection/rainbow_table.rs) - Weak password detection
- [src/detection/anomaly_detection.rs](../../src/detection/anomaly_detection.rs) - Anomaly detection
- [src/detection/detection.rs](../../src/detection/detection.rs) - Unified detection pipeline

---

## External References

### OWASP Resources

- [OWASP Threat Modeling](https://owasp.org/www-community/Threat_Modeling)
- [OWASP Threat Modeling Process](https://owasp.org/www-community/threats/)
- [OWASP Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)

### Regulatory & Compliance

- **GDPR:** Personal data protection (EU/EEA)
- **CCPA:** California privacy rights
- **HIPAA:** Health data protection (US)
- **PCI DSS:** Payment card data protection
- **SOC 2:** Security and compliance framework

### Threat Intelligence

- [Have I Been Pwned](https://haveibeenpwned.com/) - Breach database
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [CWE/CVE Databases](https://cwe.mitre.org/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)

---

## Contributing

When adding new detection capabilities, create accompanying threat cards:

1. Copy [THREAT_CARD_TEMPLATE.md](./THREAT_CARD_TEMPLATE.md)
2. Name file as `DETECTION-###-SHORTNAME.md`
3. Increment DETECTION-### number sequentially
4. Complete all sections per template
5. Update this README with entry in appropriate category
6. Ensure markdown linting compliance

---

## Maintenance

Threat cards should be reviewed and updated:

- **Quarterly:** Review for new attack vectors or information updates
- **After Incidents:** Update based on real-world attacks/findings
- **Technology Changes:** Revisit when detection algorithms are updated
- **Regulatory Changes:** Update regulatory impact sections as needed

---

## Sign-Off

| Role            | Signature | Date |
| --------------- | --------- | ---- |
| Security Review |           |      |
| Documentation   |           |      |
| Compliance      |           |      |
