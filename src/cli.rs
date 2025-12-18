//! Command-line interface for Dumptruck bulk data analysis.

use clap::{Parser, ValueEnum};
use glob::glob;
use rayon::prelude::*;
use std::path::PathBuf;

/// Dumptruck: Bulk data analysis tool for cyber threat identification
#[derive(Parser, Debug)]
#[command(name = "dumptruck")]
#[command(about = "Analyze bulk data dumps for credentials, identify breaches, and track threat intelligence", long_about = None)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
	/// Ingest and analyze data files (supports glob patterns and parallel processing)
	Ingest(IngestArgs),
	/// Show system information and connectivity
	Status(StatusArgs),
	/// Display database statistics and analytics
	Stats(StatsArgs),
	/// Export database to JSON with deduplication support
	ExportDb(ExportDbArgs),
	/// Import database from JSON export with deduplication
	ImportDb(ImportDbArgs),
	/// Start HTTP/2 server with TLS 1.3+ and OAuth authentication
	Server(ServerArgs),
	/// Generate rainbow table entries with SHA512 and NTLM hashes for weak passwords
	GenerateTables(GenerateTablesArgs),
}

/// Arguments for the ingest command
#[derive(Parser, Debug)]
pub struct IngestArgs {
	/// Path to input data file(s) - supports glob patterns (*, ?)
	/// Examples: data.csv, *.csv, /path/to/*.{csv,json}
	#[arg(value_name = "FILE|PATTERN")]
	pub input: String,

	/// Output file for results (default: stdout)
	#[arg(short, long, value_name = "FILE")]
	pub output: Option<PathBuf>,

	/// Database connection string (default: docker-compose service)
	#[arg(long, value_name = "CONN_STRING")]
	pub database: Option<String>,

	/// Use filesystem storage instead of database
	#[arg(long)]
	pub filesystem: bool,

	/// Path for filesystem storage when --filesystem is used
	#[arg(long, value_name = "PATH")]
	pub storage_path: Option<PathBuf>,

	/// Input data format (auto-detected if not specified)
	#[arg(short, long, value_enum)]
	pub format: Option<InputFormat>,

	/// Enable Ollama embeddings for address deduplication
	#[arg(long)]
	pub embeddings: bool,

	/// Ollama server URL (default: http://localhost:11434)
	#[arg(long, value_name = "URL")]
	pub ollama_url: Option<String>,

	/// Enable HIBP breach lookups
	#[arg(long)]
	pub hibp: bool,

	/// HIBP API key (or set DUMPTRUCK_HIBP_KEY environment variable)
	#[arg(long, value_name = "KEY")]
	pub hibp_key: Option<String>,

	/// Vector similarity threshold for near-duplicate detection (0.0-1.0)
	#[arg(long, default_value = "0.85")]
	pub similarity_threshold: f32,

	/// Verbosity level (repeat for more verbose: -v, -vv, -vvv)
	#[arg(short, action = clap::ArgAction::Count)]
	pub verbose: u8,

	/// Output format for results
	#[arg(long, value_enum, default_value = "json")]
	pub output_format: OutputFormat,

	/// Configuration file path (JSON format) for API keys and domain substitutions
	#[arg(long, short = 'c', value_name = "FILE")]
	pub config: Option<PathBuf>,

	/// Number of parallel workers for processing multiple files (default: number of CPU cores)
	#[arg(long)]
	pub workers: Option<usize>,

	/// Working directory for isolated file processing (default: /tmp/dumptruck/)
	/// All work happens in isolated copies here; original files are never modified
	#[arg(long, value_name = "PATH")]
	pub working_dir: Option<PathBuf>,

	/// Verify working directory is mounted with noexec flag (default: false)
	/// Set to true to enforce strict security checking (may fail if /tmp doesn't have noexec)
	#[arg(long, default_value = "false")]
	pub verify_noexec: bool,
}

