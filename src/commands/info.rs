use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Record, Signature, Type, Value,
};

#[derive(Clone)]
pub struct SecretInfoCommand;

impl PluginCommand for SecretInfoCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret info"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::Record(Box::new([])))])
            .category(Category::System)
    }

    fn description(&self) -> &str {
        "Display plugin information, supported secret types, and security best practices"
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "secret info",
            description: "Show plugin information and security guidance",
            result: None,
        }]
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let mut record = Record::new();
        
        record.push("name", Value::string("nu_plugin_secret", call.head));
        record.push("version", Value::string(plugin.version(), call.head));
        record.push(
            "description",
            Value::string(
                "Production-grade secret handling plugin for Nushell with secure CustomValue types",
                call.head,
            ),
        );
        
        record.push(
            "supported_types",
            Value::list(
                vec![
                    Value::string("secret_string", call.head),
                ],
                call.head,
            ),
        );
        
        record.push(
            "commands",
            Value::list(
                vec![
                    Value::string("secret wrap-string", call.head),
                    Value::string("secret unwrap", call.head),
                    Value::string("secret info", call.head),
                    Value::string("secret validate", call.head),
                    Value::string("secret type-of", call.head),
                ],
                call.head,
            ),
        );
        
        record.push(
            "security_features",
            Value::list(
                vec![
                    Value::string("Always displays as <redacted>", call.head),
                    Value::string("Secure memory cleanup on drop", call.head),
                    Value::string("Protection against accidental serialization", call.head),
                    Value::string("Constant-time equality comparison", call.head),
                    Value::string("Debug output redaction", call.head),
                ],
                call.head,
            ),
        );
        
        record.push(
            "best_practices",
            Value::list(
                vec![
                    Value::string("Use secret types for API keys, passwords, and tokens", call.head),
                    Value::string("Minimize unwrap operations", call.head),
                    Value::string("Store secrets as environment variables when possible", call.head),
                    Value::string("Use type-specific wrap commands for clarity", call.head),
                    Value::string("Review code for accidental secret exposure", call.head),
                ],
                call.head,
            ),
        );

        Ok(PipelineData::Value(
            Value::record(record, call.head),
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretInfoCommand;
        assert_eq!(command.name(), "secret info");
    }

    #[test]
    fn test_signature() {
        let command = SecretInfoCommand;
        let sig = command.signature();
        assert_eq!(sig.name, "secret info");
        assert_eq!(sig.input_output_types.len(), 1);
        assert_eq!(sig.input_output_types[0].0, Type::Nothing);
    }
}