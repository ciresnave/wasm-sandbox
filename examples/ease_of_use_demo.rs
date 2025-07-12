//! Simple example showing the ease-of-use improvements for wasm-sandbox
//!
//! This example demonstrates the progressive complexity API:
//! 1. One-line execution (dead simple)
//! 2. Builder pattern (more control)
//! 3. Full configuration (maximum flexibility)

use wasm_sandbox::{Result, WasmSandbox};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Example 1: Dead Simple - One line execution
    println!("=== Example 1: One-line execution ===");
    
    // For this demo, we'll use the test WASM file
    // In real usage, this could be "calculator.rs", "processor.py", etc.
    let wasm_path = "fixtures/test_module.wasm";
    
    // Single line - compile and run
    let result: i32 = wasm_sandbox::run(wasm_path, "add", &(5, 3)).await?;
    println!("5 + 3 = {}", result);
    
    // Example 2: With timeout for safety
    println!("\n=== Example 2: With timeout ===");
    
    let result: i32 = wasm_sandbox::run_with_timeout(
        wasm_path,
        "add", 
        &(10, 20),
        Duration::from_secs(5)
    ).await?;
    println!("10 + 20 = {}", result);
    
    // Example 3: Builder pattern for more control
    println!("\n=== Example 3: Builder pattern ===");
    
    let sandbox = WasmSandbox::builder()
        .source(wasm_path)
        .timeout_duration(Duration::from_secs(30))
        .memory_limit(64 * 1024 * 1024) // 64MB
        .enable_file_access(false)      // Secure by default
        .enable_network(false)          // No network access
        .build()
        .await?;
    
    // Multiple calls to the same sandbox
    let results: Vec<i32> = futures::future::try_join_all(vec![
        sandbox.call("add", &(1, 1)),
        sandbox.call("add", &(2, 2)), 
        sandbox.call("add", &(3, 3)),
    ]).await?;
    
    println!("Batch results: {:?}", results);
    
    // Example 4: Convenient from_source method
    println!("\n=== Example 4: Convenient from_source ===");
    
    let sandbox2 = WasmSandbox::from_source(wasm_path).await?;
    
    // Different parameter types
    let result1: i32 = sandbox2.call("add", &(100, 200)).await?;
    let result2: i32 = sandbox2.call("add", &[300, 400]).await?; // Array instead of tuple
    
    println!("Flexible parameters: {} and {}", result1, result2);
    
    // Example 5: Error handling
    println!("\n=== Example 5: Error handling ===");
    
    match wasm_sandbox::run::<(i32, i32), i32>("nonexistent.wasm", "add", &(1, 2)).await {
        Ok(_) => println!("This shouldn't happen"),
        Err(e) => println!("Expected error: {}", e),
    }
    
    println!("\n=== All examples completed successfully! ===");
    
    Ok(())
}
