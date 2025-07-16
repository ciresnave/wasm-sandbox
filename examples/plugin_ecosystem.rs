//! Plugin Ecosystem Example - Demonstrates plugin management and execution
//!
//! This example shows how to build a plugin system using wasm-sandbox,
//! including plugin loading, management, and secure execution.

use std::collections::HashMap;
use std::time::Duration;
use wasm_sandbox::{WasmSandbox, Result};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct Plugin {
    id: String,
    name: String,
    version: String,
    description: String,
    wasm_path: String,
}

struct PluginManager {
    plugins: RwLock<HashMap<String, Plugin>>,
}

impl PluginManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            plugins: RwLock::new(HashMap::new()),
        })
    }
    
    pub async fn register_plugin(&self, plugin: Plugin) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin.id.clone(), plugin);
        Ok(())
    }
    
    pub async fn list_plugins(&self) -> Vec<Plugin> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }
    
    pub async fn execute_plugin(&self, plugin_id: &str, operation: &str, data: i32) -> Result<i32> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(plugin_id)
            .ok_or_else(|| wasm_sandbox::Error::NotFound { 
                resource_type: "plugin".to_string(), 
                identifier: plugin_id.to_string() 
            })?;
        
        // Use the builder pattern to create a sandbox for this plugin
        let plugin_sandbox = WasmSandbox::builder()
            .source(&plugin.wasm_path)
            .timeout_duration(Duration::from_secs(10))
            .memory_limit(32 * 1024 * 1024) // 32MB
            .enable_file_access(false)
            .enable_network(false)
            .build()
            .await?;
        
        // Execute the operation
        plugin_sandbox.call(operation, &(data, 1)).await
    }
    
    pub async fn benchmark_plugin(&self, plugin_id: &str) -> Result<Duration> {
        let start = std::time::Instant::now();
        
        // Run a test operation
        let _result: i32 = self.execute_plugin(plugin_id, "add", 100).await?;
        
        Ok(start.elapsed())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔌 Plugin Ecosystem Example");
    println!("Demonstrating plugin management with wasm-sandbox");
    
    // Example 1: Create plugin manager
    println!("\n=== Example 1: Plugin Manager Setup ===");
    
    let manager = PluginManager::new().await?;
    println!("✅ Plugin manager created");
    
    // Example 2: Register plugins
    println!("\n=== Example 2: Plugin Registration ===");
    
    let math_plugin = Plugin {
        id: "math-basic".to_string(),
        name: "Basic Math Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Provides basic mathematical operations".to_string(),
        wasm_path: "fixtures/test_module.wasm".to_string(),
    };
    
    let calculator_plugin = Plugin {
        id: "calculator".to_string(),
        name: "Advanced Calculator".to_string(),
        version: "2.1.0".to_string(),
        description: "Advanced calculation functions".to_string(),
        wasm_path: "fixtures/test_module.wasm".to_string(),
    };
    
    manager.register_plugin(math_plugin).await?;
    manager.register_plugin(calculator_plugin).await?;
    
    println!("✅ Registered 2 plugins");
    
    // Example 3: List available plugins
    println!("\n=== Example 3: Plugin Discovery ===");
    
    let plugins = manager.list_plugins().await;
    for plugin in &plugins {
        println!("📦 {}: {} v{} - {}", 
            plugin.id, plugin.name, plugin.version, plugin.description);
    }
    
    // Example 4: Execute plugin operations
    println!("\n=== Example 4: Plugin Execution ===");
    
    // Test math plugin
    let result = manager.execute_plugin("math-basic", "add", 15).await?;
    println!("✅ Math plugin add(15, 1) = {result}");
    
    let result = manager.execute_plugin("math-basic", "add", 7).await?;
    println!("✅ Math plugin add(7, 1) = {result}");
    
    // Test calculator plugin
    let result = manager.execute_plugin("calculator", "add", 25).await?;
    println!("✅ Calculator plugin add(25, 1) = {result}");
    
    // Example 5: Plugin benchmarking
    println!("\n=== Example 5: Plugin Benchmarking ===");
    
    for plugin in &plugins {
        match manager.benchmark_plugin(&plugin.id).await {
            Ok(duration) => {
                println!("⏱️  {} execution time: {:?}", plugin.name, duration);
            }
            Err(e) => {
                println!("❌ Failed to benchmark {}: {}", plugin.name, e);
            }
        }
    }
    
    // Example 6: Error handling with invalid plugin
    println!("\n=== Example 6: Error Handling ===");
    
    match manager.execute_plugin("nonexistent-plugin", "add", 10).await {
        Ok(_) => println!("This shouldn't happen"),
        Err(e) => println!("✅ Expected error for invalid plugin: {e}"),
    }
    
    // Example 7: Concurrent plugin execution
    println!("\n=== Example 7: Concurrent Execution ===");
    
    let futures = vec![
        manager.execute_plugin("math-basic", "add", 10),
        manager.execute_plugin("calculator", "add", 5),
        manager.execute_plugin("math-basic", "add", 20),
    ];
    
    match futures::future::try_join_all(futures).await {
        Ok(results) => {
            println!("✅ Concurrent execution results: {results:?}");
        }
        Err(e) => {
            println!("❌ Concurrent execution failed: {e}");
        }
    }
    
    // Example 8: Plugin isolation demonstration
    println!("\n=== Example 8: Plugin Isolation ===");
    
    // Each plugin execution is isolated - they can't interfere with each other
    let isolation_tests = vec![
        ("math-basic", "add", 100),
        ("calculator", "add", 200),
        ("math-basic", "add", 3),
    ];
    
    for (plugin_id, operation, value) in isolation_tests {
        let result = manager.execute_plugin(plugin_id, operation, value).await?;
        println!("🔒 Isolated execution: {plugin_id} {operation} {value} = {result}");
    }
    
    println!("\n🎉 Plugin ecosystem example completed!");
    
    // Summary
    println!("\n📋 Summary:");
    println!("   • Registered {} plugins", plugins.len());
    println!("   • Demonstrated secure plugin execution");
    println!("   • Showed plugin isolation and concurrency");
    println!("   • Performed benchmarking and error handling");
    
    Ok(())
}