//! Canonicalization / normalization helpers.
//!
//! Small, dependency-free helpers used by adapters and tests.

use crate::core::config::Config;

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
		s_folded = s_folded.replace('\u{2019}', "'").replace('\u{2018}', "'");
	}
	if s_folded.contains('\u{2013}') || s_folded.contains('\u{2014}') {
		s_folded = s_folded.replace('\u{2013}', "-").replace('\u{2014}', "-");
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

/// Normalize an entire row of fields.
pub fn normalize_row(row: &[String]) -> Vec<String> {
	row.iter().map(|f| normalize_field(f)).collect()
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
pub fn normalize_email_with_config(email: &str, config: &Config) -> String {
	let normalized = normalize_field(email);

	// Split on @ to extract local and domain parts
	if let Some(at_idx) = normalized.rfind('@') {
		let (local, domain) = normalized.split_at(at_idx);
		let domain = &domain[1..]; // Remove the @

		// Canonicalize local part: strip plus addressing and dots
		let canonical_local = canonicalize_email_local(local);

		// Check if this domain is an alternate of any canonical suffix
		let canonical_domain = {
			let mut result = domain.to_string();
			for (canonical, alternates) in config.all_suffix_rules() {
				if alternates.contains(&domain.to_string()) {
					result = canonical.clone();
					break;
				}
			}
			result
		};

		return format!("{}@{}", canonical_local, canonical_domain);
	}

	normalized
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn normalize_field_simple() {
		assert_eq!(normalize_field("  ExAmple  "), "example");
		assert_eq!(normalize_field("A\tB  C"), "a b c");
		assert_eq!(
			normalize_field("  multiple   spaces\nand tabs\t"),
			"multiple spaces and tabs"
		);
	}

	#[test]
	fn normalize_row_works() {
		let row = vec![
			" Alice ".to_string(),
			"BOB\tsmith".to_string(),
			"  EVE  ".to_string(),
		];
		let out = normalize_row(&row);
		assert_eq!(
			out,
			vec![
				"alice".to_string(),
				"bob smith".to_string(),
				"eve".to_string()
			]
		);
	}

	#[test]
	fn normalize_email_with_config_no_rules() {
		let config = Config::default();
		let email = "user@GMAIL.COM";
		assert_eq!(
			normalize_email_with_config(email, &config),
			"user@gmail.com"
		);
	}

	#[test]
	fn normalize_email_with_config_applies_substitution() {
		let mut config = Config::default();
		config.add_suffix_rule("gmail.com".to_string(), vec!["googlemail.com".to_string()]);

		let email = "user@googlemail.com";
		assert_eq!(
			normalize_email_with_config(email, &config),
			"user@gmail.com"
		);
	}

	#[test]
	fn normalize_email_with_config_preserves_canonical() {
		let mut config = Config::default();
		config.add_suffix_rule("gmail.com".to_string(), vec!["googlemail.com".to_string()]);

		let email = "user@gmail.com";
		assert_eq!(
			normalize_email_with_config(email, &config),
			"user@gmail.com"
		);
	}

	#[test]
	fn email_plus_addressing_stripped() {
		let config = Config::default();
		assert_eq!(
			normalize_email_with_config("user+spam@example.com", &config),
			"user@example.com"
		);
		assert_eq!(
			normalize_email_with_config("john+newsletter@GMAIL.COM", &config),
			"john@gmail.com"
		);
		assert_eq!(
			normalize_email_with_config("alice+test+foo@example.org", &config),
			"alice@example.org"
		);
	}

	#[test]
	fn email_dots_removed() {
		let config = Config::default();
		assert_eq!(
			normalize_email_with_config("john.doe@example.com", &config),
			"johndoe@example.com"
		);
		assert_eq!(
			normalize_email_with_config("j.o.h.n@EXAMPLE.COM", &config),
			"john@example.com"
		);
		assert_eq!(
			normalize_email_with_config("mary.jane.smith@example.org", &config),
			"maryjanesmith@example.org"
		);
	}

	#[test]
	fn email_dots_and_plus_combined() {
		let config = Config::default();
		// Plus is processed first (stripped), then dots are removed
		assert_eq!(
			normalize_email_with_config("john.doe+tag@example.com", &config),
			"johndoe@example.com"
		);
		assert_eq!(
			normalize_email_with_config("a.b.c+spam@EXAMPLE.ORG", &config),
			"abc@example.org"
		);
		assert_eq!(
			normalize_email_with_config("m.a.r.y+newsletter@example.com", &config),
			"mary@example.com"
		);
	}

	#[test]
	fn email_canonicalization_with_domain_substitution() {
		let mut config = Config::default();
		config.add_suffix_rule("gmail.com".to_string(), vec!["googlemail.com".to_string()]);

		// Plus addressing + domain substitution
		assert_eq!(
			normalize_email_with_config("user+spam@googlemail.com", &config),
			"user@gmail.com"
		);
		// Dots + domain substitution
		assert_eq!(
			normalize_email_with_config("john.doe@googlemail.com", &config),
			"johndoe@gmail.com"
		);
		// Plus + dots + domain substitution
		assert_eq!(
			normalize_email_with_config("john.doe+tag@GOOGLEMAIL.COM", &config),
			"johndoe@gmail.com"
		);
	}

	#[test]
	fn canonicalize_email_local_edge_cases() {
		// No plus sign or dots
		assert_eq!(canonicalize_email_local("user"), "user");
		// Only plus sign
		assert_eq!(canonicalize_email_local("user+"), "user");
		// Only dots
		assert_eq!(canonicalize_email_local("u.s.e.r"), "user");
		// Empty after stripping
		assert_eq!(canonicalize_email_local("+tag"), "");
		assert_eq!(canonicalize_email_local("."), "");
	}
}
