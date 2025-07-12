# API Improvements for wasm-sandbox v0.3.0

This document outlines planned API improvements based on real-world integration feedback, particularly focusing on ease of use and progressive complexity.

## 🚨 Critical Issues Identified

### 1. **Ease of Use - HIGHEST PRIORITY**

**Current Problem**: The API requires too much ceremony for simple tasks.

**Current "simple" usage:**
```rust
let mut sandbox = WasmSandbox::new()?;
let wasm_bytes = std::fs::read("module.wasm")?;
let module_id = sandbox.load_module(&wasm_bytes)?;
let instance_id = sandbox.create_instance(module_id, None)?;
let result: i32 = sandbox.call_function(instance_id, "add", &(5, 3)).await?;
```

**Planned "actually simple" usage:**
```rust
// One-liner for basic cases
let result: i32 = wasm_sandbox::run("./calculator.rs", "add", &(5, 3))?;

// Auto-compile and reuse
let sandbox = WasmSandbox::from_source("./my_project/")?;
let result = sandbox.call("add", &(5, 3))?;
```

**Key Improvements:**
- Auto-compilation from source code (Rust, Python, C, JS, Go)
- One-line execution for simple cases
- Sane security defaults (secure by default)
- Human-readable configuration ("64MB" instead of `67108864`)
- Progressive complexity (simple → advanced as needed)

### 2. Documentation Gaps (ADDRESSED)

**Solutions Implemented:**
- ✅ [`MIGRATION.md`](../guides/MIGRATION.md) - Complete v0.1.0 → v0.2.0 upgrade guide
- ✅ [`examples/README.md`](../../examples/README.md) - Comprehensive examples overview
- ✅ [`examples/file_processor.rs`](examples/file_processor.rs) - Real-world file processing example
- ✅ [`examples/plugin_ecosystem.rs`](examples/plugin_ecosystem.rs) - PUP-style plugin system

**Planned for v0.3.0:**

- 📋 API cookbook with common patterns
- 📋 Security best practices guide
- 📋 Performance optimization guide
- 📋 Troubleshooting guide with common errors

### 2. API Ergonomics Issues

**Current Problem**: Complex lifetime requirements and verbose configuration.

**Planned Improvements:**

#### Builder Pattern for Configuration

```rust
// Current (verbose)
let config = InstanceConfig {
    capabilities: Capabilities { /* ... */ },
    resource_limits: ResourceLimits { /* ... */ },
    startup_timeout_ms: 5000,
    enable_debug: false,
};

// Planned (ergonomic)
let config = InstanceConfig::builder()
    .memory_limit(64.mb())
    .timeout(30.seconds())
    .filesystem_read(&["/data", "/config"])
    .filesystem_write(&["/output"])
    .network_deny_all()
    .build()?;
```

#### Simplified Function Calling

```rust
// Current (complex lifetimes)
async fn execute_function<'a>(&self, instance_id: &InstanceId, 
    function_name: &str, parameters: &'a [Value]) -> Result<Value>

// Planned (ergonomic)
async fn execute_function(&self, instance_id: InstanceId, 
    function_name: &str, parameters: Vec<Value>) -> Result<Value>
```

### 3. Enhanced Error Types

**Current Problem**: Generic error types make debugging difficult.

**Planned Improvements:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Security violation: {violation}")]
    SecurityViolation { 
        violation: String, 
        instance_id: InstanceId,
        context: SecurityContext,
    },
    
    #[error("Resource exhausted: {kind} used {used}/{limit}")]
    ResourceExhausted { 
        kind: ResourceKind, 
        limit: u64, 
        used: u64,
        instance_id: InstanceId,
    },
    
    #[error("WASM runtime error in function '{function}': {source}")]
    WasmRuntime {
        function: String,
        instance_id: InstanceId,
        #[source]
        source: wasmtime::Error,
    },
    
    #[error("Configuration error: {message}")]
    Configuration { 
        message: String,
        suggestion: Option<String>,
    },
}

#[derive(Debug)]
pub enum ResourceKind {
    Memory,
    CpuTime,
    Fuel,
    FileHandles,
    NetworkConnections,
}

#[derive(Debug)]
pub struct SecurityContext {
    pub attempted_operation: String,
    pub required_capability: String,
    pub available_capabilities: Vec<String>,
}
```

## 🎯 Generic Plugin System Requirements

### 1. Plugin Development Ecosystem

```rust
pub trait WasmPlugin {
    fn manifest() -> PluginManifest;
    fn validate_parameters(&self, params: &[Value]) -> Result<()>;
    fn execute(&self, context: &ExecutionContext) -> BoxFuture<'_, Result<serde_json::Value>>;
}

pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub permissions: AdvancedCapabilities,
    pub entry_points: Vec<EntryPoint>,
    pub dependencies: Vec<Dependency>,
}

