# Test runner for unified wrap commands (secret wrap and secret wrap-with)
# This tests the new functionality that was recently implemented

use setup.nu *

# Simple assertion function
def assert [condition: bool, message?: string] {
    if not $condition {
        let error_msg = $message | default "Assertion failed"
        error make { msg: $error_msg }
    }
}

def assert_eq [actual: any, expected: any, message?: string] {
    if $actual != $expected {
        let error_msg = $message | default $"Expected ($expected), got ($actual)"
        error make { msg: $error_msg }
    }
}

def assert_contains [haystack: string, needle: string, message?: string] {
    if not ($haystack | str contains $needle) {
        let error_msg = $message | default $"Expected '($haystack)' to contain '($needle)'"
        error make { msg: $error_msg }
    }
}

def assert_not_contains [haystack: string, needle: string, message?: string] {
    if ($haystack | str contains $needle) {
        let error_msg = $message | default $"Expected '($haystack)' to NOT contain '($needle)'"
        error make { msg: $error_msg }
    }
}

# Test unified secret wrap command
def test_unified_wrap [] {
    print "🧪 Testing unified secret wrap..."
    
    # Test string auto-detection
    let test_string = "my-secret-key"
    let secret = $test_string | secret wrap
    
    assert (($secret | describe) == "secret_string") "Should be secret_string type"
    assert ($secret | secret validate) "Should validate as secret"
    
    let display = $secret | to text
    assert_contains $display "redacted" "Should contain redacted text"
    assert_not_contains $display $test_string "Should not leak original content"
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_string "Should round-trip correctly"
    
    print "  ✅ String auto-detection works"
    
    # Test integer auto-detection
    let test_int = 42
    let int_secret = $test_int | secret wrap
    
    assert (($int_secret | describe) == "secret_int") "Should be secret_int type"
    assert ($int_secret | secret validate) "Should validate as secret"
    
    let int_revealed = $int_secret | secret unwrap
    assert_eq $int_revealed $test_int "Integer should round-trip correctly"
    
    print "  ✅ Integer auto-detection works"
    
    # Test boolean auto-detection
    let test_bool = true
    let bool_secret = $test_bool | secret wrap
    
    assert (($bool_secret | describe) == "secret_bool") "Should be secret_bool type"
    assert ($bool_secret | secret validate) "Should validate as secret"
    
    let bool_revealed = $bool_secret | secret unwrap
    assert_eq $bool_revealed $test_bool "Boolean should round-trip correctly"
    
    print "  ✅ Boolean auto-detection works"
}

# Test secret wrap-with command with custom templates
def test_wrap_with_templates [] {
    print "🧪 Testing secret wrap-with..."
    
    # Test literal template
    let test_string = "my-password"
    let template = "moo"
    let secret = $test_string | secret wrap-with $template
    
    assert (($secret | describe) == "secret_string") "Should be secret_string type"
    assert ($secret | secret validate) "Should validate as secret"
    
    let display = $secret | to text
    assert_eq $display $template "Should display custom template"
    assert_not_contains $display $test_string "Should not leak original content"
    
    let revealed = $secret | secret unwrap
    assert_eq $revealed $test_string "Should round-trip correctly"
    
    print "  ✅ Literal template works"
    
    # Test template with variables
    let template_with_vars = "[HIDDEN:{{secret_type}}]"
    let secret_with_vars = $test_string | secret wrap-with $template_with_vars
    
    let display_with_vars = $secret_with_vars | to text
    assert_eq $display_with_vars "[HIDDEN:string]" "Should render template variables"
    
    print "  ✅ Template variables work"
    
    # Test replicate function
    let replicate_template = "{{replicate(character='*', length=secret_length)}}"
    let replicate_secret = $test_string | secret wrap-with $replicate_template
    
    let replicate_display = $replicate_secret | to text
    let expected_length = ($test_string | str length)
    assert_eq ($replicate_display | str length) $expected_length "Should replicate correct number of characters"
    
    # Check if all characters are asterisks
    let all_asterisks = $replicate_display | split chars | all { $in == "*" }
    assert $all_asterisks "Should be all asterisks"
    
    print "  ✅ Replicate function works"
}

# Test template persistence through operations
def test_template_persistence [] {
    print "🧪 Testing template persistence..."
    
    let original = "test-secret"
    let template = "CUSTOM_{{secret_type}}"
    
    # Create secret with custom template
    let secret = $original | secret wrap-with $template
    
    # Verify display uses custom template multiple times
    let display1 = $secret | to text
    assert_eq $display1 "CUSTOM_string" "First display should use custom template"
    
    let display2 = $secret | to text
    assert_eq $display2 "CUSTOM_string" "Template should persist"
    
    # Verify unwrap still works
    let revealed = $secret | secret unwrap
    assert_eq $revealed $original "Should still unwrap correctly"
    
    # Verify template persists after unwrap
    let display3 = $secret | to text
    assert_eq $display3 "CUSTOM_string" "Template should persist after unwrap"
    
    print "  ✅ Template persistence works"
}

# Test different data types with templates
def test_different_types_with_templates [] {
    print "🧪 Testing different types with templates..."
    
    let template = "{{secret_type}}_HIDDEN"
    
    # String - should work
    let str_secret = "password" | secret wrap-with $template
    assert_eq ($str_secret | to text) "string_HIDDEN" "String template should work"
    
    # Integer - should work (we fixed this)
    let int_secret = 42 | secret wrap-with $template
    assert_eq ($int_secret | to text) "int_HIDDEN" "Int template should work"
    
    print "  ✅ Templates work with string and int types"
    print "  ℹ️  Other types (bool, float, etc.) still need custom template support"
}

# Test edge cases
def test_edge_cases [] {
    print "🧪 Testing edge cases..."
    
    # Empty string
    let empty_secret = "" | secret wrap-with "EMPTY"
    assert_eq ($empty_secret | to text) "EMPTY" "Empty string template should work"
    assert_eq ($empty_secret | secret unwrap) "" "Empty string should unwrap correctly"
    
    # Empty template
    let empty_template_secret = "secret" | secret wrap-with ""
    assert_eq ($empty_template_secret | to text) "" "Empty template should work"
    assert_eq ($empty_template_secret | secret unwrap) "secret" "Should still unwrap with empty template"
    
    print "  ✅ Edge cases work"
}

# Run all tests
def main [] {
    print "🚀 Testing Unified Wrap Commands"
    print "================================"
    
    # Setup plugin first
    setup_plugin
    
    let start_time = date now
    
    try {
        test_unified_wrap
        test_wrap_with_templates
        test_template_persistence
        test_different_types_with_templates
        test_edge_cases
        
        let duration = (date now) - $start_time
        print ""
        print $"✅ All tests passed! (took ($duration))"
        
    } catch { |e|
        print $"❌ Test failed: ($e.msg)"
        cleanup_plugin
        exit 1
    }
    
    # Cleanup
    cleanup_plugin
}