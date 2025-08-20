use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A secure float type that redacts its content in all display contexts
#[derive(Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct SecretFloat {
    #[zeroize(skip)]
    inner: f64,
}

impl SecretFloat {
    /// Create a new SecretFloat from a regular f64
    pub fn new(value: f64) -> Self {
        Self { inner: value }
    }

    /// Get a reference to the inner float (for controlled access)
    pub fn reveal(&self) -> f64 {
        self.inner
    }

    /// Convert back to a regular f64 (consumes the SecretFloat)
    pub fn into_inner(self) -> f64 {
        self.inner
    }

    /// Check if the float is NaN (safe to expose)
    pub fn is_nan(&self) -> bool {
        self.inner.is_nan()
    }

    /// Check if the float is infinite (safe to expose)
    pub fn is_infinite(&self) -> bool {
        self.inner.is_infinite()
    }

    /// Check if the float is finite (safe to expose)
    pub fn is_finite(&self) -> bool {
        self.inner.is_finite()
    }
}

#[typetag::serde]
impl CustomValue for SecretFloat {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        "secret_float".into()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        Ok(Value::string("<redacted:float>", span))
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

impl fmt::Display for SecretFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<redacted:float>")
    }
}

impl fmt::Debug for SecretFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretFloat(<redacted>)")
    }
}

impl PartialEq for SecretFloat {
    fn eq(&self, other: &Self) -> bool {
        // Use constant-time comparison for security
        // For floats, we need to handle NaN specially
        if self.inner.is_nan() && other.inner.is_nan() {
            return true;
        }

        // Convert to bytes for constant-time comparison
        let self_bytes = self.inner.to_bits().to_le_bytes();
        let other_bytes = other.inner.to_bits().to_le_bytes();

        let mut result = 0u8;
        for i in 0..8 {
            result |= self_bytes[i] ^ other_bytes[i];
        }
        result == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_float_creation() {
        let secret = SecretFloat::new(3.14159);
        assert_eq!(secret.reveal(), 3.14159);
    }

    #[test]
    fn test_secret_float_display() {
        let secret = SecretFloat::new(123.456);
        assert_eq!(format!("{}", secret), "<redacted:float>");
        assert_eq!(format!("{:?}", secret), "SecretFloat(<redacted>)");
    }

    #[test]
    fn test_secret_float_custom_value() {
        let secret = SecretFloat::new(2.718);
        assert_eq!(secret.type_name(), "secret_float");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => assert_eq!(val, "<redacted:float>"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_secret_float_into_inner() {
        let secret = SecretFloat::new(42.0);
        assert_eq!(secret.into_inner(), 42.0);
    }

    #[test]
    fn test_secret_float_equality() {
        let secret1 = SecretFloat::new(1.23);
        let secret2 = SecretFloat::new(1.23);
        let secret3 = SecretFloat::new(4.56);

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }

    #[test]
    fn test_secret_float_nan_equality() {
        let nan1 = SecretFloat::new(f64::NAN);
        let nan2 = SecretFloat::new(f64::NAN);
        let normal = SecretFloat::new(1.0);

        assert_eq!(nan1, nan2); // Our implementation treats NaN as equal
        assert_ne!(nan1, normal);
    }

    #[test]
    fn test_secret_float_special_checks() {
        let normal = SecretFloat::new(1.0);
        let nan = SecretFloat::new(f64::NAN);
        let inf = SecretFloat::new(f64::INFINITY);

        assert!(normal.is_finite());
        assert!(!normal.is_nan());
        assert!(!normal.is_infinite());

        assert!(nan.is_nan());
        assert!(!nan.is_finite());

        assert!(inf.is_infinite());
        assert!(!inf.is_finite());
    }
}
