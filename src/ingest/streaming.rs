//! Streaming file handling for arbitrarily large files without loading into memory.
//!
//! This module implements streaming CSV/JSON parsing with async I/O to handle
//! files up to OS limits (GB/TB scale) without loading the entire file into memory.
//! Only one instance writes a file at a time (enforced at application level).

use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Result of streaming parse operation
#[derive(Debug, Clone)]
pub struct StreamStats {
	/// Total rows successfully processed
	pub rows_processed: u64,
	/// Total rows that failed to parse
	pub rows_failed: u64,
	/// Total bytes read from file
	pub bytes_read: u64,
	/// Any warnings/errors encountered during parsing
	pub warnings: Vec<String>,
}

/// Streaming CSV parser that processes one row at a time
pub struct StreamingCsvParser {
	reader: BufReader<File>,
	stats: StreamStats,
	line_buffer: String,
}

impl StreamingCsvParser {
	/// Create a new streaming CSV parser from a file path
	pub async fn new(path: &Path) -> std::io::Result<Self> {
		let file = File::open(path).await?;
		let reader = BufReader::new(file);

		Ok(StreamingCsvParser {
			reader,
			stats: StreamStats {
				rows_processed: 0,
				rows_failed: 0,
				bytes_read: 0,
				warnings: Vec::new(),
			},
			line_buffer: String::with_capacity(4096),
		})
	}

	/// Read and parse the next CSV row from the file
	/// Returns None when EOF is reached
	pub async fn next_row(&mut self) -> std::io::Result<Option<Vec<String>>> {
		loop {
			self.line_buffer.clear();

			let bytes_read = self.reader.read_line(&mut self.line_buffer).await?;
			if bytes_read == 0 {
				// EOF reached
				return Ok(None);
			}

			self.stats.bytes_read += bytes_read as u64;

			// Parse CSV line (simple implementation)
			let line = self.line_buffer.trim_end();
			if line.is_empty() {
				// Skip empty lines
				continue;
			}

			match parse_csv_line(line) {
				Ok(fields) => {
					self.stats.rows_processed += 1;
					return Ok(Some(fields));
				}
				Err(e) => {
					self.stats.rows_failed += 1;
					self.stats.warnings.push(format!(
						"Failed to parse CSV line {}: {}",
						self.stats.rows_processed + self.stats.rows_failed,
						e
					));
					// Continue to next row on error
					continue;
				}
			}
		}
	}

	/// Get current parsing statistics
	pub fn stats(&self) -> &StreamStats {
		&self.stats
	}

	/// Consume parser and return final statistics
	pub fn into_stats(self) -> StreamStats {
		self.stats
	}
}

/// Streaming JSON Lines parser (one JSON object per line)
pub struct StreamingJsonLinesParser {
	reader: BufReader<File>,
	stats: StreamStats,
	line_buffer: String,
}

impl StreamingJsonLinesParser {
	/// Create a new streaming JSON Lines parser from a file path
	pub async fn new(path: &Path) -> std::io::Result<Self> {
		let file = File::open(path).await?;
		let reader = BufReader::new(file);

		Ok(StreamingJsonLinesParser {
			reader,
			stats: StreamStats {
				rows_processed: 0,
				rows_failed: 0,
				bytes_read: 0,
				warnings: Vec::new(),
			},
			line_buffer: String::with_capacity(4096),
		})
	}

