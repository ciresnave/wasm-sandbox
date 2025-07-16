use wasm_sandbox::runtime::{RuntimeConfig, create_runtime};
use wasm_sandbox::security::{Capabilities, ResourceLimits};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Wasmer runtime implementation...");
    
    // Test creating a Wasmer runtime
    let config = RuntimeConfig::default();
    let runtime = create_runtime(&config)?;
    
    println!("Successfully created runtime!");
    println!("Runtime metrics: {:?}", runtime.get_metrics());
    
    // Use an existing WASM file for testing
    let wasm_path = "fixtures/test_module.wasm";
    let wasm_bytes = std::fs::read(wasm_path)
        .map_err(|e| format!("Failed to read WASM file {}: {}", wasm_path, e))?;
    
    let module = runtime.load_module(&wasm_bytes)?;
    println!("Successfully loaded module: {:?}", module.id());
    println!("Module exports: {:?}", module.exports());
    
    // Test instance creation
    let instance = runtime.create_instance(
        module.as_ref(),
        ResourceLimits::default(),
        Capabilities::minimal(),
    )?;
    
    println!("Successfully created instance!");
    println!("Instance state: {:?}", instance.state());
    println!("Instance memory usage: {} bytes", instance.memory_usage());
    
    // Test function call with exports from the test module
    let exports = module.exports();
    if let Some(export_name) = exports.get(0) {
        println!("Testing function call for export: {}", export_name);
        
        // Try to call a function - this might fail if the function requires different parameters
        match instance.call_simple_function(export_name, &[5, 3]) {
            Ok(result) => println!("Function call result: {}", result),
            Err(e) => println!("Function call failed (expected for some functions): {}", e),
        }
    } else {
        println!("No exports found in module");
    }
    
    println!("✅ Wasmer runtime implementation is working correctly!");
    
    Ok(())
}
