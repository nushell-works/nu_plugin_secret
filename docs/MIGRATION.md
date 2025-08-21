# Migration Guide: From Plain Types to Secret Types

This guide helps you migrate from using plain Nushell types to secure `nu_plugin_secret` types for handling sensitive data.

## 🔐 Why Migrate?

Plain Nushell types can accidentally expose sensitive data through:
- Debug output and logging
- Serialization to JSON/YAML/etc.
- Copy/paste operations
- Memory dumps

Secret types prevent these exposures while maintaining full functionality.

## 📋 Migration Checklist

### Before You Start
- [ ] Install `nu_plugin_secret`: `cargo install nw-nu_plugin_secret`
- [ ] Register the plugin: `plugin add ~/.cargo/bin/nu_plugin_secret`
- [ ] Activate the plugin: `plugin use secret`
- [ ] Verify installation: `secret info`

## 🔄 Type-by-Type Migration

### 1. Strings → SecretString

**Before:**
```nushell
let api_key = "sk-1234567890abcdef"
echo $api_key  # ❌ Exposes secret
```

**After:**
```nushell
let api_key = "sk-1234567890abcdef" | secret wrap-string
echo $api_key  # ✅ Shows <redacted:string>
```

**Common Use Cases:**
- API keys and tokens
- Passwords and passphrases
- Connection strings
- Private keys

### 2. Integers → SecretInt

**Before:**
```nushell
let user_id = 12345
echo $user_id  # ❌ May expose sensitive ID
```

**After:**
```nushell
let user_id = 12345 | secret wrap-int
echo $user_id  # ✅ Shows <redacted:int>
```

**Common Use Cases:**
- User IDs and account numbers
- Port numbers for internal services
- Sensitive numeric codes
- Database IDs

### 3. Booleans → SecretBool

**Before:**
```nushell
let is_admin = true
echo $is_admin  # ❌ May expose privilege level
```

**After:**
```nushell
let is_admin = true | secret wrap-bool
echo $is_admin  # ✅ Shows <redacted:bool>
```

**Common Use Cases:**
- Permission flags
- Feature toggles
- Security settings
- Access control flags

### 4. Floats → SecretFloat

**Before:**
```nushell
let latitude = 37.7749
echo $latitude  # ❌ Exposes location data
```

**After:**
```nushell
let latitude = 37.7749 | secret wrap-float
echo $latitude  # ✅ Shows <redacted:float>
```

**Common Use Cases:**
- GPS coordinates
- Financial amounts
- Sensitive measurements
- Performance metrics

### 5. Records → SecretRecord

**Before:**
```nushell
let credentials = {
  username: "admin",
  password: "secret123",
  server: "prod.example.com"
}
echo $credentials  # ❌ Exposes all sensitive data
```

**After:**
```nushell
let credentials = {
  username: "admin",
  password: "secret123",
  server: "prod.example.com"
} | secret wrap-record
echo $credentials  # ✅ Shows <redacted:record>
```

**Common Use Cases:**
- Configuration objects
- User profiles
- Connection details
- Authentication data

### 6. Lists → SecretList

**Before:**
```nushell
let api_keys = ["key1", "key2", "key3"]
echo $api_keys  # ❌ Exposes all keys
```

**After:**
```nushell
let api_keys = ["key1", "key2", "key3"] | secret wrap-list
echo $api_keys  # ✅ Shows <redacted:list>
```

**Common Use Cases:**
- Multiple API keys
- User lists
- Permission arrays
- Sensitive collections

### 7. Binary Data → SecretBinary

**Before:**
```nushell
let key_data = 0x[deadbeef1234567890abcdef]
echo $key_data  # ❌ Exposes binary key
```

**After:**
```nushell
let key_data = 0x[deadbeef1234567890abcdef] | secret wrap-binary
echo $key_data  # ✅ Shows <redacted:binary>
```

**Common Use Cases:**
- Cryptographic keys
- Binary tokens
- Encrypted data
- Certificate data

### 8. Dates → SecretDate

**Before:**
```nushell
let birth_date = "2000-01-01" | into datetime
echo $birth_date  # ❌ Exposes personal information
```

**After:**
```nushell
let birth_date = "2000-01-01" | into datetime | secret wrap-date
echo $birth_date  # ✅ Shows <redacted:date>
```

**Common Use Cases:**
- Birth dates
- Event timestamps
- Expiration dates
- Personal milestones

## 🔧 Working with Secret Types

### Extracting Values (Use Sparingly)

```nushell
# Only when absolutely necessary
let plain_value = $secret_value | secret unwrap
# ⚠️  Logs security warning
```

### Type Checking

```nushell
# Check if value is a secret type
$value | secret validate

# Get the underlying type
$secret_value | secret type-of
```

### Pipeline Integration

```nushell
# Secret types work in pipelines
"sensitive-data" 
| secret wrap-string 
| secret type-of  # Returns "string"
```

## 🏗️ Migration Patterns

### Pattern 1: Configuration Files

**Before:**
```nushell
let config = {
  database_url: "postgres://user:pass@localhost/db",
  api_key: "sk-1234567890",
  debug: true
}
```

