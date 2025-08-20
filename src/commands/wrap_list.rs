use crate::SecretList;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

#[derive(Clone)]
pub struct SecretWrapListCommand;

impl PluginCommand for SecretWrapListCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret wrap-list"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(
                Type::List(Box::new(Type::Any)),
                Type::Custom("secret_list".into()),
            )])
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Convert a list to a SecretList type that displays as <redacted:list>"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            example: r#"["secret1", "secret2"] | secret wrap-list"#,
            description: "Convert a list to a secret list",
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
            PipelineData::Value(Value::List { vals, .. }, metadata) => {
                let secret = SecretList::new(vals);
                let custom_value = Value::custom(Box::new(secret), call.head);
                Ok(PipelineData::Value(custom_value, metadata))
            }
            PipelineData::Value(value, _) => Err(LabeledError::new("Type Error")
                .with_label(
                    format!("Expected list, got {}", value.get_type()),
                    call.head,
                )
                .with_help("Only list values can be converted to SecretList")),
            PipelineData::Empty => Err(LabeledError::new("Empty Input")
                .with_label("No input provided", call.head)
                .with_help("Provide a list value to convert to SecretList")),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot convert stream input to SecretList", call.head)
                .with_help("Provide a single list value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretWrapListCommand;
        assert_eq!(command.name(), "secret wrap-list");
    }

    #[test]
    fn test_signature() {
        let command = SecretWrapListCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret wrap-list");
        assert_eq!(sig.input_output_types.len(), 1);
    }
}
