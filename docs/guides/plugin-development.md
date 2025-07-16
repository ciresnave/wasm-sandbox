# Plugin Development Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This comprehensive guide covers building secure, high-performance plugin systems using wasm-sandbox, including hot-reload capabilities, plugin discovery, dependency management, and sandboxed execution.

## Plugin Architecture Overview

wasm-sandbox enables building plugin systems with:

1. **Security Isolation** - Each plugin runs in its own WebAssembly sandbox
2. **Hot Reload** - Update plugins without restarting the host application
3. **Type Safety** - Strong typing for plugin interfaces and communication
4. **Resource Management** - Fine-grained control over plugin resource usage
5. **Dependency Management** - Manage plugin dependencies and versions
6. **Plugin Discovery** - Automatic discovery and loading of plugins

## Quick Start - Basic Plugin System

```rust
use wasm_sandbox::{WasmSandbox, PluginManager, PluginInterface};
use serde::{Serialize, Deserialize};

// Define plugin interface
#[derive(Serialize, Deserialize)]
pub struct PluginRequest {
    pub action: String,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct PluginResponse {
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create plugin manager
    let mut plugin_manager = PluginManager::new()
        .plugin_directory("./plugins")
        .enable_hot_reload(true)
        .max_plugins(100)
        .build()
        .await?;
    
    // Load all plugins
    plugin_manager.discover_and_load().await?;
    
    // Execute plugin function
    let request = PluginRequest {
        action: "process_data".to_string(),
        data: serde_json::json!({"input": "hello world"}),
    };
    
    let response: PluginResponse = plugin_manager
        .execute_plugin("text_processor", "handle_request", &request)
        .await?;
    
    println!("Plugin result: {:?}", response);
    
    Ok(())
}
```

## Plugin Interface Design

### Core Plugin Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize plugin with configuration
    async fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError>;
    
    /// Handle plugin requests
    async fn handle_request(&self, request: &PluginRequest) -> Result<PluginResponse, PluginError>;
    
    /// Cleanup resources on shutdown
    async fn shutdown(&mut self) -> Result<(), PluginError>;
    
    /// Check if plugin is healthy
    async fn health_check(&self) -> Result<PluginHealth, PluginError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<PluginDependency>,
    pub min_runtime_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version_req: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub settings: serde_json::Value,
    pub resources: ResourceLimits,
    pub capabilities: Vec<String>,
    pub environment: std::collections::HashMap<String, String>,
}
```

### Plugin Manager Implementation

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use notify::{Watcher, RecursiveMode, watcher};

pub struct PluginManager {
    plugins: RwLock<HashMap<String, PluginInstance>>,
    plugin_directory: PathBuf,
    hot_reload: bool,
    max_plugins: usize,
    sandbox_factory: SandboxFactory,
    watcher: Option<notify::RecommendedWatcher>,
}

pub struct PluginInstance {
    pub metadata: PluginMetadata,
    pub sandbox: WasmSandbox,
    pub last_loaded: std::time::SystemTime,
    pub load_count: u64,
    pub execution_count: u64,
    pub last_error: Option<String>,
}

impl PluginManager {
    pub fn new() -> PluginManagerBuilder {
        PluginManagerBuilder::default()
    }
    
    pub async fn discover_and_load(&mut self) -> Result<usize, PluginError> {
        let mut loaded_count = 0;
        
        // Scan plugin directory
        let mut entries = tokio::fs::read_dir(&self.plugin_directory).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                match self.load_plugin(&path).await {
                    Ok(_) => {
                        loaded_count += 1;
                        info!("Loaded plugin: {:?}", path);
                    }
                    Err(e) => {
                        warn!("Failed to load plugin {:?}: {}", path, e);
                    }
                }
            }
        }
        
        info!("Loaded {} plugins", loaded_count);
        Ok(loaded_count)
    }
    
    pub async fn load_plugin(&mut self, path: &Path) -> Result<String, PluginError> {
        // Read plugin manifest
        let manifest_path = path.with_extension("toml");
        let manifest_content = tokio::fs::read_to_string(&manifest_path).await?;
        let metadata: PluginMetadata = toml::from_str(&manifest_content)?;
        
        // Validate plugin
        self.validate_plugin(&metadata)?;
        
        // Create sandbox for plugin
        let sandbox = self.sandbox_factory
            .create_for_plugin(&metadata)
            .await?;
        
        // Load WebAssembly module
        sandbox.load_module_from_file(path).await?;
        
        // Initialize plugin
        let config = self.create_plugin_config(&metadata).await?;
        sandbox.call("initialize", &config).await?;
        
        // Create plugin instance
        let instance = PluginInstance {
            metadata: metadata.clone(),
            sandbox,
            last_loaded: std::time::SystemTime::now(),
            load_count: 1,
            execution_count: 0,
            last_error: None,
        };
        
        // Store plugin
        let mut plugins = self.plugins.write().await;
        plugins.insert(metadata.name.clone(), instance);
        
        Ok(metadata.name)
    }
    
    pub async fn execute_plugin<T, R>(
        &self,
        plugin_name: &str,
        function: &str,
        input: &T,
    ) -> Result<R, PluginError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let plugins = self.plugins.read().await;
        
        let instance = plugins
            .get(plugin_name)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_name.to_string()))?;
        
        // Execute function in plugin sandbox
        let result = instance.sandbox.call(function, input).await?;
        
        // Update execution metrics
        drop(plugins);
        let mut plugins = self.plugins.write().await;
        if let Some(instance) = plugins.get_mut(plugin_name) {
            instance.execution_count += 1;
        }
        
        Ok(result)
    }
    
    pub async fn reload_plugin(&mut self, plugin_name: &str) -> Result<(), PluginError> {
        // Find plugin file
        let plugin_path = self.find_plugin_file(plugin_name).await?;
        
        // Shutdown existing plugin
        if let Some(instance) = self.plugins.write().await.remove(plugin_name) {
            let _ = instance.sandbox.call("shutdown", &()).await;
        }
        
        // Load new version
        self.load_plugin(&plugin_path).await?;
        
        info!("Reloaded plugin: {}", plugin_name);
        Ok(())
    }
    
    pub async fn unload_plugin(&mut self, plugin_name: &str) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write().await;
        
        if let Some(instance) = plugins.remove(plugin_name) {
            // Graceful shutdown
            let _ = instance.sandbox.call("shutdown", &()).await;
            info!("Unloaded plugin: {}", plugin_name);
            Ok(())
        } else {
            Err(PluginError::PluginNotFound(plugin_name.to_string()))
        }
    }
    
    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read().await;
        
        plugins
            .values()
            .map(|instance| PluginInfo {
                name: instance.metadata.name.clone(),
                version: instance.metadata.version.clone(),
                description: instance.metadata.description.clone(),
                status: PluginStatus::Loaded,
                execution_count: instance.execution_count,
                last_loaded: instance.last_loaded,
                last_error: instance.last_error.clone(),
            })
            .collect()
    }
}
```

