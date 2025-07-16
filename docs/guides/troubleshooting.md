# Troubleshooting Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This comprehensive guide helps you diagnose and resolve common issues with wasm-sandbox applications, including performance problems, security violations, resource limits, compilation errors, and runtime failures.

## Quick Diagnostics

### Health Check Script

```bash
#!/bin/bash
# wasm-sandbox-health-check.sh - Quick diagnostic script

echo "=== WebAssembly Sandbox Health Check ==="
echo "Date: $(date)"
echo

# Check basic system resources
echo "1. System Resources:"
echo "   Memory: $(free -h | grep '^Mem' | awk '{print $3 "/" $2}')"
echo "   CPU: $(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1"%"}')"
echo "   Disk: $(df -h / | tail -1 | awk '{print $5}')"
echo

# Check WebAssembly toolchain
echo "2. WebAssembly Toolchain:"
if command -v rustc &> /dev/null; then
    echo "   Rust: $(rustc --version)"
else
    echo "   Rust: NOT FOUND"
fi

if rustup target list --installed | grep -q wasm32-wasi; then
    echo "   WASM target: INSTALLED"
else
    echo "   WASM target: NOT INSTALLED"
fi

if command -v wasm-pack &> /dev/null; then
    echo "   wasm-pack: $(wasm-pack --version)"
else
    echo "   wasm-pack: NOT FOUND"
fi
echo

# Check application status
echo "3. Application Status:"
if pgrep -f "wasm-sandbox" > /dev/null; then
    echo "   Process: RUNNING (PID: $(pgrep -f 'wasm-sandbox'))"
else
    echo "   Process: NOT RUNNING"
fi

# Test health endpoint
if curl -f -s http://localhost:8080/health > /dev/null; then
    echo "   Health endpoint: HEALTHY"
else
    echo "   Health endpoint: UNHEALTHY"
fi

# Test metrics endpoint
if curl -f -s http://localhost:9090/metrics > /dev/null; then
    echo "   Metrics endpoint: ACCESSIBLE"
else
    echo "   Metrics endpoint: NOT ACCESSIBLE"
fi
echo

echo "=== Health Check Complete ==="
```

### Diagnostic Commands

```rust
use wasm_sandbox::{WasmSandbox, DiagnosticTools};

// Quick diagnostic function
pub async fn diagnose_sandbox_issues() -> DiagnosticReport {
    let mut report = DiagnosticReport::new();
    
    // Test basic functionality
    report.add_check("basic_functionality", test_basic_functionality().await);
    
    // Check resource availability
    report.add_check("resource_availability", check_resource_availability().await);
    
    // Validate configuration
    report.add_check("configuration", validate_configuration().await);
    
    // Test WebAssembly runtime
    report.add_check("wasm_runtime", test_wasm_runtime().await);
    
    // Check security settings
    report.add_check("security_settings", check_security_settings().await);
    
    report
}

async fn test_basic_functionality() -> CheckResult {
    match WasmSandbox::builder()
        .source("fn main() { println!(\"Hello, World!\"); }")
        .build()
        .await
    {
        Ok(sandbox) => {
            match sandbox.call("main", &()).await {
                Ok(_) => CheckResult::pass("Basic functionality working"),
                Err(e) => CheckResult::fail(&format!("Execution failed: {}", e)),
            }
        }
        Err(e) => CheckResult::fail(&format!("Sandbox creation failed: {}", e)),
    }
}
```

## Common Issues and Solutions

### 1. Compilation Errors

#### Issue: "Cannot find Rust compiler"

```text
Error: Rust compiler not found in PATH
```

**Diagnosis:**

```bash
# Check if Rust is installed
rustc --version
# Check PATH
echo $PATH
```

