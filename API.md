# API Documentation

## Overview

The `wasm-sandbox` crate provides a high-level API for running WebAssembly modules in a secure sandbox environment. The API is designed to be both easy to use for common cases and flexible enough for advanced use cases.

## Core Types

### `WasmSandbox`

The main entry point for the sandbox system. Manages WebAssembly modules and instances.

```rust
use wasm_sandbox::WasmSandbox;

// Create a new sandbox
let mut sandbox = WasmSandbox::new()?;

// Load a module from bytes
let module_id = sandbox.load_module(&wasm_bytes)?;

// Create an instance
let instance_id = sandbox.create_instance(module_id, None)?;

// Call a function
let result: i32 = sandbox.call_function(instance_id, "add", &(5, 3)).await?;
```

### `InstanceConfig`

Configuration for WebAssembly instances, including security settings and resource limits.

```rust
use wasm_sandbox::{InstanceConfig, security::{Capabilities, ResourceLimits}};

let config = InstanceConfig {
    capabilities: Capabilities::minimal(),
    resource_limits: ResourceLimits::default(),
    ..Default::default()
};
```

## Security API

### `Capabilities`

Defines what operations an instance is allowed to perform.

```rust
use wasm_sandbox::security::{Capabilities, NetworkCapability, FilesystemCapability};

let mut caps = Capabilities::minimal();
caps.network = NetworkCapability::Loopback;
caps.filesystem = FilesystemCapability::ReadOnly(vec!["./data".into()]);
```

### `ResourceLimits`

Defines resource usage limits for an instance.

```rust
use wasm_sandbox::security::ResourceLimits;
use std::time::Duration;

let limits = ResourceLimits {
    max_memory: 64 * 1024 * 1024,  // 64MB
    max_cpu_time: Duration::from_secs(30),
    max_file_descriptors: 10,
    max_network_connections: 5,
    ..Default::default()
};
```

## Runtime API

### Traits

The crate uses a trait-based architecture for runtime abstraction:

- `WasmRuntime`: Main runtime trait for managing modules and instances
- `WasmModule`: Represents a compiled WebAssembly module
- `WasmInstance`: Represents a runtime instance of a module
- `WasmInstanceExt`: Extension trait for type-safe function calls

### Function Calls

Functions can be called in several ways:

```rust
// Type-safe calls (recommended)
let result: i32 = sandbox.call_function(instance_id, "add", &(5, 3)).await?;

// JSON-based calls
let result_json = sandbox.call_function_json(instance_id, "add", "[5, 3]").await?;

// MessagePack-based calls
let params = rmp_serde::to_vec(&(5, 3))?;
let result_bytes = sandbox.call_function_msgpack(instance_id, "add", &params).await?;
```

## Communication API

### `CommunicationChannel`

Bidirectional communication channel between host and guest.

```rust
use wasm_sandbox::communication::{MemoryChannel, CommunicationChannel};

let channel = MemoryChannel::new(1024)?;
channel.send(b"Hello from host")?;
let response = channel.receive()?;
```

### `RpcChannel`

Higher-level RPC abstraction for structured communication.

```rust
use wasm_sandbox::communication::{JsonRpcChannel, RpcChannel};

let channel = JsonRpcChannel::new(underlying_channel);
let response: MyResponse = channel.call("my_method", &MyRequest { ... }).await?;
```

## Wrapper API

### `HttpServerWrapper`

Wrapper for HTTP server applications.

```rust
use wasm_sandbox::wrappers::HttpServerWrapper;

let wrapper = HttpServerWrapper::new()?;
let server_id = wrapper.start_server(server_spec).await?;
```

### `McpServerWrapper`

Wrapper for Model Context Protocol servers.

```rust
use wasm_sandbox::wrappers::McpServerWrapper;

let wrapper = McpServerWrapper::new()?;
let server_id = wrapper.start_server(server_spec).await?;
```

### `CliWrapper`

Wrapper for command-line applications.

```rust
use wasm_sandbox::wrappers::CliWrapper;

let wrapper = CliWrapper::new()?;
let result = wrapper.run_cli_app(app_spec).await?;
```

## Error Handling

The crate uses a comprehensive error type system:

```rust
use wasm_sandbox::error::{Error, Result};

// All functions return Result<T, Error>
match sandbox.load_module(&invalid_wasm) {
    Ok(module_id) => println!("Module loaded: {}", module_id),
    Err(Error::InvalidModule(msg)) => println!("Invalid module: {}", msg),
    Err(Error::SecurityViolation(msg)) => println!("Security violation: {}", msg),
    Err(err) => println!("Other error: {}", err),
}
```

## Async Support

The crate is fully async-compatible:

```rust
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sandbox = WasmSandbox::new()?;
    
    // All operations are async
    let module_id = sandbox.load_module(&wasm_bytes)?;
    let instance_id = sandbox.create_instance(module_id, None)?;
    let result: i32 = sandbox.call_function(instance_id, "compute", &42).await?;
    
    Ok(())
}
```

## Feature Flags

The crate supports optional features:

```toml
[dependencies]
wasm-sandbox = { version = "0.1.0", features = ["all-runtimes"] }
```

Available features:
- `wasmtime-runtime` (default): Enable Wasmtime runtime
- `wasmer-runtime`: Enable Wasmer runtime
- `all-runtimes`: Enable all available runtimes

## Best Practices

1. **Always use resource limits** in production environments
2. **Validate all inputs** to guest functions
3. **Use minimal capabilities** for maximum security
4. **Monitor resource usage** for performance optimization
5. **Handle errors gracefully** with proper error reporting
6. **Use type-safe APIs** when possible for better reliability

## Examples

See the `examples/` directory for complete working examples:

- `http_server.rs`: HTTP server sandboxing
- `mcp_server.rs`: MCP server sandboxing
- `cli_wrapper.rs`: CLI application sandboxing
