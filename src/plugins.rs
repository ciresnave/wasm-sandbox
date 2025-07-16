//! Generic plugin system traits and types for wasm-sandbox

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use serde_json::Value;

use crate::error::{Result, InstanceId};
use crate::config::AdvancedCapabilities;
use crate::monitoring::DetailedResourceUsage;

/// Plugin manifest describing a WebAssembly plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin identifier
    pub id: String,
    
    /// Human-readable plugin name
    pub name: String,
    
    /// Semantic version
    pub version: String,
    
    /// Plugin description
    pub description: String,
    
    /// Required permissions and capabilities
    pub permissions: AdvancedCapabilities,
    
    /// Plugin entry points (exported functions)
    pub entry_points: Vec<EntryPoint>,
    
    /// Dependencies on other plugins or system components
    pub dependencies: Vec<Dependency>,
    
    /// Plugin metadata
    pub metadata: HashMap<String, Value>,
    
    /// Minimum wasm-sandbox version required
    pub min_sandbox_version: String,
    
    /// Plugin author information
    pub author: Option<String>,
    
    /// Plugin license
    pub license: Option<String>,
    
    /// Repository URL
    pub repository: Option<String>,
}

/// Plugin entry point definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    /// Function name in the WASM module
    pub function_name: String,
    
    /// Human-readable name for this entry point
    pub display_name: String,
    
    /// Entry point description
    pub description: String,
    
    /// Input parameter schema (JSON Schema)
    pub input_schema: Option<Value>,
    
    /// Output schema (JSON Schema)
    pub output_schema: Option<Value>,
    
    /// Whether this entry point supports streaming
    pub supports_streaming: bool,
    
    /// Expected execution time category
    pub execution_category: ExecutionCategory,
}

/// Execution time categories for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionCategory {
    /// Very fast operations (< 10ms)
    Instant,
    /// Fast operations (< 100ms) 
    Fast,
    /// Standard operations (< 1s)
    Standard,
    /// Long-running operations (> 1s)
    LongRunning,
    /// Streaming operations
    Streaming,
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency name/ID
    pub name: String,
    
    /// Version requirement (semver)
    pub version_requirement: String,
    
    /// Whether this dependency is optional
    pub optional: bool,
    
    /// Dependency type
    pub dependency_type: DependencyType,
}

/// Types of dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Another plugin
    Plugin,
    /// System library or capability
    System,
    /// External service
    Service,
}

/// Plugin execution context
#[derive(Debug)]
pub struct ExecutionContext {
    /// Instance ID for this execution
    pub instance_id: InstanceId,
    
    /// Plugin manifest
    pub manifest: PluginManifest,
    
    /// Execution environment variables
    pub environment: HashMap<String, String>,
    
    /// Temporary directory for this execution
    pub temp_dir: PathBuf,
    
    /// Resource limits for this execution
    pub resource_limits: crate::security::ResourceLimits,
    
    /// Execution metadata
    pub metadata: HashMap<String, Value>,
}

/// Main plugin trait for WebAssembly plugins
#[async_trait]
pub trait WasmPlugin: Send + Sync {
    /// Get the plugin manifest
    fn manifest(&self) -> &PluginManifest;
    
    /// Initialize the plugin
    async fn initialize(&mut self, context: &ExecutionContext) -> Result<()>;
    
    /// Validate input parameters for a specific entry point
    fn validate_parameters(&self, entry_point: &str, params: &[Value]) -> Result<()>;
    
    /// Execute a plugin function
    async fn execute(
        &self, 
        entry_point: &str,
        parameters: &[Value],
        context: &ExecutionContext
    ) -> Result<Value>;
    
    /// Execute with streaming input
    async fn execute_streaming(
        &self,
        entry_point: &str,
        input_stream: Box<dyn futures::Stream<Item = Value> + Send + Unpin>,
        context: &ExecutionContext
    ) -> Result<Box<dyn futures::Stream<Item = Result<Value>> + Send + Unpin>>;
    
    /// Clean up resources
    async fn cleanup(&mut self, context: &ExecutionContext) -> Result<()>;
    
    /// Get plugin health status
    async fn health_check(&self) -> Result<PluginHealth>;
}

/// Plugin health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHealth {
    /// Overall health status
    pub status: HealthStatus,
    
    /// Health check timestamp
    pub timestamp: u64,
    
    /// Resource usage
    pub resource_usage: Option<DetailedResourceUsage>,
    
    /// Health messages
    pub messages: Vec<String>,
    
    /// Performance metrics
    pub metrics: HashMap<String, f64>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Plugin is healthy and ready
    Healthy,
    /// Plugin has warnings but is functional
    Warning,
    /// Plugin has errors and may not function correctly
    Error,
    /// Plugin is not responding
    Unresponsive,
}

/// Hot reload capabilities for plugins
#[async_trait]
pub trait HotReload: Send + Sync {
    /// Check if a new version is compatible for hot reload
    async fn check_compatibility(
        &self, 
        current_manifest: &PluginManifest,
        new_wasm_bytes: &[u8]
    ) -> Result<CompatibilityReport>;
    
