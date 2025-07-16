# Basic Usage

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Common usage patterns and everyday operations with wasm-sandbox.

## Quick Start

### Simple Function Calls

```rust
use wasm_sandbox::{WasmSandbox, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a sandbox
    let sandbox = WasmSandbox::builder()
        .source("calculator.wasm")
        .build()
        .await?;

    // Call functions with different types
    let sum: i32 = sandbox.call("add", (10, 20)).await?;
    let greeting: String = sandbox.call("greet", "Alice").await?;
    let result: f64 = sandbox.call("sqrt", 16.0).await?;

    println!("Sum: {}, Greeting: {}, Square root: {}", sum, greeting, result);
    Ok(())
}
```

### Working with Complex Data

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct ProcessedData {
    count: usize,
    average_age: f64,
    valid_emails: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let sandbox = WasmSandbox::builder()
        .source("data_processor.wasm")
        .build()
        .await?;

    let people = vec![
        Person {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        },
        Person {
            name: "Bob".to_string(),
            age: 25,
            email: "bob@test.com".to_string(),
        },
    ];

    // Pass complex data structures
    let result: ProcessedData = sandbox.call("process_people", &people).await?;
    println!("Processed {} people, average age: {:.1}", 
             result.count, result.average_age);
    
    Ok(())
}
```

## Configuration Patterns

### Builder Pattern Configuration

```rust
use wasm_sandbox::{WasmSandbox, SecurityPolicy, Capability};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let sandbox = WasmSandbox::builder()
        // Module source
        .source("module.wasm")
        
        // Resource limits
        .memory_limit(64 * 1024 * 1024) // 64MB
        .cpu_timeout(Duration::from_secs(30))
        .max_file_operations(100)
        
        // Security settings
        .security_policy(SecurityPolicy::strict())
        .add_capability(Capability::FileRead("/data".into()))
        
        // Performance options
        .optimization_level(OptimizationLevel::Speed)
        .enable_cache(true)
        
        // Debugging
        .debug_mode(false)
        .audit_enabled(true)
        
        .build()
        .await?;

    Ok(())
}
```

### Configuration from File

```rust
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Serialize, Deserialize)]
struct SandboxConfig {
    wasm_file: String,
    memory_limit_mb: u64,
    cpu_timeout_secs: u64,
    security_level: String,
    capabilities: Vec<String>,
}

