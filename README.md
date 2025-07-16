# wasm-sandbox

[![Crates.io](https://img.shields.io/crates/v/wasm-sandbox)](https://crates.io/crates/wasm-sandbox)
[![Documentation](https://docs.rs/wasm-sandbox/badge.svg)](https://docs.rs/wasm-sandbox)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A secure WebAssembly sandbox for running untrusted code with **dead-simple ease of use**, flexible host-guest communication, comprehensive resource limits, and capability-based security.

## ✨ NEW in v0.3.0: Ease-of-Use Revolution

We've completely reimagined the developer experience with **progressive complexity APIs**:

### 🚀 One-Line Execution (Dead Simple)

```rust
// Auto-compile and run in one line!
let result: i32 = wasm_sandbox::run("calculator.rs", "add", &(5, 3)).await?;
```

### ⏱️ With Timeout (Simple + Safe)

```rust
let result: String = wasm_sandbox::run_with_timeout(
    "processor.py", "process", &"data", Duration::from_secs(30)
).await?;
```

### 🎛️ Builder Pattern (More Control)

```rust
let sandbox = WasmSandbox::builder()
    .source("my_program.rs")
    .timeout_duration(Duration::from_secs(60))
    .memory_limit(64 * 1024 * 1024)
    .enable_file_access(false)
    .build().await?;
```

## Quick Links

📖 **[Complete Documentation](docs/README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)** | 🔧 **[Examples](examples/README.md)** | 💬 **[Discussions](https://github.com/ciresnave/wasm-sandbox/discussions)**

## Key Features

- **🔒 Security First**: Isolate untrusted code in WebAssembly sandboxes with fine-grained capability controls
- **🚀 High Performance**: Efficient host-guest communication with minimal overhead
- **🔧 Flexible APIs**: High-level convenience APIs and low-level control for advanced use cases
- **📦 Multiple Runtimes**: Support for Wasmtime and Wasmer WebAssembly runtimes
- **🌐 Application Wrappers**: Built-in support for HTTP servers, MCP servers, and CLI tools
- **📊 Resource Control**: Memory, CPU, network, and filesystem limits with monitoring
- **🔄 Async/Await**: Full async support for non-blocking operations

## ✨ NEW Major Additions in v0.4.0

### 🧩 WebAssembly Component Model

Support for the next evolution of WebAssembly with interface-driven development, multi-language components, and type-safe interactions. [Learn more](docs/guides/COMPONENT_MODEL.md)

### 🐍 Python Language Bindings

Use wasm-sandbox from Python applications with a familiar API and seamless integration. [Learn more](docs/bindings/PYTHON_BINDINGS.md)

### 📊 Streaming APIs for Large Data

Process datasets larger than memory with efficient streaming APIs for real-time data handling. [Learn more](docs/guides/STREAMING_APIS.md)

## Usage Examples

### Ultra-Simple Usage (Coming in v0.3.0)

```rust
// One line: auto-compile and run
let result: i32 = wasm_sandbox::run("./calculator.rs", "add", &(5, 3))?;

// Auto-compile project and reuse
let sandbox = WasmSandbox::from_source("./my_project/")?;
let result = sandbox.call("process_data", &input)?;
```

### Current API (v0.2.0)

#### Basic Sandbox Usage

```rust
use wasm_sandbox::{WasmSandbox, SandboxConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the sandbox with default configuration
    let mut sandbox = WasmSandbox::new()?;
    
    // Load a WebAssembly module
    let wasm_bytes = std::fs::read("my_module.wasm")?;
    let module_id = sandbox.load_module(&wasm_bytes)?;
    
    // Create an instance of the module
    let instance_id = sandbox.create_instance(module_id, None)?;
    
    // Call a function in the module
    let result: String = sandbox.call_function(instance_id, "greet", &"World").await?;
    
    println!("Result: {}", result);
    
    Ok(())
}
```

### Sandboxing an HTTP Server

```rust
use wasm_sandbox::{
    WasmSandbox, InstanceConfig, 
    security::{Capabilities, NetworkCapability}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sandbox = WasmSandbox::new()?;
    
    // Load module
    let wasm_bytes = std::fs::read("http_server.wasm")?;
    let module_id = sandbox.load_module(&wasm_bytes)?;
    
    // Configure instance with network capabilities
    let mut capabilities = Capabilities::minimal();
    capabilities.network = NetworkCapability::Loopback;
    
    let instance_config = InstanceConfig {
        capabilities,
        ..InstanceConfig::default()
    };
    
    // Create instance
    let instance_id = sandbox.create_instance(module_id, Some(instance_config))?;
    
    // Start the HTTP server
    let port: u16 = sandbox.call_function(instance_id, "start", &8080).await?;
    
    println!("HTTP server running on port {}", port);
    
    // Wait for user to press Enter
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    // Stop the server
    let _: () = sandbox.call_function(instance_id, "stop", &()).await?;
    
    Ok(())
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
wasm-sandbox = "0.2.0"
```

For all features including Wasmer runtime support:

```toml
[dependencies]
wasm-sandbox = { version = "0.2.0", features = ["all-runtimes"] }
```

## Architecture Overview

The crate features a **trait-based architecture** with two main patterns:

- **Dyn-Compatible Core Traits**: `WasmRuntime`, `WasmInstance`, `WasmModule` - can be used as trait objects
- **Extension Traits**: `WasmRuntimeExt`, `WasmInstanceExt` - provide async and generic operations

This design allows for maximum flexibility while maintaining type safety. See [`docs/design/TRAIT_DESIGN.md`](docs/design/TRAIT_DESIGN.md) for detailed information.

## Quick Start

```rust
use wasm_sandbox::WasmSandbox;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new sandbox
    let mut sandbox = WasmSandbox::new()?;
    
    // Load a WebAssembly module
    let wasm_bytes = std::fs::read("module.wasm")?;
    let module_id = sandbox.load_module(&wasm_bytes)?;
    
    // Create an instance with default security settings
    let instance_id = sandbox.create_instance(module_id, None)?;
    
    // Call a function
    let result: i32 = sandbox.call_function(instance_id, "add", &(5, 3)).await?;
    println!("5 + 3 = {}", result);
    
    Ok(())
}
```

## Advanced Features

### Security Configuration

```rust
use wasm_sandbox::{WasmSandbox, InstanceConfig};
use wasm_sandbox::security::{Capabilities, NetworkCapability, FilesystemCapability};

let mut capabilities = Capabilities::minimal();
capabilities.network = NetworkCapability::Loopback;  // Only localhost
capabilities.filesystem = FilesystemCapability::ReadOnly(vec!["./data".into()]);

let config = InstanceConfig {
    capabilities,
    max_memory: Some(64 * 1024 * 1024), // 64MB limit
    max_execution_time: Some(std::time::Duration::from_secs(30)),
    ..Default::default()
};

let instance_id = sandbox.create_instance(module_id, Some(config))?;
```

### Resource Monitoring

```rust
use wasm_sandbox::security::ResourceLimits;

let limits = ResourceLimits {
    max_memory: 128 * 1024 * 1024,  // 128MB
    max_cpu_time: std::time::Duration::from_secs(60),
    max_file_descriptors: 10,
    max_network_connections: 5,
    ..Default::default()
};

// Monitor resource usage
let usage = sandbox.get_resource_usage(instance_id)?;
println!("Memory used: {} bytes", usage.memory_used);
```

### HTTP Server Wrapping

```rust
use wasm_sandbox::wrappers::HttpServerWrapper;

let wrapper = HttpServerWrapper::new()?;
let server_spec = wrapper.create_server_spec(
    "./my_server.wasm",
    8080,
    Some("./static".into()),
)?;

// Start the HTTP server in a sandbox
let server_id = wrapper.start_server(server_spec).await?;
println!("Server running on http://localhost:8080");
```

## Building from Source

```bash
# Clone the repository
git clone https://github.com/username/wasm-sandbox.git
cd wasm-sandbox

# Build the project
cargo build --release

# Run tests
cargo test --all-features

# Run benchmarks
cargo bench

# Build examples
cargo build --examples

# Run an example
cargo run --example http_server
```

## Examples

The repository includes several examples demonstrating different use cases:

- **[Basic Usage](examples/basic_usage.rs)** - Simple function calling and sandbox setup
- **[File Processor](examples/file_processor.rs)** - Secure file processing with filesystem limits
- **[HTTP Server](examples/http_server.rs)** - Web server running in sandbox with network controls
- **[MCP Server](examples/mcp_server.rs)** - Model Context Protocol server implementation
- **[CLI Tool](examples/cli_wrapper.rs)** - Command-line tool wrapper with I/O redirection
- **[Plugin Ecosystem](examples/plugin_ecosystem.rs)** - Generic plugin system with hot reload

**Run examples:**

```bash
cargo run --example basic_usage
cargo run --example http_server
cargo run --example file_processor
```

See [`examples/README.md`](examples/README.md) for detailed descriptions and usage instructions.

## Documentation

### For New Users

- **[Quick Start](#quick-start)** - Get up and running in minutes
- **[Installation Guide](docs/guides/MIGRATION.md)** - Detailed installation and setup
- **[Basic Tutorial](docs/guides/basic-tutorial.md)** - Step-by-step first application *(planned)*
- **[Examples](examples/README.md)** - Working code examples

### API Reference

- **[docs.rs](https://docs.rs/wasm-sandbox)** - Complete API documentation (always up-to-date)
- **[API Overview](docs/api/API.md)** - Core concepts and usage patterns
- **[Planned Improvements](docs/api/API_IMPROVEMENTS.md)** - Upcoming API changes

### Advanced Topics

- **[Trait Design](docs/design/TRAIT_DESIGN.md)** - Architecture and trait patterns
- **[Generic Plugin System](docs/design/GENERIC_PLUGIN_DESIGN.md)** - Plugin development framework
- **[Migration Guide](docs/guides/MIGRATION.md)** - Upgrading between versions
- **[Security Configuration](docs/guides/security-config.md)** - Capability and resource management *(planned)*

### Complete Documentation Index

**[📖 Browse All Documentation](docs/README.md)** - Organized by category with detailed navigation

## Architecture

The crate is organized into several key modules:

- **Runtime**: WebAssembly runtime abstraction (Wasmtime, Wasmer)
- **Security**: Capability-based security and resource limits
- **Communication**: Host-guest communication channels and RPC
- **Wrappers**: Application-specific wrappers and templates
- **Compiler**: WebAssembly compilation utilities

## Performance

Benchmarks show excellent performance characteristics:

- **Function calls**: < 1μs overhead for simple function calls
- **Memory communication**: > 1GB/s throughput for large data transfers
- **Startup time**: < 10ms for typical modules
- **Resource monitoring**: < 0.1% CPU overhead

Run `cargo bench` to see detailed performance metrics.

## Contributing

We welcome contributions! Please see:

- **[Contributing Guidelines](CONTRIBUTING.md)** - How to contribute to the project
- **[Documentation Index](docs/README.md)** - Complete documentation for developers
- **[API Improvements](docs/api/API_IMPROVEMENTS.md)** - Priority development areas
- **[Design Documents](docs/design/)** - Architecture and design decisions

For questions and discussions, use [GitHub Discussions](https://github.com/ciresnave/wasm-sandbox/discussions).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Safety and Security

This crate uses WebAssembly's sandboxing capabilities to provide security isolation. However:

- Always validate input to guest functions
- Set appropriate resource limits for your use case
- Review WebAssembly modules before execution
- Consider additional security measures for production use

For security-sensitive applications, consider using additional sandboxing layers such as containers or process isolation.
