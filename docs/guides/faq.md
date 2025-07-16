# FAQ - Frequently Asked Questions

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Quick answers to common questions about wasm-sandbox usage, configuration, and troubleshooting.

## General Questions

### Q: What is wasm-sandbox and why would I use it?

**A:** wasm-sandbox is a secure WebAssembly runtime for executing untrusted code safely. Use it when you need to:

- Run user-provided scripts or plugins securely
- Isolate third-party code execution
- Build plugin architectures with strong sandboxing
- Process untrusted data with custom algorithms
- Create serverless-style execution environments

```rust
// Example: Safe execution of user code
let sandbox = WasmSandbox::builder()
    .source("user_script.wasm")
    .security_policy(SecurityPolicy::strict())
    .build()
    .await?;

let result = sandbox.call("process_data", user_data).await?;
```

### Q: How does wasm-sandbox compare to other sandboxing solutions?

**A:** wasm-sandbox offers several advantages:

| Feature | wasm-sandbox | Docker | Native sandboxing |
|---------|--------------|---------|-------------------|
| **Startup time** | ~1ms | ~100ms | Varies |
| **Memory overhead** | Low (~MB) | High (~100MB) | Medium |
| **Security model** | Capability-based | Container-based | OS-based |
| **Language support** | Any→WASM | Any | Native only |
| **Portability** | High | Medium | Low |
| **Performance** | Near-native | Good | Native |

### Q: What programming languages can I run in wasm-sandbox?

**A:** Any language that compiles to WebAssembly:

- **Rust** - First-class support, excellent performance
- **C/C++** - Via Emscripten or clang
- **AssemblyScript** - TypeScript-like syntax
- **Go** - Via TinyGo
- **Python** - Via Pyodide (limited)
- **C#** - Via Blazor WebAssembly
- **Java** - Via TeaVM or JWebAssembly

```rust
// Language-specific optimization
let sandbox = WasmSandbox::builder()
    .source("rust_module.wasm")
    .optimization_target(Language::Rust)
    .build()
    .await?;
```

## Security Questions

### Q: How secure is wasm-sandbox? Can untrusted code escape?

**A:** wasm-sandbox provides multiple security layers:

1. **WebAssembly isolation** - Memory-safe by design
2. **Capability-based security** - Explicit permission model
3. **Resource limits** - CPU, memory, I/O constraints
4. **Audit logging** - Track all privileged operations
5. **Runtime validation** - Continuous security monitoring

```rust
let sandbox = WasmSandbox::builder()
    .source("untrusted.wasm")
    .security_policy(SecurityPolicy::paranoid())
    .audit_enabled(true)
    .build()
    .await?;

// All operations are logged and validated
let result = sandbox.call_with_audit("risky_function", data).await?;
```

### Q: What capabilities can I grant to sandboxed code?

**A:** Capabilities include:

```rust
use wasm_sandbox::Capability;

// File system access
Capability::FileRead("/safe/path".into())
Capability::FileWrite("/output".into())

// Network access
Capability::NetworkAccess {
    allowed_hosts: vec!["api.example.com".to_string()],
    allowed_ports: vec![443, 80],
}

// System operations
Capability::ProcessSpawn(vec!["git".to_string()])
Capability::EnvironmentRead(vec!["PATH".to_string()])

// Custom capabilities
Capability::Custom("database_read".to_string())
```

### Q: How do I audit security violations?

**A:** Enable comprehensive auditing:

```rust
let sandbox = WasmSandbox::builder()
    .source("module.wasm")
    .audit_enabled(true)
    .security_policy(SecurityPolicy::strict())
    .build()
    .await?;

let result = sandbox.call_with_audit("function", args).await?;

// Review audit results
for violation in result.audit_log.violations {
    println!("Security violation: {}", violation.message);
    println!("Capability: {:?}", violation.capability);
    println!("Severity: {:?}", violation.severity);
}

// Export audit log
let audit_report = result.audit_log.to_json()?;
std::fs::write("audit.json", audit_report)?;
```

## Performance Questions

