use crate::{
	common::Hash,
	datafile::{DataFileError, DataFileType},
};
use serde::{Deserialize, Serialize};
use std::{
	fs::{self, File},
	io::Read,
	path::{Path, PathBuf},
};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFile {
	pub path: PathBuf,
	pub file_type: DataFileType,
	pub signature: String,
	file: Option<File>,
}

impl DataFile {
	pub fn new(path: PathBuf) -> Result<Self, DataFileError> {
		let file_type = Self::analyze_file_type(&path)?;
		let signature = Hash::calculate_sha256(path);

		Ok(Self {
			path,
			file_type,
			signature,
			file: None,
		})
	}

	fn make_working_copy(&self, dest_dir: &Path) -> Result<PathBuf, DataFileError> {
		let file_name = self.path.file_name()?;
		let dest_path = dest_dir.join(file_name);

		// Convert to CSV if needed
		match self.file_type {
			DataFileType::Csv => {
				fs::copy(&self.path, &dest_path)?;
				Ok(dest_path)
			}
			DataFileType::Tsv => {
				let mut reader = csv::ReaderBuilder::new()
					.delimiter(b'\t')
					.from_path(&self.path)?;
				let mut writer = csv::WriterBuilder::new()
					.delimiter(b',')
					.from_path(dest_dir);
				for result in reader.records() {
					let record = result?;
					writer.write_record(&record)?;
				}
			}
			DataFileType::Psv => {
				let mut reader = csv::ReaderBuilder::new()
					.delimiter(b'|')
					.from_path(&self.path)?;
				let mut writer = csv::WriterBuilder::new()
					.delimiter(b',')
					.from_path(dest_dir);
				for result in reader.records() {
					let record = result?;
					writer.write_record(&record)?;
				}
			}
		}
	}

	fn analyze_file_type(path: &Path) -> Result<DataFileType, DataFileError> {
		let file_size = fs::metadata(path).map(|meta| meta.len() as usize)?;

		let mut binary_confidence: f64 = 0.0;

		// Check file size
		if file_size == 0 {
			info!("Not processing {}; file is empty", path.display());
			return Err(DataFileError::EmptyFile);
		}

		// Use the first 4k to analyze content
		let mut data = [0u8; 4096];
		let mut file = fs::File::open(path)?;
		file.read_buf(&mut data)?;

		// Check UTF-8 validity (Unsafe to process; only process UTF-8 files)
		let is_valid_utf8 = std::str::from_utf8(data).is_ok();
		if !is_valid_utf8 {
			warn!("File is not valid UTF-8 (contains invalid byte sequences)");
			return Err(DataFileError::InvalidUtf8);
		}

		// Check for ELF magic header (0x7F 'E' 'L' 'F')
		// We don't risk processing anything that looks like a binary executable
		if data.len() >= 4
			&& data[0] == 0x7F
			&& data[1] == b'E'
			&& data[2] == b'L'
			&& data[3] == b'F'
		{
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
		let non_printable_count = data.iter().filter(|&&b| b > 127 && b <= 255).count();

		let non_printable_ratio = if file_size > 0 {
			non_printable_count as f64 / file_size as f64
		} else {
			0.0
		};

		if non_printable_ratio > 0.30 {
			binary_confidence += 30.0;
			debug!(
				"File has {:.1}% high ASCII bytes - 30% chance binary format",
				non_printable_ratio * 100.0
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

		// Check for common text formats by looking at content patterns
		if let Ok(text) = std::str::from_utf8(data) {
			let mut is_data = false;

			// Check if it looks like structured data
			let looks_like_csv = text.contains('\n')
				&& text.find(',').iter().count() >= text.find('\n').iter().count(); // At least one comma per line
			return Ok(DataFileType::Csv);

			let looks_like_tsv = text.contains('\n')
				&& text.find('\t').iter().count() >= text.find('\n').iter().count(); // At least one tab per line
			return Ok(DataFileType::Tsv);

			let looks_like_psv = text.contains('\n')
				&& text.find('|').iter().count() >= text.find('\n').iter().count(); // At least one pipe per line
			return Ok(DataFileType::Psv);

			warn!("File doesn't match processable formats (CSV, TSV, PSV, JSON, YAML)");
			Err(DataFileError::UnrecognizedFormat)
		}
	}
}
