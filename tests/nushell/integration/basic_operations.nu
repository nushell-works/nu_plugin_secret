# Basic integration tests for nu_plugin_secret
# End-to-end workflow testing with real-world scenarios

use ../setup.nu *
use ../runner.nu [assert, assert_eq, assert_contains, assert_not_contains, time_operation]
use ../fixtures/secrets.nu *

# Test complete workflow: wrap -> validate -> unwrap
export def test_complete_workflow [] {
    let test_cases = [
        {type: "string", value: "api-key-12345", wrap_cmd: "secret wrap-string"},
        {type: "int", value: 8080, wrap_cmd: "secret wrap-int"}, 
        {type: "bool", value: true, wrap_cmd: "secret wrap-bool"},
        {type: "float", value: 3.14159, wrap_cmd: "secret wrap-float"}
    ]
    
    for case in $test_cases {
        # Step 1: Wrap the value
        let secret = $case.value | do { |val| 
            match $case.type {
                "string" => { $val | secret wrap-string },
                "int" => { $val | secret wrap-int },
                "bool" => { $val | secret wrap-bool },
                "float" => { $val | secret wrap-float },
                _ => { error make {msg: $"Unknown type: ($case.type)"} }
            }
        }
        
        # Step 2: Validate it's a secret
        let is_valid = $secret | secret validate
        assert $is_valid $"($case.type) secret should validate"
        
        # Step 3: Check type identification
        let identified_type = $secret | secret type-of
        assert_eq $identified_type $case.type $"Should identify as ($case.type)"
        
        # Step 4: Verify redaction
        let display = $secret | to text
        assert_contains $display "redacted" $"($case.type) should show redacted"
        
        # Step 5: Unwrap and verify
        let revealed = $secret | secret unwrap
        assert_eq $revealed $case.value $"($case.type) should unwrap to original value"
    }
}

# Test plugin info and help commands
export def test_plugin_info [] {
    let info = secret info
    
    # Verify info structure
    assert ($info | get plugin_name | is-not-empty) "Plugin name should be present"
    assert ($info | get version | is-not-empty) "Version should be present"
    assert_eq ($info | get plugin_name) "nu_plugin_secret" "Plugin name should be correct"
    
    # Test help for various commands
    let help_commands = [
        "secret wrap-string",
        "secret wrap-int", 
        "secret unwrap",
        "secret validate",
        "secret info"
    ]
    
    for cmd in $help_commands {
        try {
            let help_output = help $cmd
            assert ($help_output | str length) > 0 $"Help for ($cmd) should not be empty"
        } catch { |e|
            assert false $"Help for ($cmd) should be available: ($e.msg)"
        }
    }
}

# Test secrets in data structures
export def test_secrets_in_data_structures [] {
    # Test secrets in records
    let config = {
        app_name: "MyApp",
        api_key: ("sk-1234567890" | secret wrap-string),
        database: {
            host: "localhost",
            port: 5432,
            password: ("secret123" | secret wrap-string)
        },
        debug: true,
        timeout: 30
    }
    
    # Verify secrets are preserved in record
    assert ($config.api_key | secret validate) "API key should remain a secret"
    assert ($config.database.password | secret validate) "Database password should remain a secret"
    
    # Verify non-secrets remain unchanged
    assert_eq $config.app_name "MyApp" "App name should remain unchanged"
    assert_eq $config.database.port 5432 "Port should remain unchanged"
    assert_eq $config.debug true "Debug flag should remain unchanged"
    
    # Test secrets in lists
    let api_keys = [
        ("key1" | secret wrap-string),
        ("key2" | secret wrap-string),
        ("key3" | secret wrap-string)
    ]
    
    for i in 0..2 {
        assert ($api_keys.($i) | secret validate) $"API key ($i) should be a secret"
    }
    
    # Test mixed list with secrets and regular values
    let mixed_list = [
        "public-info",
        ("secret-data" | secret wrap-string),
        42,
        ("another-secret" | secret wrap-string)
    ]
    
    assert_eq $mixed_list.0 "public-info" "Public info should remain unchanged"
    assert ($mixed_list.1 | secret validate) "First secret should be valid"
    assert_eq $mixed_list.2 42 "Number should remain unchanged"
    assert ($mixed_list.3 | secret validate) "Second secret should be valid"
}

