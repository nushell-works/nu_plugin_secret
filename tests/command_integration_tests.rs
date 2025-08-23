use nu_plugin::Plugin;
use nu_plugin_secret::SecretString;
use nu_protocol::{Span, Value};

#[cfg(test)]
mod secret_string_functionality_tests {
    use super::*;

    #[test]
    fn test_secret_string_creation_and_display() {
        // Test basic SecretString creation
        let secret = SecretString::new("test-secret".to_string());

        // Test that it displays as redacted
        let display = format!("{}", secret);
        assert!(display.contains("redacted"));
        assert!(!display.contains("test-secret"));
    }

    #[test]
    fn test_secret_string_reveal() {
        // Test that we can reveal the original content
        let secret = SecretString::new("my-api-key".to_string());
        assert_eq!(secret.reveal(), "my-api-key");
    }

    #[test]
    fn test_secret_string_empty() {
        // Test empty string handling
        let secret = SecretString::new("".to_string());
        assert_eq!(secret.reveal(), "");

        let display = format!("{}", secret);
        assert!(display.contains("redacted"));
    }

    #[test]
    fn test_secret_string_special_characters() {
        // Test string with special characters
        let test_string = "password123!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
        let secret = SecretString::new(test_string.to_string());
        assert_eq!(secret.reveal(), test_string);

        // Verify it still shows as redacted
        let display = format!("{}", secret);
        assert!(display.contains("redacted"));
        assert!(!display.contains(test_string));
    }

    #[test]
    fn test_secret_string_unicode() {
        // Test Unicode content
        let test_string = "🔐 secret with émojis and ñoñ-ASCII 中文";
        let secret = SecretString::new(test_string.to_string());
        assert_eq!(secret.reveal(), test_string);

        // Verify redaction works with Unicode
        let display = format!("{}", secret);
        assert!(display.contains("redacted"));
        assert!(!display.contains("🔐"));
        assert!(!display.contains("中文"));
    }

    #[test]
    fn test_secret_string_long_content() {
        // Test very long string
        let test_string = "a".repeat(10000);
        let secret = SecretString::new(test_string.clone());
        assert_eq!(secret.reveal(), &test_string);
        assert_eq!(secret.reveal().len(), 10000);

        // Verify long content is still redacted
        let display = format!("{}", secret);
        assert!(display.contains("redacted"));
        assert!(!display.contains(&test_string));
    }

    #[test]
    fn test_secret_string_debug_format() {
        // Test debug formatting doesn't leak content
        let secret = SecretString::new("debug-secret".to_string());
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("debug-secret"));
        assert!(debug.contains("SecretString") || debug.contains("redacted"));
    }

    #[test]
    fn test_secret_string_clone() {
        // Test cloning preserves secrecy
        let original = SecretString::new("clone-test".to_string());
        let cloned = original.clone();

        assert_eq!(original.reveal(), cloned.reveal());

        let original_display = format!("{}", original);
        let cloned_display = format!("{}", cloned);

        assert!(original_display.contains("redacted"));
        assert!(cloned_display.contains("redacted"));
        assert!(!original_display.contains("clone-test"));
        assert!(!cloned_display.contains("clone-test"));
    }

    #[test]
    fn test_secret_string_equality() {
        // Test equality comparison
        let secret1 = SecretString::new("same-content".to_string());
        let secret2 = SecretString::new("same-content".to_string());
        let secret3 = SecretString::new("different-content".to_string());

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
        assert_ne!(secret2, secret3);
    }

    #[test]
    fn test_secret_string_custom_value_conversion() {
        // Test conversion to Nushell CustomValue
        let secret = SecretString::new("custom-value-test".to_string());
        let custom_value = Value::custom(Box::new(secret), Span::test_data());

        match custom_value {
            Value::Custom { .. } => {
                // Success - we created a custom value
            }
            _ => panic!("Expected Custom value"),
        }
    }

    #[test]
    fn test_multiple_secret_strings() {
        // Test creating multiple secret strings
        let secrets: Vec<SecretString> = (0..100)
            .map(|i| SecretString::new(format!("secret-{}", i)))
            .collect();

        for (i, secret) in secrets.iter().enumerate() {
            assert_eq!(secret.reveal(), &format!("secret-{}", i));
            let display = format!("{}", secret);
            assert!(display.contains("redacted"));
            assert!(!display.contains(&format!("secret-{}", i)));
        }
    }
}

#[cfg(test)]
mod command_functionality_tests {
    use super::*;

