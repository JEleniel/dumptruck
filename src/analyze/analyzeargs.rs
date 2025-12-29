use std::path::PathBuf;

use clap::Parser;

use crate::{analyze::AnalyzeError, datafile::DataFileType};

/// Arguments for the analyze command
#[derive(Parser, Debug)]
pub struct AnalyzeArgs {
	/// Path to input data file
	/// Examples: data.csv, /path/to/dumps/data.json
	#[arg(value_name = "INPUT_FILE")]
	pub input: PathBuf,

	/// Breach date in YYYYMMDD format
	/// This is used along with the target and an MD5 of the data to uniquely identify
	/// breaches in the database.
	/// If not provided, the current date will be used.
	#[arg(short, long, value_name = "DATE", value_parser=Self::breach_date_is_valid)]
	pub date: Option<String>,

	/// Name of the target entity (e.g., "ExampleCorp")
	/// This is used along with the date and an MD5 of the data to uniquely identify
	/// breaches in the database.
	/// If not provided, the target will be set to "unknown".
	#[arg(short, long, value_name = "TARGET")]
	pub target: Option<String>,

	/// Output file for results
	/// If not specified, results will be printed to the console.
	#[arg(short = 'o', long, value_name = "OUTPUT_FILE")]
	pub output: Option<PathBuf>,
	/// Output format for results
	/// If not specified, will be determined from output file extension or default to JSON.
	#[arg(long, value_enum, default_value = "json")]
	pub output_type: DataFileType,

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
