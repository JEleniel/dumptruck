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

	// Dispatch to appropriate command handler
	let result = match cli.command {
		Commands::Ingest(args) => handlers::ingest(args).await,
		Commands::Status(args) => handlers::status(args).await,
		Commands::Server(args) => handlers::server(args).await,
	};

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
pub mod handlers;
pub mod hash_utils;
pub mod hibp;
pub mod job_queue;
pub mod normalization;
pub mod oauth;
pub mod ollama;
pub mod output;
pub mod pipeline;
pub mod rainbow_table;
pub mod server;
pub mod storage;
pub mod tls;
