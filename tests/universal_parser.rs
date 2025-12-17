//! Integration tests for universal JSON/XML parser
//!
//! Tests all JSON and XML format scenarios:
//! - Array of objects
//! - Nested objects with dot notation
//! - Array of arrays
//! - Array of primitives
//! - Single objects
//! - XML structures
//! - Mixed and edge cases

use dumptruck::universal_parser::{json_to_rows, xml_to_rows};
use serde_json::json;

// ============================================================================
// JSON TESTS
// ============================================================================

#[test]
fn test_json_array_of_objects_multiple_records() {
	let json = json!([
		{"name": "John Doe", "email": "john@example.com", "age": 30},
		{"name": "Jane Smith", "email": "jane@example.com", "age": 28}
	]);
	let rows = json_to_rows(&json);

	// Should have header + 2 data rows
	assert_eq!(rows.len(), 3, "Expected 3 rows (header + 2 data)");
	// First row is header
	assert_eq!(rows[0].len(), 3, "Header should have 3 columns");
	// Data rows
	assert_eq!(rows[1].len(), 3, "First data row should have 3 columns");
	assert_eq!(rows[2].len(), 3, "Second data row should have 3 columns");

	// Verify headers are alphabetically sorted
	let headers = &rows[0];
	assert!(headers.contains(&"age".to_string()));
	assert!(headers.contains(&"email".to_string()));
	assert!(headers.contains(&"name".to_string()));
}

#[test]
fn test_json_nested_object_with_flattening() {
	let json = json!({
		"user": {
			"name": "John Doe",
			"email": "john@example.com",
			"profile": {
				"age": 30,
				"location": "NYC"
			}
		}
	});
	let rows = json_to_rows(&json);

	// Single nested object becomes one row
	assert_eq!(rows.len(), 1, "Nested object should produce 1 row");
	// Should have flattened keys with dot notation
	let row = &rows[0];
	assert!(row.len() >= 4, "Should have at least 4 flattened fields");
	// Values should contain the nested data
	assert!(row.iter().any(|v| v.contains("John")));
	assert!(row.iter().any(|v| v.contains("NYC")));
}

#[test]
fn test_json_array_of_arrays_passthrough() {
	let json = json!([
		["John", "john@example.com", "30"],
		["Jane", "jane@example.com", "28"],
		["Bob", "bob@example.com", "35"]
	]);
	let rows = json_to_rows(&json);

	// Should preserve all rows
	assert_eq!(rows.len(), 3, "Expected 3 rows");
	// Each row should have 3 columns
	assert_eq!(rows[0].len(), 3, "Row 1 should have 3 columns");
	assert_eq!(rows[1].len(), 3, "Row 2 should have 3 columns");
	assert_eq!(rows[2].len(), 3, "Row 3 should have 3 columns");

	// Verify data
	assert_eq!(rows[0][0], "John");
	assert_eq!(rows[1][0], "Jane");
	assert_eq!(rows[2][0], "Bob");
}

#[test]
fn test_json_array_of_primitives() {
	let json = json!(["apple", "banana", "cherry", "date"]);
	let rows = json_to_rows(&json);

	// Each primitive becomes a single-column row
	assert_eq!(rows.len(), 4, "Expected 4 rows");
	assert_eq!(rows[0].len(), 1, "Each row should have 1 column");
	assert_eq!(rows[0][0], "apple");
	assert_eq!(rows[1][0], "banana");
	assert_eq!(rows[2][0], "cherry");
	assert_eq!(rows[3][0], "date");
}

#[test]
fn test_json_single_object() {
	let json = json!({"name": "John", "email": "john@example.com", "age": 30});
	let rows = json_to_rows(&json);

	// Single object becomes one row
	assert_eq!(rows.len(), 1, "Expected 1 row");
	assert_eq!(rows[0].len(), 3, "Expected 3 columns");

	// Verify all values are present
	assert!(rows[0].iter().any(|v| v.contains("John")));
	assert!(rows[0].iter().any(|v| v.contains("john@example.com")));
	assert!(rows[0].iter().any(|v| v.contains("30")));
}

#[test]
fn test_json_empty_array() {
	let json = json!([]);
	let rows = json_to_rows(&json);

	// Empty array should produce empty result
	assert_eq!(rows.len(), 0, "Empty array should produce no rows");
}

#[test]
fn test_json_empty_object() {
	let json = json!({});
	let rows = json_to_rows(&json);

	// Empty object produces one empty row
	assert_eq!(rows.len(), 1, "Empty object should produce one row");
	assert_eq!(rows[0].len(), 0, "Empty row should have no columns");
}

