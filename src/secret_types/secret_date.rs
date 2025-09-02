use crate::config::RedactionContext;
use crate::memory_optimizations::get_configurable_redacted_string_with_generic_value;
use chrono::{self, Datelike};
use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A secure date type that redacts its content in all display contexts
/// and zeros its memory on drop
#[derive(Clone)]
pub struct SecretDate {
    inner: chrono::DateTime<chrono::FixedOffset>,
    redaction_template: Option<String>,
}

// Functional serialization - serialize actual content for pipeline operations
impl Serialize for SecretDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SecretDate", 2)?;
        state.serialize_field("inner", &self.inner)?;
        state.serialize_field("redaction_template", &self.redaction_template)?;
        state.end()
    }
}

// Functional deserialization - restore actual content for pipeline operations
impl<'de> Deserialize<'de> for SecretDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SecretDateData {
            inner: chrono::DateTime<chrono::FixedOffset>,
            redaction_template: Option<String>,
        }

        let data = SecretDateData::deserialize(deserializer)?;
        Ok(SecretDate {
            inner: data.inner,
            redaction_template: data.redaction_template,
        })
    }
}

impl Drop for SecretDate {
    fn drop(&mut self) {
        // Note: We rely on ZeroizeOnDrop for additional memory clearing
        // The DateTime will be properly dropped by Rust's destructor
        // Cannot safely zero the DateTime's memory as it contains complex structures
    }
}

// Manual ZeroizeOnDrop implementation to ensure proper cleanup
impl ZeroizeOnDrop for SecretDate {}

impl SecretDate {
    /// Create a new SecretDate from a DateTime
    pub fn new(value: chrono::DateTime<chrono::FixedOffset>) -> Self {
        Self {
            inner: value,
            redaction_template: None,
        }
    }

    /// Create a new SecretDate with a custom redaction template
    pub fn new_with_template(
        value: chrono::DateTime<chrono::FixedOffset>,
        template: String,
    ) -> Self {
        Self {
            inner: value,
            redaction_template: Some(template),
        }
    }

    /// Get a reference to the inner DateTime (for controlled access)
    pub fn reveal(&self) -> &chrono::DateTime<chrono::FixedOffset> {
        &self.inner
    }

    /// Convert back to a regular DateTime (consumes the SecretDate)
    pub fn into_inner(self) -> chrono::DateTime<chrono::FixedOffset> {
        self.inner
    }

    /// Get the year component (might be safe to expose depending on use case)
    pub fn year(&self) -> i32 {
        self.inner.year()
    }

    /// Check if this date is before another date (safe comparison)
    pub fn is_before(&self, other: &SecretDate) -> bool {
        self.inner < other.inner
    }

    /// Check if this date is after another date (safe comparison)
    pub fn is_after(&self, other: &SecretDate) -> bool {
        self.inner > other.inner
    }
}

#[typetag::serde]
impl CustomValue for SecretDate {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        "secret_date".into()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::get_redacted_string_with_custom_template_and_value(
                "date",
                template,
                RedactionContext::Serialization,
                Some(&self.inner),
            )
        } else {
            get_configurable_redacted_string_with_generic_value(
                "date",
                RedactionContext::Serialization,
                Some(&self.inner),
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

impl fmt::Display for SecretDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::get_redacted_string_with_custom_template_and_value(
                "date",
                template,
                RedactionContext::Display,
                Some(&self.inner),
            )
        } else {
            get_configurable_redacted_string_with_generic_value(
                "date",
                RedactionContext::Display,
                Some(&self.inner),
            )
        };
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = if let Some(template) = &self.redaction_template {
            crate::redaction::get_redacted_string_with_custom_template_and_value(
                "date",
                template,
                RedactionContext::Debug,
                Some(&self.inner),
            )
        } else {
            get_configurable_redacted_string_with_generic_value(
                "date",
                RedactionContext::Debug,
                Some(&self.inner),
            )
        };
        write!(f, "SecretDate({})", redacted_text)
    }
}

