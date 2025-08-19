use crate::SecretBinary;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Type, Value,
};

#[derive(Clone)]
pub struct SecretWrapBinaryCommand;

impl PluginCommand for SecretWrapBinaryCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret wrap-binary"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Binary, Type::Custom("secret_binary".into()))])
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Convert binary data to a SecretBinary type that displays as <redacted:binary>"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: r#"0x[deadbeef] | secret wrap-binary"#,
                description: "Convert binary data to a secret binary",
                result: None, // We can't show the actual result since it's redacted
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
            PipelineData::Value(Value::Binary { val, .. }, metadata) => {
                let secret = SecretBinary::new(val);
                let custom_value = Value::custom(Box::new(secret), call.head);
                Ok(PipelineData::Value(custom_value, metadata))
            }
            PipelineData::Value(value, _) => Err(LabeledError::new("Type Error")
                .with_label(
                    format!("Expected binary, got {}", value.get_type()),
                    call.head,
                )
                .with_help("Only binary values can be converted to SecretBinary")),
            PipelineData::Empty => Err(LabeledError::new("Empty Input")
                .with_label("No input provided", call.head)
                .with_help("Provide binary data to convert to SecretBinary")),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot convert stream input to SecretBinary", call.head)
                .with_help("Provide a single binary value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretWrapBinaryCommand;
        assert_eq!(command.name(), "secret wrap-binary");
    }

    #[test]
    fn test_signature() {
        let command = SecretWrapBinaryCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret wrap-binary");
        assert_eq!(sig.input_output_types.len(), 1);
    }
}