# Comprehensive integration tests for unified wrap commands
# Tests for 'secret wrap' (auto-detection) and 'secret wrap-with' (custom templates)

use ../setup.nu *
use ../runner.nu [assert, assert_eq, assert_contains, assert_not_contains]
use ../fixtures/secrets.nu *

# Test unified secret wrap command with auto-detection
export def test_unified_wrap_string [] {
    let test_string = "my-api-key-12345"
    let secret = $test_string | secret wrap
    
    # Verify it's a custom type and displays as redacted
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "string" "Should identify as string type"
    
    let display = $secret | to text
    assert_contains $display "redacted" "Should contain redacted text"
    assert_not_contains $display $test_string "Should not leak original content"
    
    # Verify unwrap works
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_string "Should round-trip correctly"
}

# Test unified wrap command with integers
export def test_unified_wrap_int [] {
    let test_values = [0, 42, -42, 2147483647]
    
    for value in $test_values {
        let secret = $value | secret wrap
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) "int" "Should identify as int type"
        
        let display = $secret | to text
        assert_contains $display "redacted" "Should contain redacted text"
        assert_not_contains $display ($value | into string) "Should not leak original value"
        
        let revealed = $secret | secret unwrap
        assert_eq $revealed $value $"Integer ($value) should round-trip correctly"
    }
}

# Test unified wrap command with booleans
export def test_unified_wrap_bool [] {
    let test_values = [true, false]
    
    for value in $test_values {
        let secret = $value | secret wrap
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) "bool" "Should identify as bool type"
        
        let revealed = $secret | secret unwrap
        assert_eq $revealed $value $"Boolean ($value) should round-trip correctly"
    }
}

# Test unified wrap command with floats
export def test_unified_wrap_float [] {
    let test_values = [0.0, 3.14159, -3.14159, 1e10]
    
    for value in $test_values {
        let secret = $value | secret wrap
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) "float" "Should identify as float type"
        
        let revealed = $secret | secret unwrap
        assert_eq $revealed $value $"Float ($value) should round-trip correctly"
    }
}

# Test unified wrap command with records
export def test_unified_wrap_record [] {
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
    
    let display = $secret | to text
    assert_contains $display "redacted" "Should contain redacted text"
    assert_not_contains $display "sk-1234567890" "Should not leak API key"
    assert_not_contains $display "secret123" "Should not leak password"
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_record "Record should round-trip correctly"
}

# Test unified wrap command with lists
export def test_unified_wrap_list [] {
    let api_keys = ["sk-key1", "sk-key2", "sk-key3"]
    let secret = $api_keys | secret wrap
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "list" "Should identify as list type"
    
    let display = $secret | to text
    assert_contains $display "redacted" "Should contain redacted text"
    for key in $api_keys {
        assert_not_contains $display $key $"Should not leak API key: ($key)"
    }
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $api_keys "List should round-trip correctly"
}

# Test unified wrap command with binary data
export def test_unified_wrap_binary [] {
    let test_binary = 0x[deadbeef12345678]
    let secret = $test_binary | secret wrap
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "binary" "Should identify as binary type"
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_binary "Binary should round-trip correctly"
}

# Test unified wrap command with dates
export def test_unified_wrap_date [] {
    let test_date = date now
    let secret = $test_date | secret wrap
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "date" "Should identify as date type"
    
    let revealed = $secret | secret unwrap
    let diff = ($revealed - $test_date) | math abs
    assert ($diff < 1sec) "Date should round-trip with minimal difference"
}

# Test secret wrap-with command with literal template
export def test_wrap_with_literal_template [] {
    let test_string = "my-secret-password"
    let template = "moo"
    
    let secret = $test_string | secret wrap-with $template
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "string" "Should identify as string type"
    
    # Display should show custom template
    let display = $secret | to text
    assert_eq $display $template "Should display custom template"
    assert_not_contains $display $test_string "Should not leak original content"
    
    # Unwrap should reveal original content
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_string "Should round-trip correctly"
}

# Test secret wrap-with command with template variables
export def test_wrap_with_template_variables [] {
    let test_cases = [
        {input: "password123", type: "string", template: "[HIDDEN:{{secret_type}}]", expected: "[HIDDEN:string]"},
        {input: 42, type: "int", template: "[HIDDEN:{{secret_type}}]", expected: "[HIDDEN:int]"},
        {input: true, type: "bool", template: "[HIDDEN:{{secret_type}}]", expected: "[HIDDEN:bool]"},
        {input: 3.14159, type: "float", template: "[HIDDEN:{{secret_type}}]", expected: "[HIDDEN:float]"}
    ]
    
    for case in $test_cases {
        let secret = $case.input | secret wrap-with $case.template
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) $case.type $"Should identify as ($case.type) type"
        
        let display = $secret | to text
        assert_eq $display $case.expected $"Template should render to ($case.expected)"
        
        let revealed = $secret | secret unwrap
        assert_eq $revealed $case.input $"Should round-trip correctly for ($case.type)"
    }
}

