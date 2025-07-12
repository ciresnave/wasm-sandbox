// Plugin Ecosystem Example - Building a secure plugin system
//
// This example demonstrates how to build a secure plugin ecosystem using wasm-sandbox,
// providing patterns that can be adapted for any application requiring safe execution
// of untrusted code (PUP, CI/CD systems, serverless platforms, etc.)

use wasm_sandbox::{WasmSandbox, InstanceConfig, ModuleId};
use wasm_sandbox::runtime::{WasmInstanceExt, WasmRuntimeExt};
use wasm_sandbox::security::{Capabilities, ResourceLimits, FilesystemCapability, NetworkCapability};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use tokio::sync::{RwLock, mpsc};
use std::sync::Arc;

// Plugin manifest structure (generic design for any application)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub permissions: PluginPermissions,
    pub entry_points: Vec<EntryPoint>,
    pub dependencies: Vec<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    pub filesystem: Vec<FilesystemAccess>,
    pub network: NetworkPolicy,
    pub system: SystemPolicy,
    pub resources: ResourcePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemAccess {
    pub path: PathBuf,
    pub access_type: String, // "read", "write", "readwrite"
    pub recursive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub allowed_domains: Vec<String>,
    pub allowed_ports: Vec<u16>,
    pub max_connections: usize,
    pub enable_https: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPolicy {
    pub env_var_access: Vec<String>,
    pub process_spawn: bool,
    pub max_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePolicy {
    pub max_memory_mb: usize,
    pub max_execution_time_sec: u64,
    pub max_fuel: u64,
    pub max_file_handles: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub name: String,
    pub function: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

// Plugin execution context
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub plugin_id: String,
    pub request_id: String,
    pub user_data: serde_json::Value,
    pub environment: HashMap<String, String>,
    pub input_data: serde_json::Value,
}

// Plugin execution result
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output_data: serde_json::Value,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub resource_usage: ResourceUsageReport,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceUsageReport {
    pub memory_used_mb: f64,
    pub memory_peak_mb: f64,
    pub cpu_time_ms: u64,
    pub function_calls: u64,
    pub file_operations: u64,
    pub network_requests: u64,
}

// Security audit report
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityReport {
    pub is_safe: bool,
    pub risk_level: RiskLevel,
    pub violations: Vec<SecurityViolation>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityViolation {
    pub violation_type: String,
    pub description: String,
    pub severity: String,
    pub recommendation: String,
}

// Performance benchmark report
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub startup_time_ms: u64,
    pub memory_efficiency: f64,
    pub cpu_efficiency: f64,
    pub io_performance: f64,
    pub recommendations: Vec<String>,
}

// Plugin registry for managing plugins
pub struct PluginRegistry {
    sandbox: WasmSandbox,
    plugins: Arc<RwLock<HashMap<String, PluginInfo>>>,
    module_cache: Arc<RwLock<HashMap<String, ModuleId>>>,
    event_channel: mpsc::Sender<PluginEvent>,
}

#[derive(Debug, Clone)]
struct PluginInfo {
    manifest: PluginManifest,
    module_id: ModuleId,
    instance_id: Option<u32>,
    status: PluginStatus,
    statistics: PluginStatistics,
}

#[derive(Debug, Clone)]
pub enum PluginStatus {
    Loaded,
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct PluginStatistics {
    pub executions: u64,
    pub total_runtime: Duration,
    pub average_memory_usage: f64,
    pub error_count: u64,
    pub last_execution: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Loaded(String),
    Started(String),
    Stopped(String),
    Error { plugin_id: String, error: String },
    SecurityViolation { plugin_id: String, violation: String },
    ResourceExhausted { plugin_id: String, resource: String },
}

impl PluginRegistry {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let sandbox = WasmSandbox::new()?;
        let (event_tx, mut event_rx) = mpsc::channel(100);
        
        // Spawn event handler
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                Self::handle_event(event).await;
            }
        });
        
        Ok(Self {
            sandbox,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            module_cache: Arc::new(RwLock::new(HashMap::new())),
            event_channel: event_tx,
        })
    }
    
    // Validate and audit plugin security (addressing PUP feedback)
    pub async fn validate_plugin(&self, wasm_bytes: &[u8]) -> Result<SecurityReport, Box<dyn std::error::Error>> {
        println!("🔍 Validating plugin security...");
        
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();
        
        // Static analysis of WASM module
        let module = wasmparser::Parser::new(0).parse_all(wasm_bytes);
        
        let mut has_dangerous_imports = false;
        let mut has_excessive_memory = false;
        
        for payload in module {
            match payload? {
                wasmparser::Payload::ImportSection(reader) => {
                    for import in reader {
                        let import = import?;
                        match import.name {
                            "system" | "exec" | "spawn" => {
                                has_dangerous_imports = true;
                                violations.push(SecurityViolation {
                                    violation_type: "dangerous_import".to_string(),
                                    description: format!("Dangerous import detected: {}", import.name),
                                    severity: "HIGH".to_string(),
                                    recommendation: "Remove system-level imports".to_string(),
                                });
                            }
                            _ => {}
                        }
                    }
                }
                wasmparser::Payload::MemorySection(reader) => {
                    for memory in reader {
                        let memory = memory?;
                        if memory.initial > 1000 { // More than 1000 pages (64MB)
                            has_excessive_memory = true;
                            violations.push(SecurityViolation {
                                violation_type: "excessive_memory".to_string(),
                                description: format!("Excessive memory request: {} pages", memory.initial),
                                severity: "MEDIUM".to_string(),
                                recommendation: "Reduce memory requirements".to_string(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Generate recommendations
        if has_dangerous_imports {
            recommendations.push("Use safe WASI imports instead of system-level access".to_string());
        }
        if has_excessive_memory {
            recommendations.push("Optimize memory usage and use streaming for large data".to_string());
        }
        
        let risk_level = if has_dangerous_imports {
            RiskLevel::High
        } else if has_excessive_memory {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };
        
        let is_safe = matches!(risk_level, RiskLevel::Low | RiskLevel::Medium);
        
        Ok(SecurityReport {
            is_safe,
            risk_level,
            violations,
            recommendations,
        })
    }
    
    // Benchmark plugin performance
    pub async fn benchmark_plugin(&self, wasm_bytes: &[u8]) -> Result<PerformanceReport, Box<dyn std::error::Error>> {
        println!("📊 Benchmarking plugin performance...");
        
        let start_time = std::time::Instant::now();
        
        // Load module for benchmarking
        let module_id = self.sandbox.load_module(wasm_bytes).await?;
        let startup_time = start_time.elapsed();
        
        // Create minimal instance for testing
        let config = InstanceConfig {
            capabilities: Capabilities::minimal(),
            resource_limits: ResourceLimits {
                memory_bytes: Some(32 * 1024 * 1024), // 32MB for testing
                execution_timeout: Some(Duration::from_secs(5)),
                ..ResourceLimits::default()
            },
        };
        
        let instance_id = self.sandbox.create_instance(module_id, Some(config)).await?;
        
        // Run simple benchmark
        let bench_start = std::time::Instant::now();
        let _result: i32 = self.sandbox.get_instance(instance_id)?
            .call_function("benchmark_function", &42i32).await
            .unwrap_or(0);
        let execution_time = bench_start.elapsed();
        
        // Get resource usage
        let resource_usage = self.sandbox.get_resource_usage(instance_id)?;
        
        // Calculate efficiency metrics
        let memory_efficiency = 1.0 - (resource_usage.memory_used as f64 / (32.0 * 1024.0 * 1024.0));
        let cpu_efficiency = if execution_time.as_millis() > 0 {
            1.0 / execution_time.as_millis() as f64
        } else {
            1.0
        };
        
        // Cleanup
        self.sandbox.terminate_instance(instance_id).await?;
        
        let mut recommendations = Vec::new();
        if memory_efficiency < 0.5 {
            recommendations.push("Consider reducing memory allocation".to_string());
        }
        if execution_time.as_millis() > 100 {
            recommendations.push("Optimize computational complexity".to_string());
        }
        
        Ok(PerformanceReport {
            startup_time_ms: startup_time.as_millis() as u64,
            memory_efficiency,
            cpu_efficiency,
            io_performance: 1.0, // Placeholder
            recommendations,
        })
    }
    
    // Load plugin with manifest
    pub async fn load_plugin(&mut self, wasm_bytes: &[u8], manifest: PluginManifest) -> Result<(), Box<dyn std::error::Error>> {
        println!("📦 Loading plugin: {} v{}", manifest.name, manifest.version);
        
        // Validate plugin first
        let security_report = self.validate_plugin(wasm_bytes).await?;
        if !security_report.is_safe {
            return Err(format!("Plugin failed security validation: {:?}", security_report.violations).into());
        }
        
        // Load module
        let module_id = self.sandbox.load_module(wasm_bytes).await?;
        
        // Create plugin info
        let plugin_info = PluginInfo {
            manifest: manifest.clone(),
            module_id,
            instance_id: None,
            status: PluginStatus::Loaded,
            statistics: PluginStatistics::default(),
        };
        
        // Store in registry
        let mut plugins = self.plugins.write().await;
        plugins.insert(manifest.id.clone(), plugin_info);
        
        // Cache module
        let mut cache = self.module_cache.write().await;
        cache.insert(manifest.id.clone(), module_id);
        
        // Emit event
        self.event_channel.send(PluginEvent::Loaded(manifest.id)).await?;
        
        println!("✅ Plugin loaded successfully");
        Ok(())
    }
    
    // Start plugin instance
    pub async fn start_plugin(&mut self, plugin_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut plugins = self.plugins.write().await;
        let plugin = plugins.get_mut(plugin_id)
            .ok_or("Plugin not found")?;
        
        if plugin.instance_id.is_some() {
            return Err("Plugin already running".into());
        }
        
        // Convert manifest permissions to instance config
        let config = self.manifest_to_config(&plugin.manifest)?;
        
        // Create instance
        let instance_id = self.sandbox.create_instance(plugin.module_id, Some(config)).await?;
        
        // Initialize plugin
        let init_context = ExecutionContext {
            plugin_id: plugin_id.to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
            user_data: serde_json::Value::Null,
            environment: HashMap::new(),
            input_data: serde_json::Value::Null,
        };
        
        let _init_result: bool = self.sandbox.get_instance(instance_id)?
            .call_function("initialize", &init_context).await?;
        
        // Update plugin state
        plugin.instance_id = Some(instance_id);
        plugin.status = PluginStatus::Running;
        
        // Emit event
        self.event_channel.send(PluginEvent::Started(plugin_id.to_string())).await?;
        
        println!("🚀 Plugin started: {}", plugin_id);
        Ok(())
    }
    
    // Execute plugin function (type-safe)
    pub async fn execute_plugin<P, R>(
        &mut self,
        plugin_id: &str,
        function_name: &str,
        params: &P,
    ) -> Result<R, Box<dyn std::error::Error>>
    where
        P: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de> + Send + Sync,
    {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(plugin_id)
            .ok_or("Plugin not found")?;
        
        let instance_id = plugin.instance_id
            .ok_or("Plugin not running")?;
        
        let start_time = std::time::Instant::now();
        
        // Execute function
        let result = self.sandbox.get_instance(instance_id)?
            .call_function(function_name, params).await?;
        
        let execution_time = start_time.elapsed();
        
        // Update statistics (would need to modify plugin in practice)
        drop(plugins);
        let mut plugins = self.plugins.write().await;
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.statistics.executions += 1;
            plugin.statistics.total_runtime += execution_time;
            plugin.statistics.last_execution = Some(std::time::SystemTime::now());
        }
        
        Ok(result)
    }
    
    // Stop plugin
    pub async fn stop_plugin(&mut self, plugin_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut plugins = self.plugins.write().await;
        let plugin = plugins.get_mut(plugin_id)
            .ok_or("Plugin not found")?;
        
        if let Some(instance_id) = plugin.instance_id {
            self.sandbox.terminate_instance(instance_id).await?;
            plugin.instance_id = None;
            plugin.status = PluginStatus::Stopped;
            
            // Emit event
            self.event_channel.send(PluginEvent::Stopped(plugin_id.to_string())).await?;
            
            println!("🛑 Plugin stopped: {}", plugin_id);
        }
        
        Ok(())
    }
    
    // Hot reload plugin (addressing PUP feedback)
    pub async fn hot_reload_plugin(&mut self, plugin_id: &str, new_wasm_bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Hot reloading plugin: {}", plugin_id);
        
        // Get current plugin state
        let old_status = {
            let plugins = self.plugins.read().await;
            let plugin = plugins.get(plugin_id).ok_or("Plugin not found")?;
            plugin.status.clone()
        };
        
        // Stop current instance if running
        if matches!(old_status, PluginStatus::Running) {
            self.stop_plugin(plugin_id).await?;
        }
        
        // Load new module
        let new_module_id = self.sandbox.load_module(new_wasm_bytes).await?;
        
        // Update plugin info
        {
            let mut plugins = self.plugins.write().await;
            let plugin = plugins.get_mut(plugin_id).ok_or("Plugin not found")?;
            plugin.module_id = new_module_id;
            plugin.status = PluginStatus::Loaded;
        }
        
        // Update cache
        {
            let mut cache = self.module_cache.write().await;
            cache.insert(plugin_id.to_string(), new_module_id);
        }
        
        // Restart if it was running
        if matches!(old_status, PluginStatus::Running) {
            self.start_plugin(plugin_id).await?;
        }
        
        println!("✅ Plugin hot reloaded successfully");
        Ok(())
    }
    
    // Get plugin statistics
    pub async fn get_plugin_stats(&self, plugin_id: &str) -> Result<PluginStatistics, Box<dyn std::error::Error>> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(plugin_id)
            .ok_or("Plugin not found")?;
        
        Ok(plugin.statistics.clone())
    }
    
    // List all plugins
    pub async fn list_plugins(&self) -> Vec<(String, PluginStatus)> {
        let plugins = self.plugins.read().await;
        plugins.iter()
            .map(|(id, info)| (id.clone(), info.status.clone()))
            .collect()
    }
    
    // Convert manifest permissions to instance config
    fn manifest_to_config(&self, manifest: &PluginManifest) -> Result<InstanceConfig, Box<dyn std::error::Error>> {
        let mut filesystem_caps = Vec::new();
        
        for fs_access in &manifest.permissions.filesystem {
            let cap = match fs_access.access_type.as_str() {
                "read" => FilesystemCapability::ReadOnly(fs_access.path.clone()),
                "write" => FilesystemCapability::WriteOnly(fs_access.path.clone()),
                "readwrite" => FilesystemCapability::ReadWrite(fs_access.path.clone()),
                _ => return Err(format!("Invalid filesystem access type: {}", fs_access.access_type).into()),
            };
            filesystem_caps.push(cap);
        }
        
        let mut network_caps = Vec::new();
        for domain in &manifest.permissions.network.allowed_domains {
            network_caps.push(NetworkCapability::HttpsAccess(domain.clone()));
        }
        
        let capabilities = Capabilities {
            filesystem: filesystem_caps,
            network: network_caps,
            environment: manifest.permissions.system.env_var_access.clone(),
            system_calls: Vec::new(), // Restricted by default
        };
        
        let resource_limits = ResourceLimits {
            memory_bytes: Some(manifest.permissions.resources.max_memory_mb * 1024 * 1024),
            execution_timeout: Some(Duration::from_secs(manifest.permissions.resources.max_execution_time_sec)),
            max_fuel: Some(manifest.permissions.resources.max_fuel),
            max_file_handles: Some(manifest.permissions.resources.max_file_handles),
            ..ResourceLimits::default()
        };
        
        Ok(InstanceConfig {
            capabilities,
            resource_limits,
        })
    }
    
    // Event handler
    async fn handle_event(event: PluginEvent) {
        match event {
            PluginEvent::Loaded(plugin_id) => {
                println!("📦 Event: Plugin {} loaded", plugin_id);
            }
            PluginEvent::Started(plugin_id) => {
                println!("🚀 Event: Plugin {} started", plugin_id);
            }
            PluginEvent::Stopped(plugin_id) => {
                println!("🛑 Event: Plugin {} stopped", plugin_id);
            }
            PluginEvent::Error { plugin_id, error } => {
                eprintln!("❌ Event: Plugin {} error: {}", plugin_id, error);
            }
            PluginEvent::SecurityViolation { plugin_id, violation } => {
                eprintln!("🚨 Event: Plugin {} security violation: {}", plugin_id, violation);
            }
            PluginEvent::ResourceExhausted { plugin_id, resource } => {
                eprintln!("📊 Event: Plugin {} resource exhausted: {}", plugin_id, resource);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🔌 Plugin Ecosystem Example - PUP-style Architecture");
    println!("This demonstrates a complete plugin system with security, hot-reload, and monitoring");
    
    // Create plugin registry
    let mut registry = PluginRegistry::new().await?;
    
    // Create example plugin manifest
    let manifest = PluginManifest {
        id: "text-processor".to_string(),
        name: "Text Processor".to_string(),
        version: "1.0.0".to_string(),
        description: "A secure text processing plugin".to_string(),
        author: "PUP Developer".to_string(),
        permissions: PluginPermissions {
            filesystem: vec![
                FilesystemAccess {
                    path: "./input".into(),
                    access_type: "read".to_string(),
                    recursive: true,
                },
                FilesystemAccess {
                    path: "./output".into(),
                    access_type: "write".to_string(),
                    recursive: false,
                },
            ],
            network: NetworkPolicy {
                allowed_domains: vec!["api.example.com".to_string()],
                allowed_ports: vec![443],
                max_connections: 5,
                enable_https: true,
            },
            system: SystemPolicy {
                env_var_access: vec!["HOME".to_string()],
                process_spawn: false,
                max_threads: 2,
            },
            resources: ResourcePolicy {
                max_memory_mb: 64,
                max_execution_time_sec: 30,
                max_fuel: 10_000_000,
                max_file_handles: 10,
            },
        },
        entry_points: vec![
            EntryPoint {
                name: "process_text".to_string(),
                function: "process_text".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string"},
                        "options": {"type": "object"}
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "processed_text": {"type": "string"},
                        "word_count": {"type": "number"}
                    }
                }),
            }
        ],
        dependencies: vec![],
        metadata: serde_json::json!({
            "category": "text-processing",
            "tags": ["text", "nlp", "processing"]
        }),
    };
    
    // Create example plugin WASM
    let plugin_wasm = create_example_plugin().await?;
    
    // Validate plugin
    let security_report = registry.validate_plugin(&plugin_wasm).await?;
    println!("🔍 Security Report: {:?}", security_report);
    
    // Benchmark plugin
    let perf_report = registry.benchmark_plugin(&plugin_wasm).await?;
    println!("📊 Performance Report: {:?}", perf_report);
    
    // Load plugin
    registry.load_plugin(&plugin_wasm, manifest).await?;
    
    // Start plugin
    registry.start_plugin("text-processor").await?;
    
    // Execute plugin function
    let input_data = serde_json::json!({
        "text": "Hello, WebAssembly Plugin World!",
        "options": {
            "uppercase": true,
            "word_count": true
        }
    });
    
    let result: serde_json::Value = registry.execute_plugin(
        "text-processor",
        "process_text",
        &input_data,
    ).await?;
    
    println!("🎯 Plugin execution result: {}", serde_json::to_string_pretty(&result)?);
    
    // Get plugin statistics
    let stats = registry.get_plugin_stats("text-processor").await?;
    println!("📈 Plugin Statistics: {:?}", stats);
    
    // Demonstrate hot reload
    let updated_plugin_wasm = create_updated_plugin().await?;
    registry.hot_reload_plugin("text-processor", &updated_plugin_wasm).await?;
    
    // Test updated plugin
    let result2: serde_json::Value = registry.execute_plugin(
        "text-processor",
        "process_text",
        &input_data,
    ).await?;
    
    println!("🔄 Hot-reloaded plugin result: {}", serde_json::to_string_pretty(&result2)?);
    
    // List all plugins
    let plugins = registry.list_plugins().await;
    println!("📋 Active plugins: {:?}", plugins);
    
    // Stop plugin
    registry.stop_plugin("text-processor").await?;
    
    println!("✅ Plugin ecosystem demo completed");
    
    Ok(())
}

// Create example plugin WASM module
async fn create_example_plugin() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let wasm_source = r#"
        (module
            (memory (export "memory") 1)
            
            ;; Initialize function
            (func (export "initialize") (param i32) (result i32)
                i32.const 1
            )
            
            ;; Process text function
            (func (export "process_text") (param i32) (result i32)
                i32.const 42
            )
            
            ;; Benchmark function
            (func (export "benchmark_function") (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.mul
            )
        )
    "#;
    
    Ok(wat::parse_str(wasm_source)?)
}

// Create updated plugin for hot reload demo
async fn create_updated_plugin() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let wasm_source = r#"
        (module
            (memory (export "memory") 1)
            
            ;; Initialize function (updated)
            (func (export "initialize") (param i32) (result i32)
                i32.const 1
            )
            
            ;; Process text function (improved)
            (func (export "process_text") (param i32) (result i32)
                i32.const 84  ;; Different result to show hot reload
            )
            
            ;; Benchmark function
            (func (export "benchmark_function") (param i32) (result i32)
                local.get 0
                i32.const 3  ;; Improved algorithm
                i32.mul
            )
        )
    "#;
    
    Ok(wat::parse_str(wasm_source)?)
}
