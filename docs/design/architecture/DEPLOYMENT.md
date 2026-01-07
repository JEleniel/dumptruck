# Deployment

Supported modes

- Single-node CLI: run locally on an analyst's machine; minimal runtime (Rust binary).
- Server: containerized, scalable, behind load-balancer and ingress.

Runtime requirements

- Rust runtime (static binary produced by `cargo build --release`).
- Persistent storage for the SQLite database file and exported snapshots (backups).
- TLS termination at ingress or load balancer.

Configuration

- Use environment variables and a configuration file.
- Required secrets: HMAC key for history hashing, OAuth2 client secrets.

Example container flow (conceptual)

- Build: `cargo build --release` -> produce `dumptruck` binary
- Container: small distro (e.g., distroless), copy binary and config, run as non-root user.

Example Kubernetes sketch (conceptual)

- Deployment with 1..N replicas.
- Horizontal Pod Autoscaler based on CPU or queue length.
- PersistentVolume for the SQLite database file and exported snapshots.
- Service + Ingress with TLS; external OIDC provider for auth.

Operational notes

- Metrics: expose Prometheus metrics endpoint.
- Logs: structured JSON logs to stdout.
- Backups: periodic export of hashed history state; rotate HMAC keys carefully (maintain ability to re-hash when safe).
