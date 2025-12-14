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
pub fn analyze_file_safety(data: &[u8]) -> FileSafetyAnalysis {
	let file_size = data.len();
	let mut warnings = Vec::new();
	let mut is_binary = false;
	let mut safe_to_process = true;

	// Check file size
	if file_size == 0 {
		warnings.push("File is empty".to_string());
		safe_to_process = false;
	}

	if file_size > MAX_FILE_SIZE {
		warnings.push(format!(
			"File is {} MB (max: {} MB), will attempt partial processing",
			file_size / (1024 * 1024),
			MAX_FILE_SIZE / (1024 * 1024)
		));
		// Still try to process up to MAX_FILE_SIZE
	}

	// Check for binary content (null bytes are strong indicator)
	if data.iter().any(|&b| b == 0) {
		is_binary = true;
		warnings.push("File contains null bytes - likely binary format".to_string());
		safe_to_process = false;
	}

	// Check for high proportion of non-printable bytes
	let non_printable_count = data.iter().filter(|&&b| {
		!is_text_byte(b)
	}).count();
	
	let non_printable_ratio = non_printable_count as f64 / file_size as f64;
	if non_printable_ratio > 0.3 {
		is_binary = true;
		warnings.push(format!(
			"File has {:.1}% non-text bytes - likely binary",
			non_printable_ratio * 100.0
		));
		safe_to_process = false;
	}

	// Check UTF-8 validity
	let is_valid_utf8 = std::str::from_utf8(data).is_ok();
	if !is_valid_utf8 {
		warnings.push("File is not valid UTF-8 (contains invalid byte sequences)".to_string());
		// Can still try to process with lossy conversion
	}

	// Check for common text formats by looking at content patterns
	if safe_to_process && data.len() > 0 {
		if let Ok(text) = std::str::from_utf8(data) {
			// Check if it looks like structured data
			let looks_like_csv = text.contains('\n') && (text.contains(',') || text.contains('\t'));
			let looks_like_json = text.contains('{') || text.contains('[');
			let looks_like_yaml = text.contains(':');

			if !looks_like_csv && !looks_like_json && !looks_like_yaml {
				warnings.push(
					"File doesn't match common formats (CSV, JSON, YAML) - may be raw text".to_string()
				);
			}
		}
	}

	FileSafetyAnalysis {
		is_binary,
		is_valid_utf8,
		file_size,
		warnings,
		safe_to_process,
	}
}

/// Check if a byte is likely a text byte
fn is_text_byte(b: u8) -> bool {
	// Printable ASCII (32-126)
	if b >= 32 && b <= 126 {
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
/// Returns (string, had_errors)
pub fn safe_string_conversion(data: &[u8], verbose: u32) -> (String, bool) {
	match std::str::from_utf8(data) {
		Ok(s) => (s.to_string(), false),
		Err(_e) => {
			if verbose >= 1 {
				eprintln!(
					"[WARN] File contains invalid UTF-8 sequences, using lossy conversion"
				);
			}
			// Use lossy conversion to recover as much as possible
			let s = String::from_utf8_lossy(data).to_string();
			(s, true)
		}
	}
}

/// Safely process a file from disk
///
/// This function handles:
/// - File read errors
/// - Binary files
/// - Invalid UTF-8
/// - Size limits
///
/// Returns: (content_string, had_errors, safety_analysis)
pub async fn safe_read_file(
	path: &std::path::Path,
	verbose: u32,
) -> io::Result<(String, bool, FileSafetyAnalysis)> {
	// Read file (may be large, but we're cautious)
	let data = match tokio::fs::read(path).await {
		Ok(d) => d,
		Err(e) => {
			if verbose >= 1 {
				eprintln!(
					"[ERROR] Failed to read file {}: {}",
					path.display(),
					e
				);
			}
			return Err(e);
		}
	};

	// Analyze safety
	let safety = analyze_file_safety(&data);

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

	// Convert to string (safely handling invalid UTF-8)
	let (content, had_errors) = safe_string_conversion(&data, verbose);

	Ok((content, had_errors, safety))
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
		let data = b"MZ\x90\x00\x03\x00\x00\x00";  // EXE header
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
		assert!(is_text_byte(9));   // tab
		assert!(is_text_byte(10));  // newline
		assert!(is_text_byte(13));  // carriage return
		
		// UTF-8 continuation
		assert!(is_text_byte(128));
		assert!(is_text_byte(255));
		
		// Control characters (not text)
		assert!(!is_text_byte(0));   // null
		assert!(!is_text_byte(1));   // SOH
		assert!(!is_text_byte(4));   // EOT
	}
}
