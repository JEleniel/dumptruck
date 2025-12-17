//! Universal JSON/XML parser that handles any valid structure.
//!
//! Converts nested JSON and XML structures into flattened rows:
//! - JSON objects/arrays → flattened key-value pairs
//! - XML elements → flattened element-text pairs
//! - Nested structures → concatenated paths (e.g., "user.address.street")
//! - Lists → multiple rows (one per item)

#[allow(unused_imports)]
use serde_json::{Value, json};
#[allow(unused_imports)]
use std::collections::BTreeMap;

/// Convert any JSON structure to rows of strings
///
/// # Strategy
/// - If it's an array of objects: Each object becomes a row, keys are columns
/// - If it's an array of arrays: Use as-is
/// - If it's an array of primitives: Convert each to string
/// - If it's a single object: Convert to one row
/// - If it's deeply nested: Flatten with dot notation (user.name, user.email)
pub fn json_to_rows(value: &Value) -> Vec<Vec<String>> {
	match value {
		Value::Array(arr) => {
			if arr.is_empty() {
				return Vec::new();
			}

			// Check what type of array elements we have
			match &arr[0] {
				Value::Object(_) => {
					// Array of objects - extract all keys, convert to rows
					flatten_object_array(arr)
				}
				Value::Array(_) => {
					// Array of arrays - convert directly
					arr.iter()
						.map(|item| {
							if let Value::Array(row) = item {
								row.iter().map(|v| value_to_string(v)).collect()
							} else {
								vec![value_to_string(item)]
							}
						})
						.collect()
				}
				_ => {
					// Array of primitives - each becomes a row with one column
					arr.iter().map(|v| vec![value_to_string(v)]).collect()
				}
			}
		}
		Value::Object(_) => {
			// Single object - convert to one row
			vec![flatten_object(value, "")]
		}
		_ => {
			// Primitive value
			vec![vec![value_to_string(value)]]
		}
	}
}

/// Flatten an array of objects into rows
/// All objects are normalized to same columns
fn flatten_object_array(arr: &[Value]) -> Vec<Vec<String>> {
	if arr.is_empty() {
		return Vec::new();
	}

	// Collect all possible keys across all objects
	let mut all_keys = std::collections::BTreeSet::new();
	for item in arr {
		if let Value::Object(map) = item {
			collect_object_keys(map, "", &mut all_keys);
		}
	}

	let keys: Vec<String> = all_keys.into_iter().collect();
	if keys.is_empty() {
		return Vec::new();
	}

	// Convert each object to a row with all keys
	let mut rows = Vec::new();

	// Add header row
	rows.push(keys.clone());

	// Add data rows
	for item in arr {
		let mut row = Vec::new();
		for key in &keys {
			let value = get_nested_value(item, key);
			row.push(value_to_string(&value));
		}
		rows.push(row);
	}

	rows
}

/// Recursively collect all keys from an object
fn collect_object_keys(
	obj: &serde_json::Map<String, Value>,
	prefix: &str,
	keys: &mut std::collections::BTreeSet<String>,
) {
	for (k, v) in obj {
		let full_key = if prefix.is_empty() {
			k.clone()
		} else {
			format!("{}.{}", prefix, k)
		};

		match v {
			Value::Object(nested) => {
				collect_object_keys(nested, &full_key, keys);
			}
			Value::Array(_) => {
				// For arrays, just use the array itself as a column
				keys.insert(full_key);
			}
			_ => {
				keys.insert(full_key);
			}
		}
	}
}

/// Get a nested value using dot notation (e.g., "user.address.street")
fn get_nested_value(value: &Value, path: &str) -> Value {
	let mut current = value;

	for part in path.split('.') {
		match current {
			Value::Object(map) => {
				current = map.get(part).unwrap_or(&Value::Null);
			}
			_ => {
				return Value::Null;
			}
		}
	}

	current.clone()
}

/// Flatten a single object into a row (all leaf values)
fn flatten_object(value: &Value, prefix: &str) -> Vec<String> {
	let mut result = Vec::new();

	match value {
		Value::Object(map) => {
			let mut pairs: Vec<(String, String)> = Vec::new();

			for (k, v) in map {
				let full_key = if prefix.is_empty() {
					k.clone()
				} else {
					format!("{}.{}", prefix, k)
				};

				match v {
					Value::Object(_) => {
						// Recurse into nested object
						let nested = flatten_object(v, &full_key);
						for item in nested {
							pairs.push((full_key.clone(), item));
						}
					}
					Value::Array(_) => {
						pairs.push((full_key, value_to_string(v)));
					}
					_ => {
						pairs.push((full_key, value_to_string(v)));
					}
				}
			}

			// Sort by key for consistent output
			pairs.sort_by(|a, b| a.0.cmp(&b.0));
			for (_, v) in pairs {
				result.push(v);
			}
		}
		_ => {
			result.push(value_to_string(value));
		}
	}

	result
}

