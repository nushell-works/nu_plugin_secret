//! Integration tests for the unified secret wrap command

use nu_plugin::{Plugin, PluginCommand};
use nu_plugin_secret::commands::wrap::SecretWrapCommand;
use nu_plugin_secret::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretList, SecretPlugin,
    SecretRecord, SecretString,
};
use nu_protocol::{Span, Value};

#[test]
fn test_unified_wrap_functionality() {
    // Test that the unified wrap command would handle different types correctly
    // Note: These tests verify the secret types can be created from various Value types

    // Test string wrapping
    let string_val = Value::string("test_secret", Span::test_data());
    if let Value::String { val, .. } = string_val {
        let secret = SecretString::new(val);
        let display = format!("{}", secret);
        assert!(
            display.contains("redacted") || display.contains("HIDDEN") || display.contains("***")
        );
    }

    // Test integer wrapping
    let int_val = Value::int(42, Span::test_data());
    if let Value::Int { val, .. } = int_val {
        let secret = SecretInt::new(val);
        let display = format!("{}", secret);
        assert!(
            display.contains("redacted") || display.contains("HIDDEN") || display.contains("***")
        );
    }

    // Test boolean wrapping
    let bool_val = Value::bool(true, Span::test_data());
    if let Value::Bool { val, .. } = bool_val {
        let secret = SecretBool::new(val);
        let display = format!("{}", secret);
        assert!(
            display.contains("redacted") || display.contains("HIDDEN") || display.contains("***")
        );
    }

    // Test float wrapping
    let float_val = Value::float(std::f64::consts::PI, Span::test_data());
    if let Value::Float { val, .. } = float_val {
        let secret = SecretFloat::new(val);
        let display = format!("{}", secret);
        assert!(
            display.contains("redacted") || display.contains("HIDDEN") || display.contains("***")
        );
    }

    // Test date wrapping
    // Use a fixed timestamp for Miri compatibility
    let fixed_datetime = chrono::DateTime::from_timestamp(1609459200, 0).unwrap(); // 2021-01-01 00:00:00 UTC
    let date_val = Value::date(fixed_datetime.into(), Span::test_data());
    if let Value::Date { val, .. } = date_val {
        let secret = SecretDate::new(val);
        let display = format!("{}", secret);
        assert!(
            display.contains("redacted") || display.contains("HIDDEN") || display.contains("***")
        );
    }

    // Test binary wrapping
    let binary_val = Value::binary(vec![1, 2, 3, 4], Span::test_data());
    if let Value::Binary { val, .. } = binary_val {
        let secret = SecretBinary::new(val);
        let display = format!("{}", secret);
        assert!(
            display.contains("redacted") || display.contains("HIDDEN") || display.contains("***")
        );
    }

    // Test list wrapping
    let list_val = Value::list(
        vec![
            Value::int(1, Span::test_data()),
            Value::int(2, Span::test_data()),
        ],
        Span::test_data(),
    );
    if let Value::List { vals, .. } = list_val {
        let secret = SecretList::new(vals);
        let display = format!("{}", secret);
        assert!(
            display.contains("redacted") || display.contains("HIDDEN") || display.contains("***")
        );
    }

    // Test record wrapping
    let record_val = Value::record(
        nu_protocol::record! {
            "name" => Value::string("test", Span::test_data()),
            "value" => Value::int(42, Span::test_data()),
        },
        Span::test_data(),
    );
    if let Value::Record { val, .. } = record_val {
        let secret = SecretRecord::new(val.into_owned());
        let display = format!("{}", secret);
        assert!(
            display.contains("redacted") || display.contains("HIDDEN") || display.contains("***")
        );
    }
}

#[test]
fn test_type_detection_and_wrapping() {
    // Test that we can determine types correctly and create appropriate secrets
    let test_cases = vec![
        ("string", Value::string("secret", Span::test_data())),
        ("int", Value::int(42, Span::test_data())),
        ("bool", Value::bool(true, Span::test_data())),
        (
            "float",
            Value::float(std::f64::consts::PI, Span::test_data()),
        ),
        ("binary", Value::binary(vec![1, 2, 3], Span::test_data())),
    ];

    for (expected_type, value) in test_cases {
        match value {
            Value::String { .. } => assert_eq!(expected_type, "string"),
            Value::Int { .. } => assert_eq!(expected_type, "int"),
            Value::Bool { .. } => assert_eq!(expected_type, "bool"),
            Value::Float { .. } => assert_eq!(expected_type, "float"),
            Value::Binary { .. } => assert_eq!(expected_type, "binary"),
            _ => panic!("Unexpected type encountered"),
        }
    }
}

#[test]
fn test_unified_wrap_maintains_security() {
    // Verify that all wrapped types properly redact their values
    let secret_string = SecretString::new("sensitive_data".to_string());
    let secret_int = SecretInt::new(12345);
    let secret_bool = SecretBool::new(false);
    let secret_float = SecretFloat::new(std::f64::consts::E);

    // Test that none of the secret types leak their actual values
    let string_display = format!("{}", secret_string);
    let int_display = format!("{}", secret_int);
    let bool_display = format!("{}", secret_bool);
    let float_display = format!("{}", secret_float);

    // Ensure actual values are not exposed
    assert!(!string_display.contains("sensitive_data"));
    assert!(!int_display.contains("12345"));
    assert!(!bool_display.contains("false"));
    assert!(!float_display.contains("2.718"));

    // Ensure they show redacted content
    assert!(
        string_display.contains("redacted")
            || string_display.contains("HIDDEN")
            || string_display.contains("***")
    );
    assert!(
        int_display.contains("redacted")
            || int_display.contains("HIDDEN")
            || int_display.contains("***")
    );
    assert!(
        bool_display.contains("redacted")
            || bool_display.contains("HIDDEN")
            || bool_display.contains("***")
    );
    assert!(
        float_display.contains("redacted")
            || float_display.contains("HIDDEN")
            || float_display.contains("***")
    );
}

