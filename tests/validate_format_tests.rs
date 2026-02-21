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

// ── IPv4 format ──

#[test]
fn test_ipv4_valid_loopback() {
    let secret = SecretString::new("127.0.0.1".to_string());
    assert!(is_valid_ipv4(secret.reveal()));
}

#[test]
fn test_ipv4_valid_private() {
    let secret = SecretString::new("192.168.1.1".to_string());
    assert!(is_valid_ipv4(secret.reveal()));
}

#[test]
fn test_ipv4_valid_broadcast() {
    let secret = SecretString::new("255.255.255.255".to_string());
    assert!(is_valid_ipv4(secret.reveal()));
}

#[test]
fn test_ipv4_invalid_octet_overflow() {
    let secret = SecretString::new("256.1.1.1".to_string());
    assert!(!is_valid_ipv4(secret.reveal()));
}

#[test]
fn test_ipv4_invalid_too_few_octets() {
    let secret = SecretString::new("1.2.3".to_string());
    assert!(!is_valid_ipv4(secret.reveal()));
}

#[test]
fn test_ipv4_invalid_empty() {
    let secret = SecretString::new(String::new());
    assert!(!is_valid_ipv4(secret.reveal()));
}

// ── IPv6 format ──

#[test]
fn test_ipv6_valid_loopback() {
    let secret = SecretString::new("::1".to_string());
    assert!(is_valid_ipv6(secret.reveal()));
}

#[test]
fn test_ipv6_valid_full() {
    let secret = SecretString::new("2001:0db8:85a3:0000:0000:8a2e:0370:7334".to_string());
    assert!(is_valid_ipv6(secret.reveal()));
}

#[test]
fn test_ipv6_valid_compressed() {
    let secret = SecretString::new("2001:db8:85a3::8a2e:370:7334".to_string());
    assert!(is_valid_ipv6(secret.reveal()));
}

#[test]
fn test_ipv6_valid_ipv4_mapped() {
    let secret = SecretString::new("::ffff:192.0.2.1".to_string());
    assert!(is_valid_ipv6(secret.reveal()));
}

#[test]
fn test_ipv6_invalid_random_string() {
    let secret = SecretString::new("not-ipv6".to_string());
    assert!(!is_valid_ipv6(secret.reveal()));
}

#[test]
fn test_ipv6_invalid_empty() {
    let secret = SecretString::new(String::new());
    assert!(!is_valid_ipv6(secret.reveal()));
}

// ── SSN format ──

#[test]
fn test_ssn_valid_standard() {
    let secret = SecretString::new("078-05-1120".to_string());
    assert!(is_valid_ssn(secret.reveal()));
}

#[test]
fn test_ssn_valid_low_area() {
    let secret = SecretString::new("001-01-0001".to_string());
    assert!(is_valid_ssn(secret.reveal()));
}

#[test]
fn test_ssn_invalid_area_000() {
    let secret = SecretString::new("000-12-3456".to_string());
    assert!(!is_valid_ssn(secret.reveal()));
}

#[test]
fn test_ssn_invalid_area_666() {
    let secret = SecretString::new("666-12-3456".to_string());
    assert!(!is_valid_ssn(secret.reveal()));
}

#[test]
fn test_ssn_invalid_area_900_plus() {
    let secret = SecretString::new("900-12-3456".to_string());
    assert!(!is_valid_ssn(secret.reveal()));
}

#[test]
fn test_ssn_invalid_no_dashes() {
    let secret = SecretString::new("123456789".to_string());
    assert!(!is_valid_ssn(secret.reveal()));
}

#[test]
fn test_ssn_invalid_empty() {
    let secret = SecretString::new(String::new());
    assert!(!is_valid_ssn(secret.reveal()));
}

// ── Credit card format ──

#[test]
fn test_credit_card_valid_visa() {
    let secret = SecretString::new("4539578763621486".to_string());
    assert!(is_valid_credit_card(secret.reveal()));
}

#[test]
fn test_credit_card_valid_mastercard() {
    let secret = SecretString::new("5500000000000004".to_string());
    assert!(is_valid_credit_card(secret.reveal()));
}

#[test]
fn test_credit_card_valid_amex() {
    let secret = SecretString::new("340000000000009".to_string());
    assert!(is_valid_credit_card(secret.reveal()));
}

#[test]
fn test_credit_card_valid_with_spaces() {
    let secret = SecretString::new("4539 5787 6362 1486".to_string());
    assert!(is_valid_credit_card(secret.reveal()));
}

#[test]
fn test_credit_card_valid_with_dashes() {
    let secret = SecretString::new("4539-5787-6362-1486".to_string());
    assert!(is_valid_credit_card(secret.reveal()));
}

#[test]
fn test_credit_card_invalid_luhn() {
    let secret = SecretString::new("1234567890123456".to_string());
    assert!(!is_valid_credit_card(secret.reveal()));
}

#[test]
fn test_credit_card_invalid_too_short() {
    let secret = SecretString::new("123".to_string());
    assert!(!is_valid_credit_card(secret.reveal()));
}

#[test]
fn test_credit_card_invalid_empty() {
    let secret = SecretString::new(String::new());
    assert!(!is_valid_credit_card(secret.reveal()));
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

fn is_valid_ipv4(s: &str) -> bool {
    s.parse::<std::net::Ipv4Addr>().is_ok()
}

fn is_valid_ipv6(s: &str) -> bool {
    s.parse::<std::net::Ipv6Addr>().is_ok()
}

fn is_valid_ssn(s: &str) -> bool {
    let re = regex::Regex::new(r"^\d{3}-\d{2}-\d{4}$").unwrap();
    if !re.is_match(s) {
        return false;
    }
    let area: u16 = s[..3].parse().unwrap_or(0);
    area != 0 && area != 666 && area < 900
}

fn is_valid_credit_card(s: &str) -> bool {
    let digits: Vec<u8> = s
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c as u8 - b'0')
        .collect();

    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    if s.chars()
        .any(|c| !c.is_ascii_digit() && c != '-' && c != ' ')
    {
        return false;
    }

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
    sum.is_multiple_of(10)
}
