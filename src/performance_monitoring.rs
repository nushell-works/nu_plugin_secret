//! Performance monitoring system for nu_plugin_secret
//!
//! This module provides real-time performance monitoring capabilities
//! to track plugin performance, detect regressions, and gather metrics.

use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use std::time::Instant;

/// Performance metric types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// Time to create a secret value
    SecretCreation,
    /// Time to reveal (unwrap) a secret value
    SecretReveal,
    /// Time to display/format a secret value
    SecretDisplay,
    /// Time to serialize a secret value
    SecretSerialization,
    /// Time to deserialize a secret value  
    SecretDeserialization,
    /// Memory usage in bytes
    MemoryUsage,
    /// Plugin startup time
    StartupTime,
    /// Command execution time
    CommandExecution,
}

/// Individual performance measurement
#[derive(Debug, Clone)]
pub struct Measurement {
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: Unit,
    pub timestamp: Instant,
    pub context: Option<String>,
}

/// Measurement units
#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Milliseconds,
    Microseconds,
    Nanoseconds,
    Bytes,
    Kilobytes,
    Megabytes,
    Operations,
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub p95: f64,
    pub p99: f64,
}

/// Performance monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Maximum number of measurements to keep in memory
    pub max_measurements: usize,
    /// Enable detailed timing for all operations
    pub detailed_timing: bool,
    /// Enable memory usage tracking
    pub track_memory: bool,
    /// Performance regression threshold (as percentage)
    pub regression_threshold: f64,
    /// Export metrics to file
    pub export_metrics: bool,
    /// Export file path
    pub export_path: Option<String>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            max_measurements: 1000,
            detailed_timing: false,
            track_memory: true,
            regression_threshold: 20.0, // 20% regression threshold
            export_metrics: false,
            export_path: None,
        }
    }
}

/// Global performance monitor
pub struct PerformanceMonitor {
    measurements: RwLock<HashMap<MetricType, VecDeque<Measurement>>>,
    config: RwLock<MonitoringConfig>,
    baselines: RwLock<HashMap<MetricType, Statistics>>,
}

impl PerformanceMonitor {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            measurements: RwLock::new(HashMap::new()),
            config: RwLock::new(config),
            baselines: RwLock::new(HashMap::new()),
        }
    }

    /// Record a performance measurement
    pub fn record(&self, measurement: Measurement) {
        let config = self.config.read().unwrap();

        if !config.detailed_timing
            && !matches!(
                measurement.metric_type,
                MetricType::MemoryUsage | MetricType::StartupTime
            )
        {
            return;
        }

        let mut measurements = self.measurements.write().unwrap();
        let entries = measurements
            .entry(measurement.metric_type.clone())
            .or_default();

        entries.push_back(measurement);

        // Keep only the most recent measurements
        while entries.len() > config.max_measurements {
            entries.pop_front();
        }
    }

    /// Record timing for a closure
    pub fn time<T, F>(&self, metric_type: MetricType, context: Option<String>, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        self.record(Measurement {
            metric_type,
            value: duration.as_nanos() as f64,
            unit: Unit::Nanoseconds,
            timestamp: start,
            context,
        });

        result
    }

    /// Get statistics for a metric type
    pub fn get_statistics(&self, metric_type: &MetricType) -> Option<Statistics> {
        let measurements = self.measurements.read().unwrap();
        let entries = measurements.get(metric_type)?;

        if entries.is_empty() {
            return None;
        }

        let mut values: Vec<f64> = entries.iter().map(|m| m.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = values.len();
        let min = values[0];
        let max = values[count - 1];
        let mean = values.iter().sum::<f64>() / count as f64;

        let median = if count.is_multiple_of(2) {
            (values[count / 2 - 1] + values[count / 2]) / 2.0
        } else {
            values[count / 2]
        };

        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        let p95_idx = ((count as f64 * 0.95) as usize).min(count - 1);
        let p99_idx = ((count as f64 * 0.99) as usize).min(count - 1);

        Some(Statistics {
            count,
            min,
            max,
            mean,
            median,
            std_dev,
            p95: values[p95_idx],
            p99: values[p99_idx],
        })
    }

    /// Set baseline statistics for regression detection
    pub fn set_baseline(&self, metric_type: MetricType, baseline: Statistics) {
        let mut baselines = self.baselines.write().unwrap();
        baselines.insert(metric_type, baseline);
    }

    /// Check for performance regressions
    pub fn check_regressions(&self) -> Vec<RegressionAlert> {
        let mut alerts = Vec::new();
        let config = self.config.read().unwrap();
        let baselines = self.baselines.read().unwrap();

        for (metric_type, baseline) in baselines.iter() {
            if let Some(current) = self.get_statistics(metric_type) {
                let regression_pct = ((current.mean - baseline.mean) / baseline.mean) * 100.0;

                if regression_pct > config.regression_threshold {
                    alerts.push(RegressionAlert {
                        metric_type: metric_type.clone(),
                        regression_percentage: regression_pct,
                        baseline_mean: baseline.mean,
                        current_mean: current.mean,
                        threshold: config.regression_threshold,
                    });
                }
            }
        }

        alerts
    }

    /// Export current metrics to a file
    pub fn export_metrics(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::Write;

        let measurements = self.measurements.read().unwrap();
        let mut file = File::create(path)?;

        writeln!(file, "timestamp,metric_type,value,unit,context")?;

        for (metric_type, entries) in measurements.iter() {
            for measurement in entries.iter() {
                writeln!(
                    file,
                    "{},{:?},{},{:?},{}",
                    measurement.timestamp.elapsed().as_millis(),
                    metric_type,
                    measurement.value,
                    measurement.unit,
                    measurement.context.as_deref().unwrap_or("")
                )?;
            }
        }

        Ok(())
    }

    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let config = self.config.read().unwrap();
        let measurements = self.measurements.read().unwrap();

        let mut stats_by_metric = HashMap::new();
        for metric_type in measurements.keys() {
            if let Some(stats) = self.get_statistics(metric_type) {
                stats_by_metric.insert(metric_type.clone(), stats);
            }
        }

        let regressions = self.check_regressions();

        PerformanceReport {
            timestamp: Instant::now(),
            statistics: stats_by_metric,
            regressions,
            total_measurements: measurements.values().map(|v| v.len()).sum(),
            monitoring_config: config.clone(),
        }
    }
}

