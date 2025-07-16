//! Example demonstrating streaming APIs for large data handling
//!
//! This example shows how to use the streaming APIs in wasm-sandbox
//! to process large datasets that don't fit entirely in memory.

use std::time::Duration;
use wasm_sandbox::{
    WasmSandbox, Result,
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<()> {
    println!("📊 Streaming API Example");
    println!("This demonstrates processing large data with streaming in wasm-sandbox");
    println!();
    
    // Example 1: Simple demonstration of streaming concepts
    println!("=== Example 1: Streaming Simulation ===");
    
    // Create a basic sandbox to demonstrate the concept
    let mut sandbox = WasmSandbox::new()?;
    
    // Load our test module
    let module_bytes = tokio::fs::read("fixtures/test_module.wasm").await?;
    let module_id = sandbox.load_module(&module_bytes)?;
    let instance_id = sandbox.create_instance(module_id, None)?;
    
    println!("✅ Sandbox created and module loaded");
    
    // Simulate processing streaming data chunks
    println!("\nSimulating streaming data processing...");
    
    let mut total_processed = 0;
    for i in 1..=10 {
        // Simulate processing a chunk of data
        let chunk_size = i * 100; // Varying chunk sizes
        let result: i32 = sandbox.call_function(instance_id, "add", (chunk_size, total_processed)).await?;
        
        total_processed = result;
        println!("  Processed chunk {i}: {chunk_size} bytes (total: {total_processed})");
        
        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    println!("✅ Streaming processing simulation complete!");
    println!("Total data processed: {total_processed} bytes");
    
    // Example 2: File-based streaming simulation
    println!("\n=== Example 2: File-Based Streaming Simulation ===");
    
    // Create temporary files for demonstration
    let input_path = std::env::temp_dir().join("wasm-sandbox-stream-input.dat");
    let output_path = std::env::temp_dir().join("wasm-sandbox-stream-output.dat");
    
    // Generate a test file
    println!("Generating test file: {input_path:?}");
    let mut input_file = File::create(&input_path).await?;
    let test_data = b"This is streaming test data that would be processed in chunks by WebAssembly.\n";
    
    // Write test data multiple times to simulate a larger file
    for i in 0..100 {
        input_file.write_all(test_data).await?;
        if i % 20 == 0 {
            print!("\rWritten: {} lines", i + 1);
        }
    }
    println!("\n✅ Test file created: {} bytes", test_data.len() * 100);
    
    // Simulate file processing
    println!("\nSimulating file processing...");
    let mut output_file = File::create(&output_path).await?;
    
    // Process file in simulated chunks
    let file_data = tokio::fs::read(&input_path).await?;
    let chunk_size = 256;
    let mut chunks_processed = 0;
    
    for (i, chunk) in file_data.chunks(chunk_size).enumerate() {
        // Simulate processing each chunk
        let processed_size = chunk.len() as i32;
        let _result: i32 = sandbox.call_function(instance_id, "add", (processed_size, 0)).await?;
        
        // Write processed chunk (in this case, just the original data)
        output_file.write_all(chunk).await?;
        chunks_processed += 1;
        
        if i % 10 == 0 {
            print!("\rProcessed: {chunks_processed} chunks");
        }
        
        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    println!("\n✅ File processing complete!");
    println!("Processed {chunks_processed} chunks");
    
    // Clean up temporary files
    let _ = tokio::fs::remove_file(&input_path).await;
    let _ = tokio::fs::remove_file(&output_path).await;
    println!("Temporary files cleaned up");
    
    println!("\n🎉 All streaming examples completed successfully!");
    Ok(())
}