pub struct AdvancedCapabilities {
    pub network: NetworkPolicy {
        pub allowed_domains: Vec<String>,
        pub max_connections: usize,
        pub allowed_ports: Vec<u16>,
    },
    pub filesystem: FilesystemPolicy {
        pub read_paths: Vec<PathBuf>,
        pub write_paths: Vec<PathBuf>,
        pub temp_dir_access: bool,
        pub max_file_size: usize,
    },
    pub system: SystemPolicy {
        pub env_var_access: Vec<String>,
        pub process_spawn: bool,
        pub max_threads: usize,
    },
}
```

### 2. Streaming and Async Support

```rust
pub trait StreamingExecution {
    async fn execute_stream<S>(&self, input: S) -> impl Stream<Item = Result<Value>>
    where S: Stream<Item = Value> + Send;
    
    async fn execute_batch<I>(&self, calls: I) -> Vec<Result<Value>>
    where I: Iterator<Item = FunctionCall> + Send;
}

pub struct FunctionCall {
    pub function_name: String,
    pub parameters: Vec<Value>,
    pub timeout: Option<Duration>,
}
```

### 3. Hot Reload and Development Tools

```rust
pub trait HotReload {
    async fn hot_reload_module(&mut self, instance_id: InstanceId, wasm_bytes: &[u8]) -> Result<()>;
    async fn validate_reload_compatibility(&self, old_module: &dyn WasmModule, new_bytes: &[u8]) -> Result<CompatibilityReport>;
}

pub struct CompatibilityReport {
    pub is_compatible: bool,
    pub breaking_changes: Vec<BreakingChange>,
    pub warnings: Vec<String>,
}

pub trait DebugSupport {
    fn enable_debugging(&mut self, instance_id: InstanceId) -> Result<DebugSession>;
    fn get_stack_trace(&self, instance_id: InstanceId) -> Result<StackTrace>;
    fn set_breakpoint(&mut self, instance_id: InstanceId, function: &str, line: u32) -> Result<BreakpointId>;
}
```

## 📊 Observability and Metrics

### Resource Usage Monitoring

```rust
#[derive(Debug, serde::Serialize)]
pub struct DetailedResourceUsage {
    pub memory: MemoryUsage {
        pub current_bytes: usize,
        pub peak_bytes: usize,
        pub allocations: u64,
        pub deallocations: u64,
    },
    pub cpu: CpuUsage {
        pub time_spent: Duration,
        pub instructions_executed: u64,
        pub function_calls: u64,
    },
    pub io: IoUsage {
        pub files_opened: u64,
        pub bytes_read: u64,
        pub bytes_written: u64,
        pub network_requests: u64,
    },
    pub timeline: Vec<ResourceSnapshot>,
}

#[derive(Debug, serde::Serialize)]
pub struct ResourceSnapshot {
    pub timestamp: std::time::Instant,
    pub memory_bytes: usize,
    pub cpu_time: Duration,
    pub active_handles: u32,
}
```

### Performance Profiling

```rust
pub trait PerformanceProfiler {
    fn start_profiling(&mut self, instance_id: InstanceId) -> Result<()>;
    fn stop_profiling(&mut self, instance_id: InstanceId) -> Result<ProfileReport>;
    fn get_hot_functions(&self, instance_id: InstanceId) -> Result<Vec<HotFunction>>;
}

pub struct ProfileReport {
    pub total_time: Duration,
    pub function_timings: BTreeMap<String, FunctionTiming>,
    pub memory_timeline: Vec<MemoryEvent>,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub optimization_suggestions: Vec<OptimizationHint>,
}

pub struct FunctionTiming {
    pub call_count: u64,
    pub total_time: Duration,
    pub average_time: Duration,
    pub max_time: Duration,
    pub min_time: Duration,
}
```

## 🔒 Advanced Security Features

### Security Auditing

```rust
pub trait SecurityAuditor {
    fn start_audit(&mut self, instance_id: InstanceId) -> Result<AuditSession>;
    fn get_security_events(&self, instance_id: InstanceId) -> Result<Vec<SecurityEvent>>;
    fn analyze_threats(&self, instance_id: InstanceId) -> Result<ThreatAnalysis>;
}

pub struct SecurityEvent {
    pub timestamp: std::time::SystemTime,
    pub event_type: SecurityEventType,
    pub description: String,
    pub severity: SecuritySeverity,
    pub instance_id: InstanceId,
    pub context: SecurityContext,
}

pub enum SecurityEventType {
    UnauthorizedFileAccess,
    UnauthorizedNetworkAccess,
    ResourceLimitExceeded,
    SuspiciousBehavior,
    PolicyViolation,
}

pub struct ThreatAnalysis {
    pub risk_score: f64,
    pub detected_threats: Vec<Threat>,
    pub recommendations: Vec<SecurityRecommendation>,
}
```

### Multi-Tenant Isolation

```rust
pub struct TenantSandbox {
    pub fn create_tenant(&mut self, tenant_id: &str, limits: TenantLimits) -> Result<TenantId>;
    pub fn execute_for_tenant(&mut self, tenant_id: TenantId, request: ExecutionRequest) -> Result<ExecutionResult>;
    pub fn get_tenant_usage(&self, tenant_id: TenantId) -> Result<TenantUsage>;
    pub fn isolate_tenant(&mut self, tenant_id: TenantId) -> Result<()>;
}

