//! Ingest operation context and initialization.
//!
//! Handles setup of working directories and configuration for ingest operations.

use crate::cli::AnalyzeArgs;
use crate::data::working_copy::WorkingCopyManager;

/// Configuration and manager setup for ingest operations
pub struct IngestContext {
	pub working_copy_mgr: WorkingCopyManager,
}

/// Set up ingest context (config and working directory)
pub fn setup_ingest_context(args: &AnalyzeArgs) -> Result<IngestContext, String> {
	let config_path = args.config.as_ref().and_then(|p| p.to_str());
	let config = crate::core::config::Config::load_with_search(config_path, args.verbose >= 2)
		.map_err(|e| format!("Failed to load configuration: {}", e))?;

	let working_dir = if let Some(dir) = &args.working_dir {
		dir.clone()
	} else if let Some(config_path) = &config.working_directory.path {
		std::path::PathBuf::from(config_path)
	} else {
		std::path::PathBuf::from("/tmp/dumptruck")
	};

	let working_copy_mgr =
		WorkingCopyManager::new(&working_dir, args.verify_noexec, args.verbose as u32)
			.map_err(|e| format!("Failed to initialize working directory: {}", e))?;

	if args.verbose >= 1 {
		eprintln!("[INFO] Working directory initialized: {:?}", working_dir);
	}

	Ok(IngestContext { working_copy_mgr })
}
