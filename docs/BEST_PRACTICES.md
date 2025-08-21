# Secret Types Best Practices

This guide provides detailed best practices for each secret type in the `nu_plugin_secret` plugin.

## General Security Principles

### 1. Minimize Unwrapping
**Do:**
```nushell
# Keep secrets wrapped as long as possible
let api_key = "secret123" | secret wrap-string
let headers = {Authorization: $"Bearer ($api_key)"}  # Still wrapped
http get https://api.example.com --headers $headers
```

**Don't:**
```nushell
# Unnecessary early unwrapping
let api_key = "secret123" | secret wrap-string | secret unwrap
let headers = {Authorization: $"Bearer ($api_key)"}  # Now exposed
```

### 2. Use Appropriate Granularity
**Do:**
```nushell
# Only wrap sensitive fields
let config = {
    api_key: ("secret123" | secret wrap-string),
    timeout: 30,              # Not sensitive
    retry_count: 3            # Not sensitive
}
```

**Don't:**
```nushell
# Wrapping entire config when only some fields are sensitive
let config = {
    api_key: "secret123",
    timeout: 30,
    retry_count: 3
} | secret wrap-record  # Unnecessary protection for timeout/retry_count
```

### 3. Clear Naming Conventions
**Do:**
```nushell
let secret_api_key = "key123" | secret wrap-string
let secret_db_password = "pass456" | secret wrap-string
let secret_user_id = 12345 | secret wrap-int
```

**Don't:**
```nushell
let key = "key123" | secret wrap-string      # Unclear that it's secret
let data = "pass456" | secret wrap-string    # Too generic
```

## SecretString Best Practices

### Use Cases and Patterns
```nushell
# API Keys - Most common use case
let github_token = $env.GITHUB_TOKEN | secret wrap-string
let api_response = http get https://api.github.com/user --headers {
    Authorization: $"token ($github_token)"
}

# Connection Strings
let db_url = "postgresql://user:pass@host:5432/db" | secret wrap-string
let connection = connect $db_url

# Personal Identifiers
let ssn = "123-45-6789" | secret wrap-string
let credit_card = "4111-1111-1111-1111" | secret wrap-string

# File Paths with Sensitive Content
let keyfile_path = "/home/user/.ssh/id_rsa" | secret wrap-string
```

### String-Specific Security
- Use for any text that shouldn't appear in logs
- Ideal for tokens, passwords, and identifiers
- Consider length - very long strings may impact performance
- Use consistent encoding (UTF-8) for international characters

### Common Mistakes
```nushell
# Don't use SecretString for non-text data
let port = "8080" | secret wrap-string  # Should be: 8080 | secret wrap-int
let is_enabled = "true" | secret wrap-string  # Should be: true | secret wrap-bool
```

## SecretInt Best Practices

### Use Cases and Patterns
```nushell
# Database IDs that shouldn't be exposed
let user_id = 12345 | secret wrap-int
let account_id = 67890 | secret wrap-int

# Port Numbers in Security Contexts
let internal_port = 8080 | secret wrap-int
let db_port = 5432 | secret wrap-int

# Sensitive Counters
let failed_login_count = 3 | secret wrap-int
let security_level = 5 | secret wrap-int

# Version Numbers for Internal APIs
let api_version = 2 | secret wrap-int
```

### Integer-Specific Considerations
- Use for sensitive numeric identifiers
- Good for port numbers in security contexts
- Consider range - stick to standard Nushell int range
- Use for counts that could reveal system information

### When to Use SecretInt vs SecretString
```nushell
# Use SecretInt for actual numbers
let user_id = 12345 | secret wrap-int          # ✓ Correct

# Don't stringify numbers unnecessarily
let user_id = "12345" | secret wrap-string     # ✗ Wrong type choice
```

## SecretBool Best Practices

### Use Cases and Patterns
```nushell
# Permission Flags
let is_admin = true | secret wrap-bool
let has_elevated_access = false | secret wrap-bool

# Security Feature Toggles
let mfa_enabled = true | secret wrap-bool
let audit_logging = true | secret wrap-bool

# Access Control States
let is_authenticated = true | secret wrap-bool
let can_delete_users = false | secret wrap-bool

# Privacy Settings
let profile_is_public = false | secret wrap-bool
```

### Boolean-Specific Security
- Use when the boolean value itself is sensitive
- Good for permission and access control flags
- Useful for feature flags that shouldn't be exposed
- Consider if the boolean reveals sensitive system state

