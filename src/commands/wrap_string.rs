use crate::SecretString;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

#[derive(Clone)]
pub struct SecretWrapStringCommand;

impl PluginCommand for SecretWrapStringCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret wrap-string"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::String, Type::Custom("secret_string".into()))])
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Convert a string value to a SecretString type that displays as <redacted:string>"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: r#""my-api-key" | secret wrap-string"#,
                description: "Convert a string to a secret string",
                result: None, // We can't show the actual result since it's redacted
            },
            Example {
                example: r#"$env.API_KEY | secret wrap-string"#,
                description: "Convert an environment variable to a secret string",
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
            PipelineData::Value(Value::String { val, .. }, metadata) => {
                let secret = SecretString::new(val);
                let custom_value = Value::custom(Box::new(secret), call.head);
                Ok(PipelineData::Value(custom_value, metadata))
            }
            PipelineData::Value(value, _) => Err(LabeledError::new("Type Error")
                .with_label(
                    format!("Expected string, got {}", value.get_type()),
                    call.head,
                )
                .with_help("Only string values can be converted to SecretString")),
            PipelineData::Empty => Err(LabeledError::new("Empty Input")
                .with_label("No input provided", call.head)
                .with_help("Provide a string value to convert to SecretString")),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot convert stream input to SecretString", call.head)
                .with_help("Provide a single string value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::Span;

    #[test]
    fn test_command_name() {
        let command = SecretWrapStringCommand;
        assert_eq!(command.name(), "secret wrap-string");
    }

    #[test]
    fn test_signature() {
        let command = SecretWrapStringCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret wrap-string");
        assert_eq!(sig.input_output_types.len(), 1);
        assert_eq!(sig.input_output_types[0].0, Type::String);
    }
}
