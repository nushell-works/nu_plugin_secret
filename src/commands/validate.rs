use crate::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretList, SecretRecord,
    SecretString,
};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

#[derive(Clone)]
pub struct SecretValidateCommand;

impl PluginCommand for SecretValidateCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret validate"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Any, Type::Bool)])
            .category(Category::Core)
    }

    fn description(&self) -> &str {
        "Check if a value is any secret type. Returns true for secret types, false otherwise."
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: r#""my-secret" | secret wrap | secret validate"#,
                description: "Check if a secret string is a secret type",
                result: Some(Value::test_bool(true)),
            },
            Example {
                example: r#""regular-string" | secret validate"#,
                description: "Check if a regular string is a secret type",
                result: Some(Value::test_bool(false)),
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
                // Check if it's any known secret type
                let is_secret = val.as_any().downcast_ref::<SecretString>().is_some()
                    || val.as_any().downcast_ref::<SecretInt>().is_some()
                    || val.as_any().downcast_ref::<SecretBool>().is_some()
                    || val.as_any().downcast_ref::<SecretRecord>().is_some()
                    || val.as_any().downcast_ref::<SecretList>().is_some()
                    || val.as_any().downcast_ref::<SecretFloat>().is_some()
                    || val.as_any().downcast_ref::<SecretBinary>().is_some()
                    || val.as_any().downcast_ref::<SecretDate>().is_some();

                Ok(PipelineData::Value(
                    Value::bool(is_secret, call.head),
                    metadata,
                ))
            }
            PipelineData::Value(_, metadata) => {
                // Not a custom value, so not a secret type
                Ok(PipelineData::Value(Value::bool(false, call.head), metadata))
            }
            PipelineData::Empty => Ok(PipelineData::Value(Value::bool(false, call.head), None)),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot validate stream input", call.head)
                .with_help("Provide a single value to validate")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretValidateCommand;
        assert_eq!(command.name(), "secret validate");
    }

    #[test]
    fn test_signature() {
        let command = SecretValidateCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret validate");
        assert_eq!(sig.input_output_types.len(), 1);
        assert_eq!(sig.input_output_types[0].0, Type::Any);
        assert_eq!(sig.input_output_types[0].1, Type::Bool);
    }
}
