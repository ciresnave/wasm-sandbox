// File Processor Example - Real-world sandboxing scenario
// 
// This example demonstrates how to safely sandbox a file processing tool,
// addressing the specific use case mentioned in the PUP integration feedback.

use wasm_sandbox::{WasmSandbox, InstanceConfig, InstanceId};
use wasm_sandbox::security::{Capabilities, ResourceLimits, FilesystemCapability};
use std::time::Duration;
use std::path::PathBuf;
use tokio::fs;

#[derive(serde::Serialize, serde::Deserialize)]
struct ProcessingConfig {
    input_format: String,
    output_format: String,
    max_file_size: usize,
    processing_options: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ProcessingResult {
    files_processed: usize,
    bytes_processed: usize,
    processing_time_ms: u64,
    errors: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🔧 File Processor Sandbox Example");
    println!("This demonstrates secure file processing with fine-grained permissions");
    
    // Step 1: Setup directory structure
    let input_dir = PathBuf::from("./data/input");
    let output_dir = PathBuf::from("./data/output");
    let temp_dir = PathBuf::from("./data/temp");
    
    // Create directories if they don't exist
    fs::create_dir_all(&input_dir).await?;
    fs::create_dir_all(&output_dir).await?;
    fs::create_dir_all(&temp_dir).await?;
    
    // Create some test files
    fs::write(input_dir.join("test1.txt"), "Hello, World!").await?;
    fs::write(input_dir.join("test2.txt"), "WebAssembly is secure!").await?;
    fs::write(input_dir.join("data.json"), r#"{"key": "value", "number": 42}"#).await?;
    
    println!("📁 Created test files in: {}", input_dir.display());
    
    // Step 2: Load the file processor WASM module
    // In a real scenario, this would be your compiled file processing tool
    let wasm_bytes = compile_file_processor().await?;
    
    // Step 3: Configure the sandbox with strict security
    let capabilities = Capabilities {
        // Filesystem access - read-only input, write-only output
        filesystem: vec![
            FilesystemCapability::ReadOnly(input_dir.clone()),
            FilesystemCapability::WriteOnly(output_dir.clone()),
            FilesystemCapability::ReadWrite(temp_dir.clone()), // For temporary files
        ],
        // No network access for security
        network: vec![],
        // No environment variable access
        environment: vec![],
        // Basic system capabilities only
        system: vec![],
        // No subprocess spawning
        process: vec![],
        // Basic time access
        time: vec![],
        // No cryptographic capabilities  
        random: vec![],
    };
    
    let resource_limits = ResourceLimits::builder()
        .memory_pages(1024) // 64MB memory limit (64 pages * 64KB each)
        .cpu_time(Duration::from_secs(30))
        .fuel(10_000_000) // Limit computational work
        .build();
    
    let instance_config = InstanceConfig {
        capabilities,
        resource_limits,
        startup_timeout_ms: 5000,
        enable_debug: false,
    };
    
    // Step 4: Create sandbox and load module
    let mut sandbox = WasmSandbox::new()?;
    let module_id = sandbox.load_module(&wasm_bytes)?;
    let instance_id = sandbox.create_instance(module_id, Some(instance_config))?;
    
    println!("🛡️  Sandbox created with secure file processing configuration");
    
    // Step 5: Configure the processing job
    let config = ProcessingConfig {
        input_format: "text".to_string(),
        output_format: "json".to_string(),
        max_file_size: 1024 * 1024, // 1MB per file
        processing_options: serde_json::json!({
            "word_count": true,
            "line_count": true,
            "character_count": true,
            "uppercase_conversion": true
        }),
    };
    
    // Step 6: Execute the file processing
    println!("⚙️  Starting file processing...");
    
    let start_time = std::time::Instant::now();
    
    // Initialize the processor
    let init_result: bool = sandbox.call_function(instance_id, "initialize", &config).await?;
    
    if !init_result {
        return Err("Failed to initialize file processor".into());
    }
    
    // Process files in the input directory
    let processing_result: ProcessingResult = sandbox.call_function(
        instance_id,
        "process_directory", 
        &input_dir.to_string_lossy()
    ).await?;
    
    let elapsed = start_time.elapsed();
    
    // Step 7: Report results
    println!("✅ Processing completed in {:?}", elapsed);
    println!("📊 Results:");
    println!("   Files processed: {}", processing_result.files_processed);
    println!("   Bytes processed: {}", processing_result.bytes_processed);
    println!("   Processing time: {}ms", processing_result.processing_time_ms);
    
    if !processing_result.errors.is_empty() {
        println!("⚠️  Errors encountered:");
        for error in &processing_result.errors {
            println!("   - {}", error);
        }
    }
    
    // Step 8: Verify output files
    println!("📄 Output files created:");
    let mut output_entries = fs::read_dir(&output_dir).await?;
    while let Some(entry) = output_entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let size = entry.metadata().await?.len();
            println!("   - {} ({} bytes)", path.file_name().unwrap().to_string_lossy(), size);
        }
    }
    
