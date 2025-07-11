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
        println!("WebAssembly module not found at: {:?}", wasm_path);
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
    
    println!("Created sandbox instance: {}", instance_id);
    
    // Prepare arguments (as JSON array)
    let args = serde_json::to_string(&["--help"])?;
    
    // Run the tool with arguments
    let success: bool = sandbox.call_function(instance_id, "run", args).await?;
    
    if !success {
        println!("Failed to run the CLI tool");
        return Ok(());
    }
    
    // Wait for the tool to finish
    loop {
        let running: bool = sandbox.call_function(instance_id, "is_running", "").await?;
        if !running {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Get the output
    let output: String = sandbox.call_function(instance_id, "get_output_string", &()).await?;
    
    println!("CLI tool output:");
    println!("{}", output);
    
    Ok(())
}
