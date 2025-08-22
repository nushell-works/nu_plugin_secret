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
            .switch(
                "enable-partial",
                "Enable partial redaction for string secrets",
                Some('p'),
            )
            .switch(
                "disable-partial",
                "Disable partial redaction for string secrets",
                None,
            )
            .named(
                "show-first",
                SyntaxShape::Int,
                "Number of characters to show from beginning (partial redaction)",
                None,
            )
            .named(
                "show-last",
                SyntaxShape::Int,
                "Number of characters to show from end (partial redaction)",
                None,
            )
            .switch(
                "use-hash",
                "Use hash-based partial redaction instead of character-based",
                None,
            )
            .named(
                "hash-salt",
                SyntaxShape::String,
                "Salt for hash-based partial redaction",
                None,
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
                example: "secret configure --enable-partial --show-first 3 --show-last 3",
                description: "Enable partial redaction showing first 3 and last 3 characters",
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

        // Handle partial redaction settings
        if call.has_flag("enable-partial")? {
            manager.config_mut().redaction.partial.enabled = true;
            config_changed = true;
        } else if call.has_flag("disable-partial")? {
            manager.config_mut().redaction.partial.enabled = false;
            config_changed = true;
        }

        if let Some(show_first) = call.get_flag::<i64>("show-first")? {
            if show_first < 0 {
                return Err(LabeledError::new("Invalid Parameter")
                    .with_label("show-first must be non-negative", span));
            }
            manager.config_mut().redaction.partial.show_first = show_first as usize;
            config_changed = true;
        }

        if let Some(show_last) = call.get_flag::<i64>("show-last")? {
            if show_last < 0 {
                return Err(LabeledError::new("Invalid Parameter")
                    .with_label("show-last must be non-negative", span));
            }
            manager.config_mut().redaction.partial.show_last = show_last as usize;
            config_changed = true;
        }

        if call.has_flag("use-hash")? {
            manager.config_mut().redaction.partial.use_hash = true;
            config_changed = true;
        }

        if let Some(salt) = call.get_flag::<String>("hash-salt")? {
            manager.config_mut().redaction.partial.hash_salt = salt;
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

        record.push(
            "partial_redaction_enabled",
            Value::bool(manager.config().redaction.partial.enabled, span),
        );

        if manager.config().redaction.partial.enabled {
            record.push(
                "show_first",
                Value::int(manager.config().redaction.partial.show_first as i64, span),
            );
            record.push(
                "show_last",
                Value::int(manager.config().redaction.partial.show_last as i64, span),
            );
            record.push(
                "use_hash",
                Value::bool(manager.config().redaction.partial.use_hash, span),
            );
        }

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
