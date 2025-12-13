// Integration tests for configuration module

#[cfg(test)]
mod config_tests {
	use dumptruck::config::Config;

	#[test]
	fn test_config_creation() {
		let mut config = Config::default();
		config.add_suffix_rule("gmail.com".to_string(), vec!["googlemail.com".to_string()]);
		assert!(config.has_suffix_alternates("gmail.com"));
	}

	#[test]
	fn test_email_suffix_mapping() {
		let mut config = Config::default();
		config.add_suffix_rule("gmail.com".to_string(), vec!["googlemail.com".to_string()]);
		config.add_suffix_rule(
			"bankofamerica.com".to_string(),
			vec!["bofa.com".to_string()],
		);
		config.add_suffix_rule(
			"microsoft.com".to_string(),
			vec![
				"outlook.com".to_string(),
				"hotmail.com".to_string(),
				"live.com".to_string(),
			],
		);

		// Check Gmail alternates
		let gmail_alts = config.get_suffix_alternates("gmail.com");
		assert!(gmail_alts.contains(&"googlemail.com".to_string()));

		// Check Bank of America alternates
		let bofa_alts = config.get_suffix_alternates("bankofamerica.com");
		assert!(bofa_alts.contains(&"bofa.com".to_string()));

		// Check Microsoft alternates
		let ms_alts = config.get_suffix_alternates("microsoft.com");
		assert!(ms_alts.contains(&"outlook.com".to_string()));
		assert!(ms_alts.contains(&"hotmail.com".to_string()));
		assert!(ms_alts.contains(&"live.com".to_string()));
	}

	#[test]
	fn test_nonexistent_suffix() {
		let config = Config::default();
		assert!(!config.has_suffix_alternates("example.com"));
		assert!(config.get_suffix_alternates("example.com").is_empty());
	}
}
