use clap::Parser;

/// Arguments for the status command
#[derive(Parser, Debug)]
pub struct StatusArgs {
	/// URL of the DumpTruck server
	/// Defaults to localhost if not provided
	#[arg(long, value_name = "URL")]
	pub url: Option<Url>,
}