    // Step 9: Demonstrate error handling
    demonstrate_error_handling(&mut sandbox, instance_id).await?;
    
    // Step 10: Cleanup
    sandbox.remove_instance(instance_id);
    println!("🧹 Sandbox cleaned up");
    
    Ok(())
}

// Simulate compiling a file processor to WASM
// In reality, this would be your Rust code compiled to WASM
async fn compile_file_processor() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // For this example, we'll load a pre-compiled WASM module
    // In practice, you'd compile your file processor with:
    // cargo build --target wasm32-wasi --release
    
    println!("📦 Loading file processor WASM module...");
    
    // This would be your actual file processor WASM
    let wasm_source = r#"
        (module
            (memory (export "memory") 1)
            
            ;; Initialize function
            (func (export "initialize") (result i32)
                ;; Return success (1)
                i32.const 1
            )
            
            ;; Process directory function
            (func (export "process_directory") (result i32)
                ;; Mock processing - return encoded result
                i32.const 42
            )
            
            ;; Add function for simple math
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )
    "#;
    
    // Convert WAT to WASM bytes
    let wasm_bytes = wat::parse_str(wasm_source)?;
    
    println!("✅ File processor module loaded ({} bytes)", wasm_bytes.len());
    Ok(wasm_bytes)
}

// Demonstrate comprehensive error handling
async fn demonstrate_error_handling(
    sandbox: &mut WasmSandbox, 
    instance_id: InstanceId
) -> Result<(), Box<dyn std::error::Error>> {
    use wasm_sandbox::Error;
    
    println!("\n🚨 Demonstrating Error Handling Scenarios:");
    
    // 1. Function that doesn't exist
    println!("1️⃣  Testing non-existent function call...");
    match sandbox.call_function::<(), bool>(instance_id, "non_existent_function", ()).await {
        Ok(_) => println!("   ❌ Unexpected success"),
        Err(e) => println!("   ✅ Function call correctly failed: {}", e),
    }
    
    // 2. Test simple math function that exists
    println!("2️⃣  Testing valid function call (math)...");
    match sandbox.call_function::<(i32, i32), i32>(instance_id, "add", (5, 7)).await {
        Ok(result) => println!("   ✅ Math function succeeded: 5 + 7 = {}", result),
        Err(e) => println!("   ❌ Math function failed: {}", e),
    }
    
    // 3. Test initialization function  
    println!("3️⃣  Testing initialization function...");
    match sandbox.call_function::<(), i32>(instance_id, "initialize", ()).await {
        Ok(result) => println!("   ✅ Initialize function succeeded: {}", result),
        Err(e) => println!("   ❌ Initialize function failed: {}", e),
    }
    
    Ok(())
}

// Enhanced error types that PUP requested
#[derive(Debug, thiserror::Error)]
pub enum FileProcessorError {
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Resource exhausted: {kind} used {used}/{limit}")]
    ResourceExhausted { 
        kind: String, 
        limit: u64, 
        used: u64 
    },
    
    #[error("File processing error: {0}")]
    FileProcessing(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("WASM runtime error: {0}")]
    WasmRuntime(#[from] wasm_sandbox::Error),
}

// Configuration builder pattern (as requested in feedback)
impl InstanceConfig {
    pub fn for_file_processor() -> Self {
        Self {
            capabilities: Capabilities {
                filesystem: vec![
                    FilesystemCapability::ReadOnly("./input".into()),
                    FilesystemCapability::WriteOnly("./output".into()),
                ],
                network: vec![], // No network for file processing
                environment: vec![], // No env vars
                system: vec![], // Minimal system access
                process: vec![], // No subprocess spawning
                time: vec![], // Basic time access
                random: vec![], // No crypto
            },
            resource_limits: ResourceLimits::builder()
                .memory_pages(1024) // 64MB
                .cpu_time(Duration::from_secs(30))
                .fuel(10_000_000)
                .build(),
            startup_timeout_ms: 5000,
            enable_debug: false,
        }
    }
    
    pub fn with_memory_pages(mut self, pages: u32) -> Self {
        self.resource_limits = self.resource_limits.with_memory_pages(pages);
        self
    }
    
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.resource_limits = self.resource_limits.with_cpu_time(duration);
        self
    }
    
    pub fn with_filesystem_read(mut self, path: impl Into<PathBuf>) -> Self {
        self.capabilities.filesystem.push(FilesystemCapability::ReadOnly(path.into()));
        self
    }
    
    pub fn with_filesystem_write(mut self, path: impl Into<PathBuf>) -> Self {
        self.capabilities.filesystem.push(FilesystemCapability::WriteOnly(path.into()));
        self
    }
}
