use std::path::PathBuf;

use config::File;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use crate::cli::Cli;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
	pub api_keys: Option<Vec<ApiKey>>,
	pub services: Option<ServiceConfiguration>,
	pub email_suffix_substitutions: Option<Vec<EmailSuffixSubstitutionConfiguration>>,
	pub paths: Option<PathConfiguration>,
	pub server: Option<ServerConfiguration>,
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
			paths: Some(PathConfiguration {
				temp_path: Some(PathBuf::from("/tmp/dumptruck")),
				db_path: Some(data_dir),
				seed_path: Some(PathBuf::from("./seed/")),
			}),
			server: None,
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
			let builder = config::Config::builder()
				.add_source(File::with_name(config_path.to_str()?).required(true));
			let config = builder.build()?;
			return Ok(config.try_deserialize()?);
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

		let builder = config::Config::builder()
			.add_source(File::with_name(local_path.to_str()?).required(false))
			.add_source(File::with_name(system_path.to_str()?).required(false));
		let builder = if let Some(user_path) = user_path {
			builder.add_source(File::with_name(user_path.to_str()).required(false))
		} else {
			builder
		};
		let config = builder.build()?;
		Ok(config.try_deserialize()?)
	}

	pub fn apply_cli_overrides(&mut self, cli: &Cli) -> Self {
		if let Some(api_keys) = cli.api_keys {
			if self.api_keys.is_none() {
				self.api_keys = Some(Vec::new());
			}
			self.api_keys.append(api_keys);
		}

		if cli.embeddings {
			self.services.enable_embeddings = true;
		}

		if self.paths.is_none() {
			self.paths = Some(PathConfiguration {
				temp_path: None,
				db_path: None,
				seed_path: None,
			});
		}

		if let Some(db_path) = cli.database {
			if let Some(paths) = &mut self.paths {
				paths.db_path = Some(db_path);
			}
		}

		if let Some(temp_path) = cli.temp_path {
			if let Some(paths) = &mut self.paths {
				paths.temp_path = Some(temp_path);
			}
		}

		if self.services.is_none() {
			self.services = Some(ServiceConfiguration {
				enable_embeddings: false,
				ollama: None,
			});
		}
		if self.services.ollama.is_none() {
			self.services.ollama = Some(OllamaConfiguration {
				ollama_url: None,
				vector_threshold: 0.85,
			});
		}
		if let Some(ollama_url) = cli.ollama_url {
			if let Some(services) = &mut self.services {
				if let Some(ollama) = &mut services.ollama {
					ollama.ollama_url = Some(ollama_url);
				}
			}
		}
		if let Some(threshold) = cli.vector_threshold {
			if let Some(services) = &mut self.services {
				if let Some(ollama) = &mut services.ollama {
					ollama.vector_threshold = threshold;
				}
			}
		}

		self
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
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
	pub temp_path: Option<PathBuf>,
	pub db_path: Option<PathBuf>,
	pub seed_path: Option<PathBuf>,
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
	pub bind_addresses: Option<Vec<String>>,
	pub port: Option<u16>,
	pub tls_cert_path: PathBuf,
	pub tls_key_path: PathBuf,
	pub oauth: OAuthConfiguration,
}

#[derive(Debug, Error)]
pub enum ConfigurationError {}
