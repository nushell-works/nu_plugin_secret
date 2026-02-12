//! Configuration import command for nu_plugin_secret

use std::path::PathBuf;

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Record, Signature, SyntaxShape, Type, Value,
};

use crate::config::ConfigManager;

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
        plugin: &Self::Plugin,
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

        // Audit the configuration change if enabled
        if let Ok(current_manager) = plugin.config_manager().read() {
            if current_manager.config().security.audit_config_changes {
                let _ = crate::config::audit_config_change(
                    current_manager.config(),
                    imported_manager.config(),
                );
            }
        }

        // Update plugin's configuration
        {
            let mut config_manager = plugin.config_manager().write().map_err(|e| {
                LabeledError::new("Update Error")
                    .with_label(format!("Failed to acquire write lock: {}", e), span)
            })?;

            *config_manager.config_mut() = imported_manager.config().clone();

            // Save to disk
            config_manager.save().map_err(|e| {
                LabeledError::new("Save Failed").with_label(
                    format!("Failed to save imported configuration: {}", e),
                    span,
                )
            })?;
        }

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
            "redaction_template",
            Value::string(
                imported_manager.config().redaction.get_redaction_template(),
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
    use crate::config::{PluginConfig, RedactionConfig, SecurityConfig, SecurityLevel};
    use std::fs;
    use tempfile::TempDir;

    // Note: Direct testing of the run method requires complex EngineInterface setup.
    // Instead, we test the command metadata and core logic separately.
    // Integration tests can be added in separate integration test files if needed.

    #[test]
    fn test_command_name() {
        let command = SecretConfigImportCommand;
        assert_eq!(command.name(), "secret config import");
    }

    #[test]
    fn test_description() {
        let command = SecretConfigImportCommand;
        assert_eq!(
            command.description(),
            "Import secret plugin configuration from a file"
        );
    }

    #[test]
    fn test_signature() {
        let command = SecretConfigImportCommand;
        let signature = command.signature();

        assert_eq!(signature.name, "secret config import");
        assert!(!signature.required_positional.is_empty());
        assert_eq!(signature.required_positional[0].name, "path");

        // Check flags
        let backup_flag = signature.get_long_flag("backup");
        assert!(backup_flag.is_some());
        assert_eq!(backup_flag.unwrap().short, Some('b'));

        let validate_flag = signature.get_long_flag("validate");
        assert!(validate_flag.is_some());
        assert_eq!(validate_flag.unwrap().short, Some('v'));
    }

    #[test]
    fn test_examples() {
        let command = SecretConfigImportCommand;
        let examples = command.examples();

        assert_eq!(examples.len(), 2);

        // Check first example
        assert_eq!(
            examples[0].example,
            "secret config import backup_config.toml"
        );
        assert!(!examples[0].description.is_empty());

        // Check second example
        assert_eq!(
            examples[1].example,
            "secret config import --backup --validate production_config.toml"
        );
        assert!(!examples[1].description.is_empty());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_config_manager_load_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let import_path = temp_dir.path().join("test_config.toml");

        // Create a valid test configuration
        let test_config = PluginConfig {
            redaction: RedactionConfig {
                show_unredacted: false,
                mask_secret: false,
                redaction_template: Some("[HIDDEN:{{secret_type}}]".to_string()),
            },
            security: SecurityConfig {
                level: SecurityLevel::Paranoid,
                audit_config_changes: true, // Paranoid level requires audit logging
                max_custom_text_length: 30,
            },
            version: "1.0".to_string(),
        };

        let toml_content = toml::to_string_pretty(&test_config).unwrap();
        fs::write(&import_path, toml_content).unwrap();

        // Test ConfigManager can load from the path
        let manager = ConfigManager::load_from_path(&import_path);
        assert!(manager.is_ok());

        let loaded_manager = manager.unwrap();
        let config = loaded_manager.config();

        // Verify the loaded configuration matches what we wrote
        assert_eq!(config.security.level, SecurityLevel::Paranoid);
        assert!(config.security.audit_config_changes);
        assert_eq!(config.security.max_custom_text_length, 30);
        assert_eq!(
            config.redaction.redaction_template,
            Some("[HIDDEN:{{secret_type}}]".to_string())
        );
        assert!(!config.redaction.show_unredacted);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_config_manager_load_nonexistent_file() {
        let nonexistent_path = std::path::Path::new("/nonexistent/file.toml");
        let result = ConfigManager::load_from_path(nonexistent_path);

        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_config_manager_load_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("invalid.toml");

        // Write invalid TOML content
        fs::write(&invalid_path, "invalid toml content [[[").unwrap();

        let result = ConfigManager::load_from_path(&invalid_path);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_different_security_levels_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let security_levels = [
            SecurityLevel::Minimal,
            SecurityLevel::Standard,
            SecurityLevel::Paranoid,
        ];

        for level in security_levels {
            let config_path = temp_dir.path().join(format!("test_{:?}.toml", level));

            let test_config = PluginConfig {
                security: SecurityConfig {
                    level: level.clone(),
                    ..Default::default()
                },
                ..Default::default()
            };

            // Test serialization
            let toml_content = toml::to_string_pretty(&test_config).unwrap();
            fs::write(&config_path, toml_content).unwrap();

            // Test deserialization
            let manager = ConfigManager::load_from_path(&config_path);
            assert!(
                manager.is_ok(),
                "Failed to load config with security level {:?}",
                level
            );

            let loaded_manager = manager.unwrap();
            let loaded_config = loaded_manager.config();
            assert_eq!(loaded_config.security.level, level);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_custom_redaction_templates_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let templates = [
            Some("[SECRET:{{secret_type}}]".to_string()),
            Some("***{{secret_type}}***".to_string()),
            Some("REDACTED".to_string()), // Simple static template
            None,                         // Default template
        ];

        for (idx, template) in templates.iter().enumerate() {
            let config_path = temp_dir.path().join(format!("test_template_{}.toml", idx));

            let test_config = PluginConfig {
                redaction: RedactionConfig {
                    redaction_template: template.clone(),
                    ..Default::default()
                },
                ..Default::default()
            };

            // Test serialization
            let toml_content = toml::to_string_pretty(&test_config).unwrap();
            fs::write(&config_path, toml_content).unwrap();

            // Test deserialization
            let manager = ConfigManager::load_from_path(&config_path);
            assert!(
                manager.is_ok(),
                "Failed to load config with template {:?}",
                template
            );

            let loaded_manager = manager.unwrap();
            let loaded_config = loaded_manager.config();
            assert_eq!(loaded_config.redaction.redaction_template, *template);

            // Test the get_redaction_template method
            let expected = template.as_deref().unwrap_or("<redacted:{{secret_type}}>");
            assert_eq!(loaded_config.redaction.get_redaction_template(), expected);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_config_validation_logic() {
        // Test that ConfigManager::validate_config works correctly

        // Valid config should pass
        let valid_config = PluginConfig::default();
        let result = ConfigManager::validate_config(&valid_config);
        assert!(result.is_ok());

        // Test edge cases for custom text length
        let config_with_max_length = PluginConfig {
            security: SecurityConfig {
                max_custom_text_length: 100,
                ..Default::default()
            },
            ..Default::default()
        };
        let result = ConfigManager::validate_config(&config_with_max_length);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_backup_filename_format() {
        // Test that backup filenames follow expected format
        let backup_timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("config_backup_before_import_{}.toml", backup_timestamp);

        // Verify format
        assert!(backup_filename.starts_with("config_backup_before_import_"));
        assert!(backup_filename.ends_with(".toml"));
        assert!(backup_filename.len() > 30); // Should be a reasonable length

        // Verify timestamp is in correct format (YYYYMMDD_HHMMSS)
        let timestamp_part = backup_filename
            .strip_prefix("config_backup_before_import_")
            .unwrap()
            .strip_suffix(".toml")
            .unwrap();

        assert_eq!(timestamp_part.len(), 15); // YYYYMMDD_HHMMSS format
        assert!(timestamp_part.chars().nth(8) == Some('_')); // Underscore separator
    }

    #[test]
    fn test_configuration_structure_completeness() {
        // Test that PluginConfig covers all expected fields
        let config = PluginConfig::default();

        // Verify redaction config structure
        assert!(!config.redaction.show_unredacted);
        assert!(config.redaction.redaction_template.is_none());

        // Verify security config structure
        assert_eq!(config.security.level, SecurityLevel::Standard);
        assert!(config.security.audit_config_changes);
        assert_eq!(config.security.max_custom_text_length, 50);

        // Verify version
        assert_eq!(config.version, "1.0");
    }
}
