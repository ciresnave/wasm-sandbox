# Generic Plugin System Design

## Overview

This document outlines the design for a generic plugin system built on top of wasm-sandbox. While this design is informed by the needs of the PUP (Plugin Universal Platform) project, it is intentionally generic and can be adapted by any application requiring secure execution of untrusted code.

## Use Cases

The generic plugin system is designed to support various applications:

- **Plugin Platforms** (like PUP): Execute user-submitted plugins safely
- **CI/CD Systems**: Run build scripts and deployment tools in isolation  
- **Serverless Platforms**: Execute function code with resource constraints
- **Code Playgrounds**: Run user code with security boundaries
- **Data Processing**: Execute transformation scripts on sensitive data
- **Microservice Isolation**: Run services with strict resource limits
- **Development Tools**: Execute linters, formatters, and analyzers safely

## Core Traits

### WasmPlugin Trait

```rust
/// Generic trait for WASM-based plugins that can be executed securely
pub trait WasmPlugin: Send + Sync {
    /// Returns the plugin's manifest describing its capabilities and requirements
    fn manifest(&self) -> &PluginManifest;
    
    /// Validates input parameters before execution
    fn validate_parameters(&self, params: &[Value]) -> Result<(), PluginError>;
    
    /// Executes the plugin with the given context and parameters
    fn execute(&self, context: &ExecutionContext, params: &[Value]) 
        -> BoxFuture<'_, Result<PluginResult, PluginError>>;
    
    /// Optional: Cleanup resources when plugin is unloaded
    fn cleanup(&self) -> BoxFuture<'_, Result<(), PluginError>> {
        Box::pin(async { Ok(()) })
    }
}
```

### Plugin Manifest

```rust
/// Manifest describing a plugin's metadata, permissions, and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique identifier for the plugin
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Semantic version
    pub version: String,
    
    /// Description of what the plugin does
    pub description: String,
    
    /// Plugin author/organization
    pub author: String,
    
    /// Required permissions and capabilities
    pub permissions: PluginPermissions,
    
    /// Entry points for different operations
    pub entry_points: Vec<EntryPoint>,
    
    /// Dependencies on other plugins or services
    pub dependencies: Vec<Dependency>,
    
    /// Resource requirements and limits
    pub resources: ResourceRequirements,
    
    /// Additional metadata (application-specific)
    pub metadata: serde_json::Value,
}
```

### Plugin Permissions

```rust
/// Granular permission system for plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    /// Filesystem access permissions
    pub filesystem: FilesystemPermissions,
    
    /// Network access permissions  
    pub network: NetworkPermissions,
    
    /// System-level permissions
    pub system: SystemPermissions,
    
    /// Inter-plugin communication permissions
    pub communication: CommunicationPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemPermissions {
    /// Paths that can be read
    pub read_paths: Vec<PathBuf>,
    
    /// Paths that can be written to
    pub write_paths: Vec<PathBuf>,
    
    /// Whether temporary directory access is allowed
    pub temp_access: bool,
    
    /// Maximum file size that can be read/written
    pub max_file_size: Option<usize>,
    
    /// Whether directory creation is allowed
    pub create_directories: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermissions {
    /// Allowed domains/hostnames
    pub allowed_domains: Vec<String>,
    
    /// Allowed ports for outbound connections
    pub allowed_ports: Vec<u16>,
    
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    
    /// Whether HTTPS is required
    pub require_https: bool,
    
    /// Whether listening on ports is allowed
    pub can_listen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPermissions {
    /// Environment variables that can be accessed
    pub env_vars: Vec<String>,
    
    /// Whether process spawning is allowed
    pub spawn_processes: bool,
    
    /// Maximum number of threads
    pub max_threads: usize,
    
    /// Whether system time access is allowed
    pub system_time: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPermissions {
    /// Other plugins this plugin can communicate with
    pub allowed_plugins: Vec<String>,
    
    /// Whether host communication is allowed
    pub host_communication: bool,
    
    /// Message size limits
    pub max_message_size: usize,
}
```

