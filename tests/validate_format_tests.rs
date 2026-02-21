//! Integration tests for the secret validate-format command

use nu_plugin_secret::SecretString;

// ── Email format ──

#[test]
fn test_email_valid_simple() {
    let secret = SecretString::new("user@example.com".to_string());
    assert!(is_valid_email(secret.reveal()));
}

#[test]
fn test_email_valid_with_dots_and_plus() {
    let secret = SecretString::new("first.last+tag@example.co.uk".to_string());
    assert!(is_valid_email(secret.reveal()));
}

#[test]
fn test_email_invalid_no_at() {
    let secret = SecretString::new("not-an-email".to_string());
    assert!(!is_valid_email(secret.reveal()));
}

#[test]
fn test_email_invalid_no_domain() {
    let secret = SecretString::new("user@".to_string());
    assert!(!is_valid_email(secret.reveal()));
}

#[test]
fn test_email_invalid_empty() {
    let secret = SecretString::new(String::new());
    assert!(!is_valid_email(secret.reveal()));
}

// ── UUID format ──

#[test]
fn test_uuid_v4_valid() {
    let secret = SecretString::new("550e8400-e29b-41d4-a716-446655440000".to_string());
    assert!(is_valid_uuid(secret.reveal()));
}

#[test]
fn test_uuid_v1_valid() {
    let secret = SecretString::new("6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string());
    assert!(is_valid_uuid(secret.reveal()));
}

#[test]
fn test_uuid_invalid_no_dashes() {
    let secret = SecretString::new("550e8400e29b41d4a716446655440000".to_string());
    assert!(!is_valid_uuid(secret.reveal()));
}

#[test]
fn test_uuid_invalid_random_string() {
    let secret = SecretString::new("not-a-uuid-at-all".to_string());
    assert!(!is_valid_uuid(secret.reveal()));
}

#[test]
fn test_uuid_invalid_empty() {
    let secret = SecretString::new(String::new());
    assert!(!is_valid_uuid(secret.reveal()));
}

// ── Hex format ──

#[test]
fn test_hex_valid_lowercase() {
    let secret = SecretString::new("deadbeef".to_string());
    assert!(is_valid_hex(secret.reveal()));
}

#[test]
fn test_hex_valid_uppercase() {
    let secret = SecretString::new("DEADBEEF".to_string());
    assert!(is_valid_hex(secret.reveal()));
}

#[test]
fn test_hex_valid_mixed_case() {
    let secret = SecretString::new("DeAdBeEf".to_string());
    assert!(is_valid_hex(secret.reveal()));
}

#[test]
fn test_hex_invalid_with_prefix() {
    let secret = SecretString::new("0xdeadbeef".to_string());
    assert!(!is_valid_hex(secret.reveal()));
}

#[test]
fn test_hex_invalid_non_hex_chars() {
    let secret = SecretString::new("xyz123".to_string());
    assert!(!is_valid_hex(secret.reveal()));
}

#[test]
fn test_hex_invalid_empty() {
    let secret = SecretString::new(String::new());
    assert!(!is_valid_hex(secret.reveal()));
}

// ── Base64 format ──

#[test]
fn test_base64_valid_with_padding() {
    let secret = SecretString::new("SGVsbG8gV29ybGQ=".to_string());
    assert!(is_valid_base64(secret.reveal()));
}

#[test]
fn test_base64_valid_double_padding() {
    let secret = SecretString::new("dGVzdA==".to_string());
    assert!(is_valid_base64(secret.reveal()));
}

#[test]
fn test_base64_valid_no_padding() {
    let secret = SecretString::new("YWJj".to_string());
    assert!(is_valid_base64(secret.reveal()));
}

#[test]
fn test_base64_invalid_special_chars() {
    let secret = SecretString::new("not base64!".to_string());
    assert!(!is_valid_base64(secret.reveal()));
}

#[test]
fn test_base64_invalid_empty() {
    let secret = SecretString::new(String::new());
    assert!(!is_valid_base64(secret.reveal()));
}

// ── JWT format ──

#[test]
fn test_jwt_valid_structure() {
    let secret =
        SecretString::new("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abc123".to_string());
    assert!(is_valid_jwt(secret.reveal()));
}

#[test]
fn test_jwt_valid_minimal() {
    let secret = SecretString::new("a.b.c".to_string());
    assert!(is_valid_jwt(secret.reveal()));
}

#[test]
fn test_jwt_invalid_two_segments() {
    let secret = SecretString::new("only.two".to_string());
    assert!(!is_valid_jwt(secret.reveal()));
}

#[test]
fn test_jwt_invalid_four_segments() {
    let secret = SecretString::new("a.b.c.d".to_string());
    assert!(!is_valid_jwt(secret.reveal()));
}

#[test]
fn test_jwt_invalid_empty() {
    let secret = SecretString::new(String::new());
    assert!(!is_valid_jwt(secret.reveal()));
}

// ── Custom regex ──

#[test]
fn test_regex_custom_pattern_match() {
    let secret = SecretString::new("ABC-123".to_string());
    let re = regex::Regex::new(r"^[A-Z]{3}-\d{3}$").unwrap();
    assert!(re.is_match(secret.reveal()));
}

#[test]
fn test_regex_custom_pattern_no_match() {
    let secret = SecretString::new("abc-123".to_string());
    let re = regex::Regex::new(r"^[A-Z]{3}-\d{3}$").unwrap();
    assert!(!re.is_match(secret.reveal()));
}

// ── Security: redaction is preserved ──

#[test]
fn test_secret_display_does_not_expose_content() {
    let secret = SecretString::new("user@example.com".to_string());
    let display = format!("{}", secret);
    assert!(!display.contains("user@example.com"));
}

#[test]
fn test_secret_debug_does_not_expose_content() {
    let secret = SecretString::new("550e8400-e29b-41d4-a716-446655440000".to_string());
    let debug = format!("{:?}", secret);
    assert!(!debug.contains("550e8400"));
}

// ── Helpers ──
// These mirror the validation logic to test the patterns independently.

fn is_valid_email(s: &str) -> bool {
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(s)
}

fn is_valid_uuid(s: &str) -> bool {
    let re = regex::Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$",
    )
    .unwrap();
    re.is_match(s)
}

fn is_valid_hex(s: &str) -> bool {
    !s.is_empty() && regex::Regex::new(r"^[0-9a-fA-F]+$").unwrap().is_match(s)
}

fn is_valid_base64(s: &str) -> bool {
    !s.is_empty()
        && regex::Regex::new(r"^[A-Za-z0-9+/]*={0,2}$")
            .unwrap()
            .is_match(s)
}

fn is_valid_jwt(s: &str) -> bool {
    regex::Regex::new(r"^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$")
        .unwrap()
        .is_match(s)
}
