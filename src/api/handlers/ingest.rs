//! Ingest command handler.
//!
//! Coordinator module that orchestrates file ingestion, format detection,
//! and result formatting. Delegates to submodules for specific functionality.

mod context;
mod processors;
mod stats;

use std::path::Path;

use crate::{
	api::output::{
		CsvFormatter, IngestResult, JsonFormatter, JsonlFormatter, OutputFormatter, TextFormatter,
		write_output,
	},
	cli::{AnalyzeArgs, OutputFormat},
};

pub use context::setup_ingest_context;
pub use stats::IngestStats;

use context::IngestContext;
use processors::process_format;

/// Handle the ingest command
pub async fn ingest(args: AnalyzeArgs) -> Result<(), String> {
	if args.verbose >= 1 {
		eprintln!("[INFO] Starting ingest operation");
	}

	let files = args.resolve_input_files()?;
	if args.verbose >= 1 {
		eprintln!("[INFO] Found {} file(s) to process", files.len());
	}

	let ctx = setup_ingest_context(&args)?;
	let mut stats = IngestStats::default();

	for file_path in &files {
		process_single_file(&ctx, file_path, &args, &mut stats).await;
	}

	finalize_ingest(&args, &stats).await
}

// Note: `setup_ingest_context` is provided by the `context` submodule.

/// Process a single file through the ingest pipeline
async fn process_single_file(
	ctx: &IngestContext,
	file_path: &std::path::Path,
	args: &AnalyzeArgs,
	stats: &mut IngestStats,
) {
	if args.verbose >= 1 {
		eprintln!("[INFO] Processing file: {:?}", file_path);
	}

	let working_copy_path = match ctx.working_copy_mgr.create_working_copy(file_path) {
		Ok(path) => path,
		Err(e) => {
			let err_msg = format!("Failed to create working copy for {:?}: {}", file_path, e);
			if args.verbose >= 1 {
				eprintln!("[ERROR] {}", err_msg);
			}
			stats.errors.push(err_msg);
			return;
		}
	};

	if args.verbose >= 2 {
		eprintln!(
			"[DEBUG] Reading file contents from working copy: {:?}",
			working_copy_path
		);
	}

	let result =
		crate::analyze::safe_ingest::safe_read_file(&working_copy_path, args.verbose as u32).await;

	let (content, safety_analysis) = match result {
		Ok((c, _had_utf8_errors, analysis)) => (c, analysis),
		Err(e) => {
			let err_msg = format!("Failed to read file {:?}: {}", working_copy_path, e);
			if args.verbose >= 1 {
				eprintln!("[ERROR] {}", err_msg);
			}
			stats.errors.push(err_msg);
			return;
		}
	};

	if args.verbose >= 2 {
		eprintln!("[DEBUG] File read complete, size: {} bytes", content.len());
	}

	if safety_analysis.is_binary {
		if args.verbose >= 1 {
			eprintln!(
				"[WARN] Binary file detected in {:?} ({:.0}% confidence) - skipping",
				working_copy_path, safety_analysis.binary_confidence
			);
		}
		stats.errors.push(format!(
			"Cannot process file {:?}: Binary file detected ({:.0}% confidence)",
			working_copy_path, safety_analysis.binary_confidence
		));
		return;
	}

	let format_str = if let Some(fmt) = args.format {
		fmt.to_string()
	} else {
		detect_format_from_path(file_path)
	};

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Detected format: {}", format_str);
		eprintln!("[INFO] Parsing {} format file...", format_str);
	}

	process_format(&format_str, &content, file_path, args, stats).await;
}

// Processing functions are provided by the `processors` submodule; local duplicates removed.

/// Finalize ingest and format output
async fn finalize_ingest(args: &AnalyzeArgs, stats: &IngestStats) -> Result<(), String> {
	if args.verbose >= 1 {
		eprintln!(
			"[INFO] All files processed. Total rows: {}",
			stats.total_rows
		);
		eprintln!(
			"[INFO] Detection results: Unique addresses: {}, Hashed credentials: {}, Weak \
			 passwords: {}",
			stats.unique_addresses, stats.hashed_credentials, stats.weak_passwords
		);
	}

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Formatting output results...");
	}

	let result = IngestResult {
		rows_processed: stats.total_rows,
		unique_addresses: stats.unique_addresses,
		hashed_credentials_detected: stats.hashed_credentials,
		weak_passwords_found: stats.weak_passwords,
		breached_addresses: 0,
		pii_summary: Some(stats.pii_summary.clone()),
		detection_groups: stats.detection_groups.clone(),
		metadata: stats.metadata.clone(),
		errors: stats.errors.clone(),
	};

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Creating {:?} formatter...", args.output_format);
	}

	let formatter: Box<dyn OutputFormatter> = match args.output_format {
		OutputFormat::Json => Box::new(JsonFormatter),
		OutputFormat::Csv => Box::new(CsvFormatter),
		OutputFormat::Text => Box::new(TextFormatter),
		OutputFormat::Jsonl => Box::new(JsonlFormatter),
	};

	if args.verbose >= 2 {
		eprintln!(
			"[DEBUG] Formatting {} results to output format...",
			stats.total_rows
		);
	}

	let formatted = formatter
		.format(&result)
		.map_err(|e| format!("Failed to format output: {}", e))?;

	if args.verbose >= 2 {
		eprintln!(
			"[DEBUG] Output formatting complete: {} bytes",
			formatted.len()
		);
	}

	if args.verbose >= 1 {
		let target = if let Some(path) = args.output.as_deref() {
			format!("{:?}", path)
		} else {
			"stdout".to_string()
		};
		eprintln!("[INFO] Writing results to: {}", target);
	}

	let output_path = args.output.as_deref();
	write_output(&formatted, output_path).map_err(|e| format!("Failed to write output: {}", e))?;

	if args.verbose >= 2 {
		eprintln!("[DEBUG] Output write complete");
	}

	if args.verbose >= 1 {
		eprintln!("[INFO] Ingest operation completed successfully");
	}

	Ok(())
}

/// Detect format from file path
pub(crate) fn detect_format_from_path(path: &Path) -> String {
	path.extension()
		.and_then(|ext| ext.to_str())
		.unwrap_or("csv")
		.to_lowercase()
}
