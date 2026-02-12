//! Configuration management commands for nu_plugin_secret
//!
//! This module provides runtime configuration management commands including:
//! - `secret configure` - Interactive configuration changes
//! - `secret config show` - Display current settings  
//! - `secret config reset` - Restore defaults
//! - `secret config validate` - Validate configuration
//! - `secret config export/import` - Configuration backup/restore

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Record, Signature, SyntaxShape, Type, Value,
};

use crate::config::{ConfigManager, SecurityLevel};

/// Main configuration command that provides subcommands for config management
pub struct SecretConfigureCommand;

impl PluginCommand for SecretConfigureCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret configure"
    }

    fn description(&self) -> &str {
        "Configure secret plugin settings interactively"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::Record(Box::new([])))])
            .named(
                "security-level",
                SyntaxShape::String,
                "Set security level (minimal, standard, paranoid)",
                Some('s'),
            )
            .category(Category::Custom("secret".into()))
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            example: "secret configure --security-level paranoid",
            description: "Set security level to paranoid (maximum security)",
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
        let span = call.head;

        let mut config_changed = false;
        let mut new_config = None;

        // Handle security level changes
        if let Some(level_str) = call.get_flag::<String>("security-level")? {
            // Get current config
            let mut config = plugin
                .config_manager()
                .read()
                .map_err(|e| {
                    LabeledError::new("Configuration Error")
                        .with_label(format!("Failed to read configuration: {}", e), span)
                })?
                .config()
                .clone();

            let level = match level_str.as_str() {
                "minimal" => SecurityLevel::Minimal,
                "standard" => SecurityLevel::Standard,
                "paranoid" => SecurityLevel::Paranoid,
                _ => {
                    return Err(LabeledError::new("Invalid Security Level").with_label(
                        format!(
                            "Unknown level '{}'. Valid options: minimal, standard, paranoid",
                            level_str
                        ),
                        span,
                    ));
                }
            };
            config.security.level = level;
            new_config = Some(config);
            config_changed = true;
        }

        // Validate and apply configuration changes
        if config_changed {
            if let Some(config) = new_config {
                // Validate configuration
                if let Err(e) = ConfigManager::validate_config(&config) {
                    return Err(LabeledError::new("Configuration Validation Failed")
                        .with_label(format!("Invalid configuration: {}", e), span));
                }

                // Audit the change if enabled
                if let Ok(current_manager) = plugin.config_manager().read() {
                    if current_manager.config().security.audit_config_changes {
                        let _ =
                            crate::config::audit_config_change(current_manager.config(), &config);
                    }
                }

                // Update plugin configuration
                let mut config_manager = plugin.config_manager().write().map_err(|e| {
                    LabeledError::new("Configuration Error")
                        .with_label(format!("Failed to acquire write lock: {}", e), span)
                })?;

                *config_manager.config_mut() = config;

                // Save to disk
                config_manager.save().map_err(|e| {
                    LabeledError::new("Save Error")
                        .with_label(format!("Failed to save configuration: {}", e), span)
                })?;
            }
        }

        // Create summary record of current configuration
        let config_manager = plugin.config_manager().read().map_err(|e| {
            LabeledError::new("Configuration Error")
                .with_label(format!("Failed to read configuration: {}", e), span)
        })?;

        let mut record = Record::new();

        record.push(
            "redaction_template",
            Value::string(
                config_manager.config().redaction.get_redaction_template(),
                span,
            ),
        );

        record.push(
            "security_level",
            Value::string(
                format!("{:?}", config_manager.config().security.level).to_lowercase(),
                span,
            ),
        );

        if config_changed {
            record.push(
                "status",
                Value::string("Configuration updated successfully", span),
            );
        } else {
            record.push("status", Value::string("No changes made", span));
        }

        Ok(PipelineData::Value(Value::record(record, span), None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretConfigureCommand;
        assert_eq!(command.name(), "secret configure");
    }

    #[test]
    fn test_signature() {
        let command = SecretConfigureCommand;
        let signature = command.signature();

        assert_eq!(signature.name, "secret configure");
        assert!(signature.named.iter().any(|n| n.long == "security-level"));
    }
}