# Test pipeline operations with secrets
export def test_pipeline_operations [] {
    # Create a list of sensitive data
    let sensitive_data = [
        "password1",
        "password2", 
        "password3",
        "password4"
    ]
    
    # Wrap all as secrets in pipeline
    let secrets = $sensitive_data 
        | each { |item| $item | secret wrap-string }
    
    # Verify all are secrets
    let all_secrets = $secrets 
        | all { |item| $item | secret validate }
    assert $all_secrets "All items should be secrets"
    
    # Filter secrets (keeping them as secrets)
    let filtered_secrets = $secrets 
        | enumerate 
        | where index < 2
        | get item
    
    assert_eq ($filtered_secrets | length) 2 "Should have 2 filtered secrets"
    assert ($filtered_secrets.0 | secret validate) "Filtered secret 0 should be valid"
    assert ($filtered_secrets.1 | secret validate) "Filtered secret 1 should be valid"
    
    # Map over secrets (unwrap, transform, wrap again)
    let transformed_secrets = $secrets 
        | each { |secret| 
            let value = $secret | secret unwrap
            $"transformed-($value)" | secret wrap-string
        }
    
    let first_transformed = $transformed_secrets.0 | secret unwrap
    assert_eq $first_transformed "transformed-password1" "Transformation should work correctly"
}

# Test error handling and recovery
export def test_error_handling [] {
    # Test graceful handling of invalid operations
    let test_cases = [
        {
            desc: "Wrap wrong type",
            test: { 42 | secret wrap-string },
            should_fail: true,
            error_contains: "Expected string"
        },
        {
            desc: "Unwrap non-secret",
            test: { "regular-string" | secret unwrap },
            should_fail: true,
            error_contains: "Expected secret type"
        },
        {
            desc: "Validate non-secret",
            test: { "regular-string" | secret validate },
            should_fail: false,
            expected_result: false
        },
        {
            desc: "Type-of non-secret",
            test: { "regular-string" | secret type-of },
            should_fail: true,
            error_contains: "Expected secret type"
        }
    ]
    
    for case in $test_cases {
        if $case.should_fail {
            try {
                do $case.test
                assert false $"($case.desc) should have failed"
            } catch { |e|
                if "error_contains" in $case {
                    assert_contains $e.msg $case.error_contains $"($case.desc) should have appropriate error"
                }
            }
        } else {
            let result = do $case.test
            if "expected_result" in $case {
                assert_eq $result $case.expected_result $"($case.desc) should return expected result"
            }
        }
    }
}

# Test performance characteristics
export def test_performance_characteristics [] {
    # Test startup time (should be reasonable)
    let startup_test = time_operation { secret info | ignore }
    assert ($startup_test.duration < 2sec) $"Plugin startup should be under 2 seconds, was ($startup_test.duration)"
    
    # Test bulk operations
    let bulk_data = 0..50 | each { |i| $"bulk-secret-($i)" }
    
    let bulk_wrap_test = time_operation {
        $bulk_data | each { |item| $item | secret wrap-string } | ignore
    }
    assert ($bulk_wrap_test.duration < 5sec) $"Bulk wrap (51 items) should be under 5 seconds, was ($bulk_wrap_test.duration)"
    
    # Test validation performance
    let secrets = $bulk_data | each { |item| $item | secret wrap-string }
    let bulk_validate_test = time_operation {
        $secrets | each { |s| $s | secret validate } | ignore
    }
    assert ($bulk_validate_test.duration < 3sec) $"Bulk validate (51 items) should be under 3 seconds, was ($bulk_validate_test.duration)"
}

