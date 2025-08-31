//! Configuration management commands for nu_plugin_secret
//!
//! This module provides runtime configuration management commands including:
//! - `secret configure` - Interactive configuration changes
//! - `secret config show` - Display current settings  
//! - `secret config reset` - Restore defaults
//! - `secret config validate` - Validate configuration
//! - `secret config export/import` - Configuration backup/restore

use crate::config::{ConfigManager, RedactionStyle, SecurityLevel};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Record, Signature, SyntaxShape, Type, Value,
};

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
                "redaction-style",
                SyntaxShape::String,
                "Set global redaction style (typed_brackets, simple, asterisks, brackets, custom)",
                Some('r'),
            )
            .named(
                "custom-text",
                SyntaxShape::String,
                "Custom redaction text (when style is 'custom')",
                Some('c'),
            )
            .named(
                "security-level",
                SyntaxShape::String,
                "Set security level (minimal, standard, paranoid)",
                Some('s'),
            )
            .category(Category::Custom("secret".into()))
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "secret configure --redaction-style simple",
                description: "Set redaction style to simple '<redacted>'",
                result: None,
            },
            Example {
                example: "secret configure --redaction-style custom --custom-text '[HIDDEN]'",
                description: "Set custom redaction text",
                result: None,
            },
            Example {
                example: "secret configure --security-level paranoid",
                description: "Set security level to paranoid (maximum security)",
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

        // Load current configuration or create default
        let mut manager = ConfigManager::load().map_err(|e| {
            LabeledError::new("Configuration Error")
                .with_label(format!("Failed to load configuration: {}", e), span)
        })?;

        let mut config_changed = false;

        // Handle redaction style changes
        if let Some(style_str) = call.get_flag::<String>("redaction-style")? {
            let style = match style_str.as_str() {
                "typed_brackets" => RedactionStyle::TypedBrackets,
                "simple" => RedactionStyle::Simple,
                "asterisks" => RedactionStyle::Asterisks,
                "brackets" => RedactionStyle::Brackets,
                "custom" => {
                    if let Some(custom_text) = call.get_flag::<String>("custom-text")? {
                        RedactionStyle::Custom(custom_text)
                    } else {
                        return Err(LabeledError::new("Invalid Configuration")
                            .with_label("Custom redaction style requires --custom-text", span));
                    }
                }
                _ => {
                    return Err(LabeledError::new("Invalid Redaction Style")
                        .with_label(format!("Unknown style '{}'. Valid options: typed_brackets, simple, asterisks, brackets, custom", style_str), span));
                }
            };
            manager.config_mut().redaction.style = style;
            config_changed = true;
        }

        // Handle security level changes
        if let Some(level_str) = call.get_flag::<String>("security-level")? {
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
            manager.config_mut().security.level = level;
            config_changed = true;
        }

        // Validate configuration if changes were made
        if config_changed {
            if let Err(e) = ConfigManager::validate_config(manager.config()) {
                return Err(LabeledError::new("Configuration Validation Failed")
                    .with_label(format!("Invalid configuration: {}", e), span));
            }

            // Save configuration
            manager.save().map_err(|e| {
                LabeledError::new("Save Error")
                    .with_label(format!("Failed to save configuration: {}", e), span)
            })?;

            // Update global configuration
            crate::config::update_config(manager.config().clone()).map_err(|e| {
                LabeledError::new("Update Error").with_label(
                    format!("Failed to update runtime configuration: {}", e),
                    span,
                )
            })?;
        }

        // Create summary record of current configuration
        let mut record = Record::new();

        record.push(
            "redaction_style",
            Value::string(
                format!("{:?}", manager.config().redaction.style).to_lowercase(),
                span,
            ),
        );

        record.push(
            "security_level",
            Value::string(
                format!("{:?}", manager.config().security.level).to_lowercase(),
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
        assert!(signature.named.iter().any(|n| n.long == "redaction-style"));
        assert!(signature.named.iter().any(|n| n.long == "security-level"));
    }
}
