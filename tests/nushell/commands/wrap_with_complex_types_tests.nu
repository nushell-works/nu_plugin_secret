# Integration tests for wrap-with command with complex types
# Tests the new to_parsable_string() functionality with template variables and functions

use ../setup.nu *
use ../runner.nu [assert, assert_eq, assert_contains, assert_not_contains]

# Test wrap-with for SecretRecord with template functions
export def test_wrap_with_record_template_functions [] {
    let test_record = {name: "Alice", age: 30, active: true}
    
    # Test with secret_string function to access the record content
    let template = "Record type: {{secret_type}}, content: {{secret_string()}}"
    let secret = $test_record | secret wrap-with $template
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "record" "Should identify as record type"
    
    let display = $secret | to text
    assert_contains $display "Record type: record" "Should show record type"
    assert_contains $display "content: " "Should show content prefix"
    # The content should be parsable Nu syntax for the record
    assert_contains $display "name:" "Should contain record field name"
    assert_contains $display "Alice" "Should contain record field value"
    assert_contains $display "age:" "Should contain age field"
    assert_contains $display "30" "Should contain age value"
    assert_contains $display "active:" "Should contain active field"
    
    # Verify unwrap works
    let revealed = $secret | secret unwrap
    assert_eq $revealed.name $test_record.name "Should round-trip record fields correctly"
    assert_eq $revealed.age $test_record.age "Should round-trip age correctly"  
    assert_eq $revealed.active $test_record.active "Should round-trip boolean correctly"
}

# Test wrap-with for SecretRecord with complex template expressions
export def test_wrap_with_record_complex_templates [] {
    let test_record = {api_key: "secret123", endpoint: "https://api.example.com", retries: 3}
    
    let test_cases = [
        {
            template: "{{secret_type | upper}}: {{take(n=10, s=secret_string())}}...",
            description: "Should use take function with record content"
        },
        {
            template: "Config[{{secret_type}}] with {{secret_length}} chars total",
            description: "Should show length of parsed record string"
        },
        {
            template: "{{reverse(s=secret_type)}} = {{secret_string()}}",
            description: "Should reverse type name and show full content"
        }
    ]
    
    for case in $test_cases {
        let secret = $test_record | secret wrap-with $case.template
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) "record" "Should identify as record type"
        
        let display = $secret | to text
        print $"Testing template: ($case.template)"
        print $"Result: ($display)"
        
        # Verify the template processed correctly (basic checks)
        assert (($display | str length) > 0) $"Display should not be empty for: ($case.description)"
        
        # Verify unwrap works
        let revealed = $secret | secret unwrap  
        assert_eq $revealed.api_key $test_record.api_key $"Should round-trip correctly for: ($case.description)"
    }
}

# Test wrap-with for SecretList with template functions  
export def test_wrap_with_list_template_functions [] {
    let test_list = ["apple", "banana", "cherry", 42, true]
    
    # Test with secret_string function to access the list content
    let template = "List[{{secret_type}}]: {{secret_string()}}"
    let secret = $test_list | secret wrap-with $template
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "list" "Should identify as list type"
    
    let display = $secret | to text
    assert_contains $display "List[list]:" "Should show list type"
    # The content should be parsable Nu syntax for the list
    assert_contains $display "apple" "Should contain first list item"
    assert_contains $display "banana" "Should contain second list item" 
    assert_contains $display "cherry" "Should contain third list item"
    assert_contains $display "42" "Should contain integer item"
    assert_contains $display "true" "Should contain boolean item"
    
    # Verify unwrap works
    let revealed = $secret | secret unwrap
    assert_eq ($revealed | length) ($test_list | length) "Should have same list length"
    assert_eq ($revealed | get 0) ($test_list | get 0) "Should round-trip first item"
    assert_eq ($revealed | get 3) ($test_list | get 3) "Should round-trip integer item"
    assert_eq ($revealed | get 4) ($test_list | get 4) "Should round-trip boolean item"
}

