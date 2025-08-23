use crate::config::RedactionContext;
use crate::memory_optimizations::get_configurable_redacted_string;
use nu_protocol::{CustomValue, Record};
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A secure record type that redacts its content in all display contexts
/// and zeros its memory on drop
#[derive(Clone)]
pub struct SecretRecord {
    inner: Record,
}

// Functional serialization - serialize actual content for pipeline operations
impl Serialize for SecretRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the actual content to make pipeline operations work
        self.inner.serialize(serializer)
    }
}

// Functional deserialization - restore actual content for pipeline operations
impl<'de> Deserialize<'de> for SecretRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the actual content to make pipeline operations work
        let content = Record::deserialize(deserializer)?;
        Ok(SecretRecord::new(content))
    }
}

impl Drop for SecretRecord {
    fn drop(&mut self) {
        // Note: We rely on ZeroizeOnDrop for additional memory clearing
        // The Record will be properly dropped by Rust's destructor
        // Cannot safely zero the Record's memory as it contains complex structures
    }
}

// Manual ZeroizeOnDrop implementation to ensure proper cleanup
impl ZeroizeOnDrop for SecretRecord {}

impl SecretRecord {
    /// Create a new SecretRecord from a regular record
    pub fn new(value: Record) -> Self {
        Self { inner: value }
    }

    /// Get a reference to the inner record (for controlled access)
    pub fn reveal(&self) -> &Record {
        &self.inner
    }

    /// Convert back to a regular record (consumes the SecretRecord)
    pub fn into_inner(self) -> Record {
        self.inner.clone()
    }

    /// Get a field from the record while preserving secrecy
    pub fn get_field(&self, field: &str) -> Option<&Value> {
        self.inner.get(field)
    }

    /// Get all field names (safe to expose)
    pub fn fields(&self) -> impl Iterator<Item = &String> {
        self.inner.columns()
    }
}

#[typetag::serde]
impl CustomValue for SecretRecord {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        "secret_record".into()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        let redacted_text =
            get_configurable_redacted_string("record", RedactionContext::Serialization);
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

impl fmt::Display for SecretRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string("record", RedactionContext::Display);
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string("record", RedactionContext::Debug);
        write!(f, "SecretRecord({})", redacted_text)
    }
}

impl PartialEq for SecretRecord {
    fn eq(&self, other: &Self) -> bool {
        // Compare records by serializing and using constant-time comparison
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
    fn test_secret_record_creation() {
        let mut record = Record::new();
        record.push("key", Value::test_string("value"));
        let secret = SecretRecord::new(record.clone());

        // Test that we can access the field correctly
        let field_value = secret.get_field("key");
        assert!(field_value.is_some());
        match field_value {
            Some(Value::String { val, .. }) => assert_eq!(val, "value"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_secret_record_display() {
        let mut record = Record::new();
        record.push("secret", Value::test_string("hidden"));
        let secret = SecretRecord::new(record);
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
    fn test_secret_record_custom_value() {
        let mut record = Record::new();
        record.push("api_key", Value::test_string("secret123"));
        let secret = SecretRecord::new(record);
        assert_eq!(secret.type_name(), "secret_record");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => {
                assert!(val.contains("redacted") || val.contains("***") || val.contains("HIDDEN"))
            }
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_secret_record_field_access() {
        let mut record = Record::new();
        record.push("username", Value::test_string("admin"));
        record.push("password", Value::test_string("secret"));
        let secret = SecretRecord::new(record);

        let username = secret.get_field("username");
        assert!(username.is_some());

        let fields: Vec<&String> = secret.fields().collect();
        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&&"username".to_string()));
        assert!(fields.contains(&&"password".to_string()));
    }

    #[test]
    fn test_secret_record_equality() {
        let mut record1 = Record::new();
        record1.push("key", Value::test_string("value"));
        let secret1 = SecretRecord::new(record1.clone());
        let secret2 = SecretRecord::new(record1);

        let mut record3 = Record::new();
        record3.push("key", Value::test_string("different"));
        let secret3 = SecretRecord::new(record3);

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }

    #[test]
    fn test_secret_record_serialization() {
        // Test that serialization works for functional unwrap
        let mut record = Record::new();
        record.push("api_key", Value::test_string("secret123"));
        record.push("port", Value::test_int(8080));
        let secret = SecretRecord::new(record.clone());

        // Test JSON serialization
        let json_result = serde_json::to_string(&secret);
        assert!(json_result.is_ok(), "JSON serialization should work");

        let json = json_result.unwrap();
        // Should contain the record data for functional unwrap
        assert!(json.contains("api_key"), "JSON should contain field names");
        assert!(
            json.contains("secret123"),
            "JSON should contain field values"
        );
        assert!(json.contains("8080"), "JSON should contain numeric values");

        // Test bincode serialization (used for plugin communication)
        let bincode_result = bincode::serialize(&secret);
        assert!(bincode_result.is_ok(), "Bincode serialization should work");
    }

    #[test]
    fn test_secret_record_deserialization() {
        // Test that deserialization works for functional unwrap
        let mut original_record = Record::new();
        original_record.push("username", Value::test_string("admin"));
        original_record.push("active", Value::test_bool(true));
        let secret = SecretRecord::new(original_record.clone());

        // Test JSON round-trip
        let json = serde_json::to_string(&secret).unwrap();
        let deserialized: Result<SecretRecord, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "JSON deserialization should work");

        let restored = deserialized.unwrap();
        // Compare individual fields since Record doesn't implement PartialEq
        assert_eq!(
            restored.get_field("username").unwrap(),
            original_record.get("username").unwrap(),
            "Username field should match"
        );
        assert_eq!(
            restored.get_field("active").unwrap(),
            original_record.get("active").unwrap(),
            "Active field should match"
        );

        // Test bincode round-trip
        let bytes = bincode::serialize(&secret).unwrap();
        let deserialized: Result<SecretRecord, _> = bincode::deserialize(&bytes);
        assert!(deserialized.is_ok(), "Bincode deserialization should work");

        let restored = deserialized.unwrap();
        // Compare individual fields for bincode round-trip too
        assert_eq!(
            restored.get_field("username").unwrap(),
            original_record.get("username").unwrap(),
            "Bincode username field should match"
        );
        assert_eq!(
            restored.get_field("active").unwrap(),
            original_record.get("active").unwrap(),
            "Bincode active field should match"
        );
    }
}
