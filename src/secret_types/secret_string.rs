use crate::config::RedactionContext;
use crate::memory_optimizations::get_configurable_redacted_string;
use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secure string type that redacts its content in all display contexts
/// and zeros its memory on drop
#[derive(Clone)]
pub struct SecretString {
    inner: String,
}

// Custom secure serialization - never serialize actual content
impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Always serialize as redacted content for security
        let redacted_text =
            get_configurable_redacted_string("string", RedactionContext::Serialization);
        serializer.serialize_str(&redacted_text)
    }
}

// Custom secure deserialization
impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // For security, we can't deserialize actual secrets
        // This prevents injection attacks via malicious serialized data
        let _value = String::deserialize(deserializer)?;

        // Return a safe placeholder - real secrets should be created through proper channels
        Ok(SecretString::new("<deserialized:placeholder>".to_string()))
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        // Explicitly zero the string memory for security
        self.inner.zeroize();
    }
}

// Manual ZeroizeOnDrop implementation to ensure proper cleanup
impl ZeroizeOnDrop for SecretString {}

impl SecretString {
    /// Create a new SecretString from a regular string
    pub fn new(value: String) -> Self {
        Self { inner: value }
    }

    /// Get a reference to the inner string (for controlled access)
    pub fn reveal(&self) -> &str {
        &self.inner
    }

    /// Convert back to a regular string (consumes the SecretString)
    pub fn into_inner(self) -> String {
        self.inner.clone()
    }
}

#[typetag::serde]
impl CustomValue for SecretString {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        "secret_string".into()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        let redacted_text =
            get_configurable_redacted_string("string", RedactionContext::Serialization);
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

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string("string", RedactionContext::Display);
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string("string", RedactionContext::Debug);
        write!(f, "SecretString({})", redacted_text)
    }
}

impl PartialEq for SecretString {
    fn eq(&self, other: &Self) -> bool {
        // Use constant-time comparison for security
        if self.inner.len() != other.inner.len() {
            return false;
        }

        let self_bytes = self.inner.as_bytes();
        let other_bytes = other.inner.as_bytes();

        // Simple constant-time comparison
        let mut result = 0u8;
        for i in 0..self_bytes.len() {
            result |= self_bytes[i] ^ other_bytes[i];
        }
        result == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_string_creation() {
        let secret = SecretString::new("my-secret".to_string());
        assert_eq!(secret.reveal(), "my-secret");
    }

    #[test]
    fn test_secret_string_display() {
        let secret = SecretString::new("my-secret".to_string());
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
    fn test_secret_string_custom_value() {
        let secret = SecretString::new("my-secret".to_string());
        assert_eq!(secret.type_name(), "secret_string");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => {
                assert!(val.contains("redacted") || val.contains("***") || val.contains("HIDDEN"))
            }
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_secret_string_equality() {
        let secret1 = SecretString::new("same".to_string());
        let secret2 = SecretString::new("same".to_string());
        let secret3 = SecretString::new("different".to_string());

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }

    #[test]
    fn test_secret_string_into_inner() {
        let secret = SecretString::new("my-secret".to_string());
        let revealed = secret.into_inner();
        assert_eq!(revealed, "my-secret");
    }
}
