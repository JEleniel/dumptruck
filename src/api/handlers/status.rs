//! Status command handler and related functions.

use std::sync::Arc;

use crate::cli::StatusArgs;

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
			.unwrap_or_else(super::get_default_database_path);
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

/// Background job processor worker
pub(crate) async fn process_jobs(
	worker_id: usize,
	queue: Arc<crate::data::job_queue::JobQueue>,
	verbose: u32,
	shutdown_rx: &mut tokio::sync::broadcast::Receiver<()>,
) {
	use std::time::Duration;

	use crate::data::job_queue::JobStatus;

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
		let queued_job: Option<&crate::data::job_queue::Job> =
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
	queue: &Arc<crate::data::job_queue::JobQueue>,
	job_id: &str,
	_filename: &str,
	worker_id: usize,
	verbose: u32,
) {
	use crate::data::job_queue::JobStatus;

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

/// Set up signal handlers for graceful shutdown (SIGTERM, SIGINT on Unix; Ctrl-C on Windows)
pub(crate) fn setup_signal_handler(
	verbose: u32,
) -> Result<tokio::sync::broadcast::Sender<()>, String> {
	let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);
	let tx = shutdown_tx.clone();

	tokio::spawn(async move {
		#[cfg(unix)]
		{
			use tokio::signal::unix::{SignalKind, signal};

			let mut sigterm = match signal(SignalKind::terminate()) {
				Ok(sig) => sig,
				Err(e) => {
					if verbose >= 1 {
						eprintln!("[WARN] Failed to setup SIGTERM handler: {}", e);
					}
					return;
				}
			};

			let mut sigint = match signal(SignalKind::interrupt()) {
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
		}

		#[cfg(windows)]
		{
			match tokio::signal::ctrl_c().await {
				Ok(()) => {
					if verbose >= 1 {
						eprintln!("[INFO] Ctrl-C received");
					}
					let _ = tx.send(());
				}
				Err(e) => {
					if verbose >= 1 {
						eprintln!("[WARN] Failed to setup Ctrl-C handler: {}", e);
					}
				}
			}
		}
	});

	Ok(shutdown_tx)
}