# Test memory behavior with large data
export def test_memory_behavior [] {
    # Test with progressively larger strings
    let sizes = [100, 1000, 5000]
    
    for size in $sizes {
        let large_string = "x" | fill --character "x" --length $size
        let secret = $large_string | secret wrap-string
        let revealed = $secret | secret unwrap
        
        assert_eq ($revealed | str length) $size $"Large string ($size) should preserve length"
        assert_eq $revealed $large_string $"Large string ($size) should round-trip correctly"
        
        # Verify redaction still works
        let display = $secret | to text
        assert_contains $display "redacted" $"Large string ($size) should still be redacted"
        assert_not_contains $display "xxxxx" $"Large string ($size) content should not leak"
    }
}

# Test concurrent operations (simulate rapid usage)
export def test_concurrent_operations [] {
    # Simulate rapid creation and usage of secrets
    let concurrent_secrets = 0..20 | each { |i|
        let secrets = [
            ($"string-($i)" | secret wrap-string),
            ($i | secret wrap-int),
            (($i % 2 == 0) | secret wrap-bool)
        ]
        
        # Immediately use each secret
        $secrets | each { |s|
            {
                is_valid: ($s | secret validate),
                type: ($s | secret type-of),
                unwrapped: ($s | secret unwrap)
            }
        }
    } | flatten
    
    assert_eq ($concurrent_secrets | length) 63 "Should handle 63 concurrent operations"
    
    # Verify all operations succeeded
    let all_valid = $concurrent_secrets | all { |result| $result.is_valid }
    assert $all_valid "All concurrent secrets should be valid"
}

# Test plugin state consistency
export def test_plugin_state_consistency [] {
    # Test that plugin maintains consistent state across operations
    let initial_info = secret info
    
    # Perform various operations
    let _ = "test1" | secret wrap-string | secret unwrap
    let _ = 42 | secret wrap-int | secret validate
    let _ = true | secret wrap-bool | secret type-of
    
    let final_info = secret info
    
    # Plugin info should remain consistent
    assert_eq ($initial_info | get plugin_name) ($final_info | get plugin_name) "Plugin name should remain consistent"
    assert_eq ($initial_info | get version) ($final_info | get version) "Version should remain consistent"
}

# Test real-world usage patterns
export def test_real_world_patterns [] {
    # Pattern 1: Configuration file with secrets
    let app_config = {
        database_url: ("postgresql://user:pass@localhost/db" | secret wrap-string),
        api_keys: {
            stripe: ("sk_test_1234567890" | secret wrap-string),
            openai: ("sk-1234567890abcdef" | secret wrap-string)
        },
        jwt_secret: ("super-secret-jwt-key" | secret wrap-string),
        debug_mode: false,
        port: 3000
    }
    
    # Verify secrets are protected
    assert ($app_config.database_url | secret validate) "Database URL should be secret"
    assert ($app_config.api_keys.stripe | secret validate) "Stripe key should be secret"
    assert ($app_config.api_keys.openai | secret validate) "OpenAI key should be secret"
    assert ($app_config.jwt_secret | secret validate) "JWT secret should be secret"
    
    # Verify non-secrets remain accessible
    assert_eq $app_config.debug_mode false "Debug mode should be accessible"
    assert_eq $app_config.port 3000 "Port should be accessible"
    
    # Pattern 2: Processing list of credentials
    let user_credentials = [
        {username: "admin", password: ("admin123" | secret wrap-string)},
        {username: "user1", password: ("password1" | secret wrap-string)},
        {username: "user2", password: ("password2" | secret wrap-string)}
    ]
    
    # Verify we can work with the structure while keeping secrets protected
    let usernames = $user_credentials | get username
    assert_eq ($usernames | length) 3 "Should extract all usernames"
    assert_eq $usernames.0 "admin" "Should get correct username"
    
    # Verify passwords remain secret
    let passwords_are_secret = $user_credentials 
        | get password 
        | all { |p| $p | secret validate }
    assert $passwords_are_secret "All passwords should remain secret"
}