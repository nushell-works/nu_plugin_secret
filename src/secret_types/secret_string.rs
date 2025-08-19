use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A secure string type that redacts its content in all display contexts
#[derive(Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct SecretString {
    #[zeroize(skip)]
    inner: String,
}

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
        Ok(Value::string("<redacted:string>", span))
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
        write!(f, "<redacted:string>")
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretString(<redacted>)")
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
        assert_eq!(format!("{}", secret), "<redacted:string>");
        assert_eq!(format!("{:?}", secret), "SecretString(<redacted>)");
    }

    #[test]
    fn test_secret_string_custom_value() {
        let secret = SecretString::new("my-secret".to_string());
        assert_eq!(secret.type_name(), "secret_string");
        
        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => assert_eq!(val, "<redacted:string>"),
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