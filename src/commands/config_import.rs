//! Configuration import command for nu_plugin_secret

use crate::config::ConfigManager;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Record, Signature, SyntaxShape, Type, Value,
};
use std::path::PathBuf;

/// Command to import configuration from a file
pub struct SecretConfigImportCommand;

impl PluginCommand for SecretConfigImportCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret config import"
    }

    fn description(&self) -> &str {
        "Import secret plugin configuration from a file"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::Record(Box::new([])))])
            .required(
                "path",
                SyntaxShape::Filepath,
                "Path to import configuration file",
            )
            .switch(
                "backup",
                "Create backup of current configuration before import",
                Some('b'),
            )
            .switch(
                "validate",
                "Validate imported configuration before applying",
                Some('v'),
            )
            .category(Category::Custom("secret".into()))
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "secret config import backup_config.toml",
                description: "Import configuration from backup_config.toml",
                result: None,
            },
            Example {
                example: "secret config import --backup --validate production_config.toml",
                description: "Import configuration with backup and validation",
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

        // Get the import path
        let import_path: String = call.req(0)?;
        let import_path = PathBuf::from(&import_path);

        // Check if import file exists
        if !import_path.exists() {
            return Err(
                LabeledError::new("File Not Found").with_label("Import file does not exist", span)
            );
        }

        let mut result_record = Record::new();

        // Create backup if requested
        if call.has_flag("backup")? {
            match ConfigManager::load() {
                Ok(current_manager) => {
                    let backup_timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                    let backup_filename =
                        format!("config_backup_before_import_{}.toml", backup_timestamp);

                    if let Some(config_dir) = crate::config::get_config_file_path()
                        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                    {
                        let backup_path = config_dir.join(&backup_filename);

                        match current_manager.save_to_path(&backup_path) {
                            Ok(()) => {
                                result_record.push(
                                    "backup_created",
                                    Value::string(backup_path.to_string_lossy().to_string(), span),
                                );
                            }
                            Err(e) => {
                                return Err(LabeledError::new("Backup Failed")
                                    .with_label(format!("Failed to create backup: {}", e), span));
                            }
                        }
                    }
                }
                Err(_) => {
                    // If no current config exists, that's okay for import
                    result_record.push(
                        "backup_note",
                        Value::string("No existing configuration to backup", span),
                    );
                }
            }
        }

        // Load configuration from import file
        let imported_manager = ConfigManager::load_from_path(&import_path).map_err(|e| {
            LabeledError::new("Import Failed").with_label(
                format!("Failed to load configuration from import file: {}", e),
                span,
            )
        })?;

        // Validate imported configuration if requested
        if call.has_flag("validate")? {
            if let Err(e) = ConfigManager::validate_config(imported_manager.config()) {
                return Err(LabeledError::new("Validation Failed")
                    .with_label(format!("Imported configuration is invalid: {}", e), span));
            }
            result_record.push(
                "validation",
                Value::string("Imported configuration passed validation", span),
            );
        }

        // Save imported configuration as the active configuration
        imported_manager.save().map_err(|e| {
            LabeledError::new("Save Failed").with_label(
                format!("Failed to save imported configuration: {}", e),
                span,
            )
        })?;

        // Update global configuration
        crate::config::update_config(imported_manager.config().clone()).map_err(|e| {
            LabeledError::new("Update Error").with_label(
                format!("Failed to update runtime configuration: {}", e),
                span,
            )
        })?;

        result_record.push(
            "status",
            Value::string("Configuration imported successfully", span),
        );
        result_record.push(
            "import_path",
            Value::string(import_path.to_string_lossy().to_string(), span),
        );

        // Add active configuration file path
        if let Some(config_path) = crate::config::get_config_file_path() {
            result_record.push(
                "active_config",
                Value::string(config_path.to_string_lossy().to_string(), span),
            );
        }

        // Add configuration summary
        result_record.push(
            "redaction_style",
            Value::string(
                format!("{:?}", imported_manager.config().redaction.style).to_lowercase(),
                span,
            ),
        );
        result_record.push(
            "security_level",
            Value::string(
                format!("{:?}", imported_manager.config().security.level).to_lowercase(),
                span,
            ),
        );

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
        let command = SecretConfigImportCommand;
        assert_eq!(command.name(), "secret config import");
    }

    #[test]
    fn test_signature() {
        let command = SecretConfigImportCommand;
        let signature = command.signature();

        assert_eq!(signature.name, "secret config import");
        assert!(!signature.required_positional.is_empty());
        assert_eq!(signature.required_positional[0].name, "path");
    }
}
