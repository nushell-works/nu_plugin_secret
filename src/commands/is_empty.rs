//! Implements `secret is-empty` — checks if a secret contains empty data.

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

use crate::{SecretBinary, SecretList, SecretRecord, SecretString};

#[derive(Clone)]
pub struct SecretIsEmptyCommand;

/// Check whether a secret custom value is empty.
///
/// Dispatches to the appropriate `is_empty()` method for each supported
/// secret type (SecretString, SecretBinary, SecretList, SecretRecord).
fn check_secret_is_empty(
    val: &dyn nu_protocol::CustomValue,
    span: nu_protocol::Span,
) -> Result<Value, LabeledError> {
    if let Some(secret_string) = val.as_any().downcast_ref::<SecretString>() {
        Ok(Value::bool(secret_string.is_empty(), span))
    } else if let Some(secret_binary) = val.as_any().downcast_ref::<SecretBinary>() {
        Ok(Value::bool(secret_binary.is_empty(), span))
    } else if let Some(secret_list) = val.as_any().downcast_ref::<SecretList>() {
        Ok(Value::bool(secret_list.is_empty(), span))
    } else if let Some(secret_record) = val.as_any().downcast_ref::<SecretRecord>() {
        Ok(Value::bool(secret_record.is_empty(), span))
    } else {
        Err(LabeledError::new("Unsupported secret type").with_label(
            "Only SecretString, SecretBinary, SecretList, and SecretRecord support is-empty operation",
            span,
        ))
    }
}

impl PluginCommand for SecretIsEmptyCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret is-empty"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![
                (Type::Custom("secret_string".into()), Type::Bool),
                (Type::Custom("secret_binary".into()), Type::Bool),
                (Type::Custom("secret_list".into()), Type::Bool),
                (Type::Custom("secret_record".into()), Type::Bool),
            ])
            .category(Category::Filters)
    }

    fn description(&self) -> &str {
        "Check if a secret contains empty data without exposing the content"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: r#""hello" | secret wrap | secret is-empty"#,
                description: "Check if a secret string is empty (false)",
                result: Some(Value::bool(false, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""" | secret wrap | secret is-empty"#,
                description: "Check if an empty secret string is empty (true)",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#"[] | secret wrap | secret is-empty"#,
                description: "Check if an empty secret list is empty (true)",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#"[1, 2, 3] | secret wrap | secret is-empty"#,
                description: "Check if a non-empty secret list is empty (false)",
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
        match input {
            PipelineData::Value(value, metadata) => {
                let result = match value {
                    Value::Custom { val, .. } => check_secret_is_empty(val.as_ref(), call.head)?,
                    _ => {
                        return Err(LabeledError::new("Invalid input").with_label(
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
    use nu_protocol::{Record, Span};

    #[test]
    fn test_command_name() {
        let command = SecretIsEmptyCommand;
        assert_eq!(command.name(), "secret is-empty");
    }

    #[test]
    fn test_signature() {
        let command = SecretIsEmptyCommand;
        let signature = command.signature();
        assert_eq!(signature.name, "secret is-empty");
        assert_eq!(signature.required_positional.len(), 0);
        assert_eq!(signature.input_output_types.len(), 4);
    }

    #[test]
    fn test_description() {
        let command = SecretIsEmptyCommand;
        assert_eq!(
            command.description(),
            "Check if a secret contains empty data without exposing the content"
        );
    }

    #[test]
    fn test_examples_count() {
        let command = SecretIsEmptyCommand;
        let examples = command.examples();
        assert_eq!(examples.len(), 4);
    }

    #[test]
    fn test_examples_have_descriptions() {
        let command = SecretIsEmptyCommand;
        let examples = command.examples();

        for example in examples {
            assert!(!example.description.is_empty());
            assert!(example.description.len() > 10);
        }
    }

    #[test]
    fn test_examples_have_valid_results() {
        let command = SecretIsEmptyCommand;
        let examples = command.examples();

        for example in examples {
            assert!(
                example.result.is_some(),
                "All examples should have expected results"
            );
            match example.result.as_ref().unwrap() {
                Value::Bool { .. } => {}
                _ => panic!("is-empty command examples should return boolean values"),
            }
        }
    }

    // Tests exercising check_secret_is_empty — the extracted dispatch function

    #[test]
    fn test_empty_string() {
        let secret = SecretString::new(String::new());
        let result = check_secret_is_empty(&secret, Span::test_data()).unwrap();
        assert_eq!(result, Value::bool(true, Span::test_data()));
    }

    #[test]
    fn test_non_empty_string() {
        let secret = SecretString::new("hello".to_string());
        let result = check_secret_is_empty(&secret, Span::test_data()).unwrap();
        assert_eq!(result, Value::bool(false, Span::test_data()));
    }

    #[test]
    fn test_empty_binary() {
        let secret = SecretBinary::new(vec![]);
        let result = check_secret_is_empty(&secret, Span::test_data()).unwrap();
        assert_eq!(result, Value::bool(true, Span::test_data()));
    }

    #[test]
    fn test_non_empty_binary() {
        let secret = SecretBinary::new(vec![0xDE, 0xAD]);
        let result = check_secret_is_empty(&secret, Span::test_data()).unwrap();
        assert_eq!(result, Value::bool(false, Span::test_data()));
    }

    #[test]
    fn test_empty_list() {
        let secret = SecretList::new(vec![]);
        let result = check_secret_is_empty(&secret, Span::test_data()).unwrap();
        assert_eq!(result, Value::bool(true, Span::test_data()));
    }

    #[test]
    fn test_non_empty_list() {
        let secret = SecretList::new(vec![Value::int(1, Span::test_data())]);
        let result = check_secret_is_empty(&secret, Span::test_data()).unwrap();
        assert_eq!(result, Value::bool(false, Span::test_data()));
    }

    #[test]
    fn test_empty_record() {
        let secret = SecretRecord::new(Record::new());
        let result = check_secret_is_empty(&secret, Span::test_data()).unwrap();
        assert_eq!(result, Value::bool(true, Span::test_data()));
    }

    #[test]
    fn test_non_empty_record() {
        let mut record = Record::new();
        record.push("key", Value::test_string("value"));
        let secret = SecretRecord::new(record);
        let result = check_secret_is_empty(&secret, Span::test_data()).unwrap();
        assert_eq!(result, Value::bool(false, Span::test_data()));
    }

    #[test]
    fn test_unsupported_type() {
        use crate::SecretInt;
        let secret = SecretInt::new(42);
        let result = check_secret_is_empty(&secret, Span::test_data());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.msg, "Unsupported secret type");
    }
}
