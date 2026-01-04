use std::path::PathBuf;

use clap::Parser;

/// Arguments for the export-db command
#[derive(Parser, Debug)]
pub struct ExportArgs {
	/// Output file for exported database (SQLite format)
	#[arg(short, long, value_name = "OUTPUT_FILE")]
	pub output_path: PathBuf,
}
