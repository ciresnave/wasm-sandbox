//! HTTP server sandboxing example

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
        println!("WebAssembly module not found at: {:?}", wasm_path);
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
    
    println!("Created sandbox instance: {}", instance_id);
    
    // Call the start function
    let port: u16 = sandbox.call_function(instance_id, "start", 8080).await?;
    
    println!("HTTP server started on port: {}", port);
    println!("Press Ctrl+C to exit");
    
    // Wait for Ctrl+C
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let shutdown_tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));
    let shutdown_tx_clone = shutdown_tx.clone();
    
    ctrlc::set_handler(move || {
        if let Ok(mut tx_guard) = shutdown_tx_clone.lock() {
            if let Some(tx) = tx_guard.take() {
                let _ = tx.send(());
            }
        }
    })?;
    
    let _ = rx.await;
    
    // Shutdown the server
    let _: () = sandbox.call_function(instance_id, "stop", ()).await?;
    
    println!("HTTP server stopped");
    
    Ok(())
}
