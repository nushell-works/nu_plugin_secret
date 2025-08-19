use crate::{SecretString, SecretInt, SecretBool, SecretRecord, SecretList, SecretFloat, SecretBinary, SecretDate};
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
            .input_output_types(vec![
                (Type::Custom("secret_string".into()), Type::String),
                (Type::Custom("secret_int".into()), Type::Int),
                (Type::Custom("secret_bool".into()), Type::Bool),
                (Type::Custom("secret_record".into()), Type::Record(Box::new([]))),
                (Type::Custom("secret_list".into()), Type::List(Box::new(Type::Any))),
                (Type::Custom("secret_float".into()), Type::Float),
                (Type::Custom("secret_binary".into()), Type::Binary),
                (Type::Custom("secret_date".into()), Type::Date),
            ])
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Extract the underlying value from any secret type. WARNING: This exposes sensitive data!"
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
                } else if let Some(secret_int) = val.as_any().downcast_ref::<SecretInt>() {
                    let revealed = secret_int.reveal();
                    let value = Value::int(revealed, call.head);
                    Ok(PipelineData::Value(value, metadata))
                } else if let Some(secret_bool) = val.as_any().downcast_ref::<SecretBool>() {
                    let revealed = secret_bool.reveal();
                    let value = Value::bool(revealed, call.head);
                    Ok(PipelineData::Value(value, metadata))
                } else if let Some(secret_record) = val.as_any().downcast_ref::<SecretRecord>() {
                    let revealed = secret_record.reveal().clone();
                    let value = Value::record(revealed, call.head);
                    Ok(PipelineData::Value(value, metadata))
                } else if let Some(secret_list) = val.as_any().downcast_ref::<SecretList>() {
                    let revealed = secret_list.reveal().clone();
                    let value = Value::list(revealed, call.head);
                    Ok(PipelineData::Value(value, metadata))
                } else if let Some(secret_float) = val.as_any().downcast_ref::<SecretFloat>() {
                    let revealed = secret_float.reveal();
                    let value = Value::float(revealed, call.head);
                    Ok(PipelineData::Value(value, metadata))
                } else if let Some(secret_binary) = val.as_any().downcast_ref::<SecretBinary>() {
                    let revealed = secret_binary.reveal().clone();
                    let value = Value::binary(revealed, call.head);
                    Ok(PipelineData::Value(value, metadata))
                } else if let Some(secret_date) = val.as_any().downcast_ref::<SecretDate>() {
                    let revealed = secret_date.reveal().clone();
                    let value = Value::date(revealed, call.head);
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
        assert_eq!(sig.input_output_types.len(), 8);
        assert_eq!(sig.input_output_types[0].1, Type::String);
    }
}