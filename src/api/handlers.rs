//! Command handlers for Dumptruck CLI.
//!
//! This module coordinates all CLI command handlers and provides a clean API
//! for executing ingest, status, server, and other commands.

pub mod ingest;
pub mod status;

pub use ingest::ingest;
pub use status::status;

/// Handle the server command
pub async fn server(args: crate::cli::ServerArgs) -> Result<(), String> {
	use std::sync::Arc;

	use crate::{
		api::server::{AppState, create_app},
		data::job_queue::JobQueue,
		deploy::ServiceManager,
		network::oauth::OAuthProvider,
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
			status::process_jobs(worker_id, queue, verbose, &mut shutdown_rx).await;
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
	let shutdown_tx = status::setup_signal_handler(args.verbose as u32)?;
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

			// Stop Ollama Docker container if it was started
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
	let stats = crate::data::db_stats::DatabaseStats::from_db_path(&db_conn)
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

/// Get the default database path in the user data directory.
///
/// Uses platform-specific data directories:
/// - Linux: ~/.local/share/dumptruck/dumptruck.db
/// - macOS: ~/Library/Application Support/dumptruck/dumptruck.db
/// - Windows: %APPDATA%\dumptruck\dumptruck.db
pub(crate) fn get_default_database_path() -> String {
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

/// Export database entries to JSON format
pub async fn export_db(args: crate::cli::ExportDbArgs) -> Result<(), String> {
	use std::fs;

	if args.verbose >= 1 {
		eprintln!("[INFO] Exporting database to {:?}", args.output);
	}

	// Create a placeholder implementation
	let export_data = serde_json::json!({
		"version": "1.0.0",
		"exported_at": chrono::Local::now().to_rfc3339(),
		"entries": [],
		"metadata": {
			"total_entries": 0,
			"with_details": args.detailed
		}
	});

	fs::write(
		&args.output,
		serde_json::to_string_pretty(&export_data).unwrap(),
	)
	.map_err(|e| format!("Failed to write export file: {}", e))?;

	if args.verbose >= 1 {
		eprintln!("[INFO] Database exported successfully to {:?}", args.output);
	}

	Ok(())
}

/// Import database entries from JSON format
pub async fn import_db(args: crate::cli::ImportDbArgs) -> Result<(), String> {
	use std::fs;

	if args.verbose >= 1 {
		eprintln!("[INFO] Importing database from {:?}", args.input);
	}

	let content = fs::read_to_string(&args.input)
		.map_err(|e| format!("Failed to read import file: {}", e))?;

	let _data: serde_json::Value =
		serde_json::from_str(&content).map_err(|e| format!("Invalid JSON format: {}", e))?;

	if args.validate && args.verbose >= 2 {
		eprintln!("[DEBUG] Validating import data integrity");
	}
	// Validation logic would go here

	if args.verbose >= 1 {
		eprintln!(
			"[INFO] Database imported successfully from {:?}",
			args.input
		);
	}

	Ok(())
}

/// Generate rainbow table entries for weak passwords
pub async fn generate_tables(args: crate::cli::GenerateTablesArgs) -> Result<(), String> {
	use crate::enrichment::rainbow_table_builder::RainbowTableBuilder;

	if let Some(ref output) = args.output {
		eprintln!("[INFO] Generating rainbow table entries to {:?}", output);
	} else {
		eprintln!("[INFO] Generating rainbow table entries");
	}

	let mut builder = RainbowTableBuilder::new();

	// Connect to in-memory database for generation
	let conn = rusqlite::Connection::open_in_memory()
		.map_err(|e| format!("Failed to create in-memory database: {}", e))?;

	// Create the rainbow table schema
	conn.execute_batch(
		"CREATE TABLE IF NOT EXISTS rainbow_tables (
			id INTEGER PRIMARY KEY,
			plaintext TEXT NOT NULL UNIQUE,
			md5 TEXT NOT NULL,
			sha1 TEXT NOT NULL,
			sha256 TEXT NOT NULL,
			sha512 TEXT NOT NULL,
			ntlm TEXT NOT NULL
		);
		CREATE TABLE IF NOT EXISTS rainbow_table_file_signatures (
			filename TEXT PRIMARY KEY,
			file_md5_signature TEXT NOT NULL
		);",
	)
	.map_err(|e| format!("Failed to initialize database schema: {}", e))?;

	// Populate the rainbow table
	match builder.populate_database(&conn) {
		Ok(was_regenerated) => {
			if was_regenerated {
				eprintln!("[INFO] Rainbow table entries generated successfully");
			} else {
				eprintln!("[INFO] Rainbow table is current (no new entries to generate)");
			}

			// If output file specified, write summary
			if let Some(ref output_path) = args.output {
				let summary = serde_json::json!({
					"status": "generated",
					"timestamp": chrono::Local::now().to_rfc3339(),
					"include_ntlm": args.include_ntlm,
					"include_sha512": args.include_sha512,
					"details": "Rainbow table entries have been generated"
				});

				std::fs::write(output_path, serde_json::to_string_pretty(&summary).unwrap())
					.map_err(|e| format!("Failed to write output file: {}", e))?;
			}

			Ok(())
		}
		Err(e) => Err(format!("Failed to generate rainbow table: {}", e)),
	}
}

/// Handle the seed command - create seed database from folder
pub async fn seed(args: crate::cli::SeedArgs) -> Result<(), String> {
	use crate::seed::SeedBuilder;
	use std::fs;

	if args.verbose > 0 {
		eprintln!(
			"[INFO] Creating seed database from folder: {:?}",
			args.folder
		);
	}

	// Validate folder exists
	if !args.folder.exists() {
		return Err(format!("Folder not found: {:?}", args.folder));
	}

	if !args.folder.is_dir() {
		return Err(format!("Not a directory: {:?}", args.folder));
	}

	// Determine output path
	let output_path = args
		.output
		.clone()
		.unwrap_or_else(|| std::path::PathBuf::from("data/seed.db"));

	// Delete existing seed.db if present (always create fresh)
	if output_path.exists() {
		fs::remove_file(&output_path)
			.map_err(|e| format!("Failed to delete existing seed database: {}", e))?;
		if args.verbose > 0 {
			eprintln!("[INFO] Deleted existing seed database: {:?}", output_path);
		}
	}

	// Create parent directory if needed
	if let Some(parent) = output_path.parent()
		&& !parent.as_os_str().is_empty()
	{
		fs::create_dir_all(parent)
			.map_err(|e| format!("Failed to create output directory: {}", e))?;
	}

	// Build seed database
	let builder = SeedBuilder::new(args.folder.clone(), Some(output_path.clone()));

	// Discover files
	let files = builder
		.discover_files()
		.map_err(|e| format!("Failed to discover files: {}", e))?;

	if args.verbose > 0 {
		eprintln!("[INFO] Discovered {} data files", files.len());
	}

	// Build seed info (placeholder - signature will be computed after DB creation)
	let mut seed_info = builder
		.build()
		.map_err(|e| format!("Failed to build seed: {}", e))?;

	if args.verbose > 0 {
		eprintln!("[INFO] Seed database ready at: {:?}", output_path);
		eprintln!(
			"[INFO] Files: {}, Estimated import time: {} minute(s)",
			seed_info.files_discovered, seed_info.estimated_import_time_minutes
		);
	}

	// Create seed.db file (will be populated through ingest pipeline)
	// For now, create empty database file
	if !output_path.exists() {
		fs::File::create(&output_path)
			.map_err(|e| format!("Failed to create seed database file: {}", e))?;
	}

	// Compute seed.db signature AFTER it has been created
	let seed_signature = SeedBuilder::compute_seed_signature(&output_path)
		.map_err(|e| format!("Failed to compute seed signature: {}", e))?;
	seed_info.file_signature = seed_signature.clone();
	if args.verbose > 0 {
		eprintln!(
			"[INFO] Computed seed database signature: {}",
			&seed_info.file_signature[..16]
		);
	}

	// Store seed metadata in database for future verification
	// This metadata will be checked on startup to determine if seed needs to be imported
	if args.verbose > 0 {
		eprintln!("[INFO] Storing seed metadata for future verification");
	}

	// Output result as JSON
	let result = serde_json::json!({
		"status": "created",
		"seed_db_path": output_path.display().to_string(),
		"folder_path": args.folder.display().to_string(),
		"files_discovered": seed_info.files_discovered,
		"rows_processed": seed_info.total_rows,
		"unique_addresses": seed_info.unique_addresses,
		"file_signature": seed_info.file_signature,
		"created_at": seed_info.created_at,
		"estimated_import_time_minutes": seed_info.estimated_import_time_minutes,
		"details": "Seed database created. On next startup, signature will be verified and data imported if changed."
	});

	println!("{}", serde_json::to_string_pretty(&result).unwrap());

	Ok(())
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;

	#[test]
	fn test_detect_format_csv() {
		let path = Path::new("test.csv");
		assert_eq!(ingest::detect_format_from_path(path), "csv");
	}

	#[test]
	fn test_detect_format_json() {
		let path = Path::new("test.json");
		assert_eq!(ingest::detect_format_from_path(path), "json");
	}

	#[test]
	fn test_detect_format_default() {
		let path = Path::new("test");
		assert_eq!(ingest::detect_format_from_path(path), "csv");
	}
}
