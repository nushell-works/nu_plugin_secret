//! Configuration validation command for nu_plugin_secret

use crate::config::ConfigManager;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Record, Signature, Type, Value};

/// Command to validate configuration settings
pub struct SecretConfigValidateCommand;

impl PluginCommand for SecretConfigValidateCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret config validate"
    }

    fn description(&self) -> &str {
        "Validate secret plugin configuration settings"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::Record(Box::new([])))])
            .switch("verbose", "Show detailed validation results", Some('v'))
            .category(Category::Custom("secret".into()))
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "secret config validate",
                description: "Validate current configuration",
                result: None,
            },
            Example {
                example: "secret config validate --verbose",
                description: "Show detailed validation results",
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
        let verbose = call.has_flag("verbose")?;

        // Load and validate configuration
        let manager = match ConfigManager::load() {
            Ok(manager) => manager,
            Err(e) => {
                return Err(LabeledError::new("Configuration Load Error")
                    .with_label(format!("Failed to load configuration: {}", e), span));
            }
        };

        // Validate configuration
        let mut validation_results = Vec::new();
        let mut has_errors = false;
        let mut has_warnings = false;

        // Validate using the manager's validation method
        match ConfigManager::validate_config(manager.config()) {
            Ok(()) => {
                validation_results.push((
                    "Configuration",
                    "Valid",
                    "Configuration passed all validation checks",
                ));
            }
            Err(e) => {
                let error_msg = format!("Validation failed: {}", e);
                validation_results.push((
                    "Configuration",
                    "Error",
                    Box::leak(error_msg.into_boxed_str()),
                ));
                has_errors = true;
            }
        }

        // Additional detailed validations
        let config = manager.config();

        // Validate redaction style
        match &config.redaction.style {
            crate::config::RedactionStyle::Custom(text) => {
                if text.is_empty() {
                    validation_results.push((
                        "Redaction Style",
                        "Error",
                        "Custom redaction text cannot be empty",
                    ));
                    has_errors = true;
                } else if text.len() > 50 {
                    validation_results.push((
                        "Redaction Style",
                        "Warning",
                        "Custom redaction text is very long (>50 chars)",
                    ));
                    has_warnings = true;
                } else {
                    validation_results.push((
                        "Redaction Style",
                        "Valid",
                        "Custom redaction text is acceptable",
                    ));
                }
            }
            _ => {
                validation_results.push((
                    "Redaction Style",
                    "Valid",
                    "Using built-in redaction style",
                ));
            }
        }

        // Validate partial redaction settings
        let partial = &config.redaction.partial;
        if partial.enabled {
            if partial.show_first + partial.show_last >= partial.min_length {
                validation_results.push(("Partial Redaction", "Warning", 
                    "show_first + show_last should be less than min_length to ensure some content is redacted"));
                has_warnings = true;
            }

            if partial.show_first + partial.show_last > partial.max_reveal {
                validation_results.push((
                    "Partial Redaction",
                    "Error",
                    "show_first + show_last exceeds max_reveal limit",
                ));
                has_errors = true;
            }

            if partial.use_hash && partial.hash_salt.is_empty() {
                validation_results.push((
                    "Partial Redaction",
                    "Warning",
                    "Hash-based partial redaction enabled but no salt configured",
                ));
                has_warnings = true;
            }

            if !has_errors && !has_warnings {
                validation_results.push((
                    "Partial Redaction",
                    "Valid",
                    "Partial redaction settings are valid",
                ));
            }
        } else {
            validation_results.push(("Partial Redaction", "Info", "Partial redaction is disabled"));
        }

        // Validate security settings
        match config.security.level {
            crate::config::SecurityLevel::Minimal => {
                validation_results.push((
                    "Security Level",
                    "Warning",
                    "Minimal security level provides basic protection only",
                ));
                has_warnings = true;
            }
            crate::config::SecurityLevel::Standard => {
                validation_results.push((
                    "Security Level",
                    "Valid",
                    "Standard security level is recommended",
                ));
            }
            crate::config::SecurityLevel::Paranoid => {
                validation_results.push((
                    "Security Level",
                    "Valid",
                    "Paranoid security level provides maximum protection",
                ));
            }
        }

        // Check environment overrides
        let env_overrides: Vec<_> = std::env::vars()
            .filter(|(key, _)| key.starts_with("NU_PLUGIN_SECRET_"))
            .collect();

        if !env_overrides.is_empty() {
            let env_msg = format!(
                "Found {} environment variable overrides",
                env_overrides.len()
            );
            validation_results.push((
                "Environment Overrides",
                "Info",
                Box::leak(env_msg.into_boxed_str()),
            ));
        }

        // Create result record
        let mut record = Record::new();

        // Overall status
        let status = if has_errors {
            "INVALID"
        } else if has_warnings {
            "VALID_WITH_WARNINGS"
        } else {
            "VALID"
        };

        record.push("status", Value::string(status, span));
        record.push("has_errors", Value::bool(has_errors, span));
        record.push("has_warnings", Value::bool(has_warnings, span));

        // Summary counts
        let error_count = validation_results
            .iter()
            .filter(|(_, level, _)| level == &"Error")
            .count();
        let warning_count = validation_results
            .iter()
            .filter(|(_, level, _)| level == &"Warning")
            .count();

        record.push("error_count", Value::int(error_count as i64, span));
        record.push("warning_count", Value::int(warning_count as i64, span));

        // Add detailed results if verbose or if there are issues
        if verbose || has_errors || has_warnings {
            let mut details = Vec::new();

            for (category, level, message) in validation_results {
                let mut detail_record = Record::new();
                detail_record.push("category", Value::string(category, span));
                detail_record.push("level", Value::string(level, span));
                detail_record.push("message", Value::string(message, span));
                details.push(Value::record(detail_record, span));
            }

            record.push("details", Value::list(details, span));
        }

        // Configuration file path
        if let Some(config_path) = crate::config::get_config_file_path() {
            record.push(
                "config_file",
                Value::string(config_path.to_string_lossy().to_string(), span),
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
        let command = SecretConfigValidateCommand;
        assert_eq!(command.name(), "secret config validate");
    }

    #[test]
    fn test_signature() {
        let command = SecretConfigValidateCommand;
        let signature = command.signature();

        assert_eq!(signature.name, "secret config validate");
    }
}
