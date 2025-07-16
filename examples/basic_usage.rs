//! Basic Usage Example - Demonstrates core wasm-sandbox functionality
//!
//! This example shows how to use the current API to create sandboxes,
//! load WASM modules, and execute functions with security controls.

use std::time::Duration;
use wasm_sandbox::{WasmSandbox, Result};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Basic wasm-sandbox Usage Example");
    println!("This demonstrates core functionality with the current API");
    
    // Example 1: Simple execution using high-level API
    println!("\n=== Example 1: Simple Execution ===");
    
    // Use the test WASM module included in fixtures
    let wasm_path = "fixtures/test_module.wasm";
    
    // One-line execution (uses defaults)
    match wasm_sandbox::run(wasm_path, "add", &(5i32, 3i32)).await {
        Ok(result) => {
            let result: i32 = result;
            println!("✅ 5 + 3 = {result}");
        }
        Err(e) => println!("❌ Execution failed: {e}"),
    }
    
    // Example 2: Builder pattern with custom settings
    println!("\n=== Example 2: Builder Pattern ===");
    
    let sandbox = WasmSandbox::builder()
        .source(wasm_path)
        .timeout_duration(Duration::from_secs(10))
        .memory_limit(32 * 1024 * 1024) // 32MB
        .enable_file_access(false)
        .enable_network(false)
        .build()
        .await?;
    
    // Call multiple functions
    let add_result: i32 = sandbox.call("add", &(10i32, 20i32)).await?;
    println!("✅ 10 + 20 = {add_result}");
    
    // Try to call a function that doesn't exist to demonstrate error handling
    match sandbox.call::<_, i32>("subtract", &(20i32, 5i32)).await {
        Ok(result) => println!("✅ 20 - 5 = {result}"),
        Err(e) => println!("⚠️  Expected error for non-existent function: {e}"),
    }
    
    // Example 3: Error handling
    println!("\n=== Example 3: Error Handling ===");
    
    // Try to call a non-existent function
    match sandbox.call::<_, i32>("nonexistent_function", &42i32).await {
        Ok(_) => println!("This shouldn't happen"),
        Err(e) => println!("✅ Expected error: {e}"),
    }
    
    // Try with very short timeout
    match wasm_sandbox::run_with_timeout(
        wasm_path,
        "add",
        &(1i32, 2i32),
        Duration::from_millis(1), // Very short timeout
    ).await {
        Ok(result) => {
            let result: i32 = result;
            println!("✅ Fast execution: 1 + 2 = {result}");
        }
        Err(e) => println!("⏰ Timeout or error (expected): {e}"),
    }
    
    println!("\n🎉 All examples completed successfully!");
    Ok(())
}