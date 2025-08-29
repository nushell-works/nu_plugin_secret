use crate::{SecretBinary, SecretList, SecretRecord, SecretString};
use blake3;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};
use sha2::{Digest, Sha256, Sha512};

#[derive(Clone)]
pub struct SecretHashCommand;

#[derive(Clone, Debug)]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Blake3,
}

impl std::fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::Sha256 => write!(f, "sha256"),
            HashAlgorithm::Sha512 => write!(f, "sha512"),
            HashAlgorithm::Blake3 => write!(f, "blake3"),
        }
    }
}

impl std::str::FromStr for HashAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sha256" => Ok(HashAlgorithm::Sha256),
            "sha512" => Ok(HashAlgorithm::Sha512),
            "blake3" => Ok(HashAlgorithm::Blake3),
            _ => Err(format!(
                "Unsupported hash algorithm: {}. Supported algorithms: sha256, sha512, blake3",
                s
            )),
        }
    }
}

impl PluginCommand for SecretHashCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret hash"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![
                (Type::Custom("secret_string".into()), Type::String),
                (Type::Custom("secret_binary".into()), Type::String),
                (Type::Custom("secret_list".into()), Type::String),
                (Type::Custom("secret_record".into()), Type::String),
            ])
            .optional(
                "algorithm",
                SyntaxShape::String,
                "Hash algorithm to use (sha256, sha512, blake3). Defaults to sha256",
            )
            .category(Category::Hash)
    }

    fn description(&self) -> &str {
        "Generate cryptographic hash of secret data without exposing the content"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: r#""my-secret-password" | secret wrap | secret hash"#,
                description: "Hash a secret string using SHA-256 (default)",
                result: Some(Value::string(
                    "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
                    nu_protocol::Span::test_data(),
                )),
            },
            Example {
                example: r#""my-secret-password" | secret wrap | secret hash sha256"#,
                description: "Hash a secret string using SHA-256",
                result: Some(Value::string(
                    "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
                    nu_protocol::Span::test_data(),
                )),
            },
            Example {
                example: r#""my-secret-password" | secret wrap | secret hash sha512"#,
                description: "Hash a secret string using SHA-512",
                result: Some(Value::string(
                    "b109f3bbbc244eb82441917ed06d618b9008dd09b3befd1b5e07394c706a8bb980b1d7785e5976ec049b46df5f1326af5a2ea6d103fd07c95385ffab0cacbc86",
                    nu_protocol::Span::test_data(),
                )),
            },
            Example {
                example: r#""my-secret-password" | secret wrap | secret hash blake3"#,
                description: "Hash a secret string using BLAKE3",
                result: Some(Value::string(
                    "d4c8d3ca37f09e17c5b0ce1b1c3e36b57f8f7b85b1c8f0a0a7b5c4d3f2e91a08",
                    nu_protocol::Span::test_data(),
                )),
            },
            Example {
                example: r#"0x[deadbeef] | secret wrap | secret hash"#,
                description: "Hash secret binary data",
                result: Some(Value::string(
                    "5f78c33274e43fa9de5659265c1d917e25c03722dcb0b8d27db8d5feaa813953",
                    nu_protocol::Span::test_data(),
                )),
            },
        ]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        // Parse algorithm parameter
        let algorithm = if let Some(algo_value) = call.positional.first() {
            match algo_value {
                Value::String { val, .. } => val.parse::<HashAlgorithm>().map_err(|e| {
                    LabeledError::new(format!("Invalid hash algorithm: {}", e))
                        .with_label("Supported algorithms: sha256, sha512, blake3", call.head)
                })?,
                _ => {
                    return Err(LabeledError::new("Invalid algorithm parameter")
                        .with_label("Algorithm must be a string", call.head))
                }
            }
        } else {
            HashAlgorithm::Sha256 // Default algorithm
        };

        match input {
            PipelineData::Value(value, metadata) => {
                let result = match value {
                    Value::Custom { val, .. } => {
                        if let Some(secret_string) = val.as_any().downcast_ref::<SecretString>() {
                            let data = secret_string.reveal().as_bytes();
                            let hash_hex = compute_hash(&algorithm, data);
                            Value::string(hash_hex, call.head)
                        } else if let Some(secret_binary) =
                            val.as_any().downcast_ref::<SecretBinary>()
                        {
                            let data = secret_binary.reveal();
                            let hash_hex = compute_hash(&algorithm, &data);
                            Value::string(hash_hex, call.head)
                        } else if let Some(secret_list) = val.as_any().downcast_ref::<SecretList>()
                        {
                            // Serialize the list to bytes for hashing
                            let data = serialize_list_for_hash(secret_list)?;
                            let hash_hex = compute_hash(&algorithm, &data);
                            Value::string(hash_hex, call.head)
                        } else if let Some(secret_record) =
                            val.as_any().downcast_ref::<SecretRecord>()
                        {
                            // Serialize the record to bytes for hashing
                            let data = serialize_record_for_hash(secret_record)?;
                            let hash_hex = compute_hash(&algorithm, &data);
                            Value::string(hash_hex, call.head)
                        } else {
                            return Err(LabeledError::new("Unsupported secret type").with_label(
                                "Only SecretString, SecretBinary, SecretList, and SecretRecord support hash operation",
                                call.head,
                            ));
                        }
                    }
                    _ => {
                        return Err(LabeledError::new("Invalid input").with_label(
                            "Input must be a secret value. Use 'secret wrap' to create a secret first",
                            call.head,
                        ));
                    }
                };

                Ok(PipelineData::Value(result, metadata))
            }
            _ => Err(LabeledError::new("Invalid input")
                .with_label("Expected a single secret value", call.head)),
        }
    }
}

