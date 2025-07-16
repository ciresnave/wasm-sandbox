//! Example demonstrating the enhanced API features from PUP feedback
//! 
//! This example shows:
//! - Builder pattern configuration with human-readable units
//! - Enhanced error handling with detailed context
//! - Resource monitoring with detailed usage tracking
//! - Streaming execution for batch operations
//! - Plugin system integration

use wasm_sandbox::{
    MemoryUnit, InstanceConfig, SandboxError, ResourceKind, SecurityContext,
    StreamingConfig, StreamingExecutor, StreamingExecution,
    FunctionCall, PluginManifest, AdvancedCapabilities, NetworkPolicy, FilesystemPolicy,
    plugins::ExecutionCategory
};
use serde_json::Value;
use std::time::Duration;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Enhanced API Demo - PUP Feedback Implementation\n");

    // Demonstrate enhanced configuration with builder pattern
    demonstrate_builder_pattern().await?;
    
    // Demonstrate enhanced error handling
    demonstrate_enhanced_errors().await?;
    
    // Demonstrate resource monitoring
    demonstrate_resource_monitoring().await?;
    
    // Demonstrate streaming execution
    demonstrate_streaming_execution().await?;
    
    // Demonstrate plugin system
    demonstrate_plugin_system().await?;

    println!("\n✅ All enhanced API features demonstrated successfully!");
    Ok(())
}

/// Demonstrate the new builder pattern with human-readable units
async fn demonstrate_builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Builder Pattern Configuration:");
    
    // Create configuration using the new builder pattern
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("input").to_string_lossy().to_string();
    let output_path = temp_dir.join("output").to_string_lossy().to_string();
    let config_path = temp_dir.join("config").to_string_lossy().to_string();
    
    // Create the directories to ensure they exist
    std::fs::create_dir_all(&input_path).ok();
    std::fs::create_dir_all(&output_path).ok();
    std::fs::create_dir_all(&config_path).ok();
    
    let _config = InstanceConfig::builder()
        .memory_limit(MemoryUnit::mb(64))                           // 64 megabytes
        .timeout(30)                                      // 30 seconds
        .cpu_time_limit(5)                     // 5 seconds CPU time
        .filesystem_read(&[&input_path, &config_path])     // Read-only paths
        .filesystem_write(&[&output_path])              // Write-only paths
        .network_deny_all()                              // No network access
        .enable_debug()                                  // Enable debugging
        .build()?;
    
    println!("  ✓ Memory limit: 64 MB");
    println!("  ✓ Timeout: 30 seconds");
    println!("  ✓ CPU time limit: 5 seconds");
    println!("  ✓ Filesystem: read from {input_path}, {config_path}; write to {output_path}");
    println!("  ✓ Network: denied");
    println!("  ✓ Debug: enabled");
    
    // Advanced capabilities with granular control
    let data_path = temp_dir.join("data").to_string_lossy().to_string();
    let output2_path = temp_dir.join("output2").to_string_lossy().to_string();
    
    // Create the directories to ensure they exist
    std::fs::create_dir_all(&data_path).ok();
    std::fs::create_dir_all(&output2_path).ok();
    
    let _advanced_caps = AdvancedCapabilities {
        network: NetworkPolicy {
            allowed_domains: vec!["api.example.com".to_string()],
            max_connections: 5,
            allowed_ports: vec![80, 443],
            deny_all: false,
            loopback_only: false,
        },
        filesystem: FilesystemPolicy {
            read_paths: vec![data_path.into(), config_path.into()],
            write_paths: vec![output2_path.into()],
            temp_dir_access: true,
            max_file_size: MemoryUnit::mb(10) as usize,
            deny_all: false,
        },
        env_vars: vec!["HOME".to_string(), "PATH".to_string()],
        process_spawn: false,
        max_threads: 4,
    };
    
    println!("  ✓ Advanced capabilities configured with granular controls\n");
    Ok(())
}

/// Demonstrate enhanced error handling with detailed context
async fn demonstrate_enhanced_errors() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Enhanced Error Handling:");
    
    // Simulate different types of errors with detailed context
    
    // Configuration error with suggestion
    let config_error = SandboxError::config_error(
        "Memory limit cannot be zero",
        Some("Use .memory_limit() to set a valid memory limit (e.g., 64.mb())".to_string())
    );
    println!("  ✓ Configuration error: {config_error}");
    
    // Resource exhausted error with usage details
    let resource_error = SandboxError::resource_exhausted(
        ResourceKind::Memory,
        67108864, // 64MB used
        33554432, // 32MB limit
        Some("Consider increasing memory limit or optimizing WASM module".to_string())
    );
    println!("  ✓ Resource error: {resource_error}");
    
    // Security violation with context
    let security_context = SecurityContext {
        attempted_operation: "write to /etc/passwd".to_string(),
        required_capability: "filesystem.write.system".to_string(),
        available_capabilities: vec!["filesystem.write.temp".to_string()],
    };
    let security_error = SandboxError::security_violation(
        "Attempted to write to protected system file",
        security_context
    );
    println!("  ✓ Security error: {security_error}");
    
    println!();
    Ok(())
}

