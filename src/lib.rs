//! Core library for `nu_plugin_secret` — registers commands and re-exports secret types.

use std::sync::{Arc, RwLock};

use nu_plugin::{Plugin, PluginCommand};

pub mod commands;
pub mod config;
pub mod memory_optimizations;
pub mod performance_monitoring;
pub mod redaction;
mod secret_types;
pub mod startup_optimizations;
pub mod tera_functions;

use commands::*;
pub use config::ConfigManager;
pub use secret_types::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretList, SecretRecord,
    SecretString,
};

/// Secret Plugin with dependency-injected configuration
#[derive(Clone)]
pub struct SecretPlugin {
    config_manager: Arc<RwLock<ConfigManager>>,
}

impl SecretPlugin {
    /// Create a new SecretPlugin with a specific ConfigManager
    /// Useful for testing with custom configurations
    pub fn new(config_manager: ConfigManager) -> Self {
        Self {
            config_manager: Arc::new(RwLock::new(config_manager)),
        }
    }

    /// Get a reference to the config manager
    pub fn config_manager(&self) -> &Arc<RwLock<ConfigManager>> {
        &self.config_manager
    }
}

impl Default for SecretPlugin {
    fn default() -> Self {
        // Load configuration from file system or use defaults
        // Skip config loading under Miri since it involves file system operations
        #[cfg(not(miri))]
        let config_manager = ConfigManager::load().unwrap_or_else(|_| {
            // If config loading fails, use default configuration
            ConfigManager::new(config::PluginConfig::default())
        });

        #[cfg(miri)]
        let config_manager = ConfigManager::new(config::PluginConfig::default());

        Self {
            config_manager: Arc::new(RwLock::new(config_manager)),
        }
    }
}

impl Plugin for SecretPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        // Initialize optimizations on first command access
        startup_optimizations::command_optimizations::init_command_cache();

        // Initialize Tera-based redaction templating system
        let _ = redaction::init_redaction_templating();

        vec![
            // Unified wrap command
            Box::new(SecretWrapCommand),
            Box::new(SecretWrapWithCommand),
            // Utility commands
            Box::new(SecretUnwrapCommand),
            Box::new(SecretContainsCommand),
            Box::new(SecretHashCommand),
            Box::new(SecretLengthCommand),
            Box::new(SecretInfoCommand),
            Box::new(SecretValidateCommand),
            Box::new(SecretTypeOfCommand),
            // Configuration commands
            Box::new(SecretConfigureCommand),
            Box::new(SecretConfigShowCommand),
            Box::new(SecretConfigResetCommand),
            Box::new(SecretConfigValidateCommand),
            Box::new(SecretConfigExportCommand),
            Box::new(SecretConfigImportCommand),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_version() {
        let plugin = SecretPlugin::default();
        assert!(!plugin.version().is_empty());
    }

    #[test]
    fn test_plugin_commands() {
        let plugin = SecretPlugin::default();
        let commands = plugin.commands();
        assert_eq!(commands.len(), 15);

        // Test all commands to ensure they're registered correctly
        let command_names: Vec<&str> = commands.iter().map(|cmd| cmd.name()).collect();
        // Unified wrap command
        assert!(command_names.contains(&"secret wrap"));
        assert!(command_names.contains(&"secret wrap-with"));
        // Utility commands
        assert!(command_names.contains(&"secret unwrap"));
        assert!(command_names.contains(&"secret contains"));
        assert!(command_names.contains(&"secret hash"));
        assert!(command_names.contains(&"secret info"));
        assert!(command_names.contains(&"secret validate"));
        assert!(command_names.contains(&"secret type-of"));
        // Configuration commands
        assert!(command_names.contains(&"secret configure"));
        assert!(command_names.contains(&"secret config show"));
        assert!(command_names.contains(&"secret config reset"));
        assert!(command_names.contains(&"secret config validate"));
        assert!(command_names.contains(&"secret config export"));
        assert!(command_names.contains(&"secret config import"));
    }
}
