//! Format processors for various data formats.
//!
//! Handles parsing and processing of CSV, TSV, JSON, and XML formats
//! through the detection pipeline.

use std::path::Path;

use crate::{analyze::adapters::FormatAdapter, cli::AnalyzeArgs, detection};

use super::stats::IngestStats;

/// Process file based on detected format
pub async fn process_format(
	format_str: &str,
	content: &str,
	file_path: &Path,
	args: &AnalyzeArgs,
	stats: &mut IngestStats,
) {
	match format_str {
		"csv" => process_csv(content, file_path, args, stats),
		"tsv" => process_tsv(content, file_path, args, stats),
		"json" => process_json(content, file_path, args, stats),
		"xml" => process_xml(content, file_path, args, stats).await,
		_ => {
			let err_msg = format!("Unsupported format: {}", format_str);
			if args.verbose >= 1 {
				eprintln!("[ERROR] {}", err_msg);
			}
			stats.errors.push(err_msg);
		}
	}
}

/// Process CSV format
fn process_csv(content: &str, file_path: &Path, args: &AnalyzeArgs, stats: &mut IngestStats) {
	let adapter = crate::analyze::adapters::CsvAdapter::new();
	let rows = FormatAdapter::parse(&adapter, content);

	if args.verbose >= 1 {
		eprintln!("[INFO] CSV parsing complete: {} rows parsed", rows.len());
	}

	let headers = extract_headers(&rows, args);
	process_rows(&rows, &headers, file_path, args, stats);
}

/// Process TSV format
fn process_tsv(content: &str, file_path: &Path, args: &AnalyzeArgs, stats: &mut IngestStats) {
	if args.verbose >= 2 {
		eprintln!("[DEBUG] Starting TSV parsing...");
	}

	let rows: Vec<Vec<String>> = content
		.lines()
		.map(|line| line.split('\t').map(|s| s.to_string()).collect())
		.collect();

	if args.verbose >= 1 {
		eprintln!("[INFO] TSV parsing complete: {} rows parsed", rows.len());
	}

	let headers = extract_headers(&rows, args);
	process_rows(&rows, &headers, file_path, args, stats);
}

/// Process JSON format
fn process_json(content: &str, file_path: &Path, args: &AnalyzeArgs, stats: &mut IngestStats) {
	if args.verbose >= 2 {
		eprintln!("[DEBUG] Starting JSON parsing with universal parser...");
	}

	match serde_json::from_str::<serde_json::Value>(content) {
		Ok(json_value) => {
			let rows = crate::analyze::universal_parser::json_to_rows(&json_value);

			if rows.is_empty() {
				let err_msg = format!("No data rows found in JSON file {:?}", file_path);
				if args.verbose >= 1 {
					eprintln!("[ERROR] {}", err_msg);
				}
				stats.errors.push(err_msg);
				return;
			}

			if args.verbose >= 1 {
				eprintln!("[INFO] JSON parsing complete: {} rows parsed", rows.len());
			}

			let headers = extract_headers(&rows, args);
			process_rows(&rows, &headers, file_path, args, stats);
		}
		Err(e) => {
			let err_msg = format!("Failed to parse JSON from {:?}: {}", file_path, e);
			if args.verbose >= 1 {
				eprintln!("[ERROR] {}", err_msg);
			}
			stats.errors.push(err_msg);
		}
	}
}

/// Process XML format
async fn process_xml(content: &str, file_path: &Path, args: &AnalyzeArgs, stats: &mut IngestStats) {
	if args.verbose >= 2 {
		eprintln!("[DEBUG] Starting XML parsing with universal parser...");
	}

	match crate::analyze::universal_parser::xml_to_rows(content) {
		Ok(rows) => {
			if rows.is_empty() {
				let err_msg = format!("No data rows found in XML file {:?}", file_path);
				if args.verbose >= 1 {
					eprintln!("[ERROR] {}", err_msg);
				}
				stats.errors.push(err_msg);
				return;
			}

			if args.verbose >= 1 {
				eprintln!("[INFO] XML parsing complete: {} rows parsed", rows.len());
			}

			let headers = extract_headers(&rows, args);
			process_rows(&rows, &headers, file_path, args, stats);
		}
		Err(e) => {
			let err_msg = format!("Failed to parse XML structure from {:?}: {}", file_path, e);
			if args.verbose >= 1 {
				eprintln!("[ERROR] {}", err_msg);
			}
			stats.errors.push(err_msg);
		}
	}
}

/// Extract headers from rows if present
fn extract_headers(rows: &[Vec<String>], args: &AnalyzeArgs) -> Option<Vec<String>> {
	if rows.is_empty() {
		return None;
	}

	let first = &rows[0];
	if first.iter().any(|c| c.chars().any(|ch| ch.is_alphabetic())) {
		if args.verbose >= 2 {
			eprintln!("[DEBUG] First row detected as header");
		}
		Some(first.clone())
	} else {
		None
	}
}