**Solutions:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Add WebAssembly target
rustup target add wasm32-wasi
rustup target add wasm32-unknown-unknown
```

#### Issue: "WebAssembly compilation failed"

```text
Error: WebAssembly compilation failed: unknown import: `env::some_function`
```

**Diagnosis:**

```rust
// Check what imports your module expects
let module_bytes = std::fs::read("module.wasm")?;
let imports = wasm_sandbox::utils::analyze_imports(&module_bytes)?;
println!("Required imports: {:#?}", imports);
```

**Solutions:**

```rust
// Provide missing host functions
let sandbox = WasmSandbox::builder()
    .source("my_program.rs")
    .bind_host_function("some_function", |args| {
        // Implementation
        Ok(serde_json::Value::Null)
    })
    .build()
    .await?;

// Or disable missing imports if not needed
let sandbox = WasmSandbox::builder()
    .source("my_program.rs")
    .allow_missing_imports(true)
    .build()
    .await?;
```

#### Issue: "Target wasm32-wasi not found"

```text
Error: couldn't find `core` for target `wasm32-wasi`
```

**Solutions:**

```bash
# Install the target
rustup target add wasm32-wasi

# Verify installation
rustup target list --installed | grep wasm

# Update Rust if needed
rustup update
```

### 2. Runtime Errors

#### Issue: "Memory limit exceeded"

```text
Error: Memory limit exceeded: used 134217728, limit 67108864
```

**Diagnosis:**

```rust
// Check memory usage patterns
let sandbox = WasmSandbox::builder()
    .source("memory_intensive.rs")
    .enable_memory_profiling(true)
    .build()
    .await?;

let result = sandbox.call_with_profiling("function", &input).await?;
println!("Peak memory: {} bytes", result.peak_memory_usage);
```

**Solutions:**

```rust
// Increase memory limit
let sandbox = WasmSandbox::builder()
    .source("memory_intensive.rs")
    .memory_limit(256 * 1024 * 1024) // 256MB
    .build()
    .await?;

// Or optimize memory usage in your code
#[no_mangle]
pub extern "C" fn optimized_function() {
    // Use streaming instead of loading everything
    // Release large allocations early
    // Use Vec::with_capacity() when size is known
}

// Enable memory tracking
let sandbox = WasmSandbox::builder()
    .source("tracked.rs")
    .enable_memory_tracking(true)
    .memory_callback(|usage| {
        if usage.percentage > 80.0 {
            println!("Warning: High memory usage: {:.1}%", usage.percentage);
        }
    })
    .build()
    .await?;
```

#### Issue: "Execution timeout"

```text
Error: Execution timeout after 30 seconds
```

**Diagnosis:**

```rust
// Profile execution time
let start = std::time::Instant::now();
let result = sandbox.call("slow_function", &input).await;
println!("Execution took: {:?}", start.elapsed());

// Check fuel consumption
let sandbox = WasmSandbox::builder()
    .source("slow.rs")
    .max_fuel(Some(100_000_000))
    .fuel_callback(|consumed, remaining| {
        println!("Fuel: {} consumed, {} remaining", consumed, remaining);
    })
    .build()
    .await?;
```

**Solutions:**

```rust
// Increase timeout
let sandbox = WasmSandbox::builder()
    .source("slow.rs")
    .timeout_duration(Duration::from_secs(120)) // 2 minutes
    .build()
    .await?;

// Increase fuel limit
let sandbox = WasmSandbox::builder()
    .source("compute_heavy.rs")
    .max_fuel(Some(1_000_000_000)) // 1B instructions
    .build()
    .await?;

// Optimize algorithm complexity
#[no_mangle]
pub extern "C" fn optimized_algorithm() {
    // Use more efficient algorithms
    // Break large computations into smaller chunks
    // Consider iterative vs recursive approaches
}

// Use chunked processing
pub async fn process_large_dataset(sandbox: &WasmSandbox, data: Vec<u8>) -> Result<Vec<u8>, Error> {
    let chunk_size = 1024 * 1024; // 1MB chunks
    let mut results = Vec::new();
    
    for chunk in data.chunks(chunk_size) {
        let result = sandbox.call("process_chunk", chunk).await?;
        results.extend(result);
    }
    
    Ok(results)
}
```

#### Issue: "Fuel exhausted"

```text
Error: Fuel exhausted: used 1000000, limit 1000000
```

**Solutions:**

```rust
// Increase fuel limit
let sandbox = WasmSandbox::builder()
    .source("compute_heavy.rs")
    .max_fuel(Some(10_000_000)) // 10M instructions
    .build()
    .await?;