/// Demonstrate detailed resource monitoring
async fn demonstrate_resource_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Resource Monitoring:");
    
    // Create a resource monitor
    let mut monitor = wasm_sandbox::ResourceMonitor::new(None);
    
    // Simulate resource usage
    monitor.update_memory(1024 * 1024); // 1MB
    monitor.record_function_call();
    monitor.record_file_read(2048);
    monitor.update_cpu_time(Duration::from_millis(150));
    monitor.take_snapshot();
    
    // Get detailed usage
    let usage = monitor.get_detailed_usage();
    println!("  ✓ Memory usage: {} bytes (peak: {} bytes)", 
             usage.memory.current_bytes, usage.memory.peak_bytes);
    println!("  ✓ CPU time: {:?}", usage.cpu.time_spent);
    println!("  ✓ Function calls: {}", usage.cpu.function_calls);
    println!("  ✓ Bytes read: {}", usage.io.bytes_read);
    println!("  ✓ Timeline snapshots: {}", usage.timeline.len());
    
    // Check resource limits
    if let Some((_used, message)) = monitor.check_resource_limit(&ResourceKind::Memory, 512 * 1024) {
        println!("  ⚠️  Resource limit check: {message}");
    }
    
    // Get utilization percentage
    let utilization = monitor.get_utilization(&ResourceKind::Memory, 2 * 1024 * 1024);
    println!("  ✓ Memory utilization: {utilization:.1}%");
    
    println!();
    Ok(())
}

/// Demonstrate streaming execution capabilities
async fn demonstrate_streaming_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 Streaming Execution:");
    
    // Create streaming configuration
    let streaming_config = StreamingConfig {
        max_concurrency: 5,
        buffer_size: 100,
        operation_timeout: Duration::from_secs(10),
        fail_fast: false,
        monitor_resources: true,
    };
    
    println!("  ✓ Streaming config: {} max concurrency, {} buffer size", 
             streaming_config.max_concurrency, streaming_config.buffer_size);
    
    // Create streaming executor
    let instance_id = Uuid::new_v4();
    let executor = StreamingExecutor::new(instance_id, streaming_config);
    
    // Create batch of function calls
    let function_calls = vec![
        FunctionCall {
            function_name: "process_data".to_string(),
            parameters: vec![Value::String("input1".to_string())],
            timeout: Some(Duration::from_secs(5)),
        },
        FunctionCall {
            function_name: "process_data".to_string(),
            parameters: vec![Value::String("input2".to_string())],
            timeout: Some(Duration::from_secs(5)),
        },
        FunctionCall {
            function_name: "process_data".to_string(),
            parameters: vec![Value::String("input3".to_string())],
            timeout: Some(Duration::from_secs(5)),
        },
    ];
    
    // Execute batch
    let results = executor.execute_batch(function_calls).await;
    println!("  ✓ Executed {} function calls in batch", results.len());
    
    for (i, result) in results.iter().enumerate() {
        match &result.result {
            Ok(value) => println!("    Result {}: {} (took {:?})", 
                                i + 1, value, result.execution_time),
            Err(e) => println!("    Result {}: Error - {}", i + 1, e),
        }
    }
    
    println!();
    Ok(())
}

/// Demonstrate plugin system capabilities
async fn demonstrate_plugin_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Plugin System:");
    
    // Create a plugin manifest
    let manifest = PluginManifest {
        id: "data-processor".to_string(),
        name: "Data Processor Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Processes data with various transformations".to_string(),
        permissions: AdvancedCapabilities {
            filesystem: FilesystemPolicy {
                read_paths: vec!["/data".into()],
                write_paths: vec!["/output".into()],
                temp_dir_access: true,
                max_file_size: MemoryUnit::mb(10) as usize,
                deny_all: false,
            },
            network: NetworkPolicy {
                deny_all: true,
                ..Default::default()
            },
            ..Default::default()
        },
        entry_points: vec![
            wasm_sandbox::EntryPoint {
                function_name: "transform_data".to_string(),
                display_name: "Transform Data".to_string(),
                description: "Transforms input data using specified rules".to_string(),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "data": {"type": "string"},
                        "rules": {"type": "array"}
                    }
                })),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "string"},
                        "metadata": {"type": "object"}
                    }
                })),
                supports_streaming: true,
                execution_category: ExecutionCategory::Standard,
            }
        ],
        dependencies: vec![],
        metadata: std::collections::HashMap::new(),
        min_sandbox_version: "0.4.0".to_string(),
        author: Some("Example Developer".to_string()),
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/example/data-processor".to_string()),
    };
    
    println!("  ✓ Plugin manifest created:");
    println!("    ID: {}", manifest.id);
    println!("    Name: {}", manifest.name);
    println!("    Version: {}", manifest.version);
    println!("    Entry points: {}", manifest.entry_points.len());
    println!("    Required permissions: filesystem read/write");
    
    // Demonstrate plugin validation
    println!("  ✓ Plugin validation would check:");
    println!("    - Manifest schema compliance");
    println!("    - WASM bytecode security");
    println!("    - Permission requirements");
    println!("    - Performance characteristics");
    
    // Demonstrate hot reload compatibility
    println!("  ✓ Hot reload system would check:");
    println!("    - API compatibility");
    println!("    - Breaking changes");
    println!("    - Resource requirements");
    println!("    - Migration safety");
    
    println!();
    Ok(())
}