### Execution Context

```rust
/// Context provided to plugins during execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Unique execution ID for tracking
    pub execution_id: String,
    
    /// User/tenant identifier (if applicable)
    pub user_id: Option<String>,
    
    /// Workspace or working directory
    pub workspace: PathBuf,
    
    /// Environment variables available to the plugin
    pub environment: HashMap<String, String>,
    
    /// Resource limits for this execution
    pub resource_limits: ResourceLimits,
    
    /// Timeout for this specific execution
    pub timeout: Duration,
    
    /// Application-specific context data
    pub app_context: serde_json::Value,
}
```

### Plugin Result

```rust
/// Result of plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    /// Success status
    pub success: bool,
    
    /// Return value (if any)
    pub value: Option<serde_json::Value>,
    
    /// Output/log messages
    pub output: Vec<String>,
    
    /// Error messages (if any)
    pub errors: Vec<String>,
    
    /// Resource usage during execution
    pub resource_usage: ResourceUsage,
    
    /// Execution duration
    pub duration: Duration,
    
    /// Files created/modified
    pub file_changes: Vec<FileChange>,
}
```

## Plugin Manager

```rust
/// Generic plugin manager for loading, validating, and executing plugins
pub struct PluginManager {
    sandbox: WasmSandbox,
    plugins: RwLock<HashMap<String, LoadedPlugin>>,
    validators: Vec<Box<dyn PluginValidator>>,
    permissions_checker: Box<dyn PermissionsChecker>,
}

impl PluginManager {
    /// Create a new plugin manager with default configuration
    pub fn new() -> Result<Self, PluginError> {
        Self::with_config(PluginManagerConfig::default())
    }
    
    /// Create a plugin manager with custom configuration
    pub fn with_config(config: PluginManagerConfig) -> Result<Self, PluginError> {
        // Implementation
    }
    
    /// Load a plugin from WASM bytes
    pub async fn load_plugin(&self, wasm_bytes: &[u8], manifest: PluginManifest) 
        -> Result<String, PluginError> {
        // Validate manifest
        // Security scan
        // Load into sandbox
        // Store in registry
    }
    
    /// Validate a plugin before loading
    pub async fn validate_plugin(&self, wasm_bytes: &[u8], manifest: &PluginManifest) 
        -> Result<ValidationReport, PluginError> {
        // Run security validators
        // Check permissions
        // Verify WASM module structure
    }
    
    /// Execute a plugin
    pub async fn execute_plugin(&self, plugin_id: &str, context: ExecutionContext, 
        params: &[Value]) -> Result<PluginResult, PluginError> {
        // Get plugin
        // Check permissions against context
        // Execute in sandbox
        // Monitor resources
        // Return results
    }
    
    /// Hot reload a plugin
    pub async fn reload_plugin(&self, plugin_id: &str, wasm_bytes: &[u8], 
        new_manifest: PluginManifest) -> Result<(), PluginError> {
        // Validate compatibility
        // Graceful shutdown of old version
        // Load new version
        // Update registry
    }
    
    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        // Cleanup resources
        // Remove from registry
    }
    
    /// List loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        // Return plugin metadata
    }
    
    /// Get plugin status and resource usage
    pub fn get_plugin_status(&self, plugin_id: &str) -> Result<PluginStatus, PluginError> {
        // Return runtime status
    }
}
```

## Validation Framework

```rust
/// Trait for plugin validators (security, performance, compliance, etc.)
pub trait PluginValidator: Send + Sync {
    fn name(&self) -> &str;
    
    async fn validate(&self, wasm_bytes: &[u8], manifest: &PluginManifest) 
        -> Result<ValidationResult, ValidationError>;
}

/// Built-in validators
pub struct SecurityValidator {
    // Checks for malicious patterns, forbidden imports, etc.
}

pub struct PerformanceValidator {
    // Checks for infinite loops, excessive memory usage, etc.
}

pub struct PermissionsValidator {
    // Validates that requested permissions are reasonable
}

pub struct WasmStructureValidator {
    // Validates WASM module structure and imports
}
```

