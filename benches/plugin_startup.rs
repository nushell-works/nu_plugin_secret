#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(unused_imports)]

use criterion::{criterion_group, criterion_main, Criterion};
use nu_plugin::{Plugin, PluginCommand};
use nu_plugin_secret::SecretPlugin;
use std::time::Instant;

/// Benchmark plugin initialization
fn bench_plugin_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_startup");

    // Benchmark plugin struct creation
    group.bench_function("plugin_creation", |b| b.iter(|| SecretPlugin));

    // Benchmark command registration
    group.bench_function("command_registration", |b| {
        b.iter(|| {
            let plugin = SecretPlugin;
            let _commands = plugin.commands();
        })
    });

    // Benchmark plugin metadata
    group.bench_function("plugin_metadata", |b| {
        b.iter(|| {
            let plugin = SecretPlugin;
            let _metadata = plugin.version();
        })
    });

    group.finish();
}

/// Benchmark command lookup performance
fn bench_command_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_lookup");

    let plugin = SecretPlugin;
    let commands = plugin.commands();

    // Test lookup for each command
    let command_names = vec![
        "secret wrap-string",
        "secret wrap-int",
        "secret wrap-bool",
        "secret wrap-record",
        "secret wrap-list",
        "secret wrap-float",
        "secret wrap-binary",
        "secret wrap-date",
        "secret unwrap",
        "secret info",
        "secret validate",
        "secret type-of",
    ];

    for name in command_names {
        group.bench_function(&format!("lookup_{}", name.replace(' ', "_")), |b| {
            b.iter(|| commands.iter().find(|cmd| cmd.name() == name))
        });
    }

    group.finish();
}

/// Benchmark memory overhead of plugin instance
fn bench_plugin_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_memory");

    // Benchmark multiple plugin instances
    let instance_counts = vec![1, 10, 100];

    for count in instance_counts {
        group.bench_function(&format!("instances_{}", count), |b| {
            b.iter(|| {
                let mut plugins = Vec::new();
                for _ in 0..count {
                    plugins.push(SecretPlugin);
                }
                // Implicit drop
            })
        });
    }

    group.finish();
}

/// Measure actual startup time from process perspective
fn bench_process_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_startup");

    // This benchmarks the full plugin binary startup
    // Note: This is an approximation since we can't easily measure
    // the actual plugin process startup in a benchmark
    group.bench_function("full_initialization", |b| {
        b.iter(|| {
            let start = Instant::now();

            // Simulate what happens during plugin startup:
            // 1. Create plugin instance
            let plugin = SecretPlugin;

            // 2. Register all commands
            let _commands = plugin.commands();

            // 3. Setup plugin metadata
            let _metadata = plugin.version();

            // 4. Verify plugin is ready for first command
            let commands = plugin.commands();
            assert!(!commands.is_empty());

            start.elapsed()
        })
    });

    group.finish();
}

/// Benchmark the time to handle first command after startup
fn bench_first_command_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("first_command");

    group.bench_function("first_wrap_string", |b| {
        b.iter_batched(
            || {
                // Setup: Create fresh plugin instance (simulating startup)
                SecretPlugin
            },
            |plugin| {
                // Execute: Run first command
                let commands = plugin.commands();
                let wrap_cmd = commands
                    .iter()
                    .find(|cmd| cmd.name() == "secret wrap-string")
                    .expect("wrap-string command should exist");

                // Just verify the command exists and is callable
                wrap_cmd.name().to_string()
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_plugin_initialization,
    bench_command_lookup,
    bench_plugin_memory,
    bench_process_startup,
    bench_first_command_latency
);

criterion_main!(benches);
