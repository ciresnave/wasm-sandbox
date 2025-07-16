//! File Processor Example - Demonstrates file processing in a secure sandbox
//!
//! This example shows how to use wasm-sandbox for secure file processing,
//! including directory access controls and resource limits.

use std::time::Duration;
use std::path::PathBuf;
use wasm_sandbox::{WasmSandbox, Result};
use tokio::fs;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FileProcessingTask {
    input_path: String,
    output_path: String,
    operation: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ProcessingResult {
    success: bool,
    files_processed: u32,
    error_message: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🗂️  File Processor Example");
    println!("Demonstrating secure file processing with wasm-sandbox");

    // Setup directories for processing
    let input_dir = PathBuf::from("./test_input");
    let output_dir = PathBuf::from("./test_output");
    
    // Create test directories if they don't exist
    if fs::create_dir_all(&input_dir).await.is_err() {
        println!("Note: Input directory may already exist");
    }
    if fs::create_dir_all(&output_dir).await.is_err() {
        println!("Note: Output directory may already exist");
    }

    // Create some test files
    let test_content = "Hello, this is test content for file processing.";
    let _ = fs::write(input_dir.join("test1.txt"), test_content).await;
    let _ = fs::write(input_dir.join("test2.txt"), format!("{test_content}\nSecond line.")).await;

    println!("✅ Created test files in {input_dir:?}");

    // Example 1: Simple file processing with builder pattern
    println!("\n=== Example 1: Builder Pattern with File Access ===");
    
    let sandbox = WasmSandbox::builder()
        .source("fixtures/test_module.wasm")
        .timeout_duration(Duration::from_secs(30))
        .memory_limit(16 * 1024 * 1024) // 16MB
        .enable_file_access(true)       // Enable file access
        .enable_network(false)          // No network needed
        .build()
        .await?;

    // Process files using sandbox
    let _task = FileProcessingTask {
        input_path: input_dir.join("test1.txt").to_string_lossy().to_string(),
        output_path: output_dir.join("processed1.txt").to_string_lossy().to_string(),
        operation: "uppercase".to_string(),
    };

    // Since we're using the test module which has simple math functions,
    // let's simulate file processing by just doing some computation
    let file_size: i32 = sandbox.call("add", &(test_content.len() as i32, 0)).await?;
    println!("✅ Simulated file processing - input size: {file_size} bytes");

    // Example 2: Batch processing
    println!("\n=== Example 2: Batch Processing ===");
    
    let tasks = vec![
        ("test1.txt", "processed1.txt"),
        ("test2.txt", "processed2.txt"),
    ];

    for (input_file, output_file) in tasks {
        let input_path = input_dir.join(input_file);
        let output_path = output_dir.join(output_file);
        
        // Read file size (simulated)
        if let Ok(content) = fs::read_to_string(&input_path).await {
            let char_count: i32 = sandbox.call("add", &(content.len() as i32, 0)).await?;
            
            // Write processed result (simulated)
            let processed_content = format!("PROCESSED: {}", content.to_uppercase());
            let _ = fs::write(&output_path, processed_content).await;
            
            println!("✅ Processed {input_file} -> {output_file} ({char_count} chars)");
        }
    }

    // Example 3: Error handling and resource limits
    println!("\n=== Example 3: Resource Limits ===");
    
    // Create a sandbox with very strict limits
    let strict_sandbox = WasmSandbox::builder()
        .source("fixtures/test_module.wasm")
        .timeout_duration(Duration::from_millis(100)) // Very short timeout
        .memory_limit(1024 * 1024) // 1MB only
        .enable_file_access(false) // No file access
        .build()
        .await?;

    // This should work fine
    match strict_sandbox.call::<(i32, i32), i32>("add", &(1, 2)).await {
        Ok(result) => println!("✅ Simple calculation: 1 + 2 = {result}"),
        Err(e) => println!("❌ Unexpected error: {e}"),
    }

    // Example 4: Using one-liner for quick processing
    println!("\n=== Example 4: One-liner Processing ===");
    
    // Quick file size calculation
    let large_file_size = 1000000i32; // 1MB file simulation
    let processing_result: i32 = wasm_sandbox::run(
        "fixtures/test_module.wasm",
        "add",
        &(large_file_size, large_file_size) // Add to simulate processing
    ).await?;
    
    println!("✅ Large file processing simulation: {large_file_size} bytes -> {processing_result} bytes processed");

    // Example 5: Cleanup and summary
    println!("\n=== Example 5: Cleanup ===");
    
    // Check output files
    if let Ok(entries) = fs::read_dir(&output_dir).await {
        let mut count = 0;
        let mut entries = entries;
        while let Ok(Some(_entry)) = entries.next_entry().await {
            count += 1;
        }
        println!("✅ Created {count} output files in {output_dir:?}");
    }

    println!("\n🎉 File processing example completed!");
    
    // Note: In a real implementation, you would:
    // 1. Use actual file processing WASM modules
    // 2. Implement proper file access controls
    // 3. Handle different file formats
    // 4. Implement streaming for large files
    // 5. Add proper error recovery
    
    println!("\n💡 This example simulates file processing using test functions.");
    println!("   In production, you would load actual file processing WASM modules.");

    Ok(())
}