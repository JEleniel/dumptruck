use std::path::PathBuf;

use config::File;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use crate::cli::Cli;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
	pub paths: PathConfiguration,
	pub services: ServiceConfiguration,
	pub server: ServerConfiguration,
	pub api_keys: Option<Vec<APIKey>>,
	pub email_suffix_substitutions: Option<Vec<EmailSuffixSubstitutionConfiguration>>,
}

impl Default for Configuration {
	fn default() -> Self {
		let data_dir = if let Some(data_dir) = dirs::data_local_dir() {
			data_dir.join("dumptruck")
		} else if let Some(data_dir) = dirs::data_dir() {
			data_dir.join("dumptruck")
		} else {
			PathBuf::from("./data/")
		};

		Self {
			api_keys: None,
			services: ServiceConfiguration {
				enable_embeddings: false,
				ollama: None,
			},
			email_suffix_substitutions: None,
			paths: PathConfiguration {
				temp_path: std::env::temp_dir().join("dumptruck"),
				db_path: data_dir,
			},
			server: ServerConfiguration {
				tls_cert_path: PathBuf::from("/etc/tls/tls.crt"),
				tls_key_path: PathBuf::from("/etc/tls/tls.key"),
				oauth: None,
				bind_addresses: vec!["0.0.0.0".to_string(), "::".to_string()],
				port: 443,
			},
		}
	}
}

impl Configuration {
	pub fn load(path: &Option<PathBuf>) -> Result<Self, ConfigurationError> {
		if let Some(config_path) = path {
			debug!(
				"Loading configuration from user specified path: {:?}",
				config_path
			);
			if let Some(path) = config_path.to_str() {
				let builder =
					config::Config::builder().add_source(File::with_name(path).required(true));
				let config = builder.build()?;
				return Ok(config.try_deserialize()?);
			} else {
				return Err(ConfigurationError::FilePathInvalid);
			}
		};

		let local_path = PathBuf::from("./").join("config.json");
		let system_path = PathBuf::from("/etc/dumptruck/").join("config.json");
		let user_path = if let Some(local_dir) = dirs::config_local_dir() {
			Some(local_dir.join("dumptruck").join("config.json"))
		} else {
			if let Some(local_dir) = dirs::config_dir() {
				Some(local_dir.join("dumptruck").join("config.json"))
			} else {
				warn!("Could not determine user config directory");
				None
			}
		};

		let mut builder = config::Config::builder();
		if let Some(path) = local_path.to_str() {
			builder = builder.add_source(File::with_name(path).required(false));
		}
		if let Some(path) = system_path.to_str() {
			builder = builder.add_source(File::with_name(path).required(false));
		}

		if let Some(u_path) = &user_path {
			if let Some(path) = u_path.to_str() {
				builder = builder.add_source(File::with_name(path).required(false));
			}
		}

		let config = builder.build()?;
		Ok(config.try_deserialize()?)
	}

	pub fn apply_cli_overrides(&mut self, cli: &Cli) -> Result<(), ConfigurationError> {
		if let Some(api_keys) = &cli.api_keys {
			if self.api_keys.is_none() {
				self.api_keys = Some(Vec::new());
			}
			if let Some(api_keys_vec) = &mut self.api_keys {
				for api_key in api_keys {
					api_keys_vec.push(api_key.clone());
				}
			}
		}

		if cli.embeddings {
			self.services.enable_embeddings = true;
		}

		if let Some(path) = &cli.database {
			self.paths.db_path = path.clone();
		}

		if let Some(temp_path) = &cli.temp_path {
			self.paths.temp_path = temp_path.clone();
		}

		if let Some(ollama_url) = &cli.ollama_url {
			if let Some(ollama) = self.services.ollama.as_mut() {
				ollama.ollama_url = ollama_url.clone();
			} else {
				self.services.ollama = Some(OllamaConfiguration {
					ollama_url: ollama_url.clone(),
					vector_threshold: 0.85,
				});
			}
		}
		if let Some(threshold) = cli.vector_threshold {
			if let Some(ollama) = self.services.ollama.as_mut() {
				ollama.vector_threshold = threshold;
			} else {
				self.services.ollama = Some(OllamaConfiguration {
					ollama_url: Url::parse("http://localhost")?,
					vector_threshold: threshold,
				});
			}
		}

		Ok(())
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIKey {
	pub api_name: String,
	pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfiguration {
	pub enable_embeddings: bool,
	pub ollama: Option<OllamaConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfiguration {
	pub ollama_url: Url,
	pub vector_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSuffixSubstitutionConfiguration {
	pub original: String,
	pub substitute: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfiguration {
	pub temp_path: PathBuf,
	pub db_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfiguration {
	pub client_id: String,
	pub client_secret: String,
	pub discovery_uri: String,
	pub scopes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfiguration {
	pub bind_addresses: Vec<String>,
	pub port: u16,
	pub tls_cert_path: PathBuf,
	pub tls_key_path: PathBuf,
	pub oauth: Option<OAuthConfiguration>,
}

#[derive(Debug, Error)]
pub enum ConfigurationError {
	#[error("Configuration error: {0}")]
	ConfigError(#[from] config::ConfigError),
	#[error("Configuration file path is invalid")]
	FilePathInvalid,
	#[error("URL parsing error: {0}")]
	URLParseError(#[from] url::ParseError),
}
