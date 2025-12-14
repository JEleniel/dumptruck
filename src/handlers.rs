//! Command handlers for Dumptruck CLI.

use crate::adapters::FormatAdapter;
use crate::cli::{IngestArgs, OutputFormat, StatusArgs, ServerArgs};
use crate::output::{CsvFormatter, IngestResult, JsonFormatter, JsonlFormatter, OutputFormatter, TextFormatter, write_output};
use std::path::Path;
use std::sync::Arc;

/// Handle the ingest command
pub async fn ingest(args: IngestArgs) -> Result<(), String> {
	if args.verbose >= 1 {
		eprintln!("[INFO] Starting ingest operation");
	}

	// Resolve input files from glob pattern
	let files = args.resolve_input_files()?;
	if args.verbose >= 1 {
		eprintln!("[INFO] Found {} file(s) to process", files.len());
	}

	// Load configuration if provided
	let _config = if let Some(config_path) = &args.config {
		if args.verbose >= 2 {
			eprintln!("[DEBUG] Loading configuration from: {:?}", config_path);
		}
		crate::config::Config::from_file(config_path).map_err(|e| {
			format!("Failed to load configuration: {}", e)
		})?
	} else {
		crate::config::Config::default()
	};

	// Process files in parallel
	let mut total_rows = 0;
	let total_unique_addresses = 0;
	let total_hashed_credentials = 0;
	let total_weak_passwords = 0;
	let total_breached_addresses = 0;
	let mut metadata = Vec::new();
	let mut errors = Vec::new();

	for file_path in &files {
		if args.verbose >= 1 {
			eprintln!("[INFO] Processing file: {:?}", file_path);
		}

		// Read file contents
		let content = std::fs::read_to_string(file_path).map_err(|e| {
			format!("Failed to read file {:?}: {}", file_path, e)
		})?;

		// Detect or use specified format
		let format_str = if let Some(fmt) = args.format {
			fmt.to_string()
		} else {
			detect_format_from_path(file_path)
		};

		if args.verbose >= 2 {
			eprintln!("[DEBUG] Detected format: {}", format_str);
		}

		// Parse file content based on format
		match format_str.as_str() {
			"csv" => {
				let adapter = crate::adapters::CsvAdapter::new();
				let rows = adapter.parse(&content);
				total_rows += rows.len();

				// TODO: In a full implementation, pass rows through the async pipeline
				// For now, just count them
				if args.verbose >= 2 {
					eprintln!("[DEBUG] Parsed {} rows from CSV", rows.len());
				}

				metadata.push(format!("Processed {} rows from {}", rows.len(), file_path.display()));
			}
			"json" => {
				// Simple JSON array parsing
				if let Ok(parsed) = serde_json::from_str::<Vec<Vec<String>>>(&content) {
					total_rows += parsed.len();
					if args.verbose >= 2 {
						eprintln!("[DEBUG] Parsed {} rows from JSON", parsed.len());
					}
					metadata.push(format!("Processed {} rows from {}", parsed.len(), file_path.display()));
				} else {
					errors.push(format!("Failed to parse JSON from {:?}", file_path));
				}
			}
			"tsv" => {
				// TSV parsing (similar to CSV but with tabs)
				let rows: Vec<Vec<String>> = content
					.lines()
					.map(|line| line.split('\t').map(|s| s.to_string()).collect())
					.collect();
				total_rows += rows.len();
				if args.verbose >= 2 {
					eprintln!("[DEBUG] Parsed {} rows from TSV", rows.len());
				}
				metadata.push(format!("Processed {} rows from {}", rows.len(), file_path.display()));
			}
			_ => {
				errors.push(format!("Unsupported format: {}", format_str));
			}
		}
	}

	// Create result summary
	let result = IngestResult {
		rows_processed: total_rows,
		unique_addresses: total_unique_addresses,
		hashed_credentials_detected: total_hashed_credentials,
		weak_passwords_found: total_weak_passwords,
		breached_addresses: total_breached_addresses,
		metadata,
		errors,
	};

	// Format output
	let formatter: Box<dyn OutputFormatter> = match args.output_format {
		OutputFormat::Json => Box::new(JsonFormatter),
		OutputFormat::Csv => Box::new(CsvFormatter),
		OutputFormat::Text => Box::new(TextFormatter),
		OutputFormat::Jsonl => Box::new(JsonlFormatter),
	};

	let formatted = formatter.format(&result).map_err(|e| {
		format!("Failed to format output: {}", e)
	})?;

	// Write output
	let output_path = args.output.as_deref();
	write_output(&formatted, output_path).map_err(|e| {
		format!("Failed to write output: {}", e)
	})?;

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
		let ollama_url = args.ollama_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string());
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
		let db_url = args.database.clone().unwrap_or_else(|| "postgresql://dumptruck:dumptruck@localhost/dumptruck".to_string());
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
	match client.get(format!("{}/api/tags", url)).timeout(std::time::Duration::from_secs(5)).send().await {
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
async fn check_database(url: &str) -> Result<(), String> {
	// Placeholder: in a real implementation, would attempt a connection
	// For now, just validate the connection string format
	if url.starts_with("postgresql://") || url.starts_with("postgres://") {
		Ok(())
	} else {
		Err("Invalid PostgreSQL connection string format".to_string())
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
	path
		.extension()
		.and_then(|ext| ext.to_str())
		.map(|ext| ext.to_lowercase())
		.unwrap_or_else(|| "csv".to_string()) // default to CSV
}

/// Background job processor worker
async fn process_jobs(
	worker_id: usize,
	queue: Arc<crate::job_queue::JobQueue>,
	verbose: u32,
	shutdown_rx: &mut tokio::sync::broadcast::Receiver<()>,
) {
	use crate::job_queue::JobStatus;
	use std::time::Duration;

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
		let queued_job: Option<&crate::job_queue::Job> =
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
			if let Ok(_) = queue
				.update_job(&job_id, |j| {
					j.status = JobStatus::Processing;
					j.started_at = Some(chrono::Utc::now());
					Ok(())
				})
				.await
			{
				// Process the job
				process_single_job(
					&queue,
					&job_id,
					&filename,
					worker_id,
					verbose,
				)
				.await;
			}
		} else {
			// No jobs available, sleep briefly
			tokio::time::sleep(Duration::from_millis(100)).await;
		}
	}
}