	/// Read and parse the next JSON Line from the file
	/// Returns None when EOF is reached
	/// Each line should contain a complete JSON object
	pub async fn next_row(&mut self) -> std::io::Result<Option<Vec<String>>> {
		loop {
			self.line_buffer.clear();

			let bytes_read = self.reader.read_line(&mut self.line_buffer).await?;
			if bytes_read == 0 {
				// EOF reached
				return Ok(None);
			}

			self.stats.bytes_read += bytes_read as u64;

			let line = self.line_buffer.trim();
			if line.is_empty() {
				// Skip empty lines
				continue;
			}

			match serde_json::from_str::<serde_json::Value>(line) {
				Ok(value) => {
					let fields = match value {
						serde_json::Value::Object(obj) => {
							// Convert JSON object to flat string array
							obj.iter()
								.map(|(k, v)| format!("{}:{}", k, v))
								.collect()
						}
						serde_json::Value::Array(arr) => {
							// Convert JSON array to string array
							arr.iter().map(|v| v.to_string()).collect()
						}
						_ => {
							self.stats.rows_failed += 1;
							self.stats.warnings.push(format!(
								"Line {} is not JSON object or array",
								self.stats.rows_processed + self.stats.rows_failed
							));
							continue;
						}
					};
					self.stats.rows_processed += 1;
					return Ok(Some(fields));
				}
				Err(e) => {
					self.stats.rows_failed += 1;
					self.stats.warnings.push(format!(
						"Failed to parse JSON on line {}: {}",
						self.stats.rows_processed + self.stats.rows_failed,
						e
					));
					// Continue to next row on error
					continue;
				}
			}
		}
	}

	/// Get current parsing statistics
	pub fn stats(&self) -> &StreamStats {
		&self.stats
	}

	/// Consume parser and return final statistics
	pub fn into_stats(self) -> StreamStats {
		self.stats
	}
}

/// Streaming JSON array parser (for compact JSON arrays)
pub struct StreamingJsonArrayParser {
	reader: BufReader<File>,
	stats: StreamStats,
	buffer: String,
	in_array: bool,
	brace_depth: i32,
}

impl StreamingJsonArrayParser {
	/// Create a new streaming JSON array parser from a file path
	pub async fn new(path: &Path) -> std::io::Result<Self> {
		let file = File::open(path).await?;
		let reader = BufReader::new(file);

		Ok(StreamingJsonArrayParser {
			reader,
			stats: StreamStats {
				rows_processed: 0,
				rows_failed: 0,
				bytes_read: 0,
				warnings: Vec::new(),
			},
			buffer: String::with_capacity(8192),
			in_array: false,
			brace_depth: 0,
		})
	}

	/// Read and parse the next JSON object from the array
	/// Returns None when EOF or end of array is reached
	pub async fn next_row(&mut self) -> std::io::Result<Option<Vec<String>>> {
		loop {
			let mut line = String::new();

			// Read until we find a complete JSON object
			let bytes_read = self.reader.read_line(&mut line).await?;
			if bytes_read == 0 && self.buffer.is_empty() {
				return Ok(None);
			}

			self.stats.bytes_read += bytes_read as u64;

			// Build up buffer to find complete JSON objects
			for ch in line.chars() {
				match ch {
					'[' if !self.in_array => {
						self.in_array = true;
					}
					']' if self.in_array && self.brace_depth == 0 => {
						// End of array
						if !self.buffer.is_empty()
							&& let Some(result) = self.try_parse_buffer() {
								return result;
							}
						return Ok(None);
					}
					'{' => {
						self.brace_depth += 1;
						self.buffer.push(ch);
					}
					'}' => {
						self.buffer.push(ch);
						self.brace_depth -= 1;
						if self.brace_depth == 0 && !self.buffer.is_empty()
							&& let Some(result) = self.try_parse_buffer() {
								return result;
							}
					}
					',' if self.brace_depth == 0 && !self.buffer.is_empty() => {
						// End of current object
						if let Some(result) = self.try_parse_buffer() {
							return result;
						}
					}
					ch if self.brace_depth > 0 || (self.in_array && ch != '[' && ch != ']') => {
						if !ch.is_whitespace() || self.brace_depth > 0 {
							self.buffer.push(ch);
						}
					}
					_ => {}
				}
			}

			if bytes_read == 0 && !self.buffer.is_empty()
				&& let Some(result) = self.try_parse_buffer() {
					return result;
				}
		}
	}

