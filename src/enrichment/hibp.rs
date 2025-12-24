//! Have I Been Pwned (HIBP) API client for breach data enrichment.
//!
//! This module provides async HTTP client for querying the HIBP API to identify
//! which breaches have contained a given email address. Breach data is used to
//! enrich canonical address records with real-world threat intelligence.

use std::io;

/// Breach information returned by HIBP API.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Breach {
	pub name: String,
	pub title: String,
	pub domain: String,
	pub breach_date: String,
	pub added_date: String,
	pub modified_date: String,
	pub pwn_count: i64,
	pub description: String,
	pub is_verified: bool,
	pub is_fabricated: bool,
	pub is_sensitive: bool,
	pub is_retired: bool,
	pub logo_path: String,
}

/// Async HTTP client for Have I Been Pwned API v3.
///
/// Requires a User-Agent header and optional API key for higher rate limits.
/// API documentation: https://haveibeenpwned.com/API/v3
pub struct HibpClient {
	base_url: String,
	user_agent: String,
	api_key: Option<String>,
	client: reqwest::Client,
}

impl HibpClient {
	/// Create a new HIBP client.
	///
	/// # Arguments
	/// * `user_agent` - Required User-Agent header (HIBP API requires this)
	/// * `api_key` - Optional API key for higher rate limits (default: no key)
	pub fn new(user_agent: String, api_key: Option<String>) -> Self {
		HibpClient {
			base_url: "https://haveibeenpwned.com/api/v3".to_string(),
			user_agent,
			api_key,
			client: reqwest::Client::new(),
		}
	}

	/// Create a new HIBP client with default User-Agent.
	pub fn new_default(api_key: Option<String>) -> Self {
		HibpClient::new(
			"Dumptruck/1.0 (Cyber Threat Identification)".to_string(),
			api_key,
		)
	}

	/// Get the User-Agent string for this client.
	pub fn user_agent(&self) -> &str {
		&self.user_agent
	}

	/// Get the API key for this client (if set).
	pub fn api_key(&self) -> Option<&str> {
		self.api_key.as_deref()
	}

	/// Get all breaches for a given email address.
	///
	/// Returns a vector of breaches that included this email address.
	/// Empty vector means the address was not found in any known breach.
	pub async fn get_breaches_for_address(&self, email: &str) -> io::Result<Vec<Breach>> {
		let url = format!(
			"{}/breachedaccount/{}?includeUnverified=true",
			self.base_url,
			urlencoding::encode(email)
		);

		let mut req = self.client.get(&url).header("User-Agent", &self.user_agent);

		if let Some(key) = &self.api_key {
			req = req.header("hibp-api-key", key);
		}

		let resp = req
			.send()
			.await
			.map_err(|e| io::Error::other(e.to_string()))?;

		match resp.status() {
			reqwest::StatusCode::OK => {
				let body = resp
					.text()
					.await
					.map_err(|e| io::Error::other(e.to_string()))?;

				let breaches: Vec<Breach> = serde_json::from_str(&body)
					.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

				Ok(breaches)
			}
			reqwest::StatusCode::NOT_FOUND => {
				// Address not in any known breach
				Ok(vec![])
			}
			reqwest::StatusCode::BAD_REQUEST => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				"Invalid email address format",
			)),
			reqwest::StatusCode::TOO_MANY_REQUESTS => Err(io::Error::other(
				"HIBP API rate limit exceeded; retry after delay",
			)),
			status => Err(io::Error::other(
				format!("HIBP API error: {}", status),
			)),
		}
	}

	/// Check if an email address appears in any known breach.
	pub async fn is_breached(&self, email: &str) -> io::Result<bool> {
		let breaches = self.get_breaches_for_address(email).await?;
		Ok(!breaches.is_empty())
	}

	/// Get count of breaches for an email address.
	pub async fn breach_count(&self, email: &str) -> io::Result<i64> {
		let breaches = self.get_breaches_for_address(email).await?;
		Ok(breaches.len() as i64)
	}

	/// Get total number of credentials exposed across all breaches for an email.
	pub async fn pwn_count(&self, email: &str) -> io::Result<i64> {
		let breaches = self.get_breaches_for_address(email).await?;
		Ok(breaches.iter().map(|b| b.pwn_count).sum())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_hibp_client_creation() {
		let client = HibpClient::new_default(None);
		assert_eq!(
			client.user_agent,
			"Dumptruck/1.0 (Cyber Threat Identification)"
		);
		assert!(client.api_key.is_none());

		let client_with_key = HibpClient::new_default(Some("test-api-key".to_string()));
		assert_eq!(client_with_key.api_key, Some("test-api-key".to_string()));
	}

	#[test]
	fn test_hibp_client_custom_user_agent() {
		let client = HibpClient::new("CustomAgent/1.0".to_string(), None);
		assert_eq!(client.user_agent, "CustomAgent/1.0");
	}
}