// Monitor fuel usage
let sandbox = WasmSandbox::builder()
    .source("monitored.rs")
    .fuel_metering(true)
    .fuel_callback(|consumed, remaining| {
        let percentage = (consumed as f64 / (consumed + remaining) as f64) * 100.0;
        if percentage > 80.0 {
            println!("Warning: {}% fuel consumed", percentage);
        }
    })
    .build()
    .await?;

// Optimize computational complexity
// Use fuel-efficient algorithms and data structures
```

### 3. Security Issues

#### Issue: "Capability denied"

```text
Error: Capability 'network' denied for sandbox
```

**Diagnosis:**

```rust
// Check what capabilities are being requested
let audit_log = sandbox.get_security_audit().await?;
for event in audit_log.events {
    if let SecurityEvent::CapabilityRequest { capability, denied } = event {
        println!("Capability '{}' requested, denied: {}", capability, denied);
    }
}
```

**Solutions:**

```rust
// Grant necessary capabilities
let sandbox = WasmSandbox::builder()
    .source("network_client.rs")
    .allow_capability(Capability::Network)
    .allowed_hosts(&["api.example.com", "cdn.example.com"])
    .build()
    .await?;

// Or use a more permissive security policy
let sandbox = WasmSandbox::builder()
    .source("trusted_code.rs")
    .security_policy(SecurityPolicy::Permissive)
    .build()
    .await?;

// For development only - disable security (NOT for production)
let sandbox = WasmSandbox::builder()
    .source("dev_code.rs")
    .disable_security(true) // WARNING: Only for development
    .build()
    .await?;
```

#### Issue: "File access denied"

```text
Error: File access denied: /etc/passwd
```

**Solutions:**

```rust
// Allow specific file access
let sandbox = WasmSandbox::builder()
    .source("file_processor.rs")
    .allow_capability(Capability::Filesystem)
    .allowed_paths(&["/tmp", "/home/user/data"])
    .build()
    .await?;

// Use sandboxed filesystem
let sandbox = WasmSandbox::builder()
    .source("safe_file_access.rs")
    .filesystem_policy(FilesystemPolicy::Sandboxed)
    .sandbox_directory("/tmp/wasm_sandbox")
    .build()
    .await?;
```

### 4. Performance Issues

#### Issue: "Slow execution performance"

**Diagnosis:**

```rust
// Profile execution
use std::time::Instant;

let start = Instant::now();
let result = sandbox.call("slow_function", &input).await?;
let duration = start.elapsed();

println!("Execution took: {:?}", duration);

if duration > Duration::from_millis(1000) {
    println!("Performance issue detected");
}

// Use detailed profiling
let result = sandbox.call_with_profiling("function", &input).await?;
println!("Compilation time: {:?}", result.compilation_time);
println!("Execution time: {:?}", result.execution_time);
println!("Memory allocated: {} bytes", result.memory_allocated);
```

**Solutions:**

```rust
// Enable optimizations
let sandbox = WasmSandbox::builder()
    .source("optimized.rs")
    .optimization_level(OptimizationLevel::Speed)
    .enable_simd(true)
    .build()
    .await?;

// Use module caching
let cache = ModuleCache::new(100);
let sandbox = WasmSandbox::builder()
    .source("cached.rs")
    .module_cache(cache)
    .build()
    .await?;

// Pre-compile modules
let compiled_module = compile_module_ahead_of_time("module.rs").await?;
let sandbox = WasmSandbox::from_compiled_module(compiled_module).await?;

// Use connection pooling
let pool = SandboxPool::new()
    .max_instances(10)
    .build()
    .await?;

