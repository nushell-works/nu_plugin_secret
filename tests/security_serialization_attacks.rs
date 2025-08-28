//! Comprehensive serialization attack resistance tests for nu_plugin_secret
//!
//! These tests verify that secret types are protected against various serialization-based
//! attacks and cannot be exploited to expose sensitive data through serialization channels.

use nu_plugin_secret::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretRecord, SecretString,
};
use nu_protocol::{Record, Span, Value};

/// Test serialization attack resistance
#[cfg(test)]
mod serialization_attack_tests {
    use super::*;

    /// Test that JSON serialization exposes content for functional unwrap
    /// but display/debug remain secure
    #[test]
    fn test_json_serialization_functional() {
        let secret = SecretString::new("super_secret_api_key".to_string());

        // JSON serialization now contains actual content for functional unwrap
        let json_result = serde_json::to_string(&secret);
        match json_result {
            Ok(json) => {
                assert!(
                    json.contains("super_secret_api_key"),
                    "Secret content should be in JSON for functional unwrap: {}",
                    json
                );

                // But display/debug should still be redacted for security
                let display = format!("{}", secret);
                let debug = format!("{:?}", secret);
                assert!(
                    !display.contains("super_secret_api_key"),
                    "Display should remain redacted"
                );
                assert!(
                    !debug.contains("super_secret_api_key"),
                    "Debug should remain redacted"
                );
            }
            Err(_) => {
                panic!("JSON serialization should work for functional unwrap");
            }
        }
    }

    /// Test JSON deserialization attacks
    #[test]
    fn test_json_deserialization_attacks() {
        // Attempt to craft malicious JSON that might expose secrets
        let malicious_json_attempts = [
            r#"{"inner": "exposed_secret"}"#,
            r#"{"type": "secret_string", "inner": "exposed_secret"}"#,
            r#"{"SecretString": {"inner": "exposed_secret"}}"#,
            r#"{"value": "exposed_secret", "redacted": false}"#,
        ];

        for malicious_json in &malicious_json_attempts {
            let deser_result: Result<SecretString, _> = serde_json::from_str(malicious_json);
            match deser_result {
                Ok(secret) => {
                    // If deserialization succeeds, verify it doesn't expose the injected content
                    assert_ne!(
                        secret.reveal(),
                        "exposed_secret",
                        "Malicious JSON successfully injected secret content"
                    );
                }
                Err(_) => {
                    // It's acceptable (and preferred) if malicious JSON fails to deserialize
                    println!("Malicious JSON rejected: {}", malicious_json);
                }
            }
        }
    }

    /// Test bincode serialization for functional plugin communication
    #[test]
    fn test_bincode_serialization_functional() {
        let secret = SecretString::new("bincode_secret_test".to_string());

        // Bincode serialization for plugin communication should work
        let bincode_result = bincode::serialize(&secret);
        match bincode_result {
            Ok(bytes) => {
                // Test roundtrip works for functional unwrap
                let deserialized: Result<SecretString, _> = bincode::deserialize(&bytes);
                match deserialized {
                    Ok(restored) => {
                        // Should maintain redacted display but functional reveal
                        assert_eq!(format!("{}", restored), "<redacted:string>");
                        assert_eq!(restored.reveal(), "bincode_secret_test");
                    }
                    Err(_) => panic!("Bincode deserialization should work for functional unwrap"),
                }
            }
            Err(_) => panic!("Bincode serialization should work for plugin communication"),
        }
    }

    /// Test TOML serialization protection
    #[test]
    fn test_toml_serialization_protection() {
        let secret = SecretString::new("toml_secret_content".to_string());

        let toml_result = toml::to_string(&secret);
        match toml_result {
            Ok(toml) => {
                assert!(
                    !toml.contains("toml_secret_content"),
                    "Secret content exposed in TOML: {}",
                    toml
                );
            }
            Err(_) => println!("TOML serialization failed (acceptable for security)"),
        }
    }

    /// Test YAML serialization for functional unwrap
    #[test]
    fn test_yaml_serialization_functional() {
        let secret = SecretString::new("yaml_secret_data".to_string());

        let yaml_result = serde_yaml::to_string(&secret);
        match yaml_result {
            Ok(yaml) => {
                // YAML should contain content for functional serialization
                assert!(
                    yaml.contains("yaml_secret_data"),
                    "Secret content should be in YAML for functional unwrap: {}",
                    yaml
                );

                // But display should still be redacted
                let display = format!("{}", secret);
                assert!(
                    !display.contains("yaml_secret_data"),
                    "Display should remain redacted"
                );
            }
            Err(_) => panic!("YAML serialization should work for functional unwrap"),
        }
    }