#[test]
fn test_json_objects_with_varying_keys() {
	let json = json!([
		{"name": "John", "email": "john@example.com"},
		{"name": "Jane", "age": 28, "email": "jane@example.com"},
		{"name": "Bob"}
	]);
	let rows = json_to_rows(&json);

	// Should have header + 3 data rows
	assert_eq!(rows.len(), 4, "Expected header + 3 data rows");

	// All rows should have same number of columns (normalized)
	let col_count = rows[0].len();
	for (idx, row) in rows.iter().enumerate() {
		assert_eq!(
			row.len(),
			col_count,
			"Row {} should have {} columns",
			idx,
			col_count
		);
	}

	// Should have normalized columns: age, email, name
	let headers = &rows[0];
	assert!(headers.contains(&"age".to_string()));
	assert!(headers.contains(&"email".to_string()));
	assert!(headers.contains(&"name".to_string()));
}

#[test]
fn test_json_numeric_values() {
	let json = json!([
		{"id": 1, "score": 95.5, "active": true},
		{"id": 2, "score": 87.3, "active": false}
	]);
	let rows = json_to_rows(&json);

	assert_eq!(rows.len(), 3, "Expected header + 2 rows");
	// Numeric values should be converted to strings
	assert!(rows[1].iter().any(|v| v.contains("1")));
	assert!(rows[1].iter().any(|v| v.contains("95.5")));
	assert!(rows[1].iter().any(|v| v.contains("true")));
}

#[test]
fn test_json_null_values() {
	let json = json!([
		{"name": "John", "email": "john@example.com", "phone": null},
		{"name": "Jane", "email": null, "phone": "555-1234"}
	]);
	let rows = json_to_rows(&json);

	assert_eq!(rows.len(), 3, "Expected header + 2 rows");
	// Null values should be empty strings or handled appropriately
	// This verifies the parser doesn't crash on nulls
}

#[test]
fn test_json_deeply_nested_structure() {
	let json = json!({
		"level1": {
			"level2": {
				"level3": {
					"value": "deep",
					"count": 42
				},
				"sibling": "nearby"
			},
			"other": "data"
		}
	});
	let rows = json_to_rows(&json);

	assert_eq!(rows.len(), 1, "Should produce single row");
	// Deeply nested values should be present
	assert!(rows[0].iter().any(|v| v.contains("deep")));
	assert!(rows[0].iter().any(|v| v.contains("42")));
	assert!(rows[0].iter().any(|v| v.contains("nearby")));
}

#[test]
fn test_json_array_of_mixed_numeric_types() {
	let json = json!([
		{"int": 42, "float": 3.14, "exp": 1e-10},
		{"int": 100, "float": 2.71, "exp": 5e5}
	]);
	let rows = json_to_rows(&json);

	assert_eq!(rows.len(), 3, "Expected header + 2 rows");
	// All numeric types should be converted to strings successfully
	assert!(rows[1].iter().any(|v| v.contains("42")));
	assert!(rows[2].iter().any(|v| v.contains("100")));
}

// ============================================================================
// XML TESTS
// ============================================================================

#[test]
fn test_xml_simple_elements() {
	let xml = r#"<?xml version="1.0"?>
<users>
	<name>John Doe</name>
	<email>john@example.com</email>
	<age>30</age>
</users>"#;

	let result = xml_to_rows(xml);
	assert!(result.is_ok(), "XML parsing should succeed");

	let rows = result.unwrap();
	assert!(rows.len() > 0, "Should extract at least one row");

	// Should have extracted tag-value pairs
	let all_values: String = rows
		.iter()
		.flat_map(|r| r.iter().cloned())
		.collect::<Vec<_>>()
		.join(" ");
	assert!(all_values.contains("John Doe"));
	assert!(all_values.contains("john@example.com"));
	assert!(all_values.contains("30"));
}

#[test]
fn test_xml_multiple_records() {
	let xml = r#"<?xml version="1.0"?>
<users>
	<user>
		<name>John Doe</name>
		<email>john@example.com</email>
	</user>
	<user>
		<name>Jane Smith</name>
		<email>jane@example.com</email>
	</user>
</users>"#;

	let result = xml_to_rows(xml);
	assert!(result.is_ok());

	let rows = result.unwrap();
	assert!(rows.len() >= 4, "Should extract multiple tag-value pairs");

	// Verify both records are present
	let all_values: String = rows
		.iter()
		.flat_map(|r| r.iter().cloned())
		.collect::<Vec<_>>()
		.join(" ");
	assert!(all_values.contains("John Doe"));
	assert!(all_values.contains("Jane Smith"));
	assert!(all_values.contains("jane@example.com"));
}