### Usage in Conditionals
```nushell
# Safe conditional usage
let is_admin = true | secret wrap-bool
if ($is_admin | secret unwrap) {
    echo "Admin operations available"
}

# Alternative: utility functions that work with secrets
let is_admin = true | secret wrap-bool
if (secret-is-true $is_admin) {  # Hypothetical utility
    echo "Admin operations available" 
}
```

## SecretRecord Best Practices

### Use Cases and Patterns
```nushell
# Complete Credential Sets
let database_creds = {
    host: "db.internal.company.com",
    username: "app_user",
    password: "complex_password_123", 
    port: 5432,
    database: "production_db"
} | secret wrap-record

# API Configuration
let api_config = {
    base_url: "https://internal-api.company.com",
    api_key: "sk-1234567890abcdef",
    secret_key: "secret_abcdef1234567890",
    version: "v2"
} | secret wrap-record

# OAuth Credentials  
let oauth_creds = {
    client_id: "client_123456",
    client_secret: "secret_abcdef", 
    redirect_uri: "https://myapp.com/callback",
    scopes: ["read", "write"]
} | secret wrap-record
```

### Record-Specific Considerations
- Use when multiple related fields are all sensitive
- Consider partial sensitivity - mix secret and non-secret fields
- Good for structured credentials and configuration
- Enables atomic handling of related sensitive data

### Mixed Sensitivity Records
```nushell
# Better: Mix secret and non-secret fields
let api_config = {
    base_url: "https://api.example.com",        # Public
    timeout: 30,                                # Public
    api_key: ("secret123" | secret wrap-string), # Secret
    retry_count: 3                              # Public
}

# Instead of wrapping entire record
let api_config = {
    base_url: "https://api.example.com", 
    timeout: 30,
    api_key: "secret123",
    retry_count: 3
} | secret wrap-record  # All fields become secret unnecessarily
```

## SecretList Best Practices

### Use Cases and Patterns
```nushell
# Backup Codes
let backup_codes = [
    "ABC123DEF", 
    "GHI456JKL", 
    "MNO789PQR"
] | secret wrap-list

# API Key Collections
let api_keys = [
    "sk-prod-1234567890",
    "sk-staging-abcdef123", 
    "sk-dev-xyz789"
] | secret wrap-list

# User Token Arrays
let user_tokens = [
    "token_user1_abc123",
    "token_user2_def456", 
    "token_user3_ghi789"
] | secret wrap-list

# Sensitive Configuration Arrays
let allowed_ips = [
    "192.168.1.100",
    "10.0.0.5", 
    "172.16.0.10"
] | secret wrap-list
```

### List-Specific Considerations
- Use when each element is sensitive
- Good for collections of similar secret data
- Consider if the list structure itself is sensitive
- Be careful with list operations that might expose elements

### Working with SecretLists
```nushell
# Safe list operations
let secrets = ["a", "b", "c"] | secret wrap-list
let length = ($secrets | secret unwrap | length)    # Length is safe to expose
let first_secret = ($secrets | secret unwrap | get 0) | secret wrap-string  # Re-wrap individual elements
```

## SecretFloat Best Practices

### Use Cases and Patterns
```nushell
# Financial Data
let salary = 75000.50 | secret wrap-float
let bonus_percentage = 0.15 | secret wrap-float

# Sensitive Measurements
let server_load = 0.85 | secret wrap-float
let error_rate = 0.02 | secret wrap-float

# Performance Metrics
let response_time = 245.7 | secret wrap-float
let cpu_usage = 67.3 | secret wrap-float

# Scientific Data with Privacy Implications
let patient_measurement = 98.6 | secret wrap-float
```

### Float-Specific Considerations
- Use for sensitive numeric measurements
- Good for financial amounts and percentages
- Consider precision requirements
- Handle special values (NaN, infinity) appropriately

### Float Precision and Comparison
```nushell
# Be careful with float comparisons
let secret_val = 1.1 | secret wrap-float
let unwrapped = $secret_val | secret unwrap
# Use appropriate epsilon for comparisons
let is_equal = (($unwrapped - 1.1) | math abs) < 0.0001
```

## SecretBinary Best Practices

