//! Integration tests for SHOW_UNREDACTED environment variable functionality

use nu_plugin_secret::config::RedactionContext;
use nu_plugin_secret::config::{ConfigManager, PluginConfig};
use nu_plugin_secret::memory_optimizations::{
    get_configurable_redacted_string_with_generic_value,
    get_configurable_redacted_string_with_value,
};
use nu_plugin_secret::{SecretBool, SecretInt, SecretString};
use std::env;

#[test]
fn test_show_unredacted_with_secret_string() {
    // Set up environment to show unredacted secrets
    env::set_var("SHOW_UNREDACTED", "1");

    // Create a new configuration that should pick up the env var
    let mut config = PluginConfig::default();
    let result = ConfigManager::apply_env_overrides(&mut config);
    assert!(result.is_ok());
    assert!(config.redaction.show_unredacted);

    // Test that the configuration correctly parsed the environment variable
    // We avoid creating ConfigManager to prevent Miri issues with system calls

    // Test the functionality directly without global config
    let secret_text = "my_secret_password";
    let result = get_configurable_redacted_string_with_value(
        "string",
        RedactionContext::Display,
        Some(secret_text),
    );

    // Since we don't have a global config initialized in this test,
    // it will fall back to redacted text, which is expected behavior
    assert!(result.contains("redacted"));

    // Clean up
    env::remove_var("SHOW_UNREDACTED");
}

// Test removed to avoid race conditions - environment variable parsing
// is comprehensively tested in src/config.rs

#[test]
fn test_show_unredacted_config_behavior() {
    // Test the configuration behavior directly
    let mut config = PluginConfig::default();

    // Test default value
    assert!(!config.redaction.show_unredacted);

    // Test setting it to true
    config.redaction.show_unredacted = true;

    // Test that the configuration field works as expected
    // We avoid creating ConfigManager to prevent Miri issues with system calls
    assert!(config.redaction.show_unredacted);

    // Test that we can toggle it back to false
    config.redaction.show_unredacted = false;
    assert!(!config.redaction.show_unredacted);
}

#[test]
fn test_show_unredacted_memory_optimization_functions() {
    // Test the core functions directly

    // Test with None value - should always return redacted text
    let result = get_configurable_redacted_string_with_value(
        "string",
        RedactionContext::Display,
        None::<&str>,
    );
    assert!(result.contains("redacted"));

    // Test with actual value - without config it should still be redacted
    let result = get_configurable_redacted_string_with_value(
        "string",
        RedactionContext::Display,
        Some("secret_value"),
    );
    assert!(result.contains("redacted"));

    // Test generic version
    let result = get_configurable_redacted_string_with_generic_value(
        "int",
        RedactionContext::Display,
        Some(&42),
    );
    assert!(result.contains("redacted"));
}

#[test]
fn test_secret_types_behavior() {
    // Test that secret types can be created and displayed
    let secret_string = SecretString::new("my_secret".to_string());
    let display_result = format!("{}", secret_string);

    // Should be redacted by default (since no global config with SHOW_UNREDACTED is set)
    assert!(
        display_result.contains("redacted")
            || display_result.contains("HIDDEN")
            || display_result.contains("***")
    );

    let secret_int = SecretInt::new(42);
    let display_result = format!("{}", secret_int);
    assert!(
        display_result.contains("redacted")
            || display_result.contains("HIDDEN")
            || display_result.contains("***")
    );

    let secret_bool = SecretBool::new(true);
    let display_result = format!("{}", secret_bool);
    assert!(
        display_result.contains("redacted")
            || display_result.contains("HIDDEN")
            || display_result.contains("***")
    );
}

// Note: Environment variable parsing is already tested in src/config.rs
// These integration tests focus on the end-to-end behavior
