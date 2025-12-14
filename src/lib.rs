use cli::{Cli, Commands};
use clap::Parser;

pub async fn run() {
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
		Commands::Server(args) => args.verbose as u32,
	};

	// Create service manager to handle startup/shutdown
	let mut service_manager = deploy_manager::ServiceManager::new();

	// For server and ingest commands, ensure services are available
	match &cli.command {
		Commands::Server(_) | Commands::Ingest(_) => {
			if let Err(e) = service_manager.ensure_services_running(verbose).await {
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
		Commands::Server(args) => handlers::server(args).await,
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
pub mod async_pipeline;
pub mod cli;
pub mod config;
pub mod deploy_manager;
pub mod enrichment;
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
pub mod safe_ingest;
pub mod server;
pub mod storage;
pub mod streaming;
pub mod tls;