### Q: How fast is wasm-sandbox compared to native execution?

**A:** Performance varies by workload:

- **CPU-intensive**: 80-95% of native performance
- **Memory operations**: 85-98% of native performance  
- **I/O operations**: Limited by capability overhead (~5-15%)
- **Function calls**: ~1-10μs overhead per call

```rust
// Optimize for performance
let sandbox = WasmSandbox::builder()
    .source("module.wasm")
    .optimization_level(OptimizationLevel::Speed)
    .jit_enabled(true)
    .memory_pooling(true)
    .build()
    .await?;
```

### Q: How can I optimize cold start performance?

**A:** Several strategies help reduce startup time:

```rust
// 1. Use precompiled modules
let sandbox = WasmSandbox::builder()
    .precompiled_module("module.cwasm")
    .build()
    .await?;

// 2. Enable caching
let sandbox = WasmSandbox::builder()
    .source("module.wasm")
    .enable_cache(true)
    .cache_dir("./wasm_cache")
    .build()
    .await?;

// 3. Use sandbox pools
let pool = SandboxPool::builder()
    .source("module.wasm")
    .max_instances(10)
    .preload_instances(5)
    .build()
    .await?;

// 4. Optimize compilation
let sandbox = WasmSandbox::builder()
    .source("module.wasm")
    .compile_strategy(CompileStrategy::Lazy)
    .build()
    .await?;
```

### Q: How do I profile and debug performance issues?

**A:** Use built-in profiling tools:

```rust
use wasm_sandbox::profiling::{Profiler, ProfileConfig};

let profiler = Profiler::new(ProfileConfig {
    sample_rate: 1000, // 1ms samples
    track_memory: true,
    track_cpu: true,
    track_function_calls: true,
});

let sandbox = WasmSandbox::builder()
    .source("module.wasm")
    .profiler(profiler)
    .build()
    .await?;

let result = sandbox.call("expensive_function", args).await?;

// Analyze performance
let profile = sandbox.get_profile_report().await?;
println!("Execution time: {:?}", profile.total_cpu_time);
println!("Peak memory: {} bytes", profile.peak_memory_usage);
println!("Function calls: {}", profile.function_call_count);
```

## Development Questions

### Q: How do I compile my Rust code to work with wasm-sandbox?

**A:** Follow these steps:

```bash
# Install required targets
rustup target add wasm32-wasi

# Configure Cargo.toml
cat >> Cargo.toml << EOF
[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
EOF

# Compile to WebAssembly
cargo build --target wasm32-wasi --release

# Optimize the output
wasm-opt -Oz -o optimized.wasm target/wasm32-wasi/release/my_lib.wasm
```

For the Rust code:

```rust
// lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn process_data(input: &str) -> String {
    // Your processing logic
    format!("Processed: {}", input)
}

#[wasm_bindgen]
pub fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

### Q: How do I handle errors in sandboxed code?

**A:** Implement comprehensive error handling:

```rust
use wasm_sandbox::{WasmError, ErrorKind};

async fn safe_execution(sandbox: &WasmSandbox) -> Result<String> {
    match sandbox.call("risky_function", args).await {
        Ok(result) => Ok(result),
        
        Err(WasmError::RuntimeError(e)) => {
            // Handle runtime errors
            eprintln!("Runtime error: {}", e);
            
            // Try recovery
            sandbox.reset().await?;
            sandbox.call("fallback_function", args).await
        }
        
        Err(WasmError::ResourceLimitExceeded(limit)) => {
            // Handle resource limits
            eprintln!("Resource limit exceeded: {:?}", limit);
            
            // Reduce input size and retry
            let reduced_args = reduce_complexity(args);
            sandbox.call("risky_function", reduced_args).await
        }
        
        Err(WasmError::SecurityViolation(violation)) => {
            // Log security issues
            eprintln!("Security violation: {}", violation);
            Err("Security policy violation".into())
        }
        
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
            Err(e)
        }
    }
}
```

### Q: How do I pass complex data structures between host and guest?

**A:** Use serialization with structured data:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ComplexData {
    id: u32,
    name: String,
    values: Vec<f64>,
    metadata: std::collections::HashMap<String, String>,
}

// In host code
let data = ComplexData {
    id: 42,
    name: "test".to_string(),
    values: vec![1.0, 2.0, 3.0],
    metadata: [("key".to_string(), "value".to_string())].into(),
};

let result: ComplexData = sandbox.call("process_complex", &data).await?;

// In guest code (Rust)
#[wasm_bindgen]
pub fn process_complex(input: JsValue) -> JsValue {
    let data: ComplexData = input.into_serde().unwrap();
    
    // Process data
    let mut result = data;
    result.values.iter_mut().for_each(|v| *v *= 2.0);
    
    JsValue::from_serde(&result).unwrap()
}
```

