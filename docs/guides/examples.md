# Examples

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Comprehensive collection of practical examples demonstrating wasm-sandbox capabilities across different use cases and scenarios.

## Basic Usage Examples

### Hello World Sandbox

```rust
use wasm_sandbox::{WasmSandbox, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a simple sandbox
    let sandbox = WasmSandbox::builder()
        .source("examples/hello_world.wasm")
        .build()
        .await?;

    // Call a function
    let result: String = sandbox.call("greet", "World").await?;
    println!("Result: {}", result); // "Hello, World!"

    Ok(())
}
```

### Mathematical Operations

```rust
use wasm_sandbox::{WasmSandbox, SecurityPolicy};

#[tokio::main]
async fn main() -> Result<()> {
    let sandbox = WasmSandbox::builder()
        .source("examples/math_lib.wasm")
        .security_policy(SecurityPolicy::basic())
        .build()
        .await?;

    // Basic arithmetic
    let sum: i32 = sandbox.call("add", (42, 24)).await?;
    let product: i32 = sandbox.call("multiply", (7, 6)).await?;
    
    // Complex operations
    let fibonacci: Vec<i32> = sandbox.call("fibonacci_sequence", 10).await?;
    let factorial: u64 = sandbox.call("factorial", 12).await?;
    
    println!("Sum: {}, Product: {}", sum, product);
    println!("Fibonacci: {:?}", fibonacci);
    println!("Factorial of 12: {}", factorial);

    Ok(())
}
```

## Data Processing Examples

### JSON Data Processing

```rust
use serde::{Deserialize, Serialize};
use wasm_sandbox::{WasmSandbox, SerializationFormat};

#[derive(Serialize, Deserialize, Debug)]
struct UserData {
    id: u32,
    name: String,
    email: String,
    age: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProcessedData {
    user_count: usize,
    average_age: f64,
    domains: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let sandbox = WasmSandbox::builder()
        .source("examples/data_processor.wasm")
        .serialization_format(SerializationFormat::Json)
        .build()
        .await?;

    let users = vec![
        UserData {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        },
        UserData {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@gmail.com".to_string(),
            age: 30,
        },
        // ... more users
    ];

    // Process user data
    let processed: ProcessedData = sandbox.call("process_users", &users).await?;
    println!("Processed data: {:#?}", processed);

    // Filter users by criteria
    let adults: Vec<UserData> = sandbox.call("filter_adults", &users).await?;
    println!("Adult users: {}", adults.len());

    Ok(())
}
```

### File Processing Pipeline

```rust
use std::path::PathBuf;
use wasm_sandbox::{WasmSandbox, Capability, SecurityPolicy};

#[tokio::main]
async fn main() -> Result<()> {
    let mut policy = SecurityPolicy::basic();
    policy.add_capability(Capability::FileRead("/input".into()));
    policy.add_capability(Capability::FileWrite("/output".into()));

    let sandbox = WasmSandbox::builder()
        .source("examples/file_processor.wasm")
        .security_policy(policy)
        .mount_directory("/input", "./input_files")
        .mount_directory("/output", "./output_files")
        .build()
        .await?;

    // Process all files in a directory
    let file_list = vec![
        "data1.csv".to_string(),
        "data2.csv".to_string(),
        "data3.csv".to_string(),
    ];

    for filename in file_list {
        let result: String = sandbox.call("process_csv_file", &filename).await?;
        println!("Processed {}: {}", filename, result);
    }

    // Batch processing
    let summary: String = sandbox.call("generate_summary", ()).await?;
    println!("Processing summary: {}", summary);

    Ok(())
}
```

## Web Service Examples

### HTTP Server Integration

