# Project Plan: Test Configuration System for Redaction Types

## Overview
Create a structured test configuration system that allows testing all redaction modes in isolation, ensuring configurations are discoverable by both Nushell and the plugin through standard XDG configuration paths.

## Directory Structure

```
tests/
├── configurations/
│   ├── README.md                          # Documentation for test configurations
│   ├── nushell/
│   │   └── plugins/
│   │       └── secret/
│   │           ├── default.toml           # Default redaction (typed brackets)
│   │           ├── simple.toml            # Simple redaction style
│   │           ├── asterisks.toml         # Asterisks redaction style
│   │           ├── brackets.toml          # Square brackets style
│   │           ├── custom.toml            # Custom text redaction
│   │           ├── partial-char.toml      # Character-based partial redaction
│   │           ├── partial-hash.toml      # Hash-based partial redaction
│   │           ├── paranoid.toml          # Paranoid security level
│   │           └── minimal.toml           # Minimal security level
│   └── scripts/
│       ├── test-runner.nu                 # Nushell test runner script
│       ├── config-validator.nu            # Validate configurations
│       └── redaction-tester.nu            # Test redaction behaviors
├── redaction_integration/
│   ├── mod.rs                             # Integration test module
│   ├── test_default_redaction.rs          # Test default behavior
│   ├── test_partial_redaction.rs          # Test partial redaction
│   ├── test_security_levels.rs            # Test security configurations
│   └── test_custom_styles.rs              # Test custom redaction styles
└── helpers/
    ├── config_isolation.rs                # Helper for config isolation
    ├── nushell_runner.rs                  # Helper for running Nushell with custom configs
    └── redaction_assertions.rs            # Common test assertions
```

## Phase 1: Configuration Files Creation

### 1.1 Base Configuration Structure
Each configuration file follows this structure:
```toml
version = "1.0"

[redaction]
style = "typed_brackets"  # or "simple", "asterisks", "brackets", "custom"
custom_text = null        # only when style = "custom"
show_type_info = true
preserve_length = false

[redaction.partial]
enabled = false
show_first = 4
show_last = 4
min_length = 12
max_reveal = 8
use_hash = false
hash_salt = "test_salt_unique_per_config"

[security]
level = "standard"        # "minimal", "standard", "paranoid"
audit_config_changes = true
max_custom_text_length = 50
allow_partial_redaction = false
min_partial_redaction_length = 16
```

### 1.2 Specific Configuration Files

**tests/configurations/nushell/plugins/secret/default.toml**
```toml
# Standard typed brackets redaction
version = "1.0"

[redaction]
style = "typed_brackets"
show_type_info = true
preserve_length = false

[redaction.partial]
enabled = false

[security]
level = "standard"
allow_partial_redaction = false
```

**tests/configurations/nushell/plugins/secret/partial-char.toml**
```toml
# Character-based partial redaction
version = "1.0"

[redaction]
style = "typed_brackets"
show_type_info = true
preserve_length = false

[redaction.partial]
enabled = true
show_first = 3
show_last = 3
min_length = 10
max_reveal = 6
use_hash = false
hash_salt = "test_char_partial_salt"

[security]
level = "standard"
allow_partial_redaction = true
min_partial_redaction_length = 10
```

**tests/configurations/nushell/plugins/secret/partial-hash.toml**
```toml
# Hash-based partial redaction
version = "1.0"

[redaction]
style = "typed_brackets"
show_type_info = true
preserve_length = false

[redaction.partial]
enabled = true
show_first = 4
show_last = 4
min_length = 16
max_reveal = 8
use_hash = true
hash_salt = "test_hash_partial_salt_unique"

[security]
level = "standard"
allow_partial_redaction = true
min_partial_redaction_length = 16
```

## Phase 2: Test Infrastructure

