use dumptruck::normalization::normalize_field;
use proptest::prelude::*;

proptest! {
	#[test]
	fn normalize_idempotent(s in any::<String>()) {
		let a = normalize_field(&s);
		let b = normalize_field(&a);
		prop_assert_eq!(a, b);
	}

	#[test]
	fn normalize_no_uppercase(s in any::<String>()) {
		let out = normalize_field(&s);
		// no ASCII uppercase letters remain
		prop_assert!(!out.chars().any(|c| c.is_ascii_uppercase()));
	}

	#[test]
	fn normalize_collapse_whitespace(s in any::<String>()) {
		let out = normalize_field(&s);
		// no double spaces
		prop_assert!(!out.contains("  "));
	}
}
