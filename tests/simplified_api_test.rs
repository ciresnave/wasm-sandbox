//! Comprehensive tests for the simplified API

use wasm_sandbox::{Result, WasmSandbox};
use std::time::Duration;
use tempfile::TempDir;
use std::fs;

/// Create a simple Rust calculator for testing
#[allow(dead_code)]
fn create_test_calculator(dir: &TempDir) -> std::path::PathBuf {
    let calc_path = dir.path().join("calculator.rs");
    let calc_code = r#"
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[no_mangle]
pub extern "C" fn divide(a: i32, b: i32) -> i32 {
    if b == 0 { 0 } else { a / b }
}

pub fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}
"#;
    fs::write(&calc_path, calc_code).expect("Failed to write calculator.rs");
    calc_path
}

/// Create a WebAssembly file for testing
fn create_test_wasm(dir: &TempDir) -> std::path::PathBuf {
    let wasm_path = dir.path().join("test.wasm");
    // This is a minimal WebAssembly module that exports an add function
    let wasm_bytes = include_bytes!("../fixtures/test_module.wasm");
    fs::write(&wasm_path, wasm_bytes).expect("Failed to write test.wasm");
    wasm_path
}

#[tokio::test]
async fn test_basic_wasm_execution() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    // Test the simplest API - one line execution
    let result: i32 = wasm_sandbox::run(
        wasm_path.to_str().unwrap(),
        "add",
        &(5, 3)
    ).await?;
    
    assert_eq!(result, 8);
    Ok(())
}

#[tokio::test]
async fn test_timeout_functionality() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    // Test execution with timeout
    let result: i32 = wasm_sandbox::run_with_timeout(
        wasm_path.to_str().unwrap(),
        "add",
        &(10, 20),
        Duration::from_secs(5)
    ).await?;
    
    assert_eq!(result, 30);
    Ok(())
}

#[tokio::test]
async fn test_sandbox_builder_pattern() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    // Test builder pattern for more control
    let sandbox = WasmSandbox::builder()
        .source(wasm_path.to_str().unwrap())
        .timeout_duration(Duration::from_secs(10))
        .memory_limit(64 * 1024 * 1024) // 64MB
        .enable_file_access(false)
        .enable_network(false)
        .build()
        .await?;
    
    let result: i32 = sandbox.call("add", &(7, 8)).await?;
    assert_eq!(result, 15);
    Ok(())
}

#[tokio::test]
async fn test_from_source_convenience() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    // Test convenient from_source method
    let sandbox = WasmSandbox::from_source(wasm_path.to_str().unwrap()).await?;
    let result: i32 = sandbox.call("add", &(100, 200)).await?;
    assert_eq!(result, 300);
    Ok(())
}

#[tokio::test]
async fn test_multiple_function_calls() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    let sandbox = WasmSandbox::from_source(wasm_path.to_str().unwrap()).await?;
    
    // Test multiple calls to the same sandbox
    let result1: i32 = sandbox.call("add", &(1, 2)).await?;
    let result2: i32 = sandbox.call("add", &(3, 4)).await?;
    let result3: i32 = sandbox.call("add", &(5, 6)).await?;
    
    assert_eq!(result1, 3);
    assert_eq!(result2, 7);
    assert_eq!(result3, 11);
    Ok(())
}

#[tokio::test]
async fn test_error_handling_invalid_source() {
    let result = wasm_sandbox::run::<(i32, i32), i32>(
        "nonexistent.wasm",
        "add",
        &(1, 2)
    ).await;
    
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    // Check for various possible error messages
    assert!(
        error_msg.contains("not found") || 
        error_msg.contains("No such file") || 
        error_msg.contains("Source file not found") ||
        error_msg.contains("Failed to read") ||
        error_msg.contains("system cannot find"),
        "Unexpected error message: {error_msg}"
    );
}

#[tokio::test]
async fn test_error_handling_invalid_function() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    let sandbox = WasmSandbox::from_source(wasm_path.to_str().unwrap()).await?;
    let result = sandbox.call::<(i32, i32), i32>("nonexistent_function", &(1, 2)).await;
    
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_security_isolation() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    // Test that sandboxes are isolated from each other
    let sandbox1 = WasmSandbox::from_source(wasm_path.to_str().unwrap()).await?;
    let sandbox2 = WasmSandbox::from_source(wasm_path.to_str().unwrap()).await?;
    
    let result1: i32 = sandbox1.call("add", &(10, 20)).await?;
    let result2: i32 = sandbox2.call("add", &(30, 40)).await?;
    
    assert_eq!(result1, 30);
    assert_eq!(result2, 70);
    Ok(())
}