### 2.1 Configuration Isolation Helper
**tests/helpers/config_isolation.rs**
```rust
use std::env;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct ConfigIsolation {
    temp_dir: TempDir,
    original_xdg_config: Option<String>,
    original_xdg_data: Option<String>,
}

impl ConfigIsolation {
    pub fn new(config_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let config_path = PathBuf::from("tests/configurations");

        // Copy specific config to temp directory structure
        let target_dir = temp_dir.path().join("nushell/plugins/secret");
        std::fs::create_dir_all(&target_dir)?;

        let source_config = config_path.join("nushell/plugins/secret").join(format!("{}.toml", config_name));
        let target_config = target_dir.join("config.toml");
        std::fs::copy(&source_config, &target_config)?;

        // Store original environment
        let original_xdg_config = env::var("XDG_CONFIG_HOME").ok();
        let original_xdg_data = env::var("XDG_DATA_HOME").ok();

        // Set test environment
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        env::set_var("XDG_DATA_HOME", temp_dir.path());

        Ok(Self {
            temp_dir,
            original_xdg_config,
            original_xdg_data,
        })
    }

    pub fn config_path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }
}

impl Drop for ConfigIsolation {
    fn drop(&mut self) {
        // Restore original environment
        match &self.original_xdg_config {
            Some(val) => env::set_var("XDG_CONFIG_HOME", val),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
        match &self.original_xdg_data {
            Some(val) => env::set_var("XDG_DATA_HOME", val),
            None => env::remove_var("XDG_DATA_HOME"),
        }
    }
}
```

### 2.2 Nushell Runner Helper
**tests/helpers/nushell_runner.rs**
```rust
use std::process::Command;
use std::path::PathBuf;

pub struct NushellRunner {
    config_dir: PathBuf,
}

impl NushellRunner {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    pub fn run_script(&self, script: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("nu")
            .env("XDG_CONFIG_HOME", &self.config_dir)
            .env("XDG_DATA_HOME", &self.config_dir)
            .arg("-c")
            .arg(script)
            .output()?;

        if !output.status.success() {
            return Err(format!("Nushell script failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn test_redaction(&self, secret_value: &str) -> Result<String, Box<dyn std::error::Error>> {
        let script = format!(
            r#"
            use {}
            "{}" | secret wrap-string | print
            "#,
            self.plugin_path()?,
            secret_value
        );

        self.run_script(&script)
    }

    fn plugin_path(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Return path to built plugin
        Ok("target/debug/nu_plugin_secret".to_string())
    }
}
```

## Phase 3: Integration Tests

### 3.1 Per-Configuration Integration Tests
**tests/redaction_integration/test_partial_redaction.rs**
```rust
use crate::helpers::{ConfigIsolation, NushellRunner};

#[test]
fn test_character_based_partial_redaction() {
    let isolation = ConfigIsolation::new("partial-char").unwrap();
    let runner = NushellRunner::new(isolation.config_path());

    let test_secret = "verylongsecretkey123";
    let result = runner.test_redaction(test_secret).unwrap();

    // Should show partial: "ver**********...123"
    assert!(result.contains("ver"));
    assert!(result.contains("123"));
    assert!(result.contains("*"));
    assert!(result.contains("..."));
    assert!(!result.contains("longsecretkey"));
}

#[test]
fn test_hash_based_partial_redaction() {
    let isolation = ConfigIsolation::new("partial-hash").unwrap();
    let runner = NushellRunner::new(isolation.config_path());

    let test_secret = "anotherlongsecretvalue";
    let result = runner.test_redaction(test_secret).unwrap();

    // Should show hash-based partial: "a1b2c3d4...22"
    assert!(result.contains("..."));
    assert!(!result.contains("longsecret"));
    // Hash should be consistent
    let result2 = runner.test_redaction(test_secret).unwrap();
    assert_eq!(result, result2);
}
```

### 3.2 Configuration Validation Tests
**tests/redaction_integration/test_security_levels.rs**
```rust
#[test]
fn test_paranoid_security_blocks_partial_redaction() {
    let isolation = ConfigIsolation::new("paranoid").unwrap();
    let runner = NushellRunner::new(isolation.config_path());

    let test_secret = "shouldnotshowpartial";
    let result = runner.test_redaction(test_secret).unwrap();

    // Paranoid mode should never show partial redaction
    assert!(result.contains("redacted") || result.contains("SECRET"));
    assert!(!result.contains("*"));
    assert!(!result.contains("..."));
    assert!(!result.contains("should"));
}
```