let sandbox = pool.get().await?;
```

#### Issue: "High memory usage"

**Diagnosis:**

```rust
// Monitor memory usage
let sandbox = WasmSandbox::builder()
    .source("memory_test.rs")
    .enable_memory_monitoring(true)
    .memory_callback(|usage| {
        println!("Memory: {} / {} bytes ({:.1}%)",
                usage.used, usage.limit, usage.percentage);
    })
    .build()
    .await?;

// Get memory profile
let profile = sandbox.get_memory_profile().await?;
println!("Peak usage: {} bytes", profile.peak_usage);
println!("Allocation count: {}", profile.allocation_count);
```

**Solutions:**

```rust
// Optimize memory usage
#[no_mangle]
pub extern "C" fn memory_efficient() {
    // Use streaming instead of loading all data
    // Release large allocations early with drop()
    // Use Vec::with_capacity() when size is known
    // Consider using Box<[T]> instead of Vec<T> for fixed-size data
}

// Set appropriate memory limits
let sandbox = WasmSandbox::builder()
    .source("optimized.rs")
    .memory_limit(64 * 1024 * 1024) // Start with smaller limit
    .memory_growth_limit(16 * 1024 * 1024) // Allow gradual growth
    .build()
    .await?;
```

### 5. Module Loading Issues

#### Issue: "Invalid WebAssembly module"

```text
Error: Invalid WebAssembly module: unexpected end of input
```

**Diagnosis:**

```bash
# Check if the file is valid WebAssembly
file module.wasm
wasm-objdump -h module.wasm

# Validate the module
wasm-validate module.wasm
```

**Solutions:**

```rust
// Validate module before loading
fn validate_wasm_module(path: &Path) -> Result<(), Error> {
    let bytes = std::fs::read(path)?;
    
    // Check magic number
    if &bytes[0..4] != b"\x00asm" {
        return Err(Error::InvalidModule("Not a WebAssembly module".to_string()));
    }
    
    // Check version
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != 1 {
        return Err(Error::InvalidModule(format!("Unsupported version: {}", version)));
    }
    
    Ok(())
}

// Use more robust module loading
let sandbox = WasmSandbox::builder()
    .source_file("module.wasm")
    .validate_module(true)
    .build()
    .await?;
```

#### Issue: "Module compilation failed"

```text
Error: Module compilation failed: type mismatch
```

**Solutions:**

```rust
// Use compatible compilation settings
let sandbox = WasmSandbox::builder()
    .source("fixed.rs")
    .target("wasm32-wasi") // Explicit target
    .features(&["wasi"]) // Required features
    .build()
    .await?;

// Check for ABI compatibility
let sandbox = WasmSandbox::builder()
    .source("abi_compatible.rs")
    .abi_version("1.0")
    .strict_abi_checking(true)
    .build()
    .await?;
```

### 6. Network and I/O Issues

#### Issue: "Network connection failed"

```text
Error: Network connection failed: Connection refused
```

**Diagnosis:**

```rust
// Test network connectivity
async fn test_network_connectivity() -> Result<(), Error> {
    let client = reqwest::Client::new();
    let response = client.get("https://httpbin.org/get").send().await?;
    println!("Network test: {}", response.status());
    Ok(())
}

// Check allowed hosts
let sandbox = WasmSandbox::builder()
    .source("network_test.rs")
    .allow_capability(Capability::Network)
    .allowed_hosts(&["httpbin.org", "api.example.com"])
    .network_debug(true) // Enable network debugging
    .build()
    .await?;
```

**Solutions:**

```rust
// Configure network access properly
let sandbox = WasmSandbox::builder()
    .source("network_client.rs")
    .allow_capability(Capability::Network)
    .allowed_hosts(&["api.example.com"])
    .network_timeout(Duration::from_secs(30))
    .max_connections(10)
    .build()
    .await?;