pub struct TenantLimits {
    pub max_instances: u32,
    pub total_memory_limit: usize,
    pub total_cpu_time: Duration,
    pub network_bandwidth: Option<u64>,
    pub storage_quota: Option<u64>,
}

pub struct TenantUsage {
    pub active_instances: u32,
    pub memory_used: usize,
    pub cpu_time_used: Duration,
    pub network_bytes: u64,
    pub storage_used: u64,
}
```

## 🛠️ Development Experience Improvements

### Better Testing Support

```rust
pub mod testing {
    pub struct SandboxTester {
        pub fn new() -> Self;
        pub fn with_mock_filesystem(&mut self, files: HashMap<PathBuf, Vec<u8>>) -> &mut Self;
        pub fn with_mock_network(&mut self, responses: HashMap<String, MockResponse>) -> &mut Self;
        pub fn expect_security_violation(&mut self, operation: &str) -> &mut Self;
        pub fn run_test<F>(&self, test: F) -> TestResult where F: FnOnce(&mut WasmSandbox) -> BoxFuture<'_, Result<()>>;
    }
    
    pub struct MockResponse {
        pub status: u16,
        pub headers: HashMap<String, String>,
        pub body: Vec<u8>,
    }
}
```

### Configuration Validation

```rust
pub trait ConfigValidator {
    fn validate(&self) -> Result<(), Vec<ConfigError>>;
    fn suggest_improvements(&self) -> Vec<ConfigSuggestion>;
    fn estimate_resource_usage(&self) -> ResourceEstimate;
}

pub struct ConfigError {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

pub struct ConfigSuggestion {
    pub category: SuggestionCategory,
    pub message: String,
    pub impact: Impact,
}

pub enum SuggestionCategory {
    Security,
    Performance,
    Compatibility,
}
```

## 🚀 Performance Optimizations

### Connection Pooling

```rust
pub struct SandboxPool {
    pub fn new(config: PoolConfig) -> Result<Self>;
    pub async fn get_instance(&self) -> Result<PooledInstance>;
    pub fn with_warm_instances(&mut self, count: u32) -> Result<()>;
    pub fn scale_up(&mut self, additional: u32) -> Result<()>;
    pub fn scale_down(&mut self, remove: u32) -> Result<()>;
}

pub struct PoolConfig {
    pub min_instances: u32,
    pub max_instances: u32,
    pub idle_timeout: Duration,
    pub warm_up_instances: u32,
}

pub struct PooledInstance {
    instance_id: InstanceId,
    pool: Arc<SandboxPool>,
}

impl Drop for PooledInstance {
    fn drop(&mut self) {
        // Return to pool automatically
    }
}
```

### JIT Compilation Support

```rust
pub struct JitConfig {
    pub enable_jit: bool,
    pub optimization_level: OptimizationLevel,
    pub compile_threshold: u32,
    pub cache_compiled_code: bool,
}

pub enum OptimizationLevel {
    None,
    Speed,
    Size,
    SpeedAndSize,
}
```

## 📈 Roadmap for v0.3.0

### Phase 1: API Ergonomics (Target: 1 month)
- [ ] Builder pattern for all configuration types
- [ ] Simplified function calling API
- [ ] Enhanced error types with context
- [ ] Configuration validation

### Phase 2: PUP Integration Features (Target: 2 months)
- [ ] Plugin ecosystem traits and helpers
- [ ] Hot reload support
- [ ] Streaming execution
- [ ] Development tools integration

### Phase 3: Production Features (Target: 3 months)
- [ ] Advanced observability and metrics
- [ ] Security auditing and threat analysis
- [ ] Multi-tenant isolation
- [ ] Connection pooling

### Phase 4: Performance and Scale (Target: 4 months)
- [ ] JIT compilation support
- [ ] Advanced caching
- [ ] Batch execution optimizations
- [ ] Memory optimization

## 🤝 Contributing to API Improvements

We welcome contributions that improve the developer experience:

1. **Documentation**: Help improve examples and guides
2. **API Design**: Propose ergonomic improvements
3. **Performance**: Optimize hot paths and memory usage
4. **Security**: Enhance security features and auditing
5. **Testing**: Add comprehensive test coverage

### Feedback Process

1. **Create GitHub Issue**: Describe the API improvement need
2. **Discussion**: Community discussion on the proposal
3. **RFC**: Formal RFC for significant changes
4. **Implementation**: Prototype and test the improvement
5. **Integration**: Merge with full documentation and tests

## 📧 Contact

- **GitHub Issues**: [Report bugs or request features](https://github.com/ciresnave/wasm-sandbox/issues)
- **GitHub Discussions**: [Ask questions and discuss ideas](https://github.com/ciresnave/wasm-sandbox/discussions)
- **Documentation**: [docs.rs/wasm-sandbox](https://docs.rs/wasm-sandbox)

---

*This document will be updated as API improvements are implemented and released.*
