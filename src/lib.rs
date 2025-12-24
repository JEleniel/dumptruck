mod regexes;

use clap::Parser;
use cli::{Cli, Commands};

// Public module hierarchy
pub mod api;
pub mod cli;
pub mod core;
pub mod deploy;
pub mod detection;
pub mod enrichment;
pub mod ingest;
pub mod network;
pub mod normalization;
pub mod seed;
pub mod storage;

// Backwards compatibility: re-export specific items (not modules with conflicting names)

pub async fn run() {
	// Initialize rainbow table from external JSON file
	if let Err(e) = detection::rainbow_table::initialize() {
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
		Commands::Seed(args) => args.verbose as u32,
		Commands::Server(args) => args.verbose as u32,
		Commands::GenerateTables(_) => 0,
	};

	// Create service manager to handle startup/shutdown (Ollama-only if enabled and Docker available)
	let service_manager = deploy::ServiceManager::new();

	// NOTE: Service startup is not automatically triggered. Services are optional:
	// - If Ollama is enabled in config AND Docker is available, it can be manually started
	// - The system works fine without Docker or optional services

	// Dispatch to appropriate command handler
	let result = match cli.command {
		Commands::Ingest(args) => api::handlers::ingest(args).await,
		Commands::Status(args) => api::handlers::status(args).await,
		Commands::Stats(args) => api::handlers::stats(args).await,
		Commands::ExportDb(args) => api::handlers::export_db(args).await,
		Commands::ImportDb(args) => api::handlers::import_db(args).await,
		Commands::Seed(args) => api::handlers::seed(args).await,
		Commands::Server(args) => api::handlers::server(args).await,
		Commands::GenerateTables(args) => api::handlers::generate_tables(args).await,
	};

	// Stop any containers that we started
	if let Err(e) = service_manager.stop_started_containers(verbose).await
		&& verbose >= 1
	{
		eprintln!("Warning: Failed to stop containers: {}", e);
	}

	// Exit with appropriate code
	if let Err(e) = result {
		eprintln!("Error: {}", e);
		std::process::exit(1);
	}
}