impl IngestArgs {
	/// Resolve glob pattern(s) to actual file paths
	///
	/// Supports standard glob patterns:
	/// - `*` matches any sequence of characters (except /)
	/// - `?` matches any single character
	/// - `[abc]` matches any character in brackets
	/// - `**` matches zero or more directories
	pub fn resolve_input_files(&self) -> Result<Vec<PathBuf>, String> {
		let pattern = &self.input;

		// Check if it's a literal file path (no glob chars)
		if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
			let path = PathBuf::from(pattern);
			if path.exists() {
				return Ok(vec![path]);
			} else {
				return Err(format!("File not found: {}", pattern));
			}
		}

		// Try to glob the pattern
		let glob_results =
			glob(pattern).map_err(|e| format!("Invalid glob pattern '{}': {}", pattern, e))?;

		let mut files = Vec::new();
		for entry in glob_results {
			match entry {
				Ok(path) => {
					if path.is_file() {
						files.push(path);
					}
				}
				Err(e) => {
					return Err(format!("Error reading glob entry: {}", e));
				}
			}
		}

		if files.is_empty() {
			return Err(format!("No files found matching pattern: {}", pattern));
		}

		Ok(files)
	}

	/// Process multiple files in parallel
	pub fn process_files_parallel<F, T>(&self, process_fn: F) -> Result<Vec<T>, String>
	where
		F: Fn(&PathBuf) -> Result<T, String> + Send + Sync,
		T: Send,
	{
		let files = self.resolve_input_files()?;

		if files.is_empty() {
			return Ok(Vec::new());
		}

		// Set up thread pool with specified or default worker count
		if let Some(workers) = self.workers {
			rayon::ThreadPoolBuilder::new()
				.num_threads(workers)
				.build()
				.map_err(|e| format!("Failed to create thread pool: {}", e))?
				.install(|| {
					files
						.par_iter()
						.map(|f| process_fn(f))
						.collect::<Result<Vec<_>, _>>()
				})
		} else {
			// Use default parallelization
			Ok(files
				.par_iter()
				.map(|f| process_fn(f))
				.collect::<Result<Vec<_>, _>>()?)
		}
	}
}

/// Arguments for the status command
#[derive(Parser, Debug)]
pub struct StatusArgs {
	/// Check Ollama connectivity
	#[arg(long)]
	pub check_ollama: bool,

	/// Check database connectivity
	#[arg(long)]
	pub check_database: bool,

	/// Check HIBP API connectivity
	#[arg(long)]
	pub check_hibp: bool,

	/// Ollama server URL
	#[arg(long, value_name = "URL")]
	pub ollama_url: Option<String>,

	/// Database connection string
	#[arg(long, value_name = "CONN_STRING")]
	pub database: Option<String>,

	/// HIBP API key
	#[arg(long, value_name = "KEY")]
	pub hibp_key: Option<String>,

	/// Verbosity level
	#[arg(short, action = clap::ArgAction::Count)]
	pub verbose: u8,
}

/// Arguments for the stats command
#[derive(Parser, Debug)]
pub struct StatsArgs {
	/// Database connection string (default: docker-compose service)
	#[arg(long, value_name = "CONN_STRING")]
	pub database: Option<String>,

	/// Show detailed breakdown
	#[arg(long)]
	pub detailed: bool,

	/// Output format
	#[arg(long, value_enum, default_value = "text")]
	pub format: OutputFormat,

	/// Verbosity level
	#[arg(short, action = clap::ArgAction::Count)]
	pub verbose: u8,
}

/// Arguments for the export-db command
#[derive(Parser, Debug)]
pub struct ExportDbArgs {
	/// Output file for exported database (JSON format)
	#[arg(short, long, value_name = "FILE")]
	pub output: PathBuf,

	/// Database connection string (default: docker-compose service)
	#[arg(long, value_name = "CONN_STRING")]
	pub database: Option<String>,

	/// Include detailed metadata and audit trails
	#[arg(long)]
	pub detailed: bool,

	/// Verbosity level
	#[arg(short, action = clap::ArgAction::Count)]
	pub verbose: u8,
}

