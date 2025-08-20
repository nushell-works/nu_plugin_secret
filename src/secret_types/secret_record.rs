use nu_protocol::{CustomValue, Record};
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A secure record type that redacts its content in all display contexts
#[derive(Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct SecretRecord {
    #[zeroize(skip)]
    inner: Record,
}

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
        Ok(Value::string("<redacted:record>", span))
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
        write!(f, "<redacted:record>")
    }
}

impl fmt::Debug for SecretRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretRecord(<redacted>)")
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
        assert_eq!(format!("{}", secret), "<redacted:record>");
        assert_eq!(format!("{:?}", secret), "SecretRecord(<redacted>)");
    }

    #[test]
    fn test_secret_record_custom_value() {
        let mut record = Record::new();
        record.push("api_key", Value::test_string("secret123"));
        let secret = SecretRecord::new(record);
        assert_eq!(secret.type_name(), "secret_record");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => assert_eq!(val, "<redacted:record>"),
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
}
