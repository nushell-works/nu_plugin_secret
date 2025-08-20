# nu_plugin_secret

Production-grade Nushell plugin for secure handling of sensitive data with 8 comprehensive secret types that prevent accidental exposure.

## 🔐 Security First

This plugin provides secure custom types that always display as `<redacted:type>` to prevent accidental exposure of sensitive information like API keys, passwords, tokens, and other confidential data in logs, debug output, or command history.

## ✨ Features

- **8 Secret Types**: Complete coverage of Nushell's core data types
  - `SecretString` - API keys, passwords, tokens
  - `SecretInt` - sensitive numbers, IDs, ports
  - `SecretBool` - sensitive flags, permissions
  - `SecretRecord` - configuration objects, credentials
  - `SecretList` - arrays of sensitive data
  - `SecretFloat` - financial data, coordinates, measurements
  - `SecretBinary` - certificates, keys, encrypted data
  - `SecretDate` - timestamps, birthdates, sensitive dates

- **Memory Safety**: Automatic secure cleanup with ZeroizeOnDrop
- **Timing Attack Protection**: Constant-time equality comparison
- **Type Safety**: Clear distinction between secret and regular data
- **Pipeline Integration**: Works seamlessly with Nushell data flows
- **Security Warnings**: Built-in warnings for sensitive operations

## 📦 Installation

```bash
# Build the plugin
cargo build --release

# Register with Nushell
plugin add target/release/nu_plugin_secret
plugin use secret
```

## 🚀 Commands

### Wrap Commands
Convert values to secret types:
```nushell
"my-api-key" | secret wrap-string     # <redacted:string>
42 | secret wrap-int                  # <redacted:int>
true | secret wrap-bool               # <redacted:bool>
{key: "value"} | secret wrap-record   # <redacted:record>
["item1", "item2"] | secret wrap-list # <redacted:list>
3.14159 | secret wrap-float           # <redacted:float>
0x[deadbeef] | secret wrap-binary     # <redacted:binary>
date now | secret wrap-date           # <redacted:date>
```

### Utility Commands

#### `secret unwrap`  
Extract the underlying value (with security warning):
```nushell
$secret_value | secret unwrap
# WARNING: Extracting sensitive data from secret type...
# Output: original value
```

#### `secret validate`
Check if a value is a secret type:
```nushell
$value | secret validate
# Output: true/false
```

#### `secret type-of`
Get the underlying type without exposing content:
```nushell
$secret_value | secret type-of  
# Output: string, int, bool, record, list, float, binary, or date
```

#### `secret info`
Display plugin information and security guidance:
```nushell
secret info
```

## 🛡️ Security Features

- **Zero Accidental Exposure**: Secret values never display actual content in any context
- **Memory Safety**: Secure cleanup via ZeroizeOnDrop on all secret types
- **Constant-Time Comparison**: Prevents timing attacks across all types
- **Serialization Protection**: Custom implementations prevent exposure
- **Debug Safety**: Debug output never shows sensitive content
- **Type Safety**: Comprehensive validation and error handling

## 💡 Usage Examples

```nushell
# Secure API key handling
let $api_key = ($env.API_KEY | secret wrap-string)
http get "https://example.com/api" -H [Authorization $"Bearer ($api_key | secret unwrap)"]

# Database configuration with mixed types
let $db_config = {
  host: "localhost",
  port: (5432 | secret wrap-int),
  password: ($env.DB_PASSWORD | secret wrap-string),
  ssl: (true | secret wrap-bool)
}

# Financial data protection
let $balance = (1234.56 | secret wrap-float)
let $account_id = (9876543210 | secret wrap-int)

# Binary data (certificates, keys)
open cert.pem | secret wrap-binary

# Sensitive timestamps
date now | secret wrap-date

# Validate and process secrets
if ($value | secret validate) {
    let $type = ($value | secret type-of)
    print $"Processing secret ($type)"
}
```

## 🎯 Current Status

**✅ Phase 2+ Complete**: All 8 secret types implemented and tested
- **12 Commands**: 8 wrap commands + 4 utility commands  
- **74 Tests**: Comprehensive test coverage
- **Production Ready**: Memory-safe, secure, and performant

## 🗺️ Roadmap

**✅ Phase 1**: SecretString with core commands  
**✅ Phase 2**: SecretInt, SecretBool, SecretRecord, SecretList  
**✅ Phase 2+**: SecretFloat, SecretBinary, SecretDate  
**🔄 Phase 3**: CI/CD pipeline and documentation  
**📋 Phase 4**: Security audit and production hardening  

## 🛠️ Development

```bash
# Run all tests (74 tests)
cargo test

# Check code quality
cargo clippy
cargo fmt

# Build documentation
cargo doc --open

# Performance testing
cargo test --release
```

## 📄 License

BSD 3-Clause License - see [LICENSE](LICENSE) for details.

## 🤝 Contributing

Contributions welcome! Please read our security guidelines before submitting PRs involving sensitive data handling.

## ⚠️ Security Notice

This plugin is designed for defensive security purposes only. Always follow security best practices when handling sensitive data. All secret types use memory-safe implementations with automatic cleanup to prevent information leakage.