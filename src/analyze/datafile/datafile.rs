use crate::{
	analyze::datafile::{DataFieldType, DataFileError, DataFileType},
	common::Hash,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::{
	fs::{self, File},
	io::BufReader,
};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFile {
	pub path: PathBuf,
	pub file_type: DataFileType,
	pub signature: String,
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

		let working_file = Self::make_working_copy(&path, &working_path)?;

		let file_type = Self::analyze_file_structure(&working_file)?;
		let signature = Hash::calculate_sha256(&mut BufReader::new(File::open(&working_file)?))?;

		let new_datafile = Self {
			path: working_file,
			file_type,
			signature,
			column_types: Vec::new(),
			data_types: Vec::new(),
			file: None,
		};

		Ok(new_datafile)
	}

	fn make_working_copy(
		source: &PathBuf,
		target_folder: &PathBuf,
	) -> Result<PathBuf, DataFileError> {
		let file_type = Self::analyze_file_structure(source)?;

		let file_name = Uuid::new_v4().to_string().join(source.extension()?);
		let dest_path = target_folder.join(file_name);

		debug!(
			"Creating working copy of data file {:?} at {:?}",
			source,
			dest_path.as_path()
		);

		// Convert to CSV if needed
		match file_type {
			DataFileType::Csv | DataFileType::Rainbow => {
				fs::copy(source, &dest_path)?;
			}
			DataFileType::Tsv => {
				let mut reader = csv::ReaderBuilder::new()
					.delimiter(b'\t')
					.from_path(source)?;
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
					.from_path(source)?;
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

	pub fn get_reader(&self) -> Result<csv::Reader<_>, DataFileError> {
		let csv_reader = csv::Reader::from_path(&self.path)?;
		Ok(csv_reader)
	}
}