**After:**
```nushell
let config = {
  database_url: "postgres://user:pass@localhost/db" | secret wrap-string,
  api_key: "sk-1234567890" | secret wrap-string,
  debug: true | secret wrap-bool
} | secret wrap-record
```

### Pattern 2: Environment Variables

**Before:**
```nushell
let env_vars = {
  API_KEY: ($env.API_KEY),
  DB_PASSWORD: ($env.DB_PASSWORD)
}
```

**After:**
```nushell
let env_vars = {
  API_KEY: ($env.API_KEY | secret wrap-string),
  DB_PASSWORD: ($env.DB_PASSWORD | secret wrap-string)
} | secret wrap-record
```

### Pattern 3: User Input

**Before:**
```nushell
let password = (input "Enter password: ")
echo $"Password entered: {$password}"  # ❌ Exposes password
```

**After:**
```nushell
let password = (input "Enter password: ") | secret wrap-string
echo $"Password entered: {$password}"  # ✅ Shows <redacted:string>
```

### Pattern 4: Mixed Sensitivity Data

**Before:**
```nushell
let server_config = {
  hostname: "api.example.com",     # Public
  port: 443,                       # Public  
  api_key: "sk-secret123",         # Secret
  timeout: 30,                     # Public
  admin_password: "admin123"       # Secret
}
```

**After:**
```nushell
let server_config = {
  hostname: "api.example.com",                        # Keep public
  port: 443,                                          # Keep public
  api_key: ("sk-secret123" | secret wrap-string),    # Wrap secret
  timeout: 30,                                        # Keep public
  admin_password: ("admin123" | secret wrap-string)  # Wrap secret
}
# Don't wrap entire record - only sensitive fields
```

### Pattern 5: Database Credentials

**Before:**
```nushell
let db_creds = {
  host: "db.internal.com",
  port: 5432,
  username: "app_user", 
  password: "db_password_123",
  database: "production",
  ssl: true
}

# Connect using plain credentials (exposed in memory/logs)
let connection = db connect $db_creds
```

**After:**
```nushell
let db_creds = {
  host: "db.internal.com",                               # Public
  port: (5432 | secret wrap-int),                        # May be sensitive
  username: ("app_user" | secret wrap-string),           # Sensitive
  password: ("db_password_123" | secret wrap-string),    # Secret
  database: "production",                                 # Public
  ssl: true                                              # Public
}

# Connection function handles secret unwrapping internally
let connection = db connect_secure $db_creds
```

### Pattern 6: API Client Migration

**Before:**
```nushell
def call_api [endpoint: string, api_key: string] {
  http get $endpoint --headers {
    Authorization: $"Bearer ($api_key)"
  }
}

let key = "sk-1234567890"
call_api "https://api.example.com/data" $key  # Key exposed in call
```

**After:**
```nushell
def call_api [endpoint: string, api_key: any] {
  # Function expects secret type
  let key = if ($api_key | secret validate) {
    $api_key | secret unwrap
  } else {
    error make {msg: "API key must be a secret type"}
  }
  
  http get $endpoint --headers {
    Authorization: $"Bearer ($key)"
  }
}

let key = "sk-1234567890" | secret wrap-string
call_api "https://api.example.com/data" $key  # Key remains protected
```

### Pattern 7: Bulk Data Processing

**Before:**
```nushell
# Processing list of sensitive user data
let users = [
  {id: 123, name: "Alice", ssn: "123-45-6789"},
  {id: 456, name: "Bob", ssn: "987-65-4321"}
]

$users | each { |user|
  echo $"Processing user: ($user.name), SSN: ($user.ssn)"  # ❌ Exposes SSN
}
```

**After:**
```nushell
# Wrap sensitive fields during processing
let users = [
  {id: 123, name: "Alice", ssn: ("123-45-6789" | secret wrap-string)},
  {id: 456, name: "Bob", ssn: ("987-65-4321" | secret wrap-string)}
]

$users | each { |user|
  echo $"Processing user: ($user.name), SSN: ($user.ssn)"  # ✅ Shows <redacted:string>
  # Only unwrap when absolutely necessary for external systems
  process_user_external ($user.ssn | secret unwrap)
}
```

### Pattern 8: File I/O with Secrets

**Before:**
```nushell
# Reading sensitive config from file
let config = open config.json | from json
echo $"API key: ($config.api_key)"  # ❌ May expose in logs

# Writing sensitive data
{api_key: "secret123"} | to json | save output.json  # ❌ Exposes in file
```

**After:**
```nushell
# Reading and immediately protecting
let config = open config.json | from json
let secure_config = {
  api_key: ($config.api_key | secret wrap-string),
  other_field: $config.other_field
}
echo $"API key: ($secure_config.api_key)"  # ✅ Shows <redacted:string>

# Only save non-sensitive representations
let safe_config = {api_key: "<redacted>", other_field: $config.other_field}
$safe_config | to json | save output.json  # ✅ No secrets in file
```

### Pattern 9: Function Parameter Migration

