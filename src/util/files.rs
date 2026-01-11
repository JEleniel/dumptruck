use std::{
	fs::File,
	io::{BufReader, Read},
	path::PathBuf,
};

use goblin::{elf::Elf, mach::Mach, pe::PE};
use thiserror::Error;

pub struct Files {}

impl Files {
	const BINARY_EXTENSIONS: [&str; 219] = [
		"exe", "dll", "sys", "com", "cpl", "scr", "ocx", "drv", "efi", "mui", "pyd", "lib", "tlb",
		"ax", "bin", "run", "out", "elf", "so", "a", "o", "ko", "mod", "vmlinuz", "dylib",
		"bundle", "kext", "app", "macho", "jar", "war", "ear", "apk", "aab", "ipa", "xap", "msix",
		"appx", "deb", "rpm", "pkg", "msi", "cab", "dmg", "img", "iso", "udf", "zip", "rar", "7z",
		"tar", "gz", "bz2", "xz", "lz", "lzma", "zst", "ar", "cpio", "doc", "docx", "xls", "xlsx",
		"ppt", "pptx", "rtf", "odt", "ods", "odp", "pages", "numbers", "key", "pdf", "xps", "djvu",
		"epub", "mobi", "chm", "hlp", "txt", "md", "rst", "csv", "tsv", "log", "ini", "cfg",
		"conf", "yaml", "yml", "json", "toml", "xml", "html", "htm", "xhtml", "php", "asp", "aspx",
		"jsp", "js", "ts", "css", "ps1", "psm1", "psd1", "sh", "bash", "zsh", "ksh", "fish", "cmd",
		"bat", "vbs", "jsf", "hta", "py", "pyc", "pyo", "rb", "pl", "lua", "go", "rs", "c", "cpp",
		"h", "hpp", "cs", "java", "swift", "kt", "scala", "jpg", "jpeg", "jpe", "png", "gif",
		"bmp", "tif", "tiff", "webp", "heic", "heif", "ico", "cur", "psd", "ai", "eps", "raw",
		"nef", "cr2", "arw", "mp3", "aac", "m4a", "wav", "flac", "ogg", "opus", "wma", "mid",
		"midi", "mp4", "m4v", "mov", "avi", "mkv", "webm", "wmv", "flv", "3gp", "3g2", "ts", "mts",
		"m2ts", "ttf", "otf", "woff", "woff2", "eot", "db", "sqlite", "sqlite3", "mdb", "accdb",
		"ldf", "ndf", "bak", "old", "tmp", "swp", "lock", "part", "fw", "rom", "hex", "srec",
		"ihex", "cap", "fd", "bios", "uefi", "pcap", "pcapng", "etl", "evtx", "dmp", "core", "pem",
		"crt", "cer", "der", "key", "pfx", "p12", "csr", "jks", "keystore",
	];

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

	/// Validate that a file contains valid UTF-8
	pub fn validate_utf8(path: &PathBuf) -> Result<bool, FilesError> {
		let file = File::open(path)?;
		let mut reader = BufReader::new(file);

		let mut buffer: [u8; 4096] = [0; 4096];
		let mut carry: Vec<u8> = Vec::new();
		loop {
			let n: usize = reader.read(&mut buffer)?;
			if n == 0 {
				break;
			}

			carry.extend_from_slice(&buffer[..n]);

			match str::from_utf8(&carry) {
				Ok(_) => carry.clear(),
				Err(e) => {
					if e.error_len().is_some() {
						return Ok(false);
					}

					// Incomplete codepoint at end of buffer; keep only the trailing bytes.
					let valid = e.valid_up_to();
					carry.drain(..valid);
					if carry.len() > 3 {
						return Ok(false);
					};
				}
			}
		}
		Ok(true)
	}

	/// Robust identification of binary and executable files
	pub fn is_binary(path: &PathBuf) -> Result<bool, FilesError> {
		if !path.exists() {
			return Err(FilesError::Io(std::io::Error::new(
				std::io::ErrorKind::NotFound,
				"File does not exist",
			)));
		}
		if path.is_dir() {
			return Err(FilesError::Io(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				"Path is a directory",
			)));
		}

		{
			let file = File::open(path)?;
			if file.metadata()?.len() == 0 {
				return Err(FilesError::Io(std::io::Error::new(
					std::io::ErrorKind::InvalidInput,
					"File is empty",
				)));
			} else if file.metadata()?.len() < 4 {
				return Ok(false);
			}
		}

		// Check file extension against known binary extensions
		if let Some(ext) = path.extension() {
			let ext = ext.to_string_lossy().to_lowercase();

			if Self::BINARY_EXTENSIONS.contains(&ext.as_str()) {
				return Ok(true);
			}
		}

		Ok(Self::detect_binary_magic(path)? || Self::detect_binary_structure(path)?)
	}

	const EXECUTABLE_MAGIC: &[&[u8]] = &[
		b"\x7FELF",
		b"MZ\x90\x00",
		b"MZ\x00\x00",
		b"\xFE\xED\xFA\xCE",
		b"\xFE\xED\xFA\xCF",
		b"\xCF\xFA\xED\xFE",
		b"\xCA\xFE\xBA\xBE",
		b"\x00asm",
		b"dex\n",
	];

	fn detect_binary_magic(path: &PathBuf) -> Result<bool, FilesError> {
		let mut buf: [u8; 4] = [0; 4];
		let file = File::open(path)?;
		let mut reader = BufReader::new(file);
		let n = reader.read(&mut buf)?;
		// Too small to be a binary
		if n < 4 {
			return Ok(false);
		}

		if Self::EXECUTABLE_MAGIC
			.iter()
			.any(|&magic| buf.starts_with(magic))
		{
			return Ok(true);
		}
		Ok(false)
	}

	fn detect_binary_structure(_path: &PathBuf) -> Result<bool, FilesError> {
		let mut buf: [u8; 32] = [0; 32];
		let file = File::open(_path)?;
		let mut reader = BufReader::new(file);
		let n = reader.read(&mut buf)?;

		let executable = PE::parse(&buf).is_ok()
			|| Elf::parse(&buf).is_ok()
			|| Mach::parse(&buf).is_ok()
			|| buf.len() > 10 && buf.starts_with(b"\xCA\xFE\xBA\xBE")
			|| buf.len() > 8 && buf.starts_with(b"\x00asm")
			|| buf.len() > 32 && buf.starts_with(b"dex\n");

		if executable {
			return Ok(true);
		}

		Ok(false)
	}
}

#[derive(Debug, Error)]
pub enum FilesError {
	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),
}
