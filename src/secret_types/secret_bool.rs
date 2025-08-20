use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A secure boolean type that redacts its content in all display contexts
#[derive(Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct SecretBool {
    #[zeroize(skip)]
    inner: bool,
}

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
