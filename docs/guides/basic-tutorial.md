# Basic Tutorial - Your First wasm-sandbox Application

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This tutorial will walk you through creating your first secure WebAssembly sandbox application step-by-step. By the end, you'll understand the core concepts and be ready to build real applications.

## Prerequisites

- Basic Rust knowledge
- Rust toolchain installed (1.70+)
- WebAssembly target: `rustup target add wasm32-wasi`

## Step 1: Project Setup

Create a new Rust project:

```bash
cargo new my-sandbox-app
cd my-sandbox-app
```

Add wasm-sandbox to your `Cargo.toml`:

```toml
[dependencies]
wasm-sandbox = "0.3.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Step 2: Create Your First Guest Program

Create a simple calculator module that we'll run in the sandbox. Create `calculator.rs`:

```rust
// calculator.rs - This will be compiled to WebAssembly
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct MathInput {
    pub a: i32,
    pub b: i32,
}

#[derive(Serialize)]
pub struct MathOutput {
    pub result: i32,
    pub operation: String,
}

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn multiply_complex() -> i32 {
    // This function shows how to work with JSON serialization
    // In a real implementation, this would take JSON input
    42
}
```

## Step 3: Your First Sandbox (One-Liner)

Replace `src/main.rs` with:

```rust
use wasm_sandbox;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 My First Sandbox Application");
    
    // The simplest possible usage - one line!
    let result: i32 = wasm_sandbox::run("calculator.rs", "add", &(5, 3)).await?;
    
    println!("✅ 5 + 3 = {}", result);
    
    Ok(())
}
```

Run it:

```bash
cargo run
```

**What happened?**

1. `wasm_sandbox::run()` automatically compiled `calculator.rs` to WebAssembly
2. Created a secure sandbox environment
3. Loaded the compiled module
4. Called the `add` function with parameters `(5, 3)`
5. Returned the result safely

## Step 4: Adding Timeouts (Safety First)

Real applications need protection against infinite loops or slow code:

```rust
use wasm_sandbox;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Sandbox with Timeout Protection");
    
    // Same function, but with a 5-second timeout
    let result: i32 = wasm_sandbox::run_with_timeout(
        "calculator.rs", 
        "add", 
        &(10, 20),
        Duration::from_secs(5)  // Never wait more than 5 seconds
    ).await?;
    
    println!("✅ 10 + 20 = {}", result);
    
    Ok(())
}
```

## Step 5: Builder Pattern (Full Control)

For production applications, you want fine-grained control:

```rust
use wasm_sandbox::WasmSandbox;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Production-Ready Sandbox");
    
    // Build a sandbox with specific configuration
    let sandbox = WasmSandbox::builder()
        .source("calculator.rs")                    // Source file to compile
        .timeout_duration(Duration::from_secs(30))  // 30-second timeout
        .memory_limit(16 * 1024 * 1024)            // 16MB memory limit
        .enable_file_access(false)                  // No filesystem access
        .build()
        .await?;
    
    // Call functions multiple times on the same sandbox
    let result1: i32 = sandbox.call("add", &(1, 2)).await?;
    let result2: i32 = sandbox.call("add", &(3, 4)).await?;
    
    println!("✅ Results: {} and {}", result1, result2);
    
    Ok(())
}
```

## Step 6: Error Handling

Production code needs robust error handling:

```rust
use wasm_sandbox::{WasmSandbox, Error};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Robust Error Handling");
    
    // Intentionally use a non-existent function to demonstrate error handling
    match wasm_sandbox::run("calculator.rs", "nonexistent_function", &()).await {
        Ok(result) => {
            println!("✅ Unexpected success: {:?}", result);
        }
        Err(Error::FunctionNotFound { function_name }) => {
            println!("❌ Function '{}' not found in module", function_name);
        }
        Err(Error::Compilation(msg)) => {
            println!("❌ Compilation failed: {}", msg);
        }
        Err(Error::Timeout) => {
            println!("❌ Function took too long to execute");
        }
        Err(Error::MemoryLimit { used, limit }) => {
            println!("❌ Memory limit exceeded: used {}, limit {}", used, limit);
        }
        Err(e) => {
            println!("❌ Other error: {}", e);
        }
    }
    
    Ok(())
}
```

## Step 7: Working with Complex Data

Real applications need to pass structured data:

```rust
use serde::{Deserialize, Serialize};
use wasm_sandbox::WasmSandbox;

#[derive(Serialize)]
struct CalculationRequest {
    numbers: Vec<i32>,
    operation: String,
}

#[derive(Deserialize)]
struct CalculationResult {
    result: i32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Complex Data Processing");
    
    // Create complex input data
    let request = CalculationRequest {
        numbers: vec![1, 2, 3, 4, 5],
        operation: "sum".to_string(),
    };
    
    // For this example, we'll use a simple function
    // In practice, you'd create a WebAssembly module that handles JSON
    let sandbox = WasmSandbox::from_source("calculator.rs").await?;
    let result: i32 = sandbox.call("add", &(10, 15)).await?;
    
    println!("✅ Calculation result: {}", result);
    
    Ok(())
}
```

## Step 8: Production Checklist

Before deploying to production, ensure you have:

### ✅ Security Configuration

```rust
let sandbox = WasmSandbox::builder()
    .source("my_module.rs")
    .memory_limit(64 * 1024 * 1024)        // Limit memory usage
    .timeout_duration(Duration::from_secs(30)) // Prevent infinite loops
    .enable_file_access(false)              // Disable filesystem
    .enable_network_access(false)           // Disable network
    .build().await?;
```

### ✅ Error Handling

- Handle all possible error types
- Log security violations
- Implement retry logic for transient failures
- Monitor resource usage

### ✅ Resource Monitoring

- Set appropriate memory limits
- Configure execution timeouts
- Monitor CPU usage
- Implement circuit breakers

### ✅ Testing

- Unit test your WebAssembly modules
- Integration test the sandbox configuration
- Load test with realistic workloads
- Test error conditions and recovery

## Next Steps

Now that you understand the basics, explore these advanced topics:

1. **[Security Configuration](security-config.md)** - Fine-tune security settings
2. **[Resource Management](resource-management.md)** - Advanced resource control
3. **[Plugin Development](plugin-development.md)** - Build plugin systems
4. **[Production Deployment](production.md)** - Production considerations
5. **[Examples](../../examples/README.md)** - Real-world examples

## Common Issues and Solutions

### Compilation Errors

```bash
error: target 'wasm32-wasi' not found
```

**Solution**: Install the WebAssembly target:

```bash
rustup target add wasm32-wasi
```

### Memory Allocation Failures

```rust
Error::MemoryLimit { used: 67108864, limit: 16777216 }
```

**Solution**: Increase memory limit or optimize your code:

```rust
.memory_limit(128 * 1024 * 1024)  // Increase to 128MB
```

### Function Not Found

```rust
Error::FunctionNotFound { function_name: "my_func" }
```

**Solution**: Ensure function is exported with `#[no_mangle]`:

```rust
#[no_mangle]
pub extern "C" fn my_func() -> i32 {
    42
}
```

## Getting Help

- 📖 **[Complete Documentation](../README.md)**
- 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**
- 💬 **[GitHub Discussions](https://github.com/ciresnave/wasm-sandbox/discussions)**
- 🐛 **[Report Issues](https://github.com/ciresnave/wasm-sandbox/issues)**

---

**Congratulations!** 🎉 You've successfully created your first secure WebAssembly sandbox application. You now understand the progressive complexity approach and are ready to build production applications.
