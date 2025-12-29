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

/// HIBP API configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HibpConfig {
	/// Whether HIBP enrichment is enabled
	#[serde(default)]
	pub enabled: bool,

	/// HIBP API key (32-character hexadecimal string)
	#[serde(default)]
	pub api_key: String,
}

/// Ollama (Nomic) embedding service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
	/// Whether Ollama is enabled
	#[serde(default)]
	pub enabled: bool,

	/// Hostname for Ollama service (e.g., "localhost")
	#[serde(default = "default_ollama_host")]
	pub host: String,

	/// Port for Ollama service (e.g., 11435)
	#[serde(default = "default_ollama_port")]
	pub port: u16,
}

fn default_ollama_host() -> String {
	"localhost".to_string()
}

fn default_ollama_port() -> u16 {
	11435
}

impl Default for OllamaConfig {
	fn default() -> Self {
		Self {
			enabled: false,
			host: default_ollama_host(),
			port: default_ollama_port(),
		}
	}
}

/// Services configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServicesConfig {
	/// Ollama service configuration
	#[serde(default)]
	pub ollama: OllamaConfig,
}

/// OAuth configuration (optional)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OAuth {
	#[serde(default)]
	pub client_id: String,

	#[serde(default)]
	pub client_secret: String,

	// discovery_url may be empty if not using OIDC
	#[serde(default)]
	pub discovery_url: String,
}

/// API keys for external services
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiKeys {
	#[serde(default)]
	pub hibp: HibpConfig,
}

/// Email suffix substitution rules
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmailSuffixSubstitutions {
	#[serde(default)]
	pub rules: HashMap<String, Vec<String>>,
}

/// Custom plaintext passwords to check
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomPasswords {
	#[serde(default)]
	pub passwords: Vec<String>,
}

/// Working directory configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkingDirectoryConfig {
	/// Optional explicit path
	#[serde(default)]
	pub path: Option<String>,

	/// Verify noexec mount flag (default: true)
	#[serde(default = "default_verify_noexec")]
	pub verify_noexec: bool,
}

fn default_verify_noexec() -> bool {
	true
}