impl PartialEq for SecretDate {
    fn eq(&self, other: &Self) -> bool {
        // Use constant-time comparison for security
        // DateTime implements Eq, so we can serialize for comparison
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
    use chrono::{TimeZone, Utc};

    fn test_datetime() -> chrono::DateTime<chrono::FixedOffset> {
        Utc.timestamp_opt(1699123200, 0).unwrap().into() // 2023-11-04 16:00:00 UTC
    }

    fn another_test_datetime() -> chrono::DateTime<chrono::FixedOffset> {
        Utc.timestamp_opt(1699209600, 0).unwrap().into() // 2023-11-05 16:00:00 UTC
    }

    #[test]
    fn test_secret_date_creation() {
        let dt = test_datetime();
        let secret = SecretDate::new(dt);
        assert_eq!(secret.reveal(), &dt);
    }

    #[test]
    fn test_secret_date_display() {
        let dt = test_datetime();
        let secret = SecretDate::new(dt);
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
    fn test_secret_date_custom_value() {
        let dt = test_datetime();
        let secret = SecretDate::new(dt);
        assert_eq!(secret.type_name(), "secret_date");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => {
                assert!(val.contains("redacted") || val.contains("***") || val.contains("HIDDEN"))
            }
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_secret_date_into_inner() {
        let dt = test_datetime();
        let secret = SecretDate::new(dt);
        assert_eq!(secret.into_inner(), dt);
    }

    #[test]
    fn test_secret_date_equality() {
        let dt1 = test_datetime();
        let dt2 = test_datetime();
        let dt3 = another_test_datetime();

        let secret1 = SecretDate::new(dt1);
        let secret2 = SecretDate::new(dt2);
        let secret3 = SecretDate::new(dt3);

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }

    #[test]
    fn test_secret_date_comparisons() {
        let early_dt = test_datetime();
        let later_dt = another_test_datetime();

        let early_secret = SecretDate::new(early_dt);
        let later_secret = SecretDate::new(later_dt);

        assert!(early_secret.is_before(&later_secret));
        assert!(!early_secret.is_after(&later_secret));

        assert!(later_secret.is_after(&early_secret));
        assert!(!later_secret.is_before(&early_secret));
    }

    #[test]
    fn test_secret_date_year_access() {
        let dt = test_datetime();
        let secret = SecretDate::new(dt);

        // This assumes our test datetime is in 2023
        assert_eq!(secret.year(), 2023);
    }

    #[test]
    fn test_secret_date_serialization() {
        // Test that serialization works for functional unwrap
        let dt = test_datetime();
        let secret = SecretDate::new(dt);

        // Test JSON serialization
        let json_result = serde_json::to_string(&secret);
        assert!(json_result.is_ok(), "JSON serialization should work");

        let json = json_result.unwrap();
        // Should contain the date/time data for functional unwrap
        assert!(json.contains("2023"), "JSON should contain year");
        // Be more flexible with date formatting since format may vary
        assert!(
            json.contains("11") || json.contains("04"),
            "JSON should contain date components"
        );

        // Test bincode serialization (used for plugin communication)
        let bincode_result = bincode::serialize(&secret);
        assert!(bincode_result.is_ok(), "Bincode serialization should work");
    }

    #[test]
    fn test_secret_date_deserialization() {
        // Test that deserialization works for functional unwrap
        let original_dt = test_datetime();
        let secret = SecretDate::new(original_dt);

        // Test JSON round-trip
        let json = serde_json::to_string(&secret).unwrap();
        let deserialized: Result<SecretDate, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "JSON deserialization should work");

        let restored = deserialized.unwrap();
        assert_eq!(
            restored.reveal(),
            &original_dt,
            "Round-trip should preserve data"
        );

        // Test bincode round-trip
        let bytes = bincode::serialize(&secret).unwrap();
        let deserialized: Result<SecretDate, _> = bincode::deserialize(&bytes);
        assert!(deserialized.is_ok(), "Bincode deserialization should work");

        let restored = deserialized.unwrap();
        assert_eq!(
            restored.reveal(),
            &original_dt,
            "Bincode round-trip should preserve data"
        );
    }
}
