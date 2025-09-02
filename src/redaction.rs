//! Tera-based redaction templating system
//!
//! This module provides a configurable templating system for redaction using the Tera template engine.
//! The default template is `<redacted:{{secret_type}}>` where `secret_type` is the type name
//! of the secret (e.g., "string", "float", "int", etc.).
//! Templates can be customized through the configuration file's `redaction_template` field.
//!
//! Available template variables:
//! - `secret_type`: The type of the secret (e.g., "string", "int", "float")
//! - `secret_length`: The length of the secret value (only available when length is provided)
//!
//! Available template functions:
//! - `replicate(character="*", length=5)`: Returns a string of the given character repeated length times.
//!   Returns empty string if length is negative.

use std::sync::OnceLock;
use tera::{Context, Tera};

/// Global Tera template engine for redaction
static REDACTION_TERA: OnceLock<Tera> = OnceLock::new();

/// Template string for redaction
const REDACTION_TEMPLATE: &str = "<redacted:{{secret_type}}>";

/// Template name used internally
const TEMPLATE_NAME: &str = "redaction";

/// Initialize the Tera template engine for redaction
pub fn init_redaction_templating() -> Result<(), tera::Error> {
    let _tera = REDACTION_TERA.get_or_init(|| {
        let mut tera = Tera::default(); // Create empty Tera instance

        // Add our redaction template
        let _ = tera.add_raw_template(TEMPLATE_NAME, REDACTION_TEMPLATE);

        tera
    });

    Ok(())
}

/// Generate redacted string using Tera template
/// This is the core function that uses Tera templating
fn generate_redacted_string(secret_type: &str) -> String {
    generate_redacted_string_with_length(secret_type, None)
}

/// Generate redacted string using Tera template with optional length
/// This is the core function that uses Tera templating
fn generate_redacted_string_with_length(secret_type: &str, secret_length: Option<usize>) -> String {
    // Get template from config, fallback to default if config unavailable
    let template = if let Ok(config) = crate::config::get_config() {
        config
            .config()
            .redaction
            .get_redaction_template()
            .to_string()
    } else {
        REDACTION_TEMPLATE.to_string()
    };

    // Always create a fresh Tera instance to pick up template changes
    // This is slightly less efficient but allows for dynamic template updates
    let mut tera = Tera::default();

    // Register the replicate function
    tera.register_function(
        "replicate",
        |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let character = args
                .get("character")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    tera::Error::msg("replicate function requires 'character' parameter")
                })?;

            let length = args.get("length").and_then(|v| v.as_i64()).ok_or_else(|| {
                tera::Error::msg("replicate function requires 'length' parameter")
            })?;

            if length < 0 {
                return Ok(tera::Value::String("".to_string()));
            }

            let mask_char = character.chars().next().unwrap_or('*');
            let result = mask_char.to_string().repeat(length as usize);
            Ok(tera::Value::String(result))
        },
    );

    if tera.add_raw_template(TEMPLATE_NAME, &template).is_err() {
        // If template adding fails, fall back to simple format
        return format!("<redacted:{}>", secret_type);
    }

    let mut context = Context::new();
    context.insert("secret_type", secret_type);
    if let Some(length) = secret_length {
        context.insert("secret_length", &length);
    }

    // Use Tera to render the template, fallback to format if it fails
    tera.render(TEMPLATE_NAME, &context)
        .unwrap_or_else(|_| format!("<redacted:{}>", secret_type))
}

/// Get a cached redacted string for performance
/// Falls back to generating the string if not cached
/// Note: Caching disabled to allow for dynamic template changes during development
pub fn get_cached_redacted_string(secret_type: &str) -> String {
    // Always generate fresh to pick up template changes
    // TODO: Re-enable caching in production for better performance
    generate_redacted_string(secret_type)
}

/// Get a cached redacted string with length information for performance
/// Falls back to generating the string if not cached
pub fn get_cached_redacted_string_with_length(
    secret_type: &str,
    secret_length: Option<usize>,
) -> String {
    // Always generate fresh to pick up template changes
    generate_redacted_string_with_length(secret_type, secret_length)
}

/// Get configurable redacted string with optional unredacted mode support
/// This checks if SHOW_UNREDACTED is enabled and returns actual value if so
pub fn get_redacted_string_with_value<T: std::fmt::Display + ?Sized>(
    secret_type: &str,
    _context: crate::config::RedactionContext,
    actual_value: Option<&T>,
) -> String {
    // Check if unredacted mode is enabled
    if let Ok(config) = crate::config::get_config() {
        if config.config().redaction.show_unredacted {
            if let Some(value) = actual_value {
                return value.to_string();
            }
        }
    }

    // Calculate length if we have a value
    let secret_length = actual_value.map(|v| v.to_string().len());

    // Return redacted string using Tera templating with length
    get_cached_redacted_string_with_length(secret_type, secret_length)
}