# Test wrap-with for SecretList with length-based templates
export def test_wrap_with_list_length_templates [] {
    let test_cases = [
        {
            list: ["a"],
            template: "SingleItem[{{secret_length}}]: {{secret_string()}}",
            description: "Single item list"
        },
        {
            list: ["x", "y", "z"],  
            template: "Items({{secret_length}}): {{take(n=20, s=secret_string())}}",
            description: "Three item list with take function"
        },
        {
            list: [],
            template: "Empty[{{secret_length}}]: {{secret_string()}}",
            description: "Empty list"
        }
    ]
    
    for case in $test_cases {
        let secret = $case.list | secret wrap-with $case.template
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) "list" "Should identify as list type"
        
        let display = $secret | to text
        print $"Testing: ($case.description)"
        print $"Template: ($case.template)"
        print $"Result: ($display)"
        
        # Verify template processed  
        assert (($display | str length) > 0) $"Display should not be empty for: ($case.description)"
        
        # Verify unwrap works
        let revealed = $secret | secret unwrap
        assert_eq ($revealed | length) ($case.list | length) $"Should round-trip correctly for: ($case.description)"
    }
}

# Test wrap-with for SecretBinary with template functions
export def test_wrap_with_binary_template_functions [] {
    let test_binary = 0x[deadbeef42]
    
    # Test with secret_string function to access the binary content  
    let template = "Binary[{{secret_type}}] {{secret_length}} bytes: {{secret_string()}}"
    let secret = $test_binary | secret wrap-with $template
    
    assert_eq ($secret | describe) "custom" "Should be custom type"
    assert ($secret | secret validate) "Should validate as secret"
    assert_eq ($secret | secret type-of) "binary" "Should identify as binary type"
    
    let display = $secret | to text
    assert_contains $display "Binary[binary]" "Should show binary type"
    assert_contains $display "5 bytes:" "Should show correct byte length"
    # The content should be parsable Nu binary syntax
    assert_contains $display "0x[" "Should contain binary prefix"
    assert_contains $display "deadbeef42" "Should contain hex content"
    assert_contains $display "]" "Should contain binary suffix"
    
    # Verify unwrap works
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_binary "Should round-trip binary correctly"
}

# Test wrap-with for SecretBinary with length-aware templates
export def test_wrap_with_binary_length_templates [] {
    let test_cases = [
        {
            binary: 0x[ff],
            template: "Byte{{secret_length}}: {{secret_string()}}",
            description: "Single byte"
        },
        {
            binary: 0x[deadbeefcafebabe],
            template: "Data[{{secret_length}} bytes]: {{reverse(s=secret_string())}}",
            description: "8 bytes with reverse function"
        },
        {
            binary: 0x[],
            template: "Empty[{{secret_length}}]: {{secret_string()}}",
            description: "Empty binary"  
        }
    ]
    
    for case in $test_cases {
        let secret = $case.binary | secret wrap-with $case.template
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) "binary" "Should identify as binary type"
        
        let display = $secret | to text
        print $"Testing: ($case.description)"
        print $"Template: ($case.template)"
        print $"Result: ($display)"
        
        # Verify template processed
        assert (($display | str length) > 0) $"Display should not be empty for: ($case.description)"
        
        # Verify unwrap works
        let revealed = $secret | secret unwrap
        assert_eq $revealed $case.binary $"Should round-trip correctly for: ($case.description)"
    }
}

# Test complex template combinations with different complex types
export def test_wrap_with_complex_type_combinations [] {
    let test_cases = [
        {
            value: {users: ["alice", "bob"], config: {debug: true, port: 8080}},
            template: "{{secret_type | upper}}: {{take(n=50, s=secret_string())}}...",
            type: "record",
            description: "Nested record with list"
        },
        {
            value: [[1, 2], [3, 4], ["a", "b"]],
            template: "Matrix[{{secret_type}}]: {{secret_string()}}",
            type: "list", 
            description: "List of lists"
        },
        {
            value: 0x[89504e470d0a1a0a], # PNG file header
            template: "File[{{secret_length}}]: {{take(n=30, s=secret_string())}}",
            type: "binary",
            description: "Binary file header"
        }
    ]
    
    for case in $test_cases {
        let secret = $case.value | secret wrap-with $case.template
        
        assert_eq ($secret | describe) "custom" "Should be custom type"
        assert ($secret | secret validate) "Should validate as secret"
        assert_eq ($secret | secret type-of) $case.type $"Should identify as ($case.type) type"
        
        let display = $secret | to text
        print $"Testing: ($case.description)"
        print $"Result: ($display)"
        
        # Verify template processed
        assert (($display | str length) > 0) $"Display should not be empty for: ($case.description)"
        
        # Verify unwrap works
        let revealed = $secret | secret unwrap
        assert_eq $revealed $case.value $"Should round-trip correctly for: ($case.description)"
    }
}

