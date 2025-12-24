//! Command handlers for Dumptruck CLI.

use std::{path::Path, sync::Arc};

use crate::{
	api::output::{
		CsvFormatter, DetailedRowFinding, Detection, IngestResult, JsonFormatter, JsonlFormatter,
		OutputFormatter, PiiDetectionSummary, TextFormatter, write_output,
	},
	cli::{IngestArgs, OutputFormat, ServerArgs, StatusArgs},
	detection,
	ingest::adapters::FormatAdapter,
	storage::working_copy::WorkingCopyManager,
};

/// Configuration and manager setup for ingest operations
struct IngestContext {
	working_copy_mgr: WorkingCopyManager,
}

/// Statistics aggregated across files during ingest
#[derive(Default)]
struct IngestStats {
	total_rows: usize,
	unique_addresses: usize,
	hashed_credentials: usize,
	weak_passwords: usize,
	pii_summary: PiiDetectionSummary,
	detailed_findings: Vec<DetailedRowFinding>,
	metadata: Vec<String>,
	errors: Vec<String>,
}

/// Handle the ingest command
pub async fn ingest(args: IngestArgs) -> Result<(), String> {
	if args.verbose >= 1 {
		eprintln!("[INFO] Starting ingest operation");
	}

	let files = args.resolve_input_files()?;
	if args.verbose >= 1 {
		eprintln!("[INFO] Found {} file(s) to process", files.len());
	}

	let ctx = setup_ingest_context(&args)?;
	let mut stats = IngestStats::default();

	for file_path in &files {
		process_single_file(&ctx, file_path, &args, &mut stats).await;
	}

	finalize_ingest(&args, &stats).await
}

/// Set up ingest context (config and working directory)
fn setup_ingest_context(args: &IngestArgs) -> Result<IngestContext, String> {
	let config_path = args.config.as_ref().and_then(|p| p.to_str());
	let config = crate::core::config::Config::load_with_search(config_path, args.verbose >= 2)
		.map_err(|e| format!("Failed to load configuration: {}", e))?;

	let working_dir = if let Some(dir) = &args.working_dir {
		dir.clone()
	} else if let Some(config_path) = &config.working_directory.path {
		std::path::PathBuf::from(config_path)
	} else {
		std::path::PathBuf::from("/tmp/dumptruck")
	};

	let working_copy_mgr =
		WorkingCopyManager::new(&working_dir, args.verify_noexec, args.verbose as u32)
			.map_err(|e| format!("Failed to initialize working directory: {}", e))?;

	if args.verbose >= 1 {
		eprintln!("[INFO] Working directory initialized: {:?}", working_dir);
	}

	Ok(IngestContext { working_copy_mgr })
}

/// Process a single file through the ingest pipeline
async fn process_single_file(
	ctx: &IngestContext,
	file_path: &std::path::Path,
	args: &IngestArgs,
	stats: &mut IngestStats,
) {
	if args.verbose >= 1 {
		eprintln!("[INFO] Processing file: {:?}", file_path);
	}

	let working_copy_path = match ctx.working_copy_mgr.create_working_copy(file_path) {
		Ok(path) => path,
		Err(e) => {
			let err_msg = format!("Failed to create working copy for {:?}: {}", file_path, e);
			if args.verbose >= 1 {
				eprintln!("[ERROR] {}", err_msg);
			}
			stats.errors.push(err_msg);
			return;
		}
	};

	if args.verbose >= 2 {
		eprintln!(
			"[DEBUG] Reading file contents from working copy: {:?}",
			working_copy_path
		);
	}

	let result =
		crate::ingest::safe_ingest::safe_read_file(&working_copy_path, args.verbose as u32).await;

	let (content, safety_analysis) = match result {
		Ok((c, _had_utf8_errors, analysis)) => (c, analysis),
		Err(e) => {
			let err_msg = format!("Failed to read file {:?}: {}", working_copy_path, e);
			if args.verbose >= 1 {
				eprintln!("[ERROR] {}", err_msg);
			}
			stats.errors.push(err_msg);
			return;
		}
	};

	if args.verbose >= 2 {
		eprintln!("[DEBUG] File read complete, size: {} bytes", content.len());
	}

	if safety_analysis.is_binary {
		if args.verbose >= 1 {
			eprintln!(
				"[WARN] Binary file detected in {:?} ({:.0}% confidence) - skipping",
				working_copy_path, safety_analysis.binary_confidence
			);
		}
		stats.errors.push(format!(
			"Cannot process file {:?}: Binary file detected ({:.0}% confidence)",
			working_copy_path, safety_analysis.binary_confidence
		));
		return;
	}

	let format_str = if let Some(fmt) = args.format {
		fmt.to_string()
	} else {
		detect_format_from_path(file_path)
	};

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Detected format: {}", format_str);
		eprintln!("[INFO] Parsing {} format file...", format_str);
	}

	process_format(&format_str, &content, file_path, args, stats).await;
}

