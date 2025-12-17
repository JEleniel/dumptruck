//! Integration test for rainbow table file change detection

#[test]
fn test_rainbow_table_file_signatures_exist() {
    use std::fs;
    use std::path::Path;

    let cache_path = ".cache/rainbow_table.json";

    // Verify the cache file exists
    assert!(
        Path::new(cache_path).exists(),
        "Rainbow table cache should exist at {}",
        cache_path
    );

    // Read and parse the JSON
    let content = fs::read_to_string(cache_path).expect("Failed to read cache file");
    let table: dumptruck::rainbow_table_builder::RainbowTableJson =
        serde_json::from_str(&content).expect("Failed to parse rainbow table JSON");

    // Verify version
    assert_eq!(table.version, 1, "Rainbow table version should be 1");

    // Verify file signatures exist
    assert!(
        !table.file_signatures.is_empty(),
        "File signatures should not be empty"
    );

    // Verify common files are tracked
    let has_passwords = table
        .file_signatures
        .keys()
        .any(|k| k.contains("password"));
    assert!(has_passwords, "Should track at least one password file");

    // Verify entries exist
    assert!(
        !table.entries.is_empty(),
        "Rainbow table should have entries"
    );

    // Verify entries have all hash fields
    for entry in table.entries.iter().take(10) {
        assert!(!entry.plaintext.is_empty(), "Plaintext should not be empty");
        assert!(!entry.md5.is_empty(), "MD5 should not be empty");
        assert!(!entry.sha1.is_empty(), "SHA1 should not be empty");
        assert!(!entry.sha256.is_empty(), "SHA256 should not be empty");
        assert!(!entry.sha512.is_empty(), "SHA512 should not be empty");
        assert!(!entry.ntlm.is_empty(), "NTLM should not be empty");
    }
}

#[test]
fn test_rainbow_table_builder_change_detection() {
	use dumptruck::rainbow_table_builder::RainbowTableBuilder;

	let builder = RainbowTableBuilder::new();    // Load current table (should use cache if files haven't changed)
    let result = builder.load();
    assert!(result.is_ok(), "Should be able to load rainbow table");

    let table = result.unwrap();
    assert_eq!(table.version, 1, "Table version should be 1");

    // Verify signatures are present for change detection
    assert!(
        !table.file_signatures.is_empty(),
        "File signatures for change detection should be present"
    );

    // Verify entries exist
    assert!(
        !table.entries.is_empty(),
        "Table should have password entries"
    );

    println!(
        "[✓] Rainbow table loaded: {} entries, {} tracked files",
        table.entries.len(),
        table.file_signatures.len()
    );
}
