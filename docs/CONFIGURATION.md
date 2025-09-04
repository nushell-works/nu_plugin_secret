# nu_plugin_secret Configuration Guide

This guide covers all configuration options and the powerful templating system for customizing how secrets are displayed.

## Table of Contents

1. [Configuration File Location](#configuration-file-location)
2. [Configuration Structure](#configuration-structure)
3. [Redaction Configuration](#redaction-configuration)
4. [Security Configuration](#security-configuration)
5. [Templating System](#templating-system)
6. [Template Variables](#template-variables)
7. [Template Functions](#template-functions)
8. [Configuration Commands](#configuration-commands)
9. [Examples](#examples)
10. [Security Considerations](#security-considerations)

## Configuration File Location

The plugin uses a TOML configuration file located at:
- **Linux/macOS**: `~/.local/share/nushell/plugins/secret/config.toml`
- **Windows**: `%APPDATA%\nushell\plugins\secret\config.toml`

You can view the current configuration file path with:
```nushell
secret config show --file-path
```

## Configuration Structure

The configuration file uses TOML format with the following sections:

```toml
version = "1.0"

[redaction]
mask_secret = false
show_unredacted = false
redaction_template = "<redacted:{{secret_type}}>"

[security]
level = "standard"
audit_enabled = true
```

## Redaction Configuration

### `mask_secret`
**Type**: Boolean
**Default**: `false`
**Description**: When enabled, replaces the actual secret value with asterisks (`*`) in template functions that access the secret content.

```toml
[redaction]
mask_secret = true
```

### `show_unredacted`
**Type**: Boolean
**Default**: `false`
**Environment Override**: `SHOW_UNREDACTED=1` or `SHOW_UNREDACTED=true`
**Description**: **⚠️ DANGEROUS**: When enabled, secrets display their actual values instead of redacted output. Only use for debugging in secure environments.

```toml
[redaction]
show_unredacted = false  # Keep this false in production!
```

### `redaction_template`
**Type**: String
**Default**: `"<redacted:{{secret_type}}>"`
**Description**: Tera template string that defines how secrets are displayed. This is the core customization point for secret presentation.

```toml
[redaction]
redaction_template = "<redacted:{{secret_type}}>"
```

## Security Configuration

### `level`
**Type**: String
**Options**: `"minimal"`, `"standard"`, `"paranoid"`
**Default**: `"standard"`
**Description**: Sets overall security posture affecting various security checks.

- **`minimal`**: Basic security checks, allows audit disabling
- **`standard`**: Balanced security, requires audit logging
- **`paranoid`**: Maximum security, strictest validation

```toml
[security]
level = "standard"
```

### `audit_enabled`
**Type**: Boolean
**Default**: `true`
**Description**: Enables audit logging of secret operations. Required for `standard` and `paranoid` security levels.

```toml
[security]
audit_enabled = true
```

## Templating System

The plugin uses the **Tera templating engine** for flexible secret redaction. Templates allow you to customize exactly how secrets appear when displayed.

### Basic Template Syntax

Templates use Tera's template syntax with double curly braces:
```
{{variable_name}}
{{function_name(parameter=value)}}
```

### Default Templates

The default template for all secrets:
```
<redacted:{{secret_type}}>
```

This produces output like:
- `<redacted:string>` for SecretString
- `<redacted:int>` for SecretInt
- `<redacted:record>` for SecretRecord

## Template Variables

The following variables are available in templates:

### `secret_type`
**Type**: String
**Description**: The type of the secret (e.g., "string", "int", "bool", "record", "list", "float", "binary", "date")
**Always Available**: Yes

```
{{secret_type}}
```

### `secret_length`
**Type**: Number
**Description**: The length of the secret value (character count for strings, element count for lists, etc.)
**Availability**: When the secret has content

```
Secret has {{secret_length}} characters
```

### `secret_string`
**Type**: String
**Description**: **⚠️ SENSITIVE**: The actual secret value as a string
**Availability**: When `show_unredacted` is enabled OR when used with template functions
**Security**: Respects `mask_secret` setting

```
The secret is: {{secret_string}}              → Direct variable access (no parentheses)
Length: {{strlen(s=secret_string)}}           → Used as function parameter
Prefix: {{take(n=3, s=secret_string)}}       → Used in other functions
```

## Template Functions

The templating system provides several built-in functions for flexible redaction:

### `replicate(s, n)`
**Purpose**: Repeat a string pattern n times
**Parameters**:
- `s` (string): The string to repeat
- `n` (number): Number of repetitions

**Examples**:
```
{{replicate(s="*", n=8)}}          → "********"
{{replicate(s="-", n=secret_length)}} → "-------" (matches secret length)
{{replicate(s="X", n=5)}}          → "XXXXX"
```

### `mask_partial(s, l, r, c)`
**Purpose**: Show parts of a string while masking the middle
**Parameters**:
- `s` (string, required): The string to mask
- `l` (number, optional): Characters to show from left (default: 0)
- `r` (number, optional): Characters to show from right (default: 0)
- `c` (string, optional): Masking character (default: "*")

**Examples**:
```
{{mask_partial(s="password123", l=2, r=2)}}     → "pa*******23"
{{mask_partial(s="secret", l=1, r=1, c="#")}}   → "s####t"
{{mask_partial(s="api-key")}}                   → "******"
```

**⚠️ Security Warning**: This function can expose parts of secrets. Use with extreme caution.

### `take(n, s)`
**Purpose**: Take the first n characters from a string
**Parameters**:
- `n` (number): Number of characters to take
- `s` (string): Source string

**Examples**:
```
{{take(n=3, s="hello world")}}     → "hel"
{{take(n=5, s="testing")}}         → "testi"
```

### `reverse(s)`
**Purpose**: Reverse a string
**Parameters**:
- `s` (string): String to reverse

**Examples**:
```
{{reverse(s="hello")}}             → "olleh"
{{reverse(s="123abc")}}            → "cba321"
```

### `strlen(s)`
**Purpose**: Get the length of a string
**Parameters**:
- `s` (string): String to measure

**Examples**:
```
{{strlen(s="hello")}}              → "5"
{{strlen(s=secret_string)}}        → Length of the secret
```


## Configuration Commands

### View Configuration
```nushell
# Show structured configuration
secret config show

# Show raw TOML
secret config show --raw

# Show config file path
secret config show --file-path
```

### Interactive Configuration
```nushell
# Interactive configuration with prompts
secret configure

# Set security level directly
secret configure --security-level paranoid
```

### Validate Configuration
```nushell
# Check configuration validity
secret config validate

# Validate with detailed output
secret config validate --verbose
```

### Backup and Restore
```nushell
# Export configuration
secret config export backup.toml

# Import configuration
secret config import backup.toml
```

### Reset Configuration
```nushell
# Reset to defaults (with confirmation)
secret config reset
```

## Examples

### Basic Templates

**Simple type indicator**:
```toml
redaction_template = "[{{secret_type}}]"
```
Output: `[string]`, `[int]`, etc.

**With length information**:
```toml
redaction_template = "{{secret_type}}({{secret_length}})"
```
Output: `string(8)`, `int(5)`, etc.

### Masking Templates

**Fixed-length asterisks**:
```toml
redaction_template = "{{replicate(s='*', n=8)}}"
```
Output: `********` (always 8 characters)

**Length-matched masking**:
```toml
redaction_template = "{{replicate(s='*', n=secret_length)}}"
```
Output: `*******` (matches actual secret length)

**Custom characters**:
```toml
redaction_template = "[{{replicate(s='-', n=secret_length)}}]"
```
Output: `[-------]` (dashes within brackets)

### Partial Reveal Templates

**⚠️ Use with extreme caution - these expose secret data**:

```toml
# Show first 2 and last 2 characters
redaction_template = "{{mask_partial(s=secret_string, l=2, r=2)}}"
```
Output: `pa****rd` for "password"

```toml
# Show only first 3 characters
redaction_template = "{{take(n=3, s=secret_string)}}..."
```
Output: `sec...` for "secret123"

### Complex Templates

**Conditional-style display**:
```toml
redaction_template = "{{secret_type}}: {{replicate(s='█', n=secret_length)}}"
```
Output: `string: ████████`

**Security-conscious partial display**:
```toml
redaction_template = "{{secret_type}}[{{strlen(s=secret_string)}}]: {{mask_partial(s=secret_string, l=1, r=0, c='*')}}"
```
Output: `string[8]: s*******`

**Multiple function combination**:
```toml
redaction_template = "<{{reverse(s=secret_type)}}:{{replicate(s='#', n=3)}}>"
```
Output: `<gnirts:###>` (reversed type with hash marks)

## Security Considerations

### Safe Templates ✅
These templates don't expose secret content:
```toml
redaction_template = "<redacted:{{secret_type}}>"                    # Default, safest
redaction_template = "{{secret_type}}({{secret_length}})"           # Shows only length
redaction_template = "{{replicate(s='*', n=secret_length)}}"        # Length-matched masking
redaction_template = "[PROTECTED:{{secret_type}}]"                  # Custom safe format
```

### Potentially Unsafe Templates ⚠️
These templates may expose secret data:
```toml
redaction_template = "{{secret_string}}"                          # Exposes full secret!
redaction_template = "{{mask_partial(s=secret_string, l=3, r=3)}}" # Exposes partial content
redaction_template = "{{take(n=4, s=secret_string)}}"             # Exposes prefix
```

### Template Security Guidelines

1. **Never use `secret_string` directly** unless `show_unredacted` is intentionally enabled
2. **Avoid `mask_partial` in production** unless you specifically need partial reveals
3. **Test templates thoroughly** before deploying to production
4. **Use `secret config validate`** to check template syntax
5. **Consider attack scenarios** where partial reveals might compromise security
6. **Document your template choices** and their security implications
7. **Regular security review** of custom templates

### Environment-Based Configuration

For development environments:
```bash
export SHOW_UNREDACTED=1  # Temporarily show actual values
```

For production environments:
```toml
[redaction]
show_unredacted = false    # Always false in production
mask_secret = false        # Unless you need extra masking
redaction_template = "<redacted:{{secret_type}}>"  # Safe default

[security]
level = "standard"         # Or "paranoid" for high-security environments
audit_enabled = true       # Always enabled in production
```

---

## Quick Reference

| Configuration | Default | Purpose |
|--------------|---------|---------|
| `mask_secret` | `false` | Mask secret values in template functions |
| `show_unredacted` | `false` | **⚠️ DANGEROUS**: Show actual secret values |
| `redaction_template` | `"<redacted:{{secret_type}}>"` | Template for secret display |
| `security.level` | `"standard"` | Overall security posture |
| `security.audit_enabled` | `true` | Enable audit logging |

| Template Function            | Purpose               | Security Risk       |
|------------------------------|-----------------------|----------------------|
| `replicate(s, n)`            | Repeat pattern        | ✅ Safe              |
| `strlen(s)`                  | String length         | ✅ Safe              |
| `reverse(s)`                 | Reverse string        | ✅ Depends on input  |
| `take(n, s)`                 | First n characters    | ✅ Can expose data   |
| `mask_partial(s, l, r, c)`   | Partial masking       | ⚠️ Exposes parts     |
| `secret_string` (variable)   | **Full secret value** | 🚨 **HIGH RISK**    |

For questions or security concerns, please review the [Security Guidelines](../SECURITY.md) and [Best Practices](BEST_PRACTICES.md).
