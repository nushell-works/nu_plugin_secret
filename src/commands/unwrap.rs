use crate::SecretString;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Type, Value,
};

#[derive(Clone)]
pub struct SecretUnwrapCommand;

impl PluginCommand for SecretUnwrapCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret unwrap"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Custom("secret_string".into()), Type::String)])
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Extract the underlying value from a secret type. WARNING: This exposes sensitive data!"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: r#""my-secret" | secret wrap-string | secret unwrap"#,
                description: "Unwrap a secret string to get the original value",
                result: Some(Value::test_string("my-secret")),
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
            PipelineData::Value(Value::Custom { val, .. }, metadata) => {
                // Security warning would be logged here in a real implementation
                // For now, we'll just proceed with the unwrap operation

                if let Some(secret_string) = val.as_any().downcast_ref::<SecretString>() {
                    let revealed = secret_string.reveal().to_string();
                    let value = Value::string(revealed, call.head);
                    Ok(PipelineData::Value(value, metadata))
                } else {
                    Err(LabeledError::new("Type Error")
                        .with_label("Expected a secret type", call.head)
                        .with_help("Only secret types can be unwrapped"))
                }
            }
            PipelineData::Value(value, _) => Err(LabeledError::new("Type Error")
                .with_label(
                    format!("Expected secret type, got {}", value.get_type()),
                    call.head,
                )
                .with_help("Only secret types can be unwrapped")),
            PipelineData::Empty => Err(LabeledError::new("Empty Input")
                .with_label("No input provided", call.head)
                .with_help("Provide a secret value to unwrap")),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot unwrap stream input", call.head)
                .with_help("Provide a single secret value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretUnwrapCommand;
        assert_eq!(command.name(), "secret unwrap");
    }

    #[test]
    fn test_signature() {
        let command = SecretUnwrapCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret unwrap");
        assert_eq!(sig.input_output_types.len(), 1);
        assert_eq!(sig.input_output_types[0].1, Type::String);
    }
}