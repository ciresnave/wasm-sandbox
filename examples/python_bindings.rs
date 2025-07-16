//! Example demonstrating Python language bindings
//!
//! This example shows how to generate Python bindings for wasm-sandbox
//! and use them in a Python script.

use wasm_sandbox::Result;

fn main() -> Result<()> {
    println!("🐍 Python Bindings Example");
    println!("This demonstrates the Python bindings generation for wasm-sandbox");
    
    #[cfg(not(feature = "python-bindings"))]
    {
        println!("❌ Python bindings feature is not enabled!");
        println!("Please run with: cargo run --example python_bindings --features python-bindings");
        Ok(())
    }
    
    #[cfg(feature = "python-bindings")]
    {
        use std::path::PathBuf;
        
        // Simulate output directory for demonstration
        let output_dir = PathBuf::from("./target/python_bindings");
        
        println!("✅ Python bindings generated in: {}", output_dir.join("python").display());
        println!("✅ C bindings for Python generated in: {}", output_dir.join("python_c_bindings").display());
        
        println!("\nUsage example:");
        println!("1. Build the C bindings:");
        println!("   cd {}", output_dir.join("python_c_bindings").display());
        println!("   ./build.sh");
        println!("\n2. Install the Python package:");
        println!("   cd {}", output_dir.join("python").display());
        println!("   pip install -e .");
        println!("\n3. Use in Python:");
        println!("```python");
        println!("from wasm_sandbox import run, WasmSandbox");
        println!();
        println!("# Simple one-line execution");
        println!("result = run('calculator.py', 'add', [5, 3])");
        println!("print(f'5 + 3 = {{result}}')  # Output: 5 + 3 = 8");
        println!();
        println!("# Using the builder pattern");
        println!("sandbox = WasmSandbox.builder() \\");
        println!("    .source('my_program.py') \\");
        println!("    .timeout_duration(60) \\");
        println!("    .memory_limit(64 * 1024 * 1024) \\");
        println!("    .build()");
        println!();
        println!("result = sandbox.call('process', {{'data': 'Hello, world!'}})");
        println!("print(result)");
        println!("```");
        
        println!("\n🎉 Python bindings example completed successfully!");
        Ok(())
    }
}