**Before:**
```nushell
def deploy_app [
  app_name: string,
  api_key: string,        # Plain string parameter
  database_url: string,   # Plain string parameter
  port: int              # Plain int parameter
] {
  echo $"Deploying ($app_name) with key: ($api_key)"  # ❌ Exposes key
  # ... deployment logic
}

deploy_app "myapp" "sk-secret123" "postgres://user:pass@host/db" 8080
```

**After:**
```nushell
def deploy_app [
  app_name: string,
  api_key: any,          # Accept secret type
  database_url: any,     # Accept secret type  
  port: any             # Accept secret type
] {
  # Validate secret types
  if not ($api_key | secret validate) {
    error make {msg: "api_key must be a secret type"}
  }
  if not ($database_url | secret validate) {
    error make {msg: "database_url must be a secret type"}
  }
  if not ($port | secret validate) {
    error make {msg: "port must be a secret type"}
  }
  
  echo $"Deploying ($app_name) with key: ($api_key)"  # ✅ Shows <redacted:string>
  
  # Only unwrap for actual use
  let key = $api_key | secret unwrap
  let db = $database_url | secret unwrap
  let p = $port | secret unwrap
  
  # ... deployment logic with unwrapped values
}

# Call with secret types
deploy_app "myapp" 
  ("sk-secret123" | secret wrap-string)
  ("postgres://user:pass@host/db" | secret wrap-string)
  (8080 | secret wrap-int)
```

### Pattern 10: Gradual Migration Strategy

**Phase 1: Identify sensitive data**
```nushell
# Audit existing scripts for sensitive data
def audit_script [file: path] {
  open $file 
  | lines 
  | enumerate 
  | where item =~ "password|api_key|secret|token|credential"
  | each { |line| 
      echo $"Line ($line.index + 1): ($line.item)" 
    }
}
```

**Phase 2: Add secret wrapping at input boundaries**
```nushell
# Wrap secrets as soon as they enter your script
let api_key = ($env.API_KEY | default "" | secret wrap-string)
let db_pass = ($env.DB_PASSWORD | default "" | secret wrap-string)
```

**Phase 3: Update functions to accept secret types**
```nushell
# Modify functions to handle both plain and secret types during transition
def flexible_auth [token: any] {
  let auth_token = if ($token | secret validate) {
    $token
  } else {
    $token | secret wrap-string  # Auto-wrap plain types
  }
  
  # Work with secret type from here on
  use_auth_token $auth_token
}
```

**Phase 4: Remove compatibility and enforce secret types**
```nushell
# Final version - only accept secret types
def secure_auth [token: any] {
  if not ($token | secret validate) {
    error make {
      msg: "Authentication token must be a secret type",
      help: "Use: $token | secret wrap-string"
    }
  }
  
  use_auth_token $token
}
```

## 🚨 Security Best Practices

### 1. **Default to Secret Types**
Always use secret types for any data that could be sensitive.

### 2. **Minimize Unwrapping**
Only use `secret unwrap` when absolutely necessary for external APIs.

### 3. **Validate Types**
Use `secret validate` to ensure you're working with secret types.

### 4. **Pipeline Safety**
Secret types remain protected throughout pipeline operations.

### 5. **Memory Safety**
Secret types automatically clean memory when dropped.

## ⚠️ Common Pitfalls

### 1. **Forgetting to Wrap**
```nushell
# ❌ Still exposed
let api_key = "secret123"
let wrapped = $api_key | secret wrap-string  # Too late!

# ✅ Immediate protection
let api_key = "secret123" | secret wrap-string
```

### 2. **Unnecessary Unwrapping**
```nushell
# ❌ Defeats the purpose
let secret = "data" | secret wrap-string | secret unwrap

# ✅ Keep it wrapped
let secret = "data" | secret wrap-string
```

### 3. **Type Confusion**
```nushell
# ✅ Use type checking
if ($value | secret validate) {
  let type = $value | secret type-of
  echo $"Working with secret {$type}"
}
```

## 🧪 Testing Your Migration

### 1. **Verify Protection**
```nushell
let secret = "sensitive" | secret wrap-string
echo $secret  # Should show <redacted:string>
```

### 2. **Test Functionality**
```nushell
let secret = 42 | secret wrap-int
($secret | secret type-of) == "int"  # Should be true
```

### 3. **Pipeline Compatibility**
```nushell
"test" | secret wrap-string | secret validate  # Should be true
```

## 📚 Additional Resources

- **Plugin Documentation**: `secret info`
- **Security Guide**: [docs/SECURITY.md](SECURITY.md)
- **API Reference**: [docs/API.md](API.md)
- **Examples**: [examples/](../examples/)

## 🆘 Getting Help

If you encounter issues during migration:

1. **Check Plugin Status**: `plugin list | where name == secret`
2. **Validate Installation**: `secret info`
3. **Test Basic Functionality**: `"test" | secret wrap-string`
4. **Review Logs**: Check for any error messages
5. **File Issues**: [GitHub Issues](https://github.com/nushell-works/nu_plugin_secret/issues)

---

**Remember**: Migration to secret types is a security enhancement. Take time to identify all sensitive data in your scripts and wrap them appropriately. The initial effort pays off in long-term security and peace of mind.