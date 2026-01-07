fn main() {
	match aurora_validator::validate_repo_default() {
		Ok(()) => {
			println!("AURORA model is valid");
		}
		Err(err) => {
			eprintln!("AURORA model validation failed: {err}");
			std::process::exit(1);
		}
	}
}
