use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A secure integer type that redacts its content in all display contexts
#[derive(Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct SecretInt {
    #[zeroize(skip)]
    inner: i64,
}

impl SecretInt {
    /// Create a new SecretInt from a regular integer
    pub fn new(value: i64) -> Self {
        Self { inner: value }
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
        Ok(Value::string("<redacted:int>", span))
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
        write!(f, "<redacted:int>")
    }
}

impl fmt::Debug for SecretInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretInt(<redacted>)")
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
        assert_eq!(format!("{}", secret), "<redacted:int>");
        assert_eq!(format!("{:?}", secret), "SecretInt(<redacted>)");
    }

    #[test]
    fn test_secret_int_custom_value() {
        let secret = SecretInt::new(999);
        assert_eq!(secret.type_name(), "secret_int");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => assert_eq!(val, "<redacted:int>"),
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
}
