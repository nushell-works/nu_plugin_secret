use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nu_plugin_secret::*;
use nu_protocol::{Record, Span, Value};

/// Benchmark secret string operations
fn bench_secret_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("secret_string");
    
    // Test different string sizes
    let sizes = vec![10, 100, 1000, 10000];
    
    for size in sizes {
        let test_string = "a".repeat(size);
        
        group.throughput(Throughput::Bytes(size as u64));
        
        // Benchmark creation
        group.bench_with_input(
            BenchmarkId::new("creation", size),
            &test_string,
            |b, s| {
                b.iter(|| SecretString::new(s.clone()))
            },
        );
        
        // Benchmark reveal (unwrap)
        let secret = SecretString::new(test_string.clone());
        group.bench_with_input(
            BenchmarkId::new("reveal", size),
            &secret,
            |b, s| {
                b.iter(|| s.reveal())
            },
        );
        
        // Benchmark clone
        group.bench_with_input(
            BenchmarkId::new("clone", size),
            &secret,
            |b, s| {
                b.iter(|| s.clone())
            },
        );
        
        // Benchmark display (redacted)
        group.bench_with_input(
            BenchmarkId::new("display", size),
            &secret,
            |b, s| {
                b.iter(|| format!("{}", s))
            },
        );
    }
    
    group.finish();
}

/// Benchmark secret integer operations
fn bench_secret_int(c: &mut Criterion) {
    let mut group = c.benchmark_group("secret_int");
    
    let test_values = vec![0i64, 42, -42, i64::MAX, i64::MIN];
    
    for value in test_values {
        // Benchmark creation
        group.bench_with_input(
            BenchmarkId::new("creation", value),
            &value,
            |b, v| {
                b.iter(|| SecretInt::new(*v))
            },
        );
        
        // Benchmark reveal
        let secret = SecretInt::new(value);
        group.bench_with_input(
            BenchmarkId::new("reveal", value),
            &secret,
            |b, s| {
                b.iter(|| s.reveal())
            },
        );
    }
    
    group.finish();
}

/// Benchmark secret record operations
fn bench_secret_record(c: &mut Criterion) {
    let mut group = c.benchmark_group("secret_record");
    
    // Test different record sizes
    let sizes = vec![1, 5, 10, 50];
    
    for size in sizes {
        let mut record = Record::new();
        for i in 0..size {
            record.insert(
                format!("key_{}", i),
                Value::string(format!("value_{}", i), Span::test_data()),
            );
        }
        
        group.throughput(Throughput::Elements(size as u64));
        
        // Benchmark creation
        group.bench_with_input(
            BenchmarkId::new("creation", size),
            &record,
            |b, r| {
                b.iter(|| SecretRecord::new(r.clone()))
            },
        );
        
        // Benchmark reveal
        let secret = SecretRecord::new(record.clone());
        group.bench_with_input(
            BenchmarkId::new("reveal", size),
            &secret,
            |b, s| {
                b.iter(|| s.reveal())
            },
        );
    }
    
    group.finish();
}

/// Benchmark secret list operations
fn bench_secret_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("secret_list");
    
    // Test different list sizes
    let sizes = vec![1, 10, 100, 1000];
    
    for size in sizes {
        let list: Vec<Value> = (0..size)
            .map(|i| Value::string(format!("item_{}", i), Span::test_data()))
            .collect();
        
        group.throughput(Throughput::Elements(size as u64));
        
        // Benchmark creation
        group.bench_with_input(
            BenchmarkId::new("creation", size),
            &list,
            |b, l| {
                b.iter(|| SecretList::new(l.clone()))
            },
        );
        
        // Benchmark reveal
        let secret = SecretList::new(list.clone());
        group.bench_with_input(
            BenchmarkId::new("reveal", size),
            &secret,
            |b, s| {
                b.iter(|| s.reveal())
            },
        );
    }
    
    group.finish();
}

