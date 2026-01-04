mod fingerprint;
mod hash;

pub use fingerprint::*;
pub use hash::*;

use crate::analyze::AnalyzeError;
use std::path::PathBuf;

pub async fn get_files_from_path(
	path: PathBuf,
	recursive: bool,
) -> Result<Vec<PathBuf>, AnalyzeError> {
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
