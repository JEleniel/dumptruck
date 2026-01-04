use std::path::PathBuf;

use clap::Parser;

use crate::analyze::AnalyzeError;

/// Arguments for the analyze command
#[derive(Parser, Debug)]
pub struct AnalyzeArgs {
	/// Path to input data file(s) to analyze
	/// Examples: data.csv, /path/to/dumps/data.csv
	/// If a directory is provided, all supported data files within will be processed
	/// Use the --recursive flag to include subdirectories
	#[arg()]
	pub input: PathBuf,

	/// Recursively search input directory for data files
	#[arg(short, long, default_value_t = false)]
	pub recursive: bool,

	/// Breach date in YYYYMMDD format
	/// This is used along with the target and an MD5 of the data to uniquely identify
	/// breaches in the database.
	/// If not provided, the current date will be used.
	#[arg(short, long, value_parser=Self::breach_date_is_valid, default_value_t=chrono::Utc::now().format("%Y%m%d").to_string())]
	pub date: String,

	/// Name of the target entity (e.g., "ExampleCorp")
	/// This is used along with the date and an MD5 of the data to uniquely identify
	/// breaches in the database.
	/// If not provided, the target will be set to "unknown".
	#[arg(short, long, default_value_t = String::from("Unknown"))]
	pub target: String,

	/// Output a JSON file of the results to the specified path.
	/// If not specified, results will be printed to the console in a human readable format.
	#[arg(short = 'o', long)]
	pub output: Option<PathBuf>,

	/// Enable generation of Vector embeddings
	#[arg(long, default_value_t = false)]
	pub enable_embeddings: bool,
}

impl AnalyzeArgs {
	/// Validate date format (YYYYMMDD)
	/// Note: This does not check for valid calendar dates (e.g., February 30)
	pub fn breach_date_is_valid(value: &str) -> Result<bool, AnalyzeError> {
		let date_regex = regex::Regex::new(r"[1-2]\d\d\d\d[0-1]\d[0-3]\d")?;
		Ok(date_regex.is_match(value))
	}
}
