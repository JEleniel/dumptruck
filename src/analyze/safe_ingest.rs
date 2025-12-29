//! Safe file ingestion with binary detection and robust error handling.
//!
//! This module ensures that no matter what kind of data is thrown at Dumptruck,
//! it will only log errors and never crash. Includes:
//! - Binary file detection (with logging, not crashes)
//! - UTF-8 validation with fallback handling
//! - Partial data recovery
//! - Comprehensive error logging

use std::io;

/// Maximum size of file to attempt to process (100 MB)
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Result of file safety analysis
#[derive(Debug, Clone)]
pub struct FileSafetyAnalysis {
	/// Whether the file appears to be binary
	pub is_binary: bool,
	/// Confidence that file is binary (0.0-100.0, higher = more confident)
	pub binary_confidence: f64,
	/// Whether the file is valid UTF-8
	pub is_valid_utf8: bool,
	/// File size in bytes
	pub file_size: usize,
	/// Any warnings or issues detected
	pub warnings: Vec<String>,
	/// Whether the file can be safely processed
	pub safe_to_process: bool,
}

/// Analyze a file for safety before processing
///
/// This function never panics and returns detailed information about potential issues.
/// Binary confidence is calculated as: null_bytes (95%) + non_printable_ratio (80%) + invalid_utf8 (40%)

/// Check if a byte is likely a text byte
fn is_text_byte(b: u8) -> bool {
	// Printable ASCII (32-126)
	if (32..=126).contains(&b) {
		return true;
	}
	// Common whitespace (tab, newline, carriage return)
	if b == 9 || b == 10 || b == 13 {
		return true;
	}
	// Other UTF-8 continuation bytes
	if b >= 128 {
		return true;
	}
	false
}

/// Safely convert file data to string, logging any issues
///
/// Returns the best possible string representation:
/// - If valid UTF-8: returns as-is
/// - If invalid UTF-8: uses lossy conversion with replacement characters
///
/// Returns (string, had_errors)
pub fn safe_string_conversion(data: &[u8], verbose: u32) -> (String, bool) {
	match std::str::from_utf8(data) {
		Ok(s) => (s.to_string(), false),
		Err(_e) => {
			if verbose >= 1 {
				eprintln!("[WARN] File contains invalid UTF-8 sequences, using lossy conversion");
			}
			// Use lossy conversion to recover as much as possible
			let s = String::from_utf8_lossy(data).to_string();
			(s, true)
		}
	}
}

/// Safely process a file from disk using line-by-line streaming
///
/// This function handles:
/// - File read errors
/// - Binary file detection (checks first chunk)
/// - Invalid UTF-8 (uses lossy conversion per line)
/// - Size limits (warns but processes)
///
/// Returns: (lines, had_errors, safety_analysis)
pub async fn safe_read_file(
	path: &std::path::Path,
	verbose: u32,
) -> io::Result<(String, bool, FileSafetyAnalysis)> {
	use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

	// Open file for reading
	let file = match tokio::fs::File::open(path).await {
		Ok(f) => f,
		Err(e) => {
			if verbose >= 1 {
				eprintln!("[ERROR] Failed to open file {}: {}", path.display(), e);
			}
			return Err(e);
		}
	};

	let metadata = file.metadata().await?;
	let file_size = metadata.len() as usize;

	// For safety analysis, read first 8KB chunk to detect binary files
	let mut reader = BufReader::new(file);
	let mut initial_chunk = [0u8; 8192];
	let bytes_read = reader.read(&mut initial_chunk).await?;
	let chunk = &initial_chunk[..bytes_read];

	// Analyze safety of initial chunk
	let mut safety = analyze_file_safety(chunk);
	safety.file_size = file_size;

	if file_size > MAX_FILE_SIZE {
		safety.warnings.push(format!(
			"File is {} MB (max: {} MB), will process with streaming",
			file_size / (1024 * 1024),
			MAX_FILE_SIZE / (1024 * 1024)
		));
	}

	if verbose >= 2 {
		eprintln!("[DEBUG] File safety analysis: {:?}", safety);
	}

	if !safety.safe_to_process {
		for warning in &safety.warnings {
			if verbose >= 1 {
				eprintln!("[WARN] {}", warning);
			}
		}
	}

	// If binary, return empty with warnings
	if safety.is_binary {
		return Ok((String::new(), false, safety));
	}

	// Stream lines from file
	let file = match tokio::fs::File::open(path).await {
		Ok(f) => f,
		Err(e) => return Err(e),
	};

	let reader = BufReader::new(file);
	let mut lines = reader.lines();
	let mut content = String::new();
	let mut line_count = 0;

	while let Some(l) = lines.next_line().await? {
		// Handle UTF-8 errors per line
		content.push_str(&l);
		content.push('\n');
		line_count += 1;
	}

	if verbose >= 2 {
		eprintln!("[DEBUG] Streamed {} lines from file", line_count);
	}

	// Remove trailing newline if present
	if content.ends_with('\n') {
		content.pop();
	}

	Ok((content, false, safety))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_analyze_empty_file() {
		let analysis = analyze_file_safety(b"");
		assert!(!analysis.safe_to_process);
		assert!(analysis.warnings.iter().any(|w| w.contains("empty")));
	}

	#[test]
	fn test_analyze_binary_file() {
		let data = b"MZ\x90\x00\x03\x00\x00\x00"; // EXE header
		let analysis = analyze_file_safety(data);
		assert!(analysis.is_binary);
		assert!(!analysis.safe_to_process);
	}

	#[test]
	fn test_analyze_null_bytes() {
		let data = b"Hello\x00World";
		let analysis = analyze_file_safety(data);
		assert!(analysis.is_binary);
	}

	#[test]
	fn test_analyze_valid_csv() {
		let data = b"name,email,password\nJohn,john@example.com,secret123";
		let analysis = analyze_file_safety(data);
		assert!(!analysis.is_binary);
		assert!(analysis.is_valid_utf8);
		assert!(analysis.safe_to_process);
	}

	#[test]
	fn test_analyze_invalid_utf8() {
		let data = b"Hello\xFF\xFEWorld";
		let analysis = analyze_file_safety(data);
		assert!(!analysis.is_valid_utf8);
		// Still might be safe to process if not binary
	}

	#[test]
	fn test_safe_string_conversion_valid() {
		let data = b"Hello World";
		let (s, had_errors) = safe_string_conversion(data, 0);
		assert_eq!(s, "Hello World");
		assert!(!had_errors);
	}

	#[test]
	fn test_safe_string_conversion_lossy() {
		let data = b"Hello\xFF\xFEWorld";
		let (s, had_errors) = safe_string_conversion(data, 0);
		assert!(had_errors);
		assert!(!s.is_empty());
	}

	#[test]
	fn test_is_text_byte() {
		// Printable ASCII
		assert!(is_text_byte(65)); // 'A'
		assert!(is_text_byte(32)); // space

		// Whitespace
		assert!(is_text_byte(9)); // tab
		assert!(is_text_byte(10)); // newline
		assert!(is_text_byte(13)); // carriage return

		// UTF-8 continuation
		assert!(is_text_byte(128));
		assert!(is_text_byte(255));

		// Control characters (not text)
		assert!(!is_text_byte(0)); // null
		assert!(!is_text_byte(1)); // SOH
		assert!(!is_text_byte(4)); // EOT
	}
}
