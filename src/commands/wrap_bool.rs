use crate::SecretBool;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Type, Value,
};

#[derive(Clone)]
pub struct SecretWrapBoolCommand;

impl PluginCommand for SecretWrapBoolCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret wrap-bool"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Bool, Type::Custom("secret_bool".into()))])
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Convert a boolean value to a SecretBool type that displays as <redacted:bool>"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: r#"true | secret wrap-bool"#,
                description: "Convert a boolean to a secret boolean",
                result: None, // We can't show the actual result since it's redacted
            },
            Example {
                example: r#"false | secret wrap-bool"#,
                description: "Convert false to a secret boolean",
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
            PipelineData::Value(Value::Bool { val, .. }, metadata) => {
                let secret = SecretBool::new(val);
                let custom_value = Value::custom(Box::new(secret), call.head);
                Ok(PipelineData::Value(custom_value, metadata))
            }
            PipelineData::Value(value, _) => Err(LabeledError::new("Type Error")
                .with_label(
                    format!("Expected boolean, got {}", value.get_type()),
                    call.head,
                )
                .with_help("Only boolean values can be converted to SecretBool")),
            PipelineData::Empty => Err(LabeledError::new("Empty Input")
                .with_label("No input provided", call.head)
                .with_help("Provide a boolean value to convert to SecretBool")),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot convert stream input to SecretBool", call.head)
                .with_help("Provide a single boolean value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretWrapBoolCommand;
        assert_eq!(command.name(), "secret wrap-bool");
    }

    #[test]
    fn test_signature() {
        let command = SecretWrapBoolCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret wrap-bool");
        assert_eq!(sig.input_output_types.len(), 1);
        assert_eq!(sig.input_output_types[0].0, Type::Bool);
    }
}