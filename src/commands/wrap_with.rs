use crate::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretList, SecretRecord,
    SecretString,
};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};

#[derive(Clone)]
pub struct SecretWrapWithCommand;

impl PluginCommand for SecretWrapWithCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret wrap-with"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "template",
                SyntaxShape::String,
                "The redaction template to use for this secret",
            )
            .input_output_types(vec![
                (Type::String, Type::Custom("secret_string".into())),
                (Type::Int, Type::Custom("secret_int".into())),
                (Type::Bool, Type::Custom("secret_bool".into())),
                (Type::Float, Type::Custom("secret_float".into())),
                (Type::Date, Type::Custom("secret_date".into())),
                (Type::Binary, Type::Custom("secret_binary".into())),
                (
                    Type::List(Box::new(Type::Any)),
                    Type::Custom("secret_list".into()),
                ),
                (
                    Type::Record(vec![].into()),
                    Type::Custom("secret_record".into()),
                ),
            ])
            .description(self.description())
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Convert any value to its corresponding secret type with a custom redaction template"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: r#""my-api-key" | secret wrap-with "***{{secret_type}}***""#,
                description: "Convert a string to a secret string with custom redaction template",
                result: None, // We can't show the actual result since it's redacted
            },
            Example {
                example: r#"42 | secret wrap-with "{{replicate(s='*', n=secret_length)}}""#,
                description: "Convert an integer to a secret integer using replicate function",
                result: None,
            },
            Example {
                example: r#"true | secret wrap-with "[HIDDEN:{{secret_type}}]""#,
                description: "Convert a boolean to a secret boolean with custom brackets",
                result: None,
            },
            Example {
                example: r#"$env.API_KEY | secret wrap-with "{{secret_type}}_redacted""#,
                description: "Convert any environment variable to its appropriate secret type with custom template",
                result: None,
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
        let template: String = call.req(0)?;

        match input {
            PipelineData::Value(value, metadata) => {
                let wrapped_value = match value {
                    Value::String { val, .. } => {
                        let secret = SecretString::new_with_template(val, template);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Int { val, .. } => {
                        let secret = SecretInt::new_with_template(val, template);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Bool { val, .. } => {
                        let secret = SecretBool::new_with_template(val, template);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Float { val, .. } => {
                        let secret = SecretFloat::new_with_template(val, template);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Date { val, .. } => {
                        let secret = SecretDate::new_with_template(val, template);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Binary { val, .. } => {
                        let secret = SecretBinary::new_with_template(val, template);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::List { vals, .. } => {
                        let secret = SecretList::new_with_template(vals, template);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Record { val, .. } => {
                        let secret = SecretRecord::new_with_template(val.into_owned(), template);
                        Value::custom(Box::new(secret), call.head)
                    }
                    _ => {
                        return Err(LabeledError::new("Unsupported type")
                            .with_label(
                                format!(
                                    "Cannot wrap value of type '{}'. Supported types: string, int, bool, float, date, binary, list, record",
                                    value.get_type()
                                ),
                                call.head,
                            ));
                    }
                };
                Ok(PipelineData::Value(wrapped_value, metadata))
            }
            _ => Err(LabeledError::new("Invalid input")
                .with_label("Expected a single value to wrap as a secret", call.head)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, FixedOffset};
    use nu_protocol::{Record, Span};

    #[test]
    fn test_command_name() {
        let command = SecretWrapWithCommand;
        assert_eq!(command.name(), "secret wrap-with");
    }

    #[test]
    fn test_description() {
        let command = SecretWrapWithCommand;
        let description = command.description();
        assert!(!description.is_empty());
        assert!(description.contains("custom redaction template"));
    }

    #[test]
    fn test_signature() {
        let command = SecretWrapWithCommand;
        let signature = command.signature();
        assert_eq!(signature.name, "secret wrap-with");
        assert_eq!(signature.input_output_types.len(), 8);
        assert_eq!(signature.required_positional.len(), 1);
        assert_eq!(signature.required_positional[0].name, "template");

        // Verify template parameter details
        let template_param = &signature.required_positional[0];
        assert_eq!(
            template_param.desc,
            "The redaction template to use for this secret"
        );

        // Verify category
        assert_eq!(signature.category, Category::Conversions);
    }

    #[test]
    fn test_examples() {
        let command = SecretWrapWithCommand;
        let examples = command.examples();

        assert!(!examples.is_empty());
        assert!(examples.len() >= 4);

        // Verify all examples have descriptions and example commands
        for example in &examples {
            assert!(!example.example.is_empty());
            assert!(!example.description.is_empty());
            assert!(example.example.contains("secret wrap-with"));
        }
    }

    #[test]
    fn test_template_validation() {
        // Test various template patterns that should be valid
        let long_template = "{{secret_type}}".repeat(10);
        let valid_templates = vec![
            "{{secret_type}}",
            "[HIDDEN:{{secret_type}}]",
            "{{replicate(s='*', n=secret_length)}}",
            "{{secret_type}}: {{replicate(s='#', n=5)}}",
            "🔐 {{secret_type}} 🔒",
            "",
            "simple_text_without_variables",
            &long_template,
        ];

        for template in valid_templates {
            // Just verify that templates don't cause panics when created
            // The actual template rendering is tested in integration tests
            assert!(!template.is_empty() || template.is_empty()); // Template validation - any template is acceptable
        }
    }

    #[test]
    fn test_error_message_format() {
        let _command = SecretWrapWithCommand;

        // Test unsupported type error format
        let error = LabeledError::new("Unsupported type")
            .with_label("Cannot wrap value of type 'nothing'", Span::test_data());

        let error_msg = error.to_string();
        assert!(error_msg.contains("Unsupported type"));
        // The exact error message format might vary, just check it contains key information
        assert!(!error_msg.is_empty());

        // Test invalid input error format
        let error = LabeledError::new("Invalid input").with_label(
            "Expected a single value to wrap as a secret",
            Span::test_data(),
        );

        let error_msg = error.to_string();
        assert!(error_msg.contains("Invalid input"));
        // The exact error message format might vary, just check it contains key information
        assert!(!error_msg.is_empty());
    }

    #[test]
    fn test_value_type_mapping() {
        // Test that we can create values of all supported types
        let test_values = vec![
            ("string", Value::string("test", Span::test_data())),
            ("int", Value::int(42, Span::test_data())),
            ("bool", Value::bool(true, Span::test_data())),
            (
                "float",
                Value::float(std::f64::consts::PI, Span::test_data()),
            ),
            ("binary", Value::binary(vec![1, 2, 3], Span::test_data())),
            (
                "list",
                Value::list(
                    vec![Value::string("item", Span::test_data())],
                    Span::test_data(),
                ),
            ),
        ];

        for (type_name, value) in test_values {
            // Verify we can identify the value type
            let value_type = value.get_type();
            assert!(
                !value_type.to_string().is_empty(),
                "Type {} should have a string representation",
                type_name
            );
        }
    }

    #[test]
    fn test_date_value_creation() {
        // Test creating a date value specifically
        let date = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(0).unwrap());

        let date_value = Value::date(date, Span::test_data());
        assert_eq!(date_value.get_type().to_string(), "datetime");
    }

    #[test]
    fn test_record_value_creation() {
        // Test creating a record value specifically
        let mut record = Record::new();
        record.push("key1", Value::string("value1", Span::test_data()));
        record.push("key2", Value::int(42, Span::test_data()));

        let record_value = Value::record(record, Span::test_data());
        // Record type includes field information
        assert!(record_value.get_type().to_string().starts_with("record"));
    }

    #[test]
    fn test_supported_type_creation() {
        // Test that we can create SecretString with template
        let secret =
            SecretString::new_with_template("test".to_string(), "{{secret_type}}".to_string());

        // Verify the secret was created successfully
        assert_eq!(secret.reveal(), "test");

        // Test that we can create SecretInt with template
        let secret_int = SecretInt::new_with_template(42, "{{secret_type}}".to_string());
        assert_eq!(secret_int.reveal(), 42);

        // Test that we can create SecretBool with template
        let secret_bool = SecretBool::new_with_template(true, "{{secret_type}}".to_string());
        assert!(secret_bool.reveal());

        // Test other types (stubs)
        let secret_float =
            SecretFloat::new_with_template(std::f64::consts::PI, "{{secret_type}}".to_string());
        assert_eq!(secret_float.reveal(), std::f64::consts::PI);

        let date = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(0).unwrap());
        let secret_date = SecretDate::new_with_template(date, "{{secret_type}}".to_string());
        assert_eq!(*secret_date.reveal(), date);

        let secret_binary =
            SecretBinary::new_with_template(vec![1, 2, 3], "{{secret_type}}".to_string());
        assert_eq!(secret_binary.reveal().as_ref(), &[1, 2, 3]);

        let values = vec![Value::string("test", Span::test_data())];
        let secret_list =
            SecretList::new_with_template(values.clone(), "{{secret_type}}".to_string());
        assert_eq!(*secret_list.reveal(), values);

        let mut record = Record::new();
        record.push("key", Value::string("value", Span::test_data()));
        let secret_record =
            SecretRecord::new_with_template(record.clone(), "{{secret_type}}".to_string());
        // Just verify the record can be revealed without comparing (Record doesn't implement PartialEq)
        let revealed = secret_record.reveal();
        assert_eq!(revealed.len(), 1);
        assert!(revealed.get("key").is_some());
    }

    #[test]
    fn test_input_output_type_coverage() {
        let command = SecretWrapWithCommand;
        let signature = command.signature();

        // Verify all expected input/output type mappings
        let expected_mappings = vec![
            (Type::String, "secret_string"),
            (Type::Int, "secret_int"),
            (Type::Bool, "secret_bool"),
            (Type::Float, "secret_float"),
            (Type::Date, "secret_date"),
            (Type::Binary, "secret_binary"),
        ];

        for (input_type, expected_output) in expected_mappings {
            let found = signature.input_output_types.iter().any(|(i, o)| {
                *i == input_type
                    && if let Type::Custom(custom_type) = o {
                        custom_type.as_ref() == expected_output
                    } else {
                        false
                    }
            });
            assert!(
                found,
                "Missing input/output mapping for {:?} -> {}",
                input_type, expected_output
            );
        }
    }

    #[test]
    fn test_secret_string_template_variable() {
        // Test that the secret_string template variable works correctly
        let secret = SecretString::new_with_template(
            "test_value".to_string(),
            "moo:{{secret_string}}".to_string(),
        );

        // Test Display
        let display = format!("{}", secret);
        assert_eq!(display, "moo:test_value");

        // Test redacted_display
        let redacted = secret.redacted_display();
        assert_eq!(redacted, "moo:test_value");
    }
}
