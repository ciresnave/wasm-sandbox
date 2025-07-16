# Resource Management Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This guide covers how to effectively manage computational, memory, I/O, and network resources when running code in wasm-sandbox to ensure performance, stability, and security.

## Resource Management Philosophy

Effective resource management in wasm-sandbox serves three purposes:

1. **Security** - Prevent malicious code from consuming excessive resources
2. **Performance** - Ensure predictable performance under load
3. **Stability** - Maintain system stability with resource-intensive workloads

## Quick Start - Reasonable Defaults

```rust
use wasm_sandbox::WasmSandbox;
use std::time::Duration;

// Quick setup with common resource limits
let sandbox = WasmSandbox::builder()
    .source("my_program.rs")
    .memory_limit(64 * 1024 * 1024)        // 64MB
    .timeout_duration(Duration::from_secs(30)) // 30 seconds
    .build()
    .await?;
```

## Memory Management

### Understanding Memory Usage

WebAssembly memory consists of:

- **Linear Memory** - The main memory space for your program
- **Stack Memory** - Function call stack (part of linear memory)
- **Host Memory** - Memory used by the host runtime (outside sandbox)

### Setting Memory Limits

```rust
use wasm_sandbox::ResourceLimits;

let sandbox = WasmSandbox::builder()
    .source("memory_intensive.rs")
    .resource_limits(ResourceLimits {
        // Set maximum linear memory to 128MB
        memory_bytes: Some(128 * 1024 * 1024),
        
        // Optional: Limit memory growth rate
        memory_growth_limit: Some(16 * 1024 * 1024), // 16MB at a time
        
        ..ResourceLimits::default()
    })
    .build()
    .await?;
```

### Memory Usage Guidelines

**Typical Memory Requirements:**

```rust
// Small calculations and text processing
.memory_limit(16 * 1024 * 1024)    // 16MB

// Image processing or moderate datasets
.memory_limit(128 * 1024 * 1024)   // 128MB

// Large data processing or ML models
.memory_limit(512 * 1024 * 1024)   // 512MB

// Very large workloads (use with caution)
.memory_limit(2 * 1024 * 1024 * 1024) // 2GB
```

### Monitoring Memory Usage

```rust
use wasm_sandbox::{ResourceMonitor, MemoryUsage};

let sandbox = WasmSandbox::builder()
    .source("monitored_program.rs")
    .memory_limit(64 * 1024 * 1024)
    .enable_resource_monitoring(true)
    .resource_callback(|usage: ResourceUsage| {
        if let Some(memory) = usage.memory {
            println!("Memory: {} / {} bytes ({:.1}%)", 
                memory.used, 
                memory.limit, 
                (memory.used as f64 / memory.limit as f64) * 100.0
            );
            
            // Alert if memory usage is high
            if memory.used > memory.limit * 80 / 100 {
                println!("⚠️ High memory usage detected");
            }
        }
    })
    .build()
    .await?;
```

## Computational Resource Management

### Fuel System

Fuel provides deterministic execution limits independent of CPU speed:

```rust
let sandbox = WasmSandbox::builder()
    .source("compute_heavy.rs")
    .max_fuel(Some(10_000_000)) // 10 million instructions
    .build()
    .await?;
```

### Fuel Estimation Guidelines

```rust
// Simple arithmetic operations
.max_fuel(Some(1_000))         // 1K instructions

// String processing and small loops
.max_fuel(Some(100_000))       // 100K instructions

// File processing and medium algorithms
.max_fuel(Some(1_000_000))     // 1M instructions

// Complex algorithms and data processing
.max_fuel(Some(10_000_000))    // 10M instructions

// Machine learning and heavy computation
.max_fuel(Some(100_000_000))   // 100M instructions
```

### Time-Based Limits

For real-time applications, use time-based limits:

```rust
use std::time::Duration;

let sandbox = WasmSandbox::builder()
    .source("time_sensitive.rs")
    .timeout_duration(Duration::from_millis(100)) // 100ms for real-time
    .build()
    .await?;
```