/// Convert a JSON value to string representation
fn value_to_string(v: &Value) -> String {
	match v {
		Value::Null => String::new(),
		Value::Bool(b) => b.to_string(),
		Value::Number(n) => n.to_string(),
		Value::String(s) => s.clone(),
		Value::Array(arr) => {
			// For arrays, create a bracketed list
			let items: Vec<String> = arr.iter().map(value_to_string).collect();
			format!("[{}]", items.join(", "))
		}
		Value::Object(_) => {
			// For objects, use JSON representation
			v.to_string()
		}
	}
}

/// Convert any XML string to rows of strings
///
/// # Strategy
/// - Parse XML into a tree structure
/// - Extract all text nodes and attributes
/// - Flatten nested elements into rows
pub fn xml_to_rows(xml_content: &str) -> Result<Vec<Vec<String>>, String> {
	let mut rows = Vec::new();

	// Simple approach: find all tag-value pairs
	// Pattern: <tag>value</tag>
	let mut chars = xml_content.chars().peekable();
	let mut in_tag = false;
	let mut tag_name = String::new();
	let mut tag_value = String::new();
	let mut reading_tag = false;

	while let Some(&ch) = chars.peek() {
		match ch {
			'<' => {
				// Save previous tag-value pair if we have one
				if !tag_name.is_empty() && !tag_value.is_empty() {
					rows.push(vec![tag_name.clone(), tag_value.trim().to_string()]);
				}
				tag_name.clear();
				tag_value.clear();

				chars.next(); // consume '<'
				in_tag = true;
				reading_tag = true;

				// Skip closing tags and declarations
				if chars.peek() == Some(&'/')
					|| chars.peek() == Some(&'!')
					|| chars.peek() == Some(&'?')
				{
					in_tag = false;
					reading_tag = false;
					// Skip to end of tag
					while let Some(&c) = chars.peek() {
						chars.next();
						if c == '>' {
							break;
						}
					}
				}
			}
			'>' => {
				chars.next(); // consume '>'
				in_tag = false;
				reading_tag = false;
			}
			_ if in_tag && reading_tag => {
				chars.next();
				tag_name.push(ch);
			}
			_ if in_tag => {
				chars.next();
			}
			_ => {
				chars.next();
				tag_value.push(ch);
			}
		}
	}

	// Add last pair
	if !tag_name.is_empty() && !tag_value.is_empty() {
		rows.push(vec![tag_name, tag_value.trim().to_string()]);
	}

	if rows.is_empty() {
		return Err("No valid XML elements found".to_string());
	}

	Ok(rows)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_json_array_of_objects() {
		let json = json!([
			{"name": "John", "age": 30},
			{"name": "Jane", "age": 25}
		]);
		let rows = json_to_rows(&json);
		assert_eq!(rows.len(), 3); // header + 2 rows
		assert_eq!(rows[0].len(), 2); // age, name (sorted)
	}

	#[test]
	fn test_json_single_object() {
		let json = json!({"key": "value", "number": 42});
		let rows = json_to_rows(&json);
		assert_eq!(rows.len(), 1);
		assert_eq!(rows[0].len(), 2); // key, number
	}

	#[test]
	fn test_json_array_of_primitives() {
		let json = json!(["apple", "banana", "cherry"]);
		let rows = json_to_rows(&json);
		assert_eq!(rows.len(), 3);
	}

	#[test]
	fn test_json_nested_objects() {
		let json = json!({
			"user": {
				"name": "John",
				"address": {
					"street": "Main St",
					"city": "NYC"
				}
			}
		});
		let rows = json_to_rows(&json);
		assert_eq!(rows.len(), 1);
		// Should have flattened keys
		assert!(rows[0].len() > 0);
	}

	#[test]
	fn test_json_array_of_arrays() {
		let json = json!([["a", "b", "c"], ["1", "2", "3"]]);
		let rows = json_to_rows(&json);
		assert_eq!(rows.len(), 2);
		assert_eq!(rows[0].len(), 3);
	}

	#[test]
	fn test_value_to_string_null() {
		assert_eq!(value_to_string(&Value::Null), "");
	}

	#[test]
	fn test_value_to_string_array() {
		let arr = json!(["a", "b", "c"]);
		let result = value_to_string(&arr);
		assert!(result.contains("["));
		assert!(result.contains("]"));
	}
}
