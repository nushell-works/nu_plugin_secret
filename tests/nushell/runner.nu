# Nushell Test Runner for nu_plugin_secret
# Comprehensive test framework with parallel execution, reporting, and CI integration

use setup.nu *

# Test runner configuration
export def get_runner_config [] {
    {
        default_timeout: 30sec,
        max_parallel_jobs: 4,
        verbose_output: false,
        stop_on_failure: false,
        test_data_file: "tests/nushell/fixtures/test_data.json",
        report_format: "detailed",  # "summary", "detailed", "json", "junit"
        output_file: null
    }
}

# Test result structure
export def create_test_result [
    name: string,
    status: string,
    duration?: duration,
    message?: string,
    details?: record
] {
    {
        name: $name,
        status: $status,  # "passed", "failed", "skipped", "error"
        duration: ($duration | default 0ms),
        message: ($message | default ""),
        details: ($details | default {}),
        timestamp: (date now)
    }
}

# Test suite structure
export def create_test_suite [
    name: string,
    tests: list,
    setup?: closure,
    teardown?: closure
] {
    {
        name: $name,
        tests: $tests,
        setup: ($setup | default { }),
        teardown: ($teardown | default { }),
        results: []
    }
}

# Discover test files in a directory
export def discover_tests [directory: string] {
    let test_files = try {
        glob $"($directory)/*.nu" | where { |file| 
            ($file | str contains "test") and not ($file | str contains "setup")
        }
    } catch {
        []
    }
    
    $test_files | each { |file|
        {
            file: $file,
            suite: ($file | path basename | str replace ".nu" ""),
            path: $file
        }
    }
}

# Run a single test function
export def run_single_test [
    test_name: string,
    test_func: closure,
    config: record
] {
    let start_time = date now
    
    if $config.verbose_output {
        print $"  🧪 Running ($test_name)..."
    }
    
    try {
        do $test_func
        let duration = (date now) - $start_time
        
        if $config.verbose_output {
            print $"  ✅ ($test_name) passed in ($duration)"
        }
        
        create_test_result $test_name "passed" $duration
        
    } catch { |e|
        let duration = (date now) - $start_time
        let error_msg = $e.msg
        
        if $config.verbose_output {
            print $"  ❌ ($test_name) failed in ($duration): ($error_msg)"
        }
        
        create_test_result $test_name "failed" $duration $error_msg { error: $e }
    }
}

# Run tests from a test file
export def run_test_file [file: string, config: record] {
    let suite_name = $file | path basename | str replace ".nu" ""
    
    if $config.verbose_output {
        print $"📂 Running test suite: ($suite_name)"
    }
    
    let start_time = date now
    
    try {
        # Instead of dynamic sourcing, we'll execute the file directly
        # and capture any test output
        let test_output = try {
            nu $file
            "success"
        } catch { |e|
            $"Test execution failed: ($e.msg)"
        }
        
        # For now, create a simple result based on execution
        let results = if ($test_output == "success") {
            [
                (create_test_result "file_execution" "passed" ((date now) - $start_time) "Test file executed successfully")
            ]
        } else {
            [
                (create_test_result "file_execution" "failed" ((date now) - $start_time) $test_output)
            ]
        }
        
        let duration = (date now) - $start_time
        let passed = $results | where status == "passed" | length
        let failed = $results | where status == "failed" | length
        let skipped = $results | where status == "skipped" | length
        
        if $config.verbose_output {
            print $"📊 Suite ($suite_name): ($passed) passed, ($failed) failed, ($skipped) skipped in ($duration)"
        }
        
        {
            suite: $suite_name,
            file: $file,
            duration: $duration,
            results: $results,
            summary: {
                total: ($results | length),
                passed: $passed,
                failed: $failed,
                skipped: $skipped
            }
        }
        
    } catch { |e|
        let duration = (date now) - $start_time
        
        if $config.verbose_output {
            print $"💥 Suite ($suite_name) failed to load: ($e.msg)"
        }
        
        {
            suite: $suite_name,
            file: $file,
            duration: $duration,
            results: [create_test_result "suite_load" "error" $duration $e.msg],
            summary: { total: 0, passed: 0, failed: 0, skipped: 0, errors: 1 }
        }
    }
}

# Run multiple test suites in parallel
export def run_test_suites [
    suites: list,
    config: record
] {
    let batch_size = $config.max_parallel_jobs
    
    # Split suites into batches for parallel processing
    let batches = $suites | group $batch_size
    
    $batches | each { |batch|
        # For now, run sequentially as Nushell parallel execution needs more setup
        $batch | each { |suite_info|
            run_test_file $suite_info.path $config
        }
    } | flatten
}