### CPU Profiling

Monitor computational resource usage:

```rust
use wasm_sandbox::CpuUsage;

let result = sandbox.call_with_profiling("expensive_function", &input).await?;

println!("Execution took: {:?}", result.execution_time);
println!("Fuel consumed: {}", result.fuel_consumed);
println!("Instructions executed: {}", result.instruction_count);
```

## I/O Resource Management

### File System Limits

Control file operations to prevent DoS attacks:

```rust
use wasm_sandbox::IoLimits;

let sandbox = WasmSandbox::builder()
    .source("file_processor.rs")
    .resource_limits(ResourceLimits {
        // File size limits
        max_file_size: Some(100 * 1024 * 1024), // 100MB per file
        max_total_file_size: Some(1024 * 1024 * 1024), // 1GB total
        
        // File operation limits
        max_open_files: Some(100),           // 100 concurrent open files
        max_file_operations: Some(10_000),   // 10K file operations
        
        // Directory limits
        max_directory_entries: Some(1_000),  // 1K files per directory
        
        ..ResourceLimits::default()
    })
    .build()
    .await?;
```

### Monitoring File Operations

```rust
let sandbox = WasmSandbox::builder()
    .source("file_intensive.rs")
    .enable_io_monitoring(true)
    .io_callback(|operation: IoOperation| {
        match operation {
            IoOperation::FileRead { path, bytes } => {
                println!("Read {} bytes from {}", bytes, path);
            }
            IoOperation::FileWrite { path, bytes } => {
                println!("Wrote {} bytes to {}", bytes, path);
            }
            IoOperation::FileOpen { path } => {
                println!("Opened file: {}", path);
            }
        }
    })
    .build()
    .await?;
```

## Network Resource Management

### Connection Limits

Control network usage to prevent abuse:

```rust
use wasm_sandbox::NetworkLimits;

let sandbox = WasmSandbox::builder()
    .source("network_client.rs")
    .resource_limits(ResourceLimits {
        // Connection limits
        max_connections: Some(10),           // 10 concurrent connections
        max_requests_per_second: Some(100),  // Rate limiting
        
        // Data transfer limits
        max_upload_bytes: Some(10 * 1024 * 1024),   // 10MB upload
        max_download_bytes: Some(100 * 1024 * 1024), // 100MB download
        
        // Timeout settings
        network_timeout: Some(Duration::from_secs(30)), // 30 second timeout
        connect_timeout: Some(Duration::from_secs(10)),  // 10 second connect
        
        ..ResourceLimits::default()
    })
    .build()
    .await?;
```

### Bandwidth Monitoring

```rust
let sandbox = WasmSandbox::builder()
    .source("bandwidth_monitor.rs")
    .enable_network_monitoring(true)
    .network_callback(|stats: NetworkStats| {
        println!("Bytes sent: {}, received: {}", stats.bytes_sent, stats.bytes_received);
        println!("Active connections: {}", stats.active_connections);
        
        // Alert on high bandwidth usage
        if stats.bytes_received > 50 * 1024 * 1024 { // 50MB
            println!("⚠️ High bandwidth usage detected");
        }
    })
    .build()
    .await?;
```

## Resource Monitoring and Alerting

### Comprehensive Resource Monitoring

```rust
use wasm_sandbox::{ResourceMonitor, Alert, AlertLevel};

let monitor = ResourceMonitor::new()
    .memory_threshold(80) // Alert at 80% memory usage
    .cpu_threshold(90)    // Alert at 90% CPU usage
    .io_threshold(1000)   // Alert at 1000 IOPS
    .network_threshold(100 * 1024 * 1024) // Alert at 100MB/s
    .alert_callback(|alert: Alert| {
        match alert.level {
            AlertLevel::Warning => {
                println!("⚠️ Warning: {}", alert.message);
            }
            AlertLevel::Critical => {
                println!("🚨 Critical: {}", alert.message);
                // Take immediate action
            }
        }
    });

let sandbox = WasmSandbox::builder()
    .source("monitored_app.rs")
    .resource_monitor(monitor)
    .build()
    .await?;
```

