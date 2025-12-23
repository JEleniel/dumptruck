//! Hash algorithm implementations and utilities.

use md4::{Digest, Md4};
use md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};

/// Compute SHA256 hash from string.
pub fn sha256_hex(input: &str) -> String {
	sha256_hex_bytes(input.as_bytes())
}

/// Compute SHA256 hash from bytes.
pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
	let mut hasher = Sha256::new();
	hasher.update(bytes);
	let res = hasher.finalize();
	hex::encode(res)
}

/// Compute SHA512 hash from string.
pub fn sha512_hex(input: &str) -> String {
	sha512_hex_bytes(input.as_bytes())
}

/// Compute SHA512 hash from bytes.
pub fn sha512_hex_bytes(bytes: &[u8]) -> String {
	let mut hasher = Sha512::new();
	hasher.update(bytes);
	let res = hasher.finalize();
	hex::encode(res)
}

/// Compute SHA1 hash from string.
pub fn sha1_hex(input: &str) -> String {
	sha1_hex_bytes(input.as_bytes())
}

/// Compute SHA1 hash from bytes.
pub fn sha1_hex_bytes(bytes: &[u8]) -> String {
	let mut hasher = Sha1::new();
	hasher.update(bytes);
	let res = hasher.finalize();
	hex::encode(res)
}

/// Compute MD5 hash from string.
pub fn md5_hex(input: &str) -> String {
	md5_hex_bytes(input.as_bytes())
}

/// Compute MD5 hash from bytes.
pub fn md5_hex_bytes(bytes: &[u8]) -> String {
	let digest = md5::compute(bytes);
	hex::encode(digest.0)
}

/// Generate NTLM hash (Windows NT LAN Manager hash).
pub fn ntlm_hex(input: &str) -> String {
	let mut hasher = Md4::new();
	let utf16_bytes: Vec<u8> = input
		.encode_utf16()
		.flat_map(|code_unit| code_unit.to_le_bytes())
		.collect();
	hasher.update(utf16_bytes);
	let res = hasher.finalize();
	hex::encode(res)
}

/// Compute MD4 hash from bytes.
pub fn md4_hex_bytes(bytes: &[u8]) -> String {
	let mut hasher = Md4::new();
	hasher.update(bytes);
	let res = hasher.finalize();
	hex::encode(res)
}
