use crate::SecretFloat;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

#[derive(Clone)]
pub struct SecretWrapFloatCommand;

impl PluginCommand for SecretWrapFloatCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret wrap-float"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Float, Type::Custom("secret_float".into()))])
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Convert a float to a SecretFloat type that displays as <redacted:float>"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            example: r#"3.14159 | secret wrap-float"#,
            description: "Convert a float to a secret float",
            result: None, // We can't show the actual result since it's redacted
        }]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        match input {
            PipelineData::Value(Value::Float { val, .. }, metadata) => {
                let secret = SecretFloat::new(val);
                let custom_value = Value::custom(Box::new(secret), call.head);
                Ok(PipelineData::Value(custom_value, metadata))
            }
            PipelineData::Value(value, _) => Err(LabeledError::new("Type Error")
                .with_label(
                    format!("Expected float, got {}", value.get_type()),
                    call.head,
                )
                .with_help("Only float values can be converted to SecretFloat")),
            PipelineData::Empty => Err(LabeledError::new("Empty Input")
                .with_label("No input provided", call.head)
                .with_help("Provide a float value to convert to SecretFloat")),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot convert stream input to SecretFloat", call.head)
                .with_help("Provide a single float value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretWrapFloatCommand;
        assert_eq!(command.name(), "secret wrap-float");
    }

    #[test]
    fn test_signature() {
        let command = SecretWrapFloatCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret wrap-float");
        assert_eq!(sig.input_output_types.len(), 1);
    }
}
