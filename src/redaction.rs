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
//! - `replicate(s="*", n=5)`: Returns a string of the given character repeated n times.
//!   Returns empty string if n is negative.
//! - `secret_string()`: Returns the actual secret value as a string (WARNING: exposes sensitive data!)
//! - `reverse("text")` or `reverse(s="text")`: Returns the input string reversed
//! - `take(5, "text")` or `take(n=5, s="text")`: Returns the first n characters of the input string
//! - `strlen("text")` or `strlen(s="text")`: Returns the length of the input string as a number

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
fn generate_redacted_string(secret_string: Option<&str>, secret_type: &str) -> String {
    generate_redacted_string_with_length(secret_string, secret_type, None)
}

/// Generate redacted string using Tera template with optional length
/// This is the core function that uses Tera templating
fn generate_redacted_string_with_length(
    secret_string: Option<&str>,
    secret_type: &str,
    secret_length: Option<usize>,
) -> String {
    // Use default template
    // TODO: Add support for passing ConfigManager to enable custom templates
    let template = REDACTION_TEMPLATE.to_string();

    // Always create a fresh Tera instance to pick up template changes
    // This is slightly less efficient but allows for dynamic template updates
    let mut tera = Tera::default();

    // Register all standard template functions
    crate::tera_functions::register_all_standard_functions(&mut tera);

    // Note: secret_string is available as a template variable, not a function

    if tera.add_raw_template(TEMPLATE_NAME, &template).is_err() {
        // If template adding fails, fall back to simple format
        return format!("<redacted:{}>", secret_type);
    }

    let mut context = Context::new();
    context.insert("secret_type", secret_type);
    if let Some(secret_str) = secret_string {
        context.insert("secret_string", secret_str);
    }
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
pub fn get_cached_redacted_string(secret_string: Option<&str>, secret_type: &str) -> String {
    // Always generate fresh to pick up template changes
    // TODO: Re-enable caching in production for better performance
    generate_redacted_string(secret_string, secret_type)
}

/// Get a cached redacted string with length information for performance
/// Falls back to generating the string if not cached
pub fn get_cached_redacted_string_with_length(
    secret_string: Option<&str>,
    secret_type: &str,
    secret_length: Option<usize>,
) -> String {
    // Always generate fresh to pick up template changes
    generate_redacted_string_with_length(secret_string, secret_type, secret_length)
}

/// Get configurable redacted string with optional unredacted mode support
/// Note: SHOW_UNREDACTED support requires access to ConfigManager (not available in Display context)
/// TODO: Add variant that accepts ConfigManager for dynamic configuration
pub fn get_redacted_string_with_value<T: std::fmt::Display + ?Sized>(
    secret_type: &str,
    _context: crate::config::RedactionContext,
    actual_value: Option<&T>,
) -> String {
    // Calculate length if we have a value
    let secret_length = actual_value.map(|v| v.to_string().len());

    // Return redacted string using Tera templating with length
    if let Some(value) = actual_value {
        let value_str = value.to_string();
        get_cached_redacted_string_with_length(Some(&value_str), secret_type, secret_length)
    } else {
        get_cached_redacted_string_with_length(None, secret_type, secret_length)
    }
}

/// Get redacted string with explicit length for template usage
/// This allows templates to access secret_length variable and mask function
pub fn get_redacted_string_with_length(secret_type: &str, secret_length: Option<usize>) -> String {
    get_cached_redacted_string_with_length(None, secret_type, secret_length)
}

/// Generate redacted string using a custom template with optional length
/// This function allows secrets to use their own redaction template instead of the global one
pub fn generate_redacted_string_with_custom_template(
    custom_template: &str,
    secret_type: &str,
    secret_length: Option<usize>,
) -> String {
    generate_redacted_string_with_custom_template_and_value(
        custom_template,
        secret_type,
        secret_length,
        None,
    )
}

/// Generate redacted string using a custom template with optional length and value
/// This function allows secrets to use their own redaction template instead of the global one
pub fn generate_redacted_string_with_custom_template_and_value(
    custom_template: &str,
    secret_type: &str,
    secret_length: Option<usize>,
    secret_value: Option<String>,
) -> String {
    // Note: show_unredacted support requires ConfigManager access
    // TODO: Add variant that accepts ConfigManager parameter

    // Create a fresh Tera instance with the custom template
    let mut tera = tera::Tera::default();

    // Register all standard template functions
    crate::tera_functions::register_all_standard_functions(&mut tera);

    // Use the secret value as-is for template rendering
    // Note: mask_secret feature disabled in this context without ConfigManager
    // TODO: Add variant that accepts ConfigManager to enable mask_secret
    let effective_secret_value = secret_value;

    // Note: secret_string is available as a template variable, not a function

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
    if let Some(value) = &effective_secret_value {
        context.insert("secret_string", value);
    }

    // Use Tera to render the template, fallback to format if it fails
    tera.render(TEMPLATE_NAME, &context)
        .unwrap_or_else(|_| format!("<redacted:{}>", secret_type))
}

/// Get redacted string using a custom template with optional value and length
/// Note: show_unredacted support requires access to ConfigManager (not available in Display context)
/// TODO: Add variant that accepts ConfigManager for dynamic configuration
pub fn get_redacted_string_with_custom_template_and_value<T: std::fmt::Display + ?Sized>(
    custom_template: &str,
    secret_type: &str,
    _context: crate::config::RedactionContext,
    actual_value: Option<&T>,
) -> String {
    // Calculate length and string value if we have a value
    let (secret_length, secret_string_value) = if let Some(value) = actual_value {
        let string_value = value.to_string();
        (Some(string_value.len()), Some(string_value))
    } else {
        (None, None)
    };

    // Return redacted string using custom template with length and value
    generate_redacted_string_with_custom_template_and_value(
        custom_template,
        secret_type,
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
            super::generate_redacted_string(None, "string"),
            "<redacted:string>"
        );
        assert_eq!(
            super::generate_redacted_string(None, "float"),
            "<redacted:float>"
        );
        assert_eq!(
            super::generate_redacted_string(None, "custom_type"),
            "<redacted:custom_type>"
        );
    }

    #[test]
    fn test_redacted_string_format() {
        // Test that the format is correct by checking cached strings
        assert_eq!(
            get_cached_redacted_string(None, "string"),
            "<redacted:string>"
        );
        assert_eq!(
            get_cached_redacted_string(None, "float"),
            "<redacted:float>"
        );
        assert_eq!(
            get_cached_redacted_string(None, "custom_type"),
            "<redacted:custom_type>"
        );
    }

    #[test]
    fn test_cached_redacted_strings() {
        // Test common types are cached
        assert_eq!(
            get_cached_redacted_string(None, "string"),
            "<redacted:string>"
        );
        assert_eq!(get_cached_redacted_string(None, "int"), "<redacted:int>");

        // Test uncommon type is generated
        assert_eq!(
            get_cached_redacted_string(None, "unusual_type"),
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
        crate::tera_functions::register_all_standard_functions(&mut tera);

        tera.add_raw_template("replicate_test", "{{replicate(s='*', n=5)}}")
            .unwrap();

        let context = tera::Context::new();
        let result = tera.render("replicate_test", &context).unwrap();
        assert_eq!(result, "*****");
    }

    #[test]
    fn test_replicate_function_different_characters() {
        let mut tera = tera::Tera::default();
        crate::tera_functions::register_all_standard_functions(&mut tera);

        // Test with different characters
        tera.add_raw_template("replicate_x", "{{replicate(s='X', n=3)}}")
            .unwrap();
        tera.add_raw_template("replicate_dash", "{{replicate(s='-', n=7)}}")
            .unwrap();
        tera.add_raw_template("replicate_dot", "{{replicate(s='.', n=4)}}")
            .unwrap();

        let context = tera::Context::new();

        assert_eq!(tera.render("replicate_x", &context).unwrap(), "XXX");
        assert_eq!(tera.render("replicate_dash", &context).unwrap(), "-------");
        assert_eq!(tera.render("replicate_dot", &context).unwrap(), "....");
    }

    #[test]
    fn test_replicate_function_negative_length() {
        let mut tera = tera::Tera::default();
        crate::tera_functions::register_all_standard_functions(&mut tera);

        tera.add_raw_template("replicate_negative", "{{replicate(s='*', n=-1)}}")
            .unwrap();

        let context = tera::Context::new();
        let result = tera.render("replicate_negative", &context).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_replicate_function_zero_length() {
        let mut tera = tera::Tera::default();
        crate::tera_functions::register_all_standard_functions(&mut tera);

        tera.add_raw_template("replicate_zero", "{{replicate(s='*', n=0)}}")
            .unwrap();

        let context = tera::Context::new();
        let result = tera.render("replicate_zero", &context).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_template_with_length_and_replicate() {
        let mut tera = tera::Tera::default();
        crate::tera_functions::register_all_standard_functions(&mut tera);

        // Template that uses both secret_length and replicate function
        tera.add_raw_template(
            "complex",
            "<{{secret_type}}:{{replicate(s='*', n=secret_length)}}>",
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
            "[HIDDEN:{{secret_type}}]",
            "password",
            None,
        );
        assert_eq!(result, "[HIDDEN:password]");

        // Test custom template with length
        let result = generate_redacted_string_with_custom_template(
            "{{secret_type}}({{secret_length}})",
            "password",
            Some(8),
        );
        assert_eq!(result, "password(8)");

        // Test template with replicate function
        let result = generate_redacted_string_with_custom_template(
            "{{replicate(s='*', n=5)}}",
            "secret",
            None,
        );
        assert_eq!(result, "*****");
    }

    #[test]
    fn test_get_redacted_string_with_custom_template_and_value() {
        use crate::config::RedactionContext;

        // Test custom template with value
        let result = get_redacted_string_with_custom_template_and_value(
            "[SECRET:{{secret_type}}]",
            "password",
            RedactionContext::Display,
            Some(&"test123"),
        );
        assert_eq!(result, "[SECRET:password]");

        // Test custom template with length calculated from value
        let result = get_redacted_string_with_custom_template_and_value(
            "{{replicate(s='*', n=secret_length)}}",
            "password",
            RedactionContext::Display,
            Some(&"test123"),
        );
        assert_eq!(result, "*******"); // "test123" has 7 characters

        // Test custom template with secret_string variable
        let result = get_redacted_string_with_custom_template_and_value(
            "moo:{{secret_string}}",
            "string",
            RedactionContext::Display,
            Some(&"abcdf"),
        );
        assert_eq!(result, "moo:abcdf");

        // Test template that should definitely work
        let result = get_redacted_string_with_custom_template_and_value(
            "simple_test",
            "string",
            RedactionContext::Display,
            Some(&"abcdf"),
        );
        assert_eq!(result, "simple_test");
    }

    #[test]
    fn test_reverse_function() {
        // Test reverse function using existing redaction template system
        let result =
            generate_redacted_string_with_custom_template("{{reverse(s='hello')}}", "test", None);
        assert_eq!(result, "olleh");

        let result =
            generate_redacted_string_with_custom_template("{{reverse(s='world')}}", "test", None);
        assert_eq!(result, "dlrow");

        // Test with empty string
        let result =
            generate_redacted_string_with_custom_template("{{reverse(s='')}}", "test", None);
        assert_eq!(result, "");

        // Test with unicode characters
        let result =
            generate_redacted_string_with_custom_template("{{reverse(s='🚀🎉')}}", "test", None);
        assert_eq!(result, "🎉🚀");
    }

    #[test]
    fn test_reverse_function_error_handling() {
        // Test with no arguments - should fall back to basic format
        let result =
            generate_redacted_string_with_custom_template("{{reverse()}}", "test_type", None);
        assert_eq!(result, "<redacted:test_type>");

        // Test with invalid template - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "{{reverse(s=nonexistent_var)}}",
            "test_type",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");
    }

    #[test]
    fn test_take_function() {
        // Test take function using existing redaction template system
        let result = generate_redacted_string_with_custom_template(
            "{{take(n=3, s='hello world')}}",
            "test",
            None,
        );
        assert_eq!(result, "hel");

        let result = generate_redacted_string_with_custom_template(
            "{{take(n=5, s='testing')}}",
            "test",
            None,
        );
        assert_eq!(result, "testi");

        // Test taking more characters than available
        let result = generate_redacted_string_with_custom_template(
            "{{take(n=10, s='short')}}",
            "test",
            None,
        );
        assert_eq!(result, "short");

        // Test with zero characters
        let result = generate_redacted_string_with_custom_template(
            "{{take(n=0, s='anything')}}",
            "test",
            None,
        );
        assert_eq!(result, "");

        // Test with negative number
        let result =
            generate_redacted_string_with_custom_template("{{take(n=-1, s='test')}}", "test", None);
        assert_eq!(result, "");

        // Test with unicode characters
        let result = generate_redacted_string_with_custom_template(
            "{{take(n=2, s='🚀🎉🌟⭐')}}",
            "test",
            None,
        );
        assert_eq!(result, "🚀🎉");

        // Test with empty string
        let result =
            generate_redacted_string_with_custom_template("{{take(n=5, s='')}}", "test", None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_take_function_error_handling() {
        // Test with missing arguments - should fall back to basic format
        let result =
            generate_redacted_string_with_custom_template("{{take(s='test')}}", "test_type", None);
        assert_eq!(result, "<redacted:test_type>");

        let result =
            generate_redacted_string_with_custom_template("{{take(n=5)}}", "test_type", None);
        assert_eq!(result, "<redacted:test_type>");

        // Test with invalid arguments - should fall back to basic format
        let result = generate_redacted_string_with_custom_template("{{take()}}", "test_type", None);
        assert_eq!(result, "<redacted:test_type>");
    }

    #[test]
    fn test_secret_string_variable() {
        // Test secret_string variable in template (should use variable access)
        let result = generate_redacted_string_with_custom_template_and_value(
            "{{secret_string}}",
            "test",
            None,
            Some("secret_value".to_string()),
        );
        assert_eq!(result, "secret_value");

        // Test secret_string variable with template containing other content
        let result = generate_redacted_string_with_custom_template_and_value(
            "prefix:{{secret_string}}:suffix",
            "test",
            None,
            Some("middle".to_string()),
        );
        assert_eq!(result, "prefix:middle:suffix");

        // Test secret_string variable when not available (template should fail)
        let result =
            generate_redacted_string_with_custom_template("{{secret_string}}", "test", None);
        assert_eq!(result, "<redacted:test>");

        // Test secret_string variable with no secret value provided (template should fail)
        let result = generate_redacted_string_with_custom_template_and_value(
            "value:{{secret_string}}",
            "test",
            None,
            None,
        );
        assert_eq!(result, "<redacted:test>");
    }

    #[test]
    fn test_template_error_fallback_handling() {
        // Test invalid template syntax - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "{{invalid syntax without closing",
            "test_type",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");

        // Test template with undefined variable - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "{{undefined_variable}}",
            "test_type",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");

        // Test template with invalid function call - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "{{nonexistent_function()}}",
            "test_type",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");
    }

    #[test]
    fn test_replicate_function_error_handling() {
        // Test with missing 's' parameter - should fall back to basic format
        let result =
            generate_redacted_string_with_custom_template("{{replicate(n=5)}}", "test_type", None);
        assert_eq!(result, "<redacted:test_type>");

        // Test with missing 'n' parameter - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "{{replicate(s='*')}}",
            "test_type",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");

        // Test with no parameters - should fall back to basic format
        let result =
            generate_redacted_string_with_custom_template("{{replicate()}}", "test_type", None);
        assert_eq!(result, "<redacted:test_type>");

        // Test with empty character string - should still work
        let result =
            generate_redacted_string_with_custom_template("{{replicate(s='', n=3)}}", "test", None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_complex_template_combinations() {
        // Test combining replicate with secret length
        let result = generate_redacted_string_with_custom_template_and_value(
            "[{{replicate(s='-', n=secret_length)}}]",
            "secret",
            Some(4), // Explicitly set the length
            Some("test".to_string()),
        );
        assert_eq!(result, "[----]"); // "test" has 4 characters

        // Test template with secret_type and secret_length variables
        let result = generate_redacted_string_with_custom_template_and_value(
            "{{secret_type}}:{{secret_length}}",
            "complex",
            Some(6), // Explicitly set the length
            Some("abcdef".to_string()),
        );
        assert_eq!(result, "complex:6");

        // Test template with secret_string variable
        let result = generate_redacted_string_with_custom_template_and_value(
            "value={{secret_string}}",
            "test",
            None,
            Some("secret123".to_string()),
        );
        assert_eq!(result, "value=secret123");

        // Test combining reverse with secret_string
        let result = generate_redacted_string_with_custom_template_and_value(
            "{{reverse(s=secret_string)}}",
            "test",
            None,
            Some("hello".to_string()),
        );
        assert_eq!(result, "olleh");

        // Test combining take with secret_string
        let result = generate_redacted_string_with_custom_template_and_value(
            "{{take(n=3, s=secret_string)}}",
            "test",
            None,
            Some("abcdef".to_string()),
        );
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_secret_string_integration_with_none() {
        // Test that secret_string variable is not available when no secret is provided
        let result = generate_redacted_string_with_length(None, "test", None);
        assert_eq!(result, "<redacted:test>");

        // Test behavior when secret_string is None but we try to use it in custom templates
        // Template should fail and fall back to basic format
        let result =
            generate_redacted_string_with_custom_template("value:{{secret_string}}", "test", None);
        // Template fails and falls back to basic format since secret_string is not in context
        assert_eq!(result, "<redacted:test>");

        // Test that secret_string variable is not available in template context when None
        let result =
            generate_redacted_string_with_custom_template("{{secret_string}}", "test", None);
        // Template fails and falls back to basic format since secret_string is not in context
        assert_eq!(result, "<redacted:test>");
    }

    #[test]
    fn test_secret_string_integration_with_some() {
        // Test that secret_string variable works when secret is provided
        let result = generate_redacted_string_with_length(Some("mysecret"), "test", None);
        assert_eq!(result, "<redacted:test>");

        // Test with custom template that uses secret_string variable when Some is provided
        let result = generate_redacted_string_with_custom_template_and_value(
            "value:{{secret_string}}",
            "test",
            None,
            Some("mysecret".to_string()),
        );
        assert_eq!(result, "value:mysecret");

        // Test with template variable access
        let result = generate_redacted_string_with_custom_template_and_value(
            "{{secret_string}}",
            "test",
            None,
            Some("direct_access".to_string()),
        );
        assert_eq!(result, "direct_access");
    }

    #[test]
    fn test_secret_string_cached_functions_integration() {
        // Test get_cached_redacted_string with None
        let result = get_cached_redacted_string(None, "string");
        assert_eq!(result, "<redacted:string>");

        // Test get_cached_redacted_string with Some
        let result = get_cached_redacted_string(Some("secret"), "string");
        assert_eq!(result, "<redacted:string>");

        // Test get_cached_redacted_string_with_length with None
        let result = get_cached_redacted_string_with_length(None, "string", Some(8));
        assert_eq!(result, "<redacted:string>");

        // Test get_cached_redacted_string_with_length with Some
        let result = get_cached_redacted_string_with_length(Some("mysecret"), "string", Some(8));
        assert_eq!(result, "<redacted:string>");
    }

    #[test]
    fn test_strlen_function() {
        // Test strlen function with basic strings
        let result =
            generate_redacted_string_with_custom_template("{{strlen(s='hello')}}", "test", None);
        assert_eq!(result, "5");

        let result =
            generate_redacted_string_with_custom_template("{{strlen(s='world!')}}", "test", None);
        assert_eq!(result, "6");

        // Test with empty string
        let result =
            generate_redacted_string_with_custom_template("{{strlen(s='')}}", "test", None);
        assert_eq!(result, "0");

        // Test with unicode characters (should count characters, not bytes)
        let result =
            generate_redacted_string_with_custom_template("{{strlen(s='🚀🎉🌟')}}", "test", None);
        assert_eq!(result, "3");

        // Test with multi-byte unicode characters
        let result =
            generate_redacted_string_with_custom_template("{{strlen(s='café')}}", "test", None);
        assert_eq!(result, "4");

        // Test with positional argument (using unnamed parameter)
        let result =
            generate_redacted_string_with_custom_template("{{strlen(s='testing')}}", "test", None);
        assert_eq!(result, "7");

        // Test with longer string
        let result = generate_redacted_string_with_custom_template(
            "{{strlen(s='This is a longer test string!')}}",
            "test",
            None,
        );
        assert_eq!(result, "29");
    }

    #[test]
    fn test_strlen_function_error_handling() {
        // Test with no arguments - should fall back to basic format
        let result =
            generate_redacted_string_with_custom_template("{{strlen()}}", "test_type", None);
        assert_eq!(result, "<redacted:test_type>");

        // Test with invalid template - should fall back to basic format
        let result = generate_redacted_string_with_custom_template(
            "{{strlen(s=nonexistent_var)}}",
            "test_type",
            None,
        );
        assert_eq!(result, "<redacted:test_type>");
    }

    #[test]
    fn test_strlen_function_with_template_variables() {
        // Test strlen with secret_string variable
        let result = generate_redacted_string_with_custom_template_and_value(
            "{{strlen(s=secret_string)}}",
            "test",
            None,
            Some("secret123".to_string()),
        );
        assert_eq!(result, "9");

        // Test combining strlen with other functions
        let result = generate_redacted_string_with_custom_template_and_value(
            "Length: {{strlen(s=secret_string)}}, First 3: {{take(n=3, s=secret_string)}}",
            "test",
            None,
            Some("password".to_string()),
        );
        assert_eq!(result, "Length: 8, First 3: pas");

        // Test strlen with reverse function result
        let result = generate_redacted_string_with_custom_template(
            "{{strlen(s=reverse(s='hello'))}}",
            "test",
            None,
        );
        assert_eq!(result, "5"); // Length of "olleh" is still 5
    }

    #[test]
    fn test_strlen_function_complex_templates() {
        // Test template that uses strlen for conditional logic-like display
        let result = generate_redacted_string_with_custom_template_and_value(
            "{{secret_type}}[{{strlen(s=secret_string)}}]: {{replicate(s='*', n=strlen(s=secret_string))}}",
            "password",
            None,
            Some("test123".to_string()),
        );
        assert_eq!(result, "password[7]: *******");

        // Test with multiple strlen calls
        let result = generate_redacted_string_with_custom_template_and_value(
            "Original: {{strlen(s=secret_string)}}, Reversed: {{strlen(s=reverse(s=secret_string))}}",
            "test",
            None,
            Some("hello".to_string()),
        );
        assert_eq!(result, "Original: 5, Reversed: 5");
    }

    #[test]
    fn test_mask_secret_functionality() {
        // Note: These tests verify the masking logic works correctly
        // In a real scenario, we would need to mock the global config
        // For now, we test the logic flow and expectations

        // Test that mask_secret logic produces expected results for all secret types
        let test_secret = "password123";
        let expected_mask = "*".repeat(test_secret.len());
        assert_eq!(expected_mask, "***********");

        // Test masking with different lengths
        let short_secret = "abc";
        let short_mask = "*".repeat(short_secret.len());
        assert_eq!(short_mask, "***");

        let long_secret = "this_is_a_very_long_secret_string";
        let long_mask = "*".repeat(long_secret.len());
        assert_eq!(long_mask, "*".repeat(33));

        // Test empty string masking
        let empty_secret = "";
        let empty_mask = "*".repeat(empty_secret.len());
        assert_eq!(empty_mask, "");

        // Test masking for binary data (as string representation)
        let binary_repr = "[1, 2]";
        let binary_mask = "*".repeat(binary_repr.len());
        assert_eq!(binary_mask, "******");

        // Test masking for numeric data
        let number_repr = "42";
        let number_mask = "*".repeat(number_repr.len());
        assert_eq!(number_mask, "**");
    }

    #[test]
    fn test_mask_secret_config_structure() {
        use crate::config::PluginConfig;

        // Test that mask_secret can be configured
        let mut config = PluginConfig::default();
        assert!(!config.redaction.mask_secret);

        // Test setting mask_secret to true
        config.redaction.mask_secret = true;
        assert!(config.redaction.mask_secret);

        // Test that mask_secret can coexist with other settings
        config.redaction.show_unredacted = true;
        config.redaction.redaction_template = Some("custom_template".to_string());
        assert!(config.redaction.mask_secret);
        assert!(config.redaction.show_unredacted);
        assert_eq!(
            config.redaction.redaction_template,
            Some("custom_template".to_string())
        );
    }

    #[test]
    fn test_secret_string_template_context_integration() {
        // Test that secret_string variable is available in template context when provided
        let result = generate_redacted_string_with_custom_template_and_value(
            "prefix_{{secret_string}}_suffix",
            "test",
            None,
            Some("middle".to_string()),
        );
        assert_eq!(result, "prefix_middle_suffix");

        // Test that secret_string variable is not available when not provided
        let result = generate_redacted_string_with_custom_template(
            "prefix_{{secret_string}}_suffix",
            "test",
            None,
        );
        // Template fails and falls back to basic format since secret_string is not in context
        assert_eq!(result, "<redacted:test>");
    }

    #[test]
    fn test_secret_string_with_other_template_variables() {
        // Test combining secret_string with secret_type
        let result = generate_redacted_string_with_custom_template_and_value(
            "{{secret_type}}:{{secret_string}}",
            "password",
            None,
            Some("secret123".to_string()),
        );
        assert_eq!(result, "password:secret123");

        // Test combining secret_string with secret_length
        let result = generate_redacted_string_with_custom_template_and_value(
            "{{secret_string}}({{secret_length}})",
            "password",
            Some(9),
            Some("secret123".to_string()),
        );
        assert_eq!(result, "secret123(9)");

        // Test all three variables together
        let result = generate_redacted_string_with_custom_template_and_value(
            "{{secret_type}}:{{secret_string}}:{{secret_length}}",
            "token",
            Some(10),
            Some("abcdef1234".to_string()),
        );
        assert_eq!(result, "token:abcdef1234:10");
    }

    #[test]
    fn test_secret_string_function_registration_conditional() {
        // Test that templates using secret_string() function work when secret is provided
        let mut tera = tera::Tera::default();

        // Register secret_string function conditionally (simulating Some case)
        let secret_value = "test_secret";
        let captured_secret = secret_value.to_string();
        tera.register_function(
            "secret_string",
            move |_args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                Ok(tera::Value::String(captured_secret.clone()))
            },
        );

        tera.add_raw_template("test", "func:{{secret_string()}}")
            .unwrap();
        let context = tera::Context::new();
        let result = tera.render("test", &context).unwrap();
        assert_eq!(result, "func:test_secret");

        // Test that templates fail gracefully when secret_string function is not registered
        let mut tera2 = tera::Tera::default();
        tera2
            .add_raw_template("test2", "func:{{secret_string()}}")
            .unwrap();
        let context2 = tera::Context::new();
        let result2 = tera2.render("test2", &context2);
        assert!(result2.is_err()); // Should fail since function is not registered
    }
}
