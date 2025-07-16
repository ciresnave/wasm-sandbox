# Extending wasm-sandbox

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)**

This guide explains how to extend wasm-sandbox with new features, runtime backends, communication channels, and application wrappers. It's intended for developers who want to contribute to the core codebase or build custom extensions.

## Table of Contents

- [Extension Points](#extension-points)
- [Adding a New Runtime Backend](#adding-a-new-runtime-backend)
- [Creating Custom Communication Channels](#creating-custom-communication-channels)
- [Building Application Wrappers](#building-application-wrappers)
- [Adding Security Capabilities](#adding-security-capabilities)
- [Creating Custom Templates](#creating-custom-templates)
- [Extension Best Practices](#extension-best-practices)

## Extension Points

wasm-sandbox is designed to be highly extensible, with several well-defined extension points:

1. **Runtime Backends**: Support for different WebAssembly runtimes (Wasmtime, Wasmer, etc.)
2. **Communication Channels**: Methods for host-guest communication
3. **Application Wrappers**: Pre-built templates for specific application types
4. **Security Capabilities**: Permission types and enforcement mechanisms
5. **Resource Limits**: Configurable resource constraints
6. **Template Renderers**: Code generation for wrappers

The project follows a plugin-based architecture with traits (interfaces) defining the extension points. This allows for easy addition of new functionality without modifying the core code.

## Adding a New Runtime Backend

The `Runtime` trait defines the interface for WebAssembly runtime backends. To add a new runtime:

1. Create a new module in `src/runtime/`
2. Implement the `Runtime` trait
3. Register the backend with the runtime factory

### Step 1: Create a new module file

```rust
// src/runtime/new_runtime.rs

use std::path::Path;
use std::sync::Arc;

use crate::error::Result;
use crate::runtime::{Instance, Module, Runtime};
use crate::security::{Capabilities, ResourceLimits};

/// Runtime implementation for NewRuntime
pub struct NewRuntimeBackend {
    // Backend-specific fields
}

impl NewRuntimeBackend {
    /// Create a new instance of the runtime
    pub fn new() -> Result<Self> {
        // Initialize the runtime
        Ok(Self {
            // Initialize fields
        })
    }
}
```

### Step 2: Implement the Runtime trait

```rust
impl Runtime for NewRuntimeBackend {
    fn compile<P: AsRef<Path>>(&self, wasm_file: P) -> Result<Arc<dyn Module>> {
        // Compile WebAssembly module
        // ...
        
        Ok(Arc::new(NewRuntimeModule { /* ... */ }))
    }

    fn compile_from_bytes(&self, wasm_bytes: &[u8]) -> Result<Arc<dyn Module>> {
        // Compile WebAssembly module from bytes
        // ...
        
        Ok(Arc::new(NewRuntimeModule { /* ... */ }))
    }

    fn instantiate(&self, module: Arc<dyn Module>, capabilities: Capabilities, 
                  resource_limits: ResourceLimits) -> Result<Box<dyn Instance>> {
        // Create a new instance of the module
        // ...
        
        Ok(Box::new(NewRuntimeInstance { /* ... */ }))
    }

    fn name(&self) -> &'static str {
        "new_runtime"
    }
}

/// Module implementation for NewRuntime
struct NewRuntimeModule {
    // Module-specific fields
}

impl Module for NewRuntimeModule {
    // Implement Module trait methods
    // ...
}

/// Instance implementation for NewRuntime
struct NewRuntimeInstance {
    // Instance-specific fields
}

impl Instance for NewRuntimeInstance {
    // Implement Instance trait methods
    // ...
}
```

### Step 3: Register the backend with the runtime factory

```rust
// src/runtime/mod.rs

// Add module to exports
pub mod new_runtime;
pub use new_runtime::NewRuntimeBackend;

impl RuntimeFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            backends: HashMap::new(),
        };

        // Register existing backends
        factory.register("wasmtime", || Box::new(WasmtimeBackend::new()?));
        factory.register("wasmer", || Box::new(WasmerBackend::new()?));
        
        // Register new backend
        factory.register("new_runtime", || Box::new(NewRuntimeBackend::new()?));
        
        factory
    }
}
```

### Step 4: Add feature flag (optional)

For optional runtimes, add a feature flag in `Cargo.toml`:

```toml
[features]
default = ["wasmtime"]
wasmtime = ["dep:wasmtime"]
wasmer = ["dep:wasmer"]
new_runtime = ["dep:new-runtime-dependency"]
```

Then conditionally compile the backend:

```rust
#[cfg(feature = "new_runtime")]
pub mod new_runtime;

#[cfg(feature = "new_runtime")]
pub use new_runtime::NewRuntimeBackend;

impl RuntimeFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            backends: HashMap::new(),
        };

        // Register existing backends
        #[cfg(feature = "wasmtime")]
        factory.register("wasmtime", || Box::new(WasmtimeBackend::new()?));
        
        #[cfg(feature = "wasmer")]
        factory.register("wasmer", || Box::new(WasmerBackend::new()?));
        
        // Register new backend
        #[cfg(feature = "new_runtime")]
        factory.register("new_runtime", || Box::new(NewRuntimeBackend::new()?));
        
        factory
    }
}
```

## Creating Custom Communication Channels

The `Channel` trait defines the interface for host-guest communication channels. To add a new channel:

1. Create a new module in `src/communication/`
2. Implement the `Channel` trait
3. Register the channel with the channel factory

### Step 1: Create a new module file

```rust
// src/communication/custom_channel.rs

use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

use crate::error::Result;
use crate::communication::Channel;

/// A custom communication channel
pub struct CustomChannel<T, R> 
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    // Channel-specific fields
    data: Arc<Mutex<Vec<T>>>,
    results: Arc<Mutex<Vec<R>>>,
}

impl<T, R> CustomChannel<T, R> 
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    /// Create a new custom channel
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
```

### Step 2: Implement the Channel trait

```rust
impl<T, R> Channel<T, R> for CustomChannel<T, R>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    fn send(&self, data: T) -> Result<()> {
        let mut guard = self.data.lock().unwrap();
        guard.push(data);
        Ok(())
    }

    fn receive(&self) -> Result<Option<T>> {
        let mut guard = self.data.lock().unwrap();
        Ok(guard.pop())
    }

    fn send_result(&self, result: R) -> Result<()> {
        let mut guard = self.results.lock().unwrap();
        guard.push(result);
        Ok(())
    }

    fn receive_result(&self) -> Result<Option<R>> {
        let mut guard = self.results.lock().unwrap();
        Ok(guard.pop())
    }

    fn channel_type(&self) -> &'static str {
        "custom"
    }
}
```

### Step 3: Register the channel with the channel factory

```rust
// src/communication/mod.rs

// Add module to exports
pub mod custom_channel;
pub use custom_channel::CustomChannel;

impl ChannelFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            channels: HashMap::new(),
        };

        // Register existing channels
        factory.register("memory", || Box::new(MemoryChannel::new()));
        factory.register("rpc", || Box::new(RpcChannel::new()));
        
        // Register new channel
        factory.register("custom", || Box::new(CustomChannel::new()));
        
        factory
    }
}
```

### Step 4: Use the custom channel

```rust
use wasm_sandbox::communication::CustomChannel;
use wasm_sandbox::sandbox::Sandbox;

// Create sandbox with custom channel
let sandbox = Sandbox::builder()
    .module_path("path/to/module.wasm")
    .communication_channel(Box::new(CustomChannel::<MyRequest, MyResponse>::new()))
    .build()?;
```

## Building Application Wrappers

Application wrappers provide higher-level abstractions for common use cases. To add a new wrapper:

1. Create a new module in `src/wrappers/`
2. Define the wrapper interface
3. Implement the wrapper logic

### Step 1: Create a new module file

```rust
// src/wrappers/custom_wrapper.rs

use std::path::Path;
use serde::{Serialize, Deserialize};

use crate::error::Result;
use crate::sandbox::Sandbox;
use crate::security::{Capabilities, SecurityPolicy};

/// Configuration for the custom wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomWrapperConfig {
    pub module_path: String,
    pub entry_point: String,
    pub max_concurrent_requests: usize,
    pub timeout_ms: u64,
}

/// Custom application wrapper
pub struct CustomWrapper {
    sandbox: Sandbox,
    config: CustomWrapperConfig,
}

impl CustomWrapper {
    /// Create a new custom wrapper
    pub fn new<P: AsRef<Path>>(config: CustomWrapperConfig) -> Result<Self> {
        // Create security policy
        let security = SecurityPolicy::strict()
            .allow_capability(Capabilities::network_access(&["api.example.com"]));

        // Create sandbox
        let sandbox = Sandbox::builder()
            .module_path(&config.module_path)
            .security_policy(security)
            .memory_limit(64 * 1024 * 1024) // 64MB
            .build()?;

        Ok(Self {
            sandbox,
            config,
        })
    }

    /// Process a request
    pub async fn process_request<T, R>(&self, request: T) -> Result<R>
    where
        T: Serialize + Send + 'static,
        R: for<'de> Deserialize<'de> + Send + 'static,
    {
        // Call the entry point
        let result = self.sandbox.call_async::<T, R>(&self.config.entry_point, &request).await?;
        Ok(result)
    }
}
```

### Step 2: Implement wrapper-specific functionality

```rust
impl CustomWrapper {
    /// Custom wrapper-specific functionality
    pub async fn perform_operation(&self, data: &str) -> Result<String> {
        // Custom logic
        let request = CustomRequest {
            data: data.to_string(),
            operation: "process",
        };
        
        let response: CustomResponse = self.process_request(request).await?;
        Ok(response.result)
    }
    
    /// Start the wrapper service
    pub async fn start_service(&self) -> Result<()> {
        // Service implementation
        println!("Starting custom wrapper service...");
        
        // Implementation-specific code
        
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct CustomRequest {
    data: String,
    operation: &'static str,
}

#[derive(Debug, Deserialize)]
struct CustomResponse {
    result: String,
    status: String,
}
```

### Step 3: Add code generation support

To support generating code for your wrapper:

```rust
// src/templates/custom_wrapper.rs

use handlebars::Handlebars;
use serde_json::json;

use crate::error::Result;
use crate::templates::TemplateRenderer;

/// Template renderer for custom wrapper
pub struct CustomWrapperTemplate;

impl TemplateRenderer for CustomWrapperTemplate {
    fn render(&self, config: serde_json::Value) -> Result<String> {
        let mut handlebars = Handlebars::new();
        
        // Register template
        handlebars.register_template_string("custom_wrapper", include_str!("../../fixtures/custom_wrapper_template.rs.txt"))?;
        
        // Render template
        let rendered = handlebars.render("custom_wrapper", &config)?;
        
        Ok(rendered)
    }
}
```

Create a template file:

```rust
// fixtures/custom_wrapper_template.rs.txt

// Generated custom wrapper for {{name}}

use wasm_sandbox::wrappers::CustomWrapper;
use wasm_sandbox::error::{Result, Error};
use serde::{Serialize, Deserialize};

/// Custom request type
#[derive(Debug, Serialize)]
pub struct Request {
    // Request fields
    pub data: String,
    pub operation: String,
}

/// Custom response type
#[derive(Debug, Deserialize)]
pub struct Response {
    // Response fields
    pub result: String,
    pub status: String,
}

/// Create a new custom wrapper
pub fn create_wrapper() -> Result<CustomWrapper> {
    let config = CustomWrapperConfig {
        module_path: "{{module_path}}".to_string(),
        entry_point: "{{entry_point}}".to_string(),
        max_concurrent_requests: {{max_concurrent_requests}},
        timeout_ms: {{timeout_ms}},
    };
    
    CustomWrapper::new(config)
}

/// Process a request
pub async fn process_request(data: &str) -> Result<String> {
    let wrapper = create_wrapper()?;
    wrapper.perform_operation(data).await
}
```

### Step 4: Add module to exports

```rust
// src/wrappers/mod.rs

// Add module to exports
pub mod custom_wrapper;
pub use custom_wrapper::CustomWrapper;
```

## Adding Security Capabilities

The security system is extensible through the `Capability` trait. To add a new capability:

1. Create a new capability implementation
2. Add enforcement logic
3. Register the capability with the security system

### Step 1: Define the capability

```rust
// src/security/capabilities.rs

/// Custom capability for specialized access
#[derive(Debug, Clone)]
pub struct CustomCapability {
    pub resources: Vec<String>,
    pub read_only: bool,
    pub max_operations: usize,
}

impl Capability for CustomCapability {
    fn capability_type(&self) -> &'static str {
        "custom_access"
    }
    
    fn validate(&self) -> Result<()> {
        // Validate the capability configuration
        if self.resources.is_empty() {
            return Err(Error::InvalidCapability("Custom capability must have at least one resource".into()));
        }
        
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Capabilities {
    /// Create a new custom capability
    pub fn custom_access(resources: &[&str], read_only: bool) -> Box<dyn Capability> {
        Box::new(CustomCapability {
            resources: resources.iter().map(|s| s.to_string()).collect(),
            read_only,
            max_operations: 100,
        })
    }
}
```

### Step 2: Add enforcement logic

```rust
// src/security/audit_impl.rs

impl SecurityAuditor {
    pub fn audit_operation(&self, operation: &str, args: &[Value]) -> Result<()> {
        // Existing capability checks
        // ...
        
        // Custom capability check
        if operation.starts_with("custom_") {
            self.check_custom_access(operation, args)?;
        }
        
        Ok(())
    }
    
    fn check_custom_access(&self, operation: &str, args: &[Value]) -> Result<()> {
        let capability = self.get_capability::<CustomCapability>("custom_access")
            .ok_or_else(|| Error::CapabilityNotGranted("custom_access".into()))?;
            
        // Extract resource name from args
        let resource = args.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidArgument("Expected resource name as first argument".into()))?;
            
        // Check if the resource is allowed
        if !capability.resources.iter().any(|r| resource.starts_with(r)) {
            return Err(Error::AccessDenied(format!("Access to resource '{}' denied", resource)));
        }
        
        // Check read-only mode
        if capability.read_only && !operation.starts_with("custom_read_") {
            return Err(Error::AccessDenied("Write operations not allowed with read-only capability".into()));
        }
        
        // Check operation count
        // ...
        
        Ok(())
    }
}
```

### Step 3: Use the new capability

```rust
use wasm_sandbox::security::{Capabilities, SecurityPolicy};
use wasm_sandbox::sandbox::Sandbox;

// Create a policy with the custom capability
let policy = SecurityPolicy::strict()
    .allow_capability(Capabilities::custom_access(&["resource1", "resource2"], true));

// Create sandbox with the policy
let sandbox = Sandbox::builder()
    .module_path("path/to/module.wasm")
    .security_policy(policy)
    .build()?;
```

## Creating Custom Templates

The templating system allows adding new code generation templates:

1. Create a template file
2. Add a template renderer
3. Register the template with the template factory

### Step 1: Create a template file

```
// fixtures/my_template.rs.txt

// Generated code for {{name}}

use wasm_sandbox::sandbox::Sandbox;
use wasm_sandbox::error::Result;

/// Create a new sandbox for {{name}}
pub fn create_sandbox() -> Result<Sandbox> {
    Sandbox::builder()
        .module_path("{{module_path}}")
        .memory_limit({{memory_limit}})
        .build()
}

/// Run the main function
pub async fn run() -> Result<String> {
    let sandbox = create_sandbox()?;
    let result: String = sandbox.call_async("{{entry_point}}", &())?;
    Ok(result)
}
```

### Step 2: Add a template renderer

```rust
// src/templates/my_template.rs

use handlebars::Handlebars;
use serde_json::json;

use crate::error::Result;
use crate::templates::TemplateRenderer;

/// Template renderer for my template
pub struct MyTemplateRenderer;

impl TemplateRenderer for MyTemplateRenderer {
    fn render(&self, config: serde_json::Value) -> Result<String> {
        let mut handlebars = Handlebars::new();
        
        // Register template
        handlebars.register_template_string("my_template", include_str!("../../fixtures/my_template.rs.txt"))?;
        
        // Render template
        let rendered = handlebars.render("my_template", &config)?;
        
        Ok(rendered)
    }
}
```

### Step 3: Register the template with the template factory

```rust
// src/templates/mod.rs

// Add module to exports
pub mod my_template;
pub use my_template::MyTemplateRenderer;

impl TemplateFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            renderers: HashMap::new(),
        };

        // Register existing templates
        factory.register("http_server", Box::new(HttpServerTemplate));
        factory.register("cli_tool", Box::new(CliToolTemplate));
        
        // Register new template
        factory.register("my_template", Box::new(MyTemplateRenderer));
        
        factory
    }
}
```

### Step 4: Use the custom template

```rust
use wasm_sandbox::templates::TemplateFactory;
use serde_json::json;

// Create template factory
let factory = TemplateFactory::new();

// Render template
let config = json!({
    "name": "MyApp",
    "module_path": "path/to/module.wasm",
    "entry_point": "main",
    "memory_limit": 64 * 1024 * 1024,
});

let rendered = factory.render("my_template", config)?;

// Save to file
std::fs::write("my_app.rs", rendered)?;
```

## Extension Best Practices

### General Principles

1. **Follow the trait interfaces**: Ensure your extensions implement all required trait methods
2. **Maintain backward compatibility**: Avoid breaking changes to existing interfaces
3. **Add comprehensive testing**: Write unit and integration tests for your extensions
4. **Document your extensions**: Add documentation comments to all public APIs
5. **Handle errors properly**: Use the error system correctly and provide helpful error messages

### Runtime Backend Best Practices

1. Properly manage WebAssembly resources to avoid memory leaks
2. Implement all security checks and resource limits
3. Consider performance implications, especially for instance creation
4. Provide diagnostics and debugging information
5. Handle edge cases (e.g., module compilation failures, instance errors)

### Communication Channel Best Practices

1. Ensure thread safety for concurrent access
2. Implement efficient serialization/deserialization
3. Handle backpressure for high-volume communication
4. Provide clear error messages for communication failures
5. Consider adding monitoring/metrics for channel performance

### Application Wrapper Best Practices

1. Keep the API simple and consistent with other wrappers
2. Provide sensible defaults but allow customization
3. Implement proper error handling and reporting
4. Add comprehensive documentation and examples
5. Consider security implications of your wrapper design

### Security Extension Best Practices

1. Follow the principle of least privilege
2. Add comprehensive validation logic
3. Provide detailed security violation messages
4. Consider performance impact of security checks
5. Document security implications clearly

## Conclusion

Extending wasm-sandbox allows for customization to specific use cases while benefiting from the core sandbox infrastructure. By following the trait-based extension patterns, you can add new capabilities without modifying the core codebase.

For more detailed information about each extension point, refer to the API documentation and trait definitions in the source code.

---

**Community Extensions**: If you've developed a useful extension for wasm-sandbox, consider contributing it back to the main repository or publishing it as a separate crate.