/// Main configuration structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
	#[serde(default)]
	pub oauth: OAuth,

	#[serde(default)]
	pub api_keys: ApiKeys,

	#[serde(default)]
	pub services: ServicesConfig,

	#[serde(default)]
	pub email_suffix_substitutions: EmailSuffixSubstitutions,

	#[serde(default)]
	pub custom_passwords: CustomPasswords,

	/// Working directory configuration
	#[serde(default)]
	pub working_directory: WorkingDirectoryConfig,
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

	/// Load configuration from multiple standard locations.
	///
	/// Searches for config in the following order (first found is used):
	/// 1. If `explicit_path` is provided, use that file
	/// 2. User config directory (~/.config/dumptruck/config.json on Linux/macOS)
	/// 3. System-wide config (/etc/dumptruck/config.json)
	/// 4. Current working directory (config.json)
	/// 5. Application directory (config.json)
	///
	/// Returns default configuration if no config file is found.
	///
	/// # Arguments
	/// * `explicit_path` - Optional explicit path to config file
	/// * `verbose` - Print debug info about config location search
	///
	/// # Returns
	/// Configuration structure (default if no file found)
	pub fn load_with_search(
		explicit_path: Option<&str>,
		verbose: bool,
	) -> Result<Self, ConfigError> {
		let config_paths = Self::get_config_search_paths(explicit_path);

		for path in config_paths {
			if fs::metadata(&path).is_ok() {
				if verbose {
					eprintln!("[INFO] Loading configuration from: {}", path);
				}
				return Self::from_file(&path);
			}
		}

		if verbose {
			eprintln!("[INFO] No configuration file found, using defaults");
		}
		Ok(Self::default())
	}

	/// Get list of config file search paths in priority order.
	///
	/// # Arguments
	/// * `explicit_path` - Optional explicit path provided by user
	///
	/// # Returns
	/// Vector of paths to search, in priority order
	fn get_config_search_paths(explicit_path: Option<&str>) -> Vec<String> {
		let mut paths = Vec::new();

		// 1. Explicit path (highest priority)
		if let Some(path) = explicit_path {
			paths.push(path.to_string());
			return paths; // Only search explicit path if provided
		}

		// 2. User config directory
		if let Some(config_dir) = dirs::config_dir() {
			let user_config = config_dir.join("dumptruck").join("config.json");
			if let Some(path_str) = user_config.to_str() {
				paths.push(path_str.to_string());
			}
		}

		// 3. System-wide config
		paths.push("/etc/dumptruck/config.json".to_string());

		// 4. Current working directory
		paths.push("config.json".to_string());

		// 5. App working directory
		if let Ok(exe_path) = std::env::current_exe()
			&& let Some(dir) = exe_path.parent()
		{
			let app_config = dir.join("config.json");
			if let Some(path_str) = app_config.to_str() {
				paths.push(path_str.to_string());
			}
		}

		paths
	}

	/// Load configuration with environment variable overrides.
	///
	/// Environment variables take precedence:
	/// * `DUMPTRUCK_HIBP_API_KEY` - Override HIBP API key
	pub fn from_file_with_env<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
		let mut config = Self::from_file(path)?;

		// Override HIBP API key from environment if present
		if let Ok(hibp_key) = std::env::var("DUMPTRUCK_HIBP_API_KEY") {
			config.api_keys.hibp.api_key = hibp_key;
		}

		Ok(config)
	}

	/// Get the HIBP API key.
	///
	/// Returns empty string if not configured.
	pub fn hibp_api_key(&self) -> &str {
		&self.api_keys.hibp.api_key
	}

	/// Check if HIBP is enabled.
	pub fn hibp_enabled(&self) -> bool {
		self.api_keys.hibp.enabled
	}

	/// Check if Ollama is enabled.
	pub fn ollama_enabled(&self) -> bool {
		self.services.ollama.enabled
	}

	/// Get Ollama endpoint URL.
	pub fn ollama_endpoint(&self) -> String {
		format!(
			"http://{}:{}",
			self.services.ollama.host, self.services.ollama.port
		)
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

	/// Get hashed versions of custom passwords.
	///
	/// Returns a HashMap mapping plaintext passwords to their MD5 and SHA256 hashes.
	/// SHA1 support removed - use MD5 or SHA256 instead.
	///
	/// # Returns
	/// HashMap where key is plaintext password and value is tuple of (md5, sha256)
	pub fn get_custom_password_hashes(&self) -> HashMap<String, (String, String)> {
		use crate::core::hash_utils;

		self.custom_passwords
			.passwords
			.iter()
			.map(|pwd| {
				let md5 = hash_utils::md5_hex_bytes(pwd.as_bytes());
				let sha256 = hash_utils::sha256_hex(pwd);
				(pwd.clone(), (md5, sha256))
			})
			.collect()
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

	/// Add a custom password to check.
	///
	/// # Arguments
	/// * `password` - The plaintext password to add
	pub fn add_custom_password(&mut self, password: String) {
		self.custom_passwords.passwords.push(password);
	}

	/// Validate configuration against schema constraints.
	///
	/// Validates:
	/// - HIBP API key is 32 hexadecimal characters (if enabled)
	/// - Email domains are valid format
	///
	/// # Returns
	/// Ok if valid, ConfigError::ValidationError if invalid
	pub fn validate(&self) -> Result<(), ConfigError> {
		// Validate HIBP API key format (32-character hex string) only if it's not empty
		if !self.api_keys.hibp.api_key.is_empty()
			&& (self.api_keys.hibp.api_key.len() != 32
				|| !self
					.api_keys
					.hibp
					.api_key
					.chars()
					.all(|c| c.is_ascii_hexdigit()))
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
#[path = "config_tests.rs"]
mod tests;