/// Arguments for the import-db command
#[derive(Parser, Debug)]
pub struct ImportDbArgs {
	/// Input file from database export (JSON format)
	#[arg(short, long, value_name = "FILE")]
	pub input: PathBuf,

	/// Database connection string (default: docker-compose service)
	#[arg(long, value_name = "CONN_STRING")]
	pub database: Option<String>,

	/// Validate data integrity before import
	#[arg(long)]
	pub validate: bool,

	/// Verbosity level
	#[arg(short, action = clap::ArgAction::Count)]
	pub verbose: u8,
}

/// Arguments for the generate-tables command
#[derive(Debug, Clone, Parser)]
pub struct GenerateTablesArgs {
	/// Output file for generated Rust code (default: stdout)
	#[arg(short, long, value_name = "FILE")]
	pub output: Option<PathBuf>,

	/// Include NTLM hashes (requires proper MD4 support)
	#[arg(long, default_value = "true")]
	pub include_ntlm: bool,

	/// Include SHA512 hashes
	#[arg(long, default_value = "true")]
	pub include_sha512: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InputFormat {
	/// Comma-separated values
	#[value(name = "csv")]
	Csv,
	/// Tab-separated values
	#[value(name = "tsv")]
	Tsv,
	/// JavaScript Object Notation
	#[value(name = "json")]
	Json,
	/// YAML format
	#[value(name = "yaml")]
	Yaml,
	/// Protocol Buffers (binary format)
	#[value(name = "protobuf")]
	Protobuf,
}

impl std::fmt::Display for InputFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			InputFormat::Csv => write!(f, "csv"),
			InputFormat::Tsv => write!(f, "tsv"),
			InputFormat::Json => write!(f, "json"),
			InputFormat::Yaml => write!(f, "yaml"),
			InputFormat::Protobuf => write!(f, "protobuf"),
		}
	}
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
	/// JSON format
	#[value(name = "json")]
	Json,
	/// CSV format for spreadsheets
	#[value(name = "csv")]
	Csv,
	/// Human-readable summary
	#[value(name = "text")]
	Text,
	/// Newline-delimited JSON
	#[value(name = "jsonl")]
	Jsonl,
}

impl std::fmt::Display for OutputFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			OutputFormat::Json => write!(f, "json"),
			OutputFormat::Csv => write!(f, "csv"),
			OutputFormat::Text => write!(f, "text"),
			OutputFormat::Jsonl => write!(f, "jsonl"),
		}
	}
}

/// Server mode arguments for HTTP/2 API with TLS 1.3+ and OAuth 2.0
#[derive(Debug, Clone, Parser)]
pub struct ServerArgs {
	/// Port to bind the server to
	#[arg(short, long, value_name = "PORT", default_value = "8443")]
	pub port: u16,

	/// Path to configuration file (defaults to config.json)
	#[arg(short, long, value_name = "PATH")]
	pub config: Option<String>,

	/// Path to TLS certificate file (PEM format) - overrides config
	#[arg(long, value_name = "PATH")]
	pub cert: Option<String>,

	/// Path to TLS private key file (PEM format) - overrides config
	#[arg(long, value_name = "PATH")]
	pub key: Option<String>,

	/// OAuth 2.0 Client ID - overrides config
	#[arg(long, value_name = "ID")]
	pub oauth_client_id: Option<String>,

	/// OAuth 2.0 Client Secret - overrides config
	#[arg(long, value_name = "SECRET")]
	pub oauth_client_secret: Option<String>,

	/// OAuth 2.0 Token Endpoint URL - overrides config
	#[arg(long, value_name = "URL")]
	pub oauth_token_endpoint: Option<String>,

	/// OAuth 2.0 scopes (space-separated)
	#[arg(long, value_name = "SCOPES", default_value = "read:dumps write:dumps")]
	pub oauth_scope: String,

	/// Database connection string
	#[arg(long, value_name = "CONN_STRING")]
	pub database: Option<String>,