#[tokio::test]
async fn test_memory_limits() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    // Test with very low memory limit
    let sandbox = WasmSandbox::builder()
        .source(wasm_path.to_str().unwrap())
        .memory_limit(1024) // Very low - 1KB
        .build()
        .await?;
    
    // Should still work for simple operations
    let result: i32 = sandbox.call("add", &(1, 1)).await?;
    assert_eq!(result, 2);
    Ok(())
}

#[tokio::test]
async fn test_capabilities_configuration() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    // Test different capability configurations
    let sandbox_no_files = WasmSandbox::builder()
        .source(wasm_path.to_str().unwrap())
        .enable_file_access(false)
        .enable_network(false)
        .build()
        .await?;
    
    let sandbox_with_files = WasmSandbox::builder()
        .source(wasm_path.to_str().unwrap())
        .enable_file_access(true)
        .enable_network(false)
        .build()
        .await?;
    
    // Both should work for basic math
    let result1: i32 = sandbox_no_files.call("add", &(5, 5)).await?;
    let result2: i32 = sandbox_with_files.call("add", &(10, 10)).await?;
    
    assert_eq!(result1, 10);
    assert_eq!(result2, 20);
    Ok(())
}

#[tokio::test]
async fn test_different_parameter_types() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    let sandbox = WasmSandbox::from_source(wasm_path.to_str().unwrap()).await?;
    
    // Test with different parameter types
    let result_tuple: i32 = sandbox.call("add", &(1, 2)).await?;
    let result_array: i32 = sandbox.call("add", &[3, 4]).await?;
    
    assert_eq!(result_tuple, 3);
    assert_eq!(result_array, 7);
    Ok(())
}

#[tokio::test]
async fn test_edge_cases() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    let sandbox = WasmSandbox::from_source(wasm_path.to_str().unwrap()).await?;
    
    // Test edge cases
    let result_zero: i32 = sandbox.call("add", &(0, 0)).await?;
    let result_negative: i32 = sandbox.call("add", &(-1, 1)).await?;
    let result_large: i32 = sandbox.call("add", &(i32::MAX, 0)).await?;
    
    assert_eq!(result_zero, 0);
    assert_eq!(result_negative, 0);
    assert_eq!(result_large, i32::MAX);
    Ok(())
}

#[tokio::test]
async fn test_concurrent_execution() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    // Test concurrent execution of multiple sandboxes
    let handles: Vec<_> = (0..10).map(|i| {
        let path = wasm_path.to_str().unwrap().to_string();
        tokio::spawn(async move {
            let result: i32 = wasm_sandbox::run(&path, "add", &(i, i)).await?;
            Ok::<i32, wasm_sandbox::Error>(result)
        })
    }).collect();
    
    let results: Result<Vec<_>> = futures::future::try_join_all(handles)
        .await
        .map_err(|e| wasm_sandbox::Error::Generic { message: format!("Join error: {e}") })?
        .into_iter()
        .collect();
    
    let results = results?;
    for (i, &result) in results.iter().enumerate() {
        assert_eq!(result, i as i32 * 2);
    }
    
    Ok(())
}

/// Integration test that covers the full workflow
#[tokio::test]
async fn test_full_workflow_integration() -> Result<()> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wasm_path = create_test_wasm(&temp_dir);
    
    // Step 1: Simple one-liner
    let quick_result: i32 = wasm_sandbox::run(
        wasm_path.to_str().unwrap(),
        "add", 
        &(1, 1)
    ).await?;
    assert_eq!(quick_result, 2);
    
    // Step 2: With timeout
    let timeout_result: i32 = wasm_sandbox::run_with_timeout(
        wasm_path.to_str().unwrap(),
        "add",
        &(2, 2),
        Duration::from_secs(1)
    ).await?;
    assert_eq!(timeout_result, 4);
    
    // Step 3: Builder pattern with full configuration
    let sandbox = WasmSandbox::builder()
        .source(wasm_path.to_str().unwrap())
        .timeout_duration(Duration::from_secs(30))
        .memory_limit(16 * 1024 * 1024) // 16MB
        .enable_file_access(false)
        .enable_network(false)
        .build()
        .await?;
    
    // Step 4: Multiple operations on the same sandbox
    let results: Vec<i32> = futures::future::try_join_all(vec![
        sandbox.call("add", &(10, 10)),
        sandbox.call("add", &(20, 20)),
        sandbox.call("add", &(30, 30)),
    ]).await?;
    
    assert_eq!(results, vec![20, 40, 60]);
    
    Ok(())
}
