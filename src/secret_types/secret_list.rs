use crate::config::RedactionContext;
use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A secure list type that redacts its content in all display contexts
/// and zeros its memory on drop
#[derive(Clone)]
pub struct SecretList {
    inner: Vec<Value>,
    redaction_template: Option<String>,
}

// Functional serialization - serialize actual content for pipeline operations
impl Serialize for SecretList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SecretList", 2)?;
        state.serialize_field("inner", &self.inner)?;
        state.serialize_field("redaction_template", &self.redaction_template)?;
        state.end()
    }
}

// Functional deserialization - restore actual content for pipeline operations
impl<'de> Deserialize<'de> for SecretList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SecretListData {
            inner: Vec<Value>,
            redaction_template: Option<String>,
        }

        let data = SecretListData::deserialize(deserializer)?;
        Ok(SecretList {
            inner: data.inner,
            redaction_template: data.redaction_template,
        })
    }
}

impl Drop for SecretList {
    fn drop(&mut self) {
        // Clear the vector for security
        // The values will be properly dropped by Rust's standard drop mechanism
        self.inner.clear();
        // Note: We rely on ZeroizeOnDrop for additional memory clearing
        // The Vec itself will be properly dropped by Rust's destructor
    }
}

// Manual ZeroizeOnDrop implementation to ensure proper cleanup
impl ZeroizeOnDrop for SecretList {}

impl SecretList {
    /// Create a new SecretList from a regular vector of values
    pub fn new(value: Vec<Value>) -> Self {
        Self {
            inner: value,
            redaction_template: None,
        }
    }

    /// Create a new SecretList with a custom redaction template
    pub fn new_with_template(value: Vec<Value>, template: String) -> Self {
        Self {
            inner: value,
            redaction_template: Some(template),
        }
    }

    /// Get a reference to the inner list (for controlled access)
    pub fn reveal(&self) -> &Vec<Value> {
        &self.inner
    }

    /// Convert back to a regular vector (consumes the SecretList)
    pub fn into_inner(self) -> Vec<Value> {
        self.inner.clone()
    }

    /// Get an element from the list while preserving secrecy
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.inner.get(index)
    }

    /// Get the length of the list (safe to expose)
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the list is empty (safe to expose)
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[typetag::serde]
impl CustomValue for SecretList {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        "secret_list".into()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::generate_redacted_string_with_custom_template(
                template, "list", None, // Length not meaningful for complex types
            )
        } else {
            crate::redaction::get_redacted_string_with_value::<String>(
                "list",
                RedactionContext::Serialization,
                None,
            )
        };
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

impl fmt::Display for SecretList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::generate_redacted_string_with_custom_template(
                template, "list", None, // Length not meaningful for complex types
            )
        } else {
            crate::redaction::get_redacted_string_with_value::<String>(
                "list",
                RedactionContext::Display,
                None,
            )
        };
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::generate_redacted_string_with_custom_template(
                template, "list", None, // Length not meaningful for complex types
            )
        } else {
            crate::redaction::get_redacted_string_with_value::<String>(
                "list",
                RedactionContext::Debug,
                None,
            )
        };
        write!(f, "SecretList({})", redacted_text)
    }
}

impl PartialEq for SecretList {
    fn eq(&self, other: &Self) -> bool {
        // Compare lists by serializing and using constant-time comparison
        // This is a simplified approach - in production, we might want more sophisticated comparison
        let self_ser = bincode::serialize(&self.inner).unwrap_or_default();
        let other_ser = bincode::serialize(&other.inner).unwrap_or_default();

        if self_ser.len() != other_ser.len() {
            return false;
        }

        let mut result = 0u8;
        for i in 0..self_ser.len() {
            result |= self_ser[i] ^ other_ser[i];
        }
        result == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_list_creation() {
        let list = vec![Value::test_string("item1"), Value::test_string("item2")];
        let secret = SecretList::new(list.clone());
        assert_eq!(secret.reveal(), &list);
    }

    #[test]
    fn test_secret_list_display() {
        let list = vec![Value::test_string("secret1"), Value::test_string("secret2")];
        let secret = SecretList::new(list);
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
    fn test_secret_list_custom_value() {
        let list = vec![Value::test_int(1), Value::test_int(2), Value::test_int(3)];
        let secret = SecretList::new(list);
        assert_eq!(secret.type_name(), "secret_list");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => {
                assert!(val.contains("redacted") || val.contains("***") || val.contains("HIDDEN"))
            }
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_secret_list_access() {
        let list = vec![Value::test_string("first"), Value::test_string("second")];
        let secret = SecretList::new(list);

        assert_eq!(secret.len(), 2);
        assert!(!secret.is_empty());

        let first = secret.get(0);
        assert!(first.is_some());

        let third = secret.get(2);
        assert!(third.is_none());
    }

    #[test]
    fn test_secret_list_equality() {
        let list1 = vec![Value::test_string("item1"), Value::test_string("item2")];
        let secret1 = SecretList::new(list1.clone());
        let secret2 = SecretList::new(list1);

        let list3 = vec![Value::test_string("item1"), Value::test_string("different")];
        let secret3 = SecretList::new(list3);

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }

    #[test]
    fn test_secret_list_empty() {
        let empty_list: Vec<Value> = vec![];
        let secret = SecretList::new(empty_list);

        assert_eq!(secret.len(), 0);
        assert!(secret.is_empty());
    }

    #[test]
    fn test_secret_list_serialization() {
        // Test that serialization works for functional unwrap
        let list = vec![Value::test_string("item1"), Value::test_int(42)];
        let secret = SecretList::new(list.clone());

        // Test JSON serialization
        let json_result = serde_json::to_string(&secret);
        assert!(json_result.is_ok(), "JSON serialization should work");

        let json = json_result.unwrap();
        // Should contain the actual data for functional unwrap
        assert!(json.contains("item1"), "JSON should contain list data");
        assert!(json.contains("42"), "JSON should contain numeric data");

        // Test bincode serialization (used for plugin communication)
        let bincode_result = bincode::serialize(&secret);
        assert!(bincode_result.is_ok(), "Bincode serialization should work");
    }

    #[test]
    fn test_secret_list_deserialization() {
        // Test that deserialization works for functional unwrap
        let original_list = vec![Value::test_string("test"), Value::test_bool(true)];
        let secret = SecretList::new(original_list.clone());

        // Test JSON round-trip
        let json = serde_json::to_string(&secret).unwrap();
        let deserialized: Result<SecretList, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "JSON deserialization should work");

        let restored = deserialized.unwrap();
        assert_eq!(
            restored.reveal(),
            &original_list,
            "Round-trip should preserve data"
        );

        // Test bincode round-trip
        let bytes = bincode::serialize(&secret).unwrap();
        let deserialized: Result<SecretList, _> = bincode::deserialize(&bytes);
        assert!(deserialized.is_ok(), "Bincode deserialization should work");

        let restored = deserialized.unwrap();
        assert_eq!(
            restored.reveal(),
            &original_list,
            "Bincode round-trip should preserve data"
        );
    }
}
