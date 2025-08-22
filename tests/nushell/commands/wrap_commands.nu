# Basic wrap commands tests for nu_plugin_secret
# Tests all 8 wrap commands with edge cases and validation

use ../setup.nu *
use ../runner.nu [assert, assert_eq, assert_contains, assert_not_contains]
use ../fixtures/secrets.nu *

# Test basic string wrapping
export def test_wrap_string_basic [] {
    let test_string = "my-api-key-12345"
    let secret = $test_string | secret wrap-string
    
    # Verify it's a custom type
    assert_eq ($secret | describe) "custom" "Should be custom type"
    
    # Verify it displays as redacted
    let display = $secret | to text
    assert_contains $display "redacted" "Should contain redacted text"
    assert_not_contains $display $test_string "Should not leak original content"
    
    # Verify validation works
    assert ($secret | secret validate) "Should validate as secret"
    
    # Verify type detection
    assert_eq ($secret | secret type-of) "string" "Should identify as string type"
}

# Test empty string handling
export def test_wrap_string_empty [] {
    let secret = "" | secret wrap-string
    let revealed = $secret | secret unwrap
    
    assert_eq $revealed "" "Empty string should round-trip correctly"
    assert ($secret | secret validate) "Empty secret should still validate"
}

# Test unicode content
export def test_wrap_string_unicode [] {
    let unicode_string = get_unicode_test_string
    let secret = $unicode_string | secret wrap-string
    let revealed = $secret | secret unwrap
    
    assert_eq $revealed $unicode_string "Unicode should round-trip correctly"
    
    let display = $secret | to text
    assert_not_contains $display "🔐" "Should not leak emoji"
    assert_not_contains $display "中文" "Should not leak Chinese characters"
}

# Test special characters
export def test_wrap_string_special_chars [] {
    let special_string = get_special_chars_string
    let secret = $special_string | secret wrap-string
    let revealed = $secret | secret unwrap
    
    assert_eq $revealed $special_string "Special characters should round-trip correctly"
}

# Test long strings
export def test_wrap_string_long [] {
    let long_string = get_long_test_string 5000
    let secret = $long_string | secret wrap-string
    let revealed = $secret | secret unwrap
    
    assert_eq ($revealed | str length) 5000 "Long string length should be preserved"
    assert_eq $revealed $long_string "Long string should round-trip correctly"
}

# Test integer wrapping
export def test_wrap_int_basic [] {
    let test_values = [0, 42, -42, 2147483647, -2147483648]
    
    for value in $test_values {
        let secret = $value | secret wrap-int
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) "int" "Should identify as int type"
        
        let revealed = $secret | secret unwrap
        assert_eq $revealed $value $"Integer ($value) should round-trip correctly"
        
        let display = $secret | to text
        assert_contains $display "redacted" "Should contain redacted text"
        assert_not_contains $display ($value | into string) "Should not leak original value"
    }
}

# Test boolean wrapping
export def test_wrap_bool_basic [] {
    let test_values = [true, false]
    
    for value in $test_values {
        let secret = $value | secret wrap-bool
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) "bool" "Should identify as bool type"
        
        let revealed = $secret | secret unwrap
        assert_eq $revealed $value $"Boolean ($value) should round-trip correctly"
        
        let display = $secret | to text
        assert_contains $display "redacted" "Should contain redacted text"
    }
}

# Test float wrapping
export def test_wrap_float_basic [] {
    let test_values = [0.0, 3.14159, -3.14159, 1e10, -1e10]
    
    for value in $test_values {
        let secret = $value | secret wrap-float
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) "float" "Should identify as float type"
        
        let revealed = $secret | secret unwrap
        assert_eq $revealed $value $"Float ($value) should round-trip correctly"
    }
}

