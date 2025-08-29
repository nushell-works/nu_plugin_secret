use crate::config::RedactionContext;
use crate::memory_optimizations::get_configurable_redacted_string_with_value;
use nu_protocol::ast::{Comparison, Operator};
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

// Functional serialization - serialize actual content for pipeline operations
impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the actual content to make pipeline operations work
        self.inner.serialize(serializer)
    }
}

// Functional deserialization - restore actual content for pipeline operations
impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the actual content to make pipeline operations work
        let content = String::deserialize(deserializer)?;
        Ok(SecretString::new(content))
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

    /// Get length of the secret string (safe to expose)
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the secret string is empty (safe to expose)
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get a partially redacted version of the string if configured
    /// Returns None if partial redaction is not enabled or not applicable
    pub fn partial_redact(&self) -> Option<String> {
        if let Ok(config) = crate::config::get_config() {
            config.apply_partial_redaction(&self.inner, "string")
        } else {
            None
        }
    }

    /// Get redacted string with optional partial redaction
    /// This respects the user's configuration for redaction style
    pub fn redacted_display(&self) -> String {
        // First try partial redaction if configured
        if let Some(partial) = self.partial_redact() {
            return partial;
        }

        // Fall back to configured full redaction (with potential unredacted mode)
        get_configurable_redacted_string_with_value(
            "string",
            RedactionContext::Display,
            Some(&self.inner),
        )
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
        let redacted_text = get_configurable_redacted_string_with_value(
            "string",
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

    fn operation(
        &self,
        lhs_span: Span,
        operator: Operator,
        op: Span,
        right: &Value,
    ) -> Result<Value, ShellError> {
        match operator {
            Operator::Comparison(Comparison::Equal) => {
                if let Value::Custom { val, .. } = right {
                    if let Some(other_secret) = val.as_any().downcast_ref::<SecretString>() {
                        // Use our existing PartialEq implementation for comparison
                        let result = self == other_secret;
                        Ok(Value::bool(result, lhs_span))
                    } else {
                        // Different custom type, so not equal
                        Ok(Value::bool(false, lhs_span))
                    }
                } else {
                    // Comparing with non-custom value, so not equal
                    Ok(Value::bool(false, lhs_span))
                }
            }
            Operator::Comparison(Comparison::NotEqual) => {
                if let Value::Custom { val, .. } = right {
                    if let Some(other_secret) = val.as_any().downcast_ref::<SecretString>() {
                        // Use our existing PartialEq implementation for comparison
                        let result = self != other_secret;
                        Ok(Value::bool(result, lhs_span))
                    } else {
                        // Different custom type, so not equal (therefore not-equal is true)
                        Ok(Value::bool(true, lhs_span))
                    }
                } else {
                    // Comparing with non-custom value, so not equal (therefore not-equal is true)
                    Ok(Value::bool(true, lhs_span))
                }
            }
            _ => Err(ShellError::GenericError {
                error: format!("Operator {:?} is not supported for secret_string", operator),
                msg: "".to_string(),
                span: Some(op),
                help: None,
                inner: vec![],
            }),
        }
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string_with_value(
            "string",
            RedactionContext::Display,
            Some(&self.inner),
        );
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string_with_value(
            "string",
            RedactionContext::Debug,
            Some(&self.inner),
        );
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

    #[test]
    fn test_secret_string_length_and_emptiness() {
        // Test non-empty string
        let secret = SecretString::new("hello".to_string());
        assert_eq!(secret.len(), 5);
        assert!(!secret.is_empty());

        // Test empty string
        let empty_secret = SecretString::new("".to_string());
        assert_eq!(empty_secret.len(), 0);
        assert!(empty_secret.is_empty());

        // Test string with unicode characters
        let unicode_secret = SecretString::new("café".to_string());
        assert_eq!(unicode_secret.len(), 5); // 'é' is 2 bytes in UTF-8
        assert!(!unicode_secret.is_empty());
    }

    #[test]
    fn test_secret_string_clone() {
        let original = SecretString::new("secret-value".to_string());
        let cloned = original.clone();

        assert_eq!(original.reveal(), cloned.reveal());
        assert_eq!(original, cloned);

        // Verify they are separate instances
        assert_eq!(original.len(), cloned.len());
    }

    #[test]
    fn test_secret_string_clone_value() {
        let secret = SecretString::new("test-value".to_string());
        let span = Span::test_data();
        let cloned_value = secret.clone_value(span);

        // Should return a custom value
        match cloned_value {
            Value::Custom { val, .. } => {
                // Should be able to downcast back to SecretString
                assert!(val.as_any().downcast_ref::<SecretString>().is_some());
            }
            _ => panic!("Expected custom value"),
        }
    }

    #[test]
    fn test_secret_string_redacted_display() {
        let secret = SecretString::new("sensitive-data".to_string());
        let redacted = secret.redacted_display();

        // Should be redacted (not contain the actual value)
        assert!(!redacted.contains("sensitive-data"));
        assert!(
            redacted.contains("redacted")
                || redacted.contains("***")
                || redacted.contains("HIDDEN")
        );
    }

    #[test]
    fn test_secret_string_partial_redact() {
        let secret = SecretString::new("test-secret".to_string());

        // Without specific config, this should return None
        // (since we can't easily mock the config in unit tests)
        let partial = secret.partial_redact();

        // The method should not panic and should handle config errors gracefully
        // The return value depends on the configuration state
        match partial {
            Some(redacted) => {
                // If partial redaction is configured, verify it doesn't expose full secret
                assert!(!redacted.contains("test-secret") || redacted.len() < "test-secret".len());
            }
            None => {
                // No partial redaction configured - this is acceptable
            }
        }
    }

    #[test]
    fn test_secret_string_serialization() {
        let secret = SecretString::new("serialize-me".to_string());

        // Test JSON serialization
        let json_result = serde_json::to_string(&secret);
        assert!(json_result.is_ok(), "JSON serialization should work");

        let json = json_result.unwrap();
        // Should serialize the actual content for functional operations
        assert!(json.contains("serialize-me"));

        // Test bincode serialization
        let bincode_result = bincode::serialize(&secret);
        assert!(bincode_result.is_ok(), "Bincode serialization should work");
    }

    #[test]
    fn test_secret_string_deserialization() {
        let original_secret = SecretString::new("deserialize-me".to_string());

        // Test JSON round-trip
        let json = serde_json::to_string(&original_secret).unwrap();
        let deserialized: Result<SecretString, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "JSON deserialization should work");

        let restored = deserialized.unwrap();
        assert_eq!(original_secret, restored);
        assert_eq!(restored.reveal(), "deserialize-me");

        // Test bincode round-trip
        let bytes = bincode::serialize(&original_secret).unwrap();
        let deserialized: Result<SecretString, _> = bincode::deserialize(&bytes);
        assert!(deserialized.is_ok(), "Bincode deserialization should work");

        let restored = deserialized.unwrap();
        assert_eq!(original_secret, restored);
        assert_eq!(restored.reveal(), "deserialize-me");
    }

    #[test]
    fn test_secret_string_equality_edge_cases() {
        // Test empty strings
        let empty1 = SecretString::new("".to_string());
        let empty2 = SecretString::new("".to_string());
        assert_eq!(empty1, empty2);

        // Test different lengths
        let short = SecretString::new("a".to_string());
        let long = SecretString::new("abcd".to_string());
        assert_ne!(short, long);

        // Test same length, different content
        let secret1 = SecretString::new("abcd".to_string());
        let secret2 = SecretString::new("efgh".to_string());
        assert_ne!(secret1, secret2);

        // Test unicode strings
        let unicode1 = SecretString::new("café".to_string());
        let unicode2 = SecretString::new("café".to_string());
        assert_eq!(unicode1, unicode2);
    }

    #[test]
    fn test_secret_string_unicode_support() {
        let unicode_secret = SecretString::new("Hello 世界 🌍 café".to_string());

        assert_eq!(unicode_secret.reveal(), "Hello 世界 🌍 café");
        assert!(!unicode_secret.is_empty());
        assert!(unicode_secret.len() > 10); // Unicode chars take more bytes

        let display = format!("{}", unicode_secret);
        assert!(
            display.contains("redacted") || display.contains("***") || display.contains("HIDDEN")
        );
    }

    #[test]
    fn test_secret_string_large_content() {
        let large_content = "x".repeat(10000);
        let secret = SecretString::new(large_content.clone());

        assert_eq!(secret.len(), 10000);
        assert_eq!(secret.reveal(), large_content);
        assert!(!secret.is_empty());

        // Should still redact large content
        let display = format!("{}", secret);
        assert!(
            display.contains("redacted") || display.contains("***") || display.contains("HIDDEN")
        );
    }

    #[test]
    fn test_secret_string_operation_equal() {
        let secret1 = SecretString::new("same-value".to_string());
        let secret2 = SecretString::new("same-value".to_string());
        let secret3 = SecretString::new("different-value".to_string());

        let span = Span::test_data();
        let op_span = Span::test_data();

        // Test equal values
        let result = secret1.operation(
            span,
            Operator::Comparison(Comparison::Equal),
            op_span,
            &Value::custom(Box::new(secret2), span),
        );
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Bool { val, .. } => assert!(val),
            _ => panic!("Expected boolean result"),
        }

        // Test different values
        let result = secret1.operation(
            span,
            Operator::Comparison(Comparison::Equal),
            op_span,
            &Value::custom(Box::new(secret3), span),
        );
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Bool { val, .. } => assert!(!val),
            _ => panic!("Expected boolean result"),
        }
    }

    #[test]
    fn test_secret_string_operation_not_equal() {
        let secret1 = SecretString::new("value1".to_string());
        let secret2 = SecretString::new("value2".to_string());

        let span = Span::test_data();
        let op_span = Span::test_data();

        let result = secret1.operation(
            span,
            Operator::Comparison(Comparison::NotEqual),
            op_span,
            &Value::custom(Box::new(secret2), span),
        );
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Bool { val, .. } => assert!(val),
            _ => panic!("Expected boolean result"),
        }
    }

    #[test]
    fn test_secret_string_operation_with_different_types() {
        let secret = SecretString::new("test".to_string());
        let span = Span::test_data();
        let op_span = Span::test_data();

        // Test comparison with non-custom value
        let result = secret.operation(
            span,
            Operator::Comparison(Comparison::Equal),
            op_span,
            &Value::string("test", span),
        );
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Bool { val, .. } => assert!(!val), // Different types are never equal
            _ => panic!("Expected boolean result"),
        }

        // Test comparison with different custom type
        use crate::SecretInt;
        let int_secret = SecretInt::new(42);
        let result = secret.operation(
            span,
            Operator::Comparison(Comparison::Equal),
            op_span,
            &Value::custom(Box::new(int_secret), span),
        );
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Bool { val, .. } => assert!(!val), // Different custom types are never equal
            _ => panic!("Expected boolean result"),
        }
    }

    #[test]
    fn test_secret_string_operation_unsupported() {
        let secret = SecretString::new("test".to_string());
        let span = Span::test_data();
        let op_span = Span::test_data();

        // Test unsupported operator
        let result = secret.operation(
            span,
            Operator::Math(nu_protocol::ast::Math::Add),
            op_span,
            &Value::string("other", span),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ShellError::GenericError { error, .. } => {
                assert!(error.contains("not supported for secret_string"));
            }
            _ => panic!("Expected GenericError"),
        }
    }

    #[test]
    fn test_secret_string_as_any() {
        let secret = SecretString::new("test".to_string());

        // Test as_any
        let any_ref = secret.as_any();
        assert!(any_ref.downcast_ref::<SecretString>().is_some());

        // Test as_mut_any
        let mut secret_mut = SecretString::new("test".to_string());
        let any_mut_ref = secret_mut.as_mut_any();
        assert!(any_mut_ref.downcast_mut::<SecretString>().is_some());
    }

    #[test]
    fn test_secret_string_notify_plugin_on_drop() {
        let secret = SecretString::new("test".to_string());
        assert!(!secret.notify_plugin_on_drop());
    }

    #[test]
    fn test_secret_string_special_characters() {
        let special_chars = "!@#$%^&*()_+-=[]{}|;':\"<>?,./`~".to_string();
        let secret = SecretString::new(special_chars.clone());

        assert_eq!(secret.reveal(), special_chars);
        assert_eq!(secret.len(), special_chars.len());

        let display = format!("{}", secret);
        assert!(!display.contains(&special_chars)); // Should not leak special chars
    }

    #[test]
    fn test_secret_string_whitespace_handling() {
        // Test strings with various whitespace
        let whitespace_string = " \t\n\r leading and trailing \t\n\r ".to_string();
        let secret = SecretString::new(whitespace_string.clone());

        assert_eq!(secret.reveal(), whitespace_string);
        assert_eq!(secret.len(), whitespace_string.len());
        assert!(!secret.is_empty());
    }

    #[test]
    fn test_secret_string_memory_cleanup() {
        // This test ensures Drop trait is implemented and called
        // We can't directly test memory zeroing, but we can test that Drop doesn't panic
        let secret = SecretString::new("will-be-dropped".to_string());
        drop(secret); // Explicit drop should not panic

        // Test with empty string
        let empty_secret = SecretString::new("".to_string());
        drop(empty_secret);

        // Test with unicode
        let unicode_secret = SecretString::new("café 🌍".to_string());
        drop(unicode_secret);
    }
}
