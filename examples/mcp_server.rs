//! MCP server sandboxing example

use std::path::Path;
use wasm_sandbox::{
    WasmSandbox, InstanceConfig, 
    security::{Capabilities, ResourceLimits}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the sandbox
    let mut sandbox = WasmSandbox::new()?;
    
    // Load the MCP server module
    let mcp_module_path = Path::new("fixtures/mcp_server.wasm");
    
    if !mcp_module_path.exists() {
        eprintln!("WASM module not found at: {mcp_module_path:?}");
        eprintln!("Please build an MCP server module first using the compiler.");
        return Ok(());
    }
    
    let wasm_bytes = std::fs::read(mcp_module_path)?;
    let module_id = sandbox.load_module(&wasm_bytes)?;
    
    // Configure the instance with minimal capabilities for now
    let instance_config = InstanceConfig {
        capabilities: Capabilities::minimal(),
        resource_limits: ResourceLimits {
            memory: wasm_sandbox::security::MemoryLimits {
                max_memory_pages: 500, // ~32MB
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    
    // Create the instance
    let instance_id = sandbox.create_instance(module_id, Some(instance_config))?;
    
    // Start the MCP server
    println!("Starting MCP server in sandbox...");
    
    // Set up signal handler for graceful shutdown
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C, shutting down...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;
    
    // Simple test call to ensure the module works
    println!("Testing basic function call...");
    let result: i32 = sandbox.call_function(instance_id, "add", &(5, 7)).await?;
    println!("Test call result: {result}");
    
    println!("MCP server example completed successfully");
    
    Ok(())
}
