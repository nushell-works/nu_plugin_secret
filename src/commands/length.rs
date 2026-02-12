//! Implements `secret length` — returns the length of a secret value.

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

use crate::{SecretBinary, SecretList, SecretString};

#[derive(Clone)]
pub struct SecretLengthCommand;

impl PluginCommand for SecretLengthCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret length"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![
                (Type::Custom("secret_string".into()), Type::Int),
                (Type::Custom("secret_list".into()), Type::Int),
                (Type::Custom("secret_binary".into()), Type::Int),
            ])
            .category(Category::Filters)
    }

    fn description(&self) -> &str {
        "Get the length of a secret string, list, or binary data without exposing the content"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: r#""my-secret-key" | secret wrap | secret length"#,
                description: "Get the length of a secret string",
                result: Some(Value::int(13, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#"[1, 2, 3, 4, 5] | secret wrap | secret length"#,
                description: "Get the length of a secret list",
                result: Some(Value::int(5, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#"0x[deadbeef] | secret wrap | secret length"#,
                description: "Get the length of secret binary data",
                result: Some(Value::int(4, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""" | secret wrap | secret length"#,
                description: "Get the length of an empty secret string",
                result: Some(Value::int(0, nu_protocol::Span::test_data())),
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
        match input {
            PipelineData::Value(value, metadata) => {
                let result = match value {
                    Value::Custom { val, .. } => {
                        if let Some(secret_string) = val.as_any().downcast_ref::<SecretString>() {
                            Value::int(secret_string.reveal().len() as i64, call.head)
                        } else if let Some(secret_list) = val.as_any().downcast_ref::<SecretList>()
                        {
                            Value::int(secret_list.reveal().len() as i64, call.head)
                        } else if let Some(secret_binary) =
                            val.as_any().downcast_ref::<SecretBinary>()
                        {
                            Value::int(secret_binary.reveal().len() as i64, call.head)
                        } else {
                            return Err(LabeledError::new("Unsupported secret type").with_label(
                                "Only SecretString, SecretList, and SecretBinary support length operation",
                                call.head,
                            ));
                        }
                    }
                    _ => {
                        return Err(LabeledError::new("Invalid input")
                            .with_label(
                                "Input must be a secret string, list, or binary. Use 'secret wrap' to create a secret first",
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
    use nu_protocol::Span;

    #[test]
    fn test_command_name() {
        let command = SecretLengthCommand;
        assert_eq!(command.name(), "secret length");
    }

    #[test]
    fn test_signature() {
        let command = SecretLengthCommand;
        let signature = command.signature();
        assert_eq!(signature.name, "secret length");
        assert_eq!(signature.required_positional.len(), 0);
        assert_eq!(signature.input_output_types.len(), 3);
    }

    #[test]
    fn test_description() {
        let command = SecretLengthCommand;
        assert_eq!(
            command.description(),
            "Get the length of a secret string, list, or binary data without exposing the content"
        );
    }

    #[test]
    fn test_examples_count() {
        let command = SecretLengthCommand;
        let examples = command.examples();
        assert_eq!(examples.len(), 4);
    }

    #[test]
    fn test_examples_have_descriptions() {
        let command = SecretLengthCommand;
        let examples = command.examples();

        for example in examples {
            assert!(!example.description.is_empty());
            assert!(example.description.len() > 10);
        }
    }

    #[test]
    fn test_examples_have_valid_results() {
        let command = SecretLengthCommand;
        let examples = command.examples();

        for example in examples {
            if let Some(expected_result) = &example.result {
                match expected_result {
                    Value::Int { val, .. } => {
                        assert!(*val >= 0, "Length should be non-negative");
                    }
                    _ => panic!("Length command examples should return integer values"),
                }
            }
        }
    }

    // Test core logic functions that would be used in the run method
    #[test]
    fn test_string_length_logic() {
        let secret = SecretString::new("test-secret".to_string());
        assert_eq!(secret.reveal().len(), 11);

        let empty_secret = SecretString::new(String::new());
        assert_eq!(empty_secret.reveal().len(), 0);

        let unicode_secret = SecretString::new("Hello 世界".to_string());
        assert_eq!(unicode_secret.reveal().len(), "Hello 世界".len()); // Byte length
    }

    #[test]
    fn test_list_length_logic() {
        let test_list = vec![
            Value::int(1, Span::test_data()),
            Value::string("test", Span::test_data()),
            Value::bool(true, Span::test_data()),
        ];
        let secret = SecretList::new(test_list);
        assert_eq!(secret.reveal().len(), 3);

        let empty_list = SecretList::new(vec![]);
        assert_eq!(empty_list.reveal().len(), 0);

        // Test large list
        let large_list: Vec<Value> = (0..1000)
            .map(|i| Value::int(i, Span::test_data()))
            .collect();
        let large_secret = SecretList::new(large_list);
        assert_eq!(large_secret.reveal().len(), 1000);
    }

    #[test]
    fn test_binary_length_logic() {
        let test_data = vec![1, 2, 3, 4, 5];
        let secret = SecretBinary::new(test_data);
        assert_eq!(secret.reveal().len(), 5);

        let empty_binary = SecretBinary::new(vec![]);
        assert_eq!(empty_binary.reveal().len(), 0);

        // Test large binary data
        let large_binary: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let binary_secret = SecretBinary::new(large_binary);
        assert_eq!(binary_secret.reveal().len(), 10000);
    }

    #[test]
    fn test_unicode_string_length() {
        // Test Unicode strings - length in bytes vs chars
        let unicode_string = "Hello 世界 🌍 мир";
        let secret = SecretString::new(unicode_string.to_string());
        assert_eq!(secret.reveal().len(), unicode_string.len()); // Byte length
        assert_eq!(
            secret.reveal().chars().count(),
            unicode_string.chars().count()
        ); // Character count

        // Test emoji handling
        let emoji_string = "🎉🎊🔐";
        let emoji_secret = SecretString::new(emoji_string.to_string());
        assert_eq!(emoji_secret.reveal().len(), emoji_string.len()); // Byte length
        assert_eq!(
            emoji_secret.reveal().chars().count(),
            emoji_string.chars().count()
        ); // Character count
    }

    #[test]
    fn test_edge_cases() {
        // Test very long string
        let long_string = "x".repeat(10000);
        let long_secret = SecretString::new(long_string.clone());
        assert_eq!(long_secret.reveal().len(), 10000);

        // Test list with mixed types
        let mixed_list = vec![
            Value::int(42, Span::test_data()),
            Value::string("mixed", Span::test_data()),
            Value::bool(true, Span::test_data()),
            Value::float(std::f64::consts::PI, Span::test_data()),
        ];
        let mixed_secret = SecretList::new(mixed_list);
        assert_eq!(mixed_secret.reveal().len(), 4);

        // Test binary with all byte values
        let full_byte_range: Vec<u8> = (0..=255).collect();
        let full_range_secret = SecretBinary::new(full_byte_range);
        assert_eq!(full_range_secret.reveal().len(), 256);
    }

    #[test]
    fn test_nested_list_length() {
        // Test nested list - should count top-level elements only
        let nested_list = vec![
            Value::list(
                vec![
                    Value::int(1, Span::test_data()),
                    Value::int(2, Span::test_data()),
                    Value::int(3, Span::test_data()),
                ],
                Span::test_data(),
            ),
            Value::string("nested", Span::test_data()),
            Value::list(
                vec![
                    Value::string("a", Span::test_data()),
                    Value::string("b", Span::test_data()),
                ],
                Span::test_data(),
            ),
        ];
        let secret_nested = SecretList::new(nested_list);
        assert_eq!(secret_nested.reveal().len(), 3); // Only top-level elements
    }
}
