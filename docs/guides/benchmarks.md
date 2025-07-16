# Benchmarks

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Comprehensive benchmarking suite for measuring and optimizing wasm-sandbox performance across different scenarios and workloads.

## Benchmark Categories

### Core Performance Benchmarks

```rust
use criterion::{Criterion, BenchmarkId, Throughput};
use wasm_sandbox::{WasmSandbox, BenchmarkConfig};

pub fn bench_function_calls(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let sandbox = rt.block_on(async {
        WasmSandbox::builder()
            .source("benchmarks/simple_math.wasm")
            .build()
            .await
            .unwrap()
    });

    let mut group = c.benchmark_group("function_calls");
    
    // Single function call
    group.bench_function("add_two_numbers", |b| {
        b.to_async(&rt).iter(|| async {
            sandbox.call("add", (42i32, 24i32)).await.unwrap()
        })
    });

    // Batch function calls
    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_calls", batch_size),
            batch_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let mut futures = Vec::new();
                    for i in 0..size {
                        futures.push(sandbox.call("fibonacci", i as i32));
                    }
                    futures::future::join_all(futures).await
                })
            },
        );
    }
    
    group.finish();
}

pub fn bench_memory_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let sandbox = rt.block_on(async {
        WasmSandbox::builder()
            .source("benchmarks/memory_ops.wasm")
            .memory_limit(128 * 1024 * 1024) // 128MB
            .build()
            .await
            .unwrap()
    });

    let mut group = c.benchmark_group("memory_operations");
    group.throughput(Throughput::Bytes(1024 * 1024)); // 1MB

    group.bench_function("memory_allocation", |b| {
        b.to_async(&rt).iter(|| async {
            sandbox.call("allocate_memory", 1024 * 1024).await.unwrap()
        })
    });

    group.bench_function("memory_copy", |b| {
        b.to_async(&rt).iter(|| async {
            let data = vec![0u8; 1024 * 1024];
            sandbox.call("copy_memory", &data).await.unwrap()
        })
    });

    group.finish();
}

pub fn bench_compilation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let wasm_files = [
        ("small", "benchmarks/small_module.wasm"),
        ("medium", "benchmarks/medium_module.wasm"),
        ("large", "benchmarks/large_module.wasm"),
    ];

    let mut group = c.benchmark_group("compilation");
    
    for (name, file_path) in &wasm_files {
        let wasm_bytes = std::fs::read(file_path).unwrap();
        group.throughput(Throughput::Bytes(wasm_bytes.len() as u64));
        
        group.bench_with_input(
            BenchmarkId::new("compile_module", name),
            &wasm_bytes,
            |b, bytes| {
                b.to_async(&rt).iter(|| async {
                    let sandbox = WasmSandbox::builder()
                        .source_bytes(bytes)
                        .build()
                        .await
                        .unwrap();
                    drop(sandbox); // Ensure cleanup
                })
            },
        );
    }
    
    group.finish();
}
```

### Concurrency Benchmarks

```rust
pub fn bench_concurrent_execution(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrency");
    
    for num_instances in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_instances", num_instances),
            num_instances,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let futures: Vec<_> = (0..count)
                        .map(|_| async {
                            let sandbox = WasmSandbox::builder()
                                .source("benchmarks/cpu_intensive.wasm")
                                .build()
                                .await
                                .unwrap();
                            sandbox.call("compute", 10000).await.unwrap()
                        })
                        .collect();
                    futures::future::join_all(futures).await
                })
            },
        );
    }
    
    // Thread pool utilization
    group.bench_function("thread_pool_saturation", |b| {
        b.to_async(&rt).iter(|| async {
            let num_tasks = num_cpus::get() * 4;
            let futures: Vec<_> = (0..num_tasks)
                .map(|i| async move {
                    let sandbox = WasmSandbox::builder()
                        .source("benchmarks/parallel_work.wasm")
                        .build()
                        .await
                        .unwrap();
                    sandbox.call("parallel_task", i).await.unwrap()
                })
                .collect();
            futures::future::join_all(futures).await
        })
    });
    
    group.finish();
}

pub fn bench_resource_contention(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("resource_contention");
    
    // Memory contention
    group.bench_function("memory_contention", |b| {
        b.to_async(&rt).iter(|| async {
            let sandboxes: Vec<_> = (0..8)
                .map(|_| async {
                    WasmSandbox::builder()
                        .source("benchmarks/memory_heavy.wasm")
                        .memory_limit(32 * 1024 * 1024) // 32MB each
                        .build()
                        .await
                        .unwrap()
                })
                .collect();
            
            let sandboxes = futures::future::join_all(sandboxes).await;
            
            let futures: Vec<_> = sandboxes
                .iter()
                .map(|sandbox| sandbox.call("allocate_and_process", 16 * 1024 * 1024))
                .collect();
            
            futures::future::join_all(futures).await
        })
    });
    
    group.finish();
}
```

