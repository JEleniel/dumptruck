//! Dumptruck command line interface definitions
use clap::Parser;
use reqwest::Url;
use std::path::PathBuf;

use crate::{
	analyze::AnalyzeArgs,
	configuration::APIKey,
	database::{exportargs::ExportArgs, importargs::ImportArgs},
	server::ServerArgs,
	status::StatusArgs,
};

/// Shared command line interface structure, including all available commands
#[derive(Parser, Debug)]
#[command(
	about = "Analyze bulk data dumps for credentials, identify breaches, and other threat intelligence",
	long_about = "Dumptruck ingests, normalizes, and analyzes large data dumps with surgical precision. Designed for security teams analyzing credential leaks, breach datasets, and threat intelligence at scale—processing gigabytes of data in memory-efficient streaming pipelines while maintaining complete privacy of historical records through non-reversible hashing."
)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,

	/// Override the configuration file search
	/// Default locations are:
	/// 1. ./config.json
	/// 2. /etc/dumptruck/config.json
	/// 3. The user configuration folder (e.g., ~/.config/dumptruck/config.json on Linux)
	#[arg(long, short = 'c', value_name = "CONFIGURATION")]
	pub config: Option<PathBuf>,

	/// Enable generating embeddings
	/// Currently, this will start an Ollama docker container and use
	/// nomic embeddings to generate vectrs
	#[arg(long)]
	pub embeddings: bool,
	/// External Ollama server URL
	/// If this is provided the Ollama docker container will not be started
	#[arg(long, value_name = "URL")]
	pub ollama_url: Option<Url>,
	/// Vector similarity threshold for near-duplicate detection (0.0-1.0)
	/// Ignored if embeddings are not enabled
	#[arg(long)]
	pub vector_threshold: Option<f32>,

	/// Path to the main database, if different from the default, excluding the filename.
	/// The database file is always named "dumptruck.db".
	/// By default it will be created in the "data" folder of the user under which DumpTruck is runnung
	/// e.g. /home/<username>/.local/share/dumptruck/dumptruck.db on Linux
	#[arg(long, short)]
	pub database: Option<PathBuf>,

	/// Working directory for isolated file processing
	/// Defaults to the temp folder (e.g. /tmp/dumptruck/ on Linux)
	/// If specified, Dumptruck will attempt to create this entire path if it does not exist
	/// and will use it for all temporary working copies of files during processing
	#[arg(long)]
	pub temp_path: Option<PathBuf>,

	/// API Keys
	#[arg(long, short = 'a', value_parser = |v: &str| {
		let parts: Vec<&str> = v.splitn(2, '=').collect();
		if parts.len() != 2 {
			return Err(String::from(
				"API key must be in the format SERVICE=KEY, e.g., 'haveibeenpwned=YOUR_KEY_HERE'",
			));
		}
		Ok(APIKey {
			api_name: parts[0].to_string(),
			api_key: parts[1].to_string(),
		})
	})]
	pub api_keys: Option<Vec<APIKey>>,
}

/// Available commands for Dumptruck
#[derive(Parser, Debug)]
pub enum Commands {
	/// Ingest and analyze data files
	Analyze(AnalyzeArgs),
	/// Show system information for a running server instance
	Status(StatusArgs),
	/// Export the database to a "seed" database file
	Export(ExportArgs),
	/// Merge a database file into the main database
	/// This is primarily used to import "seed" database files
	Import(ImportArgs),
	/// Start in HTTP/2 server mode with TLS 1.3+ and OAuth authentication
	Serve(ServerArgs),
}