/// Performance regression alert
#[derive(Debug, Clone)]
pub struct RegressionAlert {
    pub metric_type: MetricType,
    pub regression_percentage: f64,
    pub baseline_mean: f64,
    pub current_mean: f64,
    pub threshold: f64,
}

/// Performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub timestamp: Instant,
    pub statistics: HashMap<MetricType, Statistics>,
    pub regressions: Vec<RegressionAlert>,
    pub total_measurements: usize,
    pub monitoring_config: MonitoringConfig,
}

/// Global performance monitor instance
static GLOBAL_MONITOR: std::sync::OnceLock<PerformanceMonitor> = std::sync::OnceLock::new();

/// Initialize the global performance monitor
pub fn init_monitoring(config: MonitoringConfig) {
    GLOBAL_MONITOR
        .set(PerformanceMonitor::new(config))
        .map_err(|_| "Monitor already initialized")
        .unwrap();
}

/// Get the global performance monitor
pub fn get_monitor() -> &'static PerformanceMonitor {
    GLOBAL_MONITOR.get().unwrap_or_else(|| {
        init_monitoring(MonitoringConfig::default());
        GLOBAL_MONITOR.get().unwrap()
    })
}

/// Convenience macros for performance monitoring
///
/// Time a block of code
#[macro_export]
macro_rules! time_block {
    ($metric_type:expr, $context:expr, $block:block) => {
        $crate::performance_monitoring::get_monitor().time(
            $metric_type,
            Some($context.to_string()),
            || $block,
        )
    };
    ($metric_type:expr, $block:block) => {
        $crate::performance_monitoring::get_monitor().time($metric_type, None, || $block)
    };
}

/// Record a measurement
#[macro_export]
macro_rules! record_metric {
    ($metric_type:expr, $value:expr, $unit:expr) => {
        $crate::performance_monitoring::get_monitor().record(
            $crate::performance_monitoring::Measurement {
                metric_type: $metric_type,
                value: $value,
                unit: $unit,
                timestamp: std::time::Instant::now(),
                context: None,
            },
        );
    };
    ($metric_type:expr, $value:expr, $unit:expr, $context:expr) => {
        $crate::performance_monitoring::get_monitor().record(
            $crate::performance_monitoring::Measurement {
                metric_type: $metric_type,
                value: $value,
                unit: $unit,
                timestamp: std::time::Instant::now(),
                context: Some($context.to_string()),
            },
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_performance_monitor() {
        let config = MonitoringConfig {
            detailed_timing: true,
            ..MonitoringConfig::default()
        };
        let monitor = PerformanceMonitor::new(config);

        // Record some measurements
        monitor.record(Measurement {
            metric_type: MetricType::SecretCreation,
            value: 100.0,
            unit: Unit::Nanoseconds,
            timestamp: Instant::now(),
            context: Some("test".to_string()),
        });

        monitor.record(Measurement {
            metric_type: MetricType::SecretCreation,
            value: 150.0,
            unit: Unit::Nanoseconds,
            timestamp: Instant::now(),
            context: Some("test".to_string()),
        });

        // Get statistics
        let stats = monitor.get_statistics(&MetricType::SecretCreation).unwrap();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.min, 100.0);
        assert_eq!(stats.max, 150.0);
        assert_eq!(stats.mean, 125.0);
    }

    #[test]
    fn test_timing_function() {
        let config = MonitoringConfig {
            detailed_timing: true,
            ..MonitoringConfig::default()
        };
        let monitor = PerformanceMonitor::new(config);

        let result = monitor.time(
            MetricType::SecretCreation,
            Some("test_timing".to_string()),
            || {
                thread::sleep(Duration::from_millis(1));
                42
            },
        );

        assert_eq!(result, 42);

        let stats = monitor.get_statistics(&MetricType::SecretCreation).unwrap();
        assert_eq!(stats.count, 1);
        assert!(stats.mean > 0.0);
    }

    #[test]
    fn test_regression_detection() {
        let config = MonitoringConfig {
            regression_threshold: 10.0,
            detailed_timing: true,
            ..MonitoringConfig::default()
        };
        let monitor = PerformanceMonitor::new(config);

        // Set baseline
        let baseline = Statistics {
            count: 100,
            min: 90.0,
            max: 110.0,
            mean: 100.0,
            median: 100.0,
            std_dev: 5.0,
            p95: 108.0,
            p99: 109.0,
        };
        monitor.set_baseline(MetricType::SecretCreation, baseline);

        // Add measurements that exceed threshold
        for _ in 0..10 {
            monitor.record(Measurement {
                metric_type: MetricType::SecretCreation,
                value: 120.0, // 20% increase
                unit: Unit::Nanoseconds,
                timestamp: Instant::now(),
                context: None,
            });
        }

        let regressions = monitor.check_regressions();
        assert_eq!(regressions.len(), 1);
        assert!(regressions[0].regression_percentage > 10.0);
    }
}
