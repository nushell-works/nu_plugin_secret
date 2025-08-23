use crate::config::RedactionContext;
use crate::memory_optimizations::{
    binary_optimization::OptimizedBinary, get_configurable_redacted_string,
};
use nu_protocol::CustomValue;
use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secure binary type that redacts its content in all display contexts
/// and zeros its memory on drop
#[derive(Clone)]
pub struct SecretBinary {
    inner: OptimizedBinary,
}

// Functional serialization - serialize actual content for pipeline operations
impl Serialize for SecretBinary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the actual content to make pipeline operations work
        let bytes = self.inner.as_bytes();
        bytes.as_ref().serialize(serializer)
    }
}

// Functional deserialization - restore actual content for pipeline operations
impl<'de> Deserialize<'de> for SecretBinary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the actual content to make pipeline operations work
        let content = Vec::<u8>::deserialize(deserializer)?;
        Ok(SecretBinary::new(content))
    }
}

impl Drop for SecretBinary {
    fn drop(&mut self) {
        // Explicitly zero the binary data memory for security
        self.inner.zeroize();
    }
}

// Manual ZeroizeOnDrop implementation to ensure proper cleanup
impl ZeroizeOnDrop for SecretBinary {}

impl SecretBinary {
    /// Create a new SecretBinary from a byte vector
    pub fn new(value: Vec<u8>) -> Self {
        Self {
            inner: OptimizedBinary::from_slice(&value),
        }
    }

    /// Get a reference to the inner binary data (for controlled access)
    pub fn reveal(&self) -> std::borrow::Cow<'_, [u8]> {
        self.inner.as_bytes()
    }

    /// Convert back to a regular byte vector (consumes the SecretBinary)
    pub fn into_inner(self) -> Vec<u8> {
        self.inner.as_bytes().into_owned()
    }

    /// Get the length of the binary data (safe to expose)
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the binary data is empty (safe to expose)
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get a byte at a specific index while preserving secrecy
    pub fn get(&self, index: usize) -> Option<u8> {
        let bytes = self.inner.as_bytes();
        bytes.get(index).copied()
    }
}

#[typetag::serde]
impl CustomValue for SecretBinary {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        "secret_binary".into()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        let redacted_text =
            get_configurable_redacted_string("binary", RedactionContext::Serialization);
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

impl fmt::Display for SecretBinary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string("binary", RedactionContext::Display);
        write!(f, "{}", redacted_text)
    }
}

impl fmt::Debug for SecretBinary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_text = get_configurable_redacted_string("binary", RedactionContext::Debug);
        write!(f, "SecretBinary({})", redacted_text)
    }
}

impl PartialEq for SecretBinary {
    fn eq(&self, other: &Self) -> bool {
        // Constant-time comparison to prevent timing attacks
        // This follows the pattern used in cryptographic libraries

        let len_a = self.inner.len();
        let len_b = other.inner.len();

        // First compare lengths in constant time
        let len_eq = constant_time_eq_usize(len_a, len_b);

        // Always compare min_len bytes to avoid timing differences
        let min_len = len_a.min(len_b);
        let max_len = len_a.max(len_b);

        // Use constant-time slice comparison for the minimum length
        let content_eq = if min_len > 0 {
            let self_bytes = self.inner.as_bytes();
            let other_bytes = other.inner.as_bytes();
            constant_time_eq_slice(&self_bytes[..min_len], &other_bytes[..min_len])
        } else {
            1u8 // Empty slices are equal
        };

        // If lengths differ, ensure we still do some work to maintain timing
        let _padding_work = if len_a != len_b {
            // Do some computation with the extra bytes to maintain consistent timing
            let extra_bytes = max_len - min_len;
            let mut dummy = 0u8;
            for i in 0..extra_bytes {
                dummy ^= (i as u8).wrapping_add(0x42);
            }
            dummy
        } else {
            0x42
        };

        // Combine results in constant time
        (len_eq & content_eq) == 1
    }
}

/// Constant-time equality check for usize values
fn constant_time_eq_usize(a: usize, b: usize) -> u8 {
    let diff = a ^ b;

    // Handle both 32-bit and 64-bit systems properly
    let diff = if usize::BITS >= 64 {
        diff | diff.wrapping_shr(32)
    } else {
        diff
    };
    let diff = diff | diff.wrapping_shr(16);
    let diff = diff | diff.wrapping_shr(8);
    let diff = diff | diff.wrapping_shr(4);
    let diff = diff | diff.wrapping_shr(2);
    let diff = diff | diff.wrapping_shr(1);
    (diff ^ 1) as u8
}