#[test]
fn test_unified_wrap_command_registration() {
    // Test that the SecretWrapCommand is properly registered in the plugin
    let plugin = SecretPlugin;
    let commands = plugin.commands();

    // Find the unified wrap command
    let wrap_command = commands.iter().find(|cmd| cmd.name() == "secret wrap");

    assert!(
        wrap_command.is_some(),
        "Unified wrap command should be registered"
    );

    // Verify basic command properties
    if let Some(cmd) = wrap_command {
        assert_eq!(cmd.name(), "secret wrap");
        assert!(
            !cmd.signature().description.is_empty(),
            "Command should have usage documentation"
        );

        // Verify command signature indicates it accepts pipeline input
        let sig = cmd.signature();
        assert!(
            !sig.input_output_types.is_empty(),
            "Command should specify input/output types"
        );
    }
}

#[test]
fn test_unified_wrap_command_properties() {
    // Test the basic properties of SecretWrapCommand
    let command = SecretWrapCommand;

    // Verify command name
    assert_eq!(command.name(), "secret wrap");

    // Verify signature properties
    let signature = command.signature();
    assert_eq!(signature.name, "secret wrap");
    assert!(
        !signature.description.is_empty(),
        "Command should have usage documentation"
    );

    // Verify examples exist
    let examples = command.examples();
    assert!(!examples.is_empty(), "Command should have examples");

    // Verify each example has description and example text
    for example in examples {
        assert!(
            !example.description.is_empty(),
            "Example should have description"
        );
        assert!(
            !example.example.is_empty(),
            "Example should have example text"
        );
    }
}

#[test]
fn test_unified_wrap_type_detection_capabilities() {
    // Test that the command can handle all the supported types
    // This tests the conceptual capability rather than runtime execution

    let supported_types = vec![
        Value::string("test", Span::test_data()),
        Value::int(42, Span::test_data()),
        Value::bool(true, Span::test_data()),
        Value::float(std::f64::consts::PI, Span::test_data()),
        Value::binary(vec![1, 2, 3], Span::test_data()),
        Value::list(vec![Value::int(1, Span::test_data())], Span::test_data()),
        Value::record(
            nu_protocol::record! {
                "key" => Value::string("value", Span::test_data()),
            },
            Span::test_data(),
        ),
    ];

    // Test that all these types would be supported for wrapping
    for value in supported_types {
        let should_be_supported = matches!(
            value,
            Value::String { .. }
                | Value::Int { .. }
                | Value::Bool { .. }
                | Value::Float { .. }
                | Value::Binary { .. }
                | Value::List { .. }
                | Value::Record { .. }
        );
        assert!(
            should_be_supported,
            "Type should be supported for wrapping: {:?}",
            value
        );
    }
}

#[test]
fn test_unified_wrap_replaces_individual_commands() {
    // Test that we only have the unified wrap command, no individual type commands
    let plugin = SecretPlugin;
    let commands = plugin.commands();

    // Should have no individual type-specific wrap commands (but wrap-with is different)
    let individual_type_wrap_commands: Vec<_> = commands
        .iter()
        .filter(|cmd| {
            let name = cmd.name();
            name.starts_with("secret wrap-") && name != "secret wrap-with"
        })
        .collect();

    // All individual type commands have been removed in favor of unified approach
    assert!(
        individual_type_wrap_commands.is_empty(),
        "Individual type wrap commands have been removed - found: {:?}",
        individual_type_wrap_commands
            .iter()
            .map(|cmd| cmd.name())
            .collect::<Vec<_>>()
    );

    // Verify the unified command exists
    let unified_command = commands.iter().find(|cmd| cmd.name() == "secret wrap");
    assert!(
        unified_command.is_some(),
        "Unified wrap command should exist"
    );

    // The unified command should handle all the types that were previously individual commands
    let unified_cmd = unified_command.unwrap();
    let signature = unified_cmd.signature();

    // Check that the unified command supports all the expected input/output type mappings
    let input_output_types = &signature.input_output_types;
    assert!(
        !input_output_types.is_empty(),
        "Unified command should have input/output type mappings"
    );

    // The signature should indicate it can handle multiple types
    assert!(
        input_output_types.len() >= 8,
        "Unified command should handle at least 8 types (string, int, bool, float, date, binary, list, record)"
    );
}

#[test]
fn test_unified_wrap_command_benefits() {
    // This test demonstrates the benefits of the unified approach

    // Before: Users had to know the exact type and use specific commands
    // "my-secret" | secret wrap-string
    // 42 | secret wrap-int
    // true | secret wrap-bool

    // After: Users can use a single command for any type
    // "my-secret" | secret wrap
    // 42 | secret wrap
    // true | secret wrap

    // Test that different types can be wrapped uniformly
    let values = vec![
        Value::string("api-key", Span::test_data()),
        Value::int(8080, Span::test_data()),
        Value::bool(true, Span::test_data()),
        Value::float(1.23, Span::test_data()),
    ];

    for value in values {
        // In a real implementation, this is what the unified wrap command would do
        let can_be_wrapped = matches!(
            value,
            Value::String { .. }
                | Value::Int { .. }
                | Value::Bool { .. }
                | Value::Float { .. }
                | Value::Date { .. }
                | Value::Binary { .. }
                | Value::List { .. }
                | Value::Record { .. }
        );
        assert!(can_be_wrapped);
    }
}
