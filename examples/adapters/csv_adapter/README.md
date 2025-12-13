# CSV Adapter Example

This is a minimal example crate showing a CSV format adapter for Dumptruck. It provides a `parse_csv` function with comprehensive unit tests to demonstrate the expected behavior for adapters: simple, testable, and maintainable.

## Purpose

- Provide a working example of a `FormatAdapter` that parses input into canonical rows/fields
- Demonstrate RFC4180 CSV parsing with proper quote handling
- Show how to test adapter behavior with various edge cases
- Be small and self-contained so contributors can iterate quickly

## Features

The `parse_csv` function handles:

- **Simple CSV** with headers and data rows
- **Quoted fields** with embedded commas and newlines
- **Escaped quotes** (RFC4180: `""` represents a literal `"`)
- **Mixed whitespace** and empty fields
- **Unterminated quoted fields** (graceful recovery)
- **Windows line endings** (`\r\n`)

## Usage

### Run Tests

```bash
cd examples/adapters/csv_adapter
cargo test
```

Output:

```text
running 3 tests
test tests::handles_empty_last_line ... ok
test tests::handles_escaped_and_multiline ... ok
test tests::parses_simple_csv ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

### Use in Your Code

```rust
use csv_adapter_example::parse_csv;

fn main() {
    let input = r#"email,password
alice@example.com,"SecureP@ss123"
bob@example.org,"hunter2"
"#;
    
    let rows = parse_csv(input);
    
    for row in rows {
        println!("Row with {} fields", row.len());
        for field in row {
            println!("  - {}", field);
        }
    }
}
```

## Test Coverage

The example includes three test cases covering:

1. **Simple CSV** - Basic parsing with unquoted fields
2. **Empty lines** - Proper handling of trailing newlines
3. **Quoted multiline fields** - Complex RFC4180 compliance with escaped quotes

## Extending the Adapter

To adapt this for other formats, follow this pattern:

```rust
pub fn parse_tsv(input: &str) -> Vec<Vec<String>> {
    // Similar structure to parse_csv
    // Replace comma delimiter with tab (\t)
    // Handle tab-specific edge cases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tsv_with_tabs() {
        let input = "col1\tcol2\tcol3\n1\t2\t3\n";
        let out = parse_tsv(input);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], vec!["col1", "col2", "col3"]);
    }
}
```

## Integration with Dumptruck

This adapter can be integrated into Dumptruck's pipeline via:

1. **Input Stage** - Parse raw file content
2. **Normalization Stage** - Extract and normalize fields
3. **Enrichment Stage** - Apply domain-specific logic
4. **Output Stage** - Format results for storage

See [INTERFACES.md](../../docs/architecture/INTERFACES.md) for the complete adapter contract.

## Next Steps

- Add error handling with `Result<T, E>` return types
- Implement TSV (tab-separated values) variant
- Add benchmarks for large files
- Integrate with storage adapter for direct database writes