## Integration Questions

### Q: How do I integrate wasm-sandbox with web frameworks?

**A:** Examples for popular frameworks:

**Axum:**

```rust
use axum::{extract::Json, response::Json as ResponseJson, routing::post, Router};

async fn process_data(Json(input): Json<serde_json::Value>) -> ResponseJson<serde_json::Value> {
    let sandbox = WasmSandbox::builder()
        .source("processor.wasm")
        .build()
        .await
        .unwrap();
    
    let result = sandbox.call("process", &input).await.unwrap();
    ResponseJson(result)
}

let app = Router::new().route("/process", post(process_data));
```

**Actix-web:**

```rust
use actix_web::{web, App, HttpServer, Result};

async fn process_handler(data: web::Json<serde_json::Value>) -> Result<web::Json<serde_json::Value>> {
    let sandbox = WasmSandbox::builder()
        .source("processor.wasm")
        .build()
        .await?;
    
    let result = sandbox.call("process", &data.into_inner()).await?;
    Ok(web::Json(result))
}

HttpServer::new(|| {
    App::new().route("/process", web::post().to(process_handler))
})
```

### Q: How do I use wasm-sandbox in a microservices architecture?

**A:** Design for scalability and isolation:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MicroservicePool {
    sandboxes: Arc<RwLock<Vec<WasmSandbox>>>,
    max_size: usize,
}

impl MicroservicePool {
    pub async fn new(service_wasm: &str, pool_size: usize) -> Result<Self> {
        let mut sandboxes = Vec::new();
        
        for _ in 0..pool_size {
            let sandbox = WasmSandbox::builder()
                .source(service_wasm)
                .security_policy(SecurityPolicy::microservice())
                .memory_limit(64 * 1024 * 1024) // 64MB
                .build()
                .await?;
            sandboxes.push(sandbox);
        }
        
        Ok(Self {
            sandboxes: Arc::new(RwLock::new(sandboxes)),
            max_size: pool_size,
        })
    }
    
    pub async fn execute(&self, function: &str, args: &serde_json::Value) -> Result<serde_json::Value> {
        let sandbox = {
            let mut pool = self.sandboxes.write().await;
            pool.pop().ok_or("No available sandbox instances")?
        };
        
        let result = sandbox.call(function, args).await;
        
        // Return sandbox to pool
        let mut pool = self.sandboxes.write().await;
        pool.push(sandbox);
        
        result
    }
}
```

### Q: Can I use wasm-sandbox for serverless functions?

**A:** Yes, it's well-suited for serverless:

```rust
// AWS Lambda handler example
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let sandbox = WasmSandbox::builder()
        .source("function.wasm")
        .memory_limit(128 * 1024 * 1024) // Lambda memory limit
        .cpu_timeout(Duration::from_secs(15)) // Lambda timeout
        .build()
        .await?;
    
    let result = sandbox.call("handle", &event.payload).await?;
    
    Ok(json!({
        "statusCode": 200,
        "body": result
    }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(handler)).await
}
```

## Troubleshooting Questions

### Q: My WebAssembly module won't load. What's wrong?

**A:** Common issues and solutions:

```rust
// 1. Check module validity
use wasm_sandbox::validation::validate_wasm_module;

