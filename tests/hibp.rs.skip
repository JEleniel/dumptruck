#[cfg(test)]
mod hibp_client_tests {
	use dumptruck::hibp::HibpClient;

	#[test]
	fn test_hibp_client_defaults() {
		let client = HibpClient::new_default(None);
		assert_eq!(
			client.user_agent(),
			"Dumptruck/1.0 (Cyber Threat Identification)"
		);
		assert!(client.api_key().is_none());
	}

	#[test]
	fn test_hibp_client_with_api_key() {
		let client = HibpClient::new_default(Some("test-key-123".to_string()));
		assert_eq!(client.api_key(), Some("test-key-123"));
	}

	#[test]
	fn test_hibp_client_custom_user_agent() {
		let client = HibpClient::new("MyApp/1.0".to_string(), None);
		assert_eq!(client.user_agent(), "MyApp/1.0");
	}
}