// Handle network errors gracefully
#[no_mangle]
pub extern "C" fn robust_network_call() -> i32 {
    match make_http_request("https://api.example.com/data") {
        Ok(data) => {
            // Process data
            0
        }
        Err(e) => {
            eprintln!("Network error: {}", e);
            -1
        }
    }
}
```

#### Issue: "File not found"

```text
Error: File not found: /app/data/input.txt
```

**Solutions:**

```rust
// Use relative paths and proper file access
let sandbox = WasmSandbox::builder()
    .source("file_reader.rs")
    .allow_capability(Capability::Filesystem)
    .allowed_paths(&["./data", "/tmp"])
    .working_directory("./")
    .build()
    .await?;

// Mount directories into sandbox
let sandbox = WasmSandbox::builder()
    .source("mounted_fs.rs")
    .mount_directory("./data", "/sandbox/data")
    .mount_directory("/tmp", "/sandbox/tmp")
    .build()
    .await?;
```

## Debugging Tools

### Debug Mode

```rust
// Enable comprehensive debugging
let sandbox = WasmSandbox::builder()
    .source("debug_me.rs")
    .debug_mode(true)
    .verbose_logging(true)
    .enable_tracing(true)
    .build()
    .await?;

// Set breakpoints (if supported by runtime)
sandbox.set_breakpoint("function_name", 42).await?;

// Step through execution
while let Some(step) = sandbox.step_execution().await? {
    println!("At line {}: {}", step.line, step.instruction);
    
    // Inspect variables
    let vars = sandbox.get_local_variables().await?;
    println!("Variables: {:?}", vars);
}
```

### Memory Debugging

```rust
// Enable memory debugging
let sandbox = WasmSandbox::builder()
    .source("memory_debug.rs")
    .enable_memory_debugging(true)
    .track_allocations(true)
    .build()
    .await?;

// Get memory dump
let memory_dump = sandbox.dump_memory().await?;
println!("Memory dump: {} bytes", memory_dump.len());

// Analyze memory usage
let analysis = sandbox.analyze_memory_usage().await?;
println!("Leaked objects: {}", analysis.leaked_objects);
println!("Fragmentation: {:.1}%", analysis.fragmentation_percentage);
```

### Performance Profiling

```rust
// Enable performance profiling
let sandbox = WasmSandbox::builder()
    .source("profile_me.rs")
    .enable_profiling(true)
    .profile_memory(true)
    .profile_cpu(true)
    .build()
    .await?;

// Get profiling results
let result = sandbox.call_with_profiling("function", &input).await?;

println!("=== Performance Profile ===");
println!("Total time: {:?}", result.total_time);
println!("Compilation: {:?}", result.compilation_time);
println!("Execution: {:?}", result.execution_time);
println!("Memory peak: {} bytes", result.peak_memory);
println!("Instructions: {}", result.instruction_count);

// Get hotspots
for hotspot in result.hotspots {
    println!("Hotspot: {} - {:?}", hotspot.function, hotspot.duration);
}
```

## Log Analysis

### Log Patterns for Common Issues

```bash
# Memory issues
grep -E "(memory|Memory|MEMORY)" /var/log/wasm-sandbox.log
grep -E "(limit.*exceeded|out of memory)" /var/log/wasm-sandbox.log

# Security violations
grep -E "(security|Security|violation|denied)" /var/log/wasm-sandbox.log
grep -E "(capability.*denied|access.*denied)" /var/log/wasm-sandbox.log

# Performance issues
grep -E "(timeout|slow|performance)" /var/log/wasm-sandbox.log
grep -E "(duration.*[0-9]{4,}|took.*[5-9][0-9]{3})" /var/log/wasm-sandbox.log

# Compilation errors
grep -E "(compilation.*failed|compile.*error)" /var/log/wasm-sandbox.log
grep -E "(rustc.*error|wasm.*invalid)" /var/log/wasm-sandbox.log
```

### Structured Log Analysis

```rust
use serde_json::{Value, from_str};

