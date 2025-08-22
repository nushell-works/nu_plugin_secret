use nu_plugin::{Plugin, PluginCommand};

mod commands;
pub mod memory_optimizations;
pub mod performance_monitoring;
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
        // Initialize optimizations on first command access
        startup_optimizations::command_optimizations::init_command_cache();

        vec![
            // Core secret wrap commands
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
            Box::new(SecretInfoCommand),
            Box::new(SecretValidateCommand),
            Box::new(SecretTypeOfCommand),
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
        assert_eq!(commands.len(), 12);

        // Test all commands to ensure they're registered correctly
        let command_names: Vec<&str> = commands.iter().map(|cmd| cmd.name()).collect();
        assert!(command_names.contains(&"secret wrap-string"));
        assert!(command_names.contains(&"secret wrap-int"));
        assert!(command_names.contains(&"secret wrap-bool"));
        assert!(command_names.contains(&"secret wrap-record"));
        assert!(command_names.contains(&"secret wrap-list"));
        assert!(command_names.contains(&"secret wrap-float"));
        assert!(command_names.contains(&"secret wrap-binary"));
        assert!(command_names.contains(&"secret wrap-date"));
        assert!(command_names.contains(&"secret unwrap"));
        assert!(command_names.contains(&"secret info"));
        assert!(command_names.contains(&"secret validate"));
        assert!(command_names.contains(&"secret type-of"));
    }
}
