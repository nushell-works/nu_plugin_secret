//! Implements `secret validate-format` — validates secret content against common patterns.

use std::sync::OnceLock;

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};
use regex::Regex;

use crate::SecretString;

/// Supported format names for display in error messages.
const SUPPORTED_FORMATS: &str = "email, uuid, hex, base64, jwt, regex";

/// Returns a compiled regex, caching it in the provided `OnceLock`.
fn cached_regex<'a>(lock: &'a OnceLock<Regex>, pattern: &str) -> &'a Regex {
    lock.get_or_init(|| Regex::new(pattern).expect("built-in regex pattern must be valid"))
}

/// Returns the compiled email regex.
fn email_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    cached_regex(&RE, r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
}

/// Returns the compiled UUID regex (versions 1-5).
fn uuid_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    cached_regex(
        &RE,
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$",
    )
}

/// Returns the compiled hex string regex.
fn hex_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    cached_regex(&RE, r"^[0-9a-fA-F]+$")
}

/// Returns the compiled base64 regex.
fn base64_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    cached_regex(&RE, r"^[A-Za-z0-9+/]*={0,2}$")
}

/// Returns the compiled JWT structure regex (three base64url-encoded segments).
fn jwt_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    cached_regex(&RE, r"^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$")
}

/// Represents a format validator parsed from user input.
#[derive(Clone, Debug)]
pub enum FormatValidator {
    Email,
    Uuid,
    Hex,
    Base64,
    Jwt,
    Regex(String),
}

impl std::fmt::Display for FormatValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatValidator::Email => write!(f, "email"),
            FormatValidator::Uuid => write!(f, "uuid"),
            FormatValidator::Hex => write!(f, "hex"),
            FormatValidator::Base64 => write!(f, "base64"),
            FormatValidator::Jwt => write!(f, "jwt"),
            FormatValidator::Regex(pat) => write!(f, "regex {}", pat),
        }
    }
}

impl FormatValidator {
    /// Validates the input string against this format.
    pub fn validate(&self, input: &str) -> Result<bool, String> {
        match self {
            FormatValidator::Email => Ok(email_regex().is_match(input)),
            FormatValidator::Uuid => Ok(uuid_regex().is_match(input)),
            FormatValidator::Hex => Ok(!input.is_empty() && hex_regex().is_match(input)),
            FormatValidator::Base64 => Ok(!input.is_empty() && base64_regex().is_match(input)),
            FormatValidator::Jwt => Ok(jwt_regex().is_match(input)),
            FormatValidator::Regex(pattern) => {
                let re =
                    Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;
                Ok(re.is_match(input))
            }
        }
    }
}

#[derive(Clone)]
pub struct SecretValidateFormatCommand;

impl PluginCommand for SecretValidateFormatCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret validate-format"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Custom("secret_string".into()), Type::Bool)])
            .required(
                "format",
                SyntaxShape::String,
                "Format to validate against (email, uuid, hex, base64, jwt, regex)",
            )
            .optional(
                "pattern",
                SyntaxShape::String,
                "Custom regex pattern (required when format is 'regex')",
            )
            .category(Category::Strings)
    }

    fn description(&self) -> &str {
        "Validate if a secret string matches a given format without exposing the content"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: r#""user@example.com" | secret wrap | secret validate-format email"#,
                description: "Validate email format",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""550e8400-e29b-41d4-a716-446655440000" | secret wrap | secret validate-format uuid"#,
                description: "Validate UUID format",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""deadbeef" | secret wrap | secret validate-format hex"#,
                description: "Validate hexadecimal format",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""SGVsbG8gV29ybGQ=" | secret wrap | secret validate-format base64"#,
                description: "Validate base64 format",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abc123" | secret wrap | secret validate-format jwt"#,
                description: "Validate JWT structure",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""ABC-123" | secret wrap | secret validate-format regex "^[A-Z]{3}-\d{3}$""#,
                description: "Validate against a custom regex pattern",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""not-an-email" | secret wrap | secret validate-format email"#,
                description: "Returns false for invalid format",
                result: Some(Value::bool(false, nu_protocol::Span::test_data())),
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
        let format_name = match call.positional.first() {
            Some(Value::String { val, .. }) => val.clone(),
            _ => {
                return Err(LabeledError::new("Missing format parameter").with_label(
                    format!("Supported formats: {}", SUPPORTED_FORMATS),
                    call.head,
                ))
            }
        };

        let validator = parse_format_validator(&format_name, call)?;

        match input {
            PipelineData::Value(value, metadata) => {
                let result = match value {
                    Value::Custom { val, .. } => {
                        validate_secret_format(val.as_ref(), &validator, call.head)?
                    }
                    _ => return Err(LabeledError::new("Invalid input").with_label(
                        "Input must be a SecretString. Use 'secret wrap' to create a secret first",
                        call.head,
                    )),
                };

                Ok(PipelineData::Value(result, metadata))
            }
            _ => Err(LabeledError::new("Invalid input")
                .with_label("Expected a single secret value", call.head)),
        }
    }
}