# Test record wrapping
export def test_wrap_record_basic [] {
    let test_record = {
        api_key: "sk-1234567890",
        database_password: "secret123",
        port: 5432,
        ssl_enabled: true
    }
    
    let secret = $test_record | secret wrap-record
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "record" "Should identify as record type"
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_record "Record should round-trip correctly"
    
    let display = $secret | to text
    assert_contains $display "redacted" "Should contain redacted text"
    assert_not_contains $display "sk-1234567890" "Should not leak API key"
    assert_not_contains $display "secret123" "Should not leak password"
}

# Test list wrapping
export def test_wrap_list_basic [] {
    let api_keys = get_test_api_keys
    let secret = $api_keys | secret wrap-list
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "list" "Should identify as list type"
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $api_keys "List should round-trip correctly"
    
    let display = $secret | to text
    assert_contains $display "redacted" "Should contain redacted text"
    for key in $api_keys {
        assert_not_contains $display $key $"Should not leak API key: ($key)"
    }
}

# Test binary wrapping
export def test_wrap_binary_basic [] {
    let test_binary = 0x[deadbeef]
    let secret = $test_binary | secret wrap-binary
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "binary" "Should identify as binary type"
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_binary "Binary should round-trip correctly"
}

# Test date wrapping
export def test_wrap_date_basic [] {
    let test_date = date now
    let secret = $test_date | secret wrap-date
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "date" "Should identify as date type"
    
    let revealed = $secret | secret unwrap
    # Allow small time difference due to processing time
    let diff = ($revealed - $test_date) | math abs
    assert ($diff < 1sec) "Date should round-trip with minimal difference"
}

# Test error handling for wrong types
export def test_wrap_type_errors [] {
    # Try to wrap integer as string
    try {
        42 | secret wrap-string
        assert false "Should have failed to wrap int as string"
    } catch { |e|
        assert_contains $e.msg "Expected string" "Should have appropriate error message"
    }
    
    # Try to wrap string as int
    try {
        "not-a-number" | secret wrap-int
        assert false "Should have failed to wrap string as int"
    } catch { |e|
        assert_contains $e.msg "Expected int" "Should have appropriate error message"
    }
}

# Test wrapping with empty pipeline
export def test_wrap_empty_pipeline [] {
    try {
        null | secret wrap-string
        assert false "Should have failed on empty input"
    } catch { |e|
        assert_contains $e.msg "Empty" "Should indicate empty input"
    }
}

# Test all wrap commands exist and have proper signatures
export def test_all_wrap_commands_exist [] {
    let expected_commands = [
        "secret wrap-string",
        "secret wrap-int", 
        "secret wrap-bool",
        "secret wrap-record",
        "secret wrap-list",
        "secret wrap-float",
        "secret wrap-binary",
        "secret wrap-date"
    ]
    
    for cmd in $expected_commands {
        # Test that command exists by running help
        try {
            help $cmd | ignore
        } catch { |e|
            assert false $"Command ($cmd) should exist"
        }
    }
}

# Performance test - wrap many secrets quickly
export def test_wrap_performance [] {
    let test_strings = 0..100 | each { |i| $"secret-($i)" }
    
    let start_time = date now
    let secrets = $test_strings | each { |s| $s | secret wrap-string }
    let duration = (date now) - $start_time
    
    assert ($duration < 5sec) $"Should wrap 100 strings in under 5 seconds, took ($duration)"
    assert_eq ($secrets | length) 101 "Should have wrapped all strings"
    
    # Verify all are valid secrets
    let all_valid = $secrets | all { |s| $s | secret validate }
    assert $all_valid "All wrapped values should be valid secrets"
}

# Test concurrent access doesn't cause issues
export def test_wrap_concurrent [] {
    # Simulate concurrent access by rapidly creating secrets
    let secrets = 0..50 | each { |i| 
        [
            ($"string-($i)" | secret wrap-string),
            ($i | secret wrap-int),
            (($i % 2 == 0) | secret wrap-bool)
        ]
    } | flatten
    
    assert_eq ($secrets | length) 150 "Should have created 150 secrets"
    
    # Verify all are valid
    let all_valid = $secrets | all { |s| $s | secret validate }
    assert $all_valid "All concurrent secrets should be valid"
}