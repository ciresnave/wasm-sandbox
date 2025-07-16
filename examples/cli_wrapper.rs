//! CLI tool sandboxing example

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
    let wasm_path = Path::new("examples/cli_tool.wasm");
    
    // Check if the module exists
    if !wasm_path.exists() {
        println!("WebAssembly module not found at: {wasm_path:?}");
        println!("Please build the example module first with:");
        println!("cargo build --target wasm32-wasi --example cli_tool_guest");
        return Ok(());
    }
    
    // Load the WebAssembly module
    let wasm_bytes = std::fs::read(wasm_path)?;
    let module_id = sandbox.load_module(&wasm_bytes)?;
    
    // Configure the sandbox instance with safe defaults for CLI tools
    let instance_config = InstanceConfig {
        capabilities: Capabilities {
            // No network access
            network: NetworkCapability::None,
            
            // Limited filesystem access
            filesystem: FilesystemCapability {
                readable_dirs: vec![std::env::current_dir()?],
                writable_dirs: vec![std::env::temp_dir()],
                allow_create: true,
                allow_delete: false,
                max_file_size: Some(1024 * 1024), // 1MB
            },
            
            // No environment access
            environment: EnvironmentCapability::None,
            
            // No process creation
            process: ProcessCapability::None,
            
            ..Capabilities::minimal()
        },
        resource_limits: ResourceLimits {
            memory: wasm_sandbox::security::MemoryLimits {
                max_memory_pages: 500, // ~32MB
                ..Default::default()
            },
            cpu: wasm_sandbox::security::CpuLimits {
                max_execution_time_ms: 5000, // 5 seconds
                ..Default::default()
            },
            ..ResourceLimits::default()
        },
        startup_timeout_ms: 3000,
        enable_debug: true,
    };
    
    // Create the instance
    let instance_id = sandbox.create_instance(module_id, Some(instance_config))?;
    
    println!("Created sandbox instance: {instance_id}");
    
    // Since we're using the test module, let's test the available functions
    println!("Testing available functions in the WebAssembly module...");
    
    // Test the add function (which exists in test_module.wasm)
    let result: i32 = sandbox.call_function(instance_id, "add", &(10i32, 20i32)).await?;
    println!("✅ add(10, 20) = {result}");
    
    // Test another add operation
    let result2: i32 = sandbox.call_function(instance_id, "add", &(5i32, 3i32)).await?;
    println!("✅ add(5, 3) = {result2}");
    
    // Try to call a non-existent function to demonstrate error handling
    match sandbox.call_function::<_, i32>(instance_id, "nonexistent_function", &42i32).await {
        Ok(_) => println!("This shouldn't happen"),
        Err(e) => println!("✅ Expected error for non-existent function: {e}"),
    }
    
    println!("✅ CLI wrapper example completed successfully!");
    println!("Note: This example uses the test module which only provides basic math functions.");
    println!("To use a real CLI tool, compile a proper WASM module with run(), is_running(), and get_output_string() functions.");
    
    Ok(())
}
