use crate::SecretRecord;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

#[derive(Clone)]
pub struct SecretWrapRecordCommand;

impl PluginCommand for SecretWrapRecordCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret wrap-record"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(
                Type::Record(Box::new([])),
                Type::Custom("secret_record".into()),
            )])
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Convert a record to a SecretRecord type that displays as <redacted:record>"
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: r#"{api_key: "secret", token: "hidden"} | secret wrap-record"#,
            description: "Convert a record to a secret record",
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
            PipelineData::Value(Value::Record { val, .. }, metadata) => {
                let secret = SecretRecord::new(val.into_owned());
                let custom_value = Value::custom(Box::new(secret), call.head);
                Ok(PipelineData::Value(custom_value, metadata))
            }
            PipelineData::Value(value, _) => Err(LabeledError::new("Type Error")
                .with_label(
                    format!("Expected record, got {}", value.get_type()),
                    call.head,
                )
                .with_help("Only record values can be converted to SecretRecord")),
            PipelineData::Empty => Err(LabeledError::new("Empty Input")
                .with_label("No input provided", call.head)
                .with_help("Provide a record value to convert to SecretRecord")),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot convert stream input to SecretRecord", call.head)
                .with_help("Provide a single record value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretWrapRecordCommand;
        assert_eq!(command.name(), "secret wrap-record");
    }

    #[test]
    fn test_signature() {
        let command = SecretWrapRecordCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret wrap-record");
        assert_eq!(sig.input_output_types.len(), 1);
    }
}
