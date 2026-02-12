use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, Type, Value};

use crate::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretList, SecretRecord,
    SecretString,
};

#[derive(Clone)]
pub struct SecretTypeOfCommand;

impl PluginCommand for SecretTypeOfCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret type-of"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![
                (Type::Custom("secret_string".into()), Type::String),
                (Type::Custom("secret_int".into()), Type::String),
                (Type::Custom("secret_bool".into()), Type::String),
                (Type::Custom("secret_record".into()), Type::String),
                (Type::Custom("secret_list".into()), Type::String),
                (Type::Custom("secret_float".into()), Type::String),
                (Type::Custom("secret_binary".into()), Type::String),
                (Type::Custom("secret_date".into()), Type::String),
            ])
            .category(Category::Core)
    }

    fn description(&self) -> &str {
        "Get the underlying type of a secret value without exposing the content"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            example: r#""my-secret" | secret wrap | secret type-of"#,
            description: "Get the underlying type of a secret string",
            result: Some(Value::test_string("string")),
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
            PipelineData::Value(Value::Custom { val, .. }, metadata) => {
                let underlying_type = if val.as_any().downcast_ref::<SecretString>().is_some() {
                    "string"
                } else if val.as_any().downcast_ref::<SecretInt>().is_some() {
                    "int"
                } else if val.as_any().downcast_ref::<SecretBool>().is_some() {
                    "bool"
                } else if val.as_any().downcast_ref::<SecretRecord>().is_some() {
                    "record"
                } else if val.as_any().downcast_ref::<SecretList>().is_some() {
                    "list"
                } else if val.as_any().downcast_ref::<SecretFloat>().is_some() {
                    "float"
                } else if val.as_any().downcast_ref::<SecretBinary>().is_some() {
                    "binary"
                } else if val.as_any().downcast_ref::<SecretDate>().is_some() {
                    "date"
                } else {
                    "unknown"
                };

                Ok(PipelineData::Value(
                    Value::string(underlying_type, call.head),
                    metadata,
                ))
            }
            PipelineData::Value(value, _) => Err(LabeledError::new("Type Error")
                .with_label(
                    format!("Expected secret type, got {}", value.get_type()),
                    call.head,
                )
                .with_help("Only secret types have underlying types")),
            PipelineData::Empty => Err(LabeledError::new("Empty Input")
                .with_label("No input provided", call.head)
                .with_help("Provide a secret value to get its type")),
            _ => Err(LabeledError::new("Unsupported Input")
                .with_label("Cannot get type of stream input", call.head)
                .with_help("Provide a single secret value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretTypeOfCommand;
        assert_eq!(command.name(), "secret type-of");
    }

    #[test]
    fn test_signature() {
        let command = SecretTypeOfCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret type-of");
        assert_eq!(sig.input_output_types.len(), 8);
        assert_eq!(sig.input_output_types[0].1, Type::String);
    }
}