pub fn analyze_logs(log_file: &str) -> Result<LogAnalysis, Error> {
    let content = std::fs::read_to_string(log_file)?;
    let mut analysis = LogAnalysis::new();
    
    for line in content.lines() {
        if let Ok(log_entry) = from_str::<Value>(line) {
            match log_entry.get("level").and_then(|l| l.as_str()) {
                Some("ERROR") => {
                    analysis.errors.push(parse_error_entry(&log_entry));
                }
                Some("WARN") => {
                    analysis.warnings.push(parse_warning_entry(&log_entry));
                }
                _ => {}
            }
            
            // Check for specific patterns
            if let Some(message) = log_entry.get("message").and_then(|m| m.as_str()) {
                if message.contains("memory limit exceeded") {
                    analysis.memory_issues += 1;
                } else if message.contains("timeout") {
                    analysis.timeout_issues += 1;
                } else if message.contains("security violation") {
                    analysis.security_issues += 1;
                }
            }
        }
    }
    
    Ok(analysis)
}

#[derive(Debug)]
pub struct LogAnalysis {
    pub errors: Vec<ErrorEntry>,
    pub warnings: Vec<WarningEntry>,
    pub memory_issues: u32,
    pub timeout_issues: u32,
    pub security_issues: u32,
    pub performance_issues: u32,
}
```

## Recovery Procedures

### Automatic Recovery

```rust
use tokio::time::{interval, Duration};

pub struct RecoveryManager {
    sandbox_pool: Arc<SandboxPool>,
    health_checker: HealthChecker,
    recovery_strategies: Vec<Box<dyn RecoveryStrategy>>,
}

impl RecoveryManager {
    pub async fn start_monitoring(&self) {
        let mut interval = interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            if let Err(issues) = self.health_checker.check_health().await {
                for issue in issues {
                    self.attempt_recovery(&issue).await;
                }
            }
        }
    }
    
    async fn attempt_recovery(&self, issue: &HealthIssue) -> Result<(), RecoveryError> {
        for strategy in &self.recovery_strategies {
            if strategy.can_handle(issue) {
                match strategy.recover(issue).await {
                    Ok(_) => {
                        tracing::info!(
                            issue = %issue.description,
                            strategy = %strategy.name(),
                            "Recovery successful"
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        tracing::warn!(
                            issue = %issue.description,
                            strategy = %strategy.name(),
                            error = %e,
                            "Recovery strategy failed"
                        );
                    }
                }
            }
        }
        
        Err(RecoveryError::NoStrategyAvailable)
    }
}

// Recovery strategies
pub struct MemoryRecoveryStrategy;

#[async_trait::async_trait]
impl RecoveryStrategy for MemoryRecoveryStrategy {
    fn can_handle(&self, issue: &HealthIssue) -> bool {
        matches!(issue.issue_type, IssueType::HighMemoryUsage)
    }
    
    async fn recover(&self, _issue: &HealthIssue) -> Result<(), RecoveryError> {
        // Force garbage collection
        // Restart instances with high memory usage
        // Increase memory limits temporarily
        Ok(())
    }
    
    fn name(&self) -> &str {
        "memory_recovery"
    }
}
```

### Manual Recovery Steps

```bash
#!/bin/bash
# emergency-recovery.sh - Manual recovery procedures

echo "=== Emergency Recovery Procedures ==="

# 1. Stop all processes
echo "1. Stopping processes..."
pkill -f wasm-sandbox
sleep 5

