# Simple test runner for basic functionality verification
# This avoids the dynamic sourcing issues in the main runner

use setup.nu *

# Simple assertion function
def assert [condition: bool, message?: string] {
    if not $condition {
        let error_msg = $message | default "Assertion failed"
        error make { msg: $error_msg }
    }
}

# Test basic wrap-string functionality
def test_wrap_string [] {
    print "🧪 Testing wrap-string..."
    
    let test_string = "my-api-key-12345"
    let secret = $test_string | secret wrap-string
    
    # Verify it's a custom type
    assert (($secret | describe) == "secret_string") "Should be secret_string type"
    
    # Verify it displays as redacted
    let display = $secret | to text
    assert ($display | str contains "redacted") "Should contain redacted text"
    assert (not ($display | str contains $test_string)) "Should not leak original content"
    
    # Verify validation works
    assert ($secret | secret validate) "Should validate as secret"
    
    # Verify type detection
    assert (($secret | secret type-of) == "string") "Should identify as string type"
    
    print "✅ wrap-string test passed"
}

# Test comprehensive unwrap functionality for all secret types
def test_unwrap_all_types [] {
    print "🧪 Testing unwrap for all secret types..."
    
    # Test String
    print "  Testing String unwrap..."
    let original_string = "my-secret-value"
    let secret_string = $original_string | secret wrap-string
    let revealed_string = $secret_string | secret unwrap
    assert ($revealed_string == $original_string) $"String unwrap failed. Expected: ($original_string), Got: ($revealed_string)"
    assert (($revealed_string | describe) == "string") "String unwrap should preserve type"
    
    # Test Integer
    print "  Testing Integer unwrap..."
    let original_int = 42
    let secret_int = $original_int | secret wrap-int
    let revealed_int = $secret_int | secret unwrap
    assert ($revealed_int == $original_int) $"Integer unwrap failed. Expected: ($original_int), Got: ($revealed_int)"
    assert (($revealed_int | describe) == "int") "Integer unwrap should preserve type"
    
    # Test Boolean
    print "  Testing Boolean unwrap..."
    let original_bool = true
    let secret_bool = $original_bool | secret wrap-bool
    let revealed_bool = $secret_bool | secret unwrap
    assert ($revealed_bool == $original_bool) $"Boolean unwrap failed. Expected: ($original_bool), Got: ($revealed_bool)"
    assert (($revealed_bool | describe) == "bool") "Boolean unwrap should preserve type"
    
    # Test Float
    print "  Testing Float unwrap..."
    let original_float = 3.14159
    let secret_float = $original_float | secret wrap-float
    let revealed_float = $secret_float | secret unwrap
    assert ($revealed_float == $original_float) $"Float unwrap failed. Expected: ($original_float), Got: ($revealed_float)"
    assert (($revealed_float | describe) == "float") "Float unwrap should preserve type"
    
    # Test Record
    print "  Testing Record unwrap..."
    let original_record = {name: "test", value: 123, active: true}
    let secret_record = $original_record | secret wrap-record
    let revealed_record = $secret_record | secret unwrap
    assert ($revealed_record == $original_record) $"Record unwrap failed. Expected: ($original_record), Got: ($revealed_record)"
    let record_type = $revealed_record | describe
    print $"  Record type: ($record_type)"
    assert (($record_type | str starts-with "record")) "Record unwrap should preserve record type"
    assert ($revealed_record.name == "test") "Record field should be preserved"
    assert ($revealed_record.value == 123) "Record numeric field should be preserved"
    assert ($revealed_record.active == true) "Record boolean field should be preserved"
    
    # Test List
    print "  Testing List unwrap..."
    let original_list = ["item1", "item2", "item3"]
    let secret_list = $original_list | secret wrap-list
    let revealed_list = $secret_list | secret unwrap
    assert ($revealed_list == $original_list) $"List unwrap failed. Expected: ($original_list), Got: ($revealed_list)"
    let list_type = $revealed_list | describe
    print $"  List type: ($list_type)"
    assert (($list_type | str starts-with "list")) "List unwrap should preserve list type"
    assert (($revealed_list | length) == 3) "List length should be preserved"
    assert ($revealed_list.0 == "item1") "List elements should be preserved"
    
    # Test Binary
    print "  Testing Binary unwrap..."
    let original_binary = 0x[deadbeef]
    let secret_binary = $original_binary | secret wrap-binary
    let revealed_binary = $secret_binary | secret unwrap
    assert ($revealed_binary == $original_binary) $"Binary unwrap failed. Expected: ($original_binary), Got: ($revealed_binary)"
    assert (($revealed_binary | describe) == "binary") "Binary unwrap should preserve type"
    
    # Test Date
    print "  Testing Date unwrap..."
    let original_date = "2023-12-25T10:00:00Z" | into datetime
    let secret_date = $original_date | secret wrap-date
    let revealed_date = $secret_date | secret unwrap
    assert ($revealed_date == $original_date) $"Date unwrap failed. Expected: ($original_date), Got: ($revealed_date)"
    assert (($revealed_date | describe) == "datetime") "Date unwrap should preserve datetime type"
    
    print "✅ All unwrap tests passed"
}

# Test multiple secret creation
def test_multiple_secrets [] {
    print "🧪 Testing multiple secret creation..."
    
    # Test creating multiple string secrets
    let secrets = ["secret1", "secret2", "secret3"] 
        | each { |s| $s | secret wrap-string }
    
    # Verify all are secrets
    let all_secrets = $secrets 
        | all { |s| $s | secret validate }
    assert $all_secrets "All should be valid secrets"
    
    # Verify all display as redacted
    let displays = $secrets | each { |s| $s | to text }
    for display in $displays {
        assert ($display | str contains "redacted") "Each should display as redacted"
    }
    
    # Verify round-trip works for all secrets
    let originals = ["secret1", "secret2", "secret3"]
    for i in 0..2 {
        let revealed = $secrets | get $i | secret unwrap
        let expected = $originals | get $i
        assert ($revealed == $expected) $"Round-trip failed for secret ($i). Expected: ($expected), Got: ($revealed)"
    }
    
    print "✅ multiple secrets test passed"
}

# Test validation functionality
def test_validation [] {
    print "🧪 Testing validation..."
    
    # Test secret validates as true
    let secret = "test" | secret wrap-string
    let is_secret = $secret | secret validate
    assert $is_secret "Secret should validate as true"
    
    # Test regular string validates as false
    let validation_result = "not-a-secret" | secret validate
    assert (not $validation_result) "Regular string should validate as false"
    
    print "✅ validation test passed"
}

# Main test function
def main [] {
    print "🚀 Running Simple nu_plugin_secret Tests"
    print "======================================="
    
    # Setup plugin
    setup_plugin
    
    try {
        # Run tests
        test_wrap_string
        test_unwrap_all_types  
        test_multiple_secrets
        test_validation
        
        print ""
        print "🎉 All tests passed successfully!"
        
    } catch { |e|
        print $"💥 Test failed: ($e.msg)"
        cleanup_plugin
        exit 1
    }
    
    # Cleanup
    cleanup_plugin
    
    print "✅ Tests completed and cleaned up"
}