## Application-Specific Implementations

### PUP-Style Plugin System

```rust
/// PUP-specific plugin implementation
pub struct PupPlugin {
    manifest: PluginManifest,
    tool_type: ToolType,
}

impl WasmPlugin for PupPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
    
    fn validate_parameters(&self, params: &[Value]) -> Result<(), PluginError> {
        // PUP-specific parameter validation
    }
    
    fn execute(&self, context: &ExecutionContext, params: &[Value]) 
        -> BoxFuture<'_, Result<PluginResult, PluginError>> {
        Box::pin(async move {
            // PUP-specific execution logic
            // Handle different tool types
            // Process results in PUP format
        })
    }
}

/// PUP-specific extensions
impl PupPlugin {
    pub fn as_file_processor(&self) -> Option<&dyn FileProcessor> {
        // Return file processor interface if applicable
    }
    
    pub fn as_web_scraper(&self) -> Option<&dyn WebScraper> {
        // Return web scraper interface if applicable
    }
}
```

### CI/CD Build Step Plugin

```rust
/// CI/CD-specific plugin implementation
pub struct BuildStepPlugin {
    manifest: PluginManifest,
    step_type: BuildStepType,
}

impl WasmPlugin for BuildStepPlugin {
    fn execute(&self, context: &ExecutionContext, params: &[Value]) 
        -> BoxFuture<'_, Result<PluginResult, PluginError>> {
        Box::pin(async move {
            // CI/CD-specific execution
            // Handle build artifacts
            // Process exit codes
            // Generate build reports
        })
    }
}
```

### Serverless Function Plugin

```rust
/// Serverless function implementation
pub struct ServerlessFunction {
    manifest: PluginManifest,
    handler: String,
}

impl WasmPlugin for ServerlessFunction {
    fn execute(&self, context: &ExecutionContext, params: &[Value]) 
        -> BoxFuture<'_, Result<PluginResult, PluginError>> {
        Box::pin(async move {
            // Serverless-specific execution
            // Handle HTTP request/response
            // Manage cold starts
            // Monitor invocation metrics
        })
    }
}
```

## Configuration

```rust
/// Configuration for the plugin manager
#[derive(Debug, Clone)]
pub struct PluginManagerConfig {
    /// Maximum number of concurrent plugin executions
    pub max_concurrent_executions: usize,
    
    /// Default resource limits for plugins
    pub default_resource_limits: ResourceLimits,
    
    /// Security policies
    pub security_policies: SecurityPolicies,
    
    /// Plugin discovery settings
    pub discovery: DiscoveryConfig,
    
    /// Hot reload settings
    pub hot_reload: HotReloadConfig,
    
    /// Observability settings
    pub observability: ObservabilityConfig,
}
```

## Benefits of Generic Design

### 1. **Wide Applicability**
- Can be used by any application needing secure code execution
- Not tied to specific use cases or domain terminology
- Flexible permission model adapts to different security requirements

### 2. **Extensibility**
- Applications can extend base traits for specific needs
- Validator framework allows custom security/compliance checks
- Plugin manifest metadata supports application-specific data

### 3. **Reusability**
- Common patterns (loading, validation, execution) work across applications
- Plugin development tools can target the generic interface
- Knowledge transfers between different plugin ecosystems

### 4. **Future-Proofing**
- Can evolve to support new use cases without breaking existing applications
- WebAssembly Component Model integration path is clear
- Marketplace and distribution patterns are application-agnostic

## Migration Path for PUP

PUP can adopt this generic system while maintaining its specific requirements:

1. **Implement `WasmPlugin` for existing tool types**
2. **Map PUP tool manifests to generic `PluginManifest`**
3. **Use application-specific validators for PUP requirements**
4. **Extend base functionality with PUP-specific traits**

This approach gives PUP all the benefits of the generic system while preserving its unique features and workflow patterns.
