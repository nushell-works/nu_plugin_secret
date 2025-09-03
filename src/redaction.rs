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
//! - `secret_string`: The actual secret value as a string (WARNING: exposes sensitive data!)
//!
//! Available template functions:
//! - `replicate(character="*", length=5)`: Returns a string of the given character repeated length times.
//!   Returns empty string if length is negative.
//! - `secret_string()`: Returns the actual secret value as a string (WARNING: exposes sensitive data!)
//! - `reverse("text")` or `reverse(s="text")`: Returns the input string reversed
//! - `take(5, "text")` or `take(n=5, s="text")`: Returns the first n characters of the input string

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

    // Register the secret_string function (returns empty string when no value provided)
    tera.register_function(
        "secret_string",
        |_args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            Ok(tera::Value::String("".to_string()))
        },
    );

    // Register the reverse function
    tera.register_function(
        "reverse",
        |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            // Support both positional and named arguments
            let input = if let Some(value) = args.get("0") {
                // First positional argument
                value
                    .as_str()
                    .ok_or_else(|| tera::Error::msg("reverse function argument must be a string"))?
            } else if let Some(value) = args.get("s") {
                // Named argument
                value.as_str().ok_or_else(|| {
                    tera::Error::msg("reverse function 's' parameter must be a string")
                })?
            } else {
                return Err(tera::Error::msg(
                    "reverse function requires a string argument",
                ));
            };

            let reversed: String = input.chars().rev().collect();
            Ok(tera::Value::String(reversed))
        },
    );

    // Register the take function
    tera.register_function(
        "take",
        |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            // Support both positional and named arguments
            let n = if let Some(value) = args.get("0") {
                // First positional argument
                value.as_i64().ok_or_else(|| {
                    tera::Error::msg("take function first argument must be a number")
                })?
            } else if let Some(value) = args.get("n") {
                // Named argument
                value.as_i64().ok_or_else(|| {
                    tera::Error::msg("take function 'n' parameter must be a number")
                })?
            } else {
                return Err(tera::Error::msg(
                    "take function requires first argument to be a number",
                ));
            };

            let s = if let Some(value) = args.get("1") {
                // Second positional argument
                value.as_str().ok_or_else(|| {
                    tera::Error::msg("take function second argument must be a string")
                })?
            } else if let Some(value) = args.get("s") {
                // Named argument
                value.as_str().ok_or_else(|| {
                    tera::Error::msg("take function 's' parameter must be a string")
                })?
            } else {
                return Err(tera::Error::msg(
                    "take function requires second argument to be a string",
                ));
            };

            if n < 0 {
                return Ok(tera::Value::String("".to_string()));
            }

            let chars: Vec<char> = s.chars().collect();
            let taken: String = chars.into_iter().take(n as usize).collect();
            Ok(tera::Value::String(taken))
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

/// Generate redacted string using a custom template with optional length
/// This function allows secrets to use their own redaction template instead of the global one
pub fn generate_redacted_string_with_custom_template(
    secret_type: &str,
    custom_template: &str,
    secret_length: Option<usize>,
) -> String {
    generate_redacted_string_with_custom_template_and_value(
        secret_type,
        custom_template,
        secret_length,
        None,
    )
}

/// Generate redacted string using a custom template with optional length and value
/// This function allows secrets to use their own redaction template instead of the global one
pub fn generate_redacted_string_with_custom_template_and_value(
    secret_type: &str,
    custom_template: &str,
    secret_length: Option<usize>,
    secret_value: Option<String>,
) -> String {
    // Create a fresh Tera instance with the custom template
    let mut tera = tera::Tera::default();

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

    // Capture the secret value for use in the secret_string function
    let captured_secret_value = secret_value.clone();
    tera.register_function(
        "secret_string",
        move |_args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            Ok(tera::Value::String(
                captured_secret_value.clone().unwrap_or_default(),
            ))
        },
    );

    // Register the reverse function
    tera.register_function(
        "reverse",
        |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            // Support both positional and named arguments
            let input = if let Some(value) = args.get("0") {
                // First positional argument
                value
                    .as_str()
                    .ok_or_else(|| tera::Error::msg("reverse function argument must be a string"))?
            } else if let Some(value) = args.get("s") {
                // Named argument
                value.as_str().ok_or_else(|| {
                    tera::Error::msg("reverse function 's' parameter must be a string")
                })?
            } else {
                return Err(tera::Error::msg(
                    "reverse function requires a string argument",
                ));
            };

            let reversed: String = input.chars().rev().collect();
            Ok(tera::Value::String(reversed))
        },
    );

    // Register the take function
    tera.register_function(
        "take",
        |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            // Support both positional and named arguments
            let n = if let Some(value) = args.get("0") {
                // First positional argument
                value.as_i64().ok_or_else(|| {
                    tera::Error::msg("take function first argument must be a number")
                })?
            } else if let Some(value) = args.get("n") {
                // Named argument
                value.as_i64().ok_or_else(|| {
                    tera::Error::msg("take function 'n' parameter must be a number")
                })?
            } else {
                return Err(tera::Error::msg(
                    "take function requires first argument to be a number",
                ));
            };

            let s = if let Some(value) = args.get("1") {
                // Second positional argument
                value.as_str().ok_or_else(|| {
                    tera::Error::msg("take function second argument must be a string")
                })?
            } else if let Some(value) = args.get("s") {
                // Named argument
                value.as_str().ok_or_else(|| {
                    tera::Error::msg("take function 's' parameter must be a string")
                })?
            } else {
                return Err(tera::Error::msg(
                    "take function requires second argument to be a string",
                ));
            };

            if n < 0 {
                return Ok(tera::Value::String("".to_string()));
            }

            let chars: Vec<char> = s.chars().collect();
            let taken: String = chars.into_iter().take(n as usize).collect();
            Ok(tera::Value::String(taken))
        },
    );

    if tera
        .add_raw_template(TEMPLATE_NAME, custom_template)
        .is_err()
    {
        // If template adding fails, fall back to simple format
        return format!("<redacted:{}>", secret_type);
    }

    let mut context = tera::Context::new();
    context.insert("secret_type", secret_type);
    if let Some(length) = secret_length {
        context.insert("secret_length", &length);
    }
    if let Some(value) = &secret_value {
        context.insert("secret_string", value);
    }

    // Use Tera to render the template, fallback to format if it fails
    tera.render(TEMPLATE_NAME, &context)
        .unwrap_or_else(|_| format!("<redacted:{}>", secret_type))
}