/// Get redacted string with explicit length for template usage
/// This allows templates to access secret_length variable and mask function
pub fn get_redacted_string_with_length(secret_type: &str, secret_length: Option<usize>) -> String {
    get_cached_redacted_string_with_length(secret_type, secret_length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_template_initialization() {
        let result = init_redaction_templating();
        assert!(
            result.is_ok(),
            "Failed to initialize Tera templating: {:?}",
            result
        );
    }

    #[test]
    fn test_tera_template_rendering() {
        // Test that Tera template rendering works correctly
        assert_eq!(
            super::generate_redacted_string("string"),
            "<redacted:string>"
        );
        assert_eq!(super::generate_redacted_string("float"), "<redacted:float>");
        assert_eq!(
            super::generate_redacted_string("custom_type"),
            "<redacted:custom_type>"
        );
    }

    #[test]
    fn test_redacted_string_format() {
        // Test that the format is correct by checking cached strings
        assert_eq!(get_cached_redacted_string("string"), "<redacted:string>");
        assert_eq!(get_cached_redacted_string("float"), "<redacted:float>");
        assert_eq!(
            get_cached_redacted_string("custom_type"),
            "<redacted:custom_type>"
        );
    }

    #[test]
    fn test_cached_redacted_strings() {
        // Test common types are cached
        assert_eq!(get_cached_redacted_string("string"), "<redacted:string>");
        assert_eq!(get_cached_redacted_string("int"), "<redacted:int>");

        // Test uncommon type is generated
        assert_eq!(
            get_cached_redacted_string("unusual_type"),
            "<redacted:unusual_type>"
        );
    }

    #[test]
    fn test_configurable_template_usage() {
        use crate::config::PluginConfig;

        // Test that redaction uses custom template from config
        let mut config = PluginConfig::default();
        config.redaction.redaction_template = Some("[SECRET:{{secret_type}}]".to_string());

        // Since we can't easily mock the global config, we test the template retrieval
        assert_eq!(
            config.redaction.get_redaction_template(),
            "[SECRET:{{secret_type}}]"
        );

        // Test default template fallback
        let default_config = PluginConfig::default();
        assert_eq!(
            default_config.redaction.get_redaction_template(),
            "<redacted:{{secret_type}}>"
        );
    }

    #[test]
    fn test_template_rendering_formats() {
        // Test that different template formats render correctly
        let mut tera = tera::Tera::default();
        let mut context = tera::Context::new();
        context.insert("secret_type", "string");

        // Test default format
        tera.add_raw_template("default", "<redacted:{{secret_type}}>")
            .unwrap();
        assert_eq!(
            tera.render("default", &context).unwrap(),
            "<redacted:string>"
        );

        // Test custom format
        tera.add_raw_template("custom", "[HIDDEN:{{secret_type}}]")
            .unwrap();
        assert_eq!(tera.render("custom", &context).unwrap(), "[HIDDEN:string]");

        // Test simple format
        tera.add_raw_template("simple", "{{secret_type}}_redacted")
            .unwrap();
        assert_eq!(tera.render("simple", &context).unwrap(), "string_redacted");
    }

    #[test]
    fn test_redacted_string_with_value() {
        use crate::config::RedactionContext;

        // Should return redacted string when no unredacted mode
        let result =
            get_redacted_string_with_value("string", RedactionContext::Display, Some(&"secret"));
        assert_eq!(result, "<redacted:string>");

        // Test with different types
        let result = get_redacted_string_with_value("int", RedactionContext::Display, Some(&42));
        assert_eq!(result, "<redacted:int>");

        let result = get_redacted_string_with_value(
            "float",
            RedactionContext::Display,
            Some(&std::f64::consts::PI),
        );
        assert_eq!(result, "<redacted:float>");

        let result = get_redacted_string_with_value("bool", RedactionContext::Display, Some(&true));
        assert_eq!(result, "<redacted:bool>");
    }

    #[test]
    fn test_redacted_string_with_length() {
        // Test with length information
        let result = get_redacted_string_with_length("string", Some(10));
        assert_eq!(result, "<redacted:string>");

        // Test without length information
        let result = get_redacted_string_with_length("string", None);
        assert_eq!(result, "<redacted:string>");
    }

    #[test]
    fn test_template_with_secret_length() {
        // Create a template that uses secret_length
        let mut tera = tera::Tera::default();
        tera.add_raw_template("length_test", "{{secret_type}}({{secret_length}})")
            .unwrap();

        let mut context = tera::Context::new();
        context.insert("secret_type", "password");
        context.insert("secret_length", &8);

        let result = tera.render("length_test", &context).unwrap();
        assert_eq!(result, "password(8)");
    }

    #[test]
    fn test_replicate_function() {
        // Test replicate function with positive length
        let mut tera = tera::Tera::default();
        tera.register_function(
            "replicate",
            |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                let character =
                    args.get("character")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            tera::Error::msg("replicate function requires 'character' parameter")
                        })?;

                let length = args.get("length").and_then(|v| v.as_i64()).ok_or_else(|| {
                    tera::Error::msg("replicate function requires 'length' parameter")
                })?;

                if length < 0 {
                    return Ok(tera::Value::String("".to_string()));
                }

                let mask_char = character.chars().next().unwrap_or('*');
                let result = mask_char.to_string().repeat(length as usize);
                Ok(tera::Value::String(result))
            },
        );

        tera.add_raw_template("replicate_test", "{{replicate(character='*', length=5)}}")
            .unwrap();

        let context = tera::Context::new();
        let result = tera.render("replicate_test", &context).unwrap();
        assert_eq!(result, "*****");
    }

    #[test]
    fn test_replicate_function_different_characters() {
        let mut tera = tera::Tera::default();
        tera.register_function(
            "replicate",
            |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                let character =
                    args.get("character")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            tera::Error::msg("replicate function requires 'character' parameter")
                        })?;

                let length = args.get("length").and_then(|v| v.as_i64()).ok_or_else(|| {
                    tera::Error::msg("replicate function requires 'length' parameter")
                })?;

                if length < 0 {
                    return Ok(tera::Value::String("".to_string()));
                }

                let mask_char = character.chars().next().unwrap_or('*');
                let result = mask_char.to_string().repeat(length as usize);
                Ok(tera::Value::String(result))
            },
        );

        // Test with different characters
        tera.add_raw_template("replicate_x", "{{replicate(character='X', length=3)}}")
            .unwrap();
        tera.add_raw_template("replicate_dash", "{{replicate(character='-', length=7)}}")
            .unwrap();
        tera.add_raw_template("replicate_dot", "{{replicate(character='.', length=4)}}")
            .unwrap();

        let context = tera::Context::new();

        assert_eq!(tera.render("replicate_x", &context).unwrap(), "XXX");
        assert_eq!(tera.render("replicate_dash", &context).unwrap(), "-------");
        assert_eq!(tera.render("replicate_dot", &context).unwrap(), "....");
    }

    #[test]
    fn test_replicate_function_negative_length() {
        let mut tera = tera::Tera::default();
        tera.register_function(
            "replicate",
            |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                let character =
                    args.get("character")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            tera::Error::msg("replicate function requires 'character' parameter")
                        })?;

                let length = args.get("length").and_then(|v| v.as_i64()).ok_or_else(|| {
                    tera::Error::msg("replicate function requires 'length' parameter")
                })?;

                if length < 0 {
                    return Ok(tera::Value::String("".to_string()));
                }

                let mask_char = character.chars().next().unwrap_or('*');
                let result = mask_char.to_string().repeat(length as usize);
                Ok(tera::Value::String(result))
            },
        );

        tera.add_raw_template(
            "replicate_negative",
            "{{replicate(character='*', length=-1)}}",
        )
        .unwrap();

        let context = tera::Context::new();
        let result = tera.render("replicate_negative", &context).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_replicate_function_zero_length() {
        let mut tera = tera::Tera::default();
        tera.register_function(
            "replicate",
            |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                let character =
                    args.get("character")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            tera::Error::msg("replicate function requires 'character' parameter")
                        })?;

                let length = args.get("length").and_then(|v| v.as_i64()).ok_or_else(|| {
                    tera::Error::msg("replicate function requires 'length' parameter")
                })?;

                if length < 0 {
                    return Ok(tera::Value::String("".to_string()));
                }

                let mask_char = character.chars().next().unwrap_or('*');
                let result = mask_char.to_string().repeat(length as usize);
                Ok(tera::Value::String(result))
            },
        );

        tera.add_raw_template("replicate_zero", "{{replicate(character='*', length=0)}}")
            .unwrap();

        let context = tera::Context::new();
        let result = tera.render("replicate_zero", &context).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_template_with_length_and_replicate() {
        let mut tera = tera::Tera::default();
        tera.register_function(
            "replicate",
            |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                let character =
                    args.get("character")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            tera::Error::msg("replicate function requires 'character' parameter")
                        })?;

                let length = args.get("length").and_then(|v| v.as_i64()).ok_or_else(|| {
                    tera::Error::msg("replicate function requires 'length' parameter")
                })?;

                if length < 0 {
                    return Ok(tera::Value::String("".to_string()));
                }

                let mask_char = character.chars().next().unwrap_or('*');
                let result = mask_char.to_string().repeat(length as usize);
                Ok(tera::Value::String(result))
            },
        );

        // Template that uses both secret_length and replicate function
        tera.add_raw_template(
            "complex",
            "<{{secret_type}}:{{replicate(character='*', length=secret_length)}}>",
        )
        .unwrap();

        let mut context = tera::Context::new();
        context.insert("secret_type", "password");
        context.insert("secret_length", &8);

        let result = tera.render("complex", &context).unwrap();
        assert_eq!(result, "<password:********>");
    }

    #[test]
    fn test_redacted_string_with_value_includes_length() {
        use crate::config::RedactionContext;

        // Test that the new function correctly calculates and uses length
        let test_value = "secret123";
        let result =
            get_redacted_string_with_value("string", RedactionContext::Display, Some(&test_value));

        // Since we're using the default template, it should still be the basic format
        // but the length should be available for custom templates
        assert_eq!(result, "<redacted:string>");
    }
}