### Use Cases and Patterns
```nushell
# Cryptographic Keys
let private_key = (open private.key | into binary) | secret wrap-binary
let public_key = (open public.key | into binary) | secret wrap-binary

# Certificate Data
let ssl_cert = (open certificate.pem | into binary) | secret wrap-binary

# Hash Values
let password_hash = (echo "password123" | hash sha256 | into binary) | secret wrap-binary

# Encrypted Data Blobs
let encrypted_data = (encrypt_data $plaintext | into binary) | secret wrap-binary

# Raw Binary Secrets
let random_seed = (generate_random_bytes 32) | secret wrap-binary
```

### Binary-Specific Considerations
- Use for raw cryptographic material
- Good for certificates, keys, and hashes
- Consider binary data size for performance
- Ensure proper encoding when converting to/from text

### Binary Data Handling
```nushell
# Safe binary operations
let key_data = (open key.bin | into binary) | secret wrap-binary
let key_length = ($key_data | secret unwrap | bytes length)  # Length is safe
let is_empty = ($key_data | secret unwrap | is-empty)        # Emptiness check is safe
```

## SecretDate Best Practices

### Use Cases and Patterns
```nushell
# Account Creation Dates (Privacy)
let account_created = (date now) | secret wrap-date

# Certificate Expiration
let cert_expires = ("2024-12-31T23:59:59Z" | into datetime) | secret wrap-date

# Sensitive Event Timestamps
let last_login = (date now) | secret wrap-date
let password_changed = (date now) | secret wrap-date

# Audit Timestamps
let security_event_time = (date now) | secret wrap-date
let access_granted_time = (date now) | secret wrap-date

# Privacy-Sensitive Dates
let birth_date = ("1990-01-01T00:00:00Z" | into datetime) | secret wrap-date
```

### Date-Specific Considerations
- Use for timestamps that reveal sensitive information
- Good for privacy-related dates
- Consider timezone handling
- Be careful with date formatting that might expose data

### Safe Date Operations
```nushell
# Safe date operations that don't expose sensitive info
let secret_date = (date now) | secret wrap-date
let year = ($secret_date | secret unwrap | date to-record | get year)  # Year might be safe
let is_future = ($secret_date | secret unwrap) > (date now)           # Comparison result is safe
```

## Performance Considerations

### Memory Usage
- SecretString: Proportional to string length
- SecretInt/Bool/Float: Fixed small overhead
- SecretRecord/List: Proportional to content size
- SecretBinary: Proportional to binary data size
- SecretDate: Fixed small overhead

### Operation Performance
```nushell
# Efficient: Minimize unwrapping operations
let secrets = generate_secrets | each { |s| $s | secret wrap-string }

# Less efficient: Frequent unwrapping
let secrets = generate_secrets | each { |s| 
    let wrapped = $s | secret wrap-string
    let unwrapped = $wrapped | secret unwrap
    validate_secret $unwrapped
    $s | secret wrap-string
}
```

## Testing Secret Types

### Unit Test Patterns
```nushell
# Test secret creation and unwrapping
def test_secret_string [] {
    let original = "test_value"
    let secret = $original | secret wrap-string
    let unwrapped = $secret | secret unwrap
    assert ($unwrapped == $original)
}

# Test display protection  
def test_secret_display [] {
    let secret = "sensitive" | secret wrap-string
    let display = $secret | to text
    assert ($display == "<redacted:string>")
}
```

### Integration Test Patterns
```nushell
# Test secret types in pipelines
def test_secret_pipeline [] {
    let api_key = "key123" | secret wrap-string
    # Test that secret survives pipeline operations
    let result = [$api_key] | get 0 | secret unwrap
    assert ($result == "key123")
}
```

## Error Handling

### Common Error Scenarios
```nushell
# Handle type mismatches gracefully
def safe_unwrap_string [value] {
    if ($value | secret validate) and ($value | secret type-of) == "string" {
        $value | secret unwrap
    } else {
        error make {msg: "Expected secret string"}
    }
}

# Handle unwrapping failures
def safe_process_secret [secret] {
    try {
        let value = $secret | secret unwrap
        process_value $value
    } catch {
        echo "Failed to process secret safely"
    }
}
```

## Summary

1. **Choose the right type** for your data's actual type and sensitivity
2. **Minimize unwrapping** to maintain security
3. **Use clear naming** to indicate secret status
4. **Handle errors gracefully** when working with secrets
5. **Test thoroughly** including display and serialization protection
6. **Consider performance** implications of different secret types
7. **Follow security principles** consistently across your codebase