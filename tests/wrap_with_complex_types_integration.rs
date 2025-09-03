//! Integration tests for wrap-with command with complex types
//! Tests the new to_parsable_string() functionality

use nu_plugin_secret::{SecretBinary, SecretList, SecretRecord};
use nu_protocol::{Record, Span, Value};

#[test]
fn test_secret_record_with_custom_template_uses_parsable_string() {
    // Create a record with mixed types
    let mut record = Record::new();
    record.push("name", Value::test_string("Alice"));
    record.push("age", Value::test_int(30));
    record.push("active", Value::test_bool(true));

    // Create SecretRecord with custom template that uses secret_string()
    let secret =
        SecretRecord::new_with_template(record.clone(), "Record: {{secret_string()}}".to_string());

    // Test Display implementation
    let display = format!("{}", secret);

    // Should contain parsable Nu syntax, not debug format
    assert!(display.contains("Record: "));
    assert!(display.contains("name:"));
    assert!(display.contains("Alice"));
    assert!(display.contains("age:"));
    assert!(display.contains("30"));
    assert!(display.contains("active:"));

    // Should NOT contain debug format like "Record { ... }"
    assert!(!display.contains("Record {"));

    // Test Debug implementation
    let debug = format!("{:?}", secret);
    assert!(debug.starts_with("SecretRecord("));
    assert!(debug.contains("name:"));
    assert!(debug.contains("Alice"));

    // Test unwrap still works
    let revealed = secret.reveal();
    assert_eq!(revealed.get("name").unwrap(), &Value::test_string("Alice"));
    assert_eq!(revealed.get("age").unwrap(), &Value::test_int(30));
    assert_eq!(revealed.get("active").unwrap(), &Value::test_bool(true));
}

#[test]
fn test_secret_list_with_custom_template_uses_parsable_string() {
    // Create a list with mixed types
    let list = vec![
        Value::test_string("apple"),
        Value::test_string("banana"),
        Value::test_int(42),
        Value::test_bool(true),
    ];

    // Create SecretList with custom template
    let secret =
        SecretList::new_with_template(list.clone(), "List: {{secret_string()}}".to_string());

    // Test Display implementation
    let display = format!("{}", secret);

    // Should contain parsable Nu syntax, not debug format
    assert!(display.contains("List: "));
    assert!(display.contains("apple"));
    assert!(display.contains("banana"));
    assert!(display.contains("42"));
    assert!(display.contains("true"));

    // Should NOT contain debug format like "Vec [ ... ]"
    assert!(!display.contains("Vec ["));

    // Should contain Nu list syntax
    assert!(display.contains("[")); // Nu list opening bracket

    // Test unwrap still works
    let revealed = secret.reveal();
    assert_eq!(revealed.len(), 4);
    assert_eq!(revealed[0], Value::test_string("apple"));
    assert_eq!(revealed[2], Value::test_int(42));
    assert_eq!(revealed[3], Value::test_bool(true));
}

#[test]
fn test_secret_binary_with_custom_template_uses_parsable_string() {
    // Create binary data
    let binary_data = vec![0xde, 0xad, 0xbe, 0xef, 0x42];

    // Create SecretBinary with custom template
    let secret = SecretBinary::new_with_template(
        binary_data.clone(),
        "Binary: {{secret_string()}}".to_string(),
    );

    // Test Display implementation
    let display = format!("{}", secret);

    // Should contain parsable Nu binary syntax, not hex string
    assert!(display.contains("Binary: "));
    // Nu's to_parsable_string() for binary shows decimal array format
    assert!(display.contains("[")); // Array opening bracket
    assert!(display.contains("222")); // First byte (0xde) in decimal
    assert!(display.contains("173")); // Second byte (0xad) in decimal
    assert!(display.contains("190")); // Third byte (0xbe) in decimal
    assert!(display.contains("239")); // Fourth byte (0xef) in decimal
    assert!(display.contains("66")); // Fifth byte (0x42) in decimal
    assert!(display.contains("]")); // Array closing bracket

    // Should NOT contain plain hex string format
    assert!(!display.starts_with("deadbeef42")); // Should not start with raw hex

    // Test unwrap still works
    let revealed = secret.reveal();
    assert_eq!(&revealed[..], binary_data.as_slice());
}

