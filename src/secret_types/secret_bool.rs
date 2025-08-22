use crate::config::RedactionContext;
use crate::memory_optimizations::get_configurable_redacted_string;
use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secure boolean type that redacts its content in all display contexts
/// and zeros its memory on drop
#[derive(Clone)]
pub struct SecretBool {
    inner: bool,
}

// Functional serialization - serialize actual content for pipeline operations
impl Serialize for SecretBool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the actual content to make pipeline operations work
        self.inner.serialize(serializer)
    }
}

// Functional deserialization - restore actual content for pipeline operations
impl<'de> Deserialize<'de> for SecretBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the actual content to make pipeline operations work
        let content = bool::deserialize(deserializer)?;
        Ok(SecretBool::new(content))
    }
}

impl Drop for SecretBool {
    fn drop(&mut self) {
        // Explicitly zero the boolean memory for security
        self.inner.zeroize();
    }
}

// Manual ZeroizeOnDrop implementation to ensure proper cleanup
impl ZeroizeOnDrop for SecretBool {}

impl SecretBool {
    /// Create a new SecretBool from a regular boolean
    pub fn new(value: bool) -> Self {
        Self { inner: value }
    }

    /// Get a reference to the inner boolean (for controlled access)
    pub fn reveal(&self) -> bool {
        self.inner
    }

    /// Convert back to a regular boolean (consumes the SecretBool)
    pub fn into_inner(self) -> bool {
        self.inner
    }
}

#[typetag::serde]
impl CustomValue for SecretBool {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        "secret_bool".into()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        let redacted_text =
            get_configurable_redacted_string("bool", RedactionContext::Serialization);
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

impl fmt::Display for SecretBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string("bool", RedactionContext::Display);
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string("bool", RedactionContext::Debug);
        write!(f, "SecretBool({})", redacted_text)
    }
}

impl PartialEq for SecretBool {
    fn eq(&self, other: &Self) -> bool {
        // Use constant-time comparison for security
        // For booleans, we can use XOR
        (self.inner as u8) ^ (other.inner as u8) == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_bool_creation() {
        let secret_true = SecretBool::new(true);
        let secret_false = SecretBool::new(false);
        assert!(secret_true.reveal());
        assert!(!secret_false.reveal());
    }

    #[test]
    fn test_secret_bool_display() {
        let secret = SecretBool::new(true);
        let display_result = format!("{}", secret);
        assert!(
            display_result.contains("redacted")
                || display_result.contains("***")
                || display_result.contains("HIDDEN")
        );
        let debug_result = format!("{:?}", secret);
        assert!(
            debug_result.contains("redacted")
                || debug_result.contains("***")
                || debug_result.contains("HIDDEN")
        );
    }

    #[test]
    fn test_secret_bool_custom_value() {
        let secret = SecretBool::new(false);
        assert_eq!(secret.type_name(), "secret_bool");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => {
                assert!(val.contains("redacted") || val.contains("***") || val.contains("HIDDEN"))
            }
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_secret_bool_equality() {
        let secret1 = SecretBool::new(true);
        let secret2 = SecretBool::new(true);
        let secret3 = SecretBool::new(false);

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }

    #[test]
    fn test_secret_bool_into_inner() {
        let secret = SecretBool::new(true);
        let revealed = secret.into_inner();
        assert!(revealed);
    }
}
