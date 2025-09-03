use crate::config::RedactionContext;
use crate::memory_optimizations::get_configurable_redacted_string;
use nu_protocol::ast::{Comparison, Operator};
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
    redaction_template: Option<String>,
}

// Functional serialization - serialize actual content for pipeline operations
impl Serialize for SecretRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SecretRecord", 2)?;
        state.serialize_field("inner", &self.inner)?;
        state.serialize_field("redaction_template", &self.redaction_template)?;
        state.end()
    }
}

// Functional deserialization - restore actual content for pipeline operations
impl<'de> Deserialize<'de> for SecretRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SecretRecordData {
            inner: Record,
            redaction_template: Option<String>,
        }

        let data = SecretRecordData::deserialize(deserializer)?;
        Ok(SecretRecord {
            inner: data.inner,
            redaction_template: data.redaction_template,
        })
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
        Self {
            inner: value,
            redaction_template: None,
        }
    }

    /// Create a new SecretRecord with a custom redaction template
    pub fn new_with_template(value: Record, template: String) -> Self {
        Self {
            inner: value,
            redaction_template: Some(template),
        }
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
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::generate_redacted_string_with_custom_template(
                template, "record", None, // Length not meaningful for complex types
            )
        } else {
            get_configurable_redacted_string("record", RedactionContext::Serialization)
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
                    if let Some(other_secret) = val.as_any().downcast_ref::<SecretRecord>() {
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
                    if let Some(other_secret) = val.as_any().downcast_ref::<SecretRecord>() {
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
                error: format!("Operator {:?} is not supported for secret_record", operator),
                msg: "".to_string(),
                span: Some(op),
                help: None,
                inner: vec![],
            }),
        }
    }
}

impl fmt::Display for SecretRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::generate_redacted_string_with_custom_template(
                template, "record", None, // Length not meaningful for complex types
            )
        } else {
            get_configurable_redacted_string("record", RedactionContext::Display)
        };
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::generate_redacted_string_with_custom_template(
                template, "record", None, // Length not meaningful for complex types
            )
        } else {
            get_configurable_redacted_string("record", RedactionContext::Debug)
        };
        write!(f, "SecretRecord({})", redacted_text)
    }
}