/// Process file based on detected format
async fn process_format(
	format_str: &str,
	content: &str,
	file_path: &std::path::Path,
	args: &IngestArgs,
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
fn process_csv(
	content: &str,
	file_path: &std::path::Path,
	args: &IngestArgs,
	stats: &mut IngestStats,
) {
	let adapter = crate::ingest::adapters::CsvAdapter::new();
	let rows = adapter.parse(content);

	if args.verbose >= 1 {
		eprintln!("[INFO] CSV parsing complete: {} rows parsed", rows.len());
	}

	let headers = extract_headers(&rows, args);
	process_rows(&rows, &headers, file_path, args, stats);
}

/// Process TSV format
fn process_tsv(
	content: &str,
	file_path: &std::path::Path,
	args: &IngestArgs,
	stats: &mut IngestStats,
) {
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
fn process_json(
	content: &str,
	file_path: &std::path::Path,
	args: &IngestArgs,
	stats: &mut IngestStats,
) {
	if args.verbose >= 2 {
		eprintln!("[DEBUG] Starting JSON parsing with universal parser...");
	}

	match serde_json::from_str::<serde_json::Value>(content) {
		Ok(json_value) => {
			let rows = crate::ingest::universal_parser::json_to_rows(&json_value);

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
async fn process_xml(
	content: &str,
	file_path: &std::path::Path,
	args: &IngestArgs,
	stats: &mut IngestStats,
) {
	if args.verbose >= 2 {
		eprintln!("[DEBUG] Starting XML parsing with universal parser...");
	}

	match crate::ingest::universal_parser::xml_to_rows(content) {
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
fn extract_headers(rows: &[Vec<String>], args: &IngestArgs) -> Option<Vec<String>> {
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
	file_path: &std::path::Path,
	args: &IngestArgs,
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

	// Track PII detections and capture detailed findings
	let mut rows_with_pii = vec![false; detections.len()];
	for (idx, detection) in detections.iter().enumerate() {
		let row_number = row_start_idx + idx; // User-friendly 1-based row number

		// Create detailed findings for this row if it has detections
		if !detection.pii_findings.is_empty() {
			let mut row_detections = Vec::new();
			for finding in &detection.pii_findings {
				row_detections.push(Detection {
					column: finding.column_name.clone(),
					value: finding.value.clone(),
					detection_type: finding.pii_type.to_string(),
				});

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

			stats.detailed_findings.push(DetailedRowFinding {
				row_number,
				detections: row_detections,
			});
		}
	}

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
			"[DEBUG]   Rows with detections: {}",
			stats.detailed_findings.len()
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

/// Finalize ingest and format output
async fn finalize_ingest(args: &IngestArgs, stats: &IngestStats) -> Result<(), String> {
	if args.verbose >= 1 {
		eprintln!(
			"[INFO] All files processed. Total rows: {}",
			stats.total_rows
		);
		eprintln!(
			"[INFO] Detection results: Unique addresses: {}, Hashed credentials: {}, Weak \
			 passwords: {}",
			stats.unique_addresses, stats.hashed_credentials, stats.weak_passwords
		);
	}

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Formatting output results...");
	}

	let result = IngestResult {
		rows_processed: stats.total_rows,
		unique_addresses: stats.unique_addresses,
		hashed_credentials_detected: stats.hashed_credentials,
		weak_passwords_found: stats.weak_passwords,
		breached_addresses: 0,
		pii_summary: Some(stats.pii_summary.clone()),
		detailed_findings: stats.detailed_findings.clone(),
		metadata: stats.metadata.clone(),
		errors: stats.errors.clone(),
	};

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Creating {:?} formatter...", args.output_format);
	}

	let formatter: Box<dyn OutputFormatter> = match args.output_format {
		OutputFormat::Json => Box::new(JsonFormatter),
		OutputFormat::Csv => Box::new(CsvFormatter),
		OutputFormat::Text => Box::new(TextFormatter),
		OutputFormat::Jsonl => Box::new(JsonlFormatter),
	};

	if args.verbose >= 2 {
		eprintln!(
			"[DEBUG] Formatting {} results to output format...",
			stats.total_rows
		);
	}

	let formatted = formatter
		.format(&result)
		.map_err(|e| format!("Failed to format output: {}", e))?;

	if args.verbose >= 2 {
		eprintln!(
			"[DEBUG] Output formatting complete: {} bytes",
			formatted.len()
		);
	}

	if args.verbose >= 1 {
		let target = if let Some(path) = args.output.as_deref() {
			format!("{:?}", path)
		} else {
			"stdout".to_string()
		};
		eprintln!("[INFO] Writing results to: {}", target);
	}

	let output_path = args.output.as_deref();
	write_output(&formatted, output_path).map_err(|e| format!("Failed to write output: {}", e))?;

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Output write complete");
	}

	if args.verbose >= 1 {
		eprintln!("[INFO] Ingest operation completed successfully");
	}

	Ok(())
}

/// Handle the status command
pub async fn status(args: StatusArgs) -> Result<(), String> {
	if args.verbose >= 1 {
		eprintln!("[INFO] Starting status checks");
	}

	let mut checks_passed = 0;
	let mut checks_failed = 0;

	// Check Ollama if requested
	if args.check_ollama {
		let ollama_url = args
			.ollama_url
			.clone()
			.unwrap_or_else(|| "http://localhost:11435".to_string());
		if args.verbose >= 2 {
			eprintln!("[DEBUG] Checking Ollama at: {}", ollama_url);
		}

		match check_ollama(&ollama_url).await {
			Ok(_) => {
				println!("✓ Ollama: OK");
				checks_passed += 1;
			}
			Err(e) => {
				println!("✗ Ollama: FAILED - {}", e);
				checks_failed += 1;
			}
		}
	}

	// Check database if requested
	if args.check_database {
		let db_url = args
			.database
			.clone()
			.unwrap_or_else(get_default_database_path);
		if args.verbose >= 2 {
			eprintln!("[DEBUG] Checking database at: {}", db_url);
		}

		match check_database(&db_url).await {
			Ok(_) => {
				println!("✓ Database: OK");
				checks_passed += 1;
			}
			Err(e) => {
				println!("✗ Database: FAILED - {}", e);
				checks_failed += 1;
			}
		}
	}

	// Check HIBP if requested
	if args.check_hibp {
		let api_key = args.hibp_key.clone();
		if args.verbose >= 2 {
			eprintln!("[DEBUG] Checking HIBP API");
		}

		match check_hibp(api_key.as_deref()).await {
			Ok(_) => {
				println!("✓ HIBP: OK");
				checks_passed += 1;
			}
			Err(e) => {
				println!("✗ HIBP: FAILED - {}", e);
				checks_failed += 1;
			}
		}
	}

	// If no specific checks were requested, run all
	if !args.check_ollama && !args.check_database && !args.check_hibp {
		println!("No checks specified. Use --check-ollama, --check-database, or --check-hibp");
		return Ok(());
	}

	if args.verbose >= 1 {
		eprintln!(
			"[INFO] Status checks completed: {} passed, {} failed",
			checks_passed, checks_failed
		);
	}

	if checks_failed > 0 {
		Err(format!("{} check(s) failed", checks_failed))
	} else {
		Ok(())
	}
}

/// Check Ollama connectivity
async fn check_ollama(url: &str) -> Result<(), String> {
	// Simple HTTP health check to Ollama's API endpoint
	let client = reqwest::Client::new();
	match client
		.get(format!("{}/api/tags", url))
		.timeout(std::time::Duration::from_secs(5))
		.send()
		.await
	{
		Ok(response) => {
			if response.status().is_success() {
				Ok(())
			} else {
				Err(format!("HTTP {}", response.status()))
			}
		}
		Err(e) => Err(format!("Connection failed: {}", e)),
	}
}

/// Check database connectivity
async fn check_database(db_path: &str) -> Result<(), String> {
	// Validate SQLite database file exists and is accessible
	let path = std::path::Path::new(db_path);
	if path.exists() {
		match std::fs::metadata(path) {
			Ok(metadata) if metadata.is_file() => Ok(()),
			Ok(_) => Err(format!("Database path is not a file: {}", db_path)),
			Err(e) => Err(format!("Cannot access database file: {}", e)),
		}
	} else {
		// SQLite will create the file if it doesn't exist, so this is acceptable
		Ok(())
	}
}

/// Check HIBP API connectivity
async fn check_hibp(_api_key: Option<&str>) -> Result<(), String> {
	// Simple connectivity check to HIBP API
	let client = reqwest::Client::new();
	match client
		.get("https://haveibeenpwned.com/api/v3/breaches")
		.timeout(std::time::Duration::from_secs(5))
		.send()
		.await
	{
		Ok(response) => {
			// HIBP returns 401 without API key, which is still a valid response
			if response.status().is_client_error() || response.status().is_success() {
				Ok(())
			} else {
				Err(format!("HTTP {}", response.status()))
			}
		}
		Err(e) => Err(format!("Connection failed: {}", e)),
	}
}

/// Detect file format from file extension
fn detect_format_from_path(path: &Path) -> String {
	path.extension()
		.and_then(|ext| ext.to_str())
		.map(|ext| ext.to_lowercase())
		.unwrap_or_else(|| "csv".to_string()) // default to CSV
}

/// Background job processor worker
async fn process_jobs(
	worker_id: usize,
	queue: Arc<crate::storage::job_queue::JobQueue>,
	verbose: u32,
	shutdown_rx: &mut tokio::sync::broadcast::Receiver<()>,
) {
	use std::time::Duration;

	use crate::storage::job_queue::JobStatus;

	loop {
		// Check for shutdown signal
		if shutdown_rx.try_recv().is_ok() {
			if verbose >= 2 {
				eprintln!("[DEBUG] Worker {} received shutdown signal", worker_id);
			}
			return;
		}

		// Check for jobs in the queue
		let (jobs, _total) = queue.list_jobs(0, 100).await;

		// Find first queued job
		let queued_job: Option<&crate::storage::job_queue::Job> =
			jobs.iter().find(|j| j.status == JobStatus::Queued);

		if let Some(job) = queued_job {
			let job_id = job.id.clone();
			let filename = job.filename.clone();

			if verbose >= 2 {
				eprintln!(
					"[DEBUG] Worker {} claiming job {} ({})",
					worker_id, job_id, filename
				);
			}

			// Mark job as processing
			if queue
				.update_job(&job_id, |j| {
					j.status = JobStatus::Processing;
					j.started_at = Some(chrono::Utc::now());
					Ok(())
				})
				.await
				.is_ok()
			{
				// Process the job
				process_single_job(&queue, &job_id, &filename, worker_id, verbose).await;
			}
		} else {
			// No jobs available, sleep briefly
			tokio::time::sleep(Duration::from_millis(100)).await;
		}
	}
}

/// Process a single job
async fn process_single_job(
	queue: &Arc<crate::storage::job_queue::JobQueue>,
	job_id: &str,
	_filename: &str,
	worker_id: usize,
	verbose: u32,
) {
	use crate::storage::job_queue::JobStatus;

	if verbose >= 2 {
		eprintln!("[DEBUG] Worker {} processing job {}", worker_id, job_id);
	}

	// Simulate processing with a small delay
	tokio::time::sleep(std::time::Duration::from_millis(100)).await;

	// Mark job as completed
	let result = queue
		.update_job(job_id, |j| {
			j.status = JobStatus::Completed;
			j.completed_at = Some(chrono::Utc::now());
			j.progress_percentage = 100;
			j.rows_processed = j.file_size_bytes as usize / 100; // Mock: estimate rows from file size
			Ok(())
		})
		.await;

	match result {
		Ok(_) => {
			if verbose >= 2 {
				eprintln!("[DEBUG] Worker {} completed job {}", worker_id, job_id);
			}
		}
		Err(e) => {
			if verbose >= 2 {
				eprintln!(
					"[DEBUG] Worker {} error updating job {}: {}",
					worker_id, job_id, e
				);
			}
			// Try to mark as failed
			let _ = queue
				.update_job(job_id, |j| {
					j.status = JobStatus::Failed;
					j.completed_at = Some(chrono::Utc::now());
					j.error_message = Some(format!("Processing error: {}", e));
					Ok(())
				})
				.await;
		}
	}
}

/// Set up signal handlers for graceful shutdown (SIGTERM, SIGINT)
fn setup_signal_handler(verbose: u32) -> Result<tokio::sync::broadcast::Sender<()>, String> {
	let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);
	let tx = shutdown_tx.clone();

	tokio::spawn(async move {
		let mut sigterm =
			match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
				Ok(sig) => sig,
				Err(e) => {
					if verbose >= 1 {
						eprintln!("[WARN] Failed to setup SIGTERM handler: {}", e);
					}
					return;
				}
			};

		let mut sigint =
			match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()) {
				Ok(sig) => sig,
				Err(e) => {
					if verbose >= 1 {
						eprintln!("[WARN] Failed to setup SIGINT handler: {}", e);
					}
					return;
				}
			};

		tokio::select! {
			_ = sigterm.recv() => {
				if verbose >= 1 {
					eprintln!("[INFO] SIGTERM received");
				}
				let _ = tx.send(());
			}
			_ = sigint.recv() => {
				if verbose >= 1 {
					eprintln!("[INFO] SIGINT received");
				}
				let _ = tx.send(());
			}
		}
	});

	Ok(shutdown_tx)
}

/// Handle the server command
pub async fn server(args: ServerArgs) -> Result<(), String> {
	use std::sync::Arc;

	use crate::{
		api::server::{AppState, create_app},
		deploy::ServiceManager,
		network::oauth::OAuthProvider,
		storage::job_queue::JobQueue,
	};

	// Install default crypto provider for rustls
	let _ = rustls::crypto::ring::default_provider().install_default();

	if args.verbose >= 1 {
		eprintln!("[INFO] Starting HTTPS server on port {}", args.port);
	}

	// Load configuration file
	let config =
		crate::core::config::Config::load_with_search(args.config.as_deref(), args.verbose >= 2)
			.map_err(|e| format!("Failed to load configuration: {}", e))?;

	// Get OAuth settings - CLI args override config file
	let oauth_client_id = args
		.oauth_client_id
		.unwrap_or_else(|| config.oauth.client_id.clone());
	let oauth_client_secret = args
		.oauth_client_secret
		.unwrap_or_else(|| config.oauth.client_secret.clone());
	let oauth_token_endpoint = args
		.oauth_token_endpoint
		.unwrap_or_else(|| config.oauth.discovery_url.clone());

	if oauth_client_id.is_empty()
		|| oauth_client_secret.is_empty()
		|| oauth_token_endpoint.is_empty()
	{
		return Err(
			"Missing OAuth configuration. Provide via config.json or command-line arguments."
				.to_string(),
		);
	}

	// Initialize OAuth provider
	let oauth = OAuthProvider::new(
		oauth_client_id,
		oauth_client_secret,
		oauth_token_endpoint,
		args.oauth_scope,
	);

	if args.verbose >= 2 {
		eprintln!("[DEBUG] OAuth 2.0 provider initialized");
	}

	// Create application state
	let job_queue = Arc::new(JobQueue::new());
	let state = Arc::new(AppState {
		job_queue: job_queue.clone(),
		oauth_provider: Arc::new(oauth),
	});

	// Create router with all endpoints
	let app = create_app(state.clone());

	// Create shutdown signal for worker tasks
	let (shutdown_workers_tx, _) = tokio::sync::broadcast::channel::<()>(1);

	// Spawn background job processor workers (parallel processing)
	let worker_count = std::thread::available_parallelism()
		.map(|n| n.get())
		.unwrap_or(4);
	if args.verbose >= 1 {
		eprintln!(
			"[INFO] Spawning {} worker threads for parallel job processing",
			worker_count
		);
	}

	for worker_id in 0..worker_count {
		let queue = job_queue.clone();
		let verbose = args.verbose as u32;
		let mut shutdown_rx = shutdown_workers_tx.subscribe();

		tokio::spawn(async move {
			process_jobs(worker_id, queue, verbose, &mut shutdown_rx).await;
		});
	}

	// Create socket address
	let addr = std::net::SocketAddr::from(([127, 0, 0, 1], args.port));
	if args.verbose >= 1 {
		eprintln!("[INFO] Listening on {}", addr);
	}

	// Load TLS certificates
	let cert_path = args.cert.unwrap_or_else(|| "/etc/tls/tls.crt".to_string());
	let key_path = args.key.unwrap_or_else(|| "/etc/tls/tls.key".to_string());

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Loading TLS certificates from {}", cert_path);
		eprintln!("[DEBUG] Loading TLS key from {}", key_path);
	}

	// Create TLS config using axum-server
	let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&cert_path, &key_path)
		.await
		.map_err(|e| format!("Failed to load TLS config: {}", e))?;

	if args.verbose >= 2 {
		eprintln!("[DEBUG] TLS configuration loaded successfully");
	}

	if args.verbose >= 1 {
		eprintln!("[INFO] Server started successfully, waiting for connections...");
		eprintln!("[INFO] Press Ctrl+C to shutdown gracefully");
	}

	// Set up signal handlers for graceful shutdown
	let shutdown_tx = setup_signal_handler(args.verbose as u32)?;
	let mut shutdown_rx = shutdown_tx.subscribe();

	// Start server with TLS using axum-server
	let server = axum_server::bind_rustls(addr, tls_config).serve(app.into_make_service());

	// Race between server and shutdown signal
	tokio::select! {
		result = server => {
			// Server ended (unexpected)
			result.map_err(|e| format!("Server error: {}", e))?;
		}
		_ = shutdown_rx.recv() => {
			// Graceful shutdown triggered
			if args.verbose >= 1 {
				eprintln!("[INFO] Shutdown signal received, stopping gracefully...");
			}

			// Signal all worker tasks to shut down
			let _ = shutdown_workers_tx.send(());
			if args.verbose >= 2 {
				eprintln!("[DEBUG] Signaled {} workers to shut down", worker_count);
			}

			// Give workers a moment to exit cleanly
			tokio::time::sleep(std::time::Duration::from_millis(100)).await;

			// Stop docker containers (both tracked and any that may be running)
			let service_manager = ServiceManager::new();
			let _ = service_manager.stop_all_services(args.verbose as u32).await;

			if args.verbose >= 1 {
				eprintln!("[INFO] Server shutdown complete");
			}
		}
	}

	Ok(())
}

/// Handle the stats command
pub async fn stats(args: crate::cli::StatsArgs) -> Result<(), String> {
	if args.verbose >= 1 {
		eprintln!("[INFO] Retrieving database statistics");
	}

	// Determine database connection string
	let db_conn = if let Some(db_str) = args.database {
		db_str
	} else {
		// Default SQLite database path in user data directory
		get_default_database_path()
	};

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Connecting to database: {}", db_conn);
	}

	// Retrieve statistics from database
	let stats = crate::storage::db_stats::DatabaseStats::from_db_path(&db_conn)
		.map_err(|e| format!("Failed to retrieve database statistics: {}", e))?;

	// Format and output results
	match args.format {
		crate::cli::OutputFormat::Json => {
			let json = serde_json::to_string_pretty(&stats)
				.map_err(|e| format!("Failed to serialize JSON: {}", e))?;
			println!("{}", json);
		}
		crate::cli::OutputFormat::Text => {
			let text = stats.format_text(args.detailed);
			print!("{}", text);
		}
		_ => {
			return Err(format!(
				"Output format {:?} not supported for stats command",
				args.format
			));
		}
	}

	Ok(())
}