/// Parses the format name and optional pattern into a `FormatValidator`.
fn parse_format_validator(
    format_name: &str,
    call: &EvaluatedCall,
) -> Result<FormatValidator, LabeledError> {
    match format_name.to_lowercase().as_str() {
        "email" => Ok(FormatValidator::Email),
        "uuid" => Ok(FormatValidator::Uuid),
        "hex" => Ok(FormatValidator::Hex),
        "base64" => Ok(FormatValidator::Base64),
        "jwt" => Ok(FormatValidator::Jwt),
        "regex" => {
            let pattern = match call.positional.get(1) {
                Some(Value::String { val, .. }) => val.clone(),
                _ => {
                    return Err(LabeledError::new("Missing regex pattern").with_label(
                        "The 'regex' format requires a pattern argument: secret validate-format regex \"<pattern>\"",
                        call.head,
                    ))
                }
            };
            Ok(FormatValidator::Regex(pattern))
        }
        _ => Err(
            LabeledError::new(format!("Unsupported format: {}", format_name)).with_label(
                format!("Supported formats: {}", SUPPORTED_FORMATS),
                call.head,
            ),
        ),
    }
}

/// Validates a secret custom value against the given format validator.
fn validate_secret_format(
    val: &dyn nu_protocol::CustomValue,
    validator: &FormatValidator,
    span: nu_protocol::Span,
) -> Result<Value, LabeledError> {
    if let Some(secret_string) = val.as_any().downcast_ref::<SecretString>() {
        let is_valid = validator.validate(secret_string.reveal()).map_err(|e| {
            LabeledError::new(format!("Validation error: {}", e))
                .with_label("Invalid regex pattern", span)
        })?;
        Ok(Value::bool(is_valid, span))
    } else {
        Err(LabeledError::new("Unsupported secret type")
            .with_label("Only SecretString supports validate-format operation", span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::Span;

    #[test]
    fn test_command_name() {
        let command = SecretValidateFormatCommand;
        assert_eq!(command.name(), "secret validate-format");
    }

    #[test]
    fn test_signature() {
        let command = SecretValidateFormatCommand;
        let signature = command.signature();
        assert_eq!(signature.name, "secret validate-format");
        assert_eq!(signature.required_positional.len(), 1);
        assert_eq!(signature.optional_positional.len(), 1);
        assert_eq!(signature.input_output_types.len(), 1);
    }

    #[test]
    fn test_description() {
        let command = SecretValidateFormatCommand;
        assert!(!command.description().is_empty());
        assert!(command.description().len() > 10);
    }

    #[test]
    fn test_examples_count() {
        let command = SecretValidateFormatCommand;
        let examples = command.examples();
        assert_eq!(examples.len(), 7);
    }

    #[test]
    fn test_examples_have_descriptions() {
        let command = SecretValidateFormatCommand;
        let examples = command.examples();

        for example in examples {
            assert!(!example.description.is_empty());
            assert!(example.description.len() > 10);
        }
    }

    #[test]
    fn test_examples_have_valid_results() {
        let command = SecretValidateFormatCommand;
        let examples = command.examples();

        for example in examples {
            if let Some(expected_result) = &example.result {
                match expected_result {
                    Value::Bool { .. } => {}
                    _ => panic!("validate-format command examples should return boolean values"),
                }
            }
        }
    }

    // Email validation tests
    #[test]
    fn test_email_valid() {
        let validator = FormatValidator::Email;
        assert!(validator.validate("user@example.com").unwrap());
        assert!(validator.validate("user.name@example.co.uk").unwrap());
        assert!(validator.validate("user+tag@example.com").unwrap());
        assert!(validator.validate("user123@sub.domain.com").unwrap());
    }

    #[test]
    fn test_email_invalid() {
        let validator = FormatValidator::Email;
        assert!(!validator.validate("not-an-email").unwrap());
        assert!(!validator.validate("@example.com").unwrap());
        assert!(!validator.validate("user@").unwrap());
        assert!(!validator.validate("user@.com").unwrap());
        assert!(!validator.validate("").unwrap());
    }

    // UUID validation tests
    #[test]
    fn test_uuid_valid() {
        let validator = FormatValidator::Uuid;
        assert!(validator
            .validate("550e8400-e29b-41d4-a716-446655440000")
            .unwrap());
        assert!(validator
            .validate("6ba7b810-9dad-11d1-80b4-00c04fd430c8")
            .unwrap());
        assert!(validator
            .validate("f47ac10b-58cc-4372-a567-0e02b2c3d479")
            .unwrap());
    }

    #[test]
    fn test_uuid_invalid() {
        let validator = FormatValidator::Uuid;
        assert!(!validator.validate("not-a-uuid").unwrap());
        assert!(!validator
            .validate("550e8400-e29b-41d4-a716-44665544000")
            .unwrap()); // too short
        assert!(!validator
            .validate("550e8400e29b41d4a716446655440000")
            .unwrap()); // missing dashes
        assert!(!validator.validate("").unwrap());
    }

    // Hex validation tests
    #[test]
    fn test_hex_valid() {
        let validator = FormatValidator::Hex;
        assert!(validator.validate("deadbeef").unwrap());
        assert!(validator.validate("DEADBEEF").unwrap());
        assert!(validator.validate("0123456789abcdef").unwrap());
        assert!(validator.validate("A").unwrap());
    }

    #[test]
    fn test_hex_invalid() {
        let validator = FormatValidator::Hex;
        assert!(!validator.validate("not-hex").unwrap());
        assert!(!validator.validate("0xdeadbeef").unwrap()); // 0x prefix
        assert!(!validator.validate("ghijk").unwrap());
        assert!(!validator.validate("").unwrap()); // empty
    }

    // Base64 validation tests
    #[test]
    fn test_base64_valid() {
        let validator = FormatValidator::Base64;
        assert!(validator.validate("SGVsbG8gV29ybGQ=").unwrap());
        assert!(validator.validate("YWJj").unwrap());
        assert!(validator.validate("dGVzdA==").unwrap());
        assert!(validator.validate("AAAA").unwrap());
    }

    #[test]
    fn test_base64_invalid() {
        let validator = FormatValidator::Base64;
        assert!(!validator.validate("not base64!").unwrap());
        assert!(!validator.validate("===").unwrap());
        assert!(!validator.validate("").unwrap()); // empty
    }

    // JWT validation tests
    #[test]
    fn test_jwt_valid() {
        let validator = FormatValidator::Jwt;
        assert!(validator
            .validate("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abc123")
            .unwrap());
        assert!(validator.validate("aaa.bbb.ccc").unwrap());
        assert!(validator.validate("a-b_c.d-e_f.g-h_i").unwrap());
    }

    #[test]
    fn test_jwt_invalid() {
        let validator = FormatValidator::Jwt;
        assert!(!validator.validate("not-a-jwt").unwrap());
        assert!(!validator.validate("only.two").unwrap()); // only two segments
        assert!(!validator.validate("a.b.c.d").unwrap()); // four segments
        assert!(!validator.validate("").unwrap());
    }

    // Custom regex tests
    #[test]
    fn test_custom_regex_valid() {
        let validator = FormatValidator::Regex(r"^[A-Z]{3}-\d{3}$".to_string());
        assert!(validator.validate("ABC-123").unwrap());
        assert!(!validator.validate("abc-123").unwrap());
        assert!(!validator.validate("ABCD-123").unwrap());
    }

    #[test]
    fn test_custom_regex_invalid_pattern() {
        let validator = FormatValidator::Regex(r"[invalid".to_string());
        assert!(validator.validate("anything").is_err());
    }

    // Format parsing tests
    #[test]
    fn test_format_display() {
        assert_eq!(FormatValidator::Email.to_string(), "email");
        assert_eq!(FormatValidator::Uuid.to_string(), "uuid");
        assert_eq!(FormatValidator::Hex.to_string(), "hex");
        assert_eq!(FormatValidator::Base64.to_string(), "base64");
        assert_eq!(FormatValidator::Jwt.to_string(), "jwt");
        assert_eq!(
            FormatValidator::Regex("^test$".to_string()).to_string(),
            "regex ^test$"
        );
    }

    // SecretString integration tests
    #[test]
    fn test_validate_with_secret_string() {
        let secret = SecretString::new("user@example.com".to_string());
        let validator = FormatValidator::Email;
        assert!(validator.validate(secret.reveal()).unwrap());
    }

    #[test]
    fn test_validate_secret_string_does_not_expose_content() {
        let secret = SecretString::new("user@example.com".to_string());
        let display = format!("{}", secret);
        assert!(!display.contains("user@example.com"));
        assert!(!display.contains("@"));
    }

    // Tests exercising validate_secret_format — the extracted dispatch function

    #[test]
    fn test_validate_secret_format_email_valid() {
        let secret = SecretString::new("user@example.com".to_string());
        let validator = FormatValidator::Email;
        let result = validate_secret_format(&secret, &validator, Span::test_data()).unwrap();
        assert_eq!(result, Value::bool(true, Span::test_data()));
    }

    #[test]
    fn test_validate_secret_format_email_invalid() {
        let secret = SecretString::new("not-an-email".to_string());
        let validator = FormatValidator::Email;
        let result = validate_secret_format(&secret, &validator, Span::test_data()).unwrap();
        assert_eq!(result, Value::bool(false, Span::test_data()));
    }

    #[test]
    fn test_validate_secret_format_invalid_regex() {
        let secret = SecretString::new("anything".to_string());
        let validator = FormatValidator::Regex("[invalid".to_string());
        let result = validate_secret_format(&secret, &validator, Span::test_data());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.msg.contains("Validation error"));
    }

    #[test]
    fn test_validate_secret_format_unsupported_type() {
        use crate::SecretInt;
        let secret = SecretInt::new(42);
        let validator = FormatValidator::Email;
        let result = validate_secret_format(&secret, &validator, Span::test_data());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.msg, "Unsupported secret type");
    }

    // Tests exercising parse_format_validator — the extracted parser function

    fn make_call(positional: Vec<Value>) -> EvaluatedCall {
        EvaluatedCall {
            head: Span::test_data(),
            positional,
            named: vec![],
        }
    }

    #[test]
    fn test_parse_format_validator_email() {
        let call = make_call(vec![Value::test_string("email")]);
        let validator = parse_format_validator("email", &call).unwrap();
        assert!(matches!(validator, FormatValidator::Email));
    }

    #[test]
    fn test_parse_format_validator_uuid() {
        let call = make_call(vec![Value::test_string("uuid")]);
        let validator = parse_format_validator("uuid", &call).unwrap();
        assert!(matches!(validator, FormatValidator::Uuid));
    }

    #[test]
    fn test_parse_format_validator_hex() {
        let call = make_call(vec![Value::test_string("hex")]);
        let validator = parse_format_validator("hex", &call).unwrap();
        assert!(matches!(validator, FormatValidator::Hex));
    }

    #[test]
    fn test_parse_format_validator_base64() {
        let call = make_call(vec![Value::test_string("base64")]);
        let validator = parse_format_validator("base64", &call).unwrap();
        assert!(matches!(validator, FormatValidator::Base64));
    }

    #[test]
    fn test_parse_format_validator_jwt() {
        let call = make_call(vec![Value::test_string("jwt")]);
        let validator = parse_format_validator("jwt", &call).unwrap();
        assert!(matches!(validator, FormatValidator::Jwt));
    }

    #[test]
    fn test_parse_format_validator_regex_with_pattern() {
        let call = make_call(vec![
            Value::test_string("regex"),
            Value::test_string("^test$"),
        ]);
        let validator = parse_format_validator("regex", &call).unwrap();
        assert!(matches!(validator, FormatValidator::Regex(p) if p == "^test$"));
    }

    #[test]
    fn test_parse_format_validator_regex_missing_pattern() {
        let call = make_call(vec![Value::test_string("regex")]);
        let result = parse_format_validator("regex", &call);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.msg, "Missing regex pattern");
    }

    #[test]
    fn test_parse_format_validator_unsupported_format() {
        let call = make_call(vec![Value::test_string("xml")]);
        let result = parse_format_validator("xml", &call);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.msg.contains("Unsupported format"));
    }

    #[test]
    fn test_parse_format_validator_case_insensitive() {
        let call = make_call(vec![Value::test_string("EMAIL")]);
        let validator = parse_format_validator("EMAIL", &call).unwrap();
        assert!(matches!(validator, FormatValidator::Email));
    }
}
