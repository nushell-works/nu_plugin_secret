# Basic wrap commands tests for nu_plugin_secret
# Tests unified wrap command with edge cases and validation

use ../setup.nu *
use ../runner.nu [assert, assert_eq, assert_contains, assert_not_contains]
use ../fixtures/secrets.nu *

# Test basic string wrapping
export def test_wrap_string_basic [] {
    let test_string = "my-api-key-12345"
    let secret = $test_string | secret wrap
    
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
    let secret = "" | secret wrap
    let revealed = $secret | secret unwrap
    
    assert_eq $revealed "" "Empty string should round-trip correctly"
    assert ($secret | secret validate) "Empty secret should still validate"
}

# Test unicode content
export def test_wrap_string_unicode [] {
    let unicode_string = get_unicode_test_string
    let secret = $unicode_string | secret wrap
    let revealed = $secret | secret unwrap
    
    assert_eq $revealed $unicode_string "Unicode should round-trip correctly"
    
    let display = $secret | to text
    assert_not_contains $display "🔐" "Should not leak emoji"
    assert_not_contains $display "中文" "Should not leak Chinese characters"
}

# Test special characters
export def test_wrap_string_special_chars [] {
    let special_string = get_special_chars_string
    let secret = $special_string | secret wrap
    let revealed = $secret | secret unwrap
    
    assert_eq $revealed $special_string "Special characters should round-trip correctly"
}

# Test long strings
export def test_wrap_string_long [] {
    let long_string = get_long_test_string 5000
    let secret = $long_string | secret wrap
    let revealed = $secret | secret unwrap
    
    assert_eq ($revealed | str length) 5000 "Long string length should be preserved"
    assert_eq $revealed $long_string "Long string should round-trip correctly"
}

# Test integer wrapping
export def test_wrap_int_basic [] {
    let test_values = [0, 42, -42, 2147483647, -2147483648]
    
    for value in $test_values {
        let secret = $value | secret wrap
        
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
        let secret = $value | secret wrap
        
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
        let secret = $value | secret wrap
        
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
    
    let secret = $test_record | secret wrap
    
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
    let secret = $api_keys | secret wrap
    
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
    let secret = $test_binary | secret wrap
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "binary" "Should identify as binary type"
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_binary "Binary should round-trip correctly"
}

# Test date wrapping
export def test_wrap_date_basic [] {
    let test_date = date now
    let secret = $test_date | secret wrap
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "date" "Should identify as date type"
    
    let revealed = $secret | secret unwrap
    # Allow small time difference due to processing time
    let diff = ($revealed - $test_date) | math abs
    assert ($diff < 1sec) "Date should round-trip with minimal difference"
}

# Test unified wrap command handles all types automatically
export def test_wrap_type_detection [] {
    # Test that unified wrap command automatically detects types
    let string_secret = "test" | secret wrap
    assert_eq ($string_secret | secret type-of) "string" "Should detect string type"
    
    let int_secret = 42 | secret wrap  
    assert_eq ($int_secret | secret type-of) "int" "Should detect int type"
    
    let bool_secret = true | secret wrap
    assert_eq ($bool_secret | secret type-of) "bool" "Should detect bool type"
    
    let float_secret = 3.14 | secret wrap
    assert_eq ($float_secret | secret type-of) "float" "Should detect float type"
}

# Test wrapping with empty pipeline
export def test_wrap_empty_pipeline [] {
    try {
        null | secret wrap
        assert false "Should have failed on empty input"
    } catch { |e|
        assert_contains $e.msg "Empty" "Should indicate empty input"
    }
}

# Test unified wrap command exists and has proper signature
export def test_unified_wrap_command_exists [] {
    # Test that unified command exists by running help
    try {
        help "secret wrap" | ignore
    } catch { |e|
        assert false "Unified wrap command should exist"
    }
    
    # Test other utility commands still exist
    let utility_commands = [
        "secret unwrap",
        "secret validate", 
        "secret type-of",
        "secret info"
    ]
    
    for cmd in $utility_commands {
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
    let secrets = $test_strings | each { |s| $s | secret wrap }
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
            ($"string-($i)" | secret wrap),
            ($i | secret wrap),
            (($i % 2 == 0) | secret wrap)
        ]
    } | flatten
    
    assert_eq ($secrets | length) 150 "Should have created 150 secrets"
    
    # Verify all are valid
    let all_valid = $secrets | all { |s| $s | secret validate }
    assert $all_valid "All concurrent secrets should be valid"
}