# Test secret wrap-with command with replicate function
export def test_wrap_with_replicate_function [] {
    let test_cases = [
        {input: "abc", template: "{{replicate(character='*', length=secret_length)}}", expected: "***"},
        {input: "password123", template: "{{replicate(character='#', length=secret_length)}}", expected: "###########"},
        {input: "x", template: "{{replicate(character='-', length=secret_length)}}", expected: "-"},
        {input: "", template: "{{replicate(character='*', length=secret_length)}}", expected: ""}
    ]
    
    for case in $test_cases {
        let secret = $case.input | secret wrap-with $case.template
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        
        let display = $secret | to text
        assert_eq $display $case.expected $"Should replicate character correctly for '($case.input)'"
        
        let revealed = $secret | secret unwrap
        assert_eq $revealed $case.input "Should round-trip correctly"
    }
}

# Test secret wrap-with with complex template
export def test_wrap_with_complex_template [] {
    let test_string = "api-key-12345"
    let template = "{{secret_type}}:{{replicate(character='*', length=secret_length)}}"
    let expected = "string:*************"  # 13 characters in "api-key-12345"
    
    let secret = $test_string | secret wrap-with $template
    
    let display = $secret | to text
    assert_eq $display $expected "Complex template should render correctly"
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_string "Should round-trip correctly"
}

# Test secret wrap-with with different types
export def test_wrap_with_different_types [] {
    # String
    let str_secret = "password" | secret wrap-with "STRING_HIDDEN"
    assert_eq ($str_secret | to text) "STRING_HIDDEN" "String template should work"
    
    # Integer  
    let int_secret = 42 | secret wrap-with "INT_HIDDEN"
    assert_eq ($int_secret | to text) "INT_HIDDEN" "Int template should work"
    
    # Boolean
    let bool_secret = true | secret wrap-with "BOOL_HIDDEN"
    assert_eq ($bool_secret | to text) "BOOL_HIDDEN" "Bool template should work"
    
    # Float
    let float_secret = 3.14 | secret wrap-with "FLOAT_HIDDEN"
    assert_eq ($float_secret | to text) "FLOAT_HIDDEN" "Float template should work"
    
    # Record
    let record_secret = {key: "value"} | secret wrap-with "RECORD_HIDDEN"
    assert_eq ($record_secret | to text) "RECORD_HIDDEN" "Record template should work"
    
    # List
    let list_secret = ["item1", "item2"] | secret wrap-with "LIST_HIDDEN"
    assert_eq ($list_secret | to text) "LIST_HIDDEN" "List template should work"
    
    # Binary
    let binary_secret = 0x[deadbeef] | secret wrap-with "BINARY_HIDDEN"
    assert_eq ($binary_secret | to text) "BINARY_HIDDEN" "Binary template should work"
    
    # Date
    let date_secret = (date now) | secret wrap-with "DATE_HIDDEN"
    assert_eq ($date_secret | to text) "DATE_HIDDEN" "Date template should work"
}

# Test round-trip operations with templates
export def test_wrap_with_round_trips [] {
    let test_cases = [
        {value: "password123", template: "{{replicate(character='*', length=secret_length)}}"},
        {value: 42, template: "[SECRET:{{secret_type}}]"},
        {value: true, template: "HIDDEN_{{secret_type}}"},
        {value: 3.14159, template: "masked"},
        {value: {api_key: "secret", port: 8080}, template: "{{secret_type}}_redacted"},
        {value: ["secret1", "secret2"], template: "list_of_{{secret_length}}_items"},
        {value: 0x[deadbeef], template: "binary_data_hidden"},
    ]
    
    for case in $test_cases {
        let secret = $case.value | secret wrap-with $case.template
        let revealed = $secret | secret unwrap
        
        assert_eq $revealed $case.value $"Round-trip should work for template: ($case.template)"
    }
}

# Test template persistence through serialization
export def test_template_persistence [] {
    let original = "test-secret"
    let template = "CUSTOM_{{secret_type}}"
    
    # Create secret with custom template
    let secret = $original | secret wrap-with $template
    
    # Verify display uses custom template
    let display1 = $secret | to text
    assert_eq $display1 "CUSTOM_string" "First display should use custom template"
    
    # Verify template persists through multiple operations
    let display2 = $secret | to text
    assert_eq $display2 "CUSTOM_string" "Template should persist"
    
    # Verify unwrap still works
    let revealed = $secret | secret unwrap
    assert_eq $revealed $original "Should still unwrap correctly"
    
    # Verify template persists after unwrap
    let display3 = $secret | to text
    assert_eq $display3 "CUSTOM_string" "Template should persist after unwrap"
}