    /// Perform hot reload of the plugin
    async fn hot_reload(
        &mut self,
        new_wasm_bytes: &[u8],
        context: &ExecutionContext
    ) -> Result<()>;
    
    /// Rollback to previous version
    async fn rollback(&mut self, context: &ExecutionContext) -> Result<()>;
}

/// Compatibility report for hot reload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    /// Whether hot reload is safe
    pub is_compatible: bool,
    
    /// Breaking changes detected
    pub breaking_changes: Vec<BreakingChange>,
    
    /// Warnings about the update
    pub warnings: Vec<String>,
    
    /// Recommended actions
    pub recommendations: Vec<String>,
}

/// Breaking change description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    /// Type of breaking change
    pub change_type: BreakingChangeType,
    
    /// Description of the change
    pub description: String,
    
    /// Affected entry points or features
    pub affected_items: Vec<String>,
    
    /// Suggested migration steps
    pub migration_steps: Vec<String>,
}

/// Types of breaking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakingChangeType {
    /// Function signature changed
    FunctionSignature,
    /// Entry point removed
    EntryPointRemoved,
    /// Permission requirements changed
    PermissionChange,
    /// Dependency requirements changed
    DependencyChange,
    /// API version incompatible
    ApiVersion,
}

/// Plugin validation trait
pub trait PluginValidator: Send + Sync {
    /// Validate plugin manifest
    fn validate_manifest(&self, manifest: &PluginManifest) -> Result<Vec<ValidationWarning>>;
    
    /// Validate WASM bytecode
    fn validate_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<ValidationWarning>>;
    
    /// Security audit of plugin
    fn security_audit(&self, manifest: &PluginManifest, wasm_bytes: &[u8]) -> Result<SecurityAuditReport>;
    
    /// Performance benchmark
    fn benchmark(&self, plugin: &dyn WasmPlugin) -> Result<BenchmarkReport>;
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning severity
    pub severity: WarningSeverity,
    
    /// Warning message
    pub message: String,
    
    /// Source location if available
    pub location: Option<String>,
    
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Warning severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Security audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditReport {
    /// Overall security score (0-100)
    pub security_score: u8,
    
    /// Security issues found
    pub issues: Vec<SecurityIssue>,
    
    /// Recommended security mitigations
    pub mitigations: Vec<String>,
    
    /// Whether the plugin is safe for production
    pub production_ready: bool,
}

/// Security issue description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// Issue severity
    pub severity: SecuritySeverity,
    
    /// Issue description
    pub description: String,
    
    /// Potential impact
    pub impact: String,
    
    /// Recommended fix
    pub fix: String,
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance benchmark report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// Entry point benchmarks
    pub entry_points: HashMap<String, EntryPointBenchmark>,
    
    /// Overall performance score
    pub performance_score: u8,
    
    /// Memory efficiency score
    pub memory_score: u8,
    
    /// CPU efficiency score  
    pub cpu_score: u8,
    
    /// Recommendations for optimization
    pub optimizations: Vec<String>,
}

/// Benchmark data for a specific entry point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPointBenchmark {
    /// Average execution time
    pub avg_execution_time: std::time::Duration,
    
    /// 95th percentile execution time
    pub p95_execution_time: std::time::Duration,
    
    /// Memory usage statistics
    pub memory_usage: crate::monitoring::MemoryUsage,
    
    /// Throughput (operations per second)
    pub throughput: f64,
}

/// Plugin registry for managing plugins
pub trait PluginRegistry: Send + Sync {
    /// Register a new plugin
    fn register_plugin(&mut self, manifest: PluginManifest, wasm_bytes: Vec<u8>) -> Result<()>;
    
    /// Unregister a plugin
    fn unregister_plugin(&mut self, plugin_id: &str) -> Result<()>;
    
    /// Get plugin manifest
    fn get_manifest(&self, plugin_id: &str) -> Result<&PluginManifest>;
    
    /// List all registered plugins
    fn list_plugins(&self) -> Vec<&PluginManifest>;
    
    /// Search plugins by criteria
    fn search_plugins(&self, query: &PluginQuery) -> Vec<&PluginManifest>;
    
    /// Get plugin dependencies
    fn get_dependencies(&self, plugin_id: &str) -> Result<Vec<&PluginManifest>>;
}

/// Plugin search query
#[derive(Debug, Clone)]
pub struct PluginQuery {
    /// Search by name or description
    pub text: Option<String>,
    
    /// Filter by author
    pub author: Option<String>,
    
    /// Filter by category/tags
    pub tags: Vec<String>,
    
    /// Minimum version requirement
    pub min_version: Option<String>,
    
    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// Default plugin query implementation
impl Default for PluginQuery {
    fn default() -> Self {
        Self {
            text: None,
            author: None,
            tags: Vec::new(),
            min_version: None,
            required_capabilities: Vec::new(),
        }
    }
}
