//! Startup time optimizations for the nu_plugin_secret
//!
//! This module contains optimizations to reduce plugin startup time
//! while maintaining all functionality and security guarantees.
//!
//! Startup optimizations focus on caching and initialization
//! rather than complex command management

/// Fast startup mode configuration
#[derive(Clone, Debug)]
pub struct StartupConfig {
    /// Skip expensive initialization during startup
    pub fast_mode: bool,
    /// Defer command validation until first use
    pub lazy_validation: bool,
    /// Pre-allocate common data structures
    pub pre_allocate: bool,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            fast_mode: true,
            lazy_validation: true,
            pre_allocate: false, // Disabled by default to save memory
        }
    }
}

/// Optimized plugin initialization
pub fn initialize_plugin(config: StartupConfig) -> crate::SecretPlugin {
    if config.fast_mode {
        // Initialize string cache early if needed
        crate::memory_optimizations::init_string_cache();
    }

    if config.pre_allocate {
        // Pre-allocate commonly used structures
        pre_allocate_common_structures();
    }

    crate::SecretPlugin
}

/// Pre-allocate common data structures to reduce allocation overhead
fn pre_allocate_common_structures() {
    // Pre-allocate small string capacity for common operations
    let _common_strings = [
        String::with_capacity(64),   // Small secrets
        String::with_capacity(256),  // Medium secrets
        String::with_capacity(1024), // Large secrets
    ];

    // Pre-warm the string cache
    let _ = crate::memory_optimizations::get_redacted_string("string");
    let _ = crate::memory_optimizations::get_redacted_string("int");
    let _ = crate::memory_optimizations::get_redacted_string("binary");
}

/// Startup profiling utilities
pub mod profiling {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    /// Startup timing information
    #[derive(Debug, Clone)]
    pub struct StartupTiming {
        pub total_time: Duration,
        pub init_time: Duration,
        pub command_registration_time: Duration,
        pub first_command_ready_time: Duration,
    }

    /// Global startup profiler
    static STARTUP_PROFILER: Mutex<Option<StartupProfiler>> = Mutex::new(None);

    /// Internal profiler state
    struct StartupProfiler {
        start_time: Instant,
        init_complete: Option<Instant>,
        commands_ready: Option<Instant>,
    }

    /// Start profiling plugin startup
    pub fn start_profiling() {
        let mut profiler = STARTUP_PROFILER.lock().unwrap();
        *profiler = Some(StartupProfiler {
            start_time: Instant::now(),
            init_complete: None,
            commands_ready: None,
        });
    }

    /// Mark initialization complete
    pub fn mark_init_complete() {
        let mut profiler = STARTUP_PROFILER.lock().unwrap();
        if let Some(ref mut p) = *profiler {
            p.init_complete = Some(Instant::now());
        }
    }

    /// Mark commands ready
    pub fn mark_commands_ready() {
        let mut profiler = STARTUP_PROFILER.lock().unwrap();
        if let Some(ref mut p) = *profiler {
            p.commands_ready = Some(Instant::now());
        }
    }

    /// Get startup timing information
    pub fn get_timing() -> Option<StartupTiming> {
        let profiler = STARTUP_PROFILER.lock().unwrap();
        if let Some(ref p) = *profiler {
            let now = Instant::now();
            let total_time = now.duration_since(p.start_time);

            let init_time = p
                .init_complete
                .map(|t| t.duration_since(p.start_time))
                .unwrap_or(Duration::ZERO);

            let command_registration_time = p
                .commands_ready
                .map(|t| t.duration_since(p.init_complete.unwrap_or(p.start_time)))
                .unwrap_or(Duration::ZERO);

            let first_command_ready_time = p
                .commands_ready
                .map(|t| t.duration_since(p.start_time))
                .unwrap_or(total_time);

            Some(StartupTiming {
                total_time,
                init_time,
                command_registration_time,
                first_command_ready_time,
            })
        } else {
            None
        }
    }

    /// Reset profiler for next measurement
    pub fn reset() {
        let mut profiler = STARTUP_PROFILER.lock().unwrap();
        *profiler = None;
    }
}

/// Command-specific optimizations
pub mod command_optimizations {
    use std::collections::HashMap;
    use std::sync::OnceLock;

    /// Command metadata cache for faster lookups
    static COMMAND_METADATA: OnceLock<HashMap<&'static str, CommandMeta>> = OnceLock::new();

    /// Cached command metadata
    #[derive(Clone, Debug)]
    struct CommandMeta {
        name: &'static str,
        #[allow(dead_code)]
        description: &'static str,
        category: CommandCategory,
    }

    /// Command categories for optimization
    #[derive(Clone, Debug, PartialEq)]
    pub enum CommandCategory {
        Wrap,
        Utility,
        Security,
    }

