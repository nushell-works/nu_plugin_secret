use crate::config::RedactionContext;
use crate::memory_optimizations::get_configurable_redacted_string_with_generic_value;
use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secure float type that redacts its content in all display contexts
/// and zeros its memory on drop
#[derive(Clone)]
pub struct SecretFloat {
    inner: f64,
}

// Functional serialization - serialize actual content for pipeline operations
impl Serialize for SecretFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the actual content to make pipeline operations work
        self.inner.serialize(serializer)
    }
}

// Functional deserialization - restore actual content for pipeline operations
impl<'de> Deserialize<'de> for SecretFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the actual content to make pipeline operations work
        let content = f64::deserialize(deserializer)?;
        Ok(SecretFloat::new(content))
    }
}

impl Drop for SecretFloat {
    fn drop(&mut self) {
        // Explicitly zero the float memory for security
        self.inner.zeroize();
    }
}

// Manual ZeroizeOnDrop implementation to ensure proper cleanup
impl ZeroizeOnDrop for SecretFloat {}

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
        let redacted_text = get_configurable_redacted_string_with_generic_value(
            "float",
            RedactionContext::Serialization,
            Some(&self.inner),
        );
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

impl fmt::Display for SecretFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string_with_generic_value(
            "float",
            RedactionContext::Display,
            Some(&self.inner),
        );
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string_with_generic_value(
            "float",
            RedactionContext::Debug,
            Some(&self.inner),
        );
        write!(f, "SecretFloat({})", redacted_text)
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
        let secret = SecretFloat::new(std::f64::consts::PI);
        assert_eq!(secret.reveal(), std::f64::consts::PI);
    }

    #[test]
    fn test_secret_float_display() {
        let secret = SecretFloat::new(123.456);
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
    fn test_secret_float_custom_value() {
        let secret = SecretFloat::new(std::f64::consts::E);
        assert_eq!(secret.type_name(), "secret_float");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => {
                assert!(val.contains("redacted") || val.contains("***") || val.contains("HIDDEN"))
            }
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

    #[test]
    fn test_secret_float_serialization() {
        // Test that serialization works for functional unwrap
        let value = std::f64::consts::PI;
        let secret = SecretFloat::new(value);

        // Test JSON serialization
        let json_result = serde_json::to_string(&secret);
        assert!(json_result.is_ok(), "JSON serialization should work");

        let json = json_result.unwrap();
        // Should contain the numeric data for functional unwrap
        assert!(json.contains("3.14159"), "JSON should contain PI value");

        // Test bincode serialization (used for plugin communication)
        let bincode_result = bincode::serialize(&secret);
        assert!(bincode_result.is_ok(), "Bincode serialization should work");

        // Test special values
        let nan_secret = SecretFloat::new(f64::NAN);
        let inf_secret = SecretFloat::new(f64::INFINITY);

        assert!(
            serde_json::to_string(&nan_secret).is_ok(),
            "NaN serialization should work"
        );
        assert!(
            serde_json::to_string(&inf_secret).is_ok(),
            "Infinity serialization should work"
        );
    }

    #[test]
    fn test_secret_float_deserialization() {
        // Test that deserialization works for functional unwrap
        let original_value = -42.875;
        let secret = SecretFloat::new(original_value);

        // Test JSON round-trip
        let json = serde_json::to_string(&secret).unwrap();
        let deserialized: Result<SecretFloat, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "JSON deserialization should work");

        let restored = deserialized.unwrap();
        assert_eq!(
            restored.reveal(),
            original_value,
            "Round-trip should preserve data"
        );

        // Test bincode round-trip
        let bytes = bincode::serialize(&secret).unwrap();
        let deserialized: Result<SecretFloat, _> = bincode::deserialize(&bytes);
        assert!(deserialized.is_ok(), "Bincode deserialization should work");

        let restored = deserialized.unwrap();
        assert_eq!(
            restored.reveal(),
            original_value,
            "Bincode round-trip should preserve data"
        );
    }
}
