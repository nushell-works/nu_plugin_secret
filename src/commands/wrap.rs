use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

use crate::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretList, SecretRecord,
    SecretString,
};

#[derive(Clone)]
pub struct SecretWrapCommand;

impl PluginCommand for SecretWrapCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret wrap"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
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
        "Convert any value to its corresponding secret type that displays as redacted"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: r#""my-api-key" | secret wrap"#,
                description: "Convert a string to a secret string",
                result: None, // We can't show the actual result since it's redacted
            },
            Example {
                example: r#"42 | secret wrap"#,
                description: "Convert an integer to a secret integer",
                result: None,
            },
            Example {
                example: r#"true | secret wrap"#,
                description: "Convert a boolean to a secret boolean",
                result: None,
            },
            Example {
                example: r#"3.14 | secret wrap"#,
                description: "Convert a float to a secret float",
                result: None,
            },
            Example {
                example: r#"(date now) | secret wrap"#,
                description: "Convert a date to a secret date",
                result: None,
            },
            Example {
                example: r#"[1, 2, 3] | secret wrap"#,
                description: "Convert a list to a secret list",
                result: None,
            },
            Example {
                example: r#"{name: "john", age: 30} | secret wrap"#,
                description: "Convert a record to a secret record",
                result: None,
            },
            Example {
                example: r#"$env.API_KEY | secret wrap"#,
                description: "Convert any environment variable to its appropriate secret type",
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
        match input {
            PipelineData::Value(value, metadata) => {
                let wrapped_value = match value {
                    Value::String { val, .. } => {
                        let secret = SecretString::new(val);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Int { val, .. } => {
                        let secret = SecretInt::new(val);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Bool { val, .. } => {
                        let secret = SecretBool::new(val);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Float { val, .. } => {
                        let secret = SecretFloat::new(val);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Date { val, .. } => {
                        let secret = SecretDate::new(val);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Binary { val, .. } => {
                        let secret = SecretBinary::new(val);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::List { vals, .. } => {
                        let secret = SecretList::new(vals);
                        Value::custom(Box::new(secret), call.head)
                    }
                    Value::Record { val, .. } => {
                        let secret = SecretRecord::new(val.into_owned());
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

    #[test]
    fn test_command_name() {
        let command = SecretWrapCommand;
        assert_eq!(command.name(), "secret wrap");
    }

    #[test]
    fn test_signature() {
        let command = SecretWrapCommand;
        let signature = command.signature();
        assert_eq!(signature.name, "secret wrap");
        assert_eq!(signature.input_output_types.len(), 8);
    }
}
