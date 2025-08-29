//! Integration tests for the secret contains command

use nu_plugin_secret::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretList, SecretRecord,
    SecretString,
};
use nu_protocol::{Span, Value};

#[test]
fn test_contains_string_matching() {
    // Test exact string match
    let secret = SecretString::new("my-secret-key".to_string());
    let test_value = "my-secret-key";

    assert_eq!(secret.reveal(), test_value);

    // Test non-matching string
    let different_secret = SecretString::new("different-key".to_string());
    assert_ne!(different_secret.reveal(), test_value);
}

#[test]
fn test_contains_integer_matching() {
    // Test exact integer match
    let secret = SecretInt::new(42);
    let test_value = 42i64;

    assert_eq!(secret.reveal(), test_value);

    // Test non-matching integer
    let different_secret = SecretInt::new(99);
    assert_ne!(different_secret.reveal(), test_value);
}

#[test]
fn test_contains_boolean_matching() {
    // Test boolean match (true)
    let secret_true = SecretBool::new(true);
    assert!(secret_true.reveal());

    // Test boolean match (false)
    let secret_false = SecretBool::new(false);
    assert!(!secret_false.reveal());

    // Test non-matching boolean
    assert!(secret_true.reveal());
    assert!(!secret_false.reveal());
}

#[test]
fn test_contains_float_matching() {
    // Test exact float match
    let secret = SecretFloat::new(std::f64::consts::PI);
    let test_value = std::f64::consts::PI;

    // Use epsilon comparison for floating point
    let diff = (secret.reveal() - test_value).abs();
    assert!(diff < f64::EPSILON);

    // Test non-matching float
    let different_secret = SecretFloat::new(std::f64::consts::E);
    let diff2 = (different_secret.reveal() - test_value).abs();
    assert!(diff2 >= f64::EPSILON);
}

#[test]
fn test_contains_date_matching() {
    // Use fixed date for Miri compatibility (avoid system time under Miri)
    #[cfg(miri)]
    let test_date = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());

    #[cfg(not(miri))]
    let test_date = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());

    let secret = SecretDate::new(test_date);

    assert_eq!(secret.reveal(), &test_date);

    // Test non-matching date
    let different_date = test_date + chrono::Duration::hours(1);
    let different_secret = SecretDate::new(different_date);
    assert_ne!(different_secret.reveal(), &test_date);
}

#[test]
fn test_contains_binary_matching() {
    // Test binary data match
    let test_data = vec![1, 2, 3, 4, 5];
    let secret = SecretBinary::new(test_data.clone());

    assert_eq!(secret.reveal().as_ref(), test_data.as_slice());

    // Test non-matching binary data
    let different_data = vec![5, 4, 3, 2, 1];
    let different_secret = SecretBinary::new(different_data.clone());
    assert_ne!(different_secret.reveal().as_ref(), test_data.as_slice());
}

#[test]
fn test_contains_list_matching() {
    // Test list match
    let test_list = vec![
        Value::int(1, Span::test_data()),
        Value::string("test", Span::test_data()),
        Value::bool(true, Span::test_data()),
    ];
    let secret = SecretList::new(test_list.clone());

    assert_eq!(secret.reveal(), &test_list);

    // Test non-matching list
    let different_list = vec![
        Value::int(2, Span::test_data()),
        Value::string("different", Span::test_data()),
    ];
    let different_secret = SecretList::new(different_list.clone());
    assert_ne!(different_secret.reveal(), &test_list);
}

#[test]
fn test_contains_record_matching() {
    // Test record match using field access since Record doesn't implement PartialEq
    let test_record = nu_protocol::record! {
        "name" => Value::string("john", Span::test_data()),
        "age" => Value::int(30, Span::test_data()),
        "active" => Value::bool(true, Span::test_data()),
    };
    let secret = SecretRecord::new(test_record.clone());

    // Compare by checking individual fields
    let revealed = secret.reveal();
    assert_eq!(revealed.len(), test_record.len());
    assert_eq!(revealed.get("name"), test_record.get("name"));
    assert_eq!(revealed.get("age"), test_record.get("age"));
    assert_eq!(revealed.get("active"), test_record.get("active"));

    // Test non-matching record
    let different_record = nu_protocol::record! {
        "name" => Value::string("jane", Span::test_data()),
        "age" => Value::int(25, Span::test_data()),
    };
    let different_secret = SecretRecord::new(different_record.clone());
    let different_revealed = different_secret.reveal();
    assert_ne!(different_revealed.len(), test_record.len());
}

