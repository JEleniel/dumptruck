# Security

Principles

- Least privilege for components and operators.
- Defense in depth: encrypt in transit and at rest, minimize data exposure.
- Privacy-by-design: historic values are non-reversible.

Data classification and handling

- Raw input: sensitive; process in-memory when possible.
- Derived/enriched data: treat as confidential; redact when storing if not needed.
- History: store only keyed hashes (HMAC or KDF) of values. Keep HMAC keys in secure secret store.

Encryption

- TLS for all network transport.
- At-rest: use encrypted storage where possible; encrypt exported snapshots.

Authentication & authorization

- Server mode: OAuth2/OIDC for user authentication and client credentials for machine access.
- API: role-based access to endpoints (upload, read, admin).

Secrets management

- Do not store plaintext secrets in repo. Use environment or external secret store.

Key management

- HMAC key rotation: store key versions and migrate history hashing with dual-hash windows until re-hash safe.

Threat model (top items)

- Data exfiltration from compromised server — mitigations: encrypted history, minimal retention, access controls.
- Insider risk — mitigations: role separation, audit logging, least privilege.
- Supply chain (build) — mitigations: deterministic builds, signed releases.

Auditing and logging

- Structured logs with access events, but do not log raw sensitive values.
- Retain logs per policy; rotate and protect log storage.

Testing and validation

- Fuzz and property tests for parsers and normalization rules.
- Static analysis and dependency vulnerability scanning during CI.
