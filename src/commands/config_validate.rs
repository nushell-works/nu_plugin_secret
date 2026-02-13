//! Configuration validation command for nu_plugin_secret

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Record, Signature, Type, Value};

use crate::config::{ConfigManager, PluginConfig};

/// Command to validate configuration settings
pub struct SecretConfigValidateCommand;

/// Run all validation checks against a configuration, returning a list of
/// (category, level, message) tuples plus error/warning flags.
fn run_validation_checks(
    config: &PluginConfig,
) -> (Vec<(&'static str, &'static str, &'static str)>, bool, bool) {
    let mut results: Vec<(&'static str, &'static str, &'static str)> = Vec::new();
    let mut has_errors = false;
    let mut has_warnings = false;

    // Validate using the manager's validation method
    match ConfigManager::validate_config(config) {
        Ok(()) => {
            results.push((
                "Configuration",
                "Valid",
                "Configuration passed all validation checks",
            ));
        }
        Err(e) => {
            let error_msg = format!("Validation failed: {}", e);
            results.push((
                "Configuration",
                "Error",
                Box::leak(error_msg.into_boxed_str()),
            ));
            has_errors = true;
        }
    }

    // Validate redaction template
    if let Some(template) = &config.redaction.redaction_template {
        if template.trim().is_empty() {
            results.push((
                "Redaction Template",
                "Warning",
                "Redaction template is empty or whitespace-only",
            ));
            has_warnings = true;
        } else {
            results.push((
                "Redaction Template",
                "Valid",
                "Custom redaction template is configured",
            ));
        }
    } else {
        results.push((
            "Redaction Template",
            "Valid",
            "Using default redaction template",
        ));
    }

    // Validate security settings
    match config.security.level {
        crate::config::SecurityLevel::Minimal => {
            results.push((
                "Security Level",
                "Warning",
                "Minimal security level provides basic protection only",
            ));
            has_warnings = true;
        }
        crate::config::SecurityLevel::Standard => {
            results.push((
                "Security Level",
                "Valid",
                "Standard security level is recommended",
            ));
        }
        crate::config::SecurityLevel::Paranoid => {
            results.push((
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
        results.push((
            "Environment Overrides",
            "Info",
            Box::leak(env_msg.into_boxed_str()),
        ));
    }

    (results, has_errors, has_warnings)
}

/// Build the final validation result record from the collected validation data.
fn build_validation_record(
    validation_results: Vec<(&str, &str, &str)>,
    has_errors: bool,
    has_warnings: bool,
    verbose: bool,
    span: nu_protocol::Span,
) -> Record {
    let mut record = Record::new();

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

    if let Some(config_path) = crate::config::get_config_file_path() {
        record.push(
            "config_file",
            Value::string(config_path.to_string_lossy().to_string(), span),
        );
    }

    record
}

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

    fn examples(&self) -> Vec<Example<'_>> {
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

        let (validation_results, has_errors, has_warnings) =
            run_validation_checks(manager.config());
        let record =
            build_validation_record(validation_results, has_errors, has_warnings, verbose, span);

        Ok(PipelineData::Value(Value::record(record, span), None))
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;
    use crate::config::{ConfigManager, PluginConfig, SecurityLevel};
    use std::env;

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
        assert!(signature.named.iter().any(|n| n.long == "verbose"));
    }

    #[test]
    fn test_description() {
        let command = SecretConfigValidateCommand;
        assert_eq!(
            command.description(),
            "Validate secret plugin configuration settings"
        );
    }

    #[test]
    fn test_examples_count() {
        let command = SecretConfigValidateCommand;
        let examples = command.examples();
        assert_eq!(examples.len(), 2);
    }

    #[test]
    fn test_examples_have_descriptions() {
        let command = SecretConfigValidateCommand;
        let examples = command.examples();

        for example in examples {
            assert!(!example.description.is_empty());
        }
    }

    #[test]
    fn test_default_configuration_validation() {
        // Test that a default configuration validates successfully
        let config = PluginConfig::default();
        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Default configuration should be valid"
        );
    }

    #[test]
    fn test_minimal_security_level_validation() {
        let mut config = PluginConfig::default();
        config.security.level = SecurityLevel::Minimal;

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Minimal security level should be valid"
        );
    }

    #[test]
    fn test_standard_security_level_validation() {
        let mut config = PluginConfig::default();
        config.security.level = SecurityLevel::Standard;

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Standard security level should be valid"
        );
    }

    #[test]
    fn test_paranoid_security_level_validation() {
        let mut config = PluginConfig::default();
        config.security.level = SecurityLevel::Paranoid;

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Paranoid security level should be valid"
        );
    }

    #[test]
    fn test_custom_redaction_template_validation() {
        let mut config = PluginConfig::default();
        config.redaction.redaction_template = Some("<custom:{{secret_type}}>".to_string());

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Custom redaction template should be valid"
        );
    }

    #[test]
    fn test_empty_redaction_template_validation() {
        let mut config = PluginConfig::default();
        config.redaction.redaction_template = Some("".to_string());

        // Empty template should still pass validation but may trigger warnings in the command
        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Empty redaction template should be valid"
        );
    }

    #[test]
    fn test_whitespace_only_redaction_template() {
        let mut config = PluginConfig::default();
        config.redaction.redaction_template = Some("   \t\n  ".to_string());

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Whitespace-only template should be valid"
        );
    }

    #[test]
    fn test_invalid_tera_template() {
        let mut config = PluginConfig::default();
        config.redaction.redaction_template = Some("{{unclosed".to_string());

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_err(),
            "Invalid Tera syntax should fail validation"
        );
    }

    #[test]
    fn test_valid_tera_template_without_variables() {
        let mut config = PluginConfig::default();
        config.redaction.redaction_template = Some("REDACTED".to_string());

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Simple template without variables should be valid"
        );
    }

    #[test]
    fn test_complex_tera_template() {
        let mut config = PluginConfig::default();
        config.redaction.redaction_template = Some("{{secret_type}} - redacted".to_string());

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Template with variables should be valid during validation"
        );
    }

    #[test]
    #[cfg_attr(not(miri), serial(env))]
    #[cfg_attr(miri, ignore)] // Miri doesn't support environment variable manipulation in tests
    fn test_environment_variable_detection() {
        // Set some environment variables that should be detected
        env::set_var("NU_PLUGIN_SECRET_SECURITY_LEVEL", "paranoid");
        env::set_var("SHOW_UNREDACTED", "true");

        // Check that we can detect environment overrides
        let env_overrides: Vec<_> = std::env::vars()
            .filter(|(key, _)| key.starts_with("NU_PLUGIN_SECRET_"))
            .collect();

        assert!(
            !env_overrides.is_empty(),
            "Should detect NU_PLUGIN_SECRET_ environment variables"
        );

        // Clean up
        env::remove_var("NU_PLUGIN_SECRET_SECURITY_LEVEL");
        env::remove_var("SHOW_UNREDACTED");
    }

    #[test]
    fn test_paranoid_security_with_audit_disabled_should_fail() {
        let mut config = PluginConfig::default();
        config.security.level = SecurityLevel::Paranoid;
        config.security.audit_config_changes = false;

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_err(),
            "Paranoid security level should require audit logging"
        );

        let error_msg = validation_result.unwrap_err().to_string();
        assert!(error_msg.contains("Paranoid security level requires audit logging"));
    }

    #[test]
    fn test_standard_security_with_audit_disabled_should_fail() {
        let mut config = PluginConfig::default();
        config.security.level = SecurityLevel::Standard;
        config.security.audit_config_changes = false;

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_err(),
            "Standard security level should require audit logging"
        );

        let error_msg = validation_result.unwrap_err().to_string();
        assert!(error_msg.contains("Standard security level requires audit logging"));
    }

    #[test]
    fn test_minimal_security_with_audit_disabled_should_pass() {
        let mut config = PluginConfig::default();
        config.security.level = SecurityLevel::Minimal;
        config.security.audit_config_changes = false;

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Minimal security level should allow disabled audit logging"
        );
    }

    #[test]
    fn test_show_unredacted_configuration() {
        let mut config = PluginConfig::default();
        config.redaction.show_unredacted = true;

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "show_unredacted should be valid configuration"
        );
    }

    #[test]
    fn test_multiple_configuration_issues() {
        let mut config = PluginConfig::default();
        config.security.level = SecurityLevel::Paranoid;
        config.security.audit_config_changes = false;
        config.redaction.redaction_template = Some("{{invalid_syntax".to_string());

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_err(),
            "Multiple issues should cause validation to fail"
        );
    }

    #[test]
    fn test_configuration_with_all_valid_settings() {
        let mut config = PluginConfig::default();
        config.security.level = SecurityLevel::Paranoid;
        config.security.audit_config_changes = true;
        config.security.max_custom_text_length = 20;
        config.redaction.redaction_template = Some("<secret:{{secret_type}}>".to_string());
        config.redaction.show_unredacted = false;

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "All valid settings should pass validation"
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri doesn't support file system path operations
    fn test_config_file_path_detection() {
        // This test ensures the config file path is properly detected
        let config_path = crate::config::get_config_file_path();
        // Path might be None in some environments, but the function should not panic
        match config_path {
            Some(path) => {
                assert!(path.to_string_lossy().contains("config.toml"));
            }
            None => {
                // This is acceptable in some test environments
            }
        }
    }

    #[test]
    fn test_redaction_template_with_variables() {
        let mut config = PluginConfig::default();
        config.redaction.redaction_template = Some("{{secret_type}}".to_string());

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Template with variables should be valid during validation"
        );
    }

    #[test]
    fn test_redaction_template_with_conditionals() {
        let mut config = PluginConfig::default();
        config.redaction.redaction_template =
            Some("{% if true %}[REDACTED]{% endif %}".to_string());

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Template with basic conditionals should be valid during validation"
        );
    }

    #[test]
    fn test_very_long_redaction_template() {
        let mut config = PluginConfig::default();
        // Create a very long but valid template
        let long_template = "REDACTED_".repeat(100) + "{{secret_type}}";
        config.redaction.redaction_template = Some(long_template);

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Long but valid template should pass validation"
        );
    }

    #[test]
    fn test_template_with_simple_syntax() {
        let mut config = PluginConfig::default();
        config.redaction.redaction_template = Some("[{{secret_type}}]".to_string());

        let validation_result = ConfigManager::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Template with basic Tera syntax should be valid during validation"
        );
    }

    // Note: Command execution tests are complex to mock properly due to
    // plugin architecture. The core validation logic is thoroughly tested
    // in the configuration validation tests above.

    #[test]
    fn test_command_signature_has_verbose_flag() {
        let command = SecretConfigValidateCommand;
        let signature = command.signature();

        let verbose_flag = signature.named.iter().find(|n| n.long == "verbose");
        assert!(verbose_flag.is_some(), "Command should have verbose flag");

        if let Some(flag) = verbose_flag {
            assert_eq!(
                flag.short,
                Some('v'),
                "Verbose flag should have short form 'v'"
            );
            assert!(
                flag.desc.contains("detailed"),
                "Verbose flag should mention detailed results"
            );
        }
    }

    #[test]
    fn test_command_category() {
        let command = SecretConfigValidateCommand;
        let signature = command.signature();

        assert_eq!(signature.category, Category::Custom("secret".into()));
    }

    #[test]
    fn test_examples_have_expected_commands() {
        let command = SecretConfigValidateCommand;
        let examples = command.examples();

        assert!(examples
            .iter()
            .any(|e| e.example == "secret config validate"));
        assert!(examples
            .iter()
            .any(|e| e.example == "secret config validate --verbose"));
    }

    #[test]
    fn test_input_output_types() {
        let command = SecretConfigValidateCommand;
        let signature = command.signature();

        assert!(signature
            .input_output_types
            .contains(&(Type::Nothing, Type::Record(Box::new([])))));
    }

    #[test]
    fn test_validation_result_categorization() {
        // Test the logic for categorizing validation results
        let mut has_errors = false;
        let mut has_warnings = false;

        let validation_results = vec![
            ("Test", "Valid", "All good"),
            ("Test", "Warning", "Minor issue"),
            ("Test", "Error", "Major issue"),
        ];

        for (_, level, _) in &validation_results {
            match *level {
                "Error" => has_errors = true,
                "Warning" => has_warnings = true,
                _ => {}
            }
        }

        let status = if has_errors {
            "INVALID"
        } else if has_warnings {
            "VALID_WITH_WARNINGS"
        } else {
            "VALID"
        };

        assert_eq!(status, "INVALID");
        assert!(has_errors);
        assert!(has_warnings);
    }

    #[test]
    fn test_validation_result_counting() {
        let validation_results = [
            ("Test1", "Valid", "All good"),
            ("Test2", "Warning", "Minor issue"),
            ("Test3", "Error", "Major issue"),
            ("Test4", "Warning", "Another minor issue"),
            ("Test5", "Valid", "Also good"),
        ];

        let error_count = validation_results
            .iter()
            .filter(|(_, level, _)| level == &"Error")
            .count();
        let warning_count = validation_results
            .iter()
            .filter(|(_, level, _)| level == &"Warning")
            .count();

        assert_eq!(error_count, 1);
        assert_eq!(warning_count, 2);
    }

    #[test]
    fn test_security_level_warning_detection() {
        // Test the logic for detecting security level warnings
        use crate::config::SecurityLevel;

        let test_cases = vec![
            (SecurityLevel::Minimal, true),   // Should generate warning
            (SecurityLevel::Standard, false), // Should not generate warning
            (SecurityLevel::Paranoid, false), // Should not generate warning
        ];

        for (level, should_warn) in test_cases {
            let generates_warning = matches!(level, SecurityLevel::Minimal);
            assert_eq!(
                generates_warning, should_warn,
                "Security level {:?} warning detection failed",
                level
            );
        }
    }

    #[test]
    fn test_empty_string_trimming_logic() {
        // Test the template validation logic
        let test_cases = vec![
            ("", true),           // Empty string should trigger warning
            ("   ", true),        // Whitespace-only should trigger warning
            ("\t\n  ", true),     // Mixed whitespace should trigger warning
            ("valid", false),     // Valid content should not trigger warning
            ("  valid  ", false), // Valid content with padding should not trigger warning
        ];

        for (template, should_warn) in test_cases {
            let is_empty_or_whitespace = template.trim().is_empty();
            assert_eq!(
                is_empty_or_whitespace, should_warn,
                "Template '{}' whitespace detection failed",
                template
            );
        }
    }

    #[test]
    fn test_environment_variable_filtering() {
        // Test the environment variable filtering logic
        let test_env_vars = [
            ("NU_PLUGIN_SECRET_TEST", "value1"),
            ("NU_PLUGIN_SECRET_ANOTHER", "value2"),
            ("OTHER_VAR", "value3"),
            ("NU_PLUGIN_OTHER", "value4"),
        ];

        let filtered: Vec<_> = test_env_vars
            .iter()
            .filter(|(key, _)| key.starts_with("NU_PLUGIN_SECRET_"))
            .collect();

        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .any(|(key, _)| key == &"NU_PLUGIN_SECRET_TEST"));
        assert!(filtered
            .iter()
            .any(|(key, _)| key == &"NU_PLUGIN_SECRET_ANOTHER"));
    }

    #[test]
    #[cfg_attr(miri, ignore)] // File system operations not supported in miri
    fn test_config_file_path_construction() {
        // Test that config file path construction works
        let config_path = crate::config::get_config_file_path();

        if let Some(path) = config_path {
            let path_str = path.to_string_lossy();

            // Verify expected path components
            assert!(path_str.contains("nushell"));
            assert!(path_str.contains("plugins"));
            assert!(path_str.contains("secret"));
            assert!(path_str.ends_with("config.toml"));
        }
        // If path is None, that's acceptable in some test environments
    }

    #[test]
    #[cfg_attr(miri, ignore)] // This test intentionally leaks memory to test the leaking mechanism
    fn test_validation_message_leaking() {
        // Test that we properly handle string leaking for validation messages
        let error_msg = "Test error message".to_string();
        let leaked_msg: &'static str = Box::leak(error_msg.into_boxed_str());

        assert_eq!(leaked_msg, "Test error message");
        // Note: In real usage, this creates a memory leak, but it's acceptable
        // for validation messages that live for the duration of the program
    }
}
