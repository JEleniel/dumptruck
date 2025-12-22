//! Hash algorithm fingerprinting and detection.

/// Hash algorithm types detected by fingerprinting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithmFingerprint {
	/// SHA1 without salt (40-char hex)
	Sha1Unsalted,
	/// MD5 without salt (32-char hex)
	Md5Unsalted,
	/// SHA256 without salt (64-char hex)
	Sha256Unsalted,
	/// SHA512 without salt (128-char hex)
	Sha512Unsalted,
	/// Bcrypt with salt ($2a$, $2b$, $2y$)
	Bcrypt,
	/// Scrypt with salt ($7$)
	Scrypt,
	/// Argon2 with salt ($argon2id$, $argon2i$, $argon2d$)
	Argon2,
	/// PBKDF2 with salt
	Pbkdf2,
	/// Unknown algorithm
	Unknown,
}

/// Result of hash algorithm fingerprinting.
#[derive(Debug, Clone)]
pub struct FingerprintMatch {
	/// Identified algorithm
	pub algorithm: HashAlgorithmFingerprint,
	/// Whether the algorithm is cryptographically weak
	pub is_weak: bool,
	/// Human-readable description
	pub description: String,
}

/// Detect hash algorithm type from fingerprint.
pub fn identify_hash_fingerprint(hash_value: &str) -> FingerprintMatch {
	let trimmed = hash_value.trim();

	// Check for salted hash prefixes
	if check_bcrypt(trimmed) {
		return bcrypt_match();
	}

	if check_scrypt(trimmed) {
		return scrypt_match();
	}

	if check_argon2(trimmed) {
		return argon2_match();
	}

	if check_pbkdf2(trimmed) {
		return pbkdf2_match();
	}

	// Check for hex hash lengths
	if let Some(match_) = check_hex_hashes(trimmed) {
		return match_;
	}

	unknown_match()
}

fn check_bcrypt(s: &str) -> bool {
	s.starts_with("$2a$") || s.starts_with("$2b$") || s.starts_with("$2y$")
}

fn check_scrypt(s: &str) -> bool {
	s.starts_with("$7$")
}

fn check_argon2(s: &str) -> bool {
	s.starts_with("$argon2id$") || s.starts_with("$argon2i$") || s.starts_with("$argon2d$")
}

fn check_pbkdf2(s: &str) -> bool {
	s.starts_with("$pbkdf2-sha256$") || s.starts_with("$pbkdf2-sha512$")
}

fn check_hex_hashes(s: &str) -> Option<FingerprintMatch> {
	if !s.chars().all(|c| c.is_ascii_hexdigit()) {
		return None;
	}

	match s.len() {
		40 => Some(sha1_match()),
		32 => Some(md5_match()),
		64 => Some(sha256_match()),
		128 => Some(sha512_match()),
		_ => None,
	}
}

fn bcrypt_match() -> FingerprintMatch {
	FingerprintMatch {
		algorithm: HashAlgorithmFingerprint::Bcrypt,
		is_weak: false,
		description: "Bcrypt hash (salted, cryptographically strong)".to_string(),
	}
}

fn scrypt_match() -> FingerprintMatch {
	FingerprintMatch {
		algorithm: HashAlgorithmFingerprint::Scrypt,
		is_weak: false,
		description: "Scrypt hash (salted, memory-hard)".to_string(),
	}
}

fn argon2_match() -> FingerprintMatch {
	FingerprintMatch {
		algorithm: HashAlgorithmFingerprint::Argon2,
		is_weak: false,
		description: "Argon2 hash (salted, memory-hard, time-hard)".to_string(),
	}
}

fn pbkdf2_match() -> FingerprintMatch {
	FingerprintMatch {
		algorithm: HashAlgorithmFingerprint::Pbkdf2,
		is_weak: false,
		description: "PBKDF2 hash (salted, iterated)".to_string(),
	}
}

fn sha1_match() -> FingerprintMatch {
	FingerprintMatch {
		algorithm: HashAlgorithmFingerprint::Sha1Unsalted,
		is_weak: true,
		description: "SHA1 hash (unsalted, deprecated)".to_string(),
	}
}

fn md5_match() -> FingerprintMatch {
	FingerprintMatch {
		algorithm: HashAlgorithmFingerprint::Md5Unsalted,
		is_weak: true,
		description: "MD5 hash (unsalted, weak)".to_string(),
	}
}

fn sha256_match() -> FingerprintMatch {
	FingerprintMatch {
		algorithm: HashAlgorithmFingerprint::Sha256Unsalted,
		is_weak: true,
		description: "SHA256 hash (unsalted)".to_string(),
	}
}

fn sha512_match() -> FingerprintMatch {
	FingerprintMatch {
		algorithm: HashAlgorithmFingerprint::Sha512Unsalted,
		is_weak: true,
		description: "SHA512 hash (unsalted)".to_string(),
	}
}

fn unknown_match() -> FingerprintMatch {
	FingerprintMatch {
		algorithm: HashAlgorithmFingerprint::Unknown,
		is_weak: false,
		description: "Unknown or unrecognized hash algorithm".to_string(),
	}
}