### Hot Reload Implementation

```rust
use notify::{Event, EventKind, watcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

impl PluginManager {
    pub async fn enable_hot_reload(&mut self) -> Result<(), PluginError> {
        let (tx, mut rx) = mpsc::channel(100);
        
        let mut watcher = watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if let Err(e) = tx.blocking_send(event) {
                    error!("Failed to send file event: {}", e);
                }
            }
        })?;
        
        watcher.watch(&self.plugin_directory, RecursiveMode::Recursive)?;
        
        // Store watcher to keep it alive
        self.watcher = Some(watcher);
        
        // Spawn hot reload task
        let plugin_manager = Arc::clone(&self);
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Err(e) = plugin_manager.handle_file_event(event).await {
                    error!("Hot reload error: {}", e);
                }
            }
        });
        
        info!("Hot reload enabled for directory: {:?}", self.plugin_directory);
        Ok(())
    }
    
    async fn handle_file_event(&self, event: Event) -> Result<(), PluginError> {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                for path in event.paths {
                    if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                        // Extract plugin name from filename
                        if let Some(plugin_name) = path.file_stem().and_then(|s| s.to_str()) {
                            info!("Detected change in plugin: {}", plugin_name);
                            
                            // Debounce - wait a bit for file write to complete
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            
                            // Reload plugin
                            if let Err(e) = self.reload_plugin(plugin_name).await {
                                error!("Failed to reload plugin {}: {}", plugin_name, e);
                            }
                        }
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    if let Some(plugin_name) = path.file_stem().and_then(|s| s.to_str()) {
                        info!("Plugin removed: {}", plugin_name);
                        
                        if let Err(e) = self.unload_plugin(plugin_name).await {
                            error!("Failed to unload plugin {}: {}", plugin_name, e);
                        }
                    }
                }
            }
            _ => {} // Ignore other events
        }
        
        Ok(())
    }
}
```

## Security and Sandboxing

