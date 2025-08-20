use crate::SecretInt;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

#[derive(Clone)]
pub struct SecretWrapIntCommand;

impl PluginCommand for SecretWrapIntCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret wrap-int"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Int, Type::Custom("secret_int".into()))])
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Convert an integer value to a SecretInt type that displays as <redacted:int>"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: r#"42 | secret wrap-int"#,
                description: "Convert an integer to a secret integer",
                result: None, // We can't show the actual result since it's redacted
            },
            Example {
                example: r#"$env.PORT | secret wrap-int"#,
                description: "Convert an environment variable to a secret integer",
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
            PipelineData::Value(Value::Int { val, .. }, metadata) => {
                let secret = SecretInt::new(val);
                let custom_value = Value::custom(Box::new(secret), call.head);
                Ok(PipelineData::Value(custom_value, metadata))
            }
            PipelineData::Value(value, _) => Err(LabeledError::new("Type Error")
                .with_label(
                    format!("Expected integer, got {}", value.get_type()),
                    call.head,
                )
                .with_help("Only integer values can be converted to SecretInt")),
            PipelineData::Empty => Err(LabeledError::new("Empty Input")
                .with_label("No input provided", call.head)
                .with_help("Provide an integer value to convert to SecretInt")),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot convert stream input to SecretInt", call.head)
                .with_help("Provide a single integer value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretWrapIntCommand;
        assert_eq!(command.name(), "secret wrap-int");
    }

    #[test]
    fn test_signature() {
        let command = SecretWrapIntCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret wrap-int");
        assert_eq!(sig.input_output_types.len(), 1);
        assert_eq!(sig.input_output_types[0].0, Type::Int);
    }
}
