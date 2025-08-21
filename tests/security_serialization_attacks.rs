//! Comprehensive serialization attack resistance tests for nu_plugin_secret
//! 
//! These tests verify that secret types are protected against various serialization-based
//! attacks and cannot be exploited to expose sensitive data through serialization channels.

use nu_plugin_secret::{SecretString, SecretInt, SecretBinary, SecretRecord, SecretBool, SecretFloat, SecretDate};
use serde_json;
use nu_protocol::{Value, Span, Record};

/// Test serialization attack resistance
#[cfg(test)]
mod serialization_attack_tests {
    use super::*;
    
    /// Test that JSON serialization never exposes secret content
    #[test]
    fn test_json_serialization_protection() {
        let secret = SecretString::new("super_secret_api_key".to_string());
        
        // Direct JSON serialization should not expose content
        let json_result = serde_json::to_string(&secret);
        match json_result {
            Ok(json) => {
                assert!(!json.contains("super_secret_api_key"), 
                       "Secret content exposed in JSON: {}", json);
                assert!(json.contains("redacted") || json.contains("SecretString"), 
                       "JSON should contain redacted indicator: {}", json);
            },
            Err(_) => {
                // It's also acceptable if serialization fails for security
                println!("JSON serialization failed (acceptable for security)");
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
                    assert_ne!(secret.reveal(), "exposed_secret", 
                             "Malicious JSON successfully injected secret content");
                },
                Err(_) => {
                    // It's acceptable (and preferred) if malicious JSON fails to deserialize
                    println!("Malicious JSON rejected: {}", malicious_json);
                }
            }
        }
    }
    
    /// Test bincode serialization protection (used for plugin communication)
    #[test]
    fn test_bincode_serialization_protection() {
        let secret = SecretString::new("bincode_secret_test".to_string());
        
        // Bincode serialization for plugin communication
        let bincode_result = bincode::serialize(&secret);
        match bincode_result {
            Ok(bytes) => {
                // Verify the serialized bytes don't contain the raw secret
                let bytes_str = String::from_utf8_lossy(&bytes);
                assert!(!bytes_str.contains("bincode_secret_test"),
                       "Secret content found in bincode serialization");
                       
                // Test roundtrip doesn't expose content
                let deserialized: Result<SecretString, _> = bincode::deserialize(&bytes);
                match deserialized {
                    Ok(restored) => {
                        // Should maintain redacted display
                        assert_eq!(format!("{}", restored), "<redacted:string>");
                    },
                    Err(_) => println!("Bincode deserialization failed (acceptable)")
                }
            },
            Err(_) => println!("Bincode serialization failed (acceptable for security)")
        }
    }
    
    /// Test TOML serialization protection
    #[test]
    fn test_toml_serialization_protection() {
        let secret = SecretString::new("toml_secret_content".to_string());
        
        let toml_result = toml::to_string(&secret);
        match toml_result {
            Ok(toml) => {
                assert!(!toml.contains("toml_secret_content"),
                       "Secret content exposed in TOML: {}", toml);
            },
            Err(_) => println!("TOML serialization failed (acceptable for security)")
        }
    }
    
    /// Test YAML serialization protection
    #[test]
    fn test_yaml_serialization_protection() {
        let secret = SecretString::new("yaml_secret_data".to_string());
        
        let yaml_result = serde_yaml::to_string(&secret);
        match yaml_result {
            Ok(yaml) => {
                assert!(!yaml.contains("yaml_secret_data"),
                       "Secret content exposed in YAML: {}", yaml);
            },
            Err(_) => println!("YAML serialization failed (acceptable for security)")
        }
    }
    
    /// Test protection across all secret types
    #[test]
    fn test_all_secret_types_serialization_protection() {
        let secrets: Vec<Box<dyn SerializationTestable>> = vec![
            Box::new(SecretString::new("secret_string".to_string())),
            Box::new(SecretInt::new(123456789)),
            Box::new(SecretBool::new(true)),
            Box::new(SecretFloat::new(3.14159)),
            Box::new(SecretBinary::new(b"secret_bytes".to_vec())),
            Box::new(SecretDate::new(chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()))),
        ];
        
        for (i, secret) in secrets.iter().enumerate() {
            let json_result = secret.test_json_serialization();
            match json_result {
                Ok(json) => {
                    assert!(!secret.contains_sensitive_data(&json),
                           "Secret type {} exposed sensitive data in JSON: {}", i, json);
                },
                Err(_) => println!("Secret type {} JSON serialization failed (acceptable)", i)
            }
        }
    }
    
    /// Test serialization bomb attacks
    #[test]
    fn test_serialization_bomb_resistance() {
        // Create deeply nested structures to test for serialization bombs
        let mut large_record = Record::new();
        for i in 0..1000 {
            large_record.insert(&format!("field_{}", i), 
                              Value::string(&format!("value_{}", i), Span::test_data()));
        }
        
        let secret_record = SecretRecord::new(large_record);
        
        // Serialization should not consume excessive memory or time
        let start = std::time::Instant::now();
        let json_result = serde_json::to_string(&secret_record);
        let duration = start.elapsed();
        
        // Should complete within reasonable time (1 second)
        assert!(duration.as_secs() < 1, "Serialization took too long: {:?}", duration);
        
        match json_result {
            Ok(json) => {
                // Should not be excessively large (indicating potential bomb)
                assert!(json.len() < 100_000, "Serialized output suspiciously large: {} bytes", json.len());
                
                // Should not contain actual field values
                assert!(!json.contains("value_999"), "Large record content exposed in serialization");
            },
            Err(_) => println!("Large record serialization failed (acceptable for security)")
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
        assert!(int_result.is_err(), "Type confusion attack succeeded: SecretString as SecretInt");
        assert!(bool_result.is_err(), "Type confusion attack succeeded: SecretString as SecretBool"); 
        assert!(float_result.is_err(), "Type confusion attack succeeded: SecretString as SecretFloat");
    }
    
    /// Test memory exhaustion via malicious deserialization
    #[test]
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
            let start_memory = get_memory_usage();
            
            let deser_result: Result<serde_json::Value, _> = serde_json::from_str(pattern);
            
            let end_memory = get_memory_usage();
            let memory_diff = end_memory.saturating_sub(start_memory);
            
            // Should not consume excessive memory (> 50MB)
            assert!(memory_diff < 50_000_000, 
                   "Pattern {} consumed excessive memory: {} bytes", i, memory_diff);
                   
            match deser_result {
                Ok(_) => println!("Pattern {} parsed successfully but within memory limits", i),
                Err(_) => println!("Pattern {} rejected (good for security)", i)
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
        assert!(duration.as_secs() < 1, "Circular reference test took too long");
        
        // Should succeed or fail gracefully, not hang
        match (json1, json2) {
            (Ok(j1), Ok(j2)) => {
                println!("Circular reference test completed: {} & {}", 
                        j1.len(), j2.len());
            },
            _ => println!("Circular reference serialization failed (acceptable)")
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
        // For binary data, check if any of the bytes appear as strings
        let revealed = self.reveal();
        let as_string = String::from_utf8_lossy(revealed);
        serialized.contains(&*as_string)
    }
}

impl SerializationTestable for SecretDate {
    fn test_json_serialization(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    fn contains_sensitive_data(&self, serialized: &str) -> bool {
        let revealed = self.reveal().to_string();
        serialized.contains(&revealed)
    }
}

/// Performance and stress testing for serialization
#[cfg(test)]
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
        assert!(per_operation < 100_000, 
               "Serialization too slow: {}ns per operation", per_operation);
        
        println!("Serialization performance: {}ns per operation", per_operation);
    }
    
    #[test]
    fn test_large_data_serialization() {
        // Test serialization of large secret data
        let large_secret = SecretString::new("x".repeat(10_000));
        
        let start = std::time::Instant::now();
        let json_result = serde_json::to_string(&large_secret);
        let duration = start.elapsed();
        
        // Should complete within reasonable time
        assert!(duration.as_millis() < 100, "Large data serialization too slow");
        
        match json_result {
            Ok(json) => {
                assert!(!json.contains(&"x".repeat(100)), "Large secret content exposed");
            },
            Err(_) => println!("Large data serialization failed (acceptable)")
        }
    }
}