### Plugin Security Model

```rust
use wasm_sandbox::{Capability, ResourceLimits, SecurityPolicy};

pub struct PluginSandboxFactory {
    base_policy: SecurityPolicy,
    resource_limits: ResourceLimits,
}

impl PluginSandboxFactory {
    pub async fn create_for_plugin(&self, metadata: &PluginMetadata) -> Result<WasmSandbox, Error> {
        let mut builder = WasmSandbox::builder();
        
        // Apply base security policy
        builder = builder.security_policy(self.base_policy.clone());
        
        // Configure capabilities based on plugin requirements
        for capability in &metadata.capabilities {
            match capability.as_str() {
                "network" => {
                    builder = builder.allow_capability(Capability::Network)
                        .network_policy(NetworkPolicy::Restricted)
                        .allowed_hosts(&["api.example.com", "cdn.example.com"]);
                }
                "filesystem" => {
                    builder = builder.allow_capability(Capability::Filesystem)
                        .filesystem_policy(FilesystemPolicy::Sandboxed)
                        .sandbox_directory("/tmp/plugin_sandbox");
                }
                "compute" => {
                    builder = builder.allow_capability(Capability::Compute);
                }
                "storage" => {
                    builder = builder.allow_capability(Capability::Storage)
                        .storage_quota(100 * 1024 * 1024); // 100MB
                }
                _ => {
                    warn!("Unknown capability requested: {}", capability);
                }
            }
        }
        
        // Set resource limits based on plugin type
        let limits = self.calculate_resource_limits(metadata);
        builder = builder.resource_limits(limits);
        
        // Enable monitoring for plugins
        builder = builder
            .enable_resource_monitoring(true)
            .enable_security_auditing(true)
            .audit_callback(|event| {
                match event.level {
                    AuditLevel::Warning => warn!("Plugin security event: {}", event.message),
                    AuditLevel::Critical => error!("Plugin security violation: {}", event.message),
                    _ => debug!("Plugin audit: {}", event.message),
                }
            });
        
        builder.build().await
    }
    
    fn calculate_resource_limits(&self, metadata: &PluginMetadata) -> ResourceLimits {
        // Base limits for plugins
        let mut limits = ResourceLimits {
            memory_bytes: Some(64 * 1024 * 1024), // 64MB default
            max_fuel: Some(10_000_000),           // 10M instructions
            execution_timeout: Some(Duration::from_secs(30)),
            max_open_files: Some(10),
            max_network_connections: Some(5),
            ..ResourceLimits::default()
        };
        
        // Adjust based on plugin capabilities
        if metadata.capabilities.contains(&"compute".to_string()) {
            limits.memory_bytes = Some(256 * 1024 * 1024); // 256MB for compute
            limits.max_fuel = Some(100_000_000);           // 100M instructions
        }
        
        if metadata.capabilities.contains(&"network".to_string()) {
            limits.max_network_connections = Some(20);
            limits.network_timeout = Some(Duration::from_secs(60));
        }
        
        limits
    }
}
```

### Plugin Permission System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    pub read_files: Vec<String>,
    pub write_files: Vec<String>,
    pub network_hosts: Vec<String>,
    pub environment_vars: Vec<String>,
    pub system_calls: Vec<String>,
}

impl PluginPermissions {
    pub fn from_manifest(metadata: &PluginMetadata) -> Self {
        let mut permissions = Self::default();
        
        // Parse permissions from plugin metadata
        if let Some(perms) = metadata.permissions.as_ref() {
            permissions.read_files = perms.get("read_files")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            
            permissions.write_files = perms.get("write_files")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            
            permissions.network_hosts = perms.get("network_hosts")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
        }
        
        permissions
    }
    
    pub fn validate(&self) -> Result<(), PermissionError> {
        // Validate file paths are within allowed directories
        for path in &self.read_files {
            if !self.is_safe_path(path) {
                return Err(PermissionError::UnsafePath(path.clone()));
            }
        }
        
        for path in &self.write_files {
            if !self.is_safe_path(path) {
                return Err(PermissionError::UnsafePath(path.clone()));
            }
        }
        
        // Validate network hosts
        for host in &self.network_hosts {
            if !self.is_allowed_host(host) {
                return Err(PermissionError::UnallowedHost(host.clone()));
            }
        }
        
        Ok(())
    }
    
    fn is_safe_path(&self, path: &str) -> bool {
        // Prevent directory traversal
        !path.contains("..") && !path.starts_with('/')
    }
    
