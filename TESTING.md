# Testing Guide for nu_plugin_secret

This document describes the comprehensive testing framework for the nu_plugin_secret plugin, including both Rust unit tests and Nushell integration tests.

## Overview

The plugin has two layers of testing:

1. **Rust Tests** (179+ tests): Unit tests, integration tests, and security validation
2. **Nushell Tests** (1,500+ lines): End-to-end integration tests in real Nushell environment

## Test Categories

### Rust Tests

```bash
# Run all Rust tests
cargo test

# Run specific test categories
cargo test --test command_integration_tests
cargo test --test security_cryptographic_review  
cargo test --test security_serialization_attacks
cargo test --test security_memory_safety
cargo test --test security_sidechannel_attacks
```

#### Test Files:
- **`tests/command_integration_tests.rs`** - Command functionality and integration
- **`tests/security_cryptographic_review.rs`** - Cryptographic security properties
- **`tests/security_serialization_attacks.rs`** - Serialization security validation  
- **`tests/security_memory_safety.rs`** - Memory safety and cleanup
- **`tests/security_sidechannel_attacks.rs`** - Timing attack resistance

### Nushell Integration Tests

```bash
# Quick test of core functionality
nu tests/nushell/simple_test.nu

# Run comprehensive test suite
./scripts/run_nu_tests.sh

# Run specific test categories
nu tests/nushell/runner.nu --suite commands
nu tests/nushell/runner.nu --suite integration
nu tests/nushell/runner.nu --suite security
```

#### Nushell Test Structure:
```
tests/nushell/
├── simple_test.nu          # Quick functionality test
├── runner.nu               # Test framework and automation
├── setup.nu                # Plugin setup and teardown  
├── commands/               # Command-specific tests
│   ├── wrap_commands.nu
│   └── unwrap_tests.nu
├── integration/            # End-to-end workflow tests
│   └── basic_operations.nu
└── fixtures/               # Test data and configuration
    ├── test_data.json
    ├── config_samples.toml
    └── secrets.nu
```

## Security Testing

### Dual-Layer Security Model Validation

The tests validate our dual-layer security approach:

1. **Display Layer**: Always redacted (`<redacted:type>`)
2. **Functional Layer**: Serialization enables unwrap operations

```bash
# Test serialization security
cargo test test_secure_defaults
cargo test test_all_secret_types_serialization_functional

# Test display/debug redaction
cargo test test_secret_string_no_leakage
cargo test test_secret_string_consistent_redaction
```

### Round-Trip Testing

All secret types are tested for proper unwrap functionality:

```bash
# Test in Nushell
nu -c '"test" | secret wrap-string | secret unwrap'
nu -c '42 | secret wrap-int | secret unwrap' 
nu -c 'true | secret wrap-bool | secret unwrap'
# ... and so on for all 8 types
```

## Performance Testing

```bash
# Performance benchmarks (excluded from Miri for speed)
cargo test --release cryptographic_performance
cargo test --release serialization_performance_tests
cargo test --release memory_safety_benchmarks
```

## Test Automation

### Continuous Integration

The GitHub Actions workflow runs:
- All Rust tests across multiple platforms
- Clippy linting and formatting checks
- Security audit with cargo-audit
- Documentation generation and validation

### Local Development

```bash
# Pre-commit checks
cargo clippy
cargo fmt --check
cargo test

# Full validation
./scripts/run_nu_tests.sh
cargo test --release
```

## Test Data and Fixtures

### Test Data Coverage

The tests cover:
- **Strings**: Empty, Unicode, special characters, very long strings
- **Numbers**: Zero, negative, maximum values, edge cases
- **Binary**: Empty, large data, non-UTF8 sequences
- **Records**: Nested structures, mixed types, edge cases
- **Lists**: Empty, heterogeneous, deeply nested
- **Dates**: Various timezones, edge dates, formatting variations

### Security Test Scenarios

- Memory safety under pressure (1000+ secrets)
- Timing attack resistance with statistical analysis
- Serialization attack resistance across all formats
- Side-channel attack simulation
- Constant-time comparison validation

## Debugging Tests

### Enable Debug Output

```bash
# Rust tests with output
cargo test -- --nocapture

# Nushell tests with verbose output  
nu tests/nushell/runner.nu --verbose

# Individual test debugging
RUST_LOG=debug cargo test test_name
```

### Common Issues

1. **Plugin Registration**: Tests automatically handle plugin setup/cleanup
2. **Binary Path**: Tests build the plugin automatically if needed
3. **Permissions**: Ensure execute permissions on `scripts/run_nu_tests.sh`

## Contributing

When adding tests:

1. **Rust Tests**: Add to appropriate `tests/*.rs` file
2. **Nushell Tests**: Add to `tests/nushell/` with proper setup
3. **Security Tests**: Any new functionality must include security validation
4. **Documentation**: Update this guide for new test categories

## Test Results Interpretation

### Passing Criteria

- All 179+ Rust tests pass
- All Nushell integration tests pass
- No clippy warnings
- No memory leaks detected
- Security properties validated

### Expected Behavior

- **Display/Debug**: Always shows `<redacted:type>`
- **Unwrap**: Returns original values correctly
- **Serialization**: Enables functional operations while maintaining display security
- **Memory**: Secure cleanup with ZeroizeOnDrop
- **Performance**: Constant-time operations for security-sensitive functions