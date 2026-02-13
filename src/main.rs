//! Plugin binary entry point — serves `SecretPlugin` via MsgPackSerializer.

use std::time::Instant;

use nu_plugin::{serve_plugin, MsgPackSerializer};
use nu_plugin_secret::{performance_monitoring, startup_optimizations};

fn main() {
    let start_time = Instant::now();

    // Initialize performance monitoring
    let monitoring_config = performance_monitoring::MonitoringConfig {
        detailed_timing: false, // Disabled in production for performance
        track_memory: true,
        export_metrics: false,
        ..performance_monitoring::MonitoringConfig::default()
    };
    performance_monitoring::init_monitoring(monitoring_config);

    // Initialize startup optimizations
    startup_optimizations::profiling::start_profiling();

    // Fast plugin initialization
    let config = startup_optimizations::StartupConfig::default();
    let plugin = startup_optimizations::initialize_plugin(config);

    startup_optimizations::profiling::mark_init_complete();

    // Initialize redaction templating system early for faster redaction
    let _ = nu_plugin_secret::redaction::init_redaction_templating();

    startup_optimizations::profiling::mark_commands_ready();

    // Record startup time
    let startup_duration = start_time.elapsed();
    performance_monitoring::get_monitor().record(performance_monitoring::Measurement {
        metric_type: performance_monitoring::MetricType::StartupTime,
        value: startup_duration.as_millis() as f64,
        unit: performance_monitoring::Unit::Milliseconds,
        timestamp: start_time,
        context: Some("plugin_startup".to_string()),
    });

    serve_plugin(&plugin, MsgPackSerializer {})
}
