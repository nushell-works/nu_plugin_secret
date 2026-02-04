//! Configuration system for nu_plugin_secret
//!
//! This module provides a comprehensive configuration system that supports:
//! - TOML configuration files at ~/.config/nushell/plugins/secret/config.toml
//! - Environment variable overrides
//! - Runtime configuration changes
//! - Hierarchical configuration loading with security validation

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Errors that can occur during configuration operations
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Security validation failed: {0}")]
    Security(String),

    #[error("Environment variable error: {0}")]
    Environment(String),
}

/// Context where redaction is being applied
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactionContext {
    /// Normal display/print operations
    Display,
    /// Debug output and logging
    Debug,
    /// JSON/YAML/TOML serialization
    Serialization,
    /// Audit logging contexts
    Audit,
}

/// Security levels for configuration validation
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    /// Minimal restrictions, more user flexibility
    Minimal,
    /// Standard security (recommended)
    #[default]
    Standard,
    /// Maximum security, minimal information disclosure
    Paranoid,
}

/// Main redaction configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RedactionConfig {
    /// Whether to disable redaction entirely (SHOW_UNREDACTED)
    #[serde(default)]
    pub show_unredacted: bool,
    /// Whether to mask secret strings with '*' character when displaying
    #[serde(default)]
    pub mask_secret: bool,
    /// Custom Tera template for redaction
    /// Example: "<redacted:{{secret_type}}>" or "[HIDDEN:{{secret_type}}]" or "moo"
    /// Available variables: secret_type, secret_length
    #[serde(default)]
    pub redaction_template: Option<String>,
}

impl RedactionConfig {
    /// Get the effective redaction template
    /// Returns the custom template if configured, otherwise the default template
    pub fn get_redaction_template(&self) -> &str {
        self.redaction_template
            .as_deref()
            .unwrap_or("<redacted:{{secret_type}}>")
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityConfig {
    /// Security level for validation
    #[serde(default)]
    pub level: SecurityLevel,
    /// Whether to audit configuration changes
    #[serde(default = "default_true")]
    pub audit_config_changes: bool,
    /// Maximum custom redaction text length
    #[serde(default = "default_max_custom_text_length")]
    pub max_custom_text_length: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            level: SecurityLevel::default(),
            audit_config_changes: true,
            max_custom_text_length: 50,
        }
    }
}

/// Main plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginConfig {
    /// Redaction configuration
    #[serde(default)]
    pub redaction: RedactionConfig,
    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,
    /// Configuration file version (for migration)
    #[serde(default = "default_config_version")]
    pub version: String,
}

fn default_config_version() -> String {
    "1.0".to_string()
}

// Helper functions for default values
fn default_true() -> bool {
    true
}
fn default_max_custom_text_length() -> usize {
    50
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            redaction: RedactionConfig::default(),
            security: SecurityConfig::default(),
            version: default_config_version(),
        }
    }
}

/// Configuration manager handles loading, saving, and validation
pub struct ConfigManager {
    config: PluginConfig,
    config_path: Option<PathBuf>,
}

