use std::{
	fs::File,
	io::{BufRead, BufReader},
	path::PathBuf,
};

use thiserror::Error;
use tracing::{info, warn};

use crate::{
	Files, Hash, configuration::Configuration, database::Database, rainbowtable::RainbowTableArgs,
};

pub struct RainbowTable {}

impl RainbowTable {
	pub async fn generate(
		configuration: &Configuration,
		args: RainbowTableArgs,
	) -> Result<(), RainbowTableError> {
		if configuration.services.enable_embeddings {
			warn!(
				"Ignoring the invalid 'enable_embeddings' configuration option for rainbow table generation."
			);
		}
		if configuration.services.ollama.is_some() {
			warn!(
				"Ignoring the invalid 'ollama' configuration option for rainbow table generation."
			);
		}

		let mut pending_files: Vec<PathBuf> = Vec::new();
		if args.input.is_dir() {
			pending_files
				.append(&mut Files::get_files_from_path(args.input, args.recursive).await?);
		} else {
			pending_files.push(args.input);
		}

		// Run safety checks before attempting to process files
		for file in &pending_files {
			// Warn if the file does not have a .txt extension
			if let Some(ext) = file.extension() {
				if ext != "txt" {
					warn!(
						"File does not have a `.txt` extension. Processing will continue nonetheless: {:?}",
						file
					);
				}
			}

			// Validate that the file is proper UTF-8 text
			if !Files::validate_utf8(&file)? {
				warn!("Skipping non-UTF8 file: {:?}", file);
				continue;
			}

			// Check if the file is binary
			if Files::is_binary(&file)? {
				warn!("Skipping binary file: {:?}", file);
				continue;
			}
		}

		let db = Database::open(&configuration.paths.db_path).await?;
		let mut counter: usize = 0;
		for file in &pending_files {
			info!("Processing file: {:?}", file);

			let file = File::open(file)?;
			let mut reader = BufReader::new(file);

			let mut buf = String::new();

			// Read line by line to allow processing of large files
			while reader.read_line(&mut buf)? > 0 {
				counter += 1;

				let md5 = Hash::calculate_md5(&mut BufReader::new(buf.as_bytes()))?;
				let sha1 = Hash::calculate_sha1(&mut BufReader::new(buf.as_bytes()))?;
				let sha256 = Hash::calculate_sha256(&mut BufReader::new(buf.as_bytes()))?;
				let ntlm = Hash::calculate_ntlm(&mut BufReader::new(buf.as_bytes()))?;
				let lm = Hash::calculate_lm(buf.as_str());
				let mysqlold = Hash::calculate_mysqlold(buf.as_str());

				db.rainbowtable.add("MD5", &md5.as_str()).await?;
				db.rainbowtable.add("SHA1", &sha1.as_str()).await?;
				db.rainbowtable.add("SHA256", &sha256.as_str()).await?;
				db.rainbowtable.add("NTLM", &ntlm.as_str()).await?;
				db.rainbowtable.add("LM", &lm.as_str()).await?;
				db.rainbowtable.add("MySQL_Old", &mysqlold.as_str()).await?;
			}
		}

		info!(
			"Rainbow table generation complete. Processed {} lines from {} files.",
			counter,
			pending_files.len()
		);
		Ok(())
	}
}

#[derive(Debug, Error)]
pub enum RainbowTableError {
	#[error("IO Error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Files Error: {0}")]
	FilesError(#[from] crate::FilesError),
	#[error("UTF-8 Validation Error")]
	Utf8Error,
	#[error("Hash Calculation Error: {0}")]
	HashError(#[from] crate::HashError),
	#[error("Database Error: {0}")]
	DatabaseError(#[from] crate::database::DatabaseError),
}
