# Adapter Examples

This directory contains design and working examples for implementing adapters that extend Dumptruck's data processing pipeline. Adapters handle format parsing, normalization, enrichment, and storage.

## Overview

Dumptruck's extensibility is built around adapter patterns for:

- **Format Adapters** - Parse input files (CSV, JSON, YAML, Protocol Buffers, etc.)
- **Normalization Adapters** - Apply domain-specific canonicalization rules
- **Enrichment Adapters** - Add context (breach data, embeddings, threat intelligence)
- **Storage Adapters** - Persist results (PostgreSQL, S3, filesystem, etc.)

## Available Examples

### CSV Format Adapter

**Location:** `csv_adapter/`

A minimal, dependency-free CSV parser demonstrating RFC4180 compliance.

**Features:**

- Simple RFC4180 CSV parsing
- Quoted field support with embedded commas and newlines
- Escaped quote handling (`""` → `"`)
- Whitespace preservation
- Comprehensive test coverage (3 test cases)

**Run tests:**

```bash
cd csv_adapter
cargo test
```

**Use as a template for:**

- TSV (tab-separated values) parser
- PSV (pipe-separated values) parser
- Other delimited formats

## Adapter Design Principles

### 1. Keep It Simple

Write adapters as small, focused functions or structs:

```rust
pub fn parse_format(input: &str) -> Result<Vec<Vec<String>>, Error> {
    // Parse input into rows/fields
    // Return Result for error handling
}
```

### 2. Make It Testable

Include comprehensive unit tests covering:

- Happy path (valid input)
- Edge cases (empty fields, special characters)
- Error cases (malformed input, unexpected format)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_input() { /* ... */ }

    #[test]
    fn handles_edge_cases() { /* ... */ }

    #[test]
    #[should_panic]
    fn rejects_invalid_input() { /* ... */ }
}
```

### 3. Document Clearly

Include README with:

- Purpose and scope
- Features and limitations
- Usage examples
- Extension guidance
- Integration path to Dumptruck

## Integration Path

To add a custom adapter to Dumptruck:

### Step 1: Create Adapter Crate

```bash
cargo new --lib examples/adapters/my_adapter
```

### Step 2: Implement Core Function

```rust
// examples/adapters/my_adapter/src/lib.rs
pub fn parse_format(input: &str) -> Vec<Vec<String>> {
    // Implementation
}
```

### Step 3: Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        let result = parse_format("sample input");
        assert!(!result.is_empty());
    }
}
```

### Step 4: Wire Into Dumptruck Pipeline

In `src/adapters.rs`, add your adapter:

```rust
use my_adapter::parse_format;

pub fn detect_and_parse(
    content: &str,
    format: InputFormat,
) -> Result<Vec<Vec<String>>, Error> {
    match format {
        InputFormat::Csv => parse_csv(content),
        InputFormat::MyFormat => parse_format(content),
        _ => Err("Unsupported format"),
    }
}
```

## Sample Data for Testing

Use fixtures from `tests/fixtures/`:

- `test_creds_*.csv` - Credential datasets
- `json_credentials.json` - JSON structured data
- `yaml_credentials.yaml` - YAML format
- `special_characters.csv` - Edge case testing
- `unicode_addresses.csv` - International character handling

## Architecture Reference

For detailed adapter contracts and interfaces, see:

- [INTERFACES.md](../../docs/architecture/INTERFACES.md) - Formal adapter contracts
- [DATA_FLOW_AND_EXTENSIBILITY.md](../../docs/architecture/DATA_FLOW_AND_EXTENSIBILITY.md) - Pipeline integration patterns
- [COMPONENTS.md](../../docs/architecture/COMPONENTS.md) - Component responsibilities

## Contributing a New Adapter

1. Create a minimal working adapter in `examples/adapters/`
2. Add comprehensive tests (aim for >80% coverage)
3. Write clear README with examples
4. Submit PR with adapter + integration tests
5. Include sample input/output files

## Quick Commands

```bash
# Test all adapters
cd examples/adapters/csv_adapter && cargo test

# Format code
cargo fmt --all

# Check for issues
cargo clippy --all

# Generate documentation
cargo doc --no-deps --open
```

## Next Steps

- Create **TSV adapter** using CSV as template
- Add **JSON adapter** with serde integration
- Implement **YAML adapter** with serde_yaml
- Create **enrichment adapter** example (WASM-based)
- Build **storage adapter** for S3 or Snowflake
