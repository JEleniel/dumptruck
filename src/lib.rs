use clap::Parser;
use cli::{Cli, Commands};

pub async fn run() {
	// Initialize rainbow table from external JSON file
	if let Err(e) = rainbow_table::initialize() {
		eprintln!("Warning: Failed to initialize rainbow table: {}", e);
	}

	// Parse command-line arguments
	let cli = match Cli::try_parse() {
		Ok(cli) => cli,
		Err(e) => {
			eprintln!("{}", e);
			std::process::exit(1);
		}
	};

	// Get verbose level from the appropriate command
	let verbose = match &cli.command {
		Commands::Ingest(args) => args.verbose as u32,
		Commands::Status(args) => args.verbose as u32,
		Commands::Stats(args) => args.verbose as u32,
		Commands::ExportDb(args) => args.verbose as u32,
		Commands::ImportDb(args) => args.verbose as u32,
		Commands::Server(args) => args.verbose as u32,
		Commands::GenerateTables(_) => 0,
	};

	// Load configuration for service management
	let config = match config::Config::from_file("config.json") {
		Ok(cfg) => Some(cfg),
		Err(e) => {
			if verbose >= 2 {
				eprintln!("[DEBUG] Could not load config.json, using defaults: {}", e);
			}
			None
		}
	};

	// Create service manager to handle startup/shutdown
	let mut service_manager = deploy_manager::ServiceManager::new();

	// For server, stats, and ingest commands, ensure services are available
	match &cli.command {
		Commands::Server(_)
		| Commands::Ingest(_)
		| Commands::Stats(_)
		| Commands::ExportDb(_)
		| Commands::ImportDb(_) => {
			if let Err(e) = service_manager
				.ensure_services_running(verbose, config.as_ref())
				.await
			{
				eprintln!("Error: Failed to ensure services are running: {}", e);
				std::process::exit(1);
			}
		}
		_ => {}
	}

	// Dispatch to appropriate command handler
	let result = match cli.command {
		Commands::Ingest(args) => handlers::ingest(args).await,
		Commands::Status(args) => handlers::status(args).await,
		Commands::Stats(args) => handlers::stats(args).await,
		Commands::ExportDb(args) => handlers::export_db(args).await,
		Commands::ImportDb(args) => handlers::import_db(args).await,
		Commands::Server(args) => handlers::server(args).await,
		Commands::GenerateTables(args) => handlers::generate_tables(args).await,
	};

	// Stop any containers that we started
	if let Err(e) = service_manager.stop_started_containers(verbose).await {
		if verbose >= 1 {
			eprintln!("Warning: Failed to stop containers: {}", e);
		}
	}

	// Exit with appropriate code
	if let Err(e) = result {
		eprintln!("Error: {}", e);
		std::process::exit(1);
	}
}

pub mod adapters;
pub mod alias_resolution;
pub mod anomaly_detection;
pub mod async_pipeline;
pub mod chain_of_custody;
pub mod cli;
pub mod compression;
pub mod config;
pub mod db_export;
pub mod db_import;
pub mod db_stats;
pub mod deploy_manager;
pub mod detection;
pub mod enrichment;
pub mod evidence;
pub mod file_lock;
pub mod handlers;
pub mod hash_utils;
pub mod hibp;
pub mod job_queue;
pub mod normalization;
pub mod npi_detection;
pub mod oauth;
pub mod ollama;
pub mod output;
pub mod peer_discovery;
pub mod peer_sync;
pub mod pipeline;
pub mod rainbow_table;
pub mod rainbow_table_builder;
pub mod risk_scoring;
pub mod safe_ingest;
pub mod secure_deletion;
pub mod server;
pub mod storage;
pub mod streaming;
pub mod tls;
pub mod universal_parser;
pub mod working_copy;
