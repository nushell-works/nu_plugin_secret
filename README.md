# nu_plugin_secret

Production-grade secret handling plugin for Nushell with secure CustomValue types that prevent accidental exposure of sensitive data.

## = Security First

This plugin provides secure custom types that always display as `<redacted>` to prevent accidental exposure of sensitive information like API keys, passwords, and tokens in logs, debug output, or command history.

## =€ Features

- **SecretString**: Secure string type with automatic redaction
- **Memory Safety**: Automatic secure cleanup with zeroization
- **Type Safety**: Clear distinction between secret and regular data
- **Pipeline Integration**: Works seamlessly with Nushell data flows
- **Security Warnings**: Built-in warnings for sensitive operations

## =æ Installation

```bash
# Build the plugin
cargo build --release

# Register with Nushell
plugin add target/release/nu_plugin_secret
plugin use secret
```

## =' Commands

### `secret wrap-string`
Convert a string to a SecretString type:
```nushell
"my-api-key" | secret wrap-string
# Output: <redacted:string>
```

### `secret unwrap`  
Extract the underlying value (with security warning):
```nushell
$secret_value | secret unwrap
# WARNING: Extracting sensitive data from secret type...
# Output: my-api-key
```

### `secret info`
Display plugin information and security guidance:
```nushell
secret info
```

### `secret validate`
Check if a value is a secret type:
```nushell
$value | secret validate
# Output: true/false
```

### `secret type-of`
Get the underlying type of a secret value:
```nushell
$secret_value | secret type-of  
# Output: string
```

## =á Security Features

- **Always Redacted**: Secret values never display actual content
- **Memory Safety**: Secure cleanup via zeroization on drop
- **Constant-Time Comparison**: Prevents timing attacks
- **Serialization Protection**: Custom implementations prevent exposure
- **Debug Safety**: Debug output never shows sensitive content

## =Ú Usage Examples

```nushell
# Secure API key handling
let api_key = $env.API_KEY | secret wrap-string
http get $"https://api.example.com/data" -H [Authorization $"Bearer ($api_key | secret unwrap)"]

# Environment variable protection  
$env.DATABASE_PASSWORD | secret wrap-string | save .secrets.nu

# Validate secret types
if ($value | secret validate) {
    print "This is a secret value"
}
```

## <¯ Roadmap

**Phase 1** (Current): SecretString with core commands  
**Phase 2**: SecretInt, SecretBool, SecretRecord, SecretList  
**Phase 3**: Advanced operations and field access  
**Phase 4**: Production hardening and security audit  

## =' Development

```bash
# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt

# Build documentation
cargo doc --open
```

## =Ü License

BSD 3-Clause License - see [LICENSE](LICENSE) for details.

## > Contributing

Contributions welcome! Please read our security guidelines before submitting PRs involving sensitive data handling.

##   Security Notice

This plugin is designed for defensive security purposes only. Always follow security best practices when handling sensitive data.