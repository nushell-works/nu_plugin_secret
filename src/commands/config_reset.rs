//! Configuration reset command for nu_plugin_secret

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Record, Signature, Type, Value};

use crate::config::PluginConfig;

/// Command to reset configuration to defaults
pub struct SecretConfigResetCommand;

/// Create a backup of the current configuration before resetting.
///
/// Returns the backup file path on success, or a warning message if the
/// current config could not be read.
fn create_backup(
    plugin: &crate::SecretPlugin,
    span: nu_protocol::Span,
) -> Result<Option<String>, LabeledError> {
    if let Ok(current_manager) = plugin.config_manager().read() {
        let backup_timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("config_backup_{}.toml", backup_timestamp);

        if let Some(config_dir) =
            crate::config::get_config_file_path().and_then(|p| p.parent().map(|p| p.to_path_buf()))
        {
            let backup_path = config_dir.join(&backup_filename);

            current_manager.save_to_path(&backup_path).map_err(|e| {
                LabeledError::new("Backup Failed")
                    .with_label(format!("Failed to create backup: {}", e), span)
            })?;

            Ok(Some(backup_path.to_string_lossy().to_string()))
        } else {
            Err(LabeledError::new("Backup Failed")
                .with_label("Cannot determine config directory for backup", span))
        }
    } else {
        // If we can't access current config, return None to indicate a warning
        Ok(None)
    }
}

/// Build the result record for a successful reset, including default settings.
fn build_reset_result(backup_info: &Record, span: nu_protocol::Span) -> Record {
    let mut record = backup_info.clone();

    record.push(
        "status",
        Value::string("Configuration reset to defaults", span),
    );
    record.push(
        "config_file",
        Value::string(
            crate::config::get_config_file_path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            span,
        ),
    );

    let mut defaults_record = Record::new();
    defaults_record.push("redaction_style", Value::string("typed_brackets", span));
    defaults_record.push("security_level", Value::string("standard", span));
    defaults_record.push("show_type_info", Value::bool(true, span));
    defaults_record.push("preserve_length", Value::bool(false, span));

    record.push("default_settings", Value::record(defaults_record, span));

    record
}

impl PluginCommand for SecretConfigResetCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret config reset"
    }

    fn description(&self) -> &str {
        "Reset secret plugin configuration to default settings"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::Record(Box::new([])))])
            .switch(
                "confirm",
                "Confirm the reset operation (required for safety)",
                Some('c'),
            )
            .switch(
                "backup",
                "Create backup of current configuration before reset",
                Some('b'),
            )
            .category(Category::Custom("secret".into()))
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "secret config reset --confirm",
                description: "Reset configuration to defaults",
                result: None,
            },
            Example {
                example: "secret config reset --confirm --backup",
                description: "Reset configuration with backup of current settings",
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

        // Require confirmation for safety
        if !call.has_flag("confirm")? {
            return Err(LabeledError::new("Confirmation Required").with_label(
                "This operation will reset all configuration to defaults. Use --confirm to proceed",
                span,
            ));
        }

        let mut backup_record = Record::new();

        // Create backup if requested
        if call.has_flag("backup")? {
            match create_backup(plugin, span)? {
                Some(backup_path) => {
                    backup_record.push("backup_created", Value::string(backup_path, span));
                }
                None => {
                    // If we can't access current config, warn but continue with reset
                    backup_record.push(
                        "backup_warning",
                        Value::string("Could not backup current config: lock error", span),
                    );
                }
            }
        }

        // Create default configuration
        let default_config = PluginConfig::default();

        // Audit the configuration change if enabled
        if let Ok(current_manager) = plugin.config_manager().read() {
            if current_manager.config().security.audit_config_changes {
                let _ =
                    crate::config::audit_config_change(current_manager.config(), &default_config);
            }
        }

        // Update plugin's configuration
        if let Ok(mut config_manager) = plugin.config_manager().write() {
            *config_manager.config_mut() = default_config;

            // Save to disk
            config_manager.save().map_err(|e| {
                LabeledError::new("Reset Failed")
                    .with_label(format!("Failed to save default configuration: {}", e), span)
            })?;
        } else {
            return Err(LabeledError::new("Update Error")
                .with_label("Failed to acquire write lock on configuration", span));
        }

        let result_record = build_reset_result(&backup_record, span);

        Ok(PipelineData::Value(
            Value::record(result_record, span),
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretConfigResetCommand;
        assert_eq!(command.name(), "secret config reset");
    }

    #[test]
    fn test_signature() {
        let command = SecretConfigResetCommand;
        let signature = command.signature();

        assert_eq!(signature.name, "secret config reset");
    }
}
