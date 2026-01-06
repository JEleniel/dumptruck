use std::io::{self, BufReader};

use md4::{Digest, Md4};
use md5::Context;
use thiserror::Error;

pub struct Hash {}

impl Hash {
	/// Compute MD5 signature of a reader
	pub fn calculate_md5(reader: &mut impl io::Read) -> Result<String, HashError> {
		let mut context = Context::new();
		let mut buffer = [0; 4096]; // buffer size: 4KB

		loop {
			let bytes_read = reader.read(&mut buffer)?;
			if bytes_read == 0 {
				break;
			}

			context.consume(&buffer[..bytes_read]);
		}

		let result = context.finalize();

		Ok(format!("{:x}", result))
	}

	/// Compute SHA256 of a reader
	pub fn calculate_sha256(reader: &mut impl io::Read) -> Result<String, HashError> {
		let mut hasher = sha2::Sha256::new();
		let mut buffer = [0; 4096]; // buffer size: 4KB	

		loop {
			let bytes_read = reader.read(&mut buffer)?;
			if bytes_read == 0 {
				break;
			}

			hasher.update(&buffer[..bytes_read]);
		}

		Ok(format!("{:x}", hasher.finalize()))
	}

	/// Compute NTLM hash of a reader
	pub fn calculate_ntlm(reader: &mut impl io::Read) -> Result<String, HashError> {
		let mut hasher = Md4::new();
		let mut buffer = [0; 4096]; // buffer size: 4KB

		loop {
			let bytes_read = reader.read(&mut buffer)?;
			if bytes_read == 0 {
				break;
			}

			hasher.update(&buffer[..bytes_read]);
		}

		Ok(format!("{:x}", hasher.finalize()))
	}

	/// Hash a numeric value stripped of all non-digit characters
	pub fn hash_numeric(value: &str) -> Result<String, HashError> {
		let normalized = Self::digits_only(value);
		Ok(Self::calculate_sha256(&mut BufReader::new(
			normalized.as_bytes(),
		))?)
	}

	/// Hash an alphanumeric value stripped of whitespace and punctuation,
	/// excluding '@' to preserve email-like strings.
	pub fn hash_alphanumeric(value: &str) -> Result<String, HashError> {
		let normalized: String = value
			.chars()
			.filter(|c| c.is_alphanumeric() || *c == '@')
			.collect();
		Ok(Self::calculate_sha256(&mut BufReader::new(
			normalized.as_bytes(),
		))?)
	}

	fn digits_only(value: &str) -> String {
		value.chars().filter(|c| c.is_ascii_digit()).collect()
	}
}

#[derive(Debug, Error)]
pub enum HashError {
	#[error("IO error: {0}")]
	Io(#[from] io::Error),
}