	/// Ollama server URL
	#[arg(long, value_name = "URL")]
	pub ollama_url: Option<String>,

	/// HIBP API key
	#[arg(long, value_name = "KEY")]
	pub hibp_key: Option<String>,

	/// Verbosity level
	#[arg(short, action = clap::ArgAction::Count)]
	pub verbose: u8,
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;

	#[test]
	fn test_cli_parse_basic_ingest() {
		let args = vec!["dumptruck", "ingest", "data.csv"];
		let cli = Cli::try_parse_from(args).expect("parse failed");
		match cli.command {
			Commands::Ingest(ingest) => {
				assert_eq!(ingest.input, "data.csv");
				assert_eq!(ingest.verbose, 0);
				assert!(ingest.format.is_none());
			}
			_ => panic!("Expected Ingest command"),
		}
	}

	#[test]
	fn test_cli_parse_with_options() {
		let args = vec![
			"dumptruck",
			"ingest",
			"data.csv",
			"-o",
			"output.json",
			"--embeddings",
			"--hibp",
			"-vv",
		];
		let cli = Cli::try_parse_from(args).expect("parse failed");
		match cli.command {
			Commands::Ingest(ingest) => {
				assert!(ingest.output.is_some());
				assert!(ingest.embeddings);
				assert!(ingest.hibp);
				assert_eq!(ingest.verbose, 2);
			}
			_ => panic!("Expected Ingest command"),
		}
	}

	#[test]
	fn test_cli_glob_pattern_parsing() {
		let args = vec![
			"dumptruck",
			"ingest",
			"tests/fixtures/*.csv",
			"--format",
			"csv",
			"-c",
			"config.json",
		];
		let cli = Cli::try_parse_from(args).expect("parse failed");
		match cli.command {
			Commands::Ingest(ingest) => {
				assert_eq!(ingest.input, "tests/fixtures/*.csv");
				assert!(ingest.format.is_some());
				assert!(ingest.config.is_some());
			}
			_ => panic!("Expected Ingest command"),
		}
	}

	#[test]
	fn test_cli_status_command() {
		let args = vec!["dumptruck", "status", "--check-database"];
		let cli = Cli::try_parse_from(args).expect("parse failed");
		match cli.command {
			Commands::Status(status) => {
				assert!(status.check_database);
				assert!(!status.check_ollama);
			}
			_ => panic!("Expected Status command"),
		}
	}

	#[test]
	fn test_glob_resolve_literal_path() {
		let ingest = IngestArgs {
			input: "tests/fixtures/well_formed_credentials.csv".to_string(),
			output: None,
			database: None,
			filesystem: false,
			storage_path: None,
			format: None,
			embeddings: false,
			ollama_url: None,
			hibp: false,
			hibp_key: None,
			similarity_threshold: 0.85,
			verbose: 0,
			output_format: OutputFormat::Json,
			config: None,
			workers: None,
			working_dir: None,
			verify_noexec: false,
		};

		let files = ingest.resolve_input_files().expect("failed to resolve");
		assert_eq!(files.len(), 1);
		assert!(
			files[0]
				.to_string_lossy()
				.contains("well_formed_credentials.csv")
		);
	}

	#[test]
	fn test_glob_resolve_pattern() {
		let ingest = IngestArgs {
			input: "tests/fixtures/well_formed*.csv".to_string(),
			output: None,
			database: None,
			filesystem: false,
			storage_path: None,
			format: None,
			embeddings: false,
			ollama_url: None,
			hibp: false,
			hibp_key: None,
			similarity_threshold: 0.85,
			verbose: 0,
			output_format: OutputFormat::Json,
			config: None,
			workers: None,
			working_dir: None,
			verify_noexec: false,
		};

		let files = ingest.resolve_input_files().expect("failed to resolve");
		assert!(!files.is_empty());
		assert!(
			files
				.iter()
				.any(|f| f.to_string_lossy().contains("well_formed"))
		);
	}
}
