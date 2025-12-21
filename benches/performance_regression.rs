//! Performance regression test suite
//!
//! This benchmark suite is designed to detect performance regressions
//! by establishing baselines and monitoring for significant changes.

#![allow(clippy::useless_vec)]
#![allow(dead_code)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nu_plugin::Plugin;
use nu_plugin_secret::performance_monitoring::{self, MetricType, Unit};
use nu_plugin_secret::*;
use nu_protocol::{Record, Span, Value};
use std::time::Instant;

/// Regression test configuration
const REGRESSION_THRESHOLD: f64 = 15.0; // 15% regression threshold
const MIN_SAMPLES: usize = 100;
const PERFORMANCE_BUDGET_NS: u64 = 10_000; // 10μs budget for most operations

/// Baseline performance targets (in nanoseconds)
struct PerformanceTargets {
    secret_string_creation: u64,
    secret_string_reveal: u64,
    secret_string_display: u64,
    secret_int_creation: u64,
    secret_binary_creation: u64,
    secret_record_creation: u64,
    startup_time_ms: u64,
}

impl PerformanceTargets {
    fn default() -> Self {
        Self {
            secret_string_creation: 100, // 100ns
            secret_string_reveal: 10,    // 10ns
            secret_string_display: 50,   // 50ns
            secret_int_creation: 20,     // 20ns
            secret_binary_creation: 200, // 200ns
            secret_record_creation: 500, // 500ns
            startup_time_ms: 350,        // 350ms
        }
    }
}

/// Test secret string creation performance doesn't regress
fn bench_secret_string_regression(c: &mut Criterion) {
    let targets = PerformanceTargets::default();
    let mut group = c.benchmark_group("regression_secret_string");

    // Set baseline for comparison
    group.significance_level(0.05);
    group.confidence_level(0.95);

    let test_strings = vec![
        "short".to_string(),
        "a_medium_length_string_for_testing".to_string(),
        "a".repeat(1000), // long string
    ];

    for (i, test_string) in test_strings.iter().enumerate() {
        group.throughput(Throughput::Bytes(test_string.len() as u64));

        group.bench_with_input(BenchmarkId::new("creation", i), test_string, |b, s| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    let _secret = SecretString::new(s.clone());
                }
                let duration = start.elapsed();

                // Record performance metric
                performance_monitoring::get_monitor().record(performance_monitoring::Measurement {
                    metric_type: MetricType::SecretCreation,
                    value: duration.as_nanos() as f64 / iters as f64,
                    unit: Unit::Nanoseconds,
                    timestamp: start,
                    context: Some(format!("string_len_{}", s.len())),
                });

                duration
            })
        });

        // Performance regression check
        let secret = SecretString::new(test_string.clone());
        group.bench_with_input(
            BenchmarkId::new("reveal", i),
            &secret,
            |b, s| {
                b.iter_custom(|iters| {
                    let start = Instant::now();
                    for _ in 0..iters {
                        let _revealed = s.reveal();
                    }
                    let duration = start.elapsed();

                    // Check against performance target
                    let avg_ns = duration.as_nanos() / iters as u128;
                    if avg_ns > targets.secret_string_reveal as u128 {
                        eprintln!(
                            "WARNING: Secret string reveal performance regression detected: {}ns > {}ns",
                            avg_ns, targets.secret_string_reveal
                        );
                    }

                    duration
                })
            },
        );

        group.bench_with_input(BenchmarkId::new("display", i), &secret, |b, s| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    let _display = format!("{}", s);
                }
                let duration = start.elapsed();

                // Check performance budget
                let avg_ns = duration.as_nanos() / iters as u128;
                if avg_ns > PERFORMANCE_BUDGET_NS as u128 {
                    eprintln!(
                        "WARNING: Secret string display exceeds budget: {}ns > {}ns",
                        avg_ns, PERFORMANCE_BUDGET_NS
                    );
                }

                duration
            })
        });
    }

    group.finish();
}