    fn is_allowed_host(&self, host: &str) -> bool {
        // Check against allowlist
        let allowed_domains = ["api.example.com", "cdn.example.com"];
        allowed_domains.iter().any(|&domain| host.ends_with(domain))
    }
}
```

## Plugin Communication

### Inter-Plugin Communication

```rust
use tokio::sync::mpsc;

pub struct PluginBus {
    channels: HashMap<String, mpsc::UnboundedSender<PluginMessage>>,
    subscriptions: HashMap<String, Vec<String>>, // topic -> plugin names
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessage {
    pub from: String,
    pub to: Option<String>, // None for broadcast
    pub topic: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl PluginBus {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }
    
    pub fn register_plugin(&mut self, plugin_name: String) -> mpsc::UnboundedReceiver<PluginMessage> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.channels.insert(plugin_name, tx);
        rx
    }
    
    pub fn subscribe(&mut self, plugin_name: &str, topic: &str) {
        self.subscriptions
            .entry(topic.to_string())
            .or_insert_with(Vec::new)
            .push(plugin_name.to_string());
    }
    
    pub fn publish(&self, message: PluginMessage) -> Result<(), PluginError> {
        if let Some(to) = &message.to {
            // Direct message
            if let Some(channel) = self.channels.get(to) {
                channel.send(message)?;
            }
        } else {
            // Broadcast to subscribers
            if let Some(subscribers) = self.subscriptions.get(&message.topic) {
                for subscriber in subscribers {
                    if subscriber != &message.from {
                        if let Some(channel) = self.channels.get(subscriber) {
                            let _ = channel.send(message.clone());
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

// Plugin-side API
impl PluginInstance {
    pub async fn send_message(&self, to: Option<&str>, topic: &str, payload: serde_json::Value) -> Result<(), PluginError> {
        let message = PluginMessage {
            from: self.metadata.name.clone(),
            to: to.map(String::from),
            topic: topic.to_string(),
            payload,
            timestamp: Utc::now(),
        };
        
        self.sandbox.call("send_message", &message).await
    }
    
    pub async fn subscribe_to_topic(&self, topic: &str) -> Result<(), PluginError> {
        self.sandbox.call("subscribe", &topic).await
    }
}
```

### Plugin-Host Communication

```rust
// Host-side plugin interface
impl PluginInstance {
    pub async fn call_host_function(&self, function: &str, args: &serde_json::Value) -> Result<serde_json::Value, PluginError> {
        // Create host function call request
        let request = HostFunctionCall {
            function: function.to_string(),
            args: args.clone(),
            plugin_name: self.metadata.name.clone(),
        };
        
        // Execute through sandbox with host function binding
        self.sandbox.call("call_host_function", &request).await
    }
    
    pub async fn register_host_functions(&mut self, functions: &[HostFunction]) -> Result<(), PluginError> {
        for function in functions {
            self.sandbox.bind_host_function(&function.name, function.handler.clone()).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct HostFunction {
    pub name: String,
    pub handler: Arc<dyn Fn(serde_json::Value) -> Result<serde_json::Value, HostFunctionError> + Send + Sync>,
    pub permissions: Vec<String>,
}

// Example host functions
pub fn create_standard_host_functions() -> Vec<HostFunction> {
    vec![
        HostFunction {
            name: "log".to_string(),
            handler: Arc::new(|args| {
                if let Some(message) = args.get("message").and_then(|v| v.as_str()) {
                    info!("Plugin log: {}", message);
                    Ok(serde_json::Value::Null)
                } else {
                    Err(HostFunctionError::InvalidArgs("message required".to_string()))
                }
            }),
            permissions: vec!["logging".to_string()],
        },
        HostFunction {
            name: "get_time".to_string(),
            handler: Arc::new(|_| {
                let now = Utc::now().timestamp();
                Ok(serde_json::json!({"timestamp": now}))
            }),
            permissions: vec![],
        },
        HostFunction {
            name: "http_request".to_string(),
            handler: Arc::new(|args| {
                // Implement HTTP request with validation
                todo!("Implement secure HTTP request handler")
            }),
            permissions: vec!["network".to_string()],
        },
    ]
}
```

## Plugin Development Workflow

### Plugin Template

```rust
// plugin_template.rs - Template for new plugins
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PluginRequest {
    pub action: String,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct PluginResponse {
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

// Plugin metadata (embedded in binary)
const PLUGIN_METADATA: &str = r#"
[plugin]
name = "example_plugin"
version = "1.0.0"
description = "Example plugin template"
author = "Plugin Developer"
license = "MIT"

[capabilities]
required = ["compute"]
optional = ["network", "storage"]

[permissions]
read_files = ["data/*.txt"]
write_files = ["output/*.json"]
network_hosts = ["api.example.com"]

[dependencies]
serde = "1.0"
serde_json = "1.0"
"#;

// Plugin entry points
#[no_mangle]
pub extern "C" fn initialize(config_ptr: *const u8, config_len: usize) -> i32 {
    let config_slice = unsafe { std::slice::from_raw_parts(config_ptr, config_len) };
    let config: PluginConfig = match serde_json::from_slice(config_slice) {
        Ok(c) => c,
        Err(_) => return -1,
    };
    
    // Initialize plugin state
    match initialize_plugin(config) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn handle_request(request_ptr: *const u8, request_len: usize) -> *mut u8 {
    let request_slice = unsafe { std::slice::from_raw_parts(request_ptr, request_len) };
    let request: PluginRequest = match serde_json::from_slice(request_slice) {
        Ok(r) => r,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let response = process_request(request);
    let response_json = serde_json::to_vec(&response).unwrap();
    
    // Allocate memory for response (caller must free)
    let ptr = allocate(response_json.len());
    unsafe {
        std::ptr::copy_nonoverlapping(response_json.as_ptr(), ptr, response_json.len());
    }
    ptr
}

#[no_mangle]
pub extern "C" fn shutdown() -> i32 {
    // Cleanup plugin resources
    cleanup_plugin();
    0
}

#[no_mangle]
pub extern "C" fn health_check() -> i32 {
    if is_plugin_healthy() { 1 } else { 0 }
}

// Plugin implementation
fn initialize_plugin(config: PluginConfig) -> Result<(), PluginError> {
    // Initialize plugin state, load configuration, etc.
    Ok(())
}

fn process_request(request: PluginRequest) -> PluginResponse {
    match request.action.as_str() {
        "process_data" => {
            // Implement data processing logic
            PluginResponse {
                success: true,
                result: serde_json::json!({"processed": request.data}),
                error: None,
            }
        }
        "get_status" => {
            // Return plugin status
            PluginResponse {
                success: true,
                result: serde_json::json!({"status": "running"}),
                error: None,
            }
        }
        _ => {
            PluginResponse {
                success: false,
                result: serde_json::Value::Null,
                error: Some(format!("Unknown action: {}", request.action)),
            }
        }
    }
}

fn cleanup_plugin() {
    // Cleanup resources
}

fn is_plugin_healthy() -> bool {
    // Health check logic
    true
}

// Memory management functions
#[no_mangle]
pub extern "C" fn allocate(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut u8, len: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, len);
    }
}
```

### Plugin Manifest (TOML)

```toml
# plugin.toml - Plugin configuration
[plugin]
name = "text_processor"
version = "1.2.0"
description = "Advanced text processing plugin with NLP capabilities"
author = "Example Corp"
license = "MIT"
repository = "https://github.com/example/text-processor-plugin"
documentation = "https://docs.example.com/plugins/text-processor"
min_runtime_version = "0.3.0"

[capabilities]
required = ["compute"]
optional = ["network", "storage"]

[permissions]
read_files = [
    "data/*.txt",
    "models/*.bin"
]
write_files = [
    "output/*.json",
    "cache/*.tmp"
]
network_hosts = [
    "api.nlp-service.com",
    "cdn.models.com"
]
environment_vars = [
    "NLP_API_KEY",
    "MODEL_CACHE_DIR"
]

[resources]
memory_mb = 128
timeout_seconds = 60
max_fuel = 50000000
max_open_files = 20

[dependencies]
serde = "1.0"
serde_json = "1.0"
regex = "1.0"

[build]
optimize = true
target = "wasm32-wasi"
features = ["simd"]

[metadata]
category = "text-processing"
tags = ["nlp", "text", "processing"]
```

### Build Script

```bash
#!/bin/bash
# build-plugin.sh - Plugin build script

set -e

PLUGIN_NAME=$1
if [ -z "$PLUGIN_NAME" ]; then
    echo "Usage: $0 <plugin-name>"
    exit 1
fi

echo "Building plugin: $PLUGIN_NAME"

# Set build environment
export RUSTFLAGS="-C opt-level=3 -C target-cpu=generic"

# Build for WebAssembly target
cargo build --release --target wasm32-wasi

# Optimize WebAssembly binary
wasm-opt \
    target/wasm32-wasi/release/${PLUGIN_NAME}.wasm \
    -o plugins/${PLUGIN_NAME}.wasm \
    -O3 \
    --enable-simd

# Copy plugin manifest
cp plugin.toml plugins/${PLUGIN_NAME}.toml

# Generate plugin metadata
cargo run --bin generate-metadata \
    --manifest-path plugin.toml \
    --output plugins/${PLUGIN_NAME}.json

echo "Plugin built successfully: plugins/${PLUGIN_NAME}.wasm"
echo "Plugin size: $(wc -c < plugins/${PLUGIN_NAME}.wasm) bytes"
```

## Advanced Plugin Features

### Plugin Dependency Management

```rust
use semver::{Version, VersionReq};

pub struct PluginDependencyResolver {
    available_plugins: HashMap<String, Vec<PluginVersion>>,
    loaded_plugins: HashMap<String, Version>,
}

#[derive(Debug, Clone)]
pub struct PluginVersion {
    pub version: Version,
    pub metadata: PluginMetadata,
    pub file_path: PathBuf,
}

impl PluginDependencyResolver {
    pub fn resolve_dependencies(&self, plugin: &PluginMetadata) -> Result<Vec<String>, DependencyError> {
        let mut resolved = Vec::new();
        let mut to_resolve = vec![plugin.clone()];
        let mut visited = HashSet::new();
        
        while let Some(current) = to_resolve.pop() {
            if visited.contains(&current.name) {
                continue;
            }
            visited.insert(current.name.clone());
            
            for dep in &current.dependencies {
                if dep.optional && !self.is_dependency_available(&dep.name) {
                    continue; // Skip optional dependencies that aren't available
                }
                
                let available_versions = self.available_plugins
                    .get(&dep.name)
                    .ok_or_else(|| DependencyError::NotFound(dep.name.clone()))?;
                
                let version_req = VersionReq::parse(&dep.version_req)?;
                
                let compatible_version = available_versions
                    .iter()
                    .find(|v| version_req.matches(&v.version))
                    .ok_or_else(|| DependencyError::IncompatibleVersion {
                        plugin: dep.name.clone(),
                        required: dep.version_req.clone(),
                        available: available_versions.iter().map(|v| v.version.clone()).collect(),
                    })?;
                
                if !resolved.contains(&dep.name) {
                    resolved.push(dep.name.clone());
                    to_resolve.push(compatible_version.metadata.clone());
                }
            }
        }
        
        Ok(resolved)
    }
    
    pub async fn load_with_dependencies(&mut self, plugin_name: &str) -> Result<Vec<String>, PluginError> {
        let plugin_metadata = self.get_plugin_metadata(plugin_name)?;
        let dependencies = self.resolve_dependencies(&plugin_metadata)?;
        
        let mut loaded = Vec::new();
        
        // Load dependencies first
        for dep_name in dependencies {
            if !self.loaded_plugins.contains_key(&dep_name) {
                self.load_plugin(&dep_name).await?;
                loaded.push(dep_name);
            }
        }
        
        // Load the main plugin
        self.load_plugin(plugin_name).await?;
        loaded.push(plugin_name.to_string());
        
        Ok(loaded)
    }
}
```

### Plugin Registry and Discovery

```rust
use reqwest::Client;
use sha2::{Sha256, Digest};

pub struct PluginRegistry {
    registry_url: String,
    client: Client,
    cache_dir: PathBuf,
    signature_validator: SignatureValidator,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryPlugin {
    pub name: String,
    pub version: String,
    pub description: String,
    pub download_url: String,
    pub checksum: String,
    pub signature: String,
    pub published_at: DateTime<Utc>,
    pub metadata: PluginMetadata,
}

impl PluginRegistry {
    pub async fn search(&self, query: &str) -> Result<Vec<RegistryPlugin>, RegistryError> {
        let url = format!("{}/search?q={}", self.registry_url, urlencoding::encode(query));
        let response = self.client.get(&url).send().await?;
        let plugins: Vec<RegistryPlugin> = response.json().await?;
        Ok(plugins)
    }
    
    pub async fn download_plugin(&self, name: &str, version: &str) -> Result<PathBuf, RegistryError> {
        // Get plugin info from registry
        let plugin_info = self.get_plugin_info(name, version).await?;
        
        // Check if already cached
        let cache_path = self.cache_dir.join(format!("{}_{}.wasm", name, version));
        if cache_path.exists() {
            if self.verify_checksum(&cache_path, &plugin_info.checksum).await? {
                return Ok(cache_path);
            }
        }
        
        // Download plugin
        let response = self.client.get(&plugin_info.download_url).send().await?;
        let bytes = response.bytes().await?;
        
        // Verify checksum
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let computed_checksum = format!("{:x}", hasher.finalize());
        
        if computed_checksum != plugin_info.checksum {
            return Err(RegistryError::ChecksumMismatch {
                expected: plugin_info.checksum,
                computed: computed_checksum,
            });
        }
        
        // Verify signature
        self.signature_validator.verify(&bytes, &plugin_info.signature)?;
        
        // Save to cache
        tokio::fs::write(&cache_path, &bytes).await?;
        
        Ok(cache_path)
    }
    
    pub async fn install_plugin(&self, name: &str, version: &str) -> Result<String, RegistryError> {
        // Download plugin
        let plugin_path = self.download_plugin(name, version).await?;
        
        // Install to plugins directory
        let install_path = PathBuf::from("plugins").join(format!("{}.wasm", name));
        tokio::fs::copy(&plugin_path, &install_path).await?;
        
        // Download and install manifest
        let manifest_url = format!("{}/plugins/{}/{}/manifest", self.registry_url, name, version);
        let manifest_response = self.client.get(&manifest_url).send().await?;
        let manifest_content = manifest_response.text().await?;
        
        let manifest_path = install_path.with_extension("toml");
        tokio::fs::write(&manifest_path, manifest_content).await?;
        
        info!("Installed plugin {} version {}", name, version);
        Ok(install_path.to_string_lossy().to_string())
    }
}
```

## Plugin Testing

### Plugin Test Framework

```rust
use wasm_sandbox::testing::PluginTestFramework;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_basic_functionality() {
        let test_framework = PluginTestFramework::new()
            .plugin_path("target/wasm32-wasi/release/test_plugin.wasm")
            .manifest_path("test_plugin.toml")
            .build()
            .await
            .expect("Failed to create test framework");
        
        // Test plugin initialization
        let init_result = test_framework.initialize_plugin(serde_json::json!({
            "test_mode": true,
            "config": {"key": "value"}
        })).await;
        
        assert!(init_result.is_ok());
        
        // Test plugin function
        let request = PluginRequest {
            action: "process_data".to_string(),
            data: serde_json::json!({"input": "test data"}),
        };
        
        let response: PluginResponse = test_framework
            .call_plugin_function("handle_request", &request)
            .await
            .expect("Plugin call failed");
        
        assert!(response.success);
        assert_eq!(response.result["processed"], "test data");
    }
    
    #[tokio::test]
    async fn test_plugin_security_restrictions() {
        let test_framework = PluginTestFramework::new()
            .plugin_path("target/wasm32-wasi/release/restricted_plugin.wasm")
            .security_policy(SecurityPolicy::strict())
            .deny_capability(Capability::Network)
            .build()
            .await
            .expect("Failed to create test framework");
        
        // Test that network access is denied
        let request = PluginRequest {
            action: "make_http_request".to_string(),
            data: serde_json::json!({"url": "https://example.com"}),
        };
        
        let result = test_framework
            .call_plugin_function("handle_request", &request)
            .await;
        
        // Should fail due to security restriction
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::SecurityViolation(_)));
    }
    
    #[tokio::test]
    async fn test_plugin_resource_limits() {
        let test_framework = PluginTestFramework::new()
            .plugin_path("target/wasm32-wasi/release/memory_hog.wasm")
            .memory_limit(16 * 1024 * 1024) // 16MB limit
            .timeout_duration(Duration::from_secs(5))
            .build()
            .await
            .expect("Failed to create test framework");
        
        // Test memory limit enforcement
        let request = PluginRequest {
            action: "allocate_large_memory".to_string(),
            data: serde_json::json!({"size_mb": 32}), // Try to allocate 32MB
        };
        
        let result = test_framework
            .call_plugin_function("handle_request", &request)
            .await;
        
        // Should fail due to memory limit
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::ResourceLimit(_)));
    }
}
```

## Debugging and Monitoring

### Plugin Debug Interface

```rust
pub struct PluginDebugger {
    plugin_manager: Arc<PluginManager>,
    debug_server: Option<DebugServer>,
}

impl PluginDebugger {
    pub async fn start_debug_server(&mut self, port: u16) -> Result<(), DebugError> {
        let server = DebugServer::new(port, Arc::clone(&self.plugin_manager)).await?;
        self.debug_server = Some(server);
        
        info!("Plugin debug server started on port {}", port);
        Ok(())
    }
    
    pub async fn get_plugin_state(&self, plugin_name: &str) -> Result<PluginDebugInfo, DebugError> {
        let plugins = self.plugin_manager.plugins.read().await;
        let instance = plugins
            .get(plugin_name)
            .ok_or_else(|| DebugError::PluginNotFound(plugin_name.to_string()))?;
        
        let debug_info = PluginDebugInfo {
            name: instance.metadata.name.clone(),
            version: instance.metadata.version.clone(),
            status: self.get_plugin_status(instance).await?,
            memory_usage: instance.sandbox.get_memory_usage().await?,
            execution_stats: instance.get_execution_stats(),
            last_error: instance.last_error.clone(),
            debug_logs: self.get_debug_logs(plugin_name).await?,
        };
        
        Ok(debug_info)
    }
    
    pub async fn set_breakpoint(&self, plugin_name: &str, function: &str, line: u32) -> Result<BreakpointId, DebugError> {
        // Set debugging breakpoint in plugin
        let plugins = self.plugin_manager.plugins.read().await;
        let instance = plugins
            .get(plugin_name)
            .ok_or_else(|| DebugError::PluginNotFound(plugin_name.to_string()))?;
        
        let breakpoint_id = instance.sandbox.set_breakpoint(function, line).await?;
        
        info!("Breakpoint set in plugin {} at {}:{}", plugin_name, function, line);
        Ok(breakpoint_id)
    }
    
    pub async fn step_execution(&self, plugin_name: &str) -> Result<ExecutionStep, DebugError> {
        let plugins = self.plugin_manager.plugins.read().await;
        let instance = plugins
            .get(plugin_name)
            .ok_or_else(|| DebugError::PluginNotFound(plugin_name.to_string()))?;
        
        let step = instance.sandbox.step_execution().await?;
        Ok(step)
    }
}

#[derive(Debug, Serialize)]
pub struct PluginDebugInfo {
    pub name: String,
    pub version: String,
    pub status: PluginStatus,
    pub memory_usage: MemoryUsage,
    pub execution_stats: ExecutionStats,
    pub last_error: Option<String>,
    pub debug_logs: Vec<DebugLogEntry>,
}
```

## Best Practices

### 1. Security Best Practices

```rust
// Always use minimal capabilities
let sandbox = WasmSandbox::builder()
    .deny_all_capabilities()
    .allow_capability(Capability::Compute) // Only what's needed
    .strict_mode(true)
    .build()
    .await?;

// Validate all plugin inputs
fn validate_plugin_input(input: &serde_json::Value) -> Result<(), ValidationError> {
    // Check for maximum size
    let input_str = serde_json::to_string(input)?;
    if input_str.len() > 1024 * 1024 { // 1MB limit
        return Err(ValidationError::InputTooLarge);
    }
    
    // Validate data types and ranges
    // ... validation logic
    
    Ok(())
}
```

### 2. Performance Best Practices

```rust
// Use plugin pooling for better performance
let plugin_pool = PluginPool::new()
    .max_instances(10)
    .preload_plugins(&["common_plugin"])
    .build()
    .await?;

// Cache compiled modules
let module_cache = ModuleCache::new(100); // Cache 100 modules
```

### 3. Error Handling Best Practices

```rust
// Comprehensive error handling
match plugin_manager.execute_plugin("text_processor", "process", &input).await {
    Ok(result) => {
        // Handle success
    }
    Err(PluginError::PluginNotFound(name)) => {
        error!("Plugin not found: {}", name);
        // Try to load plugin dynamically
    }
    Err(PluginError::SecurityViolation(msg)) => {
        error!("Security violation: {}", msg);
        // Alert security team
    }
    Err(PluginError::ResourceLimit(msg)) => {
        warn!("Resource limit exceeded: {}", msg);
        // Scale resources or queue request
    }
    Err(e) => {
        error!("Plugin execution failed: {}", e);
        // General error handling
    }
}
```

## Next Steps

- **[Security Configuration](security-config.md)** - Secure your plugin system
- **[Resource Management](resource-management.md)** - Optimize plugin performance
- **[Production Deployment](production.md)** - Deploy plugins in production
- **[API Reference](https://docs.rs/wasm-sandbox)** - Detailed API documentation

---

**Plugin Development:** This guide provides comprehensive coverage of building secure, performant plugin systems. Start with the basic examples and gradually incorporate advanced features as needed.
