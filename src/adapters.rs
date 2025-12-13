//! Adapter trait and a small CSV adapter implementation for examples and tests.

// `normalization` is only needed for tests; import inside `cfg(test)` to
// avoid unused-import warnings in non-test builds.

/// Simple trait for format adapters: parse input into rows of fields.
pub trait FormatAdapter {
	fn parse(&self, input: &str) -> Vec<Vec<String>>;
}

/// A tiny CSV adapter implementing `FormatAdapter`.
pub struct CsvAdapter;

impl CsvAdapter {
	pub fn new() -> Self {
		CsvAdapter
	}
}

impl FormatAdapter for CsvAdapter {
	fn parse(&self, input: &str) -> Vec<Vec<String>> {
		// RFC4180-like parser: handles quoted fields with doubled quotes and
		// allows newlines inside quoted fields.
		let mut rows: Vec<Vec<String>> = Vec::new();
		let mut row: Vec<String> = Vec::new();
		let mut field = String::new();
		let mut in_quotes = false;
		let mut chars = input.chars().peekable();

		while let Some(ch) = chars.next() {
			match ch {
				'"' => {
					if in_quotes {
						// possible escaped quote
						if let Some('"') = chars.peek() {
							// doubled quote -> literal quote
							chars.next();
							field.push('"');
						} else {
							// closing quote
							in_quotes = false;
						}
					} else {
						// opening quote
						in_quotes = true;
					}
				}
				',' if !in_quotes => {
					// Per RFC4180, spaces outside quotes are part of the field.
					row.push(field.clone());
					field.clear();
				}
				'\n' if !in_quotes => {
					row.push(field.clone());
					field.clear();
					rows.push(row);
					row = Vec::new();
				}
				'\r' => {
					// ignore CR, handle CRLF by letting LF end the record
				}
				c => field.push(c),
			}
		}

		// push remaining data
		if in_quotes {
			// unterminated quoted field: treat as-is
			row.push(field.clone());
			rows.push(row);
		} else if !field.is_empty() || !row.is_empty() {
			row.push(field.clone());
			rows.push(row);
		}

		rows
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::normalization;

	#[test]
	fn csv_adapter_parses_and_normalizes() {
		let csv = "Name,Email\n Alice , ALICE@Example.COM \nBob, bob@EX.com\n";
		let adapter = CsvAdapter::new();
		let parsed = adapter.parse(csv);
		assert_eq!(parsed.len(), 3);

		// normalize rows
		let n0 = normalization::normalize_row(&parsed[0]);
		assert_eq!(n0, vec!["name".to_string(), "email".to_string()]);

		let n1 = normalization::normalize_row(&parsed[1]);
		assert_eq!(
			n1,
			vec!["alice".to_string(), "alice@example.com".to_string()]
		);
	}

	#[test]
	fn csv_adapter_handles_escaped_quotes_and_multiline() {
		let csv = r#"a,b,c
"multi
line",d,"with ""quote"""
"#;
		let adapter = CsvAdapter::new();
		let parsed = adapter.parse(csv);
		assert_eq!(parsed.len(), 2);
		assert_eq!(parsed[1][0], "multi\nline");
		assert_eq!(parsed[1][2], "with \"quote\"");
	}
}
