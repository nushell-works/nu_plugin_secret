use crate::SecretDate;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

#[derive(Clone)]
pub struct SecretWrapDateCommand;

impl PluginCommand for SecretWrapDateCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret wrap-date"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Date, Type::Custom("secret_date".into()))])
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Convert a date/datetime to a SecretDate type that displays as <redacted:date>"
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: r#"date now | secret wrap-date"#,
            description: "Convert current datetime to a secret date",
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
            PipelineData::Value(Value::Date { val, .. }, metadata) => {
                let secret = SecretDate::new(val);
                let custom_value = Value::custom(Box::new(secret), call.head);
                Ok(PipelineData::Value(custom_value, metadata))
            }
            PipelineData::Value(value, _) => Err(LabeledError::new("Type Error")
                .with_label(
                    format!("Expected date, got {}", value.get_type()),
                    call.head,
                )
                .with_help("Only date/datetime values can be converted to SecretDate")),
            PipelineData::Empty => Err(LabeledError::new("Empty Input")
                .with_label("No input provided", call.head)
                .with_help("Provide a date value to convert to SecretDate")),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot convert stream input to SecretDate", call.head)
                .with_help("Provide a single date value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretWrapDateCommand;
        assert_eq!(command.name(), "secret wrap-date");
    }

    #[test]
    fn test_signature() {
        let command = SecretWrapDateCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret wrap-date");
        assert_eq!(sig.input_output_types.len(), 1);
    }
}