# 2. Clear temporary files
echo "2. Cleaning temporary files..."
rm -rf /tmp/wasm-sandbox-*
rm -rf /var/cache/wasm-sandbox/*

# 3. Reset resource limits
echo "3. Resetting resource limits..."
# Reset memory limits, restart services, etc.

# 4. Restart with safe configuration
echo "4. Restarting with safe configuration..."
export WASM_MEMORY_LIMIT_MB=64
export WASM_TIMEOUT_SECONDS=30
export WASM_ENABLE_NETWORK=false

# Start application
./target/release/wasm-sandbox-app --safe-mode

echo "=== Recovery Complete ==="
```

## Monitoring Integration

### Error Tracking

```rust
use sentry::{configure_scope, capture_exception, capture_message, Level};

pub fn setup_error_tracking() {
    let _guard = sentry::init(sentry::ClientOptions {
        dsn: env::var("SENTRY_DSN").ok(),
        ..Default::default()
    });
    
    configure_scope(|scope| {
        scope.set_tag("component", "wasm-sandbox");
        scope.set_context("runtime", sentry::protocol::Context::Other({
            let mut map = std::collections::BTreeMap::new();
            map.insert("version".to_string(), env!("CARGO_PKG_VERSION").into());
            map.insert("target".to_string(), "wasm32-wasi".into());
            map
        }));
    });
}

pub fn report_sandbox_error(error: &Error, context: &ExecutionContext) {
    configure_scope(|scope| {
        scope.set_tag("execution_id", &context.execution_id);
        scope.set_tag("sandbox_id", &context.sandbox_id);
        scope.set_extra("function", context.function.clone().into());
        scope.set_extra("input_size", context.input_size.into());
    });
    
    capture_exception(error);
}
```

## Best Practices for Troubleshooting

### 1. Systematic Debugging

```rust
// Use structured debugging approach
pub async fn debug_execution_failure(
    source: &str,
    function: &str,
    input: &serde_json::Value,
) -> DebugReport {
    let mut report = DebugReport::new();
    
    // Step 1: Test compilation
    match compile_source(source).await {
        Ok(module) => report.compilation = Some(CompilationResult::Success),
        Err(e) => {
            report.compilation = Some(CompilationResult::Failed(e.to_string()));
            return report; // Can't proceed without compilation
        }
    }
    
    // Step 2: Test instantiation
    match create_instance(&module).await {
        Ok(instance) => report.instantiation = Some(InstantiationResult::Success),
        Err(e) => {
            report.instantiation = Some(InstantiationResult::Failed(e.to_string()));
            return report;
        }
    }
    
    // Step 3: Test execution with minimal input
    let minimal_input = serde_json::json!({});
    match execute_function(&instance, function, &minimal_input).await {
        Ok(_) => report.minimal_execution = Some(ExecutionResult::Success),
        Err(e) => {
            report.minimal_execution = Some(ExecutionResult::Failed(e.to_string()));
        }
    }
    
    // Step 4: Test with actual input
    match execute_function(&instance, function, input).await {
        Ok(result) => report.full_execution = Some(ExecutionResult::Success),
        Err(e) => {
            report.full_execution = Some(ExecutionResult::Failed(e.to_string()));
        }
    }
    
    report
}
```

### 2. Progressive Testing

```rust
// Test with increasing complexity
pub async fn progressive_testing(source: &str) -> Vec<TestResult> {
    let mut results = Vec::new();
    
    // Test 1: Empty function
    results.push(test_empty_function(source).await);
    
    // Test 2: Simple return
    results.push(test_simple_return(source).await);
    
    // Test 3: Basic operations
    results.push(test_basic_operations(source).await);
    
    // Test 4: Memory operations
    results.push(test_memory_operations(source).await);
    
    // Test 5: Full functionality
    results.push(test_full_functionality(source).await);
    
    results
}
```

### 3. Environment Validation

```rust
pub fn validate_environment() -> EnvironmentReport {
    let mut report = EnvironmentReport::new();
    
    // Check Rust installation
    report.rust_version = check_rust_version();
    
    // Check WebAssembly targets
    report.wasm_targets = check_wasm_targets();
    
    // Check system resources
    report.system_resources = check_system_resources();
    
    // Check dependencies
    report.dependencies = check_dependencies();
    
    report
}
```

## Next Steps

- **[Monitoring Guide](monitoring.md)** - Set up monitoring to prevent issues
- **[Performance Guide](../design/performance.md)** - Optimize performance
- **[Security Configuration](security-config.md)** - Secure your deployment
- **[Production Deployment](production.md)** - Deploy reliably

---

**Troubleshooting Success:** This guide provides systematic approaches to diagnosing and resolving issues. Start with the quick diagnostics and work through the systematic debugging procedures for complex problems.
