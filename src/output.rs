//! Output formatters for Dumptruck analysis results.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

/// Result of a data ingestion operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResult {
	/// Total rows processed
	pub rows_processed: usize,
	/// Unique addresses identified
	pub unique_addresses: usize,
	/// Hashed credentials detected
	pub hashed_credentials_detected: usize,
	/// Weak passwords found
	pub weak_passwords_found: usize,
	/// Addresses with breach data
	pub breached_addresses: usize,
	/// Summary metadata events
	pub metadata: Vec<String>,
	/// Processing errors encountered
	pub errors: Vec<String>,
}

/// Output format trait for extensibility
pub trait OutputFormatter: Send + Sync {
	fn format(&self, result: &IngestResult) -> Result<String, Box<dyn std::error::Error>>;
}

/// JSON output formatter
pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
	fn format(&self, result: &IngestResult) -> Result<String, Box<dyn std::error::Error>> {
		Ok(serde_json::to_string_pretty(result)?)
	}
}

/// CSV output formatter
pub struct CsvFormatter;

impl OutputFormatter for CsvFormatter {
	fn format(&self, result: &IngestResult) -> Result<String, Box<dyn std::error::Error>> {
		let mut output = String::new();
		output.push_str("metric,value\n");
		output.push_str(&format!("rows_processed,{}\n", result.rows_processed));
		output.push_str(&format!("unique_addresses,{}\n", result.unique_addresses));
		output.push_str(&format!(
			"hashed_credentials_detected,{}\n",
			result.hashed_credentials_detected
		));
		output.push_str(&format!(
			"weak_passwords_found,{}\n",
			result.weak_passwords_found
		));
		output.push_str(&format!(
			"breached_addresses,{}\n",
			result.breached_addresses
		));
		Ok(output)
	}
}

/// Human-readable text output formatter
pub struct TextFormatter;

impl OutputFormatter for TextFormatter {
	fn format(&self, result: &IngestResult) -> Result<String, Box<dyn std::error::Error>> {
		let mut output = String::new();
		output.push_str("=== Dumptruck Analysis Results ===\n\n");
		output.push_str(&format!("Rows Processed: {}\n", result.rows_processed));
		output.push_str(&format!("Unique Addresses: {}\n", result.unique_addresses));
		output.push_str(&format!(
			"Hashed Credentials Detected: {}\n",
			result.hashed_credentials_detected
		));
		output.push_str(&format!(
			"Weak Passwords Found: {}\n",
			result.weak_passwords_found
		));
		output.push_str(&format!(
			"Breached Addresses: {}\n",
			result.breached_addresses
		));

		if !result.metadata.is_empty() {
			output.push_str("\nMetadata Events:\n");
			for event in &result.metadata {
				output.push_str(&format!("  - {}\n", event));
			}
		}

		if !result.errors.is_empty() {
			output.push_str("\nErrors:\n");
			for error in &result.errors {
				output.push_str(&format!("  - {}\n", error));
			}
		}

		Ok(output)
	}
}

/// JSONL (newline-delimited JSON) output formatter
pub struct JsonlFormatter;

impl OutputFormatter for JsonlFormatter {
	fn format(&self, result: &IngestResult) -> Result<String, Box<dyn std::error::Error>> {
		let mut output = String::new();
		let summary = serde_json::json!({
			"event": "summary",
			"rows_processed": result.rows_processed,
			"unique_addresses": result.unique_addresses,
			"hashed_credentials_detected": result.hashed_credentials_detected,
			"weak_passwords_found": result.weak_passwords_found,
			"breached_addresses": result.breached_addresses,
		});
		output.push_str(&summary.to_string());
		output.push('\n');

		for event in &result.metadata {
			let meta = serde_json::json!({
				"event": "metadata",
				"message": event,
			});
			output.push_str(&meta.to_string());
			output.push('\n');
		}

		for error in &result.errors {
			let err = serde_json::json!({
				"event": "error",
				"message": error,
			});
			output.push_str(&err.to_string());
			output.push('\n');
		}

		Ok(output)
	}
}

/// Write output to file or stdout
pub fn write_output(
	content: &str,
	output_path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
	if let Some(path) = output_path {
		let file = File::create(path)?;
		let mut writer = BufWriter::new(file);
		writer.write_all(content.as_bytes())?;
		writer.flush()?;
	} else {
		// Write to stdout
		io::stdout().write_all(content.as_bytes())?;
		io::stdout().flush()?;
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_json_formatter() {
		let result = IngestResult {
			rows_processed: 100,
			unique_addresses: 50,
			hashed_credentials_detected: 10,
			weak_passwords_found: 5,
			breached_addresses: 15,
			metadata: vec!["test".to_string()],
			errors: vec![],
		};

		let formatter = JsonFormatter;
		let output = formatter.format(&result).expect("formatting failed");
		assert!(output.contains("rows_processed"));
		assert!(output.contains("100"));
	}

	#[test]
	fn test_text_formatter() {
		let result = IngestResult {
			rows_processed: 100,
			unique_addresses: 50,
			hashed_credentials_detected: 10,
			weak_passwords_found: 5,
			breached_addresses: 15,
			metadata: vec![],
			errors: vec!["test error".to_string()],
		};

		let formatter = TextFormatter;
		let output = formatter.format(&result).expect("formatting failed");
		assert!(output.contains("Dumptruck Analysis Results"));
		assert!(output.contains("Rows Processed: 100"));
	}
}
