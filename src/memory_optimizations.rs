//! Memory optimization utilities for the secret plugin
//!
//! This module contains optimizations to reduce memory usage and improve
//! performance while maintaining security guarantees.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Static string interning for common redacted messages
/// This reduces memory allocation for repeated redaction strings
static REDACTED_STRINGS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

/// Initialize the interned string cache
pub fn init_string_cache() {
    REDACTED_STRINGS.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert("string", "<redacted:string>");
        map.insert("int", "<redacted:int>");
        map.insert("bool", "<redacted:bool>");
        map.insert("float", "<redacted:float>");
        map.insert("binary", "<redacted:binary>");
        map.insert("date", "<redacted:date>");
        map.insert("record", "<redacted:record>");
        map.insert("list", "<redacted:list>");
        map
    });
}

/// Get an interned redacted string for a given type
/// This avoids repeated allocations of the same strings
/// Falls back to hardcoded values if configuration is not available
pub fn get_redacted_string(type_name: &str) -> &'static str {
    let cache = REDACTED_STRINGS.get().unwrap_or_else(|| {
        init_string_cache();
        REDACTED_STRINGS.get().unwrap()
    });

    cache.get(type_name).copied().unwrap_or("<redacted>")
}

/// Get configurable redacted string for a given type and context
/// This uses the new configuration system when available
pub fn get_configurable_redacted_string(
    type_name: &str,
    context: crate::config::RedactionContext,
) -> String {
    if let Ok(config) = crate::config::get_config() {
        config.get_redaction_text(type_name, context)
    } else {
        // Fallback to static string if config not available
        get_redacted_string(type_name).to_string()
    }
}

/// Memory pool for small allocations
/// This reduces allocation overhead for small secret values
pub struct SecretMemoryPool {
    small_strings: Vec<String>,
    medium_strings: Vec<String>,
    large_strings: Vec<String>,
}

impl SecretMemoryPool {
    const SMALL_SIZE: usize = 64;
    const MEDIUM_SIZE: usize = 1024;
    const POOL_INITIAL_CAPACITY: usize = 16;

    pub fn new() -> Self {
        Self {
            small_strings: Vec::with_capacity(Self::POOL_INITIAL_CAPACITY),
            medium_strings: Vec::with_capacity(Self::POOL_INITIAL_CAPACITY),
            large_strings: Vec::with_capacity(Self::POOL_INITIAL_CAPACITY),
        }
    }

    /// Get a pre-allocated string from the pool or create a new one
    pub fn get_string(&mut self, size_hint: usize) -> String {
        if size_hint <= Self::SMALL_SIZE {
            self.small_strings
                .pop()
                .unwrap_or_else(|| String::with_capacity(Self::SMALL_SIZE))
        } else if size_hint <= Self::MEDIUM_SIZE {
            self.medium_strings
                .pop()
                .unwrap_or_else(|| String::with_capacity(Self::MEDIUM_SIZE))
        } else {
            self.large_strings
                .pop()
                .unwrap_or_else(|| String::with_capacity(size_hint))
        }
    }

    /// Return a string to the pool for reuse
    pub fn return_string(&mut self, mut s: String) {
        // Clear content but keep capacity
        s.clear();

        let capacity = s.capacity();
        if capacity <= Self::SMALL_SIZE && self.small_strings.len() < Self::POOL_INITIAL_CAPACITY {
            self.small_strings.push(s);
        } else if capacity <= Self::MEDIUM_SIZE
            && self.medium_strings.len() < Self::POOL_INITIAL_CAPACITY
        {
            self.medium_strings.push(s);
        } else if self.large_strings.len() < Self::POOL_INITIAL_CAPACITY {
            self.large_strings.push(s);
        }
        // If pools are full, just drop the string
    }
}

impl Default for SecretMemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimize binary data storage for common patterns
pub mod binary_optimization {
    use std::borrow::Cow;
    use zeroize::Zeroize;

    /// Common binary patterns that can be stored more efficiently
    #[derive(Clone)]
    pub enum OptimizedBinary {
        /// All zeros pattern - store just the length
        Zeros(usize),
        /// All ones pattern - store just the length  
        Ones(usize),
        /// Repeating byte pattern
        Repeated(u8, usize),
        /// Small binary data (inline storage)
        Small([u8; 32], usize),
        /// Large binary data (heap allocated)
        Large(Vec<u8>),
    }

    impl OptimizedBinary {
        /// Create optimized binary from a slice
        pub fn from_slice(data: &[u8]) -> Self {
            let len = data.len();

            if len == 0 {
                return Self::Small([0; 32], 0);
            }

            // Check for common patterns
            let first_byte = data[0];

            if data.iter().all(|&b| b == 0) {
                Self::Zeros(len)
            } else if data.iter().all(|&b| b == 0xFF) {
                Self::Ones(len)
            } else if data.iter().all(|&b| b == first_byte) {
                Self::Repeated(first_byte, len)
            } else if len <= 32 {
                let mut small = [0; 32];
                small[..len].copy_from_slice(data);
                Self::Small(small, len)
            } else {
                Self::Large(data.to_vec())
            }
        }