impl SandboxConfig {
    fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    async fn create_sandbox(&self) -> Result<WasmSandbox> {
        let mut policy = match self.security_level.as_str() {
            "strict" => SecurityPolicy::strict(),
            "basic" => SecurityPolicy::basic(),
            "permissive" => SecurityPolicy::permissive(),
            _ => SecurityPolicy::basic(),
        };

        // Add capabilities from config
        for cap in &self.capabilities {
            match cap.as_str() {
                "file_read" => policy.add_capability(Capability::FileRead("/data".into())),
                "network" => policy.add_capability(Capability::NetworkAccess {
                    allowed_hosts: vec!["api.example.com".to_string()],
                    allowed_ports: vec![443],
                }),
                _ => {}
            }
        }

        WasmSandbox::builder()
            .source(&self.wasm_file)
            .memory_limit(self.memory_limit_mb * 1024 * 1024)
            .cpu_timeout(Duration::from_secs(self.cpu_timeout_secs))
            .security_policy(policy)
            .build()
            .await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration from file
    let config = SandboxConfig::load("sandbox_config.json")?;
    let sandbox = config.create_sandbox().await?;

    // Use configured sandbox
    let result = sandbox.call("process", "test data").await?;
    println!("Result: {:?}", result);

    Ok(())
}
```

Example `sandbox_config.json`:

```json
{
    "wasm_file": "modules/processor.wasm",
    "memory_limit_mb": 128,
    "cpu_timeout_secs": 60,
    "security_level": "strict",
    "capabilities": ["file_read", "network"]
}
```

## Error Handling Patterns

### Basic Error Handling

```rust
use wasm_sandbox::{WasmError, ErrorKind};

async fn safe_call(sandbox: &WasmSandbox, input: &str) -> Result<String> {
    match sandbox.call("process", input).await {
        Ok(result) => Ok(result),
        Err(WasmError::RuntimeError(e)) => {
            eprintln!("Runtime error: {}", e);
            // Try fallback
            sandbox.call("fallback_process", input).await
        }
        Err(WasmError::ResourceLimitExceeded(limit)) => {
            eprintln!("Resource limit exceeded: {:?}", limit);
            // Reduce input size and retry
            let reduced_input = &input[..input.len().min(1000)];
            sandbox.call("process", reduced_input).await
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
            Err(e)
        }
    }
}
```

### Comprehensive Error Recovery

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

struct RetryableSandbox {
    sandbox: WasmSandbox,
    failure_count: Arc<AtomicU32>,
    max_retries: u32,
}

impl RetryableSandbox {
    async fn call_with_retry<T, A>(&self, function: &str, args: A) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        A: serde::Serialize + Clone,
    {
        let mut attempts = 0;
        
        loop {
            match self.sandbox.call(function, &args).await {
                Ok(result) => {
                    // Reset failure count on success
                    self.failure_count.store(0, Ordering::Relaxed);
                    return Ok(result);
                }
                Err(e) => {
                    attempts += 1;
                    self.failure_count.fetch_add(1, Ordering::Relaxed);
                    
                    if attempts >= self.max_retries {
                        return Err(e);
                    }
                    
                    // Exponential backoff
                    let delay = Duration::from_millis(100 * 2_u64.pow(attempts));
                    tokio::time::sleep(delay).await;
                    
                    // Reset sandbox on certain errors
                    match &e {
                        WasmError::RuntimeError(_) => {
                            if let Err(reset_err) = self.sandbox.reset().await {
                                eprintln!("Failed to reset sandbox: {}", reset_err);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
```

## Data Serialization

### Multiple Serialization Formats

```rust
use wasm_sandbox::{WasmSandbox, SerializationFormat};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct TestData {
    id: u32,
    values: Vec<f64>,
    metadata: std::collections::HashMap<String, String>,
}

async fn test_serialization_formats() -> Result<()> {
    let data = TestData {
        id: 42,
        values: vec![1.0, 2.0, 3.0],
        metadata: [("key".to_string(), "value".to_string())].into(),
    };

    // JSON serialization (default)
    let json_sandbox = WasmSandbox::builder()
        .source("processor.wasm")
        .serialization_format(SerializationFormat::Json)
        .build()
        .await?;

    let json_result: TestData = json_sandbox.call("process", &data).await?;
    println!("JSON result: {:?}", json_result);

    // MessagePack serialization (more efficient)
    let msgpack_sandbox = WasmSandbox::builder()
        .source("processor.wasm")
        .serialization_format(SerializationFormat::MessagePack)
        .build()
        .await?;

    let msgpack_result: TestData = msgpack_sandbox.call("process", &data).await?;
    println!("MessagePack result: {:?}", msgpack_result);

    // Bincode serialization (fastest)
    let bincode_sandbox = WasmSandbox::builder()
        .source("processor.wasm")
        .serialization_format(SerializationFormat::Bincode)
        .build()
        .await?;

    let bincode_result: TestData = bincode_sandbox.call("process", &data).await?;
    println!("Bincode result: {:?}", bincode_result);

    Ok(())
}
```

### Custom Serialization

```rust
use serde::{Serialize, Deserialize};

// Custom serialization trait
trait CustomSerialize {
    fn serialize_custom(&self) -> Result<Vec<u8>>;
    fn deserialize_custom(data: &[u8]) -> Result<Self> where Self: Sized;
}

struct CustomDataProcessor {
    sandbox: WasmSandbox,
}

impl CustomDataProcessor {
    async fn process_custom<T, R>(&self, data: &T) -> Result<R>
    where
        T: CustomSerialize,
        R: CustomSerialize,
    {
        // Serialize input using custom format
        let input_bytes = data.serialize_custom()?;
        
        // Call WebAssembly function with raw bytes
        let output_bytes: Vec<u8> = self.sandbox.call("process_bytes", &input_bytes).await?;
        
        // Deserialize result using custom format
        R::deserialize_custom(&output_bytes)
    }
}
```

## Resource Management

### Memory Monitoring

```rust
use wasm_sandbox::monitoring::MemoryMonitor;

async fn monitor_memory_usage() -> Result<()> {
    let sandbox = WasmSandbox::builder()
        .source("memory_intensive.wasm")
        .memory_limit(128 * 1024 * 1024) // 128MB
        .build()
        .await?;

    // Create memory monitor
    let monitor = MemoryMonitor::new();
    monitor.start_monitoring(&sandbox).await?;

    // Perform memory-intensive operation
    let result = sandbox.call("process_large_data", &large_dataset).await?;

    // Check memory usage
    let usage = monitor.get_current_usage().await?;
    println!("Peak memory usage: {} MB", usage.peak_memory / (1024 * 1024));
    println!("Current memory usage: {} MB", usage.current_memory / (1024 * 1024));

    // Generate memory report
    let report = monitor.generate_report().await?;
    if !report.potential_leaks.is_empty() {
        println!("Warning: {} potential memory leaks detected", report.potential_leaks.len());
    }

    Ok(())
}
```

### CPU Usage Monitoring

```rust
use wasm_sandbox::monitoring::CpuMonitor;
use std::time::Instant;

async fn monitor_cpu_usage() -> Result<()> {
    let sandbox = WasmSandbox::builder()
        .source("cpu_intensive.wasm")
        .cpu_timeout(Duration::from_secs(60))
        .build()
        .await?;

    let start_time = Instant::now();
    let cpu_monitor = CpuMonitor::new();
    cpu_monitor.start_monitoring(&sandbox).await?;

    // CPU-intensive operation
    let result = sandbox.call("fibonacci", 40).await?;

    let elapsed = start_time.elapsed();
    let cpu_usage = cpu_monitor.get_cpu_usage().await?;

    println!("Wall time: {:?}", elapsed);
    println!("CPU time: {:?}", cpu_usage.total_cpu_time);
    println!("CPU efficiency: {:.1}%", 
             (cpu_usage.total_cpu_time.as_secs_f64() / elapsed.as_secs_f64()) * 100.0);

    Ok(())
}
```

## Batch Processing

### Processing Multiple Items

```rust
async fn batch_process(items: Vec<String>) -> Result<Vec<String>> {
    let sandbox = WasmSandbox::builder()
        .source("batch_processor.wasm")
        .build()
        .await?;

    // Option 1: Process items individually
    let mut results = Vec::new();
    for item in &items {
        let result: String = sandbox.call("process_item", item).await?;
        results.push(result);
    }

    // Option 2: Process entire batch at once (more efficient)
    let batch_result: Vec<String> = sandbox.call("process_batch", &items).await?;

    Ok(batch_result)
}
```

### Parallel Batch Processing

```rust
use futures::stream::{FuturesUnordered, StreamExt};

async fn parallel_batch_process(items: Vec<String>) -> Result<Vec<String>> {
    let sandbox = Arc::new(WasmSandbox::builder()
        .source("processor.wasm")
        .build()
        .await?);

    // Create futures for each item
    let mut futures = FuturesUnordered::new();
    
    for item in items {
        let sandbox_clone = Arc::clone(&sandbox);
        let future = async move {
            sandbox_clone.call::<String, String>("process", &item).await
        };
        futures.push(future);
    }

    // Collect results
    let mut results = Vec::new();
    while let Some(result) = futures.next().await {
        results.push(result?);
    }

    Ok(results)
}
```

## State Management

### Stateful Sandbox Operations

```rust
struct StatefulProcessor {
    sandbox: WasmSandbox,
}

impl StatefulProcessor {
    async fn new() -> Result<Self> {
        let sandbox = WasmSandbox::builder()
            .source("stateful.wasm")
            .build()
            .await?;

        // Initialize state
        sandbox.call::<(), ()>("initialize", ()).await?;

        Ok(Self { sandbox })
    }

    async fn add_data(&self, data: &str) -> Result<()> {
        self.sandbox.call("add_data", data).await
    }

    async fn process_accumulated(&self) -> Result<String> {
        self.sandbox.call("process_all", ()).await
    }

    async fn reset_state(&self) -> Result<()> {
        self.sandbox.call("reset", ()).await
    }

    async fn get_state_info(&self) -> Result<StateInfo> {
        self.sandbox.call("get_state", ()).await
    }
}

#[derive(serde::Deserialize)]
struct StateInfo {
    item_count: u32,
    memory_usage: u64,
    processing_time: f64,
}
```

### Session Management

```rust
use std::collections::HashMap;
use uuid::Uuid;

struct SessionManager {
    sessions: HashMap<Uuid, WasmSandbox>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub async fn create_session(&mut self) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let sandbox = WasmSandbox::builder()
            .source("session_processor.wasm")
            .build()
            .await?;

        self.sessions.insert(session_id, sandbox);
        Ok(session_id)
    }

    pub async fn execute_in_session<T, A>(&self, session_id: Uuid, function: &str, args: A) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        A: serde::Serialize,
    {
        let sandbox = self.sessions.get(&session_id)
            .ok_or_else(|| "Session not found")?;

        sandbox.call(function, args).await
    }

    pub fn close_session(&mut self, session_id: Uuid) -> Result<()> {
        self.sessions.remove(&session_id)
            .ok_or_else(|| "Session not found")?;
        Ok(())
    }
}
```

## Performance Optimization

### Connection Pooling

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

struct SandboxPool {
    sandboxes: Vec<Arc<WasmSandbox>>,
    semaphore: Arc<Semaphore>,
}

impl SandboxPool {
    async fn new(wasm_path: &str, pool_size: usize) -> Result<Self> {
        let mut sandboxes = Vec::new();
        
        for _ in 0..pool_size {
            let sandbox = Arc::new(WasmSandbox::builder()
                .source(wasm_path)
                .build()
                .await?);
            sandboxes.push(sandbox);
        }

        Ok(Self {
            sandboxes,
            semaphore: Arc::new(Semaphore::new(pool_size)),
        })
    }