### Communication Benchmarks

```rust
pub fn bench_host_guest_communication(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("communication");
    
    // Data transfer sizes
    for size in [1024, 64 * 1024, 1024 * 1024].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("data_transfer", size),
            size,
            |b, &bytes| {
                b.to_async(&rt).iter(|| async {
                    let sandbox = WasmSandbox::builder()
                        .source("benchmarks/data_processor.wasm")
                        .build()
                        .await
                        .unwrap();
                    
                    let data = vec![0u8; bytes];
                    sandbox.call("process_data", &data).await.unwrap()
                })
            },
        );
    }
    
    // Serialization formats
    let test_data = generate_complex_test_data();
    
    for format in ["json", "messagepack", "bincode"].iter() {
        group.bench_with_input(
            BenchmarkId::new("serialization", format),
            format,
            |b, &fmt| {
                b.to_async(&rt).iter(|| async {
                    let sandbox = WasmSandbox::builder()
                        .source("benchmarks/serialization.wasm")
                        .serialization_format(fmt)
                        .build()
                        .await
                        .unwrap();
                    
                    sandbox.call("serialize_deserialize", &test_data).await.unwrap()
                })
            },
        );
    }
    
    group.finish();
}

pub fn bench_streaming_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("streaming");
    
    for chunk_size in [1024, 64 * 1024, 1024 * 1024].iter() {
        group.throughput(Throughput::Bytes(10 * 1024 * 1024)); // 10MB total
        
        group.bench_with_input(
            BenchmarkId::new("stream_processing", chunk_size),
            chunk_size,
            |b, &chunk_bytes| {
                b.to_async(&rt).iter(|| async {
                    let sandbox = StreamingSandbox::builder()
                        .source("benchmarks/stream_processor.wasm")
                        .buffer_size(chunk_bytes)
                        .build()
                        .await
                        .unwrap();
                    
                    let stream = generate_data_stream(10 * 1024 * 1024, chunk_bytes);
                    let mut processor = sandbox.create_stream_processor().await.unwrap();
                    
                    let output_stream = processor.process_stream(stream).await.unwrap();
                    let results: Vec<_> = output_stream.collect().await;
                    results
                })
            },
        );
    }
    
    group.finish();
}
```

### Security Benchmarks

```rust
pub fn bench_security_overhead(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("security");
    
    // Compare security levels
    let security_configs = [
        ("none", SecurityPolicy::permissive()),
        ("basic", SecurityPolicy::basic()),
        ("strict", SecurityPolicy::strict()),
        ("paranoid", SecurityPolicy::paranoid()),
    ];
    
    for (name, policy) in &security_configs {
        group.bench_with_input(
            BenchmarkId::new("security_overhead", name),
            policy,
            |b, policy| {
                b.to_async(&rt).iter(|| async {
                    let sandbox = WasmSandbox::builder()
                        .source("benchmarks/computation.wasm")
                        .security_policy(policy.clone())
                        .build()
                        .await
                        .unwrap();
                    
                    sandbox.call("compute_intensive", 1000).await.unwrap()
                })
            },
        );
    }
    
    // Capability checks
    group.bench_function("capability_checks", |b| {
        b.to_async(&rt).iter(|| async {
            let sandbox = WasmSandbox::builder()
                .source("benchmarks/capability_heavy.wasm")
                .security_policy(SecurityPolicy::strict())
                .build()
                .await
                .unwrap();
            
            // Function that performs many capability-checked operations
            sandbox.call("many_file_operations", 100).await.unwrap()
        })
    });
    
    group.finish();
}
```

### Specialized Benchmarks

```rust
pub fn bench_cold_start_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start");
    
    // Module compilation from scratch
    group.bench_function("compile_from_scratch", |b| {
        let wasm_bytes = std::fs::read("benchmarks/typical_app.wasm").unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        b.to_async(&rt).iter(|| async {
            WasmSandbox::builder()
                .source_bytes(&wasm_bytes)
                .disable_cache() // Force compilation
                .build()
                .await
                .unwrap()
        })
    });
    
    // With compilation cache
    group.bench_function("compile_with_cache", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        b.to_async(&rt).iter(|| async {
            WasmSandbox::builder()
                .source("benchmarks/typical_app.wasm")
                .enable_cache(true)
                .build()
                .await
                .unwrap()
        })
    });
    
    group.finish();
}

pub fn bench_garbage_collection(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("garbage_collection");
    
    group.bench_function("gc_pressure", |b| {
        b.to_async(&rt).iter(|| async {
            let sandbox = WasmSandbox::builder()
                .source("benchmarks/gc_heavy.wasm")
                .memory_limit(64 * 1024 * 1024) // 64MB
                .build()
                .await
                .unwrap();
            
            // Allocate and deallocate many objects
            sandbox.call("allocation_pressure", 10000).await.unwrap()
        })
    });
    
    group.finish();
}
```