	/// Try to parse the buffered JSON object, returning None if parsing fails
	/// (to continue to next object) or Some(result) if successful/complete
	fn try_parse_buffer(&mut self) -> Option<std::io::Result<Option<Vec<String>>>> {
		match serde_json::from_str::<serde_json::Value>(&self.buffer) {
			Ok(value) => {
				let fields = match value {
					serde_json::Value::Object(obj) => obj
						.iter()
						.map(|(k, v)| format!("{}:{}", k, v))
						.collect(),
					serde_json::Value::Array(arr) => arr.iter().map(|v| v.to_string()).collect(),
					_ => vec![value.to_string()],
				};
				self.buffer.clear();
				self.stats.rows_processed += 1;
				Some(Ok(Some(fields)))
			}
			Err(e) => {
				self.buffer.clear();
				self.stats.rows_failed += 1;
				self.stats
					.warnings
					.push(format!("Failed to parse JSON object: {}", e));
				// Return None to continue parsing
				None
			}
		}
	}

	/// Get current parsing statistics
	pub fn stats(&self) -> &StreamStats {
		&self.stats
	}

	/// Consume parser and return final statistics
	pub fn into_stats(self) -> StreamStats {
		self.stats
	}
}

/// Parse a single CSV line with proper quote handling
///
/// Handles:
/// - Simple comma-separated values
/// - Quoted fields containing commas or quotes
/// - Escaped quotes within quoted fields ("" -> ")
fn parse_csv_line(line: &str) -> Result<Vec<String>, String> {
	let mut fields = Vec::new();
	let mut current_field = String::new();
	let mut in_quotes = false;
	let mut chars = line.chars().peekable();

	while let Some(ch) = chars.next() {
		match ch {
			'"' => {
				if in_quotes {
					// Check for escaped quote ("")
					if chars.peek() == Some(&'"') {
						current_field.push('"');
						chars.next();
					} else {
						in_quotes = false;
					}
				} else if current_field.is_empty() {
					in_quotes = true;
				} else {
					return Err("Quote encountered mid-field".to_string());
				}
			}
			',' if !in_quotes => {
				fields.push(current_field.clone());
				current_field.clear();
			}
			_ => {
				current_field.push(ch);
			}
		}
	}

	if in_quotes {
		return Err("Unterminated quoted field".to_string());
	}

	fields.push(current_field);
	Ok(fields)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_simple_csv_line() {
		let line = "field1,field2,field3";
		let result = parse_csv_line(line).unwrap();
		assert_eq!(result, vec!["field1", "field2", "field3"]);
	}

	#[test]
	fn test_parse_csv_with_quotes() {
		let line = r#""quoted field","normal field","field with, comma""#;
		let result = parse_csv_line(line).unwrap();
		assert_eq!(result[0], "quoted field");
		assert_eq!(result[1], "normal field");
		assert_eq!(result[2], "field with, comma");
	}

	#[test]
	fn test_parse_csv_with_escaped_quotes() {
		let line = r#""field with ""escaped"" quotes",normal"#;
		let result = parse_csv_line(line).unwrap();
		assert_eq!(result[0], "field with \"escaped\" quotes");
		assert_eq!(result[1], "normal");
	}

	#[test]
	fn test_parse_csv_empty_field() {
		let line = "field1,,field3";
		let result = parse_csv_line(line).unwrap();
		assert_eq!(result, vec!["field1", "", "field3"]);
	}

	#[test]
	fn test_parse_csv_error_unterminated_quote() {
		let line = r#""unterminated quote,field2"#;
		let result = parse_csv_line(line);
		assert!(result.is_err());
	}

	#[test]
	fn test_parse_csv_error_mid_field_quote() {
		let line = r#"field"with"quote,field2"#;
		let result = parse_csv_line(line);
		assert!(result.is_err());
	}
}