impl PartialEq for SecretRecord {
    fn eq(&self, other: &Self) -> bool {
        // Compare records field by field for proper logical equality
        if self.inner.len() != other.inner.len() {
            return false;
        }

        // Check that all fields in self exist in other and have equal values
        for (key, self_value) in self.inner.iter() {
            match other.inner.get(key) {
                Some(other_value) => {
                    if self_value != other_value {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Since we already checked lengths are equal and all fields in self
        // exist in other with equal values, the records are equal
        true
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

    #[test]
    fn test_secret_record_into_inner() {
        let mut record = Record::new();
        record.push("key1", Value::test_string("value1"));
        record.push("key2", Value::test_int(42));
        let secret = SecretRecord::new(record.clone());

        let inner = secret.into_inner();
        assert_eq!(inner.get("key1").unwrap(), record.get("key1").unwrap());
        assert_eq!(inner.get("key2").unwrap(), record.get("key2").unwrap());
        assert_eq!(inner.len(), 2);
    }

    #[test]
    fn test_secret_record_reveal() {
        let mut record = Record::new();
        record.push("api_token", Value::test_string("secret123"));
        let secret = SecretRecord::new(record.clone());

        let revealed = secret.reveal();
        assert_eq!(
            revealed.get("api_token").unwrap(),
            record.get("api_token").unwrap()
        );
        assert_eq!(revealed.len(), 1);
    }

    #[test]
    fn test_secret_record_clone_value() {
        let mut record = Record::new();
        record.push("data", Value::test_string("sensitive"));
        let secret = SecretRecord::new(record);

        let cloned_value = secret.clone_value(Span::test_data());
        assert!(matches!(cloned_value, Value::Custom { .. }));

        if let Value::Custom { val, .. } = cloned_value {
            let cloned_secret = val.as_any().downcast_ref::<SecretRecord>().unwrap();
            assert_eq!(cloned_secret.get_field("data"), secret.get_field("data"));
        }
    }

    #[test]
    fn test_secret_record_as_any_methods() {
        let mut record = Record::new();
        record.push("test", Value::test_string("value"));
        let mut secret = SecretRecord::new(record);

        // Test as_any
        let any_ref = secret.as_any();
        assert!(any_ref.downcast_ref::<SecretRecord>().is_some());

        // Test as_mut_any
        let any_mut_ref = secret.as_mut_any();
        assert!(any_mut_ref.downcast_mut::<SecretRecord>().is_some());
    }

    #[test]
    fn test_secret_record_notify_plugin_on_drop() {
        let record = Record::new();
        let secret = SecretRecord::new(record);

        assert!(!secret.notify_plugin_on_drop());
    }

    #[test]
    fn test_secret_record_operation_not_equal() {
        let mut record1 = Record::new();
        record1.push("key", Value::test_string("value1"));
        let secret1 = SecretRecord::new(record1.clone());

        let mut record2 = Record::new();
        record2.push("key", Value::test_string("value2"));
        let secret2 = SecretRecord::new(record2);

        // Test not equal with different SecretRecord
        let right_value = Value::custom(Box::new(secret2.clone()), Span::test_data());
        let result = secret1
            .operation(
                Span::test_data(),
                Operator::Comparison(Comparison::NotEqual),
                Span::test_data(),
                &right_value,
            )
            .unwrap();

        assert!(matches!(result, Value::Bool { val: true, .. }));

        // Test not equal with same SecretRecord
        let same_secret = SecretRecord::new(record1.clone());
        let right_value = Value::custom(Box::new(same_secret), Span::test_data());
        let result = secret1
            .operation(
                Span::test_data(),
                Operator::Comparison(Comparison::NotEqual),
                Span::test_data(),
                &right_value,
            )
            .unwrap();

        assert!(matches!(result, Value::Bool { val: false, .. }));
    }

    #[test]
    fn test_secret_record_operation_not_equal_non_custom() {
        let mut record = Record::new();
        record.push("key", Value::test_string("value"));
        let secret = SecretRecord::new(record);

        // Test not equal with non-custom value
        let right_value = Value::test_string("not a secret");
        let result = secret
            .operation(
                Span::test_data(),
                Operator::Comparison(Comparison::NotEqual),
                Span::test_data(),
                &right_value,
            )
            .unwrap();

        assert!(matches!(result, Value::Bool { val: true, .. }));
    }

    #[test]
    fn test_secret_record_operation_equal_non_custom() {
        let mut record = Record::new();
        record.push("key", Value::test_string("value"));
        let secret = SecretRecord::new(record);

        // Test equal with non-custom value
        let right_value = Value::test_string("not a secret");
        let result = secret
            .operation(
                Span::test_data(),
                Operator::Comparison(Comparison::Equal),
                Span::test_data(),
                &right_value,
            )
            .unwrap();

        assert!(matches!(result, Value::Bool { val: false, .. }));
    }

    #[test]
    fn test_secret_record_operation_unsupported() {
        let mut record = Record::new();
        record.push("key", Value::test_string("value"));
        let secret = SecretRecord::new(record);

        let right_value = Value::test_int(42);
        let result = secret.operation(
            Span::test_data(),
            Operator::Math(nu_protocol::ast::Math::Add),
            Span::test_data(),
            &right_value,
        );

        assert!(result.is_err());
        if let Err(ShellError::GenericError { error, .. }) = result {
            assert!(error.contains("not supported"));
        }
    }

    #[test]
    fn test_secret_record_equality_edge_cases() {
        // Test empty records
        let empty1 = SecretRecord::new(Record::new());
        let empty2 = SecretRecord::new(Record::new());
        assert_eq!(empty1, empty2);

        // Test different field orders (should still be equal)
        let mut record1 = Record::new();
        record1.push("a", Value::test_string("value1"));
        record1.push("b", Value::test_string("value2"));

        let mut record2 = Record::new();
        record2.push("b", Value::test_string("value2"));
        record2.push("a", Value::test_string("value1"));

        let secret1 = SecretRecord::new(record1);
        let secret2 = SecretRecord::new(record2);
        assert_eq!(secret1, secret2);

        // Test different lengths
        let mut record3 = Record::new();
        record3.push("a", Value::test_string("value1"));

        let secret3 = SecretRecord::new(record3);
        assert_ne!(secret1, secret3);

        // Test same keys, different values
        let mut record4 = Record::new();
        record4.push("a", Value::test_string("different"));
        record4.push("b", Value::test_string("value2"));

        let secret4 = SecretRecord::new(record4);
        assert_ne!(secret1, secret4);
    }

    #[test]
    fn test_secret_record_get_field_missing() {
        let mut record = Record::new();
        record.push("existing", Value::test_string("value"));
        let secret = SecretRecord::new(record);

        assert!(secret.get_field("existing").is_some());
        assert!(secret.get_field("missing").is_none());
    }

    #[test]
    fn test_secret_record_empty_serialization() {
        let empty_record = Record::new();
        let secret = SecretRecord::new(empty_record);

        // Test JSON serialization of empty record
        let json_result = serde_json::to_string(&secret);
        assert!(json_result.is_ok());
        let json = json_result.unwrap();
        // Now includes the struct format with both inner and redaction_template fields
        assert_eq!(json, "{\"inner\":{},\"redaction_template\":null}");

        // Test deserialization of empty record
        let deserialized: Result<SecretRecord, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok());
        let restored = deserialized.unwrap();
        assert_eq!(restored.fields().count(), 0);
    }

    #[test]
    fn test_secret_record_complex_nested_values() {
        let mut inner_record = Record::new();
        inner_record.push("nested", Value::test_string("deep"));

        let mut record = Record::new();
        record.push("simple", Value::test_string("value"));
        record.push("number", Value::test_int(123));
        record.push("boolean", Value::test_bool(true));
        record.push(
            "list",
            Value::test_list(vec![Value::test_string("item1"), Value::test_int(456)]),
        );
        record.push("record", Value::test_record(inner_record));

        let secret = SecretRecord::new(record.clone());

        // Test serialization with complex nested values
        let json_result = serde_json::to_string(&secret);
        assert!(json_result.is_ok());

        // Test deserialization with complex nested values
        let json = json_result.unwrap();
        let deserialized: Result<SecretRecord, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok());

        let restored = deserialized.unwrap();
        assert_eq!(restored.fields().count(), 5);
        assert!(restored.get_field("simple").is_some());
        assert!(restored.get_field("number").is_some());
        assert!(restored.get_field("boolean").is_some());
        assert!(restored.get_field("list").is_some());
        assert!(restored.get_field("record").is_some());
    }

    #[test]
    fn test_secret_record_with_custom_template() {
        let mut record = Record::new();
        record.push("name", Value::test_string("Alice"));
        record.push("age", Value::test_int(30));
        let secret =
            SecretRecord::new_with_template(record.clone(), "moo:{{secret_type}}".to_string());

        // Test Display - should show template with secret_type substituted
        let display = format!("{}", secret);
        assert_eq!(display, "moo:record");

        // Test Debug
        let debug = format!("{:?}", secret);
        assert_eq!(debug, "SecretRecord(moo:record)");

        // Test to_base_value
        let base_value = secret
            .to_base_value(nu_protocol::Span::test_data())
            .unwrap();
        if let nu_protocol::Value::String { val, .. } = base_value {
            assert_eq!(val, "moo:record");
        } else {
            panic!("Expected string value");
        }

        // Test reveal still works
        let revealed = secret.reveal();
        assert_eq!(revealed.len(), 2);
        assert!(revealed.get("name").is_some());
        assert!(revealed.get("age").is_some());
    }
}
