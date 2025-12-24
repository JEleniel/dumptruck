//! Output formatters for Dumptruck analysis results.

use std::{
	fs::File,
	io::{self, BufWriter, Write},
	path::Path,
};

use serde::{Deserialize, Serialize};

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
	/// PII/NPI detection summary
	#[serde(default)]
	pub pii_summary: Option<PiiDetectionSummary>,
	/// Grouped detection findings (detection_type → [{field, rows}])
	#[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
	pub detection_groups: std::collections::BTreeMap<String, Vec<DetectionFieldGroup>>,
	/// Summary metadata events
	pub metadata: Vec<String>,
	/// Processing errors encountered
	pub errors: Vec<String>,
}

/// A single detected PII/NPI value (for internal use during processing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
	/// Column name (if available)
	pub column: Option<String>,
	/// Type of detection
	pub detection_type: String,
}

/// Detection grouped by field with row numbers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionFieldGroup {
	/// Field/column name where detection occurred
	pub field: String,
	/// Row numbers where this field had detections
	pub rows: Vec<usize>,
}

/// Summary of PII/NPI detections found
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PiiDetectionSummary {
	/// Count of rows with email addresses detected
	pub emails: usize,
	/// Count of rows with phone numbers detected
	pub phone_numbers: usize,
	/// Count of rows with IP addresses detected
	pub ip_addresses: usize,
	/// Count of rows with SSNs detected
	pub social_security_numbers: usize,
	/// Count of rows with national IDs detected
	pub national_ids: usize,
	/// Count of rows with credit cards detected
	pub credit_cards: usize,
	/// Count of rows with names detected
	pub names: usize,
	/// Count of rows with addresses detected
	pub mailing_addresses: usize,
	/// Count of rows with bank identifiers detected
	pub bank_identifiers: usize,
	/// Count of rows with crypto addresses detected
	pub crypto_addresses: usize,
	/// Count of rows with digital wallet tokens detected
	pub digital_wallets: usize,
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

/// Format a list of row numbers compactly (e.g., "1-5, 7, 10-15")
fn format_row_list(rows: &[usize]) -> String {
	if rows.is_empty() {
		return String::new();
	}

	let mut result = Vec::new();
	let mut start = rows[0];
	let mut end = rows[0];

	for &row in &rows[1..] {
		if row == end + 1 {
			end = row;
		} else {
			if start == end {
				result.push(start.to_string());
			} else {
				result.push(format!("{}-{}", start, end));
			}
			start = row;
			end = row;
		}
	}

	// Handle the last range
	if start == end {
		result.push(start.to_string());
	} else {
		result.push(format!("{}-{}", start, end));
	}

	result.join(", ")
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

		if let Some(pii) = &result.pii_summary {
			output.push_str("\nPII/NPI Detection Summary:\n");
			if pii.emails > 0 {
				output.push_str(&format!("  - Emails: {}\n", pii.emails));
			}
			if pii.phone_numbers > 0 {
				output.push_str(&format!("  - Phone Numbers: {}\n", pii.phone_numbers));
			}
			if pii.ip_addresses > 0 {
				output.push_str(&format!("  - IP Addresses: {}\n", pii.ip_addresses));
			}
			if pii.social_security_numbers > 0 {
				output.push_str(&format!(
					"  - Social Security Numbers: {}\n",
					pii.social_security_numbers
				));
			}
			if pii.national_ids > 0 {
				output.push_str(&format!("  - National IDs: {}\n", pii.national_ids));
			}
			if pii.credit_cards > 0 {
				output.push_str(&format!("  - Credit Cards: {}\n", pii.credit_cards));
			}
			if pii.names > 0 {
				output.push_str(&format!("  - Names: {}\n", pii.names));
			}
			if pii.mailing_addresses > 0 {
				output.push_str(&format!(
					"  - Mailing Addresses: {}\n",
					pii.mailing_addresses
				));
			}
			if pii.bank_identifiers > 0 {
				output.push_str(&format!("  - Bank Identifiers: {}\n", pii.bank_identifiers));
			}
			if pii.crypto_addresses > 0 {
				output.push_str(&format!("  - Crypto Addresses: {}\n", pii.crypto_addresses));
			}
			if pii.digital_wallets > 0 {
				output.push_str(&format!("  - Digital Wallets: {}\n", pii.digital_wallets));
			}
		}

		// Display detection findings grouped by type and field
		if !result.detection_groups.is_empty() {
			output.push_str("\n=== Detections Found ===\n\n");
			for (detection_type, field_groups) in &result.detection_groups {
				output.push_str(&format!("{}:\n", detection_type));
				for field_group in field_groups {
					output.push_str(&format!(
						"  Field '{}': rows {}\n",
						field_group.field,
						format_row_list(&field_group.rows)
					));
				}
				output.push('\n');
			}
		}

		if !result.metadata.is_empty() {
			output.push_str("Metadata Events:\n");
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
			pii_summary: None,
			detection_groups: std::collections::BTreeMap::new(),
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
			pii_summary: None,
			detection_groups: std::collections::BTreeMap::new(),
			metadata: vec![],
			errors: vec!["test error".to_string()],
		};

		let formatter = TextFormatter;
		let output = formatter.format(&result).expect("formatting failed");
		assert!(output.contains("Dumptruck Analysis Results"));
		assert!(output.contains("Rows Processed: 100"));
	}
}
