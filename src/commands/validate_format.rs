//! Implements `secret validate-format` — validates secret content against common patterns.

use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::OnceLock;

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};
use regex::Regex;

use crate::SecretString;

/// Supported format names for display in error messages.
const SUPPORTED_FORMATS: &str =
    "email, uuid, hex, base64, jwt, ipv4, ipv6, ssn, credit-card, regex";

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

/// Returns the compiled US Social Security Number regex (XXX-XX-XXXX).
fn ssn_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    cached_regex(&RE, r"^\d{3}-\d{2}-\d{4}$")
}

/// Represents a format validator parsed from user input.
#[derive(Clone, Debug)]
pub enum FormatValidator {
    Email,
    Uuid,
    Hex,
    Base64,
    Jwt,
    Ipv4,
    Ipv6,
    Ssn,
    CreditCard,
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
            FormatValidator::Ipv4 => write!(f, "ipv4"),
            FormatValidator::Ipv6 => write!(f, "ipv6"),
            FormatValidator::Ssn => write!(f, "ssn"),
            FormatValidator::CreditCard => write!(f, "credit-card"),
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
            FormatValidator::Ipv4 => Ok(input.parse::<Ipv4Addr>().is_ok()),
            FormatValidator::Ipv6 => Ok(input.parse::<Ipv6Addr>().is_ok()),
            FormatValidator::Ssn => Ok(validate_ssn(input)),
            FormatValidator::CreditCard => Ok(validate_credit_card(input)),
            FormatValidator::Regex(pattern) => {
                let re =
                    Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;
                Ok(re.is_match(input))
            }
        }
    }
}

/// Validates a US Social Security Number.
///
/// Checks format (XXX-XX-XXXX) and rejects known-invalid area numbers
/// (000, 666, 900-999).
fn validate_ssn(input: &str) -> bool {
    if !ssn_regex().is_match(input) {
        return false;
    }
    // Reject invalid area numbers per SSA rules
    let area: u16 = input[..3].parse().unwrap_or(0);
    area != 0 && area != 666 && area < 900
}

/// Validates a credit card number using the Luhn algorithm.
///
/// Accepts digits with optional dashes or spaces as separators.
/// Valid card lengths are 13-19 digits.
fn validate_credit_card(input: &str) -> bool {
    let digits: Vec<u8> = input
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c as u8 - b'0')
        .collect();

    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    // Ensure input only contains digits, dashes, and spaces
    if input
        .chars()
        .any(|c| !c.is_ascii_digit() && c != '-' && c != ' ')
    {
        return false;
    }

    luhn_check(&digits)
}

