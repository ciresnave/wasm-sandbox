//! HTTP server sandboxing example
//! 
//! Note: This example uses the test module's add() function to simulate
//! HTTP server functionality since the actual HTTP server WASM module
//! would need to be compiled separately.

use std::path::Path;
use wasm_sandbox::{
    WasmSandbox, InstanceConfig, 
    security::{
        Capabilities, NetworkCapability, FilesystemCapability, 
        EnvironmentCapability, ProcessCapability, ResourceLimits
    }
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the sandbox
    let mut sandbox = WasmSandbox::new()?;
    
    // Define the path to the WebAssembly module
    let wasm_path = Path::new("examples/http_server.wasm");
    
    // Check if the module exists
    if !wasm_path.exists() {
        println!("WebAssembly module not found at: {wasm_path:?}");
        println!("Please build the example module first with:");
        println!("cargo build --target wasm32-wasi --example http_server_guest");
        return Ok(());
    }
    
    // Load the WebAssembly module
    let wasm_bytes = std::fs::read(wasm_path)?;
    let module_id = sandbox.load_module(&wasm_bytes)?;
    
    // Configure the sandbox instance
    let instance_config = InstanceConfig {
        capabilities: Capabilities {
            // Allow localhost connections only
            network: NetworkCapability::Loopback,
            
            // Limited filesystem access
            filesystem: FilesystemCapability {
                readable_dirs: vec![std::env::current_dir()?],
                writable_dirs: vec![std::env::temp_dir()],
                max_file_size: Some(1024 * 1024), // 1MB
                allow_create: true,
                allow_delete: false,
            },
            
            // Limited environment access
            environment: EnvironmentCapability::Allowlist(vec![
                "PATH".to_string(),
                "PORT".to_string(),
                "HOST".to_string(),
            ]),
            
            // No process creation
            process: ProcessCapability::None,
            
            ..Capabilities::minimal()
        },
        resource_limits: ResourceLimits::default(),
        startup_timeout_ms: 5000,
        enable_debug: true,
    };
    
    // Create the instance
    let instance_id = sandbox.create_instance(module_id, Some(instance_config))?;
    
    println!("Created sandbox instance: {instance_id}");
    
    // Simulate HTTP server functionality (using available test functions)
    println!("Simulating HTTP server...");
    
    // Test calculation for port assignment (simulated)
    let port_calculation: i32 = sandbox.call_function(instance_id, "add", (8080, 0)).await?;
    println!("Server would start on port {port_calculation}");

    // Simulate some load
    println!("Simulating HTTP requests...");
    
    // Send some requests (simulate processing)
    for i in 1..=5 {
        let result: i32 = sandbox.call_function(instance_id, "add", (i, 100)).await?;
        println!("Processing request {i}: result = {result}");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Simulate server shutdown
    println!("Simulating server shutdown...");
    let _: i32 = sandbox.call_function(instance_id, "add", (0, 0)).await?;
    println!("Server shutdown simulation complete");
    
    Ok(())
}