/// Process rows through detection pipeline
fn process_rows(
	rows: &[Vec<String>],
	headers: &Option<Vec<String>>,
	file_path: &Path,
	args: &AnalyzeArgs,
	stats: &mut IngestStats,
) {
	if args.verbose >= 2 {
		eprintln!("[DEBUG] Starting detection pipeline on {} rows", rows.len());
	}

	let mut detections = Vec::new();
	let row_start_idx = if headers.is_some() { 1 } else { 0 };

	for (idx, row) in rows.iter().enumerate() {
		if idx == 0 && headers.is_some() {
			continue;
		}
		let detection = detection::analyzer::detect_row(row, headers.as_deref(), idx);
		detections.push(detection);
	}

	if args.verbose >= 2 {
		eprintln!(
			"[DEBUG] Detection pipeline complete on {} rows",
			detections.len()
		);
	}

	let detection_stats = detection::analyzer::aggregate_results(&detections);

	// Track PII detections and group by detection type and column
	let mut rows_with_pii = vec![false; detections.len()];
	for (idx, detection) in detections.iter().enumerate() {
		let row_number = row_start_idx + idx; // User-friendly 1-based row number

		// Group detections by type and field
		if !detection.pii_findings.is_empty() {
			for finding in &detection.pii_findings {
				let detection_type = finding.pii_type.to_string();
				let field_name = finding
					.column_name
					.clone()
					.unwrap_or_else(|| "(unknown)".to_string());

				// Add row number to the grouped structure
				let field_groups = stats.detection_groups.entry(detection_type).or_default();

				// Find or create the field group
				if let Some(field_group) = field_groups.iter_mut().find(|fg| fg.field == field_name)
				{
					field_group.rows.push(row_number);
				} else {
					field_groups.push(crate::api::output::DetectionFieldGroup {
						field: field_name,
						rows: vec![row_number],
					});
				}

				// Update summary counts
				match &finding.pii_type {
					detection::npi_detection::PiiType::Email => {
						stats.pii_summary.emails = stats.pii_summary.emails.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					detection::npi_detection::PiiType::PhoneNumber => {
						stats.pii_summary.phone_numbers =
							stats.pii_summary.phone_numbers.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					detection::npi_detection::PiiType::IpAddress
					| detection::npi_detection::PiiType::IpV4Address
					| detection::npi_detection::PiiType::IpV6Address => {
						stats.pii_summary.ip_addresses =
							stats.pii_summary.ip_addresses.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					detection::npi_detection::PiiType::SocialSecurityNumber => {
						stats.pii_summary.social_security_numbers =
							stats.pii_summary.social_security_numbers.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					detection::npi_detection::PiiType::NationalId => {
						stats.pii_summary.national_ids =
							stats.pii_summary.national_ids.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					detection::npi_detection::PiiType::CreditCardNumber => {
						stats.pii_summary.credit_cards =
							stats.pii_summary.credit_cards.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					detection::npi_detection::PiiType::Name => {
						stats.pii_summary.names = stats.pii_summary.names.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					detection::npi_detection::PiiType::MailingAddress => {
						stats.pii_summary.mailing_addresses =
							stats.pii_summary.mailing_addresses.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					detection::npi_detection::PiiType::IBAN
					| detection::npi_detection::PiiType::SWIFTCode
					| detection::npi_detection::PiiType::RoutingNumber
					| detection::npi_detection::PiiType::BankAccount => {
						stats.pii_summary.bank_identifiers =
							stats.pii_summary.bank_identifiers.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					detection::npi_detection::PiiType::CryptoAddress => {
						stats.pii_summary.crypto_addresses =
							stats.pii_summary.crypto_addresses.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					detection::npi_detection::PiiType::DigitalWalletToken => {
						stats.pii_summary.digital_wallets =
							stats.pii_summary.digital_wallets.saturating_add(1);
						rows_with_pii[idx] = true;
					}
					_ => {}
				}
			}
		}
	}

	let _ = rows_with_pii; // Silence unused warning if not used

	stats.total_rows += rows.len();
	stats.unique_addresses += detection_stats.unique_addresses;
	stats.hashed_credentials += detection_stats.hashed_credentials_detected;
	stats.weak_passwords += detection_stats.weak_passwords_found;

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Detection results for file:");
		eprintln!(
			"[DEBUG]   Unique addresses: {}",
			detection_stats.unique_addresses
		);
		eprintln!(
			"[DEBUG]   Hashed credentials: {}",
			detection_stats.hashed_credentials_detected
		);
		eprintln!(
			"[DEBUG]   Weak passwords: {}",
			detection_stats.weak_passwords_found
		);
		eprintln!(
			"[DEBUG]   Detection groups: {}",
			stats.detection_groups.len()
		);
	}

	stats.metadata.push(format!(
		"Processed {} rows from {} | Unique addresses: {}, Hashed credentials: {}, Weak \
		 passwords: {}",
		rows.len(),
		file_path.display(),
		detection_stats.unique_addresses,
		detection_stats.hashed_credentials_detected,
		detection_stats.weak_passwords_found
	));
}
