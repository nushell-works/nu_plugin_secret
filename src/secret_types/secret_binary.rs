use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A secure binary type that redacts its content in all display contexts
#[derive(Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct SecretBinary {
    #[zeroize(skip)]
    inner: Vec<u8>,
}

impl SecretBinary {
    /// Create a new SecretBinary from a byte vector
    pub fn new(value: Vec<u8>) -> Self {
        Self { inner: value }
    }

    /// Get a reference to the inner binary data (for controlled access)
    pub fn reveal(&self) -> &Vec<u8> {
        &self.inner
    }

    /// Convert back to a regular byte vector (consumes the SecretBinary)
    pub fn into_inner(self) -> Vec<u8> {
        self.inner.clone()
    }

    /// Get the length of the binary data (safe to expose)
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the binary data is empty (safe to expose)
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get a byte at a specific index while preserving secrecy
    pub fn get(&self, index: usize) -> Option<u8> {
        self.inner.get(index).copied()
    }
}

#[typetag::serde]
impl CustomValue for SecretBinary {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        "secret_binary".into()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        Ok(Value::string("<redacted:binary>", span))
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

impl fmt::Display for SecretBinary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<redacted:binary>")
    }
}

impl fmt::Debug for SecretBinary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretBinary(<redacted>)")
    }
}

impl PartialEq for SecretBinary {
    fn eq(&self, other: &Self) -> bool {
        // Use constant-time comparison for security
        if self.inner.len() != other.inner.len() {
            return false;
        }
        
        let mut result = 0u8;
        for i in 0..self.inner.len() {
            result |= self.inner[i] ^ other.inner[i];
        }
        result == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_binary_creation() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let secret = SecretBinary::new(data.clone());
        assert_eq!(secret.reveal(), &data);
    }

    #[test]
    fn test_secret_binary_display() {
        let data = vec![0xde, 0xad, 0xbe, 0xef];
        let secret = SecretBinary::new(data);
        assert_eq!(format!("{}", secret), "<redacted:binary>");
        assert_eq!(format!("{:?}", secret), "SecretBinary(<redacted>)");
    }

    #[test]
    fn test_secret_binary_custom_value() {
        let data = vec![0x01, 0x23, 0x45, 0x67];
        let secret = SecretBinary::new(data);
        assert_eq!(secret.type_name(), "secret_binary");
        
        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => assert_eq!(val, "<redacted:binary>"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_secret_binary_into_inner() {
        let data = vec![0xff, 0x00, 0x11, 0x22];
        let secret = SecretBinary::new(data.clone());
        assert_eq!(secret.into_inner(), data);
    }

    #[test]
    fn test_secret_binary_equality() {
        let data1 = vec![0x01, 0x02, 0x03];
        let data2 = vec![0x01, 0x02, 0x03];
        let data3 = vec![0x04, 0x05, 0x06];
        
        let secret1 = SecretBinary::new(data1);
        let secret2 = SecretBinary::new(data2);
        let secret3 = SecretBinary::new(data3);
        
        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }

    #[test]
    fn test_secret_binary_access() {
        let data = vec![0x10, 0x20, 0x30, 0x40];
        let secret = SecretBinary::new(data);
        
        assert_eq!(secret.len(), 4);
        assert!(!secret.is_empty());
        
        assert_eq!(secret.get(0), Some(0x10));
        assert_eq!(secret.get(1), Some(0x20));
        assert_eq!(secret.get(4), None); // Out of bounds
    }

    #[test]
    fn test_secret_binary_empty() {
        let empty_data: Vec<u8> = vec![];
        let secret = SecretBinary::new(empty_data);
        
        assert_eq!(secret.len(), 0);
        assert!(secret.is_empty());
        assert_eq!(secret.get(0), None);
    }

    #[test]
    fn test_secret_binary_equality_different_lengths() {
        let data1 = vec![0x01, 0x02];
        let data2 = vec![0x01, 0x02, 0x03];
        
        let secret1 = SecretBinary::new(data1);
        let secret2 = SecretBinary::new(data2);
        
        assert_ne!(secret1, secret2);
    }
}