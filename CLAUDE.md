# Claude AI Guidelines for nu_plugin_secret

This document contains specific guidelines for Claude when working on this codebase to avoid common mistakes and maintain code quality.

## Code Quality Guidelines

### Mathematical Constants
- **NEVER** use hardcoded mathematical constants like `3.14`, `2.718`, etc. in test code
- **ALWAYS** use standard library constants instead:
  - Use `std::f64::consts::PI` instead of `3.14` or `3.14159`
  - Use `std::f64::consts::E` instead of `2.718` or `2.71828`
  - Use `std::f64::consts::TAU` instead of `6.28` (2π)
  - This prevents clippy warnings about approximate constants

### Testing Guidelines
- When writing tests that need mathematical values, prefer standard constants
- For non-mathematical test values, use clear, descriptive values like `42` or `"test_value"`
- Avoid magic numbers that could trigger clippy warnings

### Linting and Formatting
- Always run `cargo clippy --all-targets --all-features -- -D warnings` before completing tasks
- Fix all clippy warnings, especially `approx_constant` warnings  
- Add `#[allow(dead_code)]` for genuinely unused helper functions in tests
- Run `cargo fmt` to ensure consistent code formatting
- The project has a pre-commit hook that automatically runs `cargo fmt --check` and `cargo clippy`

## Project-Specific Guidelines

### Security Focus
- This is a security-focused plugin for handling secrets
- All code changes should maintain or improve security posture
- Never introduce code that could leak sensitive information

### Test Coverage
- Maintain comprehensive test coverage for all secret types
- Include edge cases and security-related test scenarios
- Use Miri-compatible code where possible (avoid system time in tests under Miri)

### Commands
To run linting, formatting, and testing:
```bash
cargo fmt                                                    # Format code
cargo clippy --all-targets --all-features -- -D warnings    # Check for warnings
cargo test                                                   # Run tests
cargo +nightly miri test  # with MIRIFLAGS=-Zmiri-disable-isolation if needed
```

### Pre-commit Hook
The project includes a pre-commit hook at `.git/hooks/pre-commit` that automatically:
- Runs `cargo fmt --check` to ensure code is formatted
- Runs `cargo clippy --all-targets --all-features -- -D warnings` to check for warnings
- Prevents commits if either check fails

To bypass the hook in emergency situations, use `git commit --no-verify`, but this should be avoided.