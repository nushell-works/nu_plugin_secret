//! Configuration system for nu_plugin_secret
//!
//! This module provides a comprehensive configuration system that supports:
//! - TOML configuration files at ~/.config/nushell/plugins/secret/config.toml
//! - Environment variable overrides
//! - Runtime configuration changes
//! - Hierarchical configuration loading with security validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

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

/// Redaction styles available for secret display
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedactionStyle {
    /// Default: `<redacted:type>`
    TypedBrackets,
    /// Simple: `<redacted>`
    Simple,
    /// Asterisks: `***` or `****` (length-based)
    Asterisks,
    /// Square brackets: `[HIDDEN]`
    Brackets,
    /// Custom user-defined text
    Custom(String),
}

// Custom serde implementation to handle TOML format compatibility
impl<'de> serde::Deserialize<'de> for RedactionStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "typed_brackets" => Ok(RedactionStyle::TypedBrackets),
            "simple" => Ok(RedactionStyle::Simple),
            "asterisks" => Ok(RedactionStyle::Asterisks),
            "brackets" => Ok(RedactionStyle::Brackets),
            "custom" => Ok(RedactionStyle::Custom(String::new())), // Will be filled from custom_text
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &[
                    "typed_brackets",
                    "simple",
                    "asterisks",
                    "brackets",
                    "custom",
                ],
            )),
        }
    }
}

impl serde::Serialize for RedactionStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            RedactionStyle::TypedBrackets => "typed_brackets",
            RedactionStyle::Simple => "simple",
            RedactionStyle::Asterisks => "asterisks",
            RedactionStyle::Brackets => "brackets",
            RedactionStyle::Custom(_) => "custom",
        };
        serializer.serialize_str(s)
    }
}

impl Default for RedactionStyle {
    fn default() -> Self {
        Self::TypedBrackets
    }
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    /// Minimal restrictions, more user flexibility
    Minimal,
    /// Standard security (recommended)
    Standard,
    /// Maximum security, minimal information disclosure
    Paranoid,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Standard
    }
}

/// Main redaction configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedactionConfig {
    /// Global redaction style
    #[serde(default)]
    pub style: RedactionStyle,
    /// Custom redaction text (when style is Custom)
    #[serde(default)]
    pub custom_text: Option<String>,
    /// Whether to show type information
    #[serde(default = "default_true")]
    pub show_type_info: bool,
    /// Whether to preserve length information in redaction
    #[serde(default)]
    pub preserve_length: bool,
    /// Whether to disable redaction entirely (SHOW_UNREDACTED)
    #[serde(default)]
    pub show_unredacted: bool,
    /// Per-type redaction overrides
    #[serde(default)]
    pub per_type: HashMap<String, RedactionStyle>,
    /// Per-context redaction overrides
    #[serde(default)]
    pub per_context: HashMap<RedactionContext, RedactionStyle>,
}

impl RedactionConfig {
    /// Post-deserialization method to merge custom_text into Custom variant
    pub fn post_deserialize(&mut self) {
        if matches!(self.style, RedactionStyle::Custom(_)) {
            if let Some(ref custom_text) = self.custom_text {
                self.style = RedactionStyle::Custom(custom_text.clone());
            }
        }
    }
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self {
            style: RedactionStyle::default(),
            custom_text: None,
            show_type_info: true,
            preserve_length: false,
            show_unredacted: false,
            per_type: HashMap::new(),
            per_context: HashMap::new(),
        }
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

impl PluginConfig {
    /// Post-deserialization method to apply fixups
    pub fn post_deserialize(&mut self) {
        self.redaction.post_deserialize();
    }
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
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config,
            config_path: get_config_file_path(),
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
                config.post_deserialize();
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
        let mut config: PluginConfig = toml::from_str(&content)?;
        config.post_deserialize();

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
        // Redaction style override
        if let Ok(style_str) = std::env::var("NU_PLUGIN_SECRET_REDACTION_STYLE") {
            config.redaction.style = match style_str.as_str() {
                "typed_brackets" => RedactionStyle::TypedBrackets,
                "simple" => RedactionStyle::Simple,
                "asterisks" => RedactionStyle::Asterisks,
                "brackets" => RedactionStyle::Brackets,
                custom => RedactionStyle::Custom(custom.to_string()),
            };
        }