## Phase 4: Nushell Test Scripts

### 4.1 Test Runner Script
**tests/configurations/scripts/test-runner.nu**
```nu
#!/usr/bin/env nu

# Test runner for different redaction configurations
def main [config_name?: string] {
    let configs = if ($config_name | is-empty) {
        ["default", "simple", "asterisks", "brackets", "custom", "partial-char", "partial-hash", "paranoid", "minimal"]
    } else {
        [$config_name]
    }

    for config in $configs {
        print $"Testing configuration: ($config)"
        test-config $config
        print ""
    }
}

def test-config [config: string] {
    # Set up environment
    let config_dir = $"($env.PWD)/tests/configurations"
    with-env [XDG_CONFIG_HOME $config_dir XDG_DATA_HOME $config_dir] {
        # Copy specific config to expected location
        let source = $"($config_dir)/nushell/plugins/secret/($config).toml"
        let target = $"($config_dir)/nushell/plugins/secret/config.toml"
        cp $source $target

        # Test various secret values
        test-redaction-behavior $config
    }
}

def test-redaction-behavior [config: string] {
    let test_values = [
        "short",
        "mediumlength",
        "verylongsecretkeyvalue",
        "🔐 unicode test 中文",
        "special!@#$%^&*()chars"
    ]

    for value in $test_values {
        let result = ($value | secret wrap-string | to text)
        print $"  Input: '($value | str substring 0..10)...'"
        print $"  Output: '($result)'"
    }
}
```

### 4.2 Configuration Validator
**tests/configurations/scripts/config-validator.nu**
```nu
#!/usr/bin/env nu

# Validate all test configurations
def main [] {
    let config_dir = "tests/configurations/nushell/plugins/secret"
    let configs = (ls $config_dir | where type == file | where name =~ "\.toml$" | get name)

    for config in $configs {
        print $"Validating ($config)..."
        validate-config $config
    }
}

def validate-config [config_path: string] {
    try {
        let content = (open $config_path)

        # Check required fields
        if not ("version" in $content) {
            error make {msg: $"Missing version in ($config_path)"}
        }

        if not ("redaction" in $content) {
            error make {msg: $"Missing redaction section in ($config_path)"}
        }

        if not ("security" in $content) {
            error make {msg: $"Missing security section in ($config_path)"}
        }

        print $"  ✓ Valid configuration"
    } catch {
        print $"  ✗ Invalid configuration: ($config_path)"
    }
}
```

## Phase 5: Integration with Existing Test Suite

### 5.1 Test Isolation Macro
Create a macro for easy test isolation:

```rust
macro_rules! test_with_config {
    ($config:expr, $test_name:ident, $body:block) => {
        #[test]
        fn $test_name() {
            let _isolation = crate::helpers::ConfigIsolation::new($config).unwrap();
            // Reset global config state
            crate::config::reset_global_config().unwrap();
            $body
        }
    };
}
```

### 5.2 Modified Integration Tests
Update existing integration tests to use isolated configurations:

```rust
test_with_config!("default", test_secret_string_default_redaction, {
    let secret = SecretString::new("test-value".to_string());
    let display = format!("{}", secret);
    assert!(display.contains("<redacted:string>"));
});

test_with_config!("partial-char", test_secret_string_partial_redaction, {
    let secret = SecretString::new("verylongsecretvalue".to_string());
    let display = format!("{}", secret);
    assert!(display.contains("ver"));
    assert!(display.contains("..."));
    assert!(display.contains("*"));
});
```

## Phase 6: Documentation and README

### 6.1 Test Configuration README
**tests/configurations/README.md**
```markdown
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
```

## Success Criteria

1. **Isolation**: Each test runs with its own configuration without affecting others
2. **Discoverable**: Configurations follow standard XDG paths that Nushell recognizes
3. **Comprehensive**: All redaction types and security levels are covered
4. **Maintainable**: Clear structure and documentation
5. **CI/CD Ready**: Tests can run reliably in automated environments

