//! Minimal CSV adapter example.
//!
//! This crate provides a tiny `parse_csv` function suitable for unit tests
//! and examples. It is intentionally small and dependency-free.

pub fn parse_csv(input: &str) -> Vec<Vec<String>> {
	// RFC4180-like parser: supports quoted fields, doubled quotes, and
	// newlines inside quoted fields.
	let mut rows: Vec<Vec<String>> = Vec::new();
	let mut row: Vec<String> = Vec::new();
	let mut field = String::new();
	let mut in_quotes = false;
	let mut chars = input.chars().peekable();

	while let Some(ch) = chars.next() {
		match ch {
			'"' => {
				if in_quotes {
					if let Some('"') = chars.peek() {
						// escaped quote
						chars.next();
						field.push('"');
					} else {
						in_quotes = false;
					}
				} else {
					in_quotes = true;
				}
			}
			',' if !in_quotes => {
				// Preserve spaces outside quotes per RFC4180
				row.push(field.clone());
				field.clear();
			}
			'\n' if !in_quotes => {
				row.push(field.clone());
				field.clear();
				rows.push(row);
				row = Vec::new();
			}
			'\r' => {}
			c => field.push(c),
		}
	}

	if in_quotes {
		// unterminated quoted field: push what we have
		row.push(field.clone());
		rows.push(row);
	} else if !field.is_empty() || !row.is_empty() {
		row.push(field.clone());
		rows.push(row);
	}

	rows
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_simple_csv() {
		let input = "a,b,c\n1,2,3\n\"x, y\",z,\"q\"\n";
		let out = parse_csv(input);
		assert_eq!(out.len(), 3);
		assert_eq!(out[0], vec!["a", "b", "c"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
		assert_eq!(out[1], vec!["1", "2", "3"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
		assert_eq!(out[2][0], "x, y");
		assert_eq!(out[2][1], "z");
		assert_eq!(out[2][2], "q");
	}

	#[test]
	fn handles_empty_last_line() {
		let input = "a,b\n1,2\n";
		let out = parse_csv(input);
		assert_eq!(out.len(), 2);
		assert_eq!(out[0][0], "a");
	}

	#[test]
	fn handles_escaped_and_multiline() {
		let input = r#"h1,h2
"multi
line",value
"with ""quotes""",x
"#;
		let out = parse_csv(input);
		assert_eq!(out.len(), 3);
		assert_eq!(out[1][0], "multi\nline");
		assert_eq!(out[2][0], "with \"quotes\"");
	}
}
