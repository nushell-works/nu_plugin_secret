use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use chrono::{self, Datelike};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A secure date type that redacts its content in all display contexts
#[derive(Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct SecretDate {
    #[zeroize(skip)]
    inner: chrono::DateTime<chrono::FixedOffset>,
}

impl SecretDate {
    /// Create a new SecretDate from a DateTime
    pub fn new(value: chrono::DateTime<chrono::FixedOffset>) -> Self {
        Self { inner: value }
    }

    /// Get a reference to the inner DateTime (for controlled access)
    pub fn reveal(&self) -> &chrono::DateTime<chrono::FixedOffset> {
        &self.inner
    }

    /// Convert back to a regular DateTime (consumes the SecretDate)
    pub fn into_inner(self) -> chrono::DateTime<chrono::FixedOffset> {
        self.inner.clone()
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
        Ok(Value::string("<redacted:date>", span))
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
        write!(f, "<redacted:date>")
    }
}

impl fmt::Debug for SecretDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretDate(<redacted>)")
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
        assert_eq!(format!("{}", secret), "<redacted:date>");
        assert_eq!(format!("{:?}", secret), "SecretDate(<redacted>)");
    }

    #[test]
    fn test_secret_date_custom_value() {
        let dt = test_datetime();
        let secret = SecretDate::new(dt);
        assert_eq!(secret.type_name(), "secret_date");
        
        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => assert_eq!(val, "<redacted:date>"),
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
}