## Implementation Timeline

- **Week 1**: Phase 1-2 (Configuration files and infrastructure)
- **Week 2**: Phase 3-4 (Integration tests and Nushell scripts)
- **Week 3**: Phase 5-6 (Integration and documentation)

This approach provides complete test isolation while maintaining compatibility with Nushell's configuration discovery system.

## ✅ Implementation Status: Phase 1 Complete

### Configuration Files Created

All test configuration files have been implemented in the XDG-compliant directory structure:

```
tests/configurations/nushell/plugins/secret/
├── asterisks.toml      # Asterisk-based redaction (***)
├── brackets.toml       # Square bracket redaction ([HIDDEN])
├── custom.toml         # Custom text redaction ([SECRET_DATA])
├── default.toml        # Standard typed brackets (<redacted:type>)
├── minimal.toml        # Minimal security with partial redaction enabled
├── paranoid.toml       # Maximum security, no partial redaction
├── partial-char.toml   # Character-based partial redaction
├── partial-hash.toml   # Hash-based partial redaction
├── simple.toml         # Simple redaction (<redacted>)
└── README.md           # Documentation
```

### Idiomatic Nushell Configuration Access

The following patterns have been verified as the idiomatic way to access configuration files from within Nushell scripts:

#### 1. Environment Variable Setup
```bash
# Set XDG_CONFIG_HOME to point to test configurations
XDG_CONFIG_HOME="$(pwd)/tests/configurations" nu -c 'your_script'
```

#### 2. Idiomatic Path Construction Function
```nu
def get-secret-config-dir [] {
    let config_base = ($env.XDG_CONFIG_HOME? | default ($nu.config-path | path dirname))
    $config_base | path join "nushell" "plugins" "secret"
}
```

#### 3. Configuration Switching Function
```nu
def use-secret-config [config_name: string] {
    let config_dir = ($env.XDG_CONFIG_HOME? | default ($nu.config-path | path dirname)) | path join "nushell" "plugins" "secret"
    let source_config = $config_dir | path join $"($config_name).toml"  
    let target_config = $config_dir | path join "config.toml"
    
    if ($source_config | path exists) {
        cp $source_config $target_config
        print $"✓ Activated ($config_name) configuration"
        open $target_config
    } else {
        print $"✗ Configuration ($config_name) not found"
        let available = (ls $config_dir | get name | path basename | str replace ".toml" "")
        print $"Available configurations: ($available)"
    }
}
```

#### 4. Usage Examples
```bash
# List available configurations
XDG_CONFIG_HOME="$(pwd)/tests/configurations" nu -c '
    let config_dir = ($env.XDG_CONFIG_HOME + "/nushell/plugins/secret")
    ls $config_dir | get name | path basename | str replace ".toml" ""
'

# Switch to partial-char configuration  
XDG_CONFIG_HOME="$(pwd)/tests/configurations" nu -c 'use-secret-config "partial-char"'

# Test plugin with specific configuration
XDG_CONFIG_HOME="$(pwd)/tests/configurations" nu -c '"test-secret" | secret wrap-string'
```

### Nushell Idioms Used

This implementation follows Nushell best practices by:

- **Safe Environment Access**: Using `$env.XDG_CONFIG_HOME?` with optional chaining
- **Default Fallbacks**: Using `| default` for graceful fallback to `$nu.config-path`
- **Cross-Platform Paths**: Using `path join` instead of string concatenation
- **Existence Checking**: Using `path exists` for robust file operations
- **Structured Output**: Leveraging Nushell's table-based output for configuration lists
- **Error Handling**: Providing informative error messages with available options
- **Built-in Integration**: Using Nushell's `$nu.config-path` for default config location

### Verification Results

- ✅ All 9 configuration files created successfully
- ✅ XDG configuration path discovery working correctly
- ✅ Configuration switching mechanism functional
- ✅ Cross-platform path handling verified
- ✅ Error handling for missing configurations implemented
- ✅ Integration with Nushell's native configuration system confirmed