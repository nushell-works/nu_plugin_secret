use std::fmt;

use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::config::RedactionContext;

/// A secure integer type that redacts its content in all display contexts
/// and zeros its memory on drop
#[derive(Clone)]
pub struct SecretInt {
    inner: i64,
    #[allow(dead_code)]
    redaction_template: Option<String>,
}

// Functional serialization - serialize actual content for pipeline operations
impl Serialize for SecretInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SecretInt", 2)?;
        state.serialize_field("inner", &self.inner)?;
        state.serialize_field("redaction_template", &self.redaction_template)?;
        state.end()
    }
}

// Functional deserialization - restore actual content for pipeline operations
impl<'de> Deserialize<'de> for SecretInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SecretIntData {
            inner: i64,
            redaction_template: Option<String>,
        }

        let data = SecretIntData::deserialize(deserializer)?;
        Ok(SecretInt {
            inner: data.inner,
            redaction_template: data.redaction_template,
        })
    }
}

impl Drop for SecretInt {
    fn drop(&mut self) {
        // Explicitly zero the integer memory for security
        self.inner.zeroize();
    }
}

// Manual ZeroizeOnDrop implementation to ensure proper cleanup
impl ZeroizeOnDrop for SecretInt {}

impl SecretInt {
    /// Create a new SecretInt from a regular integer
    pub fn new(value: i64) -> Self {
        Self {
            inner: value,
            redaction_template: None,
        }
    }

    /// Create a new SecretInt with a custom redaction template
    pub fn new_with_template(value: i64, template: String) -> Self {
        Self {
            inner: value,
            redaction_template: Some(template),
        }
    }

    /// Get a reference to the inner integer (for controlled access)
    pub fn reveal(&self) -> i64 {
        self.inner
    }

    /// Convert back to a regular integer (consumes the SecretInt)
    pub fn into_inner(self) -> i64 {
        self.inner
    }
}

#[typetag::serde]
impl CustomValue for SecretInt {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        "secret_int".into()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::get_redacted_string_with_custom_template_and_value(
                template,
                "int",
                RedactionContext::Serialization,
                Some(&self.inner),
            )
        } else {
            crate::redaction::get_redacted_string_with_value(
                "int",
                RedactionContext::Serialization,
                Some(&self.inner),
            )
        };
        Ok(Value::string(redacted_text, span))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn notify_plugin_on_drop(&self) -> bool {
        false // We handle cleanup via ZeroizeOnDrop
    }
}

impl fmt::Display for SecretInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::get_redacted_string_with_custom_template_and_value(
                template,
                "int",
                RedactionContext::Display,
                Some(&self.inner),
            )
        } else {
            crate::redaction::get_redacted_string_with_value(
                "int",
                RedactionContext::Display,
                Some(&self.inner),
            )
        };
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::get_redacted_string_with_custom_template_and_value(
                template,
                "int",
                RedactionContext::Debug,
                Some(&self.inner),
            )
        } else {
            crate::redaction::get_redacted_string_with_value(
                "int",
                RedactionContext::Debug,
                Some(&self.inner),
            )
        };
        write!(f, "SecretInt({})", redacted_text)
    }
}

impl PartialEq for SecretInt {
    fn eq(&self, other: &Self) -> bool {
        // Use constant-time comparison for security
        // For integers, we can use simple XOR
        let diff = self.inner ^ other.inner;
        diff == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_int_creation() {
        let secret = SecretInt::new(42);
        assert_eq!(secret.reveal(), 42);
    }

    #[test]
    fn test_secret_int_display() {
        let secret = SecretInt::new(12345);
        // Test that display uses configurable redaction (falls back to default if config not loaded)
        let display_result = format!("{}", secret);
        let debug_result = format!("{:?}", secret);

        // Should contain redacted text (exact text depends on configuration)
        assert!(
            display_result.contains("redacted")
                || display_result.contains("***")
                || display_result.contains("HIDDEN")
        );
        assert!(debug_result.contains("SecretInt"));
    }

    #[test]
    fn test_secret_int_custom_value() {
        let secret = SecretInt::new(999);
        assert_eq!(secret.type_name(), "secret_int");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => {
                // Should contain redacted text (exact text depends on configuration)
                assert!(val.contains("redacted") || val.contains("***") || val.contains("HIDDEN"));
            }
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_secret_int_equality() {
        let secret1 = SecretInt::new(42);
        let secret2 = SecretInt::new(42);
        let secret3 = SecretInt::new(99);

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }

    #[test]
    fn test_secret_int_into_inner() {
        let secret = SecretInt::new(777);
        let revealed = secret.into_inner();
        assert_eq!(revealed, 777);
    }

    #[test]
    fn test_secret_int_with_custom_template() {
        let secret = SecretInt::new_with_template(42, "{{secret_type}}_HIDDEN".to_string());

        // Test Display
        let display = format!("{}", secret);
        assert_eq!(display, "int_HIDDEN");

        // Test Debug
        let debug = format!("{:?}", secret);
        assert_eq!(debug, "SecretInt(int_HIDDEN)");

        // Test to_base_value
        let base_value = secret
            .to_base_value(nu_protocol::Span::test_data())
            .unwrap();
        if let nu_protocol::Value::String { val, .. } = base_value {
            assert_eq!(val, "int_HIDDEN");
        } else {
            panic!("Expected string value");
        }

        // Test reveal still works
        assert_eq!(secret.reveal(), 42);
    }

    #[test]
    fn test_secret_int_with_replicate_template() {
        let secret = SecretInt::new_with_template(
            12345,
            "{{replicate(s='*', n=secret_length)}}".to_string(),
        );

        let display = format!("{}", secret);
        assert_eq!(display, "*****"); // "12345" has 5 characters

        assert_eq!(secret.reveal(), 12345);
    }
}
