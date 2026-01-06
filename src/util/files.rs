use std::path::PathBuf;

use thiserror::Error;

pub struct Files {}

impl Files {
	/// (Recursively) get all files from a given path
	pub async fn get_files_from_path(
		path: PathBuf,
		recursive: bool,
	) -> Result<Vec<PathBuf>, FilesError> {
		let mut files: Vec<PathBuf> = Vec::new();

		if path.is_dir() {
			let mut pending_folders: Vec<PathBuf> = vec![path];
			while let Some(folder) = pending_folders.pop() {
				let mut dir_entries = tokio::fs::read_dir(&folder).await?;
				while let Some(entry) = dir_entries.next_entry().await? {
					let path = entry.path();
					let metadata = entry.metadata().await?;
					if metadata.is_dir() && recursive {
						pending_folders.push(path);
					} else if metadata.is_file() {
						files.push(path);
					}
				}
			}
		} else {
			files.push(path);
		}

		Ok(files)
	}
}

#[derive(Debug, Error)]
pub enum FilesError {
	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),
}
