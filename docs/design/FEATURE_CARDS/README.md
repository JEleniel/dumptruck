# Feature Cards

This folder contains concise feature cards that capture the goals, acceptance criteria, and implementation notes for Dumptruck's major design areas. Each card is focused and actionable so engineers and designers can quickly understand scope and constraints.

Cards included:

- `ingestion.md` — supported input formats and pipeline
- `normalization.md` — normalization rules and schema mapping
- `enrichment.md` — deduplication, correlation, and identifier enrichment
- `analysis.md` — bulk analysis features and comparison modes
- `history_privacy.md` — hashed historic storage and privacy guarantees
- `server_modes.md` — CLI and Server operation modes and API surface
- `security.md` — authentication, encryption, and hardening
- `extensibility.md` — plugin and format extensibility model
- `storage.md` — storage model, hashing, indexing, retention

Use these cards as lightweight design artifacts suitable for planning, estimating, and handoff.

---

## Threat Modeling Integration

Each feature card is mapped to corresponding OWASP Threat Cards that document the security risks and detection mechanisms related to that feature. See [Threat Library Index](../../threat/README.md#feature-to-threat-mapping) for the complete mapping of features to threats.

**Quick Links to Related Threats by Feature:**

- [Ingestion](./ingestion.md) → Threats: Credit cards, IP addresses, weak passwords
- [Normalization](./normalization.md) → Threats: Email, phone, national ID, names
- [History & Privacy](./history_privacy.md) → Threats: Email, SSN, credit cards, bank accounts, crypto
- [Storage & Hashing](./storage.md) → Threats: All detection types via secure hashing
- [Analysis](./analysis.md) → Threats: Weak passwords, entropy outliers, anomalies
- [Enrichment](./enrichment.md) → Threats: Email HIBP lookup, SSN correlation, identity linking
- [Security & Authentication](./security.md) → Threats: All types via TLS, OAuth, and role-based access
- [Server & CLI Modes](./server_modes.md) → Threats: Centralized threat analysis with audit logging
- [Extensibility & Formats](./extensibility.md) → Threats: Custom detection rules per organization
