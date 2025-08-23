# Test Configuration System

This directory contains isolated configuration files for testing different redaction behaviors.

## Configuration Files

- `default.toml`: Standard typed brackets redaction (`<redacted:type>`)
- `simple.toml`: Simple redaction (`<redacted>`)
- `asterisks.toml`: Asterisk-based redaction (`***`)
- `brackets.toml`: Square bracket redaction (`[HIDDEN]`)
- `custom.toml`: Custom text redaction
- `partial-char.toml`: Character-based partial redaction
- `partial-hash.toml`: Hash-based partial redaction
- `paranoid.toml`: Maximum security settings
- `minimal.toml`: Minimal security settings

## Usage

### Manual Testing
```bash
# Set environment to use specific config
export XDG_CONFIG_HOME="$(pwd)/tests/configurations"
cp tests/configurations/nushell/plugins/secret/partial-char.toml \
   tests/configurations/nushell/plugins/secret/config.toml

# Run nushell with isolated config
nu
```

### Automated Testing
```bash
# Run all configuration tests
cargo test redaction_integration

# Test specific configuration
./tests/configurations/scripts/test-runner.nu partial-char
```