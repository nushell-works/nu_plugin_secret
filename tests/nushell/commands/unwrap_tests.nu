# Basic unwrap command tests for nu_plugin_secret
# Tests secret unwrap command with all secret types and error scenarios

use ../setup.nu *
use ../runner.nu [assert, assert_eq, assert_contains, assert_not_contains]
use ../fixtures/secrets.nu *

# Test basic string unwrapping
export def test_unwrap_string [] {
    let original = "my-secret-api-key"
    let secret = $original | secret wrap-string
    let revealed = $secret | secret unwrap
    
    # This assertion will now fail and show the actual issue
    assert_eq $revealed $original $"String should unwrap correctly. Expected: '($original)', Got: '($revealed)'"
    assert_eq ($revealed | describe) "string" "Unwrapped value should be string type"
}

# Test unwrapping all secret types
export def test_unwrap_all_types [] {
    # String
    let str_secret = "test-string" | secret wrap-string
    assert_eq ($str_secret | secret unwrap) "test-string" "String unwrap failed"
    
    # Integer
    let int_secret = 42 | secret wrap-int
    assert_eq ($int_secret | secret unwrap) 42 "Integer unwrap failed"
    
    # Boolean
    let bool_secret = true | secret wrap-bool
    assert_eq ($bool_secret | secret unwrap) true "Boolean unwrap failed"
    
    # Float
    let float_secret = 3.14159 | secret wrap-float
    assert_eq ($float_secret | secret unwrap) 3.14159 "Float unwrap failed"
    
    # Record
    let record_secret = {key: "value", num: 123} | secret wrap-record
    let unwrapped_record = $record_secret | secret unwrap
    assert_eq $unwrapped_record.key "value" "Record field unwrap failed"
    assert_eq $unwrapped_record.num 123 "Record numeric field unwrap failed"
    
    # List
    let list_secret = ["item1", "item2", "item3"] | secret wrap-list
    let unwrapped_list = $list_secret | secret unwrap
    assert_eq ($unwrapped_list | length) 3 "List length should be preserved"
    assert_eq $unwrapped_list.0 "item1" "List element unwrap failed"
    
    # Binary
    let binary_secret = 0x[deadbeef] | secret wrap-binary
    assert_eq ($binary_secret | secret unwrap) 0x[deadbeef] "Binary unwrap failed"
    
    # Date
    let test_date = "2025-01-01T12:00:00Z" | into datetime
    let date_secret = $test_date | secret wrap-date
    let unwrapped_date = $date_secret | secret unwrap
    assert_eq $unwrapped_date $test_date "Date unwrap failed"
}

# Test unwrap preserves data integrity for complex types
export def test_unwrap_data_integrity [] {
    # Complex record with nested structures
    let complex_record = {
        api_keys: ["key1", "key2", "key3"],
        database: {
            host: "localhost",
            port: 5432,
            credentials: {
                username: "admin",
                password: "secret123"
            }
        },
        settings: {
            debug: true,
            timeout: 30.5,
            retries: 3
        }
    }
    
    let secret = $complex_record | secret wrap-record
    let revealed = $secret | secret unwrap
    
    # Check all nested structures are preserved
    assert_eq ($revealed.api_keys | length) 3 "API keys array should be preserved"
    assert_eq $revealed.api_keys.0 "key1" "First API key should be preserved"
    assert_eq $revealed.database.host "localhost" "Database host should be preserved"
    assert_eq $revealed.database.port 5432 "Database port should be preserved"
    assert_eq $revealed.database.credentials.username "admin" "Username should be preserved"
    assert_eq $revealed.database.credentials.password "secret123" "Password should be preserved"
    assert_eq $revealed.settings.debug true "Debug flag should be preserved"
    assert_eq $revealed.settings.timeout 30.5 "Timeout should be preserved"
    assert_eq $revealed.settings.retries 3 "Retries should be preserved"
}

# Test unwrap with various string content
export def test_unwrap_string_variants [] {
    let test_strings = [
        "",  # empty string
        "simple",  # simple string
        get_unicode_test_string,  # unicode
        get_special_chars_string,  # special characters
        (get_long_test_string 1000),  # long string
        "line1\nline2\nline3",  # multiline
        "  spaces  ",  # with spaces
        "\t\r\n",  # whitespace characters
    ]
    
    for test_str in $test_strings {
        let secret = $test_str | secret wrap-string
        let revealed = $secret | secret unwrap
        assert_eq $revealed $test_str $"String variant should unwrap correctly: '($test_str | str substring 0..20)...'"
    }
}

# Test unwrap error handling
export def test_unwrap_non_secret_error [] {
    # Try to unwrap regular string
    try {
        "not-a-secret" | secret unwrap
        assert false "Should have failed to unwrap regular string"
    } catch { |e|
        assert_contains $e.msg "Expected secret type" "Should indicate expected secret type"
    }
    
    # Try to unwrap regular integer
    try {
        42 | secret unwrap
        assert false "Should have failed to unwrap regular integer"
    } catch { |e|
        assert_contains $e.msg "Expected secret type" "Should indicate expected secret type"
    }
    
    # Try to unwrap null/empty
    try {
        null | secret unwrap
        assert false "Should have failed to unwrap null"
    } catch { |e|
        assert_contains $e.msg "Empty" "Should indicate empty input"
    }
}

