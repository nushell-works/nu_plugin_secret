# Simple working test runner for nu_plugin_secret
# Avoids dynamic sourcing issues by running tests directly

use setup.nu *

# Simple test runner that executes our test files
def main [] {
    print "🚀 nu_plugin_secret Nushell Test Runner"
    print "======================================="
    
    # Setup plugin first
    setup_plugin
    
    let test_files = [
        "simple_test.nu"
    ]
    
    let start_time = date now
    let results = []
    
    for test_file in $test_files {
        print $"📂 Running test file: ($test_file)"
        
        try {
            # Run the test file directly
            nu $test_file
            print $"✅ ($test_file) passed"
        } catch { |e|
            print $"❌ ($test_file) failed: ($e.msg)"
        }
    }
    
    let duration = (date now) - $start_time
    
    print ""
    print $"⏱️  Total execution time: ($duration)"
    
    # Cleanup
    cleanup_plugin
    
    print "✅ Test runner completed"
}