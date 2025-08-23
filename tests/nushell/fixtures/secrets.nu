# Test secret data for Nushell tests

# Load test data
export def load_test_data [] {
    open tests/nushell/fixtures/test_data.json
}

# Generate test secrets for performance testing
export def generate_test_secrets [count: int] {
    0..$count | each { |i| $"test-secret-($i)" }
}

# Common test patterns
export def get_unicode_test_string [] {
    "🔐 Test with émojis, ñoñ-ASCII chars, and 中文 content"
}

export def get_long_test_string [length: int = 1000] {
    "x" | fill --character "x" --length $length
}

export def get_special_chars_string [] {
    "!@#$%^&*()_+-=[]{}|;':\",./<>?`~"
}

# API key patterns for realistic testing
export def get_test_api_keys [] {
    [
        "sk-1234567890abcdef",
        "pk_test_51HvKEbGvW4LiJi",
        "xoxb-1234567890-abcdefghijk",
        "ya29.a0AfH6SMC1234567890",
        "AIzaSyD-1234567890abcdef"
    ]
}

# Database connection strings
export def get_test_connection_strings [] {
    [
        "postgresql://user:pass@localhost:5432/db",
        "mysql://root:password@127.0.0.1:3306/test",
        "redis://user:secret@redis.example.com:6379/0",
        "mongodb://admin:password@mongo.example.com:27017/database"
    ]
}

# JWT tokens for testing
export def get_test_tokens [] {
    [
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
        "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJ0ZXN0LWlzc3VlciIsImF1ZCI6InRlc3QtYXVkaWVuY2UifQ",
        "bearer-token-1234567890abcdef",
        "access_token_abcdefghijklmnop"
    ]
}

# Binary test data
export def get_test_binary_data [] {
    [
        (0x[deadbeef] | into binary),
        (0x[cafebabe] | into binary),
        (0x[0123456789abcdef] | into binary),
        (0x[] | into binary)  # empty binary
    ]
}