pub async fn generate_tables(args: crate::cli::GenerateTablesArgs) -> Result<(), String> {
	let builder = crate::enrichment::rainbow_table_builder::RainbowTableBuilder::new()
		.with_output_path(".cache/rainbow_table.json".to_string());

	let builder = if !args.include_ntlm {
		builder.without_ntlm()
	} else {
		builder
	};

	let builder = if !args.include_sha512 {
		builder.without_sha512()
	} else {
		builder
	};

	// Generate the table
	let table = builder
		.generate()
		.map_err(|e| format!("Failed to generate rainbow table: {}", e))?;

	// Ensure output directory exists
	if let Some(output_path) = &args.output {
		if let Some(parent) = output_path.parent() {
			std::fs::create_dir_all(parent)
				.map_err(|e| format!("Failed to create output directory: {}", e))?;
		}

		let json = serde_json::to_string_pretty(&table)
			.map_err(|e| format!("Failed to serialize JSON: {}", e))?;
		std::fs::write(output_path, json)
			.map_err(|e| format!("Failed to write output file: {}", e))?;

		eprintln!(
			"[INFO] Rainbow table generated: {} ({} entries)",
			output_path.display(),
			table.entries.len()
		);
	} else {
		// Write to stdout
		let json = serde_json::to_string_pretty(&table)
			.map_err(|e| format!("Failed to serialize JSON: {}", e))?;
		println!("{}", json);
	}

	Ok(())
}

