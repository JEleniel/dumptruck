use super::*;

#[test]
fn test_config_defaults() {
	let config = Config::default();
	assert!(config.hibp_api_key().is_empty());
	assert!(config.all_suffix_rules().is_empty());
}

#[test]
fn test_suffix_alternates() {
	let mut config = Config::default();
	config.add_suffix_rule("gmail.com".to_string(), vec!["googlemail.com".to_string()]);

	assert!(config.has_suffix_alternates("gmail.com"));
	let alternates = config.get_suffix_alternates("gmail.com");
	assert_eq!(alternates, vec!["googlemail.com"]);
}

#[test]
fn test_suffix_alternates_not_found() {
	let config = Config::default();
	assert!(!config.has_suffix_alternates("example.com"));
	assert!(config.get_suffix_alternates("example.com").is_empty());
}

#[test]
fn test_validate_valid_hibp_key() {
	let mut config = Config::default();
	config.api_keys.hibp.api_key = "abcdef0123456789abcdef0123456789".to_string();
	assert!(config.validate().is_ok());
}

#[test]
fn test_validate_invalid_hibp_key_format() {
	let mut config = Config::default();
	config.api_keys.hibp.api_key = "invalid-key".to_string();
	assert!(config.validate().is_err());
}

#[test]
fn test_validate_invalid_hibp_key_length() {
	let mut config = Config::default();
	config.api_keys.hibp.api_key = "abcdef0123456789abcdef01234567".to_string();
	assert!(config.validate().is_err());
}

#[test]
fn test_validate_valid_domains() {
	let mut config = Config::default();
	config.add_suffix_rule("gmail.com".to_string(), vec!["googlemail.com".to_string()]);
	assert!(config.validate().is_ok());
}

#[test]
fn test_validate_invalid_canonical_domain() {
	let mut config = Config::default();
	config.add_suffix_rule(
		"invalid-.com".to_string(),
		vec!["alternate.com".to_string()],
	);
	assert!(config.validate().is_err());
}

#[test]
fn test_validate_invalid_alternate_domain() {
	let mut config = Config::default();
	config.add_suffix_rule("gmail.com".to_string(), vec!["-invalid.com".to_string()]);
	assert!(config.validate().is_err());
}

#[test]
fn test_is_valid_domain() {
	assert!(Config::is_valid_domain("example.com"));
	assert!(Config::is_valid_domain("sub.example.com"));
	assert!(Config::is_valid_domain("a.co"));
	assert!(Config::is_valid_domain("my-domain.com"));

	assert!(!Config::is_valid_domain(""));
	assert!(!Config::is_valid_domain("-invalid.com"));
	assert!(!Config::is_valid_domain("invalid-.com"));
	assert!(!Config::is_valid_domain("invalid..com"));
}
