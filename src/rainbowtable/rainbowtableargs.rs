use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct RainbowTableArgs {
	/// Path to input text file(s) to generate rainbow tables from
	/// Examples: words.txt, /path/to/wordlists/words.txt, /path/to/textfiles/
	/// If a directory is provided, all supported text files within will be processed
	/// Use the --recursive flag to include subdirectories
	#[arg()]
	pub input: PathBuf,

	/// Recursively search input directory for text files
	#[arg(short, long, default_value_t = false)]
	pub recursive: bool,
}