### Resource Usage Reports

Generate detailed resource usage reports:

```rust
let result = sandbox.call_with_detailed_monitoring("process_data", &input).await?;

// Get comprehensive resource report
let report = result.resource_report;

println!("=== Resource Usage Report ===");
println!("Execution time: {:?}", report.execution_time);
println!("Peak memory: {} MB", report.peak_memory_bytes / 1024 / 1024);
println!("Fuel consumed: {}", report.fuel_consumed);
println!("File operations: {}", report.file_operations);
println!("Network requests: {}", report.network_requests);
println!("Bytes transferred: {}", report.bytes_transferred);
```

## Performance Optimization

### Memory Optimization

```rust
// Use memory-efficient data structures in your WebAssembly code
#[no_mangle]
pub extern "C" fn efficient_processing() {
    // Prefer streaming over loading everything into memory
    // Use Vec::with_capacity() when size is known
    // Release memory with drop() when no longer needed
}
```

### Computational Optimization

```rust
// Optimize for fuel efficiency
let sandbox = WasmSandbox::builder()
    .source("optimized_code.rs")
    .max_fuel(Some(1_000_000))
    .optimization_level(OptimizationLevel::Speed) // Optimize for speed over size
    .build()
    .await?;
```

### I/O Optimization

```rust
// Batch file operations for efficiency
let sandbox = WasmSandbox::builder()
    .source("batch_processor.rs")
    .resource_limits(ResourceLimits {
        io_batch_size: Some(64),  // Batch I/O operations
        buffer_size: Some(64 * 1024), // 64KB I/O buffers
        ..ResourceLimits::default()
    })
    .build()
    .await?;
```

## Scaling and Load Management

### Auto-scaling Resources

```rust
use wasm_sandbox::{AutoScaler, ScalingPolicy};

let auto_scaler = AutoScaler::new()
    .memory_policy(ScalingPolicy {
        min_limit: 16 * 1024 * 1024,   // 16MB minimum
        max_limit: 512 * 1024 * 1024,  // 512MB maximum
        scale_up_threshold: 80,         // Scale up at 80% usage
        scale_down_threshold: 30,       // Scale down at 30% usage
    })
    .timeout_policy(ScalingPolicy {
        min_timeout: Duration::from_secs(10),   // 10 second minimum
        max_timeout: Duration::from_secs(300),  // 5 minute maximum
        adjustment_factor: 1.5,                 // 50% increase/decrease
    });

let sandbox = WasmSandbox::builder()
    .source("adaptive_app.rs")
    .auto_scaler(auto_scaler)
    .build()
    .await?;
```

### Load Balancing

```rust
use wasm_sandbox::LoadBalancer;

let load_balancer = LoadBalancer::new()
    .max_concurrent_executions(100)
    .queue_size(1000)
    .timeout_duration(Duration::from_secs(60))
    .overflow_strategy(OverflowStrategy::DropOldest);

// Distribute load across multiple sandbox instances
let result = load_balancer.execute("heavy_computation", &input).await?;
```

## Resource Profiles

### Predefined Resource Profiles

```rust
use wasm_sandbox::ResourceProfile;

// Lightweight profile for simple tasks
let sandbox = WasmSandbox::builder()
    .source("simple_task.rs")
    .resource_profile(ResourceProfile::Lightweight)
    .build()
    .await?;

// Standard profile for typical applications
let sandbox = WasmSandbox::builder()
    .source("standard_app.rs")
    .resource_profile(ResourceProfile::Standard)
    .build()
    .await?;

// High-performance profile for intensive workloads
let sandbox = WasmSandbox::builder()
    .source("intensive_app.rs")
    .resource_profile(ResourceProfile::HighPerformance)
    .build()
    .await?;
```

### Custom Resource Profiles

