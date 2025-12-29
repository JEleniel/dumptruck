//! Hash utilities and cryptographic fingerprinting
//!
//! This module provides cryptographic hashing operations:
//! - SHA-256 hashing
//! - Credential fingerprinting
//! - Hash verification and comparison algorithms

pub mod algorithms;
pub mod credentials;
pub mod fingerprint;

pub use algorithms::{
	md4_hex_bytes, md5_hex, md5_hex_bytes, ntlm_hex, sha1_hex, sha1_hex_bytes, sha256_hex,
	sha256_hex_bytes, sha512_hex, sha512_hex_bytes,
};
pub use credentials::is_credential_hash;
pub use fingerprint::{FingerprintMatch, HashAlgorithmFingerprint, identify_hash_fingerprint};

#[cfg(test)]
#[path = "hash_utils_tests.rs"]
mod tests;