# Test error handling with complex types and invalid templates
export def test_wrap_with_complex_types_error_handling [] {
    let test_record = {key: "value"}
    
    # Test templates that should work
    try {
        let secret = $test_record | secret wrap-with "{{secret_type}}: {{secret_string()}}"
        assert ($secret | secret validate) "Valid template should work"
    } catch { |e|
        assert false $"Valid template should not fail: ($e.msg)"
    }
    
    # Test empty template (should work)
    try {
        let secret = $test_record | secret wrap-with ""
        assert_eq ($secret | to text) "" "Empty template should work"
        let revealed = $secret | secret unwrap
        assert_eq $revealed.key "value" "Empty template should still allow unwrap"
    } catch { |e|
        assert false $"Empty template should not fail: ($e.msg)"
    }
    
    # Test template with only whitespace
    try {
        let secret = $test_record | secret wrap-with "   "
        assert_eq ($secret | to text) "   " "Whitespace template should work"
    } catch { |e|
        assert false $"Whitespace template should not fail: ($e.msg)"
    }
}

# Test performance with complex types and templates
export def test_wrap_with_complex_types_performance [] {
    # Create various complex test data
    let large_record = (0..50 | each { |i| {key: $"field_($i)", value: $"data_($i)"} } | reduce -f {} { |it, acc| $acc | merge $it })
    let large_list = (0..100 | each { |i| $"item_($i)" })
    let large_binary = (0..255 | each { |i| $i } | into binary)
    
    let template = "{{secret_type}}: {{take(n=100, s=secret_string())}}"
    let start_time = date now
    
    # Test performance with different complex types
    let secrets = [
        ($large_record | secret wrap-with $template),
        ($large_list | secret wrap-with $template), 
        ($large_binary | secret wrap-with $template)
    ]
    
    let duration = (date now) - $start_time
    
    assert ($duration < 5sec) $"Should wrap complex types with templates in under 5 seconds, took ($duration)"
    assert_eq ($secrets | length) 3 "Should have wrapped all complex types"
    
    # Verify all templates work correctly
    for secret in $secrets {
        assert ($secret | secret validate) "All complex type secrets should validate"
        let display = $secret | to text
        assert (($display | str length) > 0) "All displays should have content"
    }
}

# Test concurrent operations with complex types
export def test_wrap_with_complex_types_concurrent [] {
    let template = "Concurrent[{{secret_type}}]: {{secret_string()}}"
    
    # Create multiple complex secrets concurrently
    let secrets = 0..10 | par-each { |i|
        [
            ({key: $"value_($i)"} | secret wrap-with $template),
            ([$"item1_($i)", $"item2_($i)"] | secret wrap-with $template),
            ((0..($i + 1) | into binary) | secret wrap-with $template)
        ]
    } | flatten
    
    assert_eq ($secrets | length) 33 "Should have created 33 secrets (11 * 3)"
    
    # Verify all are valid and templates work
    let all_valid = $secrets | all { |s| $s | secret validate }
    assert $all_valid "All concurrent complex type secrets should be valid"
    
    # Verify templates are applied (showing type information)
    for secret in $secrets {
        let display = $secret | to text
        assert_contains $display "Concurrent[" "Should contain template prefix"
        assert (($display | str contains "record") or ($display | str contains "list") or ($display | str contains "binary")) "Should contain type information"
    }
}