    #[test]
    fn test_plugin_has_wrap_string_command() {
        // Test that the plugin includes the wrap-string command
        let plugin = nu_plugin_secret::SecretPlugin;
        let commands = plugin.commands();

        let command_names: Vec<&str> = commands.iter().map(|cmd| cmd.name()).collect();
        assert!(command_names.contains(&"secret wrap-string"));
    }

    #[test]
    fn test_plugin_has_unwrap_command() {
        // Test that the plugin includes the unwrap command
        let plugin = nu_plugin_secret::SecretPlugin;
        let commands = plugin.commands();

        let command_names: Vec<&str> = commands.iter().map(|cmd| cmd.name()).collect();
        assert!(command_names.contains(&"secret unwrap"));
    }

    #[test]
    fn test_wrap_string_command_signature() {
        // Test the wrap-string command signature
        let plugin = nu_plugin_secret::SecretPlugin;
        let commands = plugin.commands();

        let wrap_command = commands
            .iter()
            .find(|cmd| cmd.name() == "secret wrap-string")
            .expect("wrap-string command should exist");

        let signature = wrap_command.signature();
        assert_eq!(signature.name, "secret wrap-string");
        assert_eq!(signature.category, nu_protocol::Category::Conversions);

        // Should have input-output type mapping
        assert!(!signature.input_output_types.is_empty());
    }

    #[test]
    fn test_unwrap_command_signature() {
        // Test the unwrap command signature
        let plugin = nu_plugin_secret::SecretPlugin;
        let commands = plugin.commands();

        let unwrap_command = commands
            .iter()
            .find(|cmd| cmd.name() == "secret unwrap")
            .expect("unwrap command should exist");

        let signature = unwrap_command.signature();
        assert_eq!(signature.name, "secret unwrap");
        assert_eq!(signature.category, nu_protocol::Category::Conversions);

        // Should have multiple input-output type mappings for different secret types
        assert!(signature.input_output_types.len() >= 8); // At least 8 secret types
    }

    #[test]
    fn test_wrap_string_command_description() {
        // Test the wrap-string command description
        let plugin = nu_plugin_secret::SecretPlugin;
        let commands = plugin.commands();

        let wrap_command = commands
            .iter()
            .find(|cmd| cmd.name() == "secret wrap-string")
            .expect("wrap-string command should exist");

        let description = wrap_command.description();
        assert!(!description.is_empty());
        assert!(description.contains("string") || description.contains("String"));
        assert!(description.contains("secret") || description.contains("Secret"));
    }

    #[test]
    fn test_unwrap_command_description() {
        // Test the unwrap command description
        let plugin = nu_plugin_secret::SecretPlugin;
        let commands = plugin.commands();

        let unwrap_command = commands
            .iter()
            .find(|cmd| cmd.name() == "secret unwrap")
            .expect("unwrap command should exist");

        let description = unwrap_command.description();
        assert!(!description.is_empty());
        assert!(description.contains("WARNING") || description.contains("warning"));
        assert!(description.contains("sensitive") || description.contains("expose"));
    }

    #[test]
    fn test_command_examples() {
        // Test that commands have examples
        let plugin = nu_plugin_secret::SecretPlugin;
        let commands = plugin.commands();

        let wrap_command = commands
            .iter()
            .find(|cmd| cmd.name() == "secret wrap-string")
            .expect("wrap-string command should exist");

        let unwrap_command = commands
            .iter()
            .find(|cmd| cmd.name() == "secret unwrap")
            .expect("unwrap command should exist");

        // Both commands should have examples
        assert!(!wrap_command.examples().is_empty());
        assert!(!unwrap_command.examples().is_empty());

        // Wrap examples should not show results (for security)
        for example in wrap_command.examples() {
            if example.result.is_some() {
                // If there's a result, it should be redacted
                let result = example.result.as_ref().unwrap();
                let display = format!("{:?}", result);
                assert!(!display.contains("secret") || display.contains("redacted"));
            }
        }
    }
}

#[cfg(test)]
mod round_trip_tests {
    use super::*;

    #[test]
    fn test_wrap_unwrap_round_trip_concept() {
        // Test the concept of wrap/unwrap round trip
        // This tests the core functionality without plugin system interaction

        let long_string = "x".repeat(1000);
        let original_values = vec![
            "simple-secret",
            "",
            "🔐 unicode secret with émojis 中文",
            "special!@#$%^&*()_+-=[]{}|;':\",./<>?`~chars",
            &long_string, // Long string
        ];

        for original in original_values {
            // Step 1: Wrap as secret
            let secret = SecretString::new(original.to_string());

            // Verify it's properly redacted
            let display = format!("{}", secret);
            assert!(display.contains("redacted"));
            assert!(!display.contains(original) || original.is_empty());

            // Step 2: Unwrap to get original
            let revealed = secret.reveal();
            assert_eq!(revealed, original);

            // Step 3: Verify we can wrap again
            let secret2 = SecretString::new(revealed.to_string());
            assert_eq!(secret2.reveal(), original);

            // Verify both secrets are equal
            assert_eq!(secret, secret2);
        }
    }

