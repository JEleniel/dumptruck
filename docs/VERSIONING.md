# Versioning and Release Process

This document describes how Dumptruck uses semantic versioning and the release process.

## Semantic Versioning

Dumptruck follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** version when you make incompatible API changes
- **MINOR** version when you add functionality in a backwards-compatible manner
- **PATCH** version when you make backwards-compatible bug fixes

Version format: `vX.Y.Z` (e.g., `v1.2.3`)

Pre-release versions use hyphen format: `vX.Y.Z-rc.1` or `vX.Y.Z-beta.1`

## Release Process

### 1. Prepare Release

Before creating a release, ensure:

- [ ] All tests pass: `cargo test --all`
- [ ] All linting passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Security audit passes: `cargo audit`
- [ ] Documentation is current
- [ ] Changelog is updated (see [CHANGELOG.md](../CHANGELOG.md))

### 2. Create Release Tag

Create and push a semantic version tag:

```bash
# Create annotated tag
git tag -a v1.2.3 -m "Release version 1.2.3"

# Push tag to repository
git push origin v1.2.3
```

**Important**: Tags must follow the pattern `v*` (e.g., `v1.0.0`, `v2.1.0-rc.1`)

### 3. Automated Release Workflow

Once you push a tag matching `v*`, the GitHub Actions `Release` workflow automatically:

1. **Builds binaries** for multiple platforms:
   + Linux x86_64 (gnu)
   + Linux ARM64 (gnu)
   + macOS x86_64 (Intel)
   + macOS ARM64 (Apple Silicon)
   + Windows x86_64 (MSVC)

2. **Generates checksums** (SHA-256) for each binary

3. **Verifies reproducibility** by building twice and comparing hashes

4. **Creates GitHub Release** with:
   + Automatically generated changelog from commits
   + All binaries attached as release assets
   + SHA-256 checksums for verification

5. **Uploads to container registry** (ghcr.io) with semantic version tags

### 4. Verify Release

After the workflow completes:

1. Check the [GitHub Releases page](https://github.com/jeleniel/dumptruck/releases)
2. Verify all expected binaries are present
3. Download a binary and verify the checksum:

```bash
# Download and verify
curl -L https://github.com/jeleniel/dumptruck/releases/download/v1.2.3/dumptruck-linux-x86_64 -o dumptruck
sha256sum dumptruck
# Compare with dumptruck-linux-x86_64.sha256
```

## Version Bumping Guide

### Patch Release (Bug fixes)

```bash
# Current: v1.2.3
git tag -a v1.2.4 -m "Patch: Fix credential detection edge case"
```

### Minor Release (New features, backwards-compatible)

```bash
# Current: v1.2.3
git tag -a v1.3.0 -m "Minor: Add YAML format support and Ollama v0.3 compatibility"
```

### Major Release (Breaking changes)

```bash
# Current: v1.2.3
git tag -a v2.0.0 -m "Major: Refactor storage adapter API, add pgvector requirement"
```

### Pre-release

```bash
git tag -a v2.0.0-rc.1 -m "Release candidate: v2.0.0"
git tag -a v2.0.0-beta.1 -m "Beta release: v2.0.0"
```

## Build Reproducibility

All releases are built twice to verify reproducibility. The build hash must match both times. This ensures:

- No hidden build-time dependencies
- Deterministic compilation
- Supply chain transparency
- Binary integrity verification

If a build is not reproducible, the release workflow fails and blocks publishing.

## Changelog Format

Update [CHANGELOG.md](../CHANGELOG.md) before releasing. Use this format:

```markdown
## [1.2.3] - 2025-12-15

### Added
- New YAML format support for data ingestion
- Configurable embedding batch size for Ollama integration

### Changed
- Updated TLS certificate validation to support custom CA bundles
- Improved error messages for malformed CSV files

### Fixed
- Fixed Unicode normalization edge case with combining characters
- Corrected weak password detection for 8-character hashes

### Security
- Updated dependencies to patch rustls vulnerability

### Deprecated
- Legacy `--format=xml` flag (use `--format=json` instead)

### Removed
- Removed unused `--experimental-cache` option

## [1.2.2] - 2025-12-01
...
```

## Automated Docker Builds

When you push a tag:

- `v1.2.3` → `ghcr.io/jeleniel/dumptruck:v1.2.3` and `ghcr.io/jeleniel/dumptruck:latest`
- `v1.2.3-rc.1` → `ghcr.io/jeleniel/dumptruck:v1.2.3-rc.1` (no `latest` for pre-release)

Container images are signed and can be verified with `cosign`.

## Troubleshooting

### Release workflow failed

Check the Actions tab for the specific failure:

- **Build failed**: Check compiler/test errors in workflow logs
- **Reproducibility failed**: Rebuild and try again (usually indicates random seeds)
- **Upload failed**: Likely GitHub API issue; retry by pushing tag again

### Release already exists

If you need to replace a release:

```bash
# Delete the local tag
git tag -d v1.2.3

# Delete the remote tag
git push --delete origin v1.2.3

# Create new tag
git tag -a v1.2.3 -m "Updated release"
git push origin v1.2.3
```

## Key Release Milestones

- **v1.0.0**: Initial stable release (core pipeline, CLI, server mode)
- **v1.1.0**: Planned - k8s deployment support
- **v2.0.0**: Planned - Storage adapter refactoring, multi-tenant support

See [PROGRESS.md](../PROGRESS.md) for full roadmap.