        /// Get the data as a Cow (clone-on-write)
        pub fn as_bytes(&self) -> Cow<'_, [u8]> {
            match self {
                Self::Zeros(len) => Cow::Owned(vec![0; *len]),
                Self::Ones(len) => Cow::Owned(vec![0xFF; *len]),
                Self::Repeated(byte, len) => Cow::Owned(vec![*byte; *len]),
                Self::Small(data, len) => Cow::Borrowed(&data[..*len]),
                Self::Large(data) => Cow::Borrowed(data),
            }
        }

        /// Get the length without reconstructing the data
        pub fn len(&self) -> usize {
            match self {
                Self::Zeros(len) | Self::Ones(len) | Self::Repeated(_, len) => *len,
                Self::Small(_, len) => *len,
                Self::Large(data) => data.len(),
            }
        }

        /// Check if empty
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }

    impl Zeroize for OptimizedBinary {
        fn zeroize(&mut self) {
            match self {
                Self::Zeros(_) | Self::Ones(_) | Self::Repeated(_, _) => {
                    // These patterns don't contain actual secret data
                }
                Self::Small(data, len) => {
                    // Zero the actual used portion
                    data[..*len].zeroize();
                    *len = 0;
                }
                Self::Large(data) => {
                    data.zeroize();
                }
            }
        }
    }
}

/// Memory usage statistics for monitoring
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_secrets: usize,
    pub string_secrets: usize,
    pub binary_secrets: usize,
    pub record_secrets: usize,
    pub list_secrets: usize,
    pub estimated_memory_kb: usize,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self {
            total_secrets: 0,
            string_secrets: 0,
            binary_secrets: 0,
            record_secrets: 0,
            list_secrets: 0,
            estimated_memory_kb: 0,
        }
    }

    pub fn add_string_secret(&mut self, size: usize) {
        self.total_secrets += 1;
        self.string_secrets += 1;
        // Round up to ensure non-zero for small allocations
        self.estimated_memory_kb += (size + std::mem::size_of::<String>()).div_ceil(1024);
    }

    pub fn add_binary_secret(&mut self, size: usize) {
        self.total_secrets += 1;
        self.binary_secrets += 1;
        // Round up to ensure non-zero for small allocations
        self.estimated_memory_kb += (size + std::mem::size_of::<Vec<u8>>()).div_ceil(1024);
    }

    pub fn memory_efficiency_ratio(&self) -> f64 {
        if self.total_secrets == 0 {
            return 1.0;
        }
        // Simple heuristic: secrets should be efficient compared to plain storage
        let baseline_kb = self.total_secrets * 64 / 1024; // Assume 64 bytes per secret baseline
        if baseline_kb == 0 {
            1.0
        } else {
            baseline_kb as f64 / self.estimated_memory_kb.max(1) as f64
        }
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning() {
        init_string_cache();

        let s1 = get_redacted_string("string");
        let s2 = get_redacted_string("string");

        // Should be the same memory location (interned)
        assert_eq!(s1.as_ptr(), s2.as_ptr());
        assert_eq!(s1, "<redacted:string>");
    }

    #[test]
    fn test_binary_optimization() {
        use binary_optimization::OptimizedBinary;

        // Test zeros optimization
        let zeros = vec![0; 1000];
        let opt_zeros = OptimizedBinary::from_slice(&zeros);
        assert!(matches!(opt_zeros, OptimizedBinary::Zeros(1000)));
        assert_eq!(opt_zeros.len(), 1000);

        // Test ones optimization
        let ones = vec![0xFF; 500];
        let opt_ones = OptimizedBinary::from_slice(&ones);
        assert!(matches!(opt_ones, OptimizedBinary::Ones(500)));

        // Test repeated pattern optimization
        let repeated = vec![0xAA; 200];
        let opt_repeated = OptimizedBinary::from_slice(&repeated);
        assert!(matches!(opt_repeated, OptimizedBinary::Repeated(0xAA, 200)));

        // Test small data optimization
        let small_data = vec![1, 2, 3, 4, 5];
        let opt_small = OptimizedBinary::from_slice(&small_data);
        assert!(matches!(opt_small, OptimizedBinary::Small(_, 5)));
    }

    #[test]
    fn test_memory_pool() {
        let mut pool = SecretMemoryPool::new();

        // Get some strings
        let s1 = pool.get_string(32);
        let s2 = pool.get_string(100);
        let s3 = pool.get_string(2000);

        assert!(s1.capacity() >= 32);
        assert!(s2.capacity() >= 100);
        assert!(s3.capacity() >= 2000);

        // Return them to the pool
        pool.return_string(s1);
        pool.return_string(s2);
        pool.return_string(s3);

        // Get them back (should reuse)
        let s4 = pool.get_string(30);
        assert!(s4.capacity() >= 30);
    }

    #[test]
    fn test_memory_stats() {
        let mut stats = MemoryStats::new();
        assert_eq!(stats.total_secrets, 0);

        stats.add_string_secret(100);
        stats.add_binary_secret(200);

        assert_eq!(stats.total_secrets, 2);
        assert_eq!(stats.string_secrets, 1);
        assert_eq!(stats.binary_secrets, 1);
        assert!(stats.estimated_memory_kb > 0);
    }
}