/// Get redacted string using a custom template with optional value and length
pub fn get_redacted_string_with_custom_template_and_value<T: std::fmt::Display + ?Sized>(
    secret_type: &str,
    custom_template: &str,
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

    // Calculate length and string value if we have a value
    let (secret_length, secret_string_value) = if let Some(value) = actual_value {
        let string_value = value.to_string();
        (Some(string_value.len()), Some(string_value))
    } else {
        (None, None)
    };

    // Return redacted string using custom template with length and value
    generate_redacted_string_with_custom_template_and_value(
        secret_type,
        custom_template,
        secret_length,
        secret_string_value,
    )
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

    #[test]
    fn test_generate_redacted_string_with_custom_template() {
        // Test custom template with basic replacement
        let result = generate_redacted_string_with_custom_template(
            "password",
            "[HIDDEN:{{secret_type}}]",
            None,
        );
        assert_eq!(result, "[HIDDEN:password]");

        // Test custom template with length
        let result = generate_redacted_string_with_custom_template(
            "password",
            "{{secret_type}}({{secret_length}})",
            Some(8),
        );
        assert_eq!(result, "password(8)");

        // Test template with replicate function
        let result = generate_redacted_string_with_custom_template(
            "secret",
            "{{replicate(character='*', length=5)}}",
            None,
        );
        assert_eq!(result, "*****");
    }

    #[test]
    fn test_get_redacted_string_with_custom_template_and_value() {
        use crate::config::RedactionContext;

        // Test custom template with value
        let result = get_redacted_string_with_custom_template_and_value(
            "password",
            "[SECRET:{{secret_type}}]",
            RedactionContext::Display,
            Some(&"test123"),
        );
        assert_eq!(result, "[SECRET:password]");

        // Test custom template with length calculated from value
        let result = get_redacted_string_with_custom_template_and_value(
            "password",
            "{{replicate(character='*', length=secret_length)}}",
            RedactionContext::Display,
            Some(&"test123"),
        );
        assert_eq!(result, "*******"); // "test123" has 7 characters

        // Test custom template with secret_string variable
        let result = get_redacted_string_with_custom_template_and_value(
            "string",
            "moo:{{secret_string}}",
            RedactionContext::Display,
            Some(&"abcdf"),
        );
        assert_eq!(result, "moo:abcdf");

        // Test template that should definitely work
        let result = get_redacted_string_with_custom_template_and_value(
            "string",
            "simple_test",
            RedactionContext::Display,
            Some(&"abcdf"),
        );
        assert_eq!(result, "simple_test");
    }

    #[test]
    fn test_reverse_function() {
        // Test reverse function using existing redaction template system
        let result =
            generate_redacted_string_with_custom_template("test", "{{reverse(s='hello')}}", None);
        assert_eq!(result, "olleh");

        let result =
            generate_redacted_string_with_custom_template("test", "{{reverse(s='world')}}", None);
        assert_eq!(result, "dlrow");

        // Test with empty string
        let result =
            generate_redacted_string_with_custom_template("test", "{{reverse(s='')}}", None);
        assert_eq!(result, "");

        // Test with unicode characters
        let result =
            generate_redacted_string_with_custom_template("test", "{{reverse(s='🚀🎉')}}", None);
        assert_eq!(result, "🎉🚀");
    }

    #[test]
    fn test_reverse_function_error_handling() {
        // Test with no arguments - should fall back to basic format
        let result =
            generate_redacted_string_with_custom_template("test_type", "{{reverse()}}", None);
        assert_eq!(result, "<redacted:test_type>");

        // Test with invalid template - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "test_type",
            "{{reverse(s=nonexistent_var)}}",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");
    }

    #[test]
    fn test_take_function() {
        // Test take function using existing redaction template system
        let result = generate_redacted_string_with_custom_template(
            "test",
            "{{take(n=3, s='hello world')}}",
            None,
        );
        assert_eq!(result, "hel");

        let result = generate_redacted_string_with_custom_template(
            "test",
            "{{take(n=5, s='testing')}}",
            None,
        );
        assert_eq!(result, "testi");

        // Test taking more characters than available
        let result = generate_redacted_string_with_custom_template(
            "test",
            "{{take(n=10, s='short')}}",
            None,
        );
        assert_eq!(result, "short");

        // Test with zero characters
        let result = generate_redacted_string_with_custom_template(
            "test",
            "{{take(n=0, s='anything')}}",
            None,
        );
        assert_eq!(result, "");

        // Test with negative number
        let result =
            generate_redacted_string_with_custom_template("test", "{{take(n=-1, s='test')}}", None);
        assert_eq!(result, "");

        // Test with unicode characters
        let result = generate_redacted_string_with_custom_template(
            "test",
            "{{take(n=2, s='🚀🎉🌟⭐')}}",
            None,
        );
        assert_eq!(result, "🚀🎉");

        // Test with empty string
        let result =
            generate_redacted_string_with_custom_template("test", "{{take(n=5, s='')}}", None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_take_function_error_handling() {
        // Test with missing arguments - should fall back to basic format
        let result =
            generate_redacted_string_with_custom_template("test_type", "{{take(s='test')}}", None);
        assert_eq!(result, "<redacted:test_type>");

        let result =
            generate_redacted_string_with_custom_template("test_type", "{{take(n=5)}}", None);
        assert_eq!(result, "<redacted:test_type>");

        // Test with invalid arguments - should fall back to basic format
        let result = generate_redacted_string_with_custom_template("test_type", "{{take()}}", None);
        assert_eq!(result, "<redacted:test_type>");
    }

    #[test]
    fn test_secret_string_function() {
        // Test secret_string function in regular templating (should return empty)
        let result =
            generate_redacted_string_with_custom_template("test", "{{secret_string()}}", None);
        assert_eq!(result, "");

        // Test secret_string function with custom template and value (should return the value)
        let result = generate_redacted_string_with_custom_template_and_value(
            "test",
            "{{secret_string()}}",
            None,
            Some("secret_value".to_string()),
        );
        assert_eq!(result, "secret_value");

        // Test secret_string function with template containing other content
        let result = generate_redacted_string_with_custom_template_and_value(
            "test",
            "prefix:{{secret_string()}}:suffix",
            None,
            Some("middle".to_string()),
        );
        assert_eq!(result, "prefix:middle:suffix");

        // Test secret_string function with no secret value provided
        let result = generate_redacted_string_with_custom_template_and_value(
            "test",
            "value:{{secret_string()}}",
            None,
            None,
        );
        assert_eq!(result, "value:");
    }

    #[test]
    fn test_template_error_fallback_handling() {
        // Test invalid template syntax - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "test_type",
            "{{invalid syntax without closing",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");

        // Test template with undefined variable - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "test_type",
            "{{undefined_variable}}",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");

        // Test template with invalid function call - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "test_type",
            "{{nonexistent_function()}}",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");
    }

    #[test]
    fn test_replicate_function_error_handling() {
        // Test with missing parameters - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "test_type",
            "{{replicate(length=5)}}",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");

        let result = generate_redacted_string_with_custom_template(
            "test_type",
            "{{replicate(character='*')}}",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");

        // Test with no parameters - should fall back to basic format
        let result =
            generate_redacted_string_with_custom_template("test_type", "{{replicate()}}", None);
        assert_eq!(result, "<redacted:test_type>");

        // Test with empty character string (should fall back to '*')
        let result = generate_redacted_string_with_custom_template(
            "test",
            "{{replicate(character='', length=3)}}",
            None,
        );
        assert_eq!(result, "***");
    }

    #[test]
    fn test_complex_template_combinations() {
        // Test combining replicate with secret length
        let result = generate_redacted_string_with_custom_template_and_value(
            "secret",
            "[{{replicate(character='-', length=secret_length)}}]",
            Some(4), // Explicitly set the length
            Some("test".to_string()),
        );
        assert_eq!(result, "[----]"); // "test" has 4 characters

        // Test template with secret_type and secret_length variables
        let result = generate_redacted_string_with_custom_template_and_value(
            "complex",
            "{{secret_type}}:{{secret_length}}",
            Some(6), // Explicitly set the length
            Some("abcdef".to_string()),
        );
        assert_eq!(result, "complex:6");

        // Test template with secret_string function
        let result = generate_redacted_string_with_custom_template_and_value(
            "test",
            "value={{secret_string()}}",
            None,
            Some("secret123".to_string()),
        );
        assert_eq!(result, "value=secret123");

        // Test combining reverse with secret_string
        let result = generate_redacted_string_with_custom_template_and_value(
            "test",
            "{{reverse(s=secret_string)}}",
            None,
            Some("hello".to_string()),
        );
        assert_eq!(result, "olleh");

        // Test combining take with secret_string
        let result = generate_redacted_string_with_custom_template_and_value(
            "test",
            "{{take(n=3, s=secret_string)}}",
            None,
            Some("abcdef".to_string()),
        );
        assert_eq!(result, "abc");
    }
}