        // Custom text override
        if let Ok(custom_text) = std::env::var("NU_PLUGIN_SECRET_CUSTOM_TEXT") {
            config.redaction.custom_text = Some(custom_text);
        }

        // Type info override
        if let Ok(show_type) = std::env::var("NU_PLUGIN_SECRET_SHOW_TYPE_INFO") {
            config.redaction.show_type_info = show_type.parse().map_err(|_| {
                ConfigError::Environment("Invalid boolean for SHOW_TYPE_INFO".to_string())
            })?;
        }

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
        // Validate custom redaction text
        if let RedactionStyle::Custom(ref text) = config.redaction.style {
            if text.len() > config.security.max_custom_text_length {
                return Err(ConfigError::Security(format!(
                    "Custom redaction text too long: {} > {}",
                    text.len(),
                    config.security.max_custom_text_length
                )));
            }

            // Check for potentially revealing custom text
            if config.security.level == SecurityLevel::Paranoid {
                let lower_text = text.to_lowercase();
                let suspicious_words = ["pass", "key", "secret", "token", "auth"];
                for word in suspicious_words {
                    if lower_text.contains(word) {
                        return Err(ConfigError::Security(
                            "Custom redaction text may reveal information about secret type"
                                .to_string(),
                        ));
                    }
                }
            }
        }

        // Enhanced security validation based on security level
        Self::validate_security_level_constraints(config)?;

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

                if config.security.max_custom_text_length > 20 {
                    return Err(ConfigError::Security(
                        "Paranoid security level limits custom text to 20 characters".to_string(),
                    ));
                }

                if config.redaction.show_type_info {
                    return Err(ConfigError::Security(
                        "Paranoid security level prohibits showing type information".to_string(),
                    ));
                }