let wasm_bytes = std::fs::read("module.wasm")?;
match validate_wasm_module(&wasm_bytes) {
    Ok(info) => println!("Module valid: {:?}", info),
    Err(e) => {
        eprintln!("Invalid module: {}", e);
        // Common fixes:
        // - Recompile with correct target (wasm32-wasi)
        // - Check for unsupported instructions
        // - Verify exports are properly declared
    }
}

// 2. Enable debug mode for detailed errors
let sandbox = WasmSandbox::builder()
    .source("module.wasm")
    .debug_mode(true)
    .build()
    .await?;
```

### Q: Function calls are failing with type errors. How do I fix this?

**A:** Ensure type compatibility:

```rust
// 1. Check function signatures
let exports = sandbox.list_exports().await?;
for export in exports {
    println!("Function: {}, Signature: {:?}", export.name, export.signature);
}

// 2. Use correct types
// Wrong: i32 when function expects f64
// let result: i32 = sandbox.call("calculate", (42,)).await?;

// Correct:
let result: f64 = sandbox.call("calculate", (42.0f64,)).await?;

// 3. Handle optional return values
match sandbox.try_call::<(), Option<String>>("optional_function", ()).await? {
    Some(result) => println!("Got result: {}", result),
    None => println!("Function returned None"),
}
```

### Q: Performance is slower than expected. How do I optimize?

**A:** Performance optimization checklist:

```rust
// 1. Enable optimizations
let sandbox = WasmSandbox::builder()
    .source("module.wasm")
    .optimization_level(OptimizationLevel::Speed)
    .jit_enabled(true)
    .build()
    .await?;

// 2. Use batch operations
let results: Vec<f64> = sandbox.call("batch_process", &large_dataset).await?;
// Instead of:
// for item in large_dataset {
//     let result = sandbox.call("process_item", item).await?;
// }

// 3. Pool instances for repeated use
let pool = SandboxPool::builder()
    .source("module.wasm")
    .max_instances(num_cpus::get())
    .build()
    .await?;

// 4. Profile to find bottlenecks
let profiler = Profiler::new(ProfileConfig::default());
let sandbox = WasmSandbox::builder()
    .source("module.wasm")
    .profiler(profiler)
    .build()
    .await?;
```

## Best Practices

### Q: What are the security best practices?

**A:** Follow these guidelines:

1. **Principle of least privilege** - Grant minimal required capabilities
2. **Input validation** - Validate all data before passing to sandbox
3. **Resource limits** - Set appropriate memory/CPU/I/O limits
4. **Audit logging** - Enable comprehensive security auditing
5. **Regular updates** - Keep wasm-sandbox and dependencies updated

```rust
let sandbox = WasmSandbox::builder()
    .source("untrusted.wasm")
    .security_policy(SecurityPolicy::strict())
    .memory_limit(32 * 1024 * 1024) // 32MB max
    .cpu_timeout(Duration::from_secs(10)) // 10s max
    .audit_enabled(true)
    .build()
    .await?;
```

### Q: How should I structure my WebAssembly modules?

**A:** Module organization recommendations:

```rust
// 1. Separate concerns
// - business_logic.wasm (core algorithms)
// - data_processing.wasm (I/O operations)  
// - utilities.wasm (helper functions)

// 2. Use clear interfaces
#[wasm_bindgen]
pub struct DataProcessor {
    // Internal state
}

#[wasm_bindgen]
impl DataProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { /* ... */ }
    
    #[wasm_bindgen]
    pub fn process(&mut self, input: &str) -> String { /* ... */ }
    
    #[wasm_bindgen]
    pub fn get_stats(&self) -> JsValue { /* ... */ }
}

// 3. Handle errors gracefully
#[wasm_bindgen]
pub fn safe_operation(input: &str) -> Result<String, JsValue> {
    match risky_operation(input) {
        Ok(result) => Ok(result),
        Err(e) => Err(JsValue::from_str(&format!("Error: {}", e))),
    }
}
```

Next: **[Changelog](../CHANGELOG.md)** - Version history and updates

---

**FAQ Excellence:** Quick access to essential information and solutions for common wasm-sandbox scenarios.