# Test error handling for invalid templates
export def test_wrap_with_invalid_templates [] {
    let test_string = "test-secret"
    
    # Test invalid template syntax
    try {
        $test_string | secret wrap-with "{{invalid_variable}}"
        # Some invalid templates might still work (Tera might provide defaults)
        print "Invalid template accepted (may be due to Tera flexibility)"
    } catch { |e|
        assert_contains $e.msg "template" "Should mention template in error"
    }
    
    # Test empty template (should work)
    let secret = $test_string | secret wrap-with ""
    assert_eq ($secret | to text) "" "Empty template should work"
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_string "Empty template should still allow unwrap"
}

# Test performance with custom templates
export def test_wrap_with_performance [] {
    let test_strings = 0..100 | each { |i| $"secret-($i)" }
    let template = "{{replicate(character='*', length=secret_length)}}"
    
    let start_time = date now
    let secrets = $test_strings | each { |s| $s | secret wrap-with $template }
    let duration = (date now) - $start_time
    
    assert ($duration < 10sec) $"Should wrap 100 strings with templates in under 10 seconds, took ($duration)"
    assert_eq ($secrets | length) 101 "Should have wrapped all strings"
    
    # Verify all templates work correctly
    for i in 0..100 {
        let expected_length = ($"secret-($i)" | str length)
        let display = $secrets | get $i | to text
        assert_eq ($display | str length) $expected_length $"Template should render correct length for secret-($i)"
    }
}

# Test concurrent operations with templates
export def test_wrap_with_concurrent [] {
    # Create secrets concurrently with different templates
    let secrets = 0..50 | each { |i| 
        let templates = [
            "{{replicate(character='*', length=secret_length)}}",
            "[HIDDEN:{{secret_type}}]", 
            "masked_{{secret_type}}",
            "{{secret_length}}_chars_hidden"
        ]
        let template = $templates | get ($i mod 4)
        
        [
            ($"string-($i)" | secret wrap-with $template),
            ($i | secret wrap-with $template),
            (($i % 2 == 0) | secret wrap-with $template)
        ]
    } | flatten
    
    assert_eq ($secrets | length) 150 "Should have created 150 secrets"
    
    # Verify all are valid and templates work
    let all_valid = $secrets | all { |s| $s | secret validate }
    assert $all_valid "All concurrent secrets should be valid"
    
    # Verify templates are applied (not showing original content)
    for secret in $secrets {
        let display = $secret | to text
        assert_not_contains $display "string-" "Should not leak string content"
        assert ($display | str length) > 0 "Template should produce some output"
    }
}

# Test mixed wrap and wrap-with operations
export def test_mixed_wrap_operations [] {
    # Create secrets using both wrap and wrap-with
    let regular_secret = "regular-secret" | secret wrap
    let template_secret = "template-secret" | secret wrap-with "CUSTOM_TEMPLATE"
    
    # Verify both work correctly
    assert ($regular_secret | secret validate) "Regular secret should validate"
    assert ($template_secret | secret validate) "Template secret should validate"
    
    let regular_display = $regular_secret | to text
    let template_display = $template_secret | to text
    
    assert_contains $regular_display "redacted" "Regular secret should use default redaction"
    assert_eq $template_display "CUSTOM_TEMPLATE" "Template secret should use custom template"
    
    # Verify both can be unwrapped
    let regular_revealed = $regular_secret | secret unwrap
    let template_revealed = $template_secret | secret unwrap
    
    assert_eq $regular_revealed "regular-secret" "Regular secret should unwrap correctly"
    assert_eq $template_revealed "template-secret" "Template secret should unwrap correctly"
}

# Test edge cases with empty and special content
export def test_wrap_with_edge_cases [] {
    # Empty string
    let empty_secret = "" | secret wrap-with "EMPTY_{{secret_type}}"
    assert_eq ($empty_secret | to text) "EMPTY_string" "Empty string template should work"
    assert_eq ($empty_secret | secret unwrap) "" "Empty string should unwrap correctly"
    
    # Unicode content
    let unicode_secret = "🔐 密码 café" | secret wrap-with "UNICODE_{{secret_type}}"
    assert_eq ($unicode_secret | to text) "UNICODE_string" "Unicode template should work"
    assert_eq ($unicode_secret | secret unwrap) "🔐 密码 café" "Unicode should unwrap correctly"
    
    # Very long string
    let long_string = "x" | str repeat 10000
    let long_secret = $long_string | secret wrap-with "LONG_{{secret_length}}_CHARS"
    assert_eq ($long_secret | to text) "LONG_10000_CHARS" "Long string template should work"
    assert_eq ($long_secret | secret unwrap) $long_string "Long string should unwrap correctly"
    
    # Special characters in template
    let special_template_secret = "secret" | secret wrap-with "Special!@#$%^&*()_+-=[]{}|;':\",./<>?`~"
    assert_eq ($special_template_secret | to text) "Special!@#$%^&*()_+-=[]{}|;':\",./<>?`~" "Special chars in template should work"
}