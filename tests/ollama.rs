#[cfg(test)]
mod ollama_client_tests {
	use dumptruck::enrichment::ollama::OllamaClient;

	#[test]
	fn test_ollama_client_defaults() {
		let client = OllamaClient::new(None, None);
		assert_eq!(client.base_url(), "http://localhost:11435");
		assert_eq!(client.model(), "nomic-embed-text");
	}

	#[tokio::test]
	async fn test_ollama_client_custom_config() {
		let client = OllamaClient::new(
			Some("http://custom:11435".to_string()),
			Some("custom-model".to_string()),
		);
		// Verify custom configuration was applied
		assert_eq!(client.base_url(), "http://custom:11435");
		assert_eq!(client.model(), "custom-model");
	}
}