impl ConfigManager {
    /// Create a new ConfigManager with the given config
    /// Note: This calls get_config_file_path() which involves filesystem operations
    /// that are not supported under Miri. Use new_in_memory() for Miri-compatible testing.
    #[cfg(not(miri))]
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config,
            config_path: get_config_file_path(),
        }
    }

    /// Create a new ConfigManager with the given config (Miri-compatible version)
    /// This version doesn't attempt filesystem operations
    #[cfg(miri)]
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config,
            config_path: None,
        }
    }

    /// Create a new ConfigManager without filesystem path lookup
    /// Useful for testing and Miri compatibility
    pub fn new_in_memory(config: PluginConfig) -> Self {
        Self {
            config,
            config_path: None,
        }
    }

    /// Load configuration with hierarchical priority
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = get_config_file_path();
        let mut config = PluginConfig::default();

        // Load from file if exists
        if let Some(path) = &config_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                config = toml::from_str(&content)?;
            }
        }

        // Apply environment variable overrides
        Self::apply_env_overrides(&mut config)?;

        // Validate security constraints
        Self::validate_config(&config)?;

        Ok(Self {
            config,
            config_path,
        })
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(path) = &self.config_path {
            // Create parent directories
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let content = toml::to_string_pretty(&self.config)?;
            std::fs::write(path, content)?;
        }
        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &PluginConfig {
        &self.config
    }

    /// Get mutable configuration reference
    pub fn config_mut(&mut self) -> &mut PluginConfig {
        &mut self.config
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &std::path::Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Configuration file not found",
            )));
        }

        let content = std::fs::read_to_string(path)?;
        let config: PluginConfig = toml::from_str(&content)?;

        // Validate the loaded configuration
        Self::validate_config(&config)?;

        Ok(Self {
            config,
            config_path: Some(path.to_path_buf()),
        })
    }

    /// Save configuration to a specific path
    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), ConfigError> {
        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(&self.config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Apply environment variable overrides
    pub fn apply_env_overrides(config: &mut PluginConfig) -> Result<(), ConfigError> {
        // Security level override
        if let Ok(security_level) = std::env::var("NU_PLUGIN_SECRET_SECURITY_LEVEL") {
            config.security.level = match security_level.as_str() {
                "minimal" => SecurityLevel::Minimal,
                "standard" => SecurityLevel::Standard,
                "paranoid" => SecurityLevel::Paranoid,
                _ => {
                    return Err(ConfigError::Environment(
                        "Invalid security level".to_string(),
                    ))
                }
            };
        }

        // Show unredacted override
        if let Ok(show_unredacted) = std::env::var("SHOW_UNREDACTED") {
            config.redaction.show_unredacted = match show_unredacted.as_str() {
                "1" | "true" | "True" | "TRUE" => true,
                "0" | "false" | "False" | "FALSE" => false,
                _ => {
                    return Err(ConfigError::Environment(
                        "Invalid value for SHOW_UNREDACTED (use 1/true or 0/false)".to_string(),
                    ))
                }
            };
        }

        Ok(())
    }

    /// Validate configuration against security constraints
    pub fn validate_config(config: &PluginConfig) -> Result<(), ConfigError> {
        // Validate redaction template if present
        if let Some(ref template) = config.redaction.redaction_template {
            Self::validate_redaction_template(template)?;
        }

        // Enhanced security validation based on security level
        Self::validate_security_level_constraints(config)?;

        Ok(())
    }

    /// Validate redaction template syntax and content
    fn validate_redaction_template(template: &str) -> Result<(), ConfigError> {
        // Validate Tera template syntax by attempting to compile it
        let mut tera = tera::Tera::default();

        // Register all standard template functions for validation
        crate::tera_functions::register_all_standard_functions(&mut tera);

        // Note: secret_string is available as a template variable during validation

        if let Err(e) = tera.add_raw_template("validation", template) {
            return Err(ConfigError::Invalid(format!(
                "Invalid Tera template syntax: {}",
                e
            )));
        }

        // Test template rendering with a sample value
        let mut context = tera::Context::new();
        context.insert("secret_type", "string");
        context.insert("secret_length", &10usize);
        context.insert("secret_string", "test_secret");
        if let Err(e) = tera.render("validation", &context) {
            return Err(ConfigError::Invalid(format!(
                "Template failed to render with test data: {}",
                e
            )));
        }

        Ok(())
    }

    /// Validate security level constraints
    fn validate_security_level_constraints(config: &PluginConfig) -> Result<(), ConfigError> {
        match config.security.level {
            SecurityLevel::Minimal => {
                // Minimal security allows most configurations, but still has basic limits
            }
            SecurityLevel::Standard => {
                // Standard security requires audit logging by default
                if !config.security.audit_config_changes {
                    return Err(ConfigError::Security(
                        "Standard security level requires audit logging to be enabled".to_string(),
                    ));
                }
            }
            SecurityLevel::Paranoid => {
                // Paranoid security has strict requirements
                if !config.security.audit_config_changes {
                    return Err(ConfigError::Security(
                        "Paranoid security level requires audit logging to be enabled".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Get the configuration file path
pub fn get_config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|config| {
        config
            .join("nushell")
            .join("plugins")
            .join("secret")
            .join("config.toml")
    })
}

/// Log configuration changes for audit purposes
pub fn audit_config_change(
    old_config: &PluginConfig,
    new_config: &PluginConfig,
) -> Result<(), ConfigError> {
    use std::io::Write;

    // Only log if there are actual changes
    if old_config == new_config {
        return Ok(());
    }

    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let mut changes = Vec::new();

    // Track redaction template changes
    if old_config.redaction.redaction_template != new_config.redaction.redaction_template {
        changes.push(format!(
            "redaction.redaction_template: {:?} -> {:?}",
            old_config.redaction.redaction_template, new_config.redaction.redaction_template
        ));
    }

    // Track security level changes
    if old_config.security.level != new_config.security.level {
        changes.push(format!(
            "security.level: {:?} -> {:?}",
            old_config.security.level, new_config.security.level
        ));
    }

    // Track audit setting changes (important for security)
    if old_config.security.audit_config_changes != new_config.security.audit_config_changes {
        changes.push(format!(
            "security.audit_config_changes: {} -> {}",
            old_config.security.audit_config_changes, new_config.security.audit_config_changes
        ));
    }

    if !changes.is_empty() {
        // Try to write to audit log file
        if let Some(config_dir) =
            get_config_file_path().and_then(|p| p.parent().map(|p| p.to_path_buf()))
        {
            let audit_file = config_dir.join("audit.log");

            // Create directory if it doesn't exist
            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                eprintln!(
                    "Warning: Failed to create config directory for audit log: {}",
                    e
                );
                return Ok(());
            }

            // Append to audit log
            let mut file = match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&audit_file)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Warning: Failed to open audit log file: {}", e);
                    return Ok(());
                }
            };

            let log_entry = format!(
                "[{}] Configuration changed: {}\n",
                timestamp,
                changes.join(", ")
            );
            if let Err(e) = file.write_all(log_entry.as_bytes()) {
                eprintln!("Warning: Failed to write to audit log: {}", e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = PluginConfig::default();
        assert!(!config.redaction.show_unredacted);
        assert_eq!(config.security.level, SecurityLevel::Standard);
    }

    #[test]
    fn test_config_serialization() {
        let config = PluginConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: PluginConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(
            config.redaction.show_unredacted,
            deserialized.redaction.show_unredacted
        );
        assert_eq!(
            config.redaction.mask_secret,
            deserialized.redaction.mask_secret
        );
        assert_eq!(config.security.level, deserialized.security.level);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_config_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = PluginConfig::default();
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, content).unwrap();

        assert!(config_path.exists());
        let loaded_content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: PluginConfig = toml::from_str(&loaded_content).unwrap();

        assert_eq!(config.version, loaded_config.version);
    }

    #[test]
    fn test_show_unredacted_env_var() {
        use std::env;

        // Test valid values for SHOW_UNREDACTED
        let test_cases = vec![
            ("1", true),
            ("true", true),
            ("True", true),
            ("TRUE", true),
            ("0", false),
            ("false", false),
            ("False", false),
            ("FALSE", false),
        ];

        for (env_value, expected) in test_cases {
            env::set_var("SHOW_UNREDACTED", env_value);

            let mut config = PluginConfig::default();
            let result = ConfigManager::apply_env_overrides(&mut config);

            assert!(result.is_ok(), "Failed for value: {}", env_value);
            assert_eq!(
                config.redaction.show_unredacted, expected,
                "Failed for value: {}",
                env_value
            );
        }

        // Test invalid value
        env::set_var("SHOW_UNREDACTED", "invalid");
        let mut config = PluginConfig::default();
        let result = ConfigManager::apply_env_overrides(&mut config);
        assert!(result.is_err());

        // Clean up
        env::remove_var("SHOW_UNREDACTED");
    }

    #[test]
    fn test_show_unredacted_default_value() {
        let config = PluginConfig::default();
        assert!(!config.redaction.show_unredacted);
    }

    #[test]
    fn test_mask_secret_default_value() {
        let config = PluginConfig::default();
        assert!(!config.redaction.mask_secret);
    }

    #[test]
    fn test_mask_secret_serialization() {
        // Test with mask_secret enabled
        let mut config = PluginConfig::default();
        config.redaction.mask_secret = true;

        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: PluginConfig = toml::from_str(&toml_str).unwrap();

        assert!(deserialized.redaction.mask_secret);

        // Test TOML contains the field
        assert!(toml_str.contains("mask_secret"));
    }

    #[test]
    fn test_mask_secret_toml_parsing() {
        // Test parsing TOML with mask_secret field
        let toml_config = r#"
version = "1.0"
[redaction]
mask_secret = true
show_unredacted = false
[security]
level = "standard"
"#;

        let config: PluginConfig = toml::from_str(toml_config).unwrap();
        assert!(config.redaction.mask_secret);
        assert!(!config.redaction.show_unredacted);

        // Test parsing TOML without mask_secret field (should default to false)
        let toml_config_minimal = r#"
version = "1.0"
[redaction]
show_unredacted = true
[security]
level = "standard"
"#;

        let config_minimal: PluginConfig = toml::from_str(toml_config_minimal).unwrap();
        assert!(!config_minimal.redaction.mask_secret); // Should default to false
        assert!(config_minimal.redaction.show_unredacted);
    }

    // Phase 1 Schema Compatibility Tests for Issue #10

    #[test]
    fn test_config_schema_compatibility() {
        // Test minimal config (Issue #10)
        let minimal_toml = r#"
version = "1.0"
[redaction]
redaction_template = "<redacted:{{secret_type}}>"
[security]
level = "standard"
"#;

        let config: PluginConfig =
            toml::from_str(minimal_toml).expect("Should parse minimal config");
        assert_eq!(
            config.redaction.get_redaction_template(),
            "<redacted:{{secret_type}}>"
        );
        ConfigManager::validate_config(&config).expect("Should validate minimal config");
    }

    #[test]
    fn test_custom_template_compatibility() {
        // Test custom redaction template
        let custom_toml = r#"
version = "1.0"
[redaction]
redaction_template = "[SECRET_DATA]"
[security]
level = "standard"
"#;

        let config: PluginConfig = toml::from_str(custom_toml).expect("Should parse custom config");

        assert_eq!(config.redaction.get_redaction_template(), "[SECRET_DATA]");
    }

    #[test]
    fn test_security_config_defaults() {
        // Test that security config fields have proper defaults when not specified
        let security_minimal_toml = r#"
version = "1.0"
[redaction]
[security]
level = "paranoid"
"#;

        let config: PluginConfig =
            toml::from_str(security_minimal_toml).expect("Should parse security minimal config");

        assert_eq!(config.security.level, SecurityLevel::Paranoid);
        assert!(config.security.audit_config_changes); // default true
        assert_eq!(config.security.max_custom_text_length, 50); // default value
    }

    #[test]
    fn test_existing_config_files() {
        // Test representative configuration patterns that would be in test files
        // Using inline TOML to be Miri-compatible (no file system access)
        let test_configs = [
            // Custom template config
            (
                "custom",
                r#"
version = "1.0"
[redaction]
redaction_template = "[SECRET_DATA]"
[security]
level = "standard"
"#,
            ),
            // Minimal config
            (
                "minimal",
                r#"
version = "1.0"
[redaction]
[security]
level = "minimal"
"#,
            ),
            // Paranoid config
            (
                "paranoid",
                r#"
version = "1.0"
[redaction]
redaction_template = "<redacted>"
[security]
level = "paranoid"
"#,
            ),
        ];

        for (name, content) in &test_configs {
            let config: PluginConfig = toml::from_str(content)
                .unwrap_or_else(|e| panic!("Should parse {} config: {}", name, e));
            ConfigManager::validate_config(&config)
                .unwrap_or_else(|e| panic!("Should validate {} config: {}", name, e));
        }
    }

    #[test]
    fn test_backward_compatibility() {
        // Test configs that might exist in user environments
        let legacy_configs = vec![
            // Minimal legacy config
            (
                "minimal_legacy",
                r#"
version = "1.0"
[redaction]
"#,
            ),
            // Only security level specified
            (
                "security_only",
                r#"
version = "1.0"
[security]
level = "minimal"
"#,
            ),
        ];

        for (name, toml_content) in legacy_configs {
            let config: PluginConfig = toml::from_str(toml_content)
                .unwrap_or_else(|_| panic!("Legacy config {} should parse", name));
            ConfigManager::validate_config(&config)
                .unwrap_or_else(|_| panic!("Legacy config {} should validate", name));
        }
    }

    #[test]
    fn test_redaction_template_validation() {
        // Test valid template with secret_type
        let valid_template = "<redacted:{{secret_type}}>";
        let result = ConfigManager::validate_redaction_template(valid_template);
        assert!(result.is_ok());

        // Test template without secret_type variable (now valid)
        let simple_template = "moo";
        let result = ConfigManager::validate_redaction_template(simple_template);
        assert!(result.is_ok());

        // Test another simple template
        let static_template = "<redacted>";
        let result = ConfigManager::validate_redaction_template(static_template);
        assert!(result.is_ok());

        // Test template with invalid Tera syntax
        let syntax_error_template = "{{unclosed";
        let result = ConfigManager::validate_redaction_template(syntax_error_template);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid Tera template syntax"));

        // Test valid custom template
        let custom_template = "[HIDDEN:{{secret_type}}]";
        let result = ConfigManager::validate_redaction_template(custom_template);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_with_custom_template() {
        let config_toml = r#"
version = "1.0"
[redaction]
style = "typed_brackets"
redaction_template = "<custom:{{secret_type}}>"
[security]
level = "standard"
"#;

        let config: PluginConfig =
            toml::from_str(config_toml).expect("Should parse config with custom template");

        ConfigManager::validate_config(&config)
            .expect("Should validate config with custom template");
        assert_eq!(
            config.redaction.get_redaction_template(),
            "<custom:{{secret_type}}>"
        );
    }

    #[test]
    fn test_simple_template_acceptance() {
        // Test that simple template without {{secret_type}} is now accepted
        let simple_config_content = r#"
version = "1.0"

[redaction]
style = "typed_brackets"
redaction_template = "moo"

[security]
level = "standard"
"#;

        let config: PluginConfig =
            toml::from_str(simple_config_content).expect("Should parse config");

        // This should now pass validation
        let result = ConfigManager::validate_config(&config);
        assert!(result.is_ok());
        assert_eq!(config.redaction.get_redaction_template(), "moo");
    }

    #[test]
    fn test_real_config_file_integration() {
        // Test loading configuration from file (inline to be Miri-compatible)
        let config_content = r#"
version = "1.0"

[redaction]
redaction_template = "[HIDDEN:{{secret_type}}]"

[security]
level = "standard"
audit_config_changes = true
max_custom_text_length = 50
"#;

        let config: PluginConfig =
            toml::from_str(config_content).expect("Should parse test config file");

        // Validate the configuration
        ConfigManager::validate_config(&config).expect("Should validate test config file");

        // Test that custom template is properly set
        assert_eq!(
            config.redaction.get_redaction_template(),
            "[HIDDEN:{{secret_type}}]"
        );
        assert!(!config.redaction.show_unredacted);
        assert_eq!(config.security.level, SecurityLevel::Standard);
    }

    #[test]
    fn test_empty_config_sections() {
        // Test that empty sections get proper defaults
        let empty_sections_toml = r#"
version = "1.0"
"#;

        let config: PluginConfig =
            toml::from_str(empty_sections_toml).expect("Should parse config with empty sections");

        // Should have all defaults
        assert!(!config.redaction.show_unredacted);
        assert_eq!(config.security.level, SecurityLevel::Standard);
        assert!(config.security.audit_config_changes);

        ConfigManager::validate_config(&config).expect("Should validate empty sections config");
    }
}
