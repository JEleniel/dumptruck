use std::io::{self};

use md4::{Digest, Md4};
use md5;
use sha2::Sha256;
use thiserror::Error;

use des::Des;

pub struct Hash {}

impl Hash {
	const LM_MAGIC: &[u8; 8] = b"KGS!@#$%";

	/// Compute MD5 signature of a reader
	pub fn calculate_md5(reader: &mut impl io::Read) -> Result<String, HashError> {
		let mut context = md5::Context::new();
		let mut buffer = [0u8; 4096]; // buffer size: 4KB

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

	/// Compute SHA1 of a reader
	pub fn calculate_sha1(reader: &mut impl io::Read) -> Result<String, HashError> {
		let mut hasher = sha1::Sha1::new();
		let mut buffer = [0u8; 4096]; // buffer size: 4KB
		loop {
			let bytes_read = reader.read(&mut buffer)?;
			if bytes_read == 0 {
				break;
			}

			hasher.update(&buffer[..bytes_read]);
		}
		Ok(format!("{:x}", hasher.finalize()))
	}

	/// Compute SHA256 of a reader
	pub fn calculate_sha256(reader: &mut impl io::Read) -> Result<String, HashError> {
		let mut hasher = Sha256::new();
		let mut buffer = [0u8; 4096]; // buffer size: 4KB	

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
		let mut buffer = [0u8; 4096]; // buffer size: 4KB

		loop {
			let bytes_read = reader.read(&mut buffer)?;
			if bytes_read == 0 {
				break;
			}

			hasher.update(&buffer[..bytes_read]);
		}

		Ok(format!("{:x}", hasher.finalize()))
	}

	/// Compute LM hash of a password
	pub fn calculate_lm(password: &str) -> String {
		let pw = Self::str_to_lm_bytes(password);

		let mut left = [0u8; 7];
		let mut right = [0u8; 7];
		left.copy_from_slice(&pw[0..7]);
		right.copy_from_slice(&pw[7..14]);

		let key1 = Self::create_des_key(&left);
		let key2 = Self::create_des_key(&right);

		let enc1 = Self::des_encrypt(&key1, Self::LM_MAGIC);
		let enc2 = Self::des_encrypt(&key2, Self::LM_MAGIC);

		let mut out = Vec::with_capacity(16);
		out.extend_from_slice(&enc1);
		out.extend_from_slice(&enc2);

		hex::encode_upper(out)
	}

	/// Compute MySQL old hash of a password
	pub fn calculate_mysqlold(password: &str) -> String {
		let mut nr: u32 = 1345345333;
		let mut nr2: u32 = 0x12345671;
		let mut add: u32 = 7;

		for &c in password.as_bytes() {
			if c == b' ' || c == b'\t' {
				continue;
			}

			let tmp = c as u32;
			nr ^= (((nr & 63) + add) * tmp) + (nr << 8);
			nr2 = nr2.wrapping_add((nr2 << 8) ^ nr);
			add = add.wrapping_add(tmp);
		}

		nr &= 0x7FFF_FFFF;
		nr2 &= 0x7FFF_FFFF;

		format!("{:08x}{:08x}", nr, nr2)
	}

	/// Hash a numeric value stripped of all non-digit characters
	pub fn hash_numeric(value: &str) -> Result<String, HashError> {
		let normalized = Self::digits_only(value);
		Ok(Self::calculate_sha256(&mut normalized.as_bytes())?)
	}

	/// Hash an alphanumeric value stripped of whitespace and punctuation,
	/// excluding '@' to preserve email-like strings.
	pub fn hash_alphanumeric(value: &str) -> Result<String, HashError> {
		let normalized: String = value
			.chars()
			.filter(|c| c.is_alphanumeric() || *c == '@')
			.collect();
		Ok(Self::calculate_sha256(&mut normalized.as_bytes())?)
	}

	fn digits_only(value: &str) -> String {
		value.chars().filter(|c| c.is_ascii_digit()).collect()
	}

	fn str_to_lm_bytes(password: &str) -> [u8; 14] {
		let mut out = [0u8; 14];
		let up = password.to_uppercase();
		let bytes = up.as_bytes();

		for i in 0..14.min(bytes.len()) {
			out[i] = bytes[i];
		}
		out
	}

	// Convert 7-byte chunk to DES key with parity bits
	fn create_des_key(chunk: &[u8; 7]) -> [u8; 8] {
		let mut key = [0u8; 8];

		key[0] = chunk[0] & 0xFE;
		key[1] = ((chunk[0] << 7) | (chunk[1] >> 1)) & 0xFE;
		key[2] = ((chunk[1] << 6) | (chunk[2] >> 2)) & 0xFE;
		key[3] = ((chunk[2] << 5) | (chunk[3] >> 3)) & 0xFE;
		key[4] = ((chunk[3] << 4) | (chunk[4] >> 4)) & 0xFE;
		key[5] = ((chunk[4] << 3) | (chunk[5] >> 5)) & 0xFE;
		key[6] = ((chunk[5] << 2) | (chunk[6] >> 6)) & 0xFE;
		key[7] = (chunk[6] << 1) & 0xFE;

		// set odd parity bit (LSB)
		for b in &mut key {
			let ones = b.count_ones();
			if ones % 2 == 0 {
				*b |= 1;
			}
		}

		key
	}

	fn des_encrypt(key: &[u8; 8], data: &[u8; 8]) -> [u8; 8] {
		use des::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
		let cipher = Des::new_from_slice(key).unwrap();
		let mut block = GenericArray::clone_from_slice(data);
		cipher.encrypt_block(&mut block);
		let mut out = [0u8; 8];
		out.copy_from_slice(&block);
		out
	}
}

#[derive(Debug, Error)]
pub enum HashError {
	#[error("IO error: {0}")]
	Io(#[from] io::Error),
}