                if config.redaction.preserve_length {
                    return Err(ConfigError::Security(
                        "Paranoid security level prohibits preserving length information"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get redaction text for a specific type and context
    pub fn get_redaction_text(&self, type_name: &str, context: RedactionContext) -> String {
        // Priority order: per_context -> per_type -> global
        let style = self
            .config
            .redaction
            .per_context
            .get(&context)
            .or_else(|| self.config.redaction.per_type.get(type_name))
            .unwrap_or(&self.config.redaction.style);

        self.format_redaction_text(style, type_name)
    }

    /// Format redaction text based on style and type
    fn format_redaction_text(&self, style: &RedactionStyle, type_name: &str) -> String {
        match style {
            RedactionStyle::TypedBrackets => {
                if self.config.redaction.show_type_info {
                    format!("<redacted:{}>", type_name)
                } else {
                    "<redacted>".to_string()
                }
            }
            RedactionStyle::Simple => "<redacted>".to_string(),
            RedactionStyle::Asterisks => {
                if self.config.redaction.preserve_length {
                    // This would need length information passed in
                    "***".to_string()
                } else {
                    "***".to_string()
                }
            }
            RedactionStyle::Brackets => "[HIDDEN]".to_string(),
            RedactionStyle::Custom(text) => text.clone(),
        }
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

/// Global configuration manager instance
static CONFIG_MANAGER: OnceLock<RwLock<ConfigManager>> = OnceLock::new();

/// Initialize the global configuration manager
pub fn init_config() -> Result<(), ConfigError> {
    let manager = ConfigManager::load()?;
    CONFIG_MANAGER
        .set(RwLock::new(manager))
        .map_err(|_| ConfigError::Invalid("Configuration already initialized".to_string()))?;
    Ok(())
}

/// Get a reference to the global configuration
pub fn get_config() -> Result<std::sync::RwLockReadGuard<'static, ConfigManager>, ConfigError> {
    CONFIG_MANAGER
        .get()
        .ok_or_else(|| ConfigError::Invalid("Configuration not initialized".to_string()))?
        .read()
        .map_err(|_| ConfigError::Invalid("Configuration lock poisoned".to_string()))
}

/// Get a mutable reference to the global configuration
pub fn get_config_mut() -> Result<std::sync::RwLockWriteGuard<'static, ConfigManager>, ConfigError>
{
    CONFIG_MANAGER
        .get()
        .ok_or_else(|| ConfigError::Invalid("Configuration not initialized".to_string()))?
        .write()
        .map_err(|_| ConfigError::Invalid("Configuration lock poisoned".to_string()))
}

/// Update the global configuration with a new config
pub fn update_config(new_config: PluginConfig) -> Result<(), ConfigError> {
    let mut config_guard = get_config_mut()?;

    // Log configuration change if audit logging is enabled
    let old_config = config_guard.config().clone();
    if old_config.security.audit_config_changes {
        audit_config_change(&old_config, &new_config)?;
    }

    *config_guard.config_mut() = new_config;
    config_guard.save()?;
    Ok(())
}

/// Log configuration changes for audit purposes
fn audit_config_change(
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

    // Track redaction style changes
    if old_config.redaction.style != new_config.redaction.style {
        changes.push(format!(
            "redaction.style: {:?} -> {:?}",
            old_config.redaction.style, new_config.redaction.style
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
        assert_eq!(config.redaction.style, RedactionStyle::TypedBrackets);
        assert!(config.redaction.show_type_info);
        assert!(!config.redaction.preserve_length);
        assert_eq!(config.security.level, SecurityLevel::Standard);
    }

    #[test]
    fn test_config_serialization() {
        let config = PluginConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: PluginConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.redaction.style, deserialized.redaction.style);
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
    fn test_redaction_text_formatting() {
        let manager = ConfigManager {
            config: PluginConfig::default(),
            config_path: None,
        };

        assert_eq!(
            manager.get_redaction_text("string", RedactionContext::Display),
            "<redacted:string>"
        );

        let mut config_with_simple = PluginConfig::default();
        config_with_simple.redaction.style = RedactionStyle::Simple;
        let manager_simple = ConfigManager {
            config: config_with_simple,
            config_path: None,
        };

        assert_eq!(
            manager_simple.get_redaction_text("string", RedactionContext::Display),
            "<redacted>"
        );
    }

    #[test]
    fn test_security_validation() {
        let mut config = PluginConfig::default();
        config.redaction.style =
            RedactionStyle::Custom("verylongcustomtextthatexceedsthelimit".to_string());
        config.security.max_custom_text_length = 20;

        let result = ConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
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

    // Phase 1 Schema Compatibility Tests for Issue #10

    #[test]
    fn test_config_schema_compatibility() {
        // Test minimal config (Issue #10)
        let minimal_toml = r#"
version = "1.0"
[redaction]
style = "typed_brackets"
[security]
level = "standard"
"#;

        let mut config: PluginConfig =
            toml::from_str(minimal_toml).expect("Should parse minimal config");
        config.post_deserialize();
        assert_eq!(config.redaction.style, RedactionStyle::TypedBrackets);
        ConfigManager::validate_config(&config).expect("Should validate minimal config");
    }

    #[test]
    fn test_custom_style_compatibility() {
        // Test custom style with separate custom_text field
        let custom_toml = r#"
version = "1.0"
[redaction]
style = "custom"
custom_text = "[SECRET_DATA]"
[security]
level = "standard"
"#;

        let mut config: PluginConfig =
            toml::from_str(custom_toml).expect("Should parse custom config");
        config.post_deserialize();

        match config.redaction.style {
            RedactionStyle::Custom(text) => assert_eq!(text, "[SECRET_DATA]"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_security_config_defaults() {
        // Test that security config fields have proper defaults when not specified
        let security_minimal_toml = r#"
version = "1.0"
[redaction]
style = "simple"
[security]
level = "paranoid"
"#;

        let mut config: PluginConfig =
            toml::from_str(security_minimal_toml).expect("Should parse security minimal config");
        config.post_deserialize();

        assert_eq!(config.security.level, SecurityLevel::Paranoid);
        assert!(config.security.audit_config_changes); // default true
        assert_eq!(config.security.max_custom_text_length, 50); // default value
    }

    #[test]
    fn test_existing_config_files() {
        // Test representative configuration patterns that would be in test files
        // Using inline TOML to be Miri-compatible (no file system access)
        let test_configs = [
            // Custom style config
            (
                "custom",
                r#"
version = "1.0"
[redaction]
style = "custom"
custom_text = "[SECRET_DATA]"
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
style = "simple"
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
style = "simple"
show_type_info = false
preserve_length = false
[security]
level = "paranoid"
max_custom_text_length = 20
"#,
            ),
            // Default-style config
            (
                "default",
                r#"
version = "1.0"
[redaction]
style = "typed_brackets"
show_type_info = true
[security]
level = "standard"
"#,
            ),
            // Simple style config
            (
                "simple",
                r#"
version = "1.0"
[redaction]
style = "simple"
show_type_info = false
[security]
level = "standard"
"#,
            ),
            // Asterisks style config
            (
                "asterisks",
                r#"
version = "1.0"
[redaction]
style = "asterisks"
[security]
level = "standard"
"#,
            ),
            // Brackets style config
            (
                "brackets",
                r#"
version = "1.0"
[redaction]
style = "brackets"
[security]
level = "standard"
"#,
            ),
        ];

        for (name, content) in &test_configs {
            let mut config: PluginConfig = toml::from_str(content)
                .unwrap_or_else(|e| panic!("Should parse {} config: {}", name, e));
            config.post_deserialize();
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
style = "simple"
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
            let mut config: PluginConfig = toml::from_str(toml_content)
                .unwrap_or_else(|_| panic!("Legacy config {} should parse", name));
            config.post_deserialize();
            ConfigManager::validate_config(&config)
                .unwrap_or_else(|_| panic!("Legacy config {} should validate", name));
        }
    }

    #[test]
    fn test_redaction_style_serialization() {
        // Test that all redaction styles serialize/deserialize correctly within a config context
        let test_cases = vec![
            (RedactionStyle::TypedBrackets, "typed_brackets"),
            (RedactionStyle::Simple, "simple"),
            (RedactionStyle::Asterisks, "asterisks"),
            (RedactionStyle::Brackets, "brackets"),
        ];

        for (style, expected_str) in test_cases {
            // Create a minimal config with the style
            let mut config = PluginConfig::default();
            config.redaction.style = style.clone();

            // Test serialization
            let serialized = toml::to_string(&config).expect("Should serialize config");
            assert!(
                serialized.contains(expected_str),
                "Serialized config should contain {}",
                expected_str
            );

            // Test deserialization
            let deserialized: PluginConfig =
                toml::from_str(&serialized).expect("Should deserialize config");
            match (&style, &deserialized.redaction.style) {
                (RedactionStyle::TypedBrackets, RedactionStyle::TypedBrackets) => {}
                (RedactionStyle::Simple, RedactionStyle::Simple) => {}
                (RedactionStyle::Asterisks, RedactionStyle::Asterisks) => {}
                (RedactionStyle::Brackets, RedactionStyle::Brackets) => {}
                _ => panic!("Mismatched deserialization for {}", expected_str),
            }
        }

        // Test custom style separately
        let custom_config_toml = r#"
version = "1.0"
[redaction]
style = "custom"
custom_text = "test_custom_text"
"#;

        let mut config: PluginConfig =
            toml::from_str(custom_config_toml).expect("Should parse custom config");
        config.post_deserialize();

        match config.redaction.style {
            RedactionStyle::Custom(text) => assert_eq!(text, "test_custom_text"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_empty_config_sections() {
        // Test that empty sections get proper defaults
        let empty_sections_toml = r#"
version = "1.0"
"#;

        let mut config: PluginConfig =
            toml::from_str(empty_sections_toml).expect("Should parse config with empty sections");
        config.post_deserialize();

        // Should have all defaults
        assert_eq!(config.redaction.style, RedactionStyle::TypedBrackets);
        assert!(config.redaction.show_type_info);
        assert_eq!(config.security.level, SecurityLevel::Standard);
        assert!(config.security.audit_config_changes);

        ConfigManager::validate_config(&config).expect("Should validate empty sections config");
    }
}