# Test unwrap with pipeline data
export def test_unwrap_pipeline [] {
    let secrets = ["secret1", "secret2", "secret3"] 
        | each { |s| $s | secret wrap-string }
    
    let revealed = $secrets | each { |s| $s | secret unwrap }
    
    assert_eq ($revealed | length) 3 "Should unwrap all secrets in pipeline"
    assert_eq $revealed.0 "secret1" "First secret should unwrap correctly"
    assert_eq $revealed.1 "secret2" "Second secret should unwrap correctly"
    assert_eq $revealed.2 "secret3" "Third secret should unwrap correctly"
}

# Test round-trip consistency
export def test_unwrap_round_trip_consistency [] {
    let test_data = load_test_data
    
    # Test strings
    for test_str in $test_data.test_strings {
        if $test_str != null {
            let secret = $test_str | secret wrap-string
            let revealed = $secret | secret unwrap
            assert_eq $revealed $test_str $"String round-trip failed for: ($test_str)"
        }
    }
    
    # Test integers
    for test_int in $test_data.test_integers {
        let secret = $test_int | secret wrap-int
        let revealed = $secret | secret unwrap
        assert_eq $revealed $test_int $"Integer round-trip failed for: ($test_int)"
    }
    
    # Test floats
    for test_float in $test_data.test_floats {
        let secret = $test_float | secret wrap-float
        let revealed = $secret | secret unwrap
        assert_eq $revealed $test_float $"Float round-trip failed for: ($test_float)"
    }
    
    # Test booleans
    for test_bool in $test_data.test_booleans {
        let secret = $test_bool | secret wrap-bool
        let revealed = $secret | secret unwrap
        assert_eq $revealed $test_bool $"Boolean round-trip failed for: ($test_bool)"
    }
}

# Test unwrap multiple times (should be consistent)
export def test_unwrap_multiple_calls [] {
    let secret = "consistent-test" | secret wrap-string
    
    let revealed1 = $secret | secret unwrap
    let revealed2 = $secret | secret unwrap
    let revealed3 = $secret | secret unwrap
    
    assert_eq $revealed1 $revealed2 "Multiple unwraps should be consistent"
    assert_eq $revealed2 $revealed3 "Multiple unwraps should be consistent"
    assert_eq $revealed1 "consistent-test" "All unwraps should return original value"
}

# Test unwrap preserves type information
export def test_unwrap_type_preservation [] {
    # Test various types maintain their type after unwrap
    let string_revealed = ("test" | secret wrap-string | secret unwrap)
    assert_eq ($string_revealed | describe) "string" "String type should be preserved"
    
    let int_revealed = (42 | secret wrap-int | secret unwrap)
    assert_eq ($int_revealed | describe) "int" "Int type should be preserved"
    
    let bool_revealed = (true | secret wrap-bool | secret unwrap)
    assert_eq ($bool_revealed | describe) "bool" "Bool type should be preserved"
    
    let float_revealed = (3.14 | secret wrap-float | secret unwrap)
    assert_eq ($float_revealed | describe) "float" "Float type should be preserved"
    
    let record_revealed = ({key: "value"} | secret wrap-record | secret unwrap)
    assert_eq ($record_revealed | describe) "record" "Record type should be preserved"
    
    let list_revealed = (["item"] | secret wrap-list | secret unwrap)
    assert_eq ($list_revealed | describe) "list<string>" "List type should be preserved"
}

# Test unwrap performance
export def test_unwrap_performance [] {
    # Create many secrets
    let secrets = 0..100 | each { |i| $"secret-($i)" | secret wrap-string }
    
    let start_time = date now
    let revealed = $secrets | each { |s| $s | secret unwrap }
    let duration = (date now) - $start_time
    
    assert ($duration < 5sec) $"Should unwrap 101 secrets in under 5 seconds, took ($duration)"
    assert_eq ($revealed | length) 101 "Should have unwrapped all secrets"
    
    # Verify all values are correct
    for i in 0..100 {
        assert_eq $revealed.($i) $"secret-($i)" $"Secret ($i) should unwrap correctly"
    }
}

# Test unwrap with mixed secret types in pipeline
export def test_unwrap_mixed_types [] {
    let mixed_secrets = [
        ("string" | secret wrap-string),
        (42 | secret wrap-int),
        (true | secret wrap-bool),
        (3.14 | secret wrap-float)
    ]
    
    let revealed = $mixed_secrets | each { |s| $s | secret unwrap }
    
    assert_eq ($revealed | length) 4 "Should unwrap all mixed types"
    assert_eq $revealed.0 "string" "String should unwrap correctly"
    assert_eq $revealed.1 42 "Integer should unwrap correctly" 
    assert_eq $revealed.2 true "Boolean should unwrap correctly"
    assert_eq $revealed.3 3.14 "Float should unwrap correctly"
}

# Test unwrap edge cases
export def test_unwrap_edge_cases [] {
    # Very long string
    let long_secret = (get_long_test_string 10000) | secret wrap-string
    let long_revealed = $long_secret | secret unwrap
    assert_eq ($long_revealed | str length) 10000 "Long string length should be preserved"
    
    # Empty collections
    let empty_list = [] | secret wrap-list
    let empty_list_revealed = $empty_list | secret unwrap
    assert_eq ($empty_list_revealed | length) 0 "Empty list should remain empty"
    
    let empty_record = {} | secret wrap-record
    let empty_record_revealed = $empty_record | secret unwrap
    assert_eq ($empty_record_revealed | columns | length) 0 "Empty record should remain empty"
    
    # Extreme numbers
    let max_int = 9223372036854775807 | secret wrap-int
    assert_eq ($max_int | secret unwrap) 9223372036854775807 "Max int should unwrap correctly"
    
    let min_int = -9223372036854775808 | secret wrap-int
    assert_eq ($min_int | secret unwrap) -9223372036854775808 "Min int should unwrap correctly"
}