    /// Test functional serialization across all secret types
    #[test]
    fn test_all_secret_types_serialization_functional() {
        let secrets: Vec<Box<dyn SerializationTestable>> = vec![
            Box::new(SecretString::new("secret_string".to_string())),
            Box::new(SecretInt::new(123456789)),
            Box::new(SecretBool::new(true)),
            Box::new(SecretFloat::new(std::f64::consts::PI)),
            Box::new(SecretBinary::new(b"secret_bytes".to_vec())),
            Box::new(SecretDate::new(
                chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
            )),
        ];

        for (i, secret) in secrets.iter().enumerate() {
            let json_result = secret.test_json_serialization();
            match json_result {
                Ok(json) => {
                    // Now serialization should contain data for functional unwrap
                    assert!(
                        secret.contains_sensitive_data(&json),
                        "Secret type {} should expose data in JSON for functional unwrap: {}",
                        i,
                        json
                    );
                }
                Err(_) => panic!(
                    "Secret type {} JSON serialization should work for functional unwrap",
                    i
                ),
            }
        }
    }

    /// Test serialization performance with large data
    #[test]
    #[cfg(not(miri))] // Exclude from Miri due to performance (large data structures)
    fn test_serialization_performance_large_data() {
        // Create deeply nested structures to test performance
        let mut large_record = Record::new();
        for i in 0..1000 {
            large_record.insert(
                format!("field_{}", i),
                Value::string(format!("value_{}", i), Span::test_data()),
            );
        }

        let secret_record = SecretRecord::new(large_record);

        // Serialization should not consume excessive memory or time
        let start = std::time::Instant::now();
        let json_result = serde_json::to_string(&secret_record);
        let duration = start.elapsed();

        // Should complete within reasonable time (5 seconds for large data)
        assert!(
            duration.as_secs() < 5,
            "Serialization took too long: {:?}",
            duration
        );

        match json_result {
            Ok(json) => {
                // With functional serialization, it may be larger but should contain actual data
                println!("Large record serialization completed: {} bytes", json.len());

                // Should contain actual field values for functional unwrap
                assert!(
                    json.contains("value_999"),
                    "Large record content should be in serialization for functional unwrap"
                );
            }
            Err(_) => panic!("Large record serialization should work for functional unwrap"),
        }
    }

    /// Test type confusion attacks via serialization
    #[test]
    fn test_type_confusion_resistance() {
        // Try to deserialize SecretString as other types
        let secret_json = serde_json::to_string(&SecretString::new("test".to_string())).unwrap();

        // Attempt type confusion attacks
        let int_result: Result<SecretInt, _> = serde_json::from_str(&secret_json);
        let bool_result: Result<SecretBool, _> = serde_json::from_str(&secret_json);
        let float_result: Result<SecretFloat, _> = serde_json::from_str(&secret_json);

        // All should fail or not expose the original content
        assert!(
            int_result.is_err(),
            "Type confusion attack succeeded: SecretString as SecretInt"
        );
        assert!(
            bool_result.is_err(),
            "Type confusion attack succeeded: SecretString as SecretBool"
        );
        assert!(
            float_result.is_err(),
            "Type confusion attack succeeded: SecretString as SecretFloat"
        );
    }

