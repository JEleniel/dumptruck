//! OAuth 2.0 Client Credentials Flow implementation for Dumptruck server.
//!
//! Provides secure token acquisition and validation for API authentication.
//! Supports token caching with automatic refresh and scope validation.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// OAuth errors
#[derive(Debug, Error)]
pub enum OAuthError {
	#[error("Token request failed: {0}")]
	TokenRequestFailed(String),

	#[error("Invalid token response: {0}")]
	InvalidTokenResponse(String),

	#[error("Token expired")]
	TokenExpired,

	#[error("Invalid scope: {0}")]
	InvalidScope(String),

	#[error("Missing token")]
	MissingToken,

	#[error("HTTP error: {0}")]
	HttpError(String),
}

/// OAuth token response from authorization server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
	pub access_token: String,
	pub token_type: String,
	pub expires_in: i64,
	pub scope: Option<String>,
}

/// Cached token with expiry information
#[derive(Debug, Clone)]
struct CachedToken {
	token: String,
	expires_at: DateTime<Utc>,
	scope: Option<String>,
}

/// OAuth 2.0 Client Credentials Flow provider
pub struct OAuthProvider {
	client_id: String,
	client_secret: String,
	token_endpoint: String,
	scope: String,
	cached_token: Arc<RwLock<Option<CachedToken>>>,
	http_client: reqwest::Client,
}

impl OAuthProvider {
	/// Create a new OAuth provider with client credentials
	pub fn new(
		client_id: String,
		client_secret: String,
		token_endpoint: String,
		scope: String,
	) -> Self {
		Self {
			client_id,
			client_secret,
			token_endpoint,
			scope,
			cached_token: Arc::new(RwLock::new(None)),
			http_client: reqwest::Client::new(),
		}
	}

	/// Get a valid access token, refreshing if necessary
	pub async fn get_access_token(&self) -> Result<String, OAuthError> {
		// Check if cached token is still valid
		{
			let cached = self.cached_token.read().await;
			if let Some(token) = cached.as_ref() {
				// Refresh if expiring within 1 minute
				if let Some(refresh_time) = token.expires_at.checked_sub_signed(Duration::minutes(1)) {
					if Utc::now() < refresh_time {
						return Ok(token.token.clone());
					}
				}
			}
		}

		// Request new token
		let token = self.request_token().await?;

		// Cache the token
		let cached = CachedToken {
			token: token.access_token.clone(),
			expires_at: Utc::now()
				+ Duration::seconds(token.expires_in)
				- Duration::minutes(1), // Refresh 1 minute before expiry
			scope: token.scope.clone(),
		};

		{
			let mut cache = self.cached_token.write().await;
			*cache = Some(cached);
		}

		Ok(token.access_token)
	}

	/// Request a new token from the authorization server
	async fn request_token(&self) -> Result<OAuthToken, OAuthError> {
		let params = [
			("grant_type", "client_credentials"),
			("client_id", &self.client_id),
			("client_secret", &self.client_secret),
			("scope", &self.scope),
		];

		let response = self
			.http_client
			.post(&self.token_endpoint)
			.form(&params)
			.timeout(std::time::Duration::from_secs(10))
			.send()
			.await
			.map_err(|e| OAuthError::HttpError(e.to_string()))?;

		if !response.status().is_success() {
			return Err(OAuthError::TokenRequestFailed(format!(
				"HTTP {}",
				response.status()
			)));
		}

		response
			.json::<OAuthToken>()
			.await
			.map_err(|e| OAuthError::InvalidTokenResponse(e.to_string()))
	}

	/// Validate token scope
	pub fn validate_scope(&self, required_scopes: &[&str]) -> Result<(), OAuthError> {
		for scope in required_scopes {
			if !self.scope.contains(scope) {
				return Err(OAuthError::InvalidScope(scope.to_string()));
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_oauth_provider_creation() {
		let provider = OAuthProvider::new(
			"client_id".to_string(),
			"client_secret".to_string(),
			"https://oauth.example.com/token".to_string(),
			"read write".to_string(),
		);

		assert_eq!(provider.client_id, "client_id");
		assert_eq!(provider.scope, "read write");
	}

	#[test]
	fn test_scope_validation_success() {
		let provider = OAuthProvider::new(
			"client_id".to_string(),
			"client_secret".to_string(),
			"https://oauth.example.com/token".to_string(),
			"read write ingest".to_string(),
		);

		assert!(provider.validate_scope(&["read", "ingest"]).is_ok());
	}

	#[test]
	fn test_scope_validation_failure() {
		let provider = OAuthProvider::new(
			"client_id".to_string(),
			"client_secret".to_string(),
			"https://oauth.example.com/token".to_string(),
			"read write".to_string(),
		);

		assert!(provider.validate_scope(&["admin"]).is_err());
	}
}