## Benchmark Results Analysis

```rust
use std::collections::HashMap;

pub struct BenchmarkAnalyzer {
    historical_results: HashMap<String, Vec<BenchmarkResult>>,
    regression_threshold: f64,
}

impl BenchmarkAnalyzer {
    pub fn analyze_results(&self, current: &BenchmarkResults) -> AnalysisReport {
        let mut report = AnalysisReport::new();
        
        // Performance regression detection
        for (test_name, result) in &current.results {
            if let Some(historical) = self.historical_results.get(test_name) {
                let regression = self.detect_regression(result, historical);
                if let Some(reg) = regression {
                    report.regressions.push(reg);
                }
            }
        }
        
        // Performance improvements
        report.improvements = self.detect_improvements(current);
        
        // Outlier detection
        report.outliers = self.detect_outliers(current);
        
        // Resource utilization analysis
        report.resource_analysis = self.analyze_resource_usage(current);
        
        report
    }

    fn detect_regression(&self, current: &BenchmarkResult, historical: &[BenchmarkResult]) -> Option<RegressionAlert> {
        let historical_avg = historical.iter()
            .map(|r| r.execution_time.as_nanos() as f64)
            .sum::<f64>() / historical.len() as f64;
        
        let current_time = current.execution_time.as_nanos() as f64;
        let regression_ratio = current_time / historical_avg;
        
        if regression_ratio > (1.0 + self.regression_threshold) {
            Some(RegressionAlert {
                test_name: current.test_name.clone(),
                regression_percentage: (regression_ratio - 1.0) * 100.0,
                current_time: current.execution_time,
                historical_avg: Duration::from_nanos(historical_avg as u64),
                severity: if regression_ratio > 1.2 { 
                    Severity::High 
                } else { 
                    Severity::Medium 
                },
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct AnalysisReport {
    pub regressions: Vec<RegressionAlert>,
    pub improvements: Vec<ImprovementNote>,
    pub outliers: Vec<OutlierDetection>,
    pub resource_analysis: ResourceAnalysis,
    pub recommendations: Vec<PerformanceRecommendation>,
}

#[derive(Debug)]
pub struct RegressionAlert {
    pub test_name: String,
    pub regression_percentage: f64,
    pub current_time: Duration,
    pub historical_avg: Duration,
    pub severity: Severity,
}
```

## Continuous Benchmarking

```rust
pub struct ContinuousBenchmark {
    config: BenchmarkConfig,
    scheduler: BenchmarkScheduler,
    reporter: BenchmarkReporter,
}

impl ContinuousBenchmark {
    pub async fn run_scheduled_benchmarks(&mut self) -> Result<(), BenchmarkError> {
        let tests_to_run = self.scheduler.get_pending_tests().await?;
        
        for test_suite in tests_to_run {
            let results = self.run_benchmark_suite(&test_suite).await?;
            
            // Analyze results
            let analysis = self.analyzer.analyze_results(&results);
            
            // Report results
            self.reporter.report_results(&results, &analysis).await?;
            
            // Store historical data
            self.store_historical_results(&results).await?;
            
            // Trigger alerts if regressions detected
            if !analysis.regressions.is_empty() {
                self.trigger_regression_alerts(&analysis.regressions).await?;
            }
        }
        
        Ok(())
    }

    async fn run_benchmark_suite(&self, suite: &BenchmarkSuite) -> Result<BenchmarkResults, BenchmarkError> {
        let mut results = BenchmarkResults::new();
        
        for test in &suite.tests {
            let test_result = self.run_single_benchmark(test).await?;
            results.add_result(test.name.clone(), test_result);
        }
        
        Ok(results)
    }
}
```

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench function_calls

# Run benchmarks with custom configuration
cargo bench -- --output-format html --save-baseline main

# Compare with baseline
cargo bench -- --baseline main

# Generate benchmark report
cargo bench -- --plotting-backend plotters
```

Next: **[Examples](examples.md)** - Comprehensive usage examples

---

**Benchmark Excellence:** Comprehensive performance measurement and regression detection ensure optimal wasm-sandbox performance.
