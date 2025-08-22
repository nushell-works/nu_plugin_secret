//! Configuration display command for nu_plugin_secret

use crate::config::ConfigManager;
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

    fn examples(&self) -> Vec<Example> {
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
        _plugin: &Self::Plugin,
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

        // Load current configuration
        let manager = ConfigManager::load().map_err(|e| {
            LabeledError::new("Configuration Error")
                .with_label(format!("Failed to load configuration: {}", e), span)
        })?;

        // Handle raw TOML flag
        if call.has_flag("raw")? {
            let toml_content = toml::to_string_pretty(manager.config()).map_err(|e| {
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
            Value::string(manager.config().version.clone(), span),
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
            "style",
            Value::string(
                format!("{:?}", manager.config().redaction.style).to_lowercase(),
                span,
            ),
        );
        redaction_record.push(
            "show_type_info",
            Value::bool(manager.config().redaction.show_type_info, span),
        );
        redaction_record.push(
            "preserve_length",
            Value::bool(manager.config().redaction.preserve_length, span),
        );

        // Custom text if applicable
        if let Some(ref custom_text) = manager.config().redaction.custom_text {
            redaction_record.push("custom_text", Value::string(custom_text.clone(), span));
        }

        // Per-type overrides
        if !manager.config().redaction.per_type.is_empty() {
            let mut per_type_record = Record::new();
            for (type_name, style) in &manager.config().redaction.per_type {
                per_type_record.push(
                    type_name.clone(),
                    Value::string(format!("{:?}", style).to_lowercase(), span),
                );
            }
            redaction_record.push("per_type_overrides", Value::record(per_type_record, span));
        }

        // Per-context overrides
        if !manager.config().redaction.per_context.is_empty() {
            let mut per_context_record = Record::new();
            for (context, style) in &manager.config().redaction.per_context {
                per_context_record.push(
                    format!("{:?}", context).to_lowercase(),
                    Value::string(format!("{:?}", style).to_lowercase(), span),
                );
            }
            redaction_record.push(
                "per_context_overrides",
                Value::record(per_context_record, span),
            );
        }

        // Partial redaction configuration
        let partial = &manager.config().redaction.partial;
        let mut partial_record = Record::new();
        partial_record.push("enabled", Value::bool(partial.enabled, span));
        partial_record.push("show_first", Value::int(partial.show_first as i64, span));
        partial_record.push("show_last", Value::int(partial.show_last as i64, span));
        partial_record.push("min_length", Value::int(partial.min_length as i64, span));
        partial_record.push("max_reveal", Value::int(partial.max_reveal as i64, span));
        partial_record.push("use_hash", Value::bool(partial.use_hash, span));

        // Only show salt info, not the actual salt for security
        partial_record.push(
            "hash_salt_configured",
            Value::bool(!partial.hash_salt.is_empty(), span),
        );

        redaction_record.push("partial", Value::record(partial_record, span));
        record.push("redaction", Value::record(redaction_record, span));

        // Security configuration
        let mut security_record = Record::new();
        security_record.push(
            "level",
            Value::string(
                format!("{:?}", manager.config().security.level).to_lowercase(),
                span,
            ),
        );
        security_record.push(
            "audit_config_changes",
            Value::bool(manager.config().security.audit_config_changes, span),
        );
        security_record.push(
            "allow_partial_redaction",
            Value::bool(manager.config().security.allow_partial_redaction, span),
        );
        security_record.push(
            "max_custom_text_length",
            Value::int(
                manager.config().security.max_custom_text_length as i64,
                span,
            ),
        );
        security_record.push(
            "min_partial_redaction_length",
            Value::int(
                manager.config().security.min_partial_redaction_length as i64,
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
