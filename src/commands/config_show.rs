//! Configuration display command for nu_plugin_secret

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Record, Signature, Type, Value};

/// Command to display current configuration settings
pub struct SecretConfigShowCommand;

impl PluginCommand for SecretConfigShowCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret config show"
    }

    fn description(&self) -> &str {
        "Display current secret plugin configuration settings"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::Record(Box::new([])))])
            .switch("raw", "Show raw TOML configuration", Some('r'))
            .switch("file-path", "Show configuration file path only", Some('f'))
            .category(Category::Custom("secret".into()))
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "secret config show",
                description: "Display current configuration in structured format",
                result: None,
            },
            Example {
                example: "secret config show --raw",
                description: "Display raw TOML configuration",
                result: None,
            },
            Example {
                example: "secret config show --file-path",
                description: "Show only the configuration file path",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let span = call.head;

        // Handle file-path flag
        if call.has_flag("file-path")? {
            let config_path = crate::config::get_config_file_path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "Configuration file path not available".to_string());

            return Ok(PipelineData::Value(Value::string(config_path, span), None));
        }

        // Get configuration from plugin
        let config_manager = plugin.config_manager().read().map_err(|e| {
            LabeledError::new("Configuration Error")
                .with_label(format!("Failed to access configuration: {}", e), span)
        })?;

        // Handle raw TOML flag
        if call.has_flag("raw")? {
            let toml_content = toml::to_string_pretty(config_manager.config()).map_err(|e| {
                LabeledError::new("Serialization Error")
                    .with_label(format!("Failed to serialize configuration: {}", e), span)
            })?;

            return Ok(PipelineData::Value(Value::string(toml_content, span), None));
        }

        // Create structured configuration display
        let mut record = Record::new();

        // General information
        record.push(
            "version",
            Value::string(config_manager.config().version.clone(), span),
        );

        // Configuration file path
        if let Some(config_path) = crate::config::get_config_file_path() {
            record.push(
                "config_file",
                Value::string(config_path.to_string_lossy().to_string(), span),
            );
        }

        // Redaction configuration
        let mut redaction_record = Record::new();
        redaction_record.push(
            "redaction_template",
            Value::string(
                config_manager.config().redaction.get_redaction_template(),
                span,
            ),
        );

        record.push("redaction", Value::record(redaction_record, span));

        // Security configuration
        let mut security_record = Record::new();
        security_record.push(
            "level",
            Value::string(
                format!("{:?}", config_manager.config().security.level).to_lowercase(),
                span,
            ),
        );
        security_record.push(
            "audit_config_changes",
            Value::bool(config_manager.config().security.audit_config_changes, span),
        );
        security_record.push(
            "max_custom_text_length",
            Value::int(
                config_manager.config().security.max_custom_text_length as i64,
                span,
            ),
        );

        record.push("security", Value::record(security_record, span));

        // Environment variable overrides status
        let env_overrides = std::env::vars()
            .filter(|(key, _)| key.starts_with("NU_PLUGIN_SECRET_"))
            .count();

        if env_overrides > 0 {
            record.push(
                "environment_overrides",
                Value::int(env_overrides as i64, span),
            );
        }

        Ok(PipelineData::Value(Value::record(record, span), None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretConfigShowCommand;
        assert_eq!(command.name(), "secret config show");
    }

    #[test]
    fn test_signature() {
        let command = SecretConfigShowCommand;
        let signature = command.signature();

        assert_eq!(signature.name, "secret config show");
    }
}