/// Benchmark secret binary operations
fn bench_secret_binary(c: &mut Criterion) {
    let mut group = c.benchmark_group("secret_binary");
    
    // Test different binary sizes
    let sizes = vec![16, 256, 1024, 4096];
    
    for size in sizes {
        let data = vec![0xaa; size];
        
        group.throughput(Throughput::Bytes(size as u64));
        
        // Benchmark creation
        group.bench_with_input(
            BenchmarkId::new("creation", size),
            &data,
            |b, d| {
                b.iter(|| SecretBinary::new(d.clone()))
            },
        );
        
        // Benchmark reveal
        let secret = SecretBinary::new(data.clone());
        group.bench_with_input(
            BenchmarkId::new("reveal", size),
            &secret,
            |b, s| {
                b.iter(|| s.reveal())
            },
        );
        
        // Benchmark length check (safe operation)
        group.bench_with_input(
            BenchmarkId::new("length", size),
            &secret,
            |b, s| {
                b.iter(|| s.len())
            },
        );
    }
    
    group.finish();
}

/// Benchmark secret float operations
fn bench_secret_float(c: &mut Criterion) {
    let mut group = c.benchmark_group("secret_float");
    
    let test_values = vec![0.0, 3.14159, -3.14159, f64::MAX, f64::MIN, f64::NAN, f64::INFINITY];
    
    for (i, value) in test_values.iter().enumerate() {
        // Benchmark creation
        group.bench_with_input(
            BenchmarkId::new("creation", i),
            value,
            |b, v| {
                b.iter(|| SecretFloat::new(*v))
            },
        );
        
        // Benchmark reveal
        let secret = SecretFloat::new(*value);
        group.bench_with_input(
            BenchmarkId::new("reveal", i),
            &secret,
            |b, s| {
                b.iter(|| s.reveal())
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory safety operations (Drop performance)
fn bench_memory_safety(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_safety");
    
    // Benchmark bulk creation and cleanup
    let sizes = vec![100, 1000, 10000];
    
    for size in sizes {
        group.bench_with_input(
            BenchmarkId::new("bulk_string_cleanup", size),
            &size,
            |b, &s| {
                b.iter(|| {
                    let mut secrets = Vec::new();
                    for i in 0..s {
                        secrets.push(SecretString::new(format!("secret_{}", i)));
                    }
                    // Implicit drop of all secrets
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark serialization performance (for plugin communication)
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    let secret_string = SecretString::new("test_secret".to_string());
    let secret_int = SecretInt::new(42);
    
    // Benchmark bincode serialization
    group.bench_function("string_serialize", |b| {
        b.iter(|| bincode::serialize(&secret_string).unwrap())
    });
    
    group.bench_function("int_serialize", |b| {
        b.iter(|| bincode::serialize(&secret_int).unwrap())
    });
    
    // Benchmark deserialization
    let serialized_string = bincode::serialize(&secret_string).unwrap();
    let serialized_int = bincode::serialize(&secret_int).unwrap();
    
    group.bench_function("string_deserialize", |b| {
        b.iter(|| {
            let _: SecretString = bincode::deserialize(&serialized_string).unwrap();
        })
    });
    
    group.bench_function("int_deserialize", |b| {
        b.iter(|| {
            let _: SecretInt = bincode::deserialize(&serialized_int).unwrap();
        })
    });
    
    group.finish();
}

/// Benchmark comparison with regular types (overhead measurement)
fn bench_overhead_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("overhead_comparison");
    
    let test_string = "performance_test_string_1234567890".to_string();
    let test_int = 42i64;
    
    // Regular string operations
    group.bench_function("regular_string_clone", |b| {
        b.iter(|| test_string.clone())
    });
    
    group.bench_function("regular_string_display", |b| {
        b.iter(|| format!("{}", test_string))
    });
    
    // Secret string operations
    let secret_string = SecretString::new(test_string.clone());
    group.bench_function("secret_string_clone", |b| {
        b.iter(|| secret_string.clone())
    });
    
    group.bench_function("secret_string_display", |b| {
        b.iter(|| format!("{}", secret_string))
    });
    
    // Regular int operations
    group.bench_function("regular_int_clone", |b| {
        b.iter(|| test_int.clone())
    });
    
    // Secret int operations
    let secret_int = SecretInt::new(test_int);
    group.bench_function("secret_int_clone", |b| {
        b.iter(|| secret_int.clone())
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_secret_string,
    bench_secret_int,
    bench_secret_record,
    bench_secret_list,
    bench_secret_binary,
    bench_secret_float,
    bench_memory_safety,
    bench_serialization,
    bench_overhead_comparison
);

criterion_main!(benches);