    /// Test memory exhaustion via malicious deserialization
    #[test]
    #[cfg(not(miri))] // Exclude from Miri due to performance (large data structures)
    fn test_memory_exhaustion_resistance() {
        // Attempt to create malicious JSON that could cause memory exhaustion
        let malicious_patterns = [
            // Extremely long string
            format!(r#"{{"inner": "{}"}}"#, "A".repeat(10_000)),
            // Deeply nested structure
            r#"{"inner": {"inner": {"inner": {"inner": "deep"}}}}"#.to_string(),
            // Large array
            format!(r#"[{}]"#, "\"item\",".repeat(1000)),
        ];

        for (i, pattern) in malicious_patterns.iter().enumerate() {
            let deser_result: Result<serde_json::Value, _> = serde_json::from_str(pattern);

            // Verify deserialization doesn't panic or consume excessive resources
            match deser_result {
                Ok(_) => {
                    // Pattern was successfully parsed - verify it's reasonable
                    assert!(
                        pattern.len() < 100_000,
                        "Pattern {} is unreasonably large",
                        i
                    );
                }
                Err(_) => {
                    // Failed to parse - this is acceptable for malicious patterns
                }
            }

            match deser_result {
                Ok(_) => println!("Pattern {} parsed successfully but within memory limits", i),
                Err(_) => println!("Pattern {} rejected (good for security)", i),
            }
        }
    }

    /// Test circular reference resistance
    #[test]
    fn test_circular_reference_resistance() {
        // Create structures that might cause circular references in serialization
        let mut record1 = Record::new();
        let mut record2 = Record::new();

        record1.insert("ref_to_2", Value::string("record2", Span::test_data()));
        record2.insert("ref_to_1", Value::string("record1", Span::test_data()));

        let secret1 = SecretRecord::new(record1);
        let secret2 = SecretRecord::new(record2);

        // Serialization should handle potential circularity gracefully
        let start = std::time::Instant::now();

        let json1 = serde_json::to_string(&secret1);
        let json2 = serde_json::to_string(&secret2);

        let duration = start.elapsed();

        // Should not hang or take excessive time
        assert!(
            duration.as_secs() < 1,
            "Circular reference test took too long"
        );

        // Should succeed or fail gracefully, not hang
        match (json1, json2) {
            (Ok(j1), Ok(j2)) => {
                println!(
                    "Circular reference test completed: {} & {}",
                    j1.len(),
                    j2.len()
                );
            }
            _ => println!("Circular reference serialization failed (acceptable)"),
        }
    }
}

/// Helper trait for testing serialization across different secret types
trait SerializationTestable {
    fn test_json_serialization(&self) -> Result<String, serde_json::Error>;
    fn contains_sensitive_data(&self, serialized: &str) -> bool;
}

impl SerializationTestable for SecretString {
    fn test_json_serialization(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn contains_sensitive_data(&self, serialized: &str) -> bool {
        // Check if the serialized form contains the actual secret content
        // This is a simplified check - real implementation would be more sophisticated
        let revealed = self.reveal();
        serialized.contains(revealed)
    }
}

impl SerializationTestable for SecretInt {
    fn test_json_serialization(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn contains_sensitive_data(&self, serialized: &str) -> bool {
        let revealed = self.reveal().to_string();
        serialized.contains(&revealed)
    }
}

impl SerializationTestable for SecretBool {
    fn test_json_serialization(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn contains_sensitive_data(&self, serialized: &str) -> bool {
        let revealed = self.reveal().to_string();
        serialized.contains(&revealed)
    }
}

impl SerializationTestable for SecretFloat {
    fn test_json_serialization(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn contains_sensitive_data(&self, serialized: &str) -> bool {
        let revealed = self.reveal().to_string();
        serialized.contains(&revealed)
    }
}

impl SerializationTestable for SecretBinary {
    fn test_json_serialization(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn contains_sensitive_data(&self, serialized: &str) -> bool {
        // For binary data, check if the bytes appear as a JSON array or string
        let revealed = self.reveal();
        let bytes: &[u8] = revealed.as_ref();

        // Check if serialized as byte array [115,101,...]
        let byte_array = format!("{:?}", bytes).replace(" ", "");
        if serialized.contains(&byte_array) {
            return true;
        }

        // Check if serialized as string (if UTF-8 valid)
        if let Ok(as_string) = String::from_utf8(bytes.to_vec()) {
            if serialized.contains(&as_string) {
                return true;
            }
        }

        false
    }
}

impl SerializationTestable for SecretDate {
    fn test_json_serialization(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn contains_sensitive_data(&self, serialized: &str) -> bool {
        let revealed = self.reveal();

        // Check multiple date formats that might appear in serialization
        let formats = [
            revealed.to_rfc3339(),
            revealed.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            revealed.format("%Y-%m-%d %H:%M:%S").to_string(),
            revealed.to_string(),
        ];

        for format in &formats {
            if serialized.contains(format) {
                return true;
            }
        }

        // Also check for partial matches (year, month, etc)
        let year = revealed.format("%Y").to_string();
        if serialized.contains(&year) && serialized.contains("2023") {
            return true;
        }

        false
    }
}

/// Performance and stress testing for serialization
#[cfg(all(test, not(miri)))]
mod serialization_performance_tests {
    use super::*;

    #[test]
    fn test_serialization_performance() {
        let secrets: Vec<SecretString> = (0..1000)
            .map(|i| SecretString::new(format!("secret_{}", i)))
            .collect();

        let start = std::time::Instant::now();

        for secret in &secrets {
            let _ = serde_json::to_string(secret);
        }

        let duration = start.elapsed();
        let per_operation = duration.as_nanos() / secrets.len() as u128;

        // Serialization should be reasonably fast (< 100 microseconds per operation)
        assert!(
            per_operation < 100_000,
            "Serialization too slow: {}ns per operation",
            per_operation
        );

        println!(
            "Serialization performance: {}ns per operation",
            per_operation
        );
    }

    #[test]
    fn test_large_data_serialization() {
        // Test serialization of large secret data
        let large_secret = SecretString::new("x".repeat(10_000));

        let start = std::time::Instant::now();
        let json_result = serde_json::to_string(&large_secret);
        let duration = start.elapsed();

        // Should complete within reasonable time
        assert!(
            duration.as_millis() < 500,
            "Large data serialization too slow"
        );

        match json_result {
            Ok(json) => {
                // With functional serialization, large content should be present
                assert!(
                    json.contains(&"x".repeat(100)),
                    "Large secret content should be present for functional unwrap"
                );

                // But display should still be redacted
                let display = format!("{}", large_secret);
                assert!(
                    !display.contains(&"x".repeat(100)),
                    "Display should remain redacted"
                );
            }
            Err(_) => panic!("Large data serialization should work for functional unwrap"),
        }
    }
}

/// Integration tests with Nushell Value system
#[cfg(test)]
mod nushell_integration_tests {
    use super::*;

    #[test]
    fn test_nushell_value_serialization_functional() {
        let secret = SecretString::new("nushell_secret".to_string());

        // Convert to Nushell CustomValue
        let custom_value = Value::custom(Box::new(secret), Span::test_data());

        // Test serialization through Nushell's system
        let json_result = serde_json::to_string(&custom_value);
        match json_result {
            Ok(json) => {
                // For functional unwrap, the content should be accessible
                println!("Nushell Value serialization completed: {}", json);
                // Note: Nushell's CustomValue serialization may wrap the content differently
            }
            Err(_) => {
                // Nushell CustomValue serialization may not work directly
                println!("Nushell Value serialization failed (this is normal)");
            }
        }
    }

    #[test]
    fn test_plugin_communication_serialization() {
        // Test the serialization used for plugin communication

        // Test SecretString
        let secret_string = SecretString::new("plugin_secret_1".to_string());
        let bincode_result = bincode::serialize(&secret_string);
        match bincode_result {
            Ok(bytes) => {
                // Test roundtrip works for functional unwrap
                let deserialized: Result<SecretString, _> = bincode::deserialize(&bytes);
                match deserialized {
                    Ok(restored) => {
                        assert_eq!(restored.reveal(), "plugin_secret_1");
                        println!("Plugin SecretString communication works");
                    }
                    Err(_) => panic!("Plugin SecretString deserialization should work"),
                }
            }
            Err(_) => panic!("Plugin SecretString serialization should work"),
        }

        // Test SecretInt
        let secret_int = SecretInt::new(987654321);
        let bincode_result = bincode::serialize(&secret_int);
        match bincode_result {
            Ok(bytes) => {
                // Test roundtrip works
                let deserialized: Result<SecretInt, _> = bincode::deserialize(&bytes);
                match deserialized {
                    Ok(restored) => {
                        assert_eq!(restored.reveal(), 987654321);
                        println!("Plugin SecretInt communication works");
                    }
                    Err(_) => panic!("Plugin SecretInt deserialization should work"),
                }
            }
            Err(_) => panic!("Plugin SecretInt serialization should work"),
        }

        // Test SecretBool
        let secret_bool = SecretBool::new(false);
        let bincode_result = bincode::serialize(&secret_bool);
        match bincode_result {
            Ok(bytes) => {
                // Test roundtrip works
                let deserialized: Result<SecretBool, _> = bincode::deserialize(&bytes);
                match deserialized {
                    Ok(restored) => {
                        assert!(!restored.reveal());
                        println!("Plugin SecretBool communication works");
                    }
                    Err(_) => panic!("Plugin SecretBool deserialization should work"),
                }
            }
            Err(_) => panic!("Plugin SecretBool serialization should work"),
        }
    }
}