/// Export the database to a JSON file with deduplication support.
pub async fn export_db(args: crate::cli::ExportDbArgs) -> Result<(), String> {
	use rusqlite::Connection;

	if args.verbose >= 1 {
		eprintln!("[INFO] Exporting database to {}", args.output.display());
	}

	// Connect to database
	let db_path = args
		.database
		.unwrap_or_else(|| "./dumptruck.db".to_string());

	let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Connected to database at {}", db_path);
	}

	// Export database
	let export = crate::storage::db_export::export_database(&conn)
		.map_err(|e| format!("Failed to export database: {}", e))?;

	if args.verbose >= 1 {
		eprintln!(
			"[INFO] Exported {} canonical addresses, {} file metadata records",
			export.canonical_addresses.len(),
			export.file_metadata.len()
		);
	}

	// Ensure output directory exists
	if let Some(parent) = args.output.parent() {
		std::fs::create_dir_all(parent)
			.map_err(|e| format!("Failed to create output directory: {}", e))?;
	}

	// Write to file
	let json = serde_json::to_string_pretty(&export)
		.map_err(|e| format!("Failed to serialize JSON: {}", e))?;
	std::fs::write(&args.output, json)
		.map_err(|e| format!("Failed to write output file: {}", e))?;

	if args.verbose >= 1 {
		eprintln!(
			"[INFO] Database exported successfully to {}",
			args.output.display()
		);
	}

	Ok(())
}

