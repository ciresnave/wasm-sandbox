# wasm-sandbox Examples

📖 **[← Back to Documentation](../docs/README.md)** | 🏠 **[← Main README](../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This directory contains comprehensive examples demonstrating how to use wasm-sandbox for various use cases.

## Example Overview

- [`basic_usage.rs`](basic_usage.rs) - Simple function calls and basic sandbox setup
- [`file_processor.rs`](file_processor.rs) - Sandboxing a file processing tool (addresses PUP feedback)
- [`ml_model.rs`](ml_model.rs) - Running an ML model safely in sandbox
- [`advanced_capabilities.rs`](advanced_capabilities.rs) - Fine-grained security controls
- [`streaming_execution.rs`](streaming_execution.rs) - Handling large datasets with streaming
- [`error_handling.rs`](error_handling.rs) - Comprehensive error handling patterns
- [`resource_monitoring.rs`](resource_monitoring.rs) - Monitoring and limiting resource usage
- [`plugin_ecosystem.rs`](plugin_ecosystem.rs) - Building a plugin system (PUP-style)

## Running Examples

```bash
# Basic usage
cargo run --example basic_usage

# File processor (real-world scenario)
cargo run --example file_processor

# ML model execution
cargo run --example ml_model

# Advanced security capabilities
cargo run --example advanced_capabilities
```

## Integration Patterns

### For Plugin Systems (like PUP)

See [`plugin_ecosystem.rs`](plugin_ecosystem.rs) for a complete example of building a plugin system with:

- Plugin validation and security scanning
- Hot-reload capabilities
- Resource monitoring and limits
- Type-safe plugin communication

### For File Processing

See [`file_processor.rs`](file_processor.rs) for secure file processing with:

- Read-only input directories
- Write-only output directories
- File size limits
- Processing timeouts

### For ML/AI Workloads

See [`ml_model.rs`](ml_model.rs) for ML model execution with:

- Memory limits for large models
- GPU resource management (when available)
- Model caching and hot-swapping
- Streaming inference

## Common Patterns

### Configuration Builder Pattern

```rust
use wasm_sandbox::{WasmSandbox, SandboxConfig, InstanceConfig};
use wasm_sandbox::security::{Capabilities, ResourceLimits, FilesystemCapability};
use std::time::Duration;

let instance_config = InstanceConfig {
    capabilities: Capabilities {
        filesystem: vec![
            FilesystemCapability::ReadOnly("/input".into()),
            FilesystemCapability::WriteOnly("/output".into()),
        ],
        network: vec![], // No network access
        ..Capabilities::minimal()
    },
    resource_limits: ResourceLimits {
        memory_bytes: Some(64 * 1024 * 1024), // 64MB
        execution_timeout: Some(Duration::from_secs(30)),
        max_fuel: Some(1_000_000), // Computational limit
        ..ResourceLimits::default()
    },
};
```

### Error Handling Pattern

```rust
use wasm_sandbox::Error;

match sandbox.execute_function(instance_id, "process", &params).await {
    Ok(result) => { /* handle success */ },
    Err(Error::Security(msg)) => {
        eprintln!("Security violation: {}", msg);
        // Log security incident, possibly terminate sandbox
    },
    Err(Error::ResourceLimit { kind, limit, used }) => {
        eprintln!("Resource limit exceeded: {} used {}/{}", kind, used, limit);
        // Handle resource exhaustion, possibly scale resources
    },
    Err(Error::WasmRuntime(e)) => {
        eprintln!("WASM execution error: {}", e);
        // Handle runtime errors, possibly restart instance
    },
    Err(Error::Configuration(msg)) => {
        eprintln!("Configuration error: {}", msg);
        // Fix configuration and retry
    },
}
```

### Async Streaming Pattern

```rust
use wasm_sandbox::runtime::WasmInstanceExt;
use futures::StreamExt;

// Process data in chunks
let mut stream = data_stream.chunks(1000);
while let Some(chunk) = stream.next().await {
    let result = instance.call_function("process_chunk", &chunk).await?;
    // Handle result...
}
```

## Best Practices

1. **Always set resource limits** - Prevent runaway processes
2. **Use minimal capabilities** - Start with least privilege
3. **Handle errors specifically** - Don't catch all errors the same way
4. **Monitor resource usage** - Track memory, CPU, and execution time
5. **Validate input/output** - Don't trust sandboxed code implicitly
6. **Use extension traits** - For type-safe, async operations
7. **Implement timeout handling** - For production reliability

## Troubleshooting

### Common Issues

1. **"trait WasmInstanceExt is not implemented"**
   - Solution: `use wasm_sandbox::runtime::WasmInstanceExt;`

2. **"async function without .await"**
   - Solution: Extension trait methods are async: `result.await?`

3. **"security violation" errors**
   - Solution: Check capabilities configuration, ensure required permissions

4. **Memory limit errors**
   - Solution: Increase `memory_bytes` in `ResourceLimits`

5. **Timeout errors**
   - Solution: Increase `execution_timeout` or optimize WASM code

### Debug Mode

```rust
let sandbox = WasmSandbox::new()?
    .with_debug(true)  // Enable detailed logging
    .with_profiling(true);  // Enable performance profiling
```

### Performance Tips

1. **Reuse instances** - Creating instances is expensive
2. **Use fuel limits** - For fine-grained execution control
3. **Enable JIT** - For compute-intensive workloads
4. **Pool connections** - For high-throughput scenarios
5. **Cache modules** - Avoid recompiling the same WASM

See individual example files for detailed implementations of these patterns.
