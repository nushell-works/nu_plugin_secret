//! Configuration system for nu_plugin_secret
//!
//! This module provides a comprehensive configuration system that supports:
//! - TOML configuration files at ~/.config/nushell/plugins/secret/config.toml
//! - Environment variable overrides
//! - Runtime configuration changes
//! - Hierarchical configuration loading with security validation

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

/// Partial redaction configuration for string secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialRedactionConfig {
    /// Whether partial redaction is enabled
    pub enabled: bool,
    /// Number of characters to show from the beginning
    pub show_first: usize,
    /// Number of characters to show from the end
    pub show_last: usize,
    /// Minimum length required for partial redaction
    pub min_length: usize,
    /// Maximum length to show (security limit)
    pub max_reveal: usize,
    /// Use salted hash instead of actual characters
    pub use_hash: bool,
    /// Salt for hash-based partial redaction (base64 encoded)
    pub hash_salt: String,
}

impl Default for PartialRedactionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_first: 4,
            show_last: 4,
            min_length: 12,
            max_reveal: 8,
            use_hash: false,
            hash_salt: "nu_plugin_secret_default_salt".to_string(),
        }
    }
}

/// Main redaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionConfig {
    /// Global redaction style
    pub style: RedactionStyle,
    /// Custom redaction text (when style is Custom)
    pub custom_text: Option<String>,
    /// Whether to show type information
    pub show_type_info: bool,
    /// Whether to preserve length information in redaction
    pub preserve_length: bool,
    /// Per-type redaction overrides
    #[serde(default)]
    pub per_type: HashMap<String, RedactionStyle>,
    /// Per-context redaction overrides
    #[serde(default)]
    pub per_context: HashMap<RedactionContext, RedactionStyle>,
    /// Partial redaction configuration
    pub partial: PartialRedactionConfig,
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self {
            style: RedactionStyle::default(),
            custom_text: None,
            show_type_info: true,
            preserve_length: false,
            per_type: HashMap::new(),
            per_context: HashMap::new(),
            partial: PartialRedactionConfig::default(),
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Security level for validation
    pub level: SecurityLevel,
    /// Whether to audit configuration changes
    pub audit_config_changes: bool,
    /// Maximum custom redaction text length
    pub max_custom_text_length: usize,
    /// Allow partial redaction (can reveal information)
    pub allow_partial_redaction: bool,
    /// Minimum secret length to allow partial redaction
    pub min_partial_redaction_length: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            level: SecurityLevel::default(),
            audit_config_changes: true,
            max_custom_text_length: 50,
            allow_partial_redaction: false,
            min_partial_redaction_length: 16,
        }
    }
}

/// Main plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Redaction configuration
    pub redaction: RedactionConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Configuration file version (for migration)
    #[serde(default = "default_config_version")]
    pub version: String,
}

fn default_config_version() -> String {
    "1.0".to_string()
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
    fn apply_env_overrides(config: &mut PluginConfig) -> Result<(), ConfigError> {
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

        // Partial redaction override
        if let Ok(allow_partial) = std::env::var("NU_PLUGIN_SECRET_ALLOW_PARTIAL_REDACTION") {
            config.security.allow_partial_redaction = allow_partial.parse().map_err(|_| {
                ConfigError::Environment("Invalid boolean for ALLOW_PARTIAL_REDACTION".to_string())
            })?;
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

        // Validate partial redaction settings
        if config.redaction.partial.enabled {
            if !config.security.allow_partial_redaction {
                return Err(ConfigError::Security(
                    "Partial redaction is disabled by security policy".to_string(),
                ));
            }

            let partial = &config.redaction.partial;
            let total_reveal = partial.show_first + partial.show_last;

            if total_reveal > partial.max_reveal {
                return Err(ConfigError::Security(format!(
                    "Total partial reveal {} > max allowed {}",
                    total_reveal, partial.max_reveal
                )));
            }

            if partial.min_length < config.security.min_partial_redaction_length {
                return Err(ConfigError::Security(format!(
                    "Minimum partial redaction length {} < security minimum {}",
                    partial.min_length, config.security.min_partial_redaction_length
                )));
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

    /// Apply partial redaction to a string secret
    pub fn apply_partial_redaction(&self, secret: &str, type_name: &str) -> Option<String> {
        if !self.config.redaction.partial.enabled
            || secret.len() < self.config.redaction.partial.min_length
        {
            return None;
        }

        let partial = &self.config.redaction.partial;

        if partial.use_hash {
            self.apply_hash_partial_redaction(secret, type_name)
        } else {
            self.apply_char_partial_redaction(secret, type_name)
        }
    }

    /// Apply character-based partial redaction
    fn apply_char_partial_redaction(&self, secret: &str, _type_name: &str) -> Option<String> {
        let partial = &self.config.redaction.partial;
        let chars: Vec<char> = secret.chars().collect();

        if chars.len() < partial.min_length {
            return None;
        }

        let show_first = partial.show_first.min(chars.len() / 3);
        let show_last = partial.show_last.min(chars.len() / 3);
        let total_show = show_first + show_last;

        if total_show >= chars.len() || total_show > partial.max_reveal {
            return None;
        }

        let first_part: String = chars.iter().take(show_first).collect();
        let last_part: String = chars.iter().skip(chars.len() - show_last).collect();
        let middle_len = chars.len() - total_show;
        let middle = "*".repeat(middle_len.min(10));

        Some(format!("{}{}...{}", first_part, middle, last_part))
    }

    /// Apply hash-based partial redaction
    fn apply_hash_partial_redaction(&self, secret: &str, type_name: &str) -> Option<String> {
        let partial = &self.config.redaction.partial;

        let mut hasher = Sha256::new();
        hasher.update(partial.hash_salt.as_bytes());
        hasher.update(secret.as_bytes());
        hasher.update(type_name.as_bytes());
        let hash = hasher.finalize();

        // Take first 8 characters of hex hash for partial reveal
        let hash_str = format!("{:x}", hash);
        let partial_hash = &hash_str[..8];

        Some(format!("{}...{}", partial_hash, secret.len()))
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
    *config_guard.config_mut() = new_config;
    config_guard.save()?;
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
    fn test_partial_redaction() {
        let mut config = PluginConfig::default();
        config.redaction.partial.enabled = true;
        config.redaction.partial.show_first = 2;
        config.redaction.partial.show_last = 2;
        config.redaction.partial.min_length = 8;
        config.security.allow_partial_redaction = true;

        let manager = ConfigManager {
            config,
            config_path: None,
        };

        let result = manager.apply_partial_redaction("verylongsecret", "string");
        assert!(result.is_some());
        let result_str = result.unwrap();
        assert!(result_str.contains("ve") && result_str.contains("et"));

        let short_result = manager.apply_partial_redaction("short", "string");
        assert!(short_result.is_none());
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
}
