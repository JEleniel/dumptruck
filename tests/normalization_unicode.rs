use dumptruck::normalization::normalize_field;

#[test]
fn unicode_equivalence_pairs() {
	// Tests that exercise common Unicode equivalences relevant to identifying
	// personal names and email addresses. The goal is that the same logical
	// name/email will compare equal after `normalize_field` regardless of
	// typical user-facing orthographic differences:
	// - Composed vs decomposed accents (NFC/NFD)
	// - Compatibility forms (fullwidth ASCII, ligatures, compatibility ideographs)
	// - Language-sensitive case mappings (e.g., Turkish dotted I) via ICU4X
	// - Case differences in email local/domain for comparison purposes
	// We keep these expectations intentionally broad because upstream matching
	// logic (ingest/pipeline) relies on canonicalized, case-folded comparisons.

	// Composed vs decomposed (e + acute)
	assert_eq!(normalize_field("José"), normalize_field("Jose\u{0301}"));

	// Fullwidth ASCII compatibility characters (names sometimes come from
	// legacy content or copy/paste and use fullwidth characters)
	assert_eq!(normalize_field("Ａlice"), normalize_field("Alice"));

	// Additional common name examples
	// - Renée (composed) vs decomposed (note: decomposed has separate base character plus combining
	//   accent; preserve the trailing plain 'e')
	assert_eq!(normalize_field("Renée"), normalize_field("Rene\u{0301}e"));

	// - Scandinavian Å vs A + ring
	assert_eq!(normalize_field("Åke"), normalize_field("A\u{030A}ke"));

	// German Eszett should fold to SS
	assert_eq!(normalize_field("Straße"), normalize_field("STRASSE"));

	// Simple Greek sigma equivalence: small sigma vs final sigma
	assert_eq!(normalize_field("σ"), normalize_field("ς"));

	// Turkish dotted I: ICU4X folding produces an explicit dotted-i form
	// in the root mapping; ensure typical forms compare equal for names.
	assert_eq!(
		normalize_field("İstanbul"),
		normalize_field("i\u{0307}stanbul")
	);

	// Ligature: U+FB01 (ﬁ) -> "fi"
	assert_eq!(normalize_field("\u{FB01}ancé"), normalize_field("fiancé"));

	// Additional name punctuation: different apostrophe characters are
	// common in surnames (O'Connor vs O’Connor). NFKC normalizes some
	// punctuation; verify the two forms compare equal if present.
	assert_eq!(
		normalize_field("O'Connor"),
		normalize_field("O\u{2019}Connor")
	);

	// Fullwidth digits vs ASCII digits
	assert_eq!(normalize_field("１２３"), normalize_field("123"));

	// Emails: domain and local-part case/compatibility normalization
	assert_eq!(
		normalize_field("ExAmple@Example.COM"),
		normalize_field("example@example.com")
	);
	// Fullwidth local-part and punctuation should normalize
	assert_eq!(
		normalize_field("Ｅxample@Example.COM"),
		normalize_field("example@example.com")
	);
	// Accented local-part should normalize equivalently
	assert_eq!(
		normalize_field("josé@example.com"),
		normalize_field("jose\u{0301}@example.com")
	);

	// ANGSTROM SIGN (compatibility) vs A + ring above
	assert_eq!(normalize_field("\u{212B}"), normalize_field("A\u{030A}"));
}