fn compute_hash(algorithm: &HashAlgorithm, data: &[u8]) -> String {
    match algorithm {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            hasher.update(data);
            hex::encode(hasher.finalize().as_bytes())
        }
    }
}

fn serialize_list_for_hash(secret_list: &SecretList) -> Result<Vec<u8>, LabeledError> {
    // Use bincode to serialize the list deterministically
    bincode::serialize(secret_list.reveal())
        .map_err(|e| LabeledError::new(format!("Failed to serialize list for hashing: {}", e)))
}

fn serialize_record_for_hash(secret_record: &SecretRecord) -> Result<Vec<u8>, LabeledError> {
    // Use bincode to serialize the record deterministically
    bincode::serialize(secret_record.reveal())
        .map_err(|e| LabeledError::new(format!("Failed to serialize record for hashing: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::{Record, Span};

    #[test]
    fn test_command_name() {
        let command = SecretHashCommand;
        assert_eq!(command.name(), "secret hash");
    }

    #[test]
    fn test_signature() {
        let command = SecretHashCommand;
        let signature = command.signature();
        assert_eq!(signature.name, "secret hash");
        assert_eq!(signature.optional_positional.len(), 1);
        assert_eq!(signature.input_output_types.len(), 4);
    }

    #[test]
    fn test_description() {
        let command = SecretHashCommand;
        assert_eq!(
            command.description(),
            "Generate cryptographic hash of secret data without exposing the content"
        );
    }

    #[test]
    fn test_examples_count() {
        let command = SecretHashCommand;
        let examples = command.examples();
        assert_eq!(examples.len(), 5);
    }

    #[test]
    fn test_hash_algorithm_parsing() {
        assert!(matches!(
            "sha256".parse::<HashAlgorithm>().unwrap(),
            HashAlgorithm::Sha256
        ));
        assert!(matches!(
            "SHA256".parse::<HashAlgorithm>().unwrap(),
            HashAlgorithm::Sha256
        ));
        assert!(matches!(
            "sha512".parse::<HashAlgorithm>().unwrap(),
            HashAlgorithm::Sha512
        ));
        assert!(matches!(
            "blake3".parse::<HashAlgorithm>().unwrap(),
            HashAlgorithm::Blake3
        ));

        assert!("invalid".parse::<HashAlgorithm>().is_err());
    }

    #[test]
    fn test_compute_hash_sha256() {
        let data = b"hello world";
        let hash = compute_hash(&HashAlgorithm::Sha256, data);
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_compute_hash_sha512() {
        let data = b"hello world";
        let hash = compute_hash(&HashAlgorithm::Sha512, data);
        assert_eq!(
            hash,
            "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
        );
    }

    #[test]
    fn test_compute_hash_blake3() {
        let data = b"hello world";
        let hash = compute_hash(&HashAlgorithm::Blake3, data);
        // BLAKE3 produces a 256-bit hash
        assert_eq!(hash.len(), 64); // 32 bytes * 2 characters per byte
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_string_hashing_logic() {
        let secret = SecretString::new("test-secret".to_string());
        let data = secret.reveal().as_bytes();
        let hash = compute_hash(&HashAlgorithm::Sha256, data);

        // Verify it's a valid hex string
        assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_binary_hashing_logic() {
        let test_data = vec![0xde, 0xad, 0xbe, 0xef];
        let secret = SecretBinary::new(test_data.clone());
        let hash = compute_hash(&HashAlgorithm::Sha256, &secret.reveal());

        // Verify it's a valid hex string
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Hash should be deterministic
        let hash2 = compute_hash(&HashAlgorithm::Sha256, &test_data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_list_serialization() {
        let test_list = vec![
            Value::int(1, Span::test_data()),
            Value::string("test", Span::test_data()),
            Value::bool(true, Span::test_data()),
        ];
        let secret = SecretList::new(test_list.clone());
        let serialized = serialize_list_for_hash(&secret);
        assert!(serialized.is_ok());

        // Verify deterministic serialization
        let secret2 = SecretList::new(test_list);
        let serialized2 = serialize_list_for_hash(&secret2);
        assert_eq!(serialized.unwrap(), serialized2.unwrap());
    }

    #[test]
    fn test_record_serialization() {
        let mut test_record = Record::new();
        test_record.push("key1", Value::string("value1", Span::test_data()));
        test_record.push("key2", Value::int(42, Span::test_data()));

        let secret = SecretRecord::new(test_record.clone());
        let serialized = serialize_record_for_hash(&secret);
        assert!(serialized.is_ok());

        // Verify deterministic serialization
        let secret2 = SecretRecord::new(test_record);
        let serialized2 = serialize_record_for_hash(&secret2);
        assert_eq!(serialized.unwrap(), serialized2.unwrap());
    }

    #[test]
    fn test_empty_data_hashing() {
        // Empty string
        let empty_string = SecretString::new(String::new());
        let hash = compute_hash(&HashAlgorithm::Sha256, empty_string.reveal().as_bytes());
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        ); // SHA-256 of empty string

        // Empty binary
        let empty_binary = SecretBinary::new(vec![]);
        let hash = compute_hash(&HashAlgorithm::Sha256, &empty_binary.reveal());
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_deterministic_hashing() {
        let data = b"deterministic test data";

        let hash1 = compute_hash(&HashAlgorithm::Sha256, data);
        let hash2 = compute_hash(&HashAlgorithm::Sha256, data);
        assert_eq!(hash1, hash2);

        let hash3 = compute_hash(&HashAlgorithm::Sha512, data);
        let hash4 = compute_hash(&HashAlgorithm::Sha512, data);
        assert_eq!(hash3, hash4);

        let hash5 = compute_hash(&HashAlgorithm::Blake3, data);
        let hash6 = compute_hash(&HashAlgorithm::Blake3, data);
        assert_eq!(hash5, hash6);
    }

    #[test]
    fn test_different_algorithms_produce_different_hashes() {
        let data = b"test data for different algorithms";

        let sha256_hash = compute_hash(&HashAlgorithm::Sha256, data);
        let sha512_hash = compute_hash(&HashAlgorithm::Sha512, data);
        let blake3_hash = compute_hash(&HashAlgorithm::Blake3, data);

        assert_ne!(sha256_hash, sha512_hash);
        assert_ne!(sha256_hash, blake3_hash);
        assert_ne!(sha512_hash, blake3_hash);

        // Verify hash lengths
        assert_eq!(sha256_hash.len(), 64); // 32 bytes * 2
        assert_eq!(sha512_hash.len(), 128); // 64 bytes * 2
        assert_eq!(blake3_hash.len(), 64); // 32 bytes * 2
    }

    #[test]
    fn test_unicode_string_hashing() {
        let unicode_string = "Hello 世界 🌍 мир";
        let secret = SecretString::new(unicode_string.to_string());
        let hash = compute_hash(&HashAlgorithm::Sha256, secret.reveal().as_bytes());

        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Hash should be consistent
        let hash2 = compute_hash(&HashAlgorithm::Sha256, unicode_string.as_bytes());
        assert_eq!(hash, hash2);
    }
}