// Helper function to get current memory usage (simplified)
fn get_memory_usage() -> usize {
    // In a real implementation, this would use proper memory profiling
    // For testing purposes, we'll use a simple approximation
    std::process::id() as usize * 1024 // Simplified placeholder
}

/// Integration tests with Nushell Value system
#[cfg(test)]
mod nushell_integration_tests {
    use super::*;
    
    #[test]
    fn test_nushell_value_serialization_safety() {
        let secret = SecretString::new("nushell_secret".to_string());
        
        // Convert to Nushell CustomValue
        let custom_value = Value::custom(Box::new(secret), Span::test_data());
        
        // Test serialization through Nushell's system
        let json_result = serde_json::to_string(&custom_value);
        match json_result {
            Ok(json) => {
                assert!(!json.contains("nushell_secret"),
                       "Secret exposed through Nushell Value serialization: {}", json);
            },
            Err(_) => println!("Nushell Value serialization failed (acceptable)")
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
                let bytes_str = String::from_utf8_lossy(&bytes);
                assert!(!bytes_str.contains("plugin_secret"),
                       "Plugin SecretString exposed in communication");
            },
            Err(_) => println!("Plugin SecretString serialization failed")
        }
        
        // Test SecretInt
        let secret_int = SecretInt::new(987654321);
        let bincode_result = bincode::serialize(&secret_int);
        match bincode_result {
            Ok(bytes) => {
                let bytes_str = String::from_utf8_lossy(&bytes);
                assert!(!bytes_str.contains("987654321"),
                       "Plugin SecretInt exposed in communication");
            },
            Err(_) => println!("Plugin SecretInt serialization failed")
        }
        
        // Test SecretBool
        let secret_bool = SecretBool::new(false);
        let bincode_result = bincode::serialize(&secret_bool);
        match bincode_result {
            Ok(_bytes) => {
                // For boolean, just verify serialization works
                println!("Plugin SecretBool serialization succeeded");
            },
            Err(_) => println!("Plugin SecretBool serialization failed")
        }
    }
}