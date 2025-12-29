use std::path::PathBuf;

use clap::Parser;

/// Arguments for the seed command
#[derive(Parser, Debug)]
pub struct SeedArgs {
	/// Path to folder containing data files to create seed from
	#[arg(value_name = "SOURCE_FOLDER")]
	pub folder: PathBuf,

	/// Output path for seed database (default: ./data/)
	/// The database file will be named "seed.db"
	#[arg(short, long, value_name = "OUTPUT_PATH")]
	pub output: Option<PathBuf>,
}
