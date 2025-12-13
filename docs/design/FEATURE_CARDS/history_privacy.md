# Feature: History & Privacy

Summary

- Purpose: Store historic indicators safely and enable detection of repeats without exposing raw sensitive data.

Goals

- Persist only irreversible representations (salted hashes, bloom filters, or similar) for items of interest.
- Allow TTL and retention policies for historic data.

Acceptance Criteria

- Historic lookup detects repeats but raw values are never stored in cleartext.
- Retention and deletion policies are enforceable and auditable.

Implementation Notes

- Use HMAC/Scrypt-like salted hashing with per-deployment secrets for stored identifiers.
- Consider privacy-preserving bloom filters or keyed hashing for fast lookups.
