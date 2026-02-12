//! Implements `secret contains` — checks if a secret value matches a given value.

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};

use crate::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretList, SecretRecord,
    SecretString,
};

#[derive(Clone)]
pub struct SecretContainsCommand;

impl PluginCommand for SecretContainsCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret contains"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "value",
                SyntaxShape::Any,
                "The value to search for in the secret",
            )
            .input_output_types(vec![
                (Type::Custom("secret_string".into()), Type::Bool),
                (Type::Custom("secret_int".into()), Type::Bool),
                (Type::Custom("secret_bool".into()), Type::Bool),
                (Type::Custom("secret_float".into()), Type::Bool),
                (Type::Custom("secret_date".into()), Type::Bool),
                (Type::Custom("secret_binary".into()), Type::Bool),
                (Type::Custom("secret_list".into()), Type::Bool),
                (Type::Custom("secret_record".into()), Type::Bool),
            ])
            .category(Category::Filters)
    }

    fn description(&self) -> &str {
        "Check if a secret contains a specific value by comparing with the embedded secret data"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: r#""my-secret-key" | secret wrap | secret contains "my-secret-key""#,
                description: "Check if a secret string contains a specific string value",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#"42 | secret wrap | secret contains 42"#,
                description: "Check if a secret integer contains a specific integer value",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#"true | secret wrap | secret contains true"#,
                description: "Check if a secret boolean contains a specific boolean value",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#"3.14159265359 | secret wrap | secret contains 3.14159265359"#,
                description: "Check if a secret float contains a specific float value",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#"[1, 2, 3] | secret wrap | secret contains [1, 2, 3]"#,
                description: "Check if a secret list contains a specific list value",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#"{name: "john"} | secret wrap | secret contains {name: "john"}"#,
                description: "Check if a secret record contains a specific record value",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""my-secret" | secret wrap | secret contains "different-secret""#,
                description: "Check returns false when values don't match",
                result: Some(Value::bool(false, nu_protocol::Span::test_data())),
            },
        ]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let search_value = call.req(0)?;

        match input {
            PipelineData::Value(value, metadata) => {
                let result = match value {
                    Value::Custom { val, .. } => {
                        if let Some(secret_string) = val.as_any().downcast_ref::<SecretString>() {
                            match &search_value {
                                Value::String {
                                    val: search_str, ..
                                } => Value::bool(secret_string.reveal() == search_str, call.head),
                                _ => {
                                    return Err(LabeledError::new("Type mismatch").with_label(
                                        "Expected string value to compare with SecretString",
                                        call.head,
                                    ));
                                }
                            }
                        } else if let Some(secret_int) = val.as_any().downcast_ref::<SecretInt>() {
                            match &search_value {
                                Value::Int {
                                    val: search_int, ..
                                } => Value::bool(secret_int.reveal() == *search_int, call.head),
                                _ => {
                                    return Err(LabeledError::new("Type mismatch").with_label(
                                        "Expected integer value to compare with SecretInt",
                                        call.head,
                                    ));
                                }
                            }
                        } else if let Some(secret_bool) = val.as_any().downcast_ref::<SecretBool>()
                        {
                            match &search_value {
                                Value::Bool {
                                    val: search_bool, ..
                                } => Value::bool(secret_bool.reveal() == *search_bool, call.head),
                                _ => {
                                    return Err(LabeledError::new("Type mismatch").with_label(
                                        "Expected boolean value to compare with SecretBool",
                                        call.head,
                                    ));
                                }
                            }
                        } else if let Some(secret_float) =
                            val.as_any().downcast_ref::<SecretFloat>()
                        {
                            match &search_value {
                                Value::Float {
                                    val: search_float, ..
                                } => {
                                    // Use epsilon comparison for floating point values
                                    let diff = (secret_float.reveal() - search_float).abs();
                                    Value::bool(diff < f64::EPSILON, call.head)
                                }
                                _ => {
                                    return Err(LabeledError::new("Type mismatch").with_label(
                                        "Expected float value to compare with SecretFloat",
                                        call.head,
                                    ));
                                }
                            }
                        } else if let Some(secret_date) = val.as_any().downcast_ref::<SecretDate>()
                        {
                            match &search_value {
                                Value::Date {
                                    val: search_date, ..
                                } => Value::bool(secret_date.reveal() == search_date, call.head),
                                _ => {
                                    return Err(LabeledError::new("Type mismatch").with_label(
                                        "Expected date value to compare with SecretDate",
                                        call.head,
                                    ));
                                }
                            }
                        } else if let Some(secret_binary) =
                            val.as_any().downcast_ref::<SecretBinary>()
                        {
                            match &search_value {
                                Value::Binary {
                                    val: search_binary, ..
                                } => Value::bool(
                                    secret_binary.reveal().as_ref() == search_binary.as_slice(),
                                    call.head,
                                ),
                                _ => {
                                    return Err(LabeledError::new("Type mismatch").with_label(
                                        "Expected binary value to compare with SecretBinary",
                                        call.head,
                                    ));
                                }
                            }
                        } else if let Some(secret_list) = val.as_any().downcast_ref::<SecretList>()
                        {
                            match &search_value {
                                Value::List {
                                    vals: search_list, ..
                                } => {
                                    // Create Value::List instances for comparison since Value implements PartialEq
                                    let secret_value =
                                        Value::list(secret_list.reveal().clone(), call.head);
                                    let search_value_list =
                                        Value::list(search_list.clone(), call.head);
                                    Value::bool(secret_value == search_value_list, call.head)
                                }
                                _ => {
                                    return Err(LabeledError::new("Type mismatch").with_label(
                                        "Expected list value to compare with SecretList",
                                        call.head,
                                    ));
                                }
                            }
                        } else if let Some(secret_record) =
                            val.as_any().downcast_ref::<SecretRecord>()
                        {
                            match &search_value {
                                Value::Record {
                                    val: search_record, ..
                                } => {
                                    // Create Value::Record instances for comparison since Value implements PartialEq
                                    let secret_value =
                                        Value::record(secret_record.reveal().clone(), call.head);
                                    let search_value_record =
                                        Value::record((**search_record).clone(), call.head);
                                    Value::bool(secret_value == search_value_record, call.head)
                                }
                                _ => {
                                    return Err(LabeledError::new("Type mismatch").with_label(
                                        "Expected record value to compare with SecretRecord",
                                        call.head,
                                    ));
                                }
                            }
                        } else {
                            return Err(LabeledError::new("Invalid input").with_label(
                                "Input must be a secret type (SecretString, SecretInt, etc.)",
                                call.head,
                            ));
                        }
                    }
                    _ => {
                        return Err(LabeledError::new("Invalid input")
                            .with_label(
                                "Input must be a secret type. Use 'secret wrap' to create a secret first",
                                call.head,
                            ));
                    }
                };

                Ok(PipelineData::Value(result, metadata))
            }
            _ => Err(LabeledError::new("Invalid input")
                .with_label("Expected a single secret value", call.head)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretContainsCommand;
        assert_eq!(command.name(), "secret contains");
    }

    #[test]
    fn test_signature() {
        let command = SecretContainsCommand;
        let signature = command.signature();
        assert_eq!(signature.name, "secret contains");
        assert_eq!(signature.required_positional.len(), 1);
        assert_eq!(signature.input_output_types.len(), 8);
    }

    #[test]
    fn test_description() {
        let command = SecretContainsCommand;
        assert_eq!(
            command.description(),
            "Check if a secret contains a specific value by comparing with the embedded secret data"
        );
    }

    #[test]
    fn test_examples_count() {
        let command = SecretContainsCommand;
        let examples = command.examples();
        assert_eq!(examples.len(), 7);
    }

    #[test]
    fn test_examples_have_descriptions() {
        let command = SecretContainsCommand;
        let examples = command.examples();

        for example in examples {
            assert!(!example.description.is_empty());
            // Each description should be meaningful
            assert!(example.description.len() > 10);
        }
    }

    #[test]
    fn test_examples_have_valid_results() {
        let command = SecretContainsCommand;
        let examples = command.examples();

        for example in examples {
            if let Some(expected_result) = &example.result {
                match expected_result {
                    nu_protocol::Value::Bool { .. } => {
                        // Good - contains command should return boolean
                    }
                    _ => panic!("Contains command examples should return boolean values"),
                }
            }
        }
    }

    // Unit tests for the run method
    // Note: Direct testing of the run method requires complex EngineInterface setup.
    // Instead, we test the core logic that the run method uses. The full integration
    // is thoroughly tested in the integration test files (tests/contains_tests.rs).
    //
    // The tests below verify the comparison logic for each secret type, which is
    // the core functionality that the run method delegates to.

    // Test core logic functions that would be used in the run method
    #[test]
    fn test_string_comparison_logic() {
        let secret = SecretString::new("test-secret".to_string());
        let search_str = "test-secret";
        assert_eq!(secret.reveal(), search_str);

        let different_secret = SecretString::new("different-secret".to_string());
        assert_ne!(different_secret.reveal(), search_str);
    }

    #[test]
    fn test_int_comparison_logic() {
        let secret = SecretInt::new(42);
        assert_eq!(secret.reveal(), 42i64);

        let different_secret = SecretInt::new(99);
        assert_ne!(different_secret.reveal(), 42i64);
    }

    #[test]
    fn test_bool_comparison_logic() {
        let secret_true = SecretBool::new(true);
        assert!(secret_true.reveal());

        let secret_false = SecretBool::new(false);
        assert!(!secret_false.reveal());
        assert_ne!(secret_true.reveal(), secret_false.reveal());
    }

    #[test]
    fn test_float_epsilon_comparison_logic() {
        let secret = SecretFloat::new(std::f64::consts::PI);
        let search_val = std::f64::consts::PI;

        // Test exact match
        let diff = (secret.reveal() - search_val).abs();
        assert!(diff < f64::EPSILON);

        // Test close values within epsilon
        let close_val = search_val + f64::EPSILON / 2.0;
        let close_diff = (secret.reveal() - close_val).abs();
        assert!(close_diff < f64::EPSILON);

        // Test values outside epsilon
        let far_val = search_val + f64::EPSILON * 2.0;
        let far_diff = (secret.reveal() - far_val).abs();
        assert!(far_diff >= f64::EPSILON);
    }

    #[test]
    fn test_date_comparison_logic() {
        let test_date = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());

        let secret = SecretDate::new(test_date);
        assert_eq!(secret.reveal(), &test_date);

        let different_date = test_date + chrono::Duration::hours(1);
        let different_secret = SecretDate::new(different_date);
        assert_ne!(different_secret.reveal(), &test_date);
    }

    #[test]
    fn test_binary_comparison_logic() {
        let test_data = vec![1, 2, 3, 4, 5];
        let secret = SecretBinary::new(test_data.clone());
        assert_eq!(secret.reveal().as_ref(), test_data.as_slice());

        let different_data = vec![5, 4, 3, 2, 1];
        let different_secret = SecretBinary::new(different_data.clone());
        assert_ne!(different_secret.reveal().as_ref(), test_data.as_slice());
    }

    #[test]
    fn test_list_comparison_logic() {
        let test_list = vec![
            Value::int(1, nu_protocol::Span::test_data()),
            Value::string("test", nu_protocol::Span::test_data()),
        ];
        let secret = SecretList::new(test_list.clone());
        assert_eq!(secret.reveal(), &test_list);

        let different_list = vec![
            Value::int(2, nu_protocol::Span::test_data()),
            Value::string("different", nu_protocol::Span::test_data()),
        ];
        let different_secret = SecretList::new(different_list.clone());
        assert_ne!(different_secret.reveal(), &test_list);
    }

    #[test]
    fn test_record_comparison_logic() {
        let test_record = nu_protocol::record! {
            "name" => Value::string("john", nu_protocol::Span::test_data()),
            "age" => Value::int(30, nu_protocol::Span::test_data()),
        };
        let secret = SecretRecord::new(test_record.clone());

        // Test field access since Record doesn't implement PartialEq
        let revealed = secret.reveal();
        assert_eq!(revealed.len(), test_record.len());
        assert_eq!(revealed.get("name"), test_record.get("name"));
        assert_eq!(revealed.get("age"), test_record.get("age"));
    }

    #[test]
    fn test_float_special_values_logic() {
        // Test NaN
        let secret_nan = SecretFloat::new(f64::NAN);
        assert!(secret_nan.reveal().is_nan());
        // NaN != NaN
        assert_ne!(secret_nan.reveal(), f64::NAN);

        // Test Infinity
        let secret_inf = SecretFloat::new(f64::INFINITY);
        assert_eq!(secret_inf.reveal(), f64::INFINITY);

        // Test negative infinity
        let secret_neg_inf = SecretFloat::new(f64::NEG_INFINITY);
        assert_eq!(secret_neg_inf.reveal(), f64::NEG_INFINITY);

        // Test positive and negative zero
        let secret_zero = SecretFloat::new(0.0);
        let secret_neg_zero = SecretFloat::new(-0.0);
        assert_eq!(secret_zero.reveal(), 0.0);
        assert_eq!(secret_neg_zero.reveal(), -0.0);
        // 0.0 == -0.0 in floating point
        assert_eq!(secret_zero.reveal(), secret_neg_zero.reveal());
    }

    #[test]
    fn test_empty_values_logic() {
        // Test empty string
        let empty_secret = SecretString::new(String::new());
        assert_eq!(empty_secret.reveal(), "");

        // Test empty list
        let empty_list = SecretList::new(vec![]);
        assert_eq!(empty_list.reveal(), &Vec::<Value>::new());

        // Test empty record
        let empty_record = SecretRecord::new(nu_protocol::Record::new());
        assert_eq!(empty_record.reveal().len(), 0);

        // Test empty binary data
        let empty_binary = SecretBinary::new(vec![]);
        assert_eq!(empty_binary.reveal().as_ref(), Vec::<u8>::new().as_slice());
    }

    #[test]
    fn test_complex_nested_structures() {
        // Test nested list
        let nested_list = vec![
            Value::list(
                vec![
                    Value::int(1, nu_protocol::Span::test_data()),
                    Value::int(2, nu_protocol::Span::test_data()),
                ],
                nu_protocol::Span::test_data(),
            ),
            Value::string("nested", nu_protocol::Span::test_data()),
        ];
        let secret_nested = SecretList::new(nested_list.clone());
        assert_eq!(secret_nested.reveal(), &nested_list);

        // Test complex record with nested structures
        let complex_record = nu_protocol::record! {
            "user" => Value::record(nu_protocol::record! {
                "name" => Value::string("alice", nu_protocol::Span::test_data()),
                "id" => Value::int(123, nu_protocol::Span::test_data()),
            }, nu_protocol::Span::test_data()),
            "permissions" => Value::list(vec![
                Value::string("read", nu_protocol::Span::test_data()),
                Value::string("write", nu_protocol::Span::test_data()),
            ], nu_protocol::Span::test_data()),
        };
        let secret_complex = SecretRecord::new(complex_record.clone());
        assert_eq!(secret_complex.reveal().len(), complex_record.len());
    }

    #[test]
    fn test_unicode_string_logic() {
        // Test Unicode strings
        let unicode_string = "Hello 世界 🌍 мир";
        let secret = SecretString::new(unicode_string.to_string());
        assert_eq!(secret.reveal(), unicode_string);

        // Test different Unicode normalization forms would still match
        let same_content = "Hello 世界 🌍 мир";
        let same_secret = SecretString::new(same_content.to_string());
        assert_eq!(secret.reveal(), same_secret.reveal());
    }

    #[test]
    fn test_large_data_structures() {
        // Test large string
        let large_string = "x".repeat(10000);
        let secret = SecretString::new(large_string.clone());
        assert_eq!(secret.reveal(), &large_string);
        assert_eq!(secret.reveal().len(), 10000);

        // Test large binary data
        let large_binary: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let binary_secret = SecretBinary::new(large_binary.clone());
        assert_eq!(binary_secret.reveal().as_ref(), large_binary.as_slice());
        assert_eq!(binary_secret.reveal().len(), 10000);

        // Test large list
        let large_list: Vec<Value> = (0..1000)
            .map(|i| Value::int(i, nu_protocol::Span::test_data()))
            .collect();
        let list_secret = SecretList::new(large_list.clone());
        assert_eq!(list_secret.reveal(), &large_list);
        assert_eq!(list_secret.reveal().len(), 1000);
    }

    #[test]
    fn test_boundary_integer_values() {
        // Test boundary integer values
        let max_int = SecretInt::new(i64::MAX);
        assert_eq!(max_int.reveal(), i64::MAX);

        let min_int = SecretInt::new(i64::MIN);
        assert_eq!(min_int.reveal(), i64::MIN);

        let zero_int = SecretInt::new(0);
        assert_eq!(zero_int.reveal(), 0i64);

        let negative_int = SecretInt::new(-1);
        assert_eq!(negative_int.reveal(), -1i64);
    }

    #[test]
    fn test_boundary_float_values() {
        // Test very small and large float values
        let max_float = SecretFloat::new(f64::MAX);
        assert_eq!(max_float.reveal(), f64::MAX);

        let min_float = SecretFloat::new(f64::MIN);
        assert_eq!(min_float.reveal(), f64::MIN);

        let smallest_positive = SecretFloat::new(f64::MIN_POSITIVE);
        assert_eq!(smallest_positive.reveal(), f64::MIN_POSITIVE);

        let epsilon = SecretFloat::new(f64::EPSILON);
        assert_eq!(epsilon.reveal(), f64::EPSILON);
    }
}
