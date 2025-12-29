use std::path::PathBuf;

use clap::Parser;
use reqwest::Url;

/// Server mode arguments for HTTP/2 API with TLS 1.3+
#[derive(Debug, Clone, Parser)]
pub struct ServerArgs {
	/// Bind addresses for the server, default is
	#[arg(short, long, value_name = "ADDRESSES", default_value = "0.0.0.0,::")]
	pub bind_addresses: Option<Vec<String>>,

	/// Port to bind the server to, default is 443 (HTTPS)
	#[arg(short, long, value_name = "PORT", default_value = "443")]
	pub port: u16,

	/// Path to TLS certificate file (PEM format)
	/// Defaults to `/etc/tls/tls.crt` if not provided here or in the config file
	#[arg(long, value_name = "PATH")]
	pub cert: Option<PathBuf>,

	/// Path to TLS private key file (PEM format)
	/// Defaults to `/etc/tls/tls.key` if not provided here or in the config file
	#[arg(long, value_name = "PATH")]
	pub key: Option<PathBuf>,

	/// OAuth 2.0 Client ID
	#[arg(long, value_name = "ID")]
	pub oauth_id: Option<String>,

	/// OAuth 2.0 Client Secret
	#[arg(long, value_name = "SECRET")]
	pub oauth_secret: Option<String>,

	/// OAuth 2.0 Token Endpoint URL
	#[arg(long, value_name = "URL")]
	pub oauth_discovery_url: Option<Url>,

	/// OAuth 2.0 scopes (space-separated)
	#[arg(long, value_name = "SCOPES", default_value = "openid profile email")]
	pub oauth_scope: String,

	/// Enable generation of Vector embeddings
	#[arg(long, default_value_t = false)]
	pub enable_embeddings: bool,
}
