use nu_plugin::{Plugin, PluginCommand};

pub mod commands;
pub mod config;
pub mod memory_optimizations;
pub mod performance_monitoring;
pub mod redaction;
mod secret_types;
pub mod startup_optimizations;

use commands::*;
pub use secret_types::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretList, SecretRecord,
    SecretString,
};

pub struct SecretPlugin;

impl Plugin for SecretPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        // Initialize optimizations and configuration on first command access
        startup_optimizations::command_optimizations::init_command_cache();

        // Initialize Tera-based redaction templating system
        let _ = redaction::init_redaction_templating();

        // Initialize configuration system (ignore errors for now)
        // Skip config initialization under Miri since it involves file system operations
        #[cfg(not(miri))]
        let _ = config::init_config();

        vec![
            // Unified wrap command
            Box::new(SecretWrapCommand),
            // Core secret wrap commands (kept for backward compatibility)
            Box::new(SecretWrapStringCommand),
            Box::new(SecretWrapIntCommand),
            Box::new(SecretWrapBoolCommand),
            Box::new(SecretWrapRecordCommand),
            Box::new(SecretWrapListCommand),
            Box::new(SecretWrapFloatCommand),
            Box::new(SecretWrapBinaryCommand),
            Box::new(SecretWrapDateCommand),
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
        let plugin = SecretPlugin;
        assert!(!plugin.version().is_empty());
    }

    #[test]
    fn test_plugin_commands() {
        let plugin = SecretPlugin;
        let commands = plugin.commands();
        assert_eq!(commands.len(), 22);

        // Test all commands to ensure they're registered correctly
        let command_names: Vec<&str> = commands.iter().map(|cmd| cmd.name()).collect();
        // Unified wrap command
        assert!(command_names.contains(&"secret wrap"));
        // Core secret wrap commands (kept for backward compatibility)
        assert!(command_names.contains(&"secret wrap-string"));
        assert!(command_names.contains(&"secret wrap-int"));
        assert!(command_names.contains(&"secret wrap-bool"));
        assert!(command_names.contains(&"secret wrap-record"));
        assert!(command_names.contains(&"secret wrap-list"));
        assert!(command_names.contains(&"secret wrap-float"));
        assert!(command_names.contains(&"secret wrap-binary"));
        assert!(command_names.contains(&"secret wrap-date"));
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