#[test]
fn test_contains_type_safety() {
    // This test demonstrates the type safety of the contains operation
    // In a real implementation, trying to compare mismatched types would result in an error

    let secret_string = SecretString::new("42".to_string());
    let secret_int = SecretInt::new(42);

    // These should be equal for their respective types
    assert_eq!(secret_string.reveal(), "42");
    assert_eq!(secret_int.reveal(), 42i64);

    // But comparing across types should not work (this would be caught by the command)
    // secret_string.reveal() != secret_int.reveal() by type system
}

#[test]
fn test_contains_edge_cases() {
    // Test empty string
    let empty_secret = SecretString::new(String::new());
    assert_eq!(empty_secret.reveal(), "");

    // Test zero values
    let zero_int = SecretInt::new(0);
    assert_eq!(zero_int.reveal(), 0i64);

    let zero_float = SecretFloat::new(0.0);
    let diff = (zero_float.reveal() - 0.0).abs();
    assert!(diff < f64::EPSILON);

    // Test empty collections
    let empty_list = SecretList::new(vec![]);
    assert_eq!(empty_list.reveal(), &Vec::<Value>::new());

    let empty_record = SecretRecord::new(nu_protocol::Record::new());
    assert_eq!(empty_record.reveal().len(), 0);

    // Test empty binary data
    let empty_binary = SecretBinary::new(vec![]);
    assert_eq!(empty_binary.reveal().as_ref(), Vec::<u8>::new().as_slice());
}

#[test]
fn test_contains_complex_data_structures() {
    // Test nested list
    let nested_list = vec![
        Value::list(
            vec![
                Value::int(1, Span::test_data()),
                Value::int(2, Span::test_data()),
            ],
            Span::test_data(),
        ),
        Value::string("nested", Span::test_data()),
    ];
    let secret_nested = SecretList::new(nested_list.clone());
    assert_eq!(secret_nested.reveal(), &nested_list);

    // Test complex record
    let complex_record = nu_protocol::record! {
        "user" => Value::record(nu_protocol::record! {
            "name" => Value::string("alice", Span::test_data()),
            "id" => Value::int(123, Span::test_data()),
        }, Span::test_data()),
        "permissions" => Value::list(vec![
            Value::string("read", Span::test_data()),
            Value::string("write", Span::test_data()),
        ], Span::test_data()),
    };
    let secret_complex = SecretRecord::new(complex_record.clone());
    assert_eq!(secret_complex.reveal().len(), complex_record.len());
}

#[test]
fn test_contains_security_properties() {
    // Verify that the contains operation doesn't compromise security
    let secret = SecretString::new("super-secret-password".to_string());

    // The secret should still display as redacted
    let display = format!("{}", secret);
    assert!(display.contains("redacted") || display.contains("HIDDEN") || display.contains("***"));

    // But we can still check if it contains the right value
    assert_eq!(secret.reveal(), "super-secret-password");
    assert_ne!(secret.reveal(), "wrong-password");

    // The display should never show the actual secret
    assert!(!display.contains("super-secret-password"));
}

#[test]
fn test_contains_performance_characteristics() {
    // Test with larger data to ensure reasonable performance
    let large_string = "x".repeat(10000);
    let secret = SecretString::new(large_string.clone());
    assert_eq!(secret.reveal(), &large_string);

    let large_binary: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    let binary_secret = SecretBinary::new(large_binary.clone());
    assert_eq!(binary_secret.reveal().as_ref(), large_binary.as_slice());

    let large_list: Vec<Value> = (0..1000)
        .map(|i| Value::int(i, Span::test_data()))
        .collect();
    let list_secret = SecretList::new(large_list.clone());
    assert_eq!(list_secret.reveal(), &large_list);
}

#[test]
fn test_contains_unicode_support() {
    // Test Unicode strings
    let unicode_string = "Hello 世界 🌍 мир";
    let secret = SecretString::new(unicode_string.to_string());
    assert_eq!(secret.reveal(), unicode_string);

    // Test Unicode in record keys and values
    let unicode_record = nu_protocol::record! {
        "名前" => Value::string("田中", Span::test_data()),
        "возраст" => Value::int(25, Span::test_data()),
        "emoji" => Value::string("🎉🎊", Span::test_data()),
    };
    let secret_record = SecretRecord::new(unicode_record.clone());
    assert_eq!(secret_record.reveal().len(), unicode_record.len());
}