/// Import a database from a JSON export file with deduplication.
pub async fn import_db(args: crate::cli::ImportDbArgs) -> Result<(), String> {
	use rusqlite::Connection;

	if args.verbose >= 1 {
		eprintln!("[INFO] Importing database from {}", args.input.display());
	}

	// Read JSON file
	let json_content = std::fs::read_to_string(&args.input)
		.map_err(|e| format!("Failed to read input file: {}", e))?;

	// Parse JSON
	let export: crate::storage::db_export::DatabaseExport =
		serde_json::from_str(&json_content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

	if args.verbose >= 2 {
		eprintln!(
			"[DEBUG] Parsed export: {} canonical addresses, {} file metadata",
			export.canonical_addresses.len(),
			export.file_metadata.len()
		);
	}

	// Validate if requested
	if args.validate {
		if args.verbose >= 1 {
			eprintln!("[INFO] Validating export data...");
		}

		if export.canonical_addresses.is_empty() {
			return Err("Export contains no canonical addresses".to_string());
		}

		if args.verbose >= 2 {
			eprintln!("[DEBUG] Validation passed");
		}
	}

	// Connect to database
	let db_path = args
		.database
		.unwrap_or_else(|| "./dumptruck.db".to_string());

	let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Connected to database at {}", db_path);
	}

	// Import with deduplication
	let stats = crate::storage::db_import::import_database(&conn, &export)
		.map_err(|e| format!("Failed to import database: {}", e))?;

	if args.verbose >= 1 {
		eprintln!(
			"[INFO] Import complete: {} records imported, {} skipped (duplicates)",
			stats.total_imported(),
			stats.total_skipped()
		);
		eprintln!(
			"  - Canonical addresses: {} imported, {} skipped",
			stats.canonical_addresses_imported, stats.canonical_addresses_skipped
		);
		eprintln!(
			"  - File metadata: {} imported, {} skipped",
			stats.file_metadata_imported, stats.file_metadata_skipped
		);
		eprintln!(
			"  - Chain of custody: {} imported, {} skipped",
			stats.chain_of_custody_imported, stats.chain_of_custody_skipped
		);
	}

	Ok(())
}
/// Get the default database path in the user data directory.
///
/// Uses platform-specific data directories:
/// - Linux: ~/.local/share/dumptruck/dumptruck.db
/// - macOS: ~/Library/Application Support/dumptruck/dumptruck.db
/// - Windows: %APPDATA%\dumptruck\dumptruck.db
fn get_default_database_path() -> String {
	if let Some(data_dir) = dirs::data_dir() {
		let db_dir = data_dir.join("dumptruck");
		let db_path = db_dir.join("dumptruck.db");
		// Create the directory if it doesn't exist
		let _ = std::fs::create_dir_all(&db_dir);
		db_path.to_string_lossy().to_string()
	} else {
		// Fallback to current directory if dirs crate cannot determine data dir
		"dumptruck.db".to_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_detect_format_csv() {
		let path = Path::new("test.csv");
		assert_eq!(detect_format_from_path(path), "csv");
	}

	#[test]
	fn test_detect_format_json() {
		let path = Path::new("test.json");
		assert_eq!(detect_format_from_path(path), "json");
	}

	#[test]
	fn test_detect_format_default() {
		let path = Path::new("test");
		assert_eq!(detect_format_from_path(path), "csv");
	}
}