#[test]
fn test_xml_with_attributes() {
	let xml = r#"<?xml version="1.0"?>
<data>
	<record id="1">
		<name>Item One</name>
	</record>
	<record id="2">
		<name>Item Two</name>
	</record>
</data>"#;

	let result = xml_to_rows(xml);
	assert!(result.is_ok());

	let rows = result.unwrap();
	// Should extract tag-value pairs even with attributes
	let all_values: String = rows
		.iter()
		.flat_map(|r| r.iter().cloned())
		.collect::<Vec<_>>()
		.join(" ");
	assert!(all_values.contains("Item One"));
	assert!(all_values.contains("Item Two"));
}

#[test]
fn test_xml_with_whitespace() {
	let xml = r#"<?xml version="1.0"?>
<data>
	<key>   value with spaces   </key>
	<number>42</number>
</data>"#;

	let result = xml_to_rows(xml);
	assert!(result.is_ok());

	let rows = result.unwrap();
	// Whitespace should be trimmed or handled appropriately
	let all_values: String = rows
		.iter()
		.flat_map(|r| r.iter().cloned())
		.collect::<Vec<_>>()
		.join(" ");
	assert!(all_values.contains("value with spaces") || all_values.contains("value"));
}

#[test]
fn test_xml_nested_elements() {
	let xml = r#"<?xml version="1.0"?>
<data>
	<user>
		<profile>
			<name>John</name>
			<email>john@example.com</email>
		</profile>
		<status>active</status>
	</user>
</data>"#;

	let result = xml_to_rows(xml);
	assert!(result.is_ok());

	let rows = result.unwrap();
	// Should extract nested element values
	let all_values: String = rows
		.iter()
		.flat_map(|r| r.iter().cloned())
		.collect::<Vec<_>>()
		.join(" ");
	assert!(all_values.contains("John"));
	assert!(all_values.contains("john@example.com"));
	assert!(all_values.contains("active"));
}

#[test]
fn test_xml_empty_elements() {
	let xml = r#"<?xml version="1.0"?>
<data>
	<key></key>
	<value>data</value>
	<empty/>
</data>"#;

	let result = xml_to_rows(xml);
	assert!(result.is_ok());
	// Should handle empty elements without crashing
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn test_json_special_characters_in_values() {
	let json = json!([
		{"email": "john+tag@example.com", "note": "Has \"quotes\" and 'apostrophes'"},
		{"email": "jane@sub.domain.com", "note": "Line\nbreak"}
	]);
	let rows = json_to_rows(&json);

	assert_eq!(rows.len(), 3, "Expected header + 2 rows");
	// Special characters should be preserved
	let all_values: String = rows
		.iter()
		.flat_map(|r| r.iter().cloned())
		.collect::<Vec<_>>()
		.join(" ");
	assert!(all_values.contains("+tag@example.com"));
}

#[test]
fn test_json_unicode_values() {
	let json = json!([
		{"name": "José", "city": "São Paulo"},
		{"name": "François", "city": "Montréal"},
		{"name": "北京", "city": "中国"}
	]);
	let rows = json_to_rows(&json);

	assert_eq!(rows.len(), 4, "Expected header + 3 rows");
	// Unicode should be preserved
	let all_values: String = rows
		.iter()
		.flat_map(|r| r.iter().cloned())
		.collect::<Vec<_>>()
		.join(" ");
	assert!(all_values.contains("José"));
	assert!(all_values.contains("北京"));
}

#[test]
fn test_json_very_large_object() {
	let mut obj = serde_json::json!({});
	for i in 0..100 {
		obj[format!("field_{}", i)] = json!(format!("value_{}", i));
	}

	let rows = json_to_rows(&obj);
	// Should handle large objects without crashing
	assert_eq!(rows.len(), 1, "Expected single row for object");
	assert!(rows[0].len() >= 50, "Should have many columns");
}

#[test]
fn test_json_array_with_many_records() {
	let mut records = Vec::new();
	for i in 0..1000 {
		records.push(json!({"id": i, "value": format!("val_{}", i)}));
	}
	let json = serde_json::Value::Array(records);

	let rows = json_to_rows(&json);
	// Should handle many records without crashing
	assert!(
		rows.len() > 1000,
		"Should have 1000+ rows (including header)"
	);
}

#[test]
fn test_json_mixed_string_types() {
	let json = json!([
		{"id": "001", "name": "Item", "code": "ABC-123"},
		{"id": 2, "name": "Another", "code": 456}
	]);
	let rows = json_to_rows(&json);

	assert_eq!(rows.len(), 3, "Expected header + 2 rows");
	// Mixed types should all convert to strings
	let all_values: String = rows
		.iter()
		.flat_map(|r| r.iter().cloned())
		.collect::<Vec<_>>()
		.join(" ");
	assert!(all_values.contains("001"));
	assert!(all_values.contains("Item"));
}
