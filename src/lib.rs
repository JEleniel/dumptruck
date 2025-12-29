pub mod analyze;
pub mod api;
pub mod cli;
mod common;
mod configuration;
pub mod core;
pub mod data;
mod datafile;
pub mod deploy;
pub mod detection;
pub mod enrichment;
pub mod network;
pub mod normalization;
mod regexes;
pub mod seed;
mod server;
mod status;

use crate::{configuration::Configuration, seed::SeedManager};
use clap::Parser;
use cli::{Cli, Commands};
use thiserror::Error;

pub async fn run() -> Result<(), RunError> {
	// Parse command-line arguments
	let cli = match Cli::try_parse() {
		Ok(cli) => cli,
		Err(e) => {
			eprintln!("{}", e);
			std::process::exit(1);
		}
	};

	let configuration = Configuration::load(&cli.config)?;

	let configuration = configuration.apply_cli_overrides(&cli);

	match cli.command {
		Commands::Seed(args) => {
			SeedManager::run(args).await?;
		}
		Commands::Analyze(args) => {}
		Commands::Import(args) => {}
		Commands::Export(args) => {}
		Commands::Serve(args) => {}
		Commands::Status(args) => {}
	};
}

#[derive(Debug, Error)]
pub enum RunError {
	#[error("Configuration error: {0}")]
	ConfigurationError(#[from] configuration::ConfigurationError),
}
