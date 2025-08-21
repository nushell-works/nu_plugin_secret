# Secret Type Selection Guide

This guide helps you choose the appropriate secret type for your sensitive data in the `nu_plugin_secret` plugin.

## Quick Reference

| Data Type | Secret Type | Use Case | Command |
|-----------|-------------|----------|---------|
| String | `SecretString` | API keys, passwords, tokens | `secret wrap-string` |
| Integer | `SecretInt` | Port numbers, IDs, counts | `secret wrap-int` |
| Boolean | `SecretBool` | Feature flags, permissions | `secret wrap-bool` |
| Record | `SecretRecord` | Credentials, config objects | `secret wrap-record` |
| List | `SecretList` | Arrays of secrets | `secret wrap-list` |
| Float | `SecretFloat` | Sensitive measurements | `secret wrap-float` |
| Binary | `SecretBinary` | Keys, certificates, hashes | `secret wrap-binary` |
| Date | `SecretDate` | Sensitive timestamps | `secret wrap-date` |

## Detailed Type Selection

### SecretString - The Most Common Choice
**Best for:**
- API keys and access tokens
- Passwords and passphrases 
- Connection strings
- Personal identifiers (SSN, credit card numbers)
- Any sensitive text data

**Example:**
```nushell
# Protect an API key
let api_key = "sk-1234567890abcdef" | secret wrap-string

# Use in HTTP headers (remains protected)
http get https://api.example.com --headers {Authorization: $"Bearer ($api_key)"}
```

**When to use:** Any time you have sensitive text data that should never appear in logs or output.

### SecretInt - Sensitive Numeric Data
**Best for:**
- Database IDs that shouldn't be exposed
- Port numbers in security contexts
- Sensitive counters or metrics
- User IDs or account numbers

**Example:**
```nushell
# Protect a database ID
let user_id = 12345 | secret wrap-int

# Use in queries (remains protected)
let query = $"SELECT * FROM users WHERE id = ($user_id)"
```

**When to use:** Integer values that are sensitive identifiers or could reveal system information.

### SecretBool - Sensitive Flags
**Best for:**
- Permission flags
- Security feature toggles  
- Audit trail flags
- Access control booleans

**Example:**
```nushell
# Protect admin status
let is_admin = true | secret wrap-bool

# Use in conditional logic
if ($is_admin | secret unwrap) { 
    echo "Admin access granted" 
}
```

**When to use:** Boolean values that indicate sensitive permissions or security states.

### SecretRecord - Structured Credentials
**Best for:**
- Complete credential objects
- Database connection configs
- API endpoint configurations
- Multi-field sensitive data

**Example:**
```nushell
# Protect entire credential structure
let db_config = {
    host: "db.internal.com",
    username: "app_user", 
    password: "secret123",
    port: 5432
} | secret wrap-record

# Access remains protected
$db_config | secret unwrap | get password
```

**When to use:** When you have multiple related sensitive fields that should be protected as a unit.

### SecretList - Arrays of Secrets
**Best for:**
- Lists of API keys
- Arrays of user tokens
- Collections of sensitive identifiers
- Backup codes or recovery keys

**Example:**
```nushell
# Protect array of backup codes
let backup_codes = ["ABC123", "DEF456", "GHI789"] | secret wrap-list

# Individual elements remain protected
$backup_codes | secret unwrap | get 0
```

**When to use:** Arrays where each element is sensitive and the collection should be protected.

### SecretFloat - Sensitive Measurements
**Best for:**
- Financial amounts
- Sensitive metrics or measurements
- Performance data that shouldn't be exposed
- Scientific measurements with privacy implications

**Example:**
```nushell
# Protect salary information
let salary = 85000.50 | secret wrap-float

# Use in calculations (data remains protected)
let bonus = ($salary | secret unwrap) * 0.1
```

**When to use:** Floating-point numbers that represent sensitive measurements or financial data.

### SecretBinary - Raw Sensitive Data
**Best for:**
- Cryptographic keys
- Certificate data
- Hash values
- Encrypted blobs
- Raw binary secrets

**Example:**
```nushell
# Protect binary key data
let cert_data = (open cert.pem | into binary) | secret wrap-binary

# Length checks remain safe
$cert_data | secret unwrap | length
```

**When to use:** Raw binary data that contains sensitive cryptographic material or encoded secrets.

### SecretDate - Sensitive Timestamps
**Best for:**
- Account creation dates (privacy)
- Expiration timestamps
- Sensitive event times
- Audit timestamps

**Example:**
```nushell
# Protect account creation date
let created_at = (date now) | secret wrap-date

# Safe date operations
$created_at | secret unwrap | date to-record | get year
```

**When to use:** Date/time values that could reveal sensitive timing information or privacy data.

## Decision Tree

```
Is your data sensitive? → Yes → Continue
                       → No → Use regular Nushell types

What type is your data?
├── Text/String → SecretString
├── Number 
│   ├── Integer → SecretInt  
│   └── Decimal → SecretFloat
├── True/False → SecretBool
├── Object/Record → SecretRecord
├── Array/List → SecretList  
├── Binary Data → SecretBinary
└── Date/Time → SecretDate
```

## Common Patterns

### API Credentials
```nushell
# Individual components
let api_key = "key123" | secret wrap-string
let api_secret = "secret456" | secret wrap-string

# Or as structured data
let api_creds = {
    key: "key123",
    secret: "secret456", 
    endpoint: "https://api.example.com"
} | secret wrap-record
```

### Database Connections
```nushell
# Complete connection info
let db_config = {
    host: "localhost",
    port: 5432,
    username: "myuser",
    password: "mypass",
    database: "mydb"
} | secret wrap-record
```

### Mixed Data Types
```nushell
# Different types for different purposes
let user_id = 12345 | secret wrap-int           # Sensitive ID
let api_token = "token123" | secret wrap-string  # Access token  
let is_premium = true | secret wrap-bool         # Feature flag
let signup_date = (date now) | secret wrap-date # Privacy timestamp
```

## Best Practices

1. **Choose the most specific type:** Use `SecretInt` for integers, not `SecretString` for stringified numbers
2. **Group related secrets:** Use `SecretRecord` for credentials that belong together  
3. **Minimize unwrapping:** Keep data in secret form as long as possible
4. **Use appropriate granularity:** Don't wrap non-sensitive fields in a record
5. **Consider the context:** Port numbers might be sensitive in security contexts but not in documentation

## Security Implications

- All secret types provide the same security guarantees
- Type choice affects usability and API clarity
- Proper type selection makes intent clear to other developers
- Wrong type choice doesn't reduce security but may cause confusion

## Migration Considerations

When migrating existing code:
1. Identify all sensitive data first
2. Choose appropriate secret types based on data types
3. Update code to handle secret types
4. Test that unwrapping works correctly
5. Verify no accidental exposure in logs or output

See [MIGRATION.md](MIGRATION.md) for detailed migration patterns.