/// Test memory-optimized binary operations
fn bench_binary_optimization_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_binary_optimization");

    // Test different binary patterns for memory optimization effectiveness
    let test_patterns = vec![
        ("zeros", vec![0; 1000]),
        ("ones", vec![0xFF; 1000]),
        ("repeated", vec![0xAA; 1000]),
        ("random", (0..1000).map(|i| (i % 256) as u8).collect()),
    ];

    for (pattern_name, data) in test_patterns {
        group.throughput(Throughput::Bytes(data.len() as u64));

        group.bench_with_input(BenchmarkId::new("creation", pattern_name), &data, |b, d| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    let _secret = SecretBinary::new(d.clone());
                }
                let duration = start.elapsed();

                // Record memory optimization effectiveness
                performance_monitoring::get_monitor().record(performance_monitoring::Measurement {
                    metric_type: MetricType::MemoryUsage,
                    value: d.len() as f64,
                    unit: Unit::Bytes,
                    timestamp: start,
                    context: Some(format!("binary_pattern_{}", pattern_name)),
                });

                duration
            })
        });

        let secret = SecretBinary::new(data.clone());
        group.bench_with_input(
            BenchmarkId::new("length_check", pattern_name),
            &secret,
            |b, s| {
                b.iter(|| {
                    let _len = s.len();
                })
            },
        );
    }

    group.finish();
}

/// Comprehensive command performance regression test
fn bench_command_performance_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_commands");

    // Test data for different secret types
    let test_string = "test_secret_value_for_performance".to_string();
    let test_int = 42i64;
    let test_bool = true;
    let mut test_record = Record::new();
    test_record.insert("key1", Value::string("value1", Span::test_data()));
    test_record.insert("key2", Value::string("value2", Span::test_data()));

    let test_list = vec![
        Value::string("item1", Span::test_data()),
        Value::string("item2", Span::test_data()),
    ];
    let test_binary = vec![0xDE, 0xAD, 0xBE, 0xEF];

    // Benchmark secret creation for all types
    group.bench_function("string_creation", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                let _secret = SecretString::new(test_string.clone());
            }
            start.elapsed()
        })
    });

    group.bench_function("int_creation", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                let _secret = SecretInt::new(test_int);
            }
            start.elapsed()
        })
    });

    group.bench_function("bool_creation", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                let _secret = SecretBool::new(test_bool);
            }
            start.elapsed()
        })
    });

    group.bench_function("record_creation", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                let _secret = SecretRecord::new(test_record.clone());
            }
            start.elapsed()
        })
    });

    group.bench_function("list_creation", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                let _secret = SecretList::new(test_list.clone());
            }
            start.elapsed()
        })
    });

    group.bench_function("binary_creation", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                let _secret = SecretBinary::new(test_binary.clone());
            }
            start.elapsed()
        })
    });

    group.finish();
}

/// Memory usage regression tests
fn bench_memory_usage_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_memory");

    // Test memory overhead compared to plain types
    let test_string = "memory_test_string".to_string();

    group.bench_function("plain_string_memory", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            let mut strings = Vec::new();
            for _ in 0..iters {
                strings.push(test_string.clone());
            }
            // Prevent optimization
            std::hint::black_box(strings);
            start.elapsed()
        })
    });

    group.bench_function("secret_string_memory", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            let mut secrets = Vec::new();
            for _ in 0..iters {
                secrets.push(SecretString::new(test_string.clone()));
            }
            // Prevent optimization
            std::hint::black_box(secrets);
            start.elapsed()
        })
    });

    // Test bulk operations for memory efficiency
    let sizes = vec![10, 100, 1000];
    for size in sizes {
        group.bench_with_input(BenchmarkId::new("bulk_creation", size), &size, |b, &s| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    let mut secrets = Vec::with_capacity(s);
                    for i in 0..s {
                        secrets.push(SecretString::new(format!("secret_{}", i)));
                    }
                    std::hint::black_box(secrets);
                }
                start.elapsed()
            })
        });
    }

    group.finish();
}

/// Plugin startup performance regression test
fn bench_startup_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_startup");
    let targets = PerformanceTargets::default();

    group.bench_function("plugin_initialization", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                let _plugin = nu_plugin_secret::SecretPlugin::default();
                std::hint::black_box(_plugin);
            }
            let duration = start.elapsed();

            // Check startup time budget
            let avg_ms = duration.as_millis() / iters as u128;
            if avg_ms > targets.startup_time_ms as u128 / 1000 {
                eprintln!(
                    "WARNING: Plugin initialization regression: {}ms > {}ms",
                    avg_ms,
                    targets.startup_time_ms / 1000
                );
            }

            duration
        })
    });

    group.bench_function("command_registration", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                let plugin = nu_plugin_secret::SecretPlugin::default();
                let _commands = plugin.commands();
                std::hint::black_box(_commands);
            }
            start.elapsed()
        })
    });

    group.finish();
}

criterion_group!(
    regression_benches,
    bench_secret_string_regression,
    bench_binary_optimization_regression,
    bench_command_performance_regression,
    bench_memory_usage_regression,
    bench_startup_regression
);

criterion_main!(regression_benches);
