//! Helper normalization and hashing utilities used by the NPI/PII detection module.
mod alias_resolution;
mod evidence;

use crate::{HashError, configuration::EmailSuffixSubstitutionConfiguration};
use thiserror::Error;

pub struct Normalize {}

impl Normalize {
	/// Normalize an IP address
	pub fn normalize_ip(value: &str) -> Result<String, NormalizeError> {
		if value.contains(':') {
			// IPv6
			let addr: std::net::Ipv6Addr = value.parse()?;
			Ok(addr.to_string())
		} else if value.contains('.') {
			// IPv4
			let addr: std::net::Ipv4Addr = value.parse()?;
			Ok(addr.to_string())
		} else {
			Err(NormalizeError::InvalidIPFormat(value.to_string()))
		}
	}

	/// Normalize an entire row of fields.
	pub fn normalize_row(row: &[String]) -> Vec<String> {
		row.iter().map(|f| Self::normalize_field(f)).collect()
	}

	/// Normalize a single field: trim, apply Unicode compatibility normalization (NFKC),
	/// lowercase, and collapse whitespace. This makes field comparisons resilient to
	/// interchangeable Unicode characters (e.g. composed vs decomposed forms, fullwidth
	/// vs ASCII compatibility characters).
	pub fn normalize_field(input: &str) -> String {
		use icu_casemap::CaseMapperBorrowed;
		use unicode_normalization::UnicodeNormalization;

		// Use the borrowed, `'static` case mapper compiled into the crate via
		// the `compiled_data` feature. This type is const-constructible and
		// safe to use as a static value.
		static CASE_MAPPER: CaseMapperBorrowed<'static> = CaseMapperBorrowed::new();

		// Trim first, then apply NFKC to fold compatibility variants.
		let s = input.trim();
		let s_nfkc: String = s.nfkc().collect();

		// Use the static borrowed CaseMapper for full, spec-correct Unicode case-folding.
		let mut s_folded = CASE_MAPPER.fold_string(&s_nfkc).into_owned();

		// Normalize common punctuation variants that appear in names/emails so
		// that different user-entered glyphs compare equal. Map curly apostrophes
		// to ASCII apostrophe and common dash variants to ASCII hyphen.
		if s_folded.contains('\u{2019}') || s_folded.contains('\u{2018}') {
			s_folded = s_folded.replace(['\u{2019}', '\u{2018}'], "'");
		}
		if s_folded.contains('\u{2013}') || s_folded.contains('\u{2014}') {
			s_folded = s_folded.replace(['\u{2013}', '\u{2014}'], "-");
		}

		let mut out = String::with_capacity(s_folded.len());
		let mut last_was_space = false;

		for ch in s_folded.chars() {
			if ch.is_whitespace() {
				if !last_was_space {
					out.push(' ');
					last_was_space = true;
				}
			} else {
				out.push(ch);
				last_was_space = false;
			}
		}

		// Ensure we don't introduce leading/trailing whitespace during NFKC
		// normalization (keeps the function idempotent).
		out.trim().to_string()
	}

	/// Normalize an email address by applying normalization and then resolving
	/// email domain alternates using configuration rules.
	///
	/// Also canonicalizes the local part by stripping plus addressing (user+tag)
	/// and removing dots, as these are ignored by most email systems and don't
	/// represent different individuals.
	///
	/// Examples:
	/// - "user+spam@GMAIL.COM" → "user@gmail.com"
	/// - "john.doe@googlemail.com" → "johndoe@gmail.com"
	/// - "a.b+c@EXAMPLE.COM" → "ab@example.com"
	///
	/// # Arguments
	/// * `email` - The raw email address
	/// * `config` - Configuration containing suffix substitution rules
	///
	/// # Returns
	/// Canonical email address with normalized domain and stripped local part
	pub fn normalize_and_map_email(
		email: &str,
		mappings: Vec<EmailSuffixSubstitutionConfiguration>,
	) -> Result<Vec<String>, NormalizeError> {
		let normalized = Self::normalize_field(email);

		// Split on @ to extract local and domain parts
		if let Some(at_idx) = normalized.rfind('@') {
			let (local, domain) = normalized.split_at(at_idx);
			let domain = &domain[1..]; // Remove the @

			// Canonicalize local part: strip plus addressing and dots
			let canonical_local = Self::canonicalize_email_local(local);

			// Check if this domain is an alternate of any canonical suffix
			let apply_mappings: Vec<&EmailSuffixSubstitutionConfiguration> =
				mappings.iter().filter(|m| m.original == domain).collect();
			if !apply_mappings.is_empty() {
				return Ok(apply_mappings
					.iter()
					.map(|m| format!("{}@{}", canonical_local, m.substitute))
					.collect());
			}

			return Ok(vec![format!("{}@{}", canonical_local, domain)]);
		}
		Err(NormalizeError::InvalidEmailFormat)
	}

	fn strip_whitespace(value: &str) -> String {
		value.chars().filter(|c| !c.is_whitespace()).collect()
	}

	fn strip_hyphens(value: &str) -> String {
		value.chars().filter(|c| *c != '-').collect()
	}

	fn strip_slashes(value: &str) -> String {
		value.chars().filter(|c| *c != '/').collect()
	}

	fn normalize_lower_trim(value: &str) -> String {
		value.trim().to_lowercase()
	}

	/// Canonicalize the local part of an email by stripping plus addressing
	/// and removing dots, which are ignored by most email systems.
	///
	/// Examples:
	/// - "john+spam" → "john"
	/// - "john.doe" → "johndoe"
	/// - "a.b+tag" → "ab"
	///
	/// # Arguments
	/// * `local` - The local part of an email (before @), already lowercased
	///
	/// # Returns
	/// Canonical local part with plus addressing stripped and dots removed
	fn canonicalize_email_local(local: &str) -> String {
		// First, strip plus addressing (user+tag → user)
		let without_plus = local.split('+').next().unwrap_or(local);

		// Then, remove all dots (john.doe → johndoe)
		without_plus.replace('.', "")
	}
}

#[derive(Debug, Error)]
pub enum NormalizeError {
	/// Invalid email format
	#[error("Invalid email format")]
	InvalidEmailFormat,
	#[error("Hashing error: {0}")]
	HashingError(#[from] HashError),
	#[error("IP parsing error: {0}")]
	IpParsingError(#[from] std::net::AddrParseError),
	#[error("Invalid IP format: {0}")]
	InvalidIPFormat(String),
}