/// Performs the Luhn checksum validation on a slice of digits.
fn luhn_check(digits: &[u8]) -> bool {
    let mut sum: u32 = 0;
    let parity = digits.len() % 2;

    for (i, &digit) in digits.iter().enumerate() {
        let mut d = u32::from(digit);
        if i % 2 == parity {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
    }

    sum % 10 == 0
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
                "Format to validate against (email, uuid, hex, base64, jwt, ipv4, ipv6, ssn, credit-card, regex)",
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
                example: r#""192.168.1.1" | secret wrap | secret validate-format ipv4"#,
                description: "Validate IPv4 address format",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""::1" | secret wrap | secret validate-format ipv6"#,
                description: "Validate IPv6 address format",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""078-05-1120" | secret wrap | secret validate-format ssn"#,
                description: "Validate US Social Security Number format",
                result: Some(Value::bool(true, nu_protocol::Span::test_data())),
            },
            Example {
                example: r#""4539578763621486" | secret wrap | secret validate-format credit-card"#,
                description: "Validate credit card number (Luhn check)",
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
        "ipv4" => Ok(FormatValidator::Ipv4),
        "ipv6" => Ok(FormatValidator::Ipv6),
        "ssn" => Ok(FormatValidator::Ssn),
        "credit-card" => Ok(FormatValidator::CreditCard),
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
        assert_eq!(examples.len(), 11);
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

    // IPv4 validation tests
    #[test]
    fn test_ipv4_valid() {
        let validator = FormatValidator::Ipv4;
        assert!(validator.validate("192.168.1.1").unwrap());
        assert!(validator.validate("0.0.0.0").unwrap());
        assert!(validator.validate("255.255.255.255").unwrap());
        assert!(validator.validate("127.0.0.1").unwrap());
        assert!(validator.validate("10.0.0.1").unwrap());
    }

    #[test]
    fn test_ipv4_invalid() {
        let validator = FormatValidator::Ipv4;
        assert!(!validator.validate("256.1.1.1").unwrap()); // octet > 255
        assert!(!validator.validate("1.2.3").unwrap()); // too few octets
        assert!(!validator.validate("1.2.3.4.5").unwrap()); // too many octets
        assert!(!validator.validate("abc.def.ghi.jkl").unwrap());
        assert!(!validator.validate("").unwrap());
        assert!(!validator.validate("192.168.01.1").unwrap()); // leading zero
    }

    // IPv6 validation tests
    #[test]
    fn test_ipv6_valid() {
        let validator = FormatValidator::Ipv6;
        assert!(validator.validate("::1").unwrap()); // loopback
        assert!(validator.validate("::").unwrap()); // unspecified
        assert!(validator
            .validate("2001:0db8:85a3:0000:0000:8a2e:0370:7334")
            .unwrap()); // full form
        assert!(validator.validate("2001:db8:85a3::8a2e:370:7334").unwrap()); // compressed
        assert!(validator.validate("fe80::1").unwrap()); // link-local
        assert!(validator.validate("::ffff:192.0.2.1").unwrap()); // IPv4-mapped
    }

    #[test]
    fn test_ipv6_invalid() {
        let validator = FormatValidator::Ipv6;
        assert!(!validator.validate("not-ipv6").unwrap());
        assert!(!validator
            .validate("2001:db8:85a3::8a2e:370:7334:extra:segment")
            .unwrap()); // too many
        assert!(!validator.validate("").unwrap());
        assert!(!validator.validate("192.168.1.1").unwrap()); // IPv4, not IPv6
        assert!(!validator.validate("gggg::1").unwrap()); // invalid hex
    }

    // SSN validation tests
    #[test]
    fn test_ssn_valid() {
        let validator = FormatValidator::Ssn;
        assert!(validator.validate("078-05-1120").unwrap());
        assert!(validator.validate("123-45-6789").unwrap());
        assert!(validator.validate("001-01-0001").unwrap());
    }

    #[test]
    fn test_ssn_invalid() {
        let validator = FormatValidator::Ssn;
        assert!(!validator.validate("000-12-3456").unwrap()); // area 000 invalid
        assert!(!validator.validate("666-12-3456").unwrap()); // area 666 invalid
        assert!(!validator.validate("900-12-3456").unwrap()); // area 900+ invalid
        assert!(!validator.validate("999-12-3456").unwrap()); // area 999 invalid
        assert!(!validator.validate("123456789").unwrap()); // no dashes
        assert!(!validator.validate("12-345-6789").unwrap()); // wrong dash positions
        assert!(!validator.validate("").unwrap());
    }

    // Credit card validation tests
    #[test]
    fn test_credit_card_valid() {
        let validator = FormatValidator::CreditCard;
        assert!(validator.validate("4539578763621486").unwrap()); // Visa
        assert!(validator.validate("5500000000000004").unwrap()); // Mastercard
        assert!(validator.validate("340000000000009").unwrap()); // Amex (15 digits)
        assert!(validator.validate("4539 5787 6362 1486").unwrap()); // with spaces
        assert!(validator.validate("4539-5787-6362-1486").unwrap()); // with dashes
    }

    #[test]
    fn test_credit_card_invalid() {
        let validator = FormatValidator::CreditCard;
        assert!(!validator.validate("1234567890123456").unwrap()); // fails Luhn
        assert!(!validator.validate("123").unwrap()); // too short
        assert!(!validator.validate("12345678901234567890").unwrap()); // too long
        assert!(!validator.validate("abcdefghijklmnop").unwrap()); // not digits
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
        assert_eq!(FormatValidator::Ipv4.to_string(), "ipv4");
        assert_eq!(FormatValidator::Ipv6.to_string(), "ipv6");
        assert_eq!(FormatValidator::Ssn.to_string(), "ssn");
        assert_eq!(FormatValidator::CreditCard.to_string(), "credit-card");
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
    fn test_parse_format_validator_ipv4() {
        let call = make_call(vec![Value::test_string("ipv4")]);
        let validator = parse_format_validator("ipv4", &call).unwrap();
        assert!(matches!(validator, FormatValidator::Ipv4));
    }

    #[test]
    fn test_parse_format_validator_ipv6() {
        let call = make_call(vec![Value::test_string("ipv6")]);
        let validator = parse_format_validator("ipv6", &call).unwrap();
        assert!(matches!(validator, FormatValidator::Ipv6));
    }

    #[test]
    fn test_parse_format_validator_ssn() {
        let call = make_call(vec![Value::test_string("ssn")]);
        let validator = parse_format_validator("ssn", &call).unwrap();
        assert!(matches!(validator, FormatValidator::Ssn));
    }

    #[test]
    fn test_parse_format_validator_credit_card() {
        let call = make_call(vec![Value::test_string("credit-card")]);
        let validator = parse_format_validator("credit-card", &call).unwrap();
        assert!(matches!(validator, FormatValidator::CreditCard));
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
