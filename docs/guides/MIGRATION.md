# Migration Guide

## Migrating from v0.1.0 to v0.2.0

This guide helps you upgrade your code from wasm-sandbox v0.1.0 to v0.2.0.

### Major Changes

#### 1. Trait-Based Architecture

**Before (v0.1.0):**
```rust
use wasm_sandbox::WasmSandbox;

let mut sandbox = WasmSandbox::new()?;
let module_id = sandbox.load_module(&wasm_bytes)?;
let instance_id = sandbox.create_instance(module_id, None)?;
let result: String = sandbox.call_function(instance_id, "greet", &"World").await?;
```

**After (v0.2.0):**
```rust
use wasm_sandbox::{WasmSandbox, runtime::WasmInstanceExt};

let mut sandbox = WasmSandbox::new()?;
let module_id = sandbox.load_module(&wasm_bytes)?;
let instance_id = sandbox.create_instance(module_id, None)?;

// For generic/async operations, import extension traits
let result: String = sandbox.get_instance(instance_id)?
    .call_function("greet", &"World").await?;
```

#### 2. Configuration Changes

**Before (v0.1.0):**
```rust
let config = SandboxConfig {
    memory_limit: Some(64 * 1024 * 1024),
    execution_timeout: Some(Duration::from_secs(30)),
    // Direct field access
};
```

**After (v0.2.0):**
```rust
let config = SandboxConfig {
    runtime: RuntimeConfig::default(),
    security: SecurityConfig::default(),
    // New structured approach
};

// Or use builder pattern (recommended):
let config = SandboxConfig::default()
    .with_memory_limit(64 * 1024 * 1024)
    .with_timeout(Duration::from_secs(30));
```

#### 3. Extension Traits for Advanced Features

**Key Change**: Generic and async operations moved to extension traits.

**Import Required Extension Traits:**
```rust
use wasm_sandbox::runtime::{
    WasmInstanceExt,    // For generic function calls
    WasmRuntimeExt,     // For advanced runtime operations  
    RpcChannelExt,      // For type-safe communication
};
```

#### 4. Error Handling Improvements

**Before (v0.1.0):**
```rust
match sandbox.call_function(id, "func", &params) {
    Ok(result) => { /* handle result */ },
    Err(e) => { /* generic error */ },
}
```

**After (v0.2.0):**
```rust
match sandbox.get_instance(id)?.call_function("func", &params).await {
    Ok(result) => { /* handle result */ },
    Err(wasm_sandbox::Error::Security(msg)) => { /* security violation */ },
    Err(wasm_sandbox::Error::ResourceLimit(limit)) => { /* resource exceeded */ },
    Err(e) => { /* other errors */ },
}
```

### Breaking Changes Checklist

- [ ] **Import extension traits** for generic/async operations
- [ ] **Update configuration** to use new structured format
- [ ] **Add .await** to async function calls
- [ ] **Update error handling** for more specific error types
- [ ] **Use trait objects** if you need runtime flexibility

### Example: Complete Migration

**Before (v0.1.0):**
```rust
use wasm_sandbox::{WasmSandbox, SandboxConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SandboxConfig {
        memory_limit: Some(64 * 1024 * 1024),
        execution_timeout: Some(Duration::from_secs(30)),
    };
    
    let mut sandbox = WasmSandbox::with_config(config)?;
    let wasm_bytes = std::fs::read("module.wasm")?;
    let module_id = sandbox.load_module(&wasm_bytes)?;
    let instance_id = sandbox.create_instance(module_id, None)?;
    
    let result: i32 = sandbox.call_function(instance_id, "add", &[5, 7]).await?;
    println!("Result: {}", result);
    
    Ok(())
}
```

**After (v0.2.0):**
```rust
use wasm_sandbox::{WasmSandbox, SandboxConfig, InstanceConfig};
use wasm_sandbox::runtime::WasmInstanceExt;
use wasm_sandbox::security::{Capabilities, ResourceLimits};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // New configuration structure
    let mut sandbox = WasmSandbox::new()?;
    
    let wasm_bytes = std::fs::read("module.wasm")?;
    let module_id = sandbox.load_module(&wasm_bytes)?;
    
    // Configure instance with resource limits
    let instance_config = InstanceConfig {
        capabilities: Capabilities::minimal(),
        resource_limits: ResourceLimits {
            memory_bytes: Some(64 * 1024 * 1024),
            execution_timeout: Some(Duration::from_secs(30)),
            ..ResourceLimits::default()
        },
    };
    
    let instance_id = sandbox.create_instance(module_id, Some(instance_config))?;
    
    // Use extension trait for generic calls
    let result: i32 = sandbox.get_instance(instance_id)?
        .call_function("add", &[5, 7]).await?;
    println!("Result: {}", result);
    
    Ok(())
}
```

### Common Issues and Solutions

#### Issue: "trait `WasmInstanceExt` is not implemented"

**Solution**: Import the extension trait:
```rust
use wasm_sandbox::runtime::WasmInstanceExt;
```

#### Issue: "async function call without .await"

**Solution**: Extension trait methods are async:
```rust
// Before
let result = instance.call_function("func", &params)?;

// After  
let result = instance.call_function("func", &params).await?;
```

#### Issue: "cannot find function `with_memory_limit`"

**Solution**: Use the new configuration structure:
```rust
// Before
let config = SandboxConfig::new().with_memory_limit(64 * 1024 * 1024);

// After
let instance_config = InstanceConfig {
    resource_limits: ResourceLimits {
        memory_bytes: Some(64 * 1024 * 1024),
        ..ResourceLimits::default()
    },
    ..InstanceConfig::default()
};
```

### Performance Notes

- **Extension traits**: Zero-cost abstractions when used with concrete types
- **Trait objects**: Minimal overhead for dynamic dispatch
- **Async operations**: Better resource utilization for I/O-bound workloads

### Getting Help

- **Documentation**: See [docs.rs/wasm-sandbox](https://docs.rs/wasm-sandbox)
- **Examples**: Check the `examples/` directory
- **Issues**: Report problems at [GitHub Issues](https://github.com/ciresnave/wasm-sandbox/issues)
- **Discussions**: Ask questions at [GitHub Discussions](https://github.com/ciresnave/wasm-sandbox/discussions)