    #[test]
    fn test_multiple_round_trips() {
        // Test multiple wrap/unwrap cycles
        let mut current_value = "initial-value".to_string();

        for i in 0..10 {
            // Append iteration number
            current_value = format!("{}-{}", current_value, i);

            // Wrap as secret
            let secret = SecretString::new(current_value.clone());

            // Verify it's redacted
            let display = format!("{}", secret);
            assert!(display.contains("redacted"));
            assert!(!display.contains(&current_value));

            // Unwrap and verify
            let revealed = secret.reveal();
            assert_eq!(revealed, &current_value);
        }
    }

    #[test]
    fn test_concurrent_secrets() {
        // Test handling multiple secrets concurrently
        use std::collections::HashMap;

        let mut secrets = HashMap::new();
        let test_data = vec![
            ("api_key", "sk-1234567890abcdef"),
            ("password", "MyS3cur3P@ssw0rd!"),
            ("token", "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0"),
            ("database_url", "postgresql://user:pass@localhost/db"),
            ("private_key", "-----BEGIN PRIVATE KEY-----\nMIIEvQ..."),
        ];

        // Create all secrets
        for (key, value) in &test_data {
            secrets.insert(*key, SecretString::new(value.to_string()));
        }

        // Verify all secrets are properly redacted
        for (key, secret) in &secrets {
            let display = format!("{}", secret);
            assert!(display.contains("redacted"), "Secret {} not redacted", key);

            let original_value = test_data
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| *v)
                .unwrap();

            assert!(
                !display.contains(original_value),
                "Secret {} leaked in display",
                key
            );
        }

        // Verify all secrets can be revealed correctly
        for (key, secret) in &secrets {
            let original_value = test_data
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| *v)
                .unwrap();

            assert_eq!(
                secret.reveal(),
                original_value,
                "Secret {} revelation failed",
                key
            );
        }
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_secret_string_no_leakage_in_debug() {
        // Test that debug output doesn't leak sensitive content
        let sensitive_data = "super-secret-password-12345";
        let secret = SecretString::new(sensitive_data.to_string());

        let debug_output = format!("{:?}", secret);
        assert!(!debug_output.contains(sensitive_data));
    }

    #[test]
    fn test_secret_string_no_leakage_in_display() {
        // Test that display output doesn't leak sensitive content
        let sensitive_data = "api-key-abcdef123456";
        let secret = SecretString::new(sensitive_data.to_string());

        let display_output = format!("{}", secret);
        assert!(!display_output.contains(sensitive_data));
        assert!(display_output.contains("redacted") || display_output.contains("SECRET"));
    }

    #[test]
    fn test_secret_string_consistent_redaction() {
        // Test that redaction is consistent across multiple calls
        let secret = SecretString::new("consistent-test".to_string());

        let display1 = format!("{}", secret);
        let display2 = format!("{}", secret);
        let debug1 = format!("{:?}", secret);
        let debug2 = format!("{:?}", secret);

        assert_eq!(display1, display2);
        assert_eq!(debug1, debug2);

        // All should be redacted
        assert!(!display1.contains("consistent-test"));
        assert!(!display2.contains("consistent-test"));
        assert!(!debug1.contains("consistent-test"));
        assert!(!debug2.contains("consistent-test"));
    }

    #[test]
    #[cfg(not(miri))] // Exclude from Miri due to SystemTime usage in random function
    fn test_secret_string_memory_safety() {
        // Test that creating and dropping many secrets doesn't cause issues
        for _ in 0..1000 {
            let secret = SecretString::new(format!("secret-{}", rand::random::<u64>() as u32));
            let _ = format!("{}", secret); // Use the secret
                                           // Secret should be safely dropped here
        }
    }

    #[test]
    #[cfg(miri)] // Alternative test for Miri that doesn't use system time
    fn test_secret_string_memory_safety_miri() {
        // Test that creating and dropping many secrets doesn't cause issues (Miri version)
        for i in 0..100 {
            // Reduced iterations for Miri
            let secret = SecretString::new(format!("secret-{}", i));
            let _ = format!("{}", secret); // Use the secret
                                           // Secret should be safely dropped here
        }
    }
}

// Simple random function for testing
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random<T: From<u64>>() -> T {
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        T::from(hasher.finish())
    }
}