# Generate test report
export def generate_report [results: list, format: string, output_file?: string] {
    let total_suites = $results | length
    let all_results = $results | get results | flatten
    let total_tests = $all_results | length
    let passed = $all_results | where status == "passed" | length
    let failed = $all_results | where status == "failed" | length  
    let skipped = $all_results | where status == "skipped" | length
    let errors = $all_results | where status == "error" | length
    
    let summary = {
        suites: $total_suites,
        tests: $total_tests,
        passed: $passed,
        failed: $failed,
        skipped: $skipped,
        errors: $errors,
        success_rate: (if $total_tests > 0 { ($passed / $total_tests * 100) | math round --precision 1 } else { 0 })
    }
    
    match $format {
        "summary" => {
            let status_emoji = if $failed == 0 and $errors == 0 { "✅" } else { "❌" }
            
            print ""
            print $"($status_emoji) Test Summary"
            print "=================="
            print $"Suites: ($total_suites)"
            print $"Tests:  ($total_tests)"
            print $"Passed: ($passed)"
            print $"Failed: ($failed)"
            print $"Skipped: ($skipped)"
            print $"Errors: ($errors)"
            print $"Success Rate: ($summary.success_rate)%"
        }
        
        "detailed" => {
            print ""
            print "📊 Detailed Test Report"
            print "======================="
            print $"Summary: ($passed)✅ ($failed)❌ ($skipped)⏭️  ($errors)💥"
            print ""
            
            for result in $results {
                let suite_status = if $result.summary.failed > 0 or $result.summary.errors > 0 { "❌" } else { "✅" }
                print $"($suite_status) ($result.suite) - ($result.summary.passed)/($result.summary.total) passed"
                
                let failed_tests = $result.results | where status in ["failed", "error"]
                if not ($failed_tests | is-empty) {
                    for test in $failed_tests {
                        print $"  ❌ ($test.name): ($test.message)"
                    }
                }
            }
        }
        
        "json" => {
            let report = {
                timestamp: (date now),
                summary: $summary,
                results: $results
            }
            
            if $output_file != null {
                $report | to json | save $output_file
                print $"📄 JSON report saved to ($output_file)"
            } else {
                $report | to json
            }
        }
        
        _ => {
            generate_report $results "summary" $output_file
        }
    }
    
    $summary
}

# Main test runner function
export def main [
    --suite (-s): string = "all",      # Test suite to run ("all", "commands", "integration", etc.)
    --verbose (-v),                     # Verbose output
    --parallel (-p): int = 1,          # Number of parallel jobs
    --timeout (-t): duration = 30sec,  # Test timeout
    --format (-f): string = "detailed", # Report format
    --output (-o): string,             # Output file for results
    --stop-on-failure,                 # Stop on first failure
    --setup-only,                      # Only run setup, don't run tests
    --cleanup-only                     # Only run cleanup
] {
    let config = get_runner_config | merge {
        verbose_output: $verbose,
        max_parallel_jobs: $parallel,
        default_timeout: $timeout,
        report_format: $format,
        output_file: $output,
        stop_on_failure: $stop_on_failure
    }
    
    if $cleanup_only {
        cleanup_plugin
        return
    }
    
    print "🚀 nu_plugin_secret Test Runner"
    print "==============================="
    
    # Setup plugin
    setup_plugin
    
    if $setup_only {
        print "✅ Setup completed. Plugin ready for testing."
        return
    }
    
    # Discover test suites
    let available_suites = [
        "commands",
        "integration", 
        "security",
        "performance"
    ]
    
    let suites_to_run = if $suite == "all" {
        $available_suites | each { |suite_name|
            discover_tests $"tests/nushell/($suite_name)"
        } | flatten
    } else if $suite in $available_suites {
        discover_tests $"tests/nushell/($suite)"
    } else {
        print $"❌ Unknown test suite: ($suite)"
        print $"Available suites: ($available_suites | str join ', '), all"
        cleanup_plugin
        return
    }
    
    if ($suites_to_run | is-empty) {
        print "⚠️  No test files found"
        cleanup_plugin
        return
    }
    
    print $"📁 Found ($suites_to_run | length) test files"
    
    # Run tests
    let start_time = date now
    let results = run_test_suites $suites_to_run $config
    let total_duration = (date now) - $start_time
    
    # Generate report
    print ""
    let summary = generate_report $results $config.report_format $config.output_file
    
    print ""
    print $"⏱️  Total execution time: ($total_duration)"
    
    # Cleanup
    cleanup_plugin
    
    # Exit with appropriate code
    if $summary.failed > 0 or $summary.errors > 0 {
        print "❌ Tests failed"
        exit 1
    } else {
        print "✅ All tests passed"
        exit 0
    }
}

# Utility functions for test files

# Assert function for tests
export def assert [condition: bool, message?: string] {
    if not $condition {
        let error_msg = $message | default "Assertion failed"
        error make { msg: $error_msg }
    }
}

# Assert equality
export def assert_eq [actual: any, expected: any, message?: string] {
    if $actual != $expected {
        let error_msg = $message | default $"Expected ($expected), got ($actual)"
        error make { msg: $error_msg }
    }
}

# Assert that a value contains a substring
export def assert_contains [text: string, substring: string, message?: string] {
    if not ($text | str contains $substring) {
        let error_msg = $message | default $"Expected '($text)' to contain '($substring)'"
        error make { msg: $error_msg }
    }
}

# Assert that a value doesn't contain a substring  
export def assert_not_contains [text: string, substring: string, message?: string] {
    if ($text | str contains $substring) {
        let error_msg = $message | default $"Expected '($text)' to not contain '($substring)'"
        error make { msg: $error_msg }
    }
}

# Time a operation
export def time_operation [operation: closure] {
    let start = date now
    let result = do $operation
    let duration = (date now) - $start
    { result: $result, duration: $duration }
}