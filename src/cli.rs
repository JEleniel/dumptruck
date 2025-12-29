//! Dumptruck command line interface definitions
use clap::Parser;
use reqwest::Url;
use std::{collections::HashMap, env, path::PathBuf};

use crate::common::Hash;

/// Shared command line interface structure, including all available commands
#[derive(Parser, Debug)]
#[command(name = env::var("CARGO_BIN_NAME").unwrap_or_else(|_| "dumptruck".to_string()))]
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
	#[arg(long, default_value = "0.85")]
	pub vector_threshold: f32,

	/// Path to the main database, if different from the default, excluding the filename.
	/// The database file is always named "dumptruck.db".
	/// By default it will be created in the "data" folder of the user under which DumpTruck is runnung
	/// e.g. /home/<username>/.local/share/dumptruck/dumptruck.db on Linux
	#[arg(long, value_name = "DATABASE_PATH")]
	pub database: Option<String>,

	/// Working directory for isolated file processing
	/// Defaults to the temp folder (e.g. /tmp/dumptruck/ on Linux)
	/// If specified, Dumptruck will attempt to create this entire path if it does not exist
	/// and will use it for all temporary working copies of files during processing
	#[arg(long, value_name = "PATH")]
	pub temp_path: Option<PathBuf>,

	/// API Keys
	#[arg(long, short = "k", value_name = "API_KEY")]
	pub api_keys: Option<HashMap<String, String>>,
}

/// Available commands for Dumptruck
#[derive(Parser, Debug)]
pub enum Commands {
	/// Ingest and analyze data files
	Analyze(super::analyze::AnalyzeArgs),
	/// Show system information for a running server instance
	Status(super::status::StatusArgs),
	/// Export the database to a "seed" database file
	Export(super::data::ExportArgs),
	/// Merge a "seed" database file into the main database
	Import(super::data::ImportArgs),
	/// Create seed database from folder of data files
	/// *.txt files contain plain text entries to generate Rainbow Tables
	/// *.csv files contain previous data breach information building a "seed" database
	/// The "import" command can then be used to merge this seed database
	/// into the main database
	Seed(super::seed::SeedArgs),
	/// Start in HTTP/2 server mode with TLS 1.3+ and OAuth authentication
	Serve(super::server::ServerArgs),
}
