//! Data file analysis module
mod analyzeargs;
mod analyzeerror;
mod datafile;
mod detection;
mod normalization;

use std::{collections::BTreeMap, path::PathBuf};

use crate::{
	analyze::datafile::{DataFieldType, DataFile, DataFileError, DataFileType},
	configuration::Configuration,
	database::Database,
	util::get_files_from_path,
};
pub use analyzeargs::*;
pub use analyzeerror::*;
use tokio::fs;
use tracing::{debug, warn};

pub async fn analyze(configuration: Configuration, args: AnalyzeArgs) -> Result<(), AnalyzeError> {
	let mut pending_files = get_files_from_path(args.input, args.recursive).await?;

	Ok(())
}

fn analyze_file_structure(path: &PathBuf) -> Result<DataFileType, DataFileError> {
	if path.is_dir() {
		return Err(DataFileError::IsDirectory);
	}

	if fs::metadata(path).map(|meta| meta.len())? == 0 {
		return Err(DataFileError::EmptyFile);
	}

	let mut file_types: BTreeMap<DataFileType, f32> = BTreeMap::new();
	for e in DataFileType::iter() {
		file_types.insert(e, 0.0);
	}

	// Check file extension first for quick hints, weighted 50% except for txt files
	if let Some(ext) = path.extension() {
		match ext.to_str()?.to_lowercase().as_str() {
			"txt" => {
				file_types[DataFileType::Rainbow] += 1.0;
				return Ok(DataFileType::Rainbow);
			}
			"csv" => file_types[DataFileType::Csv] += 0.5,
			"tsv" | "tab" => file_types[DataFileType::Tsv] += 0.5,
			"psv" | "pipe" => file_types[DataFileType::Psv] += 0.5,
			_ => {}
		}
	}

	let mut binary_confidence: f64 = 0.0;

	// Use the first 4k to analyze content
	let mut data = [0u8; 4096];
	let mut file = fs::File::open(path)?;
	file.read_buf(&mut data)?;

	// Check UTF-8 validity (Unsafe to process; only process UTF-8 files)
	let is_valid_utf8 = std::str::from_utf8(&data).is_ok();
	if !is_valid_utf8 {
		warn!("File is not valid UTF-8 (contains invalid byte sequences)");
		return Err(DataFileError::Utf8Error);
	}

	// Check for ELF magic header (0x7F 'E' 'L' 'F')
	// We don't risk processing anything that looks like a binary executable
	if data.len() >= 4 && data[0] == 0x7F && data[1] == b'E' && data[2] == b'L' && data[3] == b'F' {
		warn!("{0} appears to be ELF binary (Linux/Unix)", path.display());
		return Err(DataFileError::BinaryFile);
	}

	// Check for PE (Windows) magic header (0x4D 0x5A = 'MZ')
	// We don't risk processing anything that looks like a binary executable
	if data.len() >= 2 && data[0] == 0x4D && data[1] == 0x5A {
		warn!("{0} appears to be Windows PE binary", path.display());
		return Err(DataFileError::BinaryFile);
	}

	// Check for Mach-O magic header (0xFE 0xED 0xFA / 0xCE 0xFA = macOS binary)
	// We don't risk processing anything that looks like a binary executable
	if data.len() >= 2
		&& ((data[0] == 0xFE && data[1] == 0xED) || (data[0] == 0xCE && data[1] == 0xFA))
	{
		warn!("{0} appears to be Mach-O binary (macOS)", path.display());
		return Err(DataFileError::BinaryFile);
	}

	// Check for null bytes - 50% confidence)
	if data.contains(&0) {
		binary_confidence += 50.0;
		debug!("File contains null bytes - 50% chance binary format");
	}

	// Check for high ASCII characters - 30% confidence if >30% of bytes are in 0x80-0xFF range
	let high_ascii = data.iter().filter(|&&b| b > 127 && b <= 255).count();

	let high_ascii_ratio = high_ascii as f64 / 4096.0;

	if high_ascii_ratio > 0.30 {
		binary_confidence += 30.0;
		debug!(
			"File has {:.1}% high ASCII bytes - 30% chance binary format",
			high_ascii_ratio * 100.0
		);
	}

	if binary_confidence >= 80.0 {
		warn!(
			"{0} appears to be binary format (confidence: {1:.1}%)",
			path.display(),
			binary_confidence
		);
		return Err(DataFileError::BinaryFile);
	}

	// Validate UTF-8 encoding
	let text = std::str::from_utf8(&data)?;

	// Check for at least a header and one row of data
	if text.lines().count() < 2 {
		debug!("File has less than two lines of text");
		return Err(DataFileError::NotEnoughData);
	}

	// Check if it looks like CSV
	let looks_like_csv =
		text.contains('\n') && text.find(',').iter().count() >= text.find('\n').iter().count(); // At least one comma per line
	if looks_like_csv {
		file_types[DataFileType::Csv] += 0.3;
	}

	let looks_like_tsv =
		text.contains('\n') && text.find('\t').iter().count() >= text.find('\n').iter().count(); // At least one tab per line
	if looks_like_tsv {
		file_types[DataFileType::Tsv] += 0.3;
	}

	let looks_like_psv =
		text.contains('\n') && text.find('|').iter().count() >= text.find('\n').iter().count(); // At least one pipe per line
	if looks_like_psv {
		file_types[DataFileType::Psv] += 0.3;
	}

	let file_type = file_types.iter().max_by(|a, b| a.compare(b))?.0;
	if file_types[file_type] == 0.0 {
		debug!("Unable to determine file type based on content analysis");
		return Err(DataFileError::UnrecognizedFormat);
	}
	debug!(
		"File type determined as {:?} based on content analysis",
		file_type
	);

	Ok(*file_type)
}

fn analyze_file_header(data_file: &mut DataFile) -> Result<(), DataFileError> {
	let csv_reader = data_file.get_reader()?;
	let headers = csv_reader.headers()?;
	for header in headers.iter() {
		let field_type = DataFieldType::from_header_name(header);
		data_file.column_types.push(field_type);
	}
	Ok(())
}

fn analyze_file_content(db: &Database, data_file: &mut DataFile) -> Result<(), DataFileError> {
	let reader = data_file.get_reader()?;
	let mut field_type_counters: Vec<BTreeMap<DataFieldType, u32>> = Vec::new();
	for _ in 0..data_file.column_types.len() {
		let mut counter: BTreeMap<DataFieldType, u32> = BTreeMap::new();
		for field_type in DataFieldType::iter() {
			counter.insert(field_type, 0);
		}
		field_type_counters.push(counter);
	}

	for record in reader.into_records() {
		let record = record?;
	}

	Ok(())
}