    async fn execute<T, A>(&self, function: &str, args: A) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        A: serde::Serialize,
    {
        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await.unwrap();
        
        // Get available sandbox (simple round-robin)
        let sandbox_index = fastrand::usize(..self.sandboxes.len());
        let sandbox = &self.sandboxes[sandbox_index];
        
        sandbox.call(function, args).await
    }
}
```

### Caching Results

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

struct CachedSandbox {
    sandbox: WasmSandbox,
    cache: HashMap<u64, serde_json::Value>,
    cache_ttl: Duration,
    cache_timestamps: HashMap<u64, Instant>,
}

impl CachedSandbox {
    async fn call_cached<T, A>(&mut self, function: &str, args: A) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        A: serde::Serialize + Hash,
    {
        // Generate cache key
        let mut hasher = DefaultHasher::new();
        function.hash(&mut hasher);
        args.hash(&mut hasher);
        let cache_key = hasher.finish();

        // Check cache
        if let Some(cached_value) = self.cache.get(&cache_key) {
            if let Some(timestamp) = self.cache_timestamps.get(&cache_key) {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(serde_json::from_value(cached_value.clone())?);
                }
            }
        }

        // Execute function
        let result = self.sandbox.call(function, args).await?;
        
        // Cache result
        let json_value = serde_json::to_value(&result)?;
        self.cache.insert(cache_key, json_value);
        self.cache_timestamps.insert(cache_key, Instant::now());

        Ok(result)
    }
}
```

Next: **[Security Configuration](security-config.md)** - Configure capabilities and security policies

---

**Usage Patterns:** Master common wasm-sandbox operations for efficient and reliable WebAssembly execution.