```rust
use axum::{extract::Query, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower::ServiceBuilder;
use wasm_sandbox::{WasmSandbox, SecurityPolicy};

#[derive(Deserialize)]
struct ProcessRequest {
    algorithm: String,
    data: serde_json::Value,
    parameters: HashMap<String, String>,
}

#[derive(Serialize)]
struct ProcessResponse {
    result: serde_json::Value,
    execution_time_ms: u64,
    memory_used: u64,
}

async fn process_data(
    Json(request): Json<ProcessRequest>,
) -> Result<Json<ProcessResponse>, String> {
    let start_time = std::time::Instant::now();
    
    // Create sandbox with appropriate algorithm
    let wasm_file = format!("algorithms/{}.wasm", request.algorithm);
    let sandbox = WasmSandbox::builder()
        .source(&wasm_file)
        .security_policy(SecurityPolicy::strict())
        .memory_limit(64 * 1024 * 1024) // 64MB
        .cpu_timeout(Duration::from_secs(30))
        .build()
        .await
        .map_err(|e| format!("Failed to create sandbox: {}", e))?;

    // Execute algorithm
    let result = sandbox
        .call("process", (&request.data, &request.parameters))
        .await
        .map_err(|e| format!("Execution failed: {}", e))?;

    let execution_time = start_time.elapsed();
    let memory_used = sandbox.memory_usage().await.unwrap_or(0);

    Ok(Json(ProcessResponse {
        result,
        execution_time_ms: execution_time.as_millis() as u64,
        memory_used,
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/process", post(process_data))
        .layer(ServiceBuilder::new().layer(tower_http::cors::CorsLayer::permissive()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
```

### Microservice Architecture

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use wasm_sandbox::{SandboxPool, WasmSandbox};

pub struct MicroserviceHandler {
    sandbox_pool: Arc<SandboxPool>,
    metrics: Arc<RwLock<ServiceMetrics>>,
}

impl MicroserviceHandler {
    pub async fn new(service_name: &str, max_instances: usize) -> Result<Self> {
        let pool = SandboxPool::builder()
            .source(format!("services/{}.wasm", service_name))
            .max_instances(max_instances)
            .idle_timeout(Duration::from_secs(300))
            .security_policy(SecurityPolicy::microservice())
            .build()
            .await?;

        Ok(Self {
            sandbox_pool: Arc::new(pool),
            metrics: Arc::new(RwLock::new(ServiceMetrics::new())),
        })
    }

    pub async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse> {
        let start_time = Instant::now();
        
        // Get sandbox from pool
        let sandbox = self.sandbox_pool.acquire().await?;
        
        // Process request
        let result = sandbox.call("handle_request", &request).await?;
        
        // Update metrics
        let duration = start_time.elapsed();
        let mut metrics = self.metrics.write().await;
        metrics.record_request(duration, true);
        
        // Return sandbox to pool
        self.sandbox_pool.release(sandbox).await?;
        
        Ok(result)
    }
}

#[derive(Clone)]
pub struct ServiceMetrics {
    total_requests: u64,
    successful_requests: u64,
    total_duration: Duration,
    max_duration: Duration,
}

impl ServiceMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            total_duration: Duration::ZERO,
            max_duration: Duration::ZERO,
        }
    }

    pub fn record_request(&mut self, duration: Duration, success: bool) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        }
        self.total_duration += duration;
        if duration > self.max_duration {
            self.max_duration = duration;
        }
    }

    pub fn average_duration(&self) -> Duration {
        if self.total_requests > 0 {
            self.total_duration / self.total_requests as u32
        } else {
            Duration::ZERO
        }
    }
}
```

## Plugin System Examples

### Dynamic Plugin Loading

```rust
use std::collections::HashMap;
use wasm_sandbox::{WasmSandbox, PluginManager, PluginMetadata};

pub struct ApplicationCore {
    plugins: HashMap<String, WasmSandbox>,
    plugin_manager: PluginManager,
}

impl ApplicationCore {
    pub async fn new() -> Result<Self> {
        let plugin_manager = PluginManager::new("./plugins");
        
        Ok(Self {
            plugins: HashMap::new(),
            plugin_manager,
        })
    }

    pub async fn load_plugin(&mut self, plugin_name: &str) -> Result<()> {
        let plugin_path = format!("./plugins/{}.wasm", plugin_name);
        
        // Load plugin metadata
        let metadata = self.plugin_manager.load_metadata(&plugin_path).await?;
        self.validate_plugin_compatibility(&metadata)?;
        
        // Create sandbox for plugin
        let sandbox = WasmSandbox::builder()
            .source(&plugin_path)
            .security_policy(SecurityPolicy::plugin())
            .memory_limit(metadata.memory_limit)
            .add_host_function("log", self.create_log_function())
            .add_host_function("get_config", self.create_config_function())
            .build()
            .await?;

        // Initialize plugin
        sandbox.call("initialize", &metadata.config).await?;
        
        self.plugins.insert(plugin_name.to_string(), sandbox);
        println!("Plugin '{}' loaded successfully", plugin_name);
        
        Ok(())
    }

    pub async fn execute_plugin(&self, plugin_name: &str, command: &str, args: &serde_json::Value) -> Result<serde_json::Value> {
        let sandbox = self.plugins.get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not loaded", plugin_name))?;

        let result = sandbox.call("execute", (command, args)).await?;
        Ok(result)
    }

