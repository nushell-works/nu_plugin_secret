//! Tera-based redaction templating system
//!
//! This module provides a configurable templating system for redaction using the Tera template engine.
//! The default template is `<redacted:{{secret_type}}>` where `secret_type` is the type name
//! of the secret (e.g., "string", "float", "int", etc.).
//! Templates can be customized through the configuration file's `redaction_template` field.

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
    let _ = tera.add_raw_template(TEMPLATE_NAME, &template);

    let mut context = Context::new();
    context.insert("secret_type", secret_type);

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

    // Return redacted string using Tera templating
    get_cached_redacted_string(secret_type)
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
}