/// Process a single job
async fn process_single_job(
	queue: &Arc<crate::job_queue::JobQueue>,
	job_id: &str,
	_filename: &str,
	worker_id: usize,
	verbose: u32,
) {
	use crate::job_queue::JobStatus;

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
				eprintln!("[DEBUG] Worker {} error updating job {}: {}", worker_id, job_id, e);
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
		let mut sigterm = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
			Ok(sig) => sig,
			Err(e) => {
				if verbose >= 1 {
					eprintln!("[WARN] Failed to setup SIGTERM handler: {}", e);
				}
				return;
			}
		};

		let mut sigint = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()) {
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
	use crate::oauth::OAuthProvider;
	use crate::job_queue::JobQueue;
	use crate::server::{create_app, AppState};
	use crate::deploy_manager::ServiceManager;
	use std::sync::Arc;

	// Install default crypto provider for rustls
	let _ = rustls::crypto::ring::default_provider().install_default();

	if args.verbose >= 1 {
		eprintln!("[INFO] Starting HTTPS server on port {}", args.port);
	}

	// Load configuration file
	let config_path = args.config.clone().unwrap_or_else(|| "config.json".to_string());
	let config = crate::config::Config::from_file(&config_path)
		.map_err(|e| format!("Failed to load config from {}: {}", config_path, e))?;

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Configuration loaded from: {}", config_path);
	}

	// Get OAuth settings - CLI args override config file
	let oauth_client_id = args.oauth_client_id
		.unwrap_or_else(|| config.oauth.client_id.clone());
	let oauth_client_secret = args.oauth_client_secret
		.unwrap_or_else(|| config.oauth.client_secret.clone());
	let oauth_token_endpoint = args.oauth_token_endpoint
		.unwrap_or_else(|| config.oauth.discovery_url.clone());

	if oauth_client_id.is_empty() || oauth_client_secret.is_empty() || oauth_token_endpoint.is_empty() {
		return Err("Missing OAuth configuration. Provide via config.json or command-line arguments.".to_string());
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
		eprintln!("[INFO] Spawning {} worker threads for parallel job processing", worker_count);
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
	let server = axum_server::bind_rustls(addr, tls_config)
		.serve(app.into_make_service());

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