#[test]
fn test_complex_types_template_with_functions() {
    // Test record with template functions
    let mut record = Record::new();
    record.push("config", Value::test_string("production"));
    record.push("port", Value::test_int(8080));

    let secret = SecretRecord::new_with_template(
        record,
        "{{secret_type | upper}}: {{take(n=20, s=secret_string())}}...".to_string(),
    );

    let display = format!("{}", secret);
    assert!(display.contains("RECORD:"));
    // Should show truncated parsable content due to take(n=20)
    assert!(display.contains("config:"));
    assert!(display.ends_with("..."));

    // Test list with reverse function
    let list = vec![Value::test_string("item1"), Value::test_string("item2")];
    let list_secret = SecretList::new_with_template(
        list,
        "{{reverse(s=secret_type)}}: {{secret_string()}}".to_string(),
    );

    let list_display = format!("{}", list_secret);
    assert!(list_display.contains("tsil:")); // "list" reversed
    assert!(list_display.contains("item1"));
    assert!(list_display.contains("item2"));
}

#[test]
fn test_empty_complex_types_with_templates() {
    // Empty record
    let empty_record = Record::new();
    let secret = SecretRecord::new_with_template(
        empty_record,
        "Empty {{secret_type}}: {{secret_string()}}".to_string(),
    );

    let display = format!("{}", secret);
    assert!(display.contains("Empty record:"));
    // Empty record should still produce valid Nu syntax
    assert!(display.contains("{"));
    assert!(display.contains("}"));

    // Empty list
    let empty_list = Vec::new();
    let list_secret = SecretList::new_with_template(
        empty_list,
        "Empty {{secret_type}}: {{secret_string()}}".to_string(),
    );

    let list_display = format!("{}", list_secret);
    assert!(list_display.contains("Empty list:"));
    // Empty list should produce valid Nu syntax
    assert!(list_display.contains("[]"));

    // Empty binary
    let empty_binary = Vec::new();
    let binary_secret = SecretBinary::new_with_template(
        empty_binary,
        "Empty {{secret_type}}: {{secret_string()}}".to_string(),
    );

    let binary_display = format!("{}", binary_secret);
    assert!(binary_display.contains("Empty binary:"));
    // Empty binary should produce valid Nu syntax (decimal array format)
    assert!(binary_display.contains("[]"));
}

#[test]
fn test_nested_complex_types_with_templates() {
    // Create a record containing a list
    let mut record = Record::new();
    let nested_list = vec![Value::test_string("nested_item1"), Value::test_int(123)];
    record.push("data", Value::list(nested_list, Span::test_data()));
    record.push("enabled", Value::test_bool(true));

    let secret = SecretRecord::new_with_template(
        record,
        "Nested[{{secret_type}}]: {{secret_string()}}".to_string(),
    );

    let display = format!("{}", secret);
    assert!(display.contains("Nested[record]:"));
    assert!(display.contains("data:"));
    assert!(display.contains("nested_item1"));
    assert!(display.contains("123"));
    assert!(display.contains("enabled:"));
    assert!(display.contains("true"));

    // Should maintain proper Nu syntax structure for nested data
    assert!(display.contains("[")); // For the nested list
    assert!(display.contains("]"));
}

#[test]
fn test_complex_types_serialization_with_templates() {
    // Test that templates persist through serialization
    let mut record = Record::new();
    record.push("api_key", Value::test_string("secret123"));

    let secret = SecretRecord::new_with_template(
        record,
        "Serialized[{{secret_type}}]: {{secret_string()}}".to_string(),
    );

    // Serialize to JSON
    let json = serde_json::to_string(&secret).expect("Should serialize");

    // Deserialize back
    let restored: SecretRecord = serde_json::from_str(&json).expect("Should deserialize");

    // Template should persist and still use parsable string
    let display = format!("{}", restored);
    assert!(display.contains("Serialized[record]:"));
    assert!(display.contains("api_key:"));
    assert!(display.contains("secret123"));

    // Should still be parsable Nu syntax
    assert!(display.contains("{"));
    assert!(display.contains("}"));
}

#[test]
fn test_performance_with_complex_type_templates() {
    use std::time::Instant;

    // Create moderately complex data
    let mut large_record = Record::new();
    for i in 0..20 {
        large_record.push(
            format!("field_{}", i),
            Value::test_string(format!("value_{}", i)),
        );
    }

    let template = "Large[{{secret_type}}]: {{take(n=50, s=secret_string())}}...";

    let start = Instant::now();

    // Create multiple secrets with templates
    let secrets: Vec<_> = (0..100)
        .map(|_| SecretRecord::new_with_template(large_record.clone(), template.to_string()))
        .collect();

    let duration = start.elapsed();

    // Should be reasonably fast (less than 10 seconds for 100 complex templates)
    assert!(
        duration.as_secs() < 10,
        "Template processing should be fast, took {:?}",
        duration
    );

    // Verify all templates work
    for secret in secrets {
        let display = format!("{}", secret);
        assert!(display.contains("Large[record]:"));
        assert!(display.contains("field_0:"));
        assert!(display.ends_with("..."));
    }
}
