//! Ollama/Nomic embedding client for vector-based deduplication.
//!
//! This module provides async HTTP client for generating Nomic-based embeddings
//! from the Ollama API, which are used for near-duplicate detection via pgvector
//! similarity search.

use std::io;

/// Ollama embedding request payload.
#[derive(serde::Serialize)]
pub struct EmbedRequest {
	pub model: String,
	pub input: String,
}

/// Ollama embedding response payload.
#[derive(serde::Deserialize)]
pub struct EmbedResponse {
	pub embedding: Vec<f32>,
}

/// Async HTTP client for Ollama API.
pub struct OllamaClient {
	base_url: String,
	model: String,
	client: reqwest::Client,
}

impl OllamaClient {
	/// Create a new Ollama client pointing to the given base URL and model.
	/// Default base_url is http://localhost:11435, model is "nomic-embed-text".
	pub fn new(base_url: Option<String>, model: Option<String>) -> Self {
		OllamaClient {
			base_url: base_url.unwrap_or_else(|| "http://localhost:11435".to_string()),
			model: model.unwrap_or_else(|| "nomic-embed-text".to_string()),
			client: reqwest::Client::new(),
		}
	}

	/// Get the base URL for this Ollama client.
	pub fn base_url(&self) -> &str {
		&self.base_url
	}

	/// Get the model name for this Ollama client.
	pub fn model(&self) -> &str {
		&self.model
	}

	/// Generate a 768-dimensional embedding for the given text using Nomic.
	pub async fn embed(&self, text: &str) -> io::Result<Vec<f32>> {
		let url = format!("{}/api/embed", self.base_url);
		let req = EmbedRequest {
			model: self.model.clone(),
			input: text.to_string(),
		};

		let resp = self
			.client
			.post(&url)
			.json(&req)
			.send()
			.await
			.map_err(|e| io::Error::other(e.to_string()))?;

		if !resp.status().is_success() {
			return Err(io::Error::other(
				format!("Ollama API error: {}", resp.status()),
			));
		}

		let body = resp
			.text()
			.await
			.map_err(|e| io::Error::other(e.to_string()))?;

		let embed_resp: EmbedResponse = serde_json::from_str(&body)
			.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

		Ok(embed_resp.embedding)
	}

	/// Check if the Ollama service is available (health check).
	pub async fn health_check(&self) -> io::Result<bool> {
		let url = format!("{}/api/tags", self.base_url);
		match self.client.get(&url).send().await {
			Ok(resp) => Ok(resp.status().is_success()),
			Err(_) => Ok(false),
		}
	}

	/// Pull the Nomic embedding model if not already available.
	pub async fn ensure_model(&self) -> io::Result<()> {
		let url = format!("{}/api/pull", self.base_url);
		let req = serde_json::json!({"name": self.model});

		let resp = self
			.client
			.post(&url)
			.json(&req)
			.send()
			.await
			.map_err(|e| io::Error::other(e.to_string()))?;

		if !resp.status().is_success() {
			return Err(io::Error::other(
				format!("Failed to pull Nomic model: {}", resp.status()),
			));
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ollama_client_new() {
		let client = OllamaClient::new(None, None);
		assert_eq!(client.base_url, "http://localhost:11435");
		assert_eq!(client.model, "nomic-embed-text");

		let client = OllamaClient::new(
			Some("http://example.com:11435".to_string()),
			Some("custom-model".to_string()),
		);
		assert_eq!(client.base_url, "http://example.com:11435");
		assert_eq!(client.model, "custom-model");
	}
}