    /// Initialize command metadata cache
    pub fn init_command_cache() {
        COMMAND_METADATA.get_or_init(|| {
            let mut cache = HashMap::new();

            // Wrap commands
            cache.insert(
                "secret wrap-string",
                CommandMeta {
                    name: "secret wrap-string",
                    description: "Convert string to SecretString",
                    category: CommandCategory::Wrap,
                },
            );
            cache.insert(
                "secret wrap-int",
                CommandMeta {
                    name: "secret wrap-int",
                    description: "Convert int to SecretInt",
                    category: CommandCategory::Wrap,
                },
            );
            cache.insert(
                "secret wrap-bool",
                CommandMeta {
                    name: "secret wrap-bool",
                    description: "Convert bool to SecretBool",
                    category: CommandCategory::Wrap,
                },
            );
            cache.insert(
                "secret wrap-record",
                CommandMeta {
                    name: "secret wrap-record",
                    description: "Convert record to SecretRecord",
                    category: CommandCategory::Wrap,
                },
            );
            cache.insert(
                "secret wrap-list",
                CommandMeta {
                    name: "secret wrap-list",
                    description: "Convert list to SecretList",
                    category: CommandCategory::Wrap,
                },
            );
            cache.insert(
                "secret wrap-float",
                CommandMeta {
                    name: "secret wrap-float",
                    description: "Convert float to SecretFloat",
                    category: CommandCategory::Wrap,
                },
            );
            cache.insert(
                "secret wrap-binary",
                CommandMeta {
                    name: "secret wrap-binary",
                    description: "Convert binary to SecretBinary",
                    category: CommandCategory::Wrap,
                },
            );
            cache.insert(
                "secret wrap-date",
                CommandMeta {
                    name: "secret wrap-date",
                    description: "Convert date to SecretDate",
                    category: CommandCategory::Wrap,
                },
            );

            // Utility commands
            cache.insert(
                "secret unwrap",
                CommandMeta {
                    name: "secret unwrap",
                    description: "Extract underlying value (WARNING: exposes data)",
                    category: CommandCategory::Security,
                },
            );
            cache.insert(
                "secret info",
                CommandMeta {
                    name: "secret info",
                    description: "Display plugin information",
                    category: CommandCategory::Utility,
                },
            );
            cache.insert(
                "secret validate",
                CommandMeta {
                    name: "secret validate",
                    description: "Check if value is secret type",
                    category: CommandCategory::Utility,
                },
            );
            cache.insert(
                "secret type-of",
                CommandMeta {
                    name: "secret type-of",
                    description: "Get underlying type of secret",
                    category: CommandCategory::Utility,
                },
            );

            cache
        });
    }

    /// Fast command lookup by category
    pub fn get_commands_by_category(category: CommandCategory) -> Vec<&'static str> {
        let cache = COMMAND_METADATA.get().unwrap_or_else(|| {
            init_command_cache();
            COMMAND_METADATA.get().unwrap()
        });

        cache
            .values()
            .filter(|meta| meta.category == category)
            .map(|meta| meta.name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_startup_config() {
        let config = StartupConfig::default();
        assert!(config.fast_mode);
        assert!(config.lazy_validation);
        assert!(!config.pre_allocate);
    }

    #[test]
    fn test_command_cache_initialization() {
        command_optimizations::init_command_cache();

        let wrap_commands = command_optimizations::get_commands_by_category(
            command_optimizations::CommandCategory::Wrap,
        );
        assert_eq!(wrap_commands.len(), 8);
    }

    #[test]
    fn test_startup_profiling() {
        profiling::reset();
        profiling::start_profiling();

        std::thread::sleep(Duration::from_millis(1));
        profiling::mark_init_complete();

        std::thread::sleep(Duration::from_millis(1));
        profiling::mark_commands_ready();

        let timing = profiling::get_timing().unwrap();
        assert!(timing.total_time > Duration::ZERO);
        assert!(timing.init_time > Duration::ZERO);
        assert!(timing.first_command_ready_time > timing.init_time);
    }

    #[test]
    fn test_command_cache() {
        command_optimizations::init_command_cache();

        let wrap_commands = command_optimizations::get_commands_by_category(
            command_optimizations::CommandCategory::Wrap,
        );
        assert_eq!(wrap_commands.len(), 8);

        let utility_commands = command_optimizations::get_commands_by_category(
            command_optimizations::CommandCategory::Utility,
        );
        assert_eq!(utility_commands.len(), 3);

        let security_commands = command_optimizations::get_commands_by_category(
            command_optimizations::CommandCategory::Security,
        );
        assert_eq!(security_commands.len(), 1);
    }
}
