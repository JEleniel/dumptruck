mod analyze;
mod api;
mod cli;
mod configuration;
pub mod database;
mod enrichment;
mod network;
mod rainbowtable;
mod server;
mod status;
mod util;

use crate::{configuration::Configuration, rainbowtable::RainbowTable};
use clap::Parser;
use cli::{Cli, Commands};
use thiserror::Error;
pub use util::*;

pub async fn run() -> Result<(), RunError> {
	// Parse command-line arguments
	let cli = match Cli::try_parse() {
		Ok(cli) => cli,
		Err(e) => {
			eprintln!("{}", e);
			std::process::exit(1);
		}
	};

	let mut configuration = Configuration::load(&cli.config)?;

	configuration.apply_cli_overrides(&cli)?;

	match cli.command {
		Commands::Analyze(args) => {
			analyze::analyze(configuration, args).await?;
		}
		Commands::Import(args) => {}
		Commands::Export(args) => {}
		Commands::Serve(args) => {}
		Commands::Status(args) => {}
		Commands::Rainbow(args) => {
			RainbowTable::generate(&configuration, args).await?;
		}
	};
	Ok(())
}

#[derive(Debug, Error)]
pub enum RunError {
	#[error("Configuration error: {0}")]
	ConfigurationError(#[from] configuration::ConfigurationError),
	#[error("Analyze error: {0}")]
	AnalyzeError(#[from] analyze::AnalyzeError),
	#[error("Rainbow table error: {0}")]
	RainbowTableError(#[from] rainbowtable::RainbowTableError),
}
