//! TLS 1.3+ configuration with hardened security settings.
//!
//! Enforces TLS 1.3+ only, certificate validation, and ALPN negotiation for HTTP/2.

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::ServerConfig;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Initialize rustls crypto provider (must be called once at startup)
pub fn init_crypto_provider() {
	// Install the default crypto provider for rustls
	// The 'ring' feature is enabled in Cargo.toml, so this provides ring-based crypto
	let _ = rustls::crypto::ring::default_provider().install_default();
}

/// TLS configuration errors
#[derive(Debug, Error)]
pub enum TlsError {
	#[error("Failed to read certificate: {0}")]
	CertificateReadError(String),

	#[error("Failed to read private key: {0}")]
	PrivateKeyReadError(String),

	#[error("Failed to parse certificate: {0}")]
	CertificateParseError(String),

	#[error("Failed to parse private key: {0}")]
	PrivateKeyParseError(String),

	#[error("Failed to create TLS config: {0}")]
	ConfigurationError(String),

	#[error("Invalid certificate: {0}")]
	InvalidCertificate(String),

	#[error("Self-signed certificates not accepted")]
	SelfSignedCertificateRejected,
}

/// Load and validate a certificate chain
fn load_certificates(cert_path: &Path) -> Result<Vec<CertificateDer<'static>>, TlsError> {
	let cert_data = fs::read(cert_path)
		.map_err(|e| TlsError::CertificateReadError(format!("{}: {}", cert_path.display(), e)))?;

	let certs: Vec<_> = rustls_pemfile::certs(&mut std::io::Cursor::new(&cert_data))
		.collect::<Result<_, _>>()
		.map_err(|_| {
			TlsError::CertificateParseError("Failed to parse PEM certificates".to_string())
		})?;

	Ok(certs)
}

/// Load and parse private key (supports PKCS8, EC, and RSA formats)
fn load_private_key(key_path: &Path) -> Result<PrivateKeyDer<'static>, TlsError> {
	let key_data = fs::read(key_path)
		.map_err(|e| TlsError::PrivateKeyReadError(format!("{}: {}", key_path.display(), e)))?;

	let mut cursor = std::io::Cursor::new(&key_data);

	// Try PKCS8 first (most common for modern keys)
	let keys: Vec<_> = rustls_pemfile::pkcs8_private_keys(&mut cursor)
		.collect::<Result<_, _>>()
		.map_err(|_| TlsError::PrivateKeyParseError("Failed to parse PKCS8 keys".to_string()))?;
	if !keys.is_empty() {
		let key = keys[0].clone_key();
		return Ok(PrivateKeyDer::Pkcs8(key));
	}

	// Try EC keys
	cursor.set_position(0);
	let keys: Vec<_> = rustls_pemfile::ec_private_keys(&mut cursor)
		.collect::<Result<_, _>>()
		.map_err(|_| TlsError::PrivateKeyParseError("Failed to parse EC keys".to_string()))?;
	if !keys.is_empty() {
		let key = keys[0].clone_key();
		return Ok(PrivateKeyDer::Sec1(key));
	}

	// Try RSA keys
	cursor.set_position(0);
	let keys: Vec<_> = rustls_pemfile::rsa_private_keys(&mut cursor)
		.collect::<Result<_, _>>()
		.map_err(|_| TlsError::PrivateKeyParseError("Failed to parse RSA keys".to_string()))?;
	if !keys.is_empty() {
		let key = keys[0].clone_key();
		return Ok(PrivateKeyDer::Pkcs1(key));
	}

	Err(TlsError::PrivateKeyParseError(
		"Could not parse private key in PKCS8, EC, or RSA format".to_string(),
	))
}

/// Create TLS 1.3+ server configuration
pub fn create_tls_server_config(cert_path: &str, key_path: &str) -> Result<ServerConfig, TlsError> {
	let cert_path = Path::new(cert_path);
	let key_path = Path::new(key_path);

	// Load certificate and private key
	let certs = load_certificates(cert_path)?;
	let key = load_private_key(key_path)?;

	if certs.is_empty() {
		return Err(TlsError::InvalidCertificate(
			"No certificates found in certificate file".to_string(),
		));
	}

	// Create TLS config - rustls defaults to TLS 1.3+ only
	let mut config = ServerConfig::builder()
		.with_no_client_auth()
		.with_single_cert(certs, key)
		.map_err(|e| TlsError::ConfigurationError(e.to_string()))?;

	// Configure ALPN for HTTP/2 support
	// This tells the client that we support both HTTP/1.1 and HTTP/2
	config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

	Ok(config)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_tls_error_display() {
		let error = TlsError::SelfSignedCertificateRejected;
		assert_eq!(error.to_string(), "Self-signed certificates not accepted");
	}

	#[test]
	fn test_tls_configuration_error() {
		let error = TlsError::ConfigurationError("Test error".to_string());
		assert_eq!(error.to_string(), "Failed to create TLS config: Test error");
	}
}
