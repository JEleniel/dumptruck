use std::path::PathBuf;

use clap::Parser;

/// Arguments for the import-db command
#[derive(Parser, Debug)]
pub struct ImportArgs {
	/// Input file from database export (SQLite format)
	#[arg(short, long, value_name = "INPUT_FILE")]
	pub input: PathBuf,
}
