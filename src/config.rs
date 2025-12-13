//! Configuration management for API keys and email suffix substitutions.
//!
//! Loads configuration from a JSON file with support for environment variable overrides.
//! Configuration includes API keys for external services (HIBP) and email domain aliases.
//!
//! ## Schema
//!
//! Configuration is validated against `config.schema.json` which defines:
//! - `api_keys.hibp`: 32-character hexadecimal string for HIBP API access
//! - `email_suffix_substitutions`: Map of canonical domains to alternate domain forms
//!
//! See `config.schema.json` for complete schema definition and validation rules.

use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Configuration errors.
#[derive(Debug, Error)]
pub enum ConfigError {
	#[error("Failed to read config file: {0}")]
	ReadError(String),

	#[error("Failed to parse config JSON: {0}")]
	ParseError(#[from] serde_json::Error),

	#[error("Missing required configuration: {0}")]
	MissingKey(String),

	#[error("Configuration validation error: {0}")]
	ValidationError(String),
}

/// API keys for external services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeys {
	#[serde(default)]
	pub hibp: String,
}

/// Email suffix substitution rules.
/// The key is the canonical suffix, and the value is a list of alternate suffixes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSuffixSubstitutions {
	#[serde(flatten)]
	pub rules: HashMap<String, Vec<String>>,
}

/// Main configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	#[serde(default)]
	pub api_keys: ApiKeys,

	#[serde(default)]
	pub email_suffix_substitutions: EmailSuffixSubstitutions,
}

impl Default for ApiKeys {
	fn default() -> Self {
		Self {
			hibp: String::new(),
		}
	}
}

impl Default for EmailSuffixSubstitutions {
	fn default() -> Self {
		Self {
			rules: HashMap::new(),
		}
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			api_keys: ApiKeys::default(),
			email_suffix_substitutions: EmailSuffixSubstitutions::default(),
		}
	}
}

impl Config {
	/// Load configuration from a JSON file.
	///
	/// # Arguments
	/// * `path` - Path to the configuration file
	///
	/// # Returns
	/// Configuration structure or error if file cannot be read/parsed
	pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
		let path = path.as_ref();
		let contents = fs::read_to_string(path)
			.map_err(|e| ConfigError::ReadError(format!("{}: {}", path.display(), e)))?;

		let config: Config = serde_json::from_str(&contents)?;
		Ok(config)
	}

	/// Load configuration with environment variable overrides.
	///
	/// Environment variables take precedence:
	/// * `DUMPTRUCK_HIBP_API_KEY` - Override HIBP API key
	pub fn from_file_with_env<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
		let mut config = Self::from_file(path)?;

		// Override HIBP API key from environment if present
		if let Ok(hibp_key) = std::env::var("DUMPTRUCK_HIBP_API_KEY") {
			config.api_keys.hibp = hibp_key;
		}

		Ok(config)
	}

	/// Get the HIBP API key.
	///
	/// Returns empty string if not configured.
	pub fn hibp_api_key(&self) -> &str {
		&self.api_keys.hibp
	}

	/// Check if an email suffix has registered alternates.
	pub fn has_suffix_alternates(&self, suffix: &str) -> bool {
		self.email_suffix_substitutions.rules.contains_key(suffix)
	}

	/// Get all alternate suffixes for a canonical suffix.
	///
	/// # Arguments
	/// * `canonical_suffix` - The canonical email suffix (e.g., "gmail.com")
	///
	/// # Returns
	/// Vector of alternate suffixes, or empty vector if not found
	pub fn get_suffix_alternates(&self, canonical_suffix: &str) -> Vec<String> {
		self.email_suffix_substitutions
			.rules
			.get(canonical_suffix)
			.cloned()
			.unwrap_or_default()
	}

	/// Get all canonical suffixes with their alternates.
	///
	/// # Returns
	/// HashMap where key is canonical suffix and value is vector of alternates
	pub fn all_suffix_rules(&self) -> &HashMap<String, Vec<String>> {
		&self.email_suffix_substitutions.rules
	}

	/// Register a new suffix substitution rule.
	///
	/// # Arguments
	/// * `canonical_suffix` - The canonical suffix
	/// * `alternates` - Vector of alternate suffixes
	pub fn add_suffix_rule(&mut self, canonical_suffix: String, alternates: Vec<String>) {
		self.email_suffix_substitutions
			.rules
			.insert(canonical_suffix, alternates);
	}

	/// Validate configuration against schema constraints.
	///
	/// Validates:
	/// - HIBP API key is 32 hexadecimal characters
	/// - Email domains are valid format
	///
	/// # Returns
	/// Ok if valid, ConfigError::ValidationError if invalid
	pub fn validate(&self) -> Result<(), ConfigError> {
		// Validate HIBP API key format (32-character hex string)
		if !self.api_keys.hibp.is_empty()
			&& (self.api_keys.hibp.len() != 32
				|| !self.api_keys.hibp.chars().all(|c| c.is_ascii_hexdigit()))
		{
			return Err(ConfigError::ValidationError(
				"hibp key must be 32 hexadecimal characters".to_string(),
			));
		}

		// Validate email suffix substitutions structure
		for (canonical, alternates) in &self.email_suffix_substitutions.rules {
			// Validate canonical domain format
			if !Self::is_valid_domain(canonical) {
				return Err(ConfigError::ValidationError(format!(
					"invalid canonical domain: {}",
					canonical
				)));
			}

			// Validate each alternate domain
			for alternate in alternates {
				if !Self::is_valid_domain(alternate) {
					return Err(ConfigError::ValidationError(format!(
						"invalid alternate domain: {} (for canonical: {})",
						alternate, canonical
					)));
				}
			}
		}

		Ok(())
	}

	/// Check if a string is a valid domain name.
	///
	/// Validates against RFC 1123 domain name rules:
	/// - Labels separated by dots
	/// - Each label 1-63 characters
	/// - Labels contain only alphanumeric and hyphen
	/// - Labels don't start or end with hyphen
	/// - Total length 1-253 characters
	fn is_valid_domain(domain: &str) -> bool {
		if domain.is_empty() || domain.len() > 253 {
			return false;
		}

		domain.split('.').all(|label| {
			!label.is_empty()
				&& label.len() <= 63
				&& label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
				&& !label.starts_with('-')
				&& !label.ends_with('-')
		})
	}
}

#[cfg(test)]
mod tests {
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
		config.api_keys.hibp = "abcdef0123456789abcdef0123456789".to_string();
		assert!(config.validate().is_ok());
	}

	#[test]
	fn test_validate_invalid_hibp_key_format() {
		let mut config = Config::default();
		config.api_keys.hibp = "invalid-key".to_string();
		assert!(config.validate().is_err());
	}

	#[test]
	fn test_validate_invalid_hibp_key_length() {
		let mut config = Config::default();
		config.api_keys.hibp = "abcdef0123456789abcdef01234567".to_string();
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
}