    fn create_log_function(&self) -> impl Fn(&str) -> Result<()> {
        |message: &str| {
            println!("[PLUGIN] {}", message);
            Ok(())
        }
    }

    fn create_config_function(&self) -> impl Fn(&str) -> Result<serde_json::Value> {
        |key: &str| {
            // Return configuration value for the given key
            match key {
                "api_endpoint" => Ok(serde_json::json!("https://api.example.com")),
                "max_retries" => Ok(serde_json::json!(3)),
                _ => Ok(serde_json::Value::Null),
            }
        }
    }

    fn validate_plugin_compatibility(&self, metadata: &PluginMetadata) -> Result<()> {
        if metadata.api_version != "1.0" {
            return Err(format!("Incompatible API version: {}", metadata.api_version).into());
        }
        
        if metadata.min_memory > 128 * 1024 * 1024 {
            return Err("Plugin requires too much memory".into());
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = ApplicationCore::new().await?;
    
    // Load plugins
    app.load_plugin("image_processor").await?;
    app.load_plugin("data_validator").await?;
    app.load_plugin("custom_formatter").await?;
    
    // Use plugins
    let image_result = app.execute_plugin(
        "image_processor",
        "resize",
        &serde_json::json!({
            "input": "/path/to/image.jpg",
            "width": 800,
            "height": 600
        })
    ).await?;
    
    let validation_result = app.execute_plugin(
        "data_validator",
        "validate_email",
        &serde_json::json!("user@example.com")
    ).await?;
    
    println!("Image processing result: {:?}", image_result);
    println!("Validation result: {:?}", validation_result);
    
    Ok(())
}
```

## Advanced Streaming Examples

### Large File Processing

```rust
use futures::StreamExt;
use wasm_sandbox::{StreamingSandbox, StreamProcessor};

#[tokio::main]
async fn main() -> Result<()> {
    let sandbox = StreamingSandbox::builder()
        .source("examples/file_stream_processor.wasm")
        .buffer_size(64 * 1024) // 64KB chunks
        .build()
        .await?;

    // Process a large file in chunks
    let input_file = tokio::fs::File::open("large_dataset.csv").await?;
    let input_stream = tokio_util::codec::FramedRead::new(
        input_file,
        tokio_util::codec::LinesCodec::new()
    );

    let mut processor = sandbox.create_stream_processor().await?;
    
    // Transform each line
    let output_stream = processor.process_stream(
        input_stream.map(|line| {
            line.map_err(|e| e.into())
        })
    ).await?;

    // Collect results and write to output file
    let output_file = tokio::fs::File::create("processed_dataset.json").await?;
    let mut writer = tokio::io::BufWriter::new(output_file);
    
    pin_mut!(output_stream);
    while let Some(result) = output_stream.next().await {
        let processed_line = result?;
        tokio::io::AsyncWriteExt::write_all(&mut writer, processed_line.as_bytes()).await?;
        tokio::io::AsyncWriteExt::write_all(&mut writer, b"\n").await?;
    }
    
    tokio::io::AsyncWriteExt::flush(&mut writer).await?;
    println!("File processing completed");
    
    Ok(())
}
```

### Real-time Data Processing

```rust
use tokio_stream::wrappers::ReceiverStream;
use wasm_sandbox::{StreamingSandbox, StreamConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up streaming sandbox
    let sandbox = StreamingSandbox::builder()
        .source("examples/realtime_processor.wasm")
        .stream_config(StreamConfig {
            buffer_size: 1024,
            max_buffered_items: 100,
            backpressure_threshold: 80,
        })
        .build()
        .await?;

    // Create data source (simulating real-time sensor data)
    let (tx, rx) = tokio::sync::mpsc::channel(1000);
    let input_stream = ReceiverStream::new(rx);
    
    // Start data generator
    tokio::spawn(async move {
        let mut counter = 0;
        loop {
            let sensor_data = SensorReading {
                timestamp: chrono::Utc::now(),
                sensor_id: format!("sensor_{}", counter % 10),
                value: fastrand::f64() * 100.0,
                unit: "celsius".to_string(),
            };
            
            if tx.send(sensor_data).await.is_err() {
                break;
            }
            
            counter += 1;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    // Process stream
    let mut processor = sandbox.create_stream_processor().await?;
    let output_stream = processor.process_stream(input_stream).await?;

    // Handle processed results
    pin_mut!(output_stream);
    while let Some(result) = output_stream.next().await {
        match result {
            Ok(processed_data) => {
                println!("Processed: {:?}", processed_data);
                // Send to next stage, database, etc.
            }
            Err(e) => {
                eprintln!("Processing error: {}", e);
                // Handle error, maybe retry or log
            }
        }
    }
    
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct SensorReading {
    timestamp: chrono::DateTime<chrono::Utc>,
    sensor_id: String,
    value: f64,
    unit: String,
}
```

## Security Examples

### Sandboxed Script Execution

```rust
use wasm_sandbox::{WasmSandbox, SecurityPolicy, Capability};

#[tokio::main]
async fn main() -> Result<()> {
    // Create restrictive security policy
    let mut policy = SecurityPolicy::strict();
    policy.add_capability(Capability::NetworkAccess {
        allowed_hosts: vec!["api.example.com".to_string()],
        allowed_ports: vec![443],
    });
    policy.add_capability(Capability::FileRead("/data".into()));
    
    // Add resource limits
    policy.set_memory_limit(32 * 1024 * 1024); // 32MB
    policy.set_cpu_timeout(Duration::from_secs(10));
    policy.set_max_file_operations(100);

    let sandbox = WasmSandbox::builder()
        .source("untrusted_scripts/user_script.wasm")
        .security_policy(policy)
        .audit_enabled(true)
        .build()
        .await?;

    // Execute untrusted code with monitoring
    let result = sandbox.call_with_audit("main", &serde_json::json!({
        "input_data": "user provided data",
        "options": {
            "format": "json",
            "validate": true
        }
    })).await?;

    println!("Execution result: {:?}", result.output);
    println!("Security audit: {:?}", result.audit_log);
    
    // Check for policy violations
    if !result.audit_log.violations.is_empty() {
        println!("Security violations detected:");
        for violation in &result.audit_log.violations {
            println!("  - {}: {}", violation.severity, violation.message);
        }
    }

    Ok(())
}
```

### Multi-tenant Execution

```rust
use std::collections::HashMap;
use wasm_sandbox::{WasmSandbox, TenantIsolation, ResourceQuota};

pub struct MultiTenantRunner {
    tenant_sandboxes: HashMap<String, WasmSandbox>,
    tenant_quotas: HashMap<String, ResourceQuota>,
}

impl MultiTenantRunner {
    pub async fn new() -> Self {
        Self {
            tenant_sandboxes: HashMap::new(),
            tenant_quotas: HashMap::new(),
        }
    }

    pub async fn register_tenant(&mut self, tenant_id: &str, script_path: &str, quota: ResourceQuota) -> Result<()> {
        let isolation = TenantIsolation::builder()
            .tenant_id(tenant_id)
            .resource_quota(quota.clone())
            .network_isolation(true)
            .filesystem_isolation(true)
            .build();

        let sandbox = WasmSandbox::builder()
            .source(script_path)
            .tenant_isolation(isolation)
            .build()
            .await?;

        self.tenant_sandboxes.insert(tenant_id.to_string(), sandbox);
        self.tenant_quotas.insert(tenant_id.to_string(), quota);
        
        Ok(())
    }

    pub async fn execute_for_tenant(&self, tenant_id: &str, function: &str, args: &serde_json::Value) -> Result<serde_json::Value> {
        let sandbox = self.tenant_sandboxes.get(tenant_id)
            .ok_or_else(|| format!("Tenant '{}' not registered", tenant_id))?;

        // Check quota before execution
        let current_usage = sandbox.get_resource_usage().await?;
        let quota = &self.tenant_quotas[tenant_id];
        
        if current_usage.memory > quota.max_memory {
            return Err("Memory quota exceeded".into());
        }
        
        if current_usage.cpu_time > quota.max_cpu_time {
            return Err("CPU time quota exceeded".into());
        }

        // Execute with tenant context
        let result = sandbox.call(function, args).await?;
        Ok(result)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = MultiTenantRunner::new().await;
    
    // Register tenants with different quotas
    runner.register_tenant(
        "tenant_a",
        "scripts/tenant_a_script.wasm",
        ResourceQuota {
            max_memory: 64 * 1024 * 1024, // 64MB
            max_cpu_time: Duration::from_secs(30),
            max_network_requests: 100,
            max_file_operations: 50,
        }
    ).await?;
    
    runner.register_tenant(
        "tenant_b", 
        "scripts/tenant_b_script.wasm",
        ResourceQuota {
            max_memory: 32 * 1024 * 1024, // 32MB  
            max_cpu_time: Duration::from_secs(10),
            max_network_requests: 50,
            max_file_operations: 25,
        }
    ).await?;

    // Execute code for different tenants
    let result_a = runner.execute_for_tenant(
        "tenant_a",
        "process_data",
        &serde_json::json!({"data": "large dataset"})
    ).await?;
    
    let result_b = runner.execute_for_tenant(
        "tenant_b",
        "quick_calculation", 
        &serde_json::json!({"numbers": [1, 2, 3, 4, 5]})
    ).await?;
    
    println!("Tenant A result: {:?}", result_a);
    println!("Tenant B result: {:?}", result_b);
    
    Ok(())
}
```

## Performance Examples

### Parallel Processing

```rust
use futures::stream::{FuturesUnordered, StreamExt};
use wasm_sandbox::{SandboxPool, WasmSandbox};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a pool of sandboxes for parallel processing
    let pool = SandboxPool::builder()
        .source("examples/parallel_processor.wasm")
        .max_instances(num_cpus::get())
        .preload_instances(true)
        .build()
        .await?;

    // Large dataset to process
    let data_chunks: Vec<_> = (0..1000)
        .map(|i| format!("data_chunk_{}", i))
        .collect();

    // Process chunks in parallel
    let mut futures = FuturesUnordered::new();
    
    for chunk in data_chunks {
        let pool = pool.clone();
        let future = async move {
            let sandbox = pool.acquire().await?;
            let result = sandbox.call("process_chunk", &chunk).await?;
            pool.release(sandbox).await?;
            Ok::<_, Box<dyn std::error::Error>>(result)
        };
        futures.push(future);
    }

    // Collect results
    let mut results = Vec::new();
    while let Some(result) = futures.next().await {
        match result {
            Ok(processed_chunk) => results.push(processed_chunk),
            Err(e) => eprintln!("Processing error: {}", e),
        }
    }

    println!("Processed {} chunks", results.len());
    
    // Aggregate results
    let sandbox = pool.acquire().await?;
    let final_result = sandbox.call("aggregate_results", &results).await?;
    pool.release(sandbox).await?;
    
    println!("Final result: {:?}", final_result);
    
    Ok(())
}
```

### Cached Execution

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasm_sandbox::{WasmSandbox, ExecutionCache};

pub struct CachedExecutor {
    sandbox: WasmSandbox,
    cache: Arc<RwLock<HashMap<String, (serde_json::Value, std::time::Instant)>>>,
    cache_ttl: Duration,
}

impl CachedExecutor {
    pub async fn new(wasm_path: &str, cache_ttl: Duration) -> Result<Self> {
        let sandbox = WasmSandbox::builder()
            .source(wasm_path)
            .enable_execution_cache(true)
            .build()
            .await?;

        Ok(Self {
            sandbox,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
        })
    }

    pub async fn call_cached(&self, function: &str, args: &serde_json::Value) -> Result<serde_json::Value> {
        let cache_key = format!("{}:{}", function, serde_json::to_string(args)?);
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some((result, timestamp)) = cache.get(&cache_key) {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(result.clone());
                }
            }
        }

        // Execute function
        let result = self.sandbox.call(function, args).await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, (result.clone(), std::time::Instant::now()));
        }

        Ok(result)
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        let total_entries = cache.len();
        let expired_entries = cache.values()
            .filter(|(_, timestamp)| timestamp.elapsed() > self.cache_ttl)
            .count();
        
        (total_entries, expired_entries)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let executor = CachedExecutor::new(
        "examples/expensive_computation.wasm",
        Duration::from_secs(300) // 5 minute cache
    ).await?;

    // These calls will be cached
    let result1 = executor.call_cached("fibonacci", &serde_json::json!(40)).await?;
    let result2 = executor.call_cached("fibonacci", &serde_json::json!(40)).await?; // Cached
    let result3 = executor.call_cached("factorial", &serde_json::json!(20)).await?;
    
    println!("Results: {:?}, {:?}, {:?}", result1, result2, result3);
    
    let (total, expired) = executor.cache_stats().await;
    println!("Cache stats: {} total, {} expired", total, expired);
    
    Ok(())
}
```

Next: **[Troubleshooting](troubleshooting.md)** - Common issues and solutions

---

**Example Excellence:** Practical examples demonstrate real-world usage patterns and best practices for wasm-sandbox implementation.