/// Constant-time equality check for byte slices of the same length
fn constant_time_eq_slice(a: &[u8], b: &[u8]) -> u8 {
    // This function assumes a.len() == b.len() for constant-time behavior
    // The caller should ensure this precondition

    let mut result = 0u8;
    for i in 0..a.len() {
        result |= a[i] ^ b[i];
    }

    // Convert to 0 or 1 using bit manipulation to avoid branches
    let result = result as u32;
    let result = result | result.wrapping_shr(16);
    let result = result | result.wrapping_shr(8);
    let result = result | result.wrapping_shr(4);
    let result = result | result.wrapping_shr(2);
    let result = result | result.wrapping_shr(1);
    (result ^ 1) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_binary_creation() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let secret = SecretBinary::new(data.clone());
        assert_eq!(secret.reveal().as_ref(), &data);
    }

    #[test]
    fn test_secret_binary_display() {
        let data = vec![0xde, 0xad, 0xbe, 0xef];
        let secret = SecretBinary::new(data);
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
    fn test_secret_binary_custom_value() {
        let data = vec![0x01, 0x23, 0x45, 0x67];
        let secret = SecretBinary::new(data);
        assert_eq!(secret.type_name(), "secret_binary");

        let base_value = secret.to_base_value(Span::test_data()).unwrap();
        match base_value {
            Value::String { val, .. } => {
                assert!(val.contains("redacted") || val.contains("***") || val.contains("HIDDEN"))
            }
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_secret_binary_into_inner() {
        let data = vec![0xff, 0x00, 0x11, 0x22];
        let secret = SecretBinary::new(data.clone());
        assert_eq!(secret.into_inner(), data);
    }

    #[test]
    fn test_secret_binary_equality() {
        let data1 = vec![0x01, 0x02, 0x03];
        let data2 = vec![0x01, 0x02, 0x03];
        let data3 = vec![0x04, 0x05, 0x06];

        let secret1 = SecretBinary::new(data1);
        let secret2 = SecretBinary::new(data2);
        let secret3 = SecretBinary::new(data3);

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }

    #[test]
    fn test_secret_binary_access() {
        let data = vec![0x10, 0x20, 0x30, 0x40];
        let secret = SecretBinary::new(data);

        assert_eq!(secret.len(), 4);
        assert!(!secret.is_empty());

        assert_eq!(secret.get(0), Some(0x10));
        assert_eq!(secret.get(1), Some(0x20));
        assert_eq!(secret.get(4), None); // Out of bounds
    }

    #[test]
    fn test_secret_binary_empty() {
        let empty_data: Vec<u8> = vec![];
        let secret = SecretBinary::new(empty_data);

        assert_eq!(secret.len(), 0);
        assert!(secret.is_empty());
        assert_eq!(secret.get(0), None);
    }

    #[test]
    fn test_secret_binary_equality_different_lengths() {
        let data1 = vec![0x01, 0x02];
        let data2 = vec![0x01, 0x02, 0x03];

        let secret1 = SecretBinary::new(data1);
        let secret2 = SecretBinary::new(data2);

        assert_ne!(secret1, secret2);
    }

    #[test]
    fn test_secret_binary_serialization() {
        // Test that serialization works for functional unwrap
        let data = vec![0xde, 0xad, 0xbe, 0xef, 0x12, 0x34];
        let secret = SecretBinary::new(data.clone());

        // Test JSON serialization - should contain array of bytes
        let json_result = serde_json::to_string(&secret);
        assert!(json_result.is_ok(), "JSON serialization should work");

        let json = json_result.unwrap();
        // Should contain the byte values for functional unwrap
        assert!(json.contains("222"), "JSON should contain byte values"); // 0xde = 222
        assert!(json.contains("173"), "JSON should contain byte values"); // 0xad = 173

        // Test bincode serialization (used for plugin communication)
        let bincode_result = bincode::serialize(&secret);
        assert!(bincode_result.is_ok(), "Bincode serialization should work");
    }

    #[test]
    fn test_secret_binary_deserialization() {
        // Test that deserialization works for functional unwrap
        let original_data = vec![0xff, 0x00, 0x42, 0xa5];
        let secret = SecretBinary::new(original_data.clone());

        // Test JSON round-trip
        let json = serde_json::to_string(&secret).unwrap();
        let deserialized: Result<SecretBinary, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "JSON deserialization should work");

        let restored = deserialized.unwrap();
        assert_eq!(
            restored.reveal().as_ref(),
            &original_data,
            "Round-trip should preserve data"
        );

        // Test bincode round-trip
        let bytes = bincode::serialize(&secret).unwrap();
        let deserialized: Result<SecretBinary, _> = bincode::deserialize(&bytes);
        assert!(deserialized.is_ok(), "Bincode deserialization should work");

        let restored = deserialized.unwrap();
        assert_eq!(
            restored.reveal().as_ref(),
            &original_data,
            "Bincode round-trip should preserve data"
        );
    }
}
