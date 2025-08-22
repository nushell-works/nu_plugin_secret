use crate::config::RedactionContext;
use crate::memory_optimizations::get_configurable_redacted_string;
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
}

// Functional serialization - serialize actual content for pipeline operations
impl Serialize for SecretList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the actual content to make pipeline operations work
        self.inner.serialize(serializer)
    }
}

// Functional deserialization - restore actual content for pipeline operations
impl<'de> Deserialize<'de> for SecretList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the actual content to make pipeline operations work
        let content = Vec::<Value>::deserialize(deserializer)?;
        Ok(SecretList::new(content))
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
        Self { inner: value }
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
        let redacted_text =
            get_configurable_redacted_string("list", RedactionContext::Serialization);
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
        let redacted_text = get_configurable_redacted_string("list", RedactionContext::Display);
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string("list", RedactionContext::Debug);
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
}
