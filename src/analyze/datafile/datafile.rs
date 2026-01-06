use crate::{
	Hash,
	analyze::datafile::{DataFieldType, DataFileError, DataFileType},
};
use std::{
	fs::{self, File},
	io::{BufReader, Read},
	path::PathBuf,
};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug)]
pub struct DataFile {
	pub path: PathBuf,
	pub signature: String,
	pub file_type: DataFileType,
	pub column_types: Vec<DataFieldType>, // Field types based on header analysis
	pub data_types: Vec<DataFieldType>,   // Field types based on content analysis
	file: Option<File>,
}

impl DataFile {
	pub fn open(path: &PathBuf, working_path: Option<&PathBuf>) -> Result<Self, DataFileError> {
		let working_path = if let Some(p) = working_path {
			p
		} else {
			&std::env::temp_dir()
				.as_path()
				.join(std::env::var("CARGO_PKG_NAME")?)
		};
		let file_type = Self::detect_file_type(path)?;
		let signature = Hash::calculate_sha256(&mut BufReader::new(File::open(&working_path)?))?;

		Self::make_working_copy(path, working_path, &file_type)?;

		let new_datafile = Self {
			path: working_path.clone(),
			file_type,
			signature,
			column_types: Vec::new(),
			data_types: Vec::new(),
			file: None,
		};

		Ok(new_datafile)
	}

	pub fn get_reader(&self) -> Result<csv::Reader<File>, DataFileError> {
		let csv_reader = csv::Reader::from_path(&self.path)?;
		Ok(csv_reader)
	}

	fn detect_file_type(path: &PathBuf) -> Result<DataFileType, DataFileError> {
		let extension = if let Some(ext) = path.extension() {
			ext.to_string_lossy().to_lowercase()
		} else {
			String::new()
		};

		let ext_file_type = match extension.as_str() {
			"csv" => DataFileType::Csv,
			"tsv" => DataFileType::Tsv,
			"psv" | "pipe" => DataFileType::Psv,
			_ => return Err(DataFileError::UnsupportedFileType(extension)),
		};

		let file = File::open(path)?;
		let mut reader = BufReader::new(file);
		let mut buffer = [0; 4096];
		let bytes_read = reader.read(&mut buffer)?;
		let data = &buffer[..bytes_read];
		let new_lines = data.iter().filter(|&&b| b == b'\n').count();
		// Simple heuristic: count delimiters in the first chunk of data, the correct delimeter
		// should appear at least as many times as there are new lines (minumim 1 delimeter/line)
		if data.iter().filter(|&&b| b == b',').count() >= new_lines {
			DataFileType::Csv
		} else if data.iter().filter(|&&b| b == b'\t').count() >= new_lines {
			DataFileType::Tsv
		} else if data.iter().filter(|&&b| b == b'|').count() >= new_lines {
			DataFileType::Psv
		} else {
			return Err(DataFileError::UnsupportedFileType(
				"Extension and content do not match".to_string(),
			));
		};

		Ok(ext_file_type)
	}

	fn make_working_copy(
		source_path: &PathBuf,
		target_folder: &PathBuf,
		file_type: &DataFileType,
	) -> Result<PathBuf, DataFileError> {
		let file_name = Uuid::new_v4().to_string();
		let dest_path = target_folder.join(file_name).join(".csv");

		debug!(
			"Creating working copy of data file {:?} at {:?}",
			source_path, dest_path
		);

		// Convert to CSV if needed
		// CSV files are copied directly
		// Other files are converted line by line to avoid loading entire file into memory
		match file_type {
			DataFileType::Csv => {
				fs::copy(source_path, &dest_path)?;
			}
			DataFileType::Tsv => {
				let mut reader = csv::ReaderBuilder::new()
					.delimiter(b'\t')
					.from_path(source_path)?;
				let mut writer = csv::WriterBuilder::new()
					.delimiter(b',')
					.from_path(&dest_path)?;
				for result in reader.records() {
					let record = result?;
					writer.write_record(&record)?;
				}
			}
			DataFileType::Psv => {
				let mut reader = csv::ReaderBuilder::new()
					.delimiter(b'|')
					.from_path(source_path)?;
				let mut writer = csv::WriterBuilder::new()
					.delimiter(b',')
					.from_path(&dest_path)?;
				for result in reader.records() {
					let record = result?;
					writer.write_record(&record)?;
				}
			}
		}
		Ok(dest_path)
	}
}
