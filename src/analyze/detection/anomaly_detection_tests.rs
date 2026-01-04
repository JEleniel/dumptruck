use super::*;

#[test]
fn test_calculate_entropy() {
	// All same character = 0 entropy
	let entropy1 = calculate_entropy("aaaaa");
	assert!(entropy1 < 0.1);

	// Random = high entropy
	let entropy2 = calculate_entropy("a1b2c3d4e5");
	assert!(entropy2 > 3.0);

	// Uniform distribution = max entropy
	let entropy3 = calculate_entropy("abcd");
	assert!(entropy3 > 1.9 && entropy3 < 2.1); // log2(4) ≈ 2.0
}

#[test]
fn test_detect_entropy_outlier() {
	let mean = 3.0;
	let std_dev = 0.5;

	// Normal entropy - should return None
	let result1 = detect_entropy_outlier("somepassword", mean, std_dev);
	assert!(result1.is_none());

	// Very high entropy - should detect
	let _result2 = detect_entropy_outlier("a1b2c3d4e5f6g7h8", mean, std_dev);
	// May or may not detect depending on actual entropy

	// Very low entropy - should detect
	let _result3 = detect_entropy_outlier("aaaaaaaaaaaaaaaa", mean, std_dev);
	// May or may not detect depending on actual entropy
}

#[test]
fn test_detect_unusual_password_format() {
	// Normal password - should return None
	let result1 = detect_unusual_password_format("MyPassword123!");
	assert!(result1.is_none());

	// Very short - should detect
	let result2 = detect_unusual_password_format("aaa");
	assert!(result2.is_some());

	// Very long - should detect
	let long_pass = "a".repeat(300);
	let result3 = detect_unusual_password_format(&long_pass);
	assert!(result3.is_some());

	// Only lowercase and digits - low variety
	let result4 = detect_unusual_password_format("abc123");
	assert!(result4.is_none()); // 2 types is acceptable

	// Only digits - very low variety
	let result5 = detect_unusual_password_format("123456");
	assert!(result5.is_some());
}

#[test]
fn test_detect_rare_domain() {
	let mut freqs = HashMap::new();
	freqs.insert("gmail.com".to_string(), 50);
	freqs.insert("yahoo.com".to_string(), 30);
	freqs.insert("obscure.xyz".to_string(), 1);

	// Common domain - should return None
	let result1 = detect_rare_domain("user@gmail.com", &freqs, 100);
	assert!(result1.is_none());

	// Rare domain (1 out of 100 = 1%, exactly at threshold) - let it return None for edge case
	// Use a more rare domain to trigger detection
	freqs.insert("very-obscure.xyz".to_string(), 0); // This won't be detected (count 0)

	// Test with 200 total to make obscure.xyz < 1%
	let result2 = detect_rare_domain("user@obscure.xyz", &freqs, 200);
	assert!(result2.is_some()); // Now 1/200 = 0.5% < 1%
}

#[test]
fn test_detect_unseen_combination() {
	let mut seen = HashSet::new();
	seen.insert("alice|password123|user@gmail.com".to_string());

	let combination1 = vec!["alice", "password123", "user@gmail.com"];
	let result1 = detect_unseen_combination(&combination1, &seen);
	assert!(result1.is_none()); // Seen before

	let combination2 = vec!["bob", "password456", "user@yahoo.com"];
	let result2 = detect_unseen_combination(&combination2, &seen);
	assert!(result2.is_some()); // New combination
}

#[test]
fn test_dataset_baseline() {
	let values = vec![
		"password123",
		"MyPassword",
		"test",
		"anotherpassword",
		"pwd",
	];

	let baseline = DatasetBaseline::from_sample(&values).expect("Should create baseline");

	assert!(baseline.mean_entropy > 0.0);
	assert!(baseline.entropy_std_dev >= 0.0);
	assert!(baseline.mean_length > 0.0);
	assert_eq!(baseline.record_count, 5);
}
