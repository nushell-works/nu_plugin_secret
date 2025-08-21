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

// Custom secure serialization - never serialize actual content
impl Serialize for SecretBool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Always serialize as redacted content for security
        serializer.serialize_str("<redacted:bool>")
    }
}

// Custom secure deserialization
impl<'de> Deserialize<'de> for SecretBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // For security, we can't deserialize actual secrets
        // This prevents injection attacks via malicious serialized data
        let _value = bool::deserialize(deserializer)?;
        
        // Return a safe placeholder - real secrets should be created through proper channels
        Ok(SecretBool::new(false))
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
        Ok(Value::string("<redacted:bool>", span))
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
        write!(f, "<redacted:bool>")
    }
}

impl fmt::Debug for SecretBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretBool(<redacted>)")
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
        assert_eq!(format!("{}", secret), "<redacted:bool>");
        assert_eq!(format!("{:?}", secret), "SecretBool(<redacted>)");
    }

    #[test]
    fn test_secret_bool_custom_value() {
        let secret = SecretBool::new(false);
        assert_eq!(secret.type_name(), "secret_bool");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => assert_eq!(val, "<redacted:bool>"),
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