```rust
let custom_profile = ResourceProfile::custom()
    .memory_mb(256)
    .timeout_seconds(120)
    .fuel_limit(50_000_000)
    .max_file_size_mb(200)
    .max_connections(20)
    .build();

let sandbox = WasmSandbox::builder()
    .source("custom_app.rs")
    .resource_profile(custom_profile)
    .build()
    .await?;
```

## Troubleshooting Resource Issues

### Common Resource Errors

**Memory Limit Exceeded:**

```text
Error: Memory limit exceeded: used 134217728, limit 67108864
```

**Solutions:**

1. Increase memory limit
2. Optimize memory usage in code
3. Use streaming instead of loading all data

```rust
// Increase memory limit
.memory_limit(256 * 1024 * 1024)

// Or optimize code to use less memory
```

**Timeout Exceeded:**

```text
Error: Execution timeout after 30 seconds
```

**Solutions:**

1. Increase timeout duration
2. Optimize algorithm complexity
3. Break work into smaller chunks

```rust
// Increase timeout
.timeout_duration(Duration::from_secs(120))

// Or implement chunked processing
```

**Fuel Exhausted:**

```text
Error: Fuel exhausted: used 1000000, limit 1000000
```

**Solutions:**

1. Increase fuel limit
2. Optimize computational complexity
3. Use more efficient algorithms

```rust
// Increase fuel limit
.max_fuel(Some(10_000_000))
```

### Resource Debugging

Enable detailed resource debugging:

```rust
let sandbox = WasmSandbox::builder()
    .source("debug_resources.rs")
    .debug_mode(true)
    .verbose_resource_logging(true)
    .build()
    .await?;
```

## Resource Management Best Practices

### 1. Start Conservative, Scale Up

```rust
// Start with conservative limits
let initial_limits = ResourceLimits {
    memory_bytes: Some(32 * 1024 * 1024),  // 32MB
    execution_timeout: Some(Duration::from_secs(10)), // 10 seconds
    max_fuel: Some(1_000_000), // 1M instructions
    ..ResourceLimits::default()
};

// Monitor and adjust based on actual usage
```

### 2. Monitor and Alert

```rust
// Always enable monitoring for production
.enable_resource_monitoring(true)
.alert_on_high_usage(true)
.resource_callback(|usage| {
    // Log and alert on resource issues
})
```

### 3. Plan for Peak Load

```rust
// Set limits based on peak expected load, not average
let peak_limits = ResourceLimits {
    memory_bytes: Some(128 * 1024 * 1024), // Plan for 4x average usage
    max_connections: Some(50),              // Handle traffic spikes
    ..ResourceLimits::default()
};
```

### 4. Test Resource Limits

```rust
#[tokio::test]
async fn test_memory_limits() {
    let sandbox = WasmSandbox::builder()
        .source("memory_hog.rs")
        .memory_limit(16 * 1024 * 1024) // 16MB limit
        .build()
        .await?;
    
    // This should fail with memory limit exceeded
    let result = sandbox.call("allocate_100mb", &()).await;
    assert!(matches!(result, Err(Error::MemoryLimit { .. })));
}
```

## Performance Benchmarking

### Benchmark Resource Usage

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_resource_usage(c: &mut Criterion) {
    c.bench_function("file_processing", |b| {
        b.iter(|| {
            let sandbox = WasmSandbox::builder()
                .source("file_processor.rs")
                .memory_limit(64 * 1024 * 1024)
                .build()
                .await?;
            
            sandbox.call("process_file", &test_data).await
        })
    });
}

criterion_group!(benches, benchmark_resource_usage);
criterion_main!(benches);
```

## Next Steps

- **[Security Configuration](security-config.md)** - Secure your resource limits
- **[Production Deployment](production.md)** - Deploy with appropriate resources
- **[Performance Guide](../design/performance.md)** - Optimize for performance
- **[Plugin Development](plugin-development.md)** - Build resource-aware plugins

---

**Remember:** Resource management is about finding the right balance between security, performance, and functionality for your specific use case.
