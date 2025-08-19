use nu_plugin::{Plugin, PluginCommand};

mod commands;
mod secret_types;

use commands::*;
pub use secret_types::{SecretString, SecretInt, SecretBool, SecretRecord, SecretList};

pub struct SecretPlugin;

impl Plugin for SecretPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            // Core secret wrap commands
            Box::new(SecretWrapStringCommand),
            Box::new(SecretWrapIntCommand),
            Box::new(SecretWrapBoolCommand),
            Box::new(SecretWrapRecordCommand),
            Box::new(SecretWrapListCommand),
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
        assert_eq!(commands.len(), 9);

        // Test all commands to ensure they're registered correctly
        let command_names: Vec<&str> = commands.iter().map(|cmd| cmd.name()).collect();
        assert!(command_names.contains(&"secret wrap-string"));
        assert!(command_names.contains(&"secret wrap-int"));
        assert!(command_names.contains(&"secret wrap-bool"));
        assert!(command_names.contains(&"secret wrap-record"));
        assert!(command_names.contains(&"secret wrap-list"));
        assert!(command_names.contains(&"secret unwrap"));
        assert!(command_names.contains(&"secret info"));
        assert!(command_names.contains(&"secret validate"));
        assert!(command_names.contains(&"secret type-of"));
    }
}