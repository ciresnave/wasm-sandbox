// Example: Using wasm-sandbox with Rust-compiled WASM
//
// This demonstrates the FUTURE simplified API for wasm-sandbox v0.3.0
// Currently this won't compile as the simple API doesn't exist yet

use wasm_sandbox::WasmSandbox;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Rust Calculator Example");
    println!("This shows how easy it should be to sandbox Rust code");
    
    // FUTURE v0.3.0 API - Dead simple!
    /*
    // One-liner: auto-compile and run
    let result: i32 = wasm_sandbox::run("./calculator/", "add", &(5, 3))?;
    println!("5 + 3 = {}", result);
    
    // Auto-compile once, reuse many times
    let calculator = WasmSandbox::from_source("./calculator/")?;
    
    println!("\n📊 Running calculations in sandbox:");
    
    // Basic arithmetic
    let sum: i32 = calculator.call("add", &(10, 20))?;
    println!("10 + 20 = {}", sum);
    
    let difference: i32 = calculator.call("subtract", &(100, 25))?;
    println!("100 - 25 = {}", difference);
    
    let product: i32 = calculator.call("multiply", &(7, 8))?;
    println!("7 * 8 = {}", product);
    
    let quotient: f64 = calculator.call("divide", &(22.0, 7.0))?;
    println!("22 / 7 = {}", quotient);
    
    // More complex operations
    let fact: u64 = calculator.call("factorial", &6u32)?;
    println!("6! = {}", fact);
    
    let numbers = vec![1, 2, 3, 4, 5];
    let sum_all: i32 = calculator.call("sum_array", &numbers)?;
    println!("Sum of {:?} = {}", numbers, sum_all);
    
    let reversed: String = calculator.call("reverse_string", &"Hello, WASM!")?;
    println!("Reversed: {}", reversed);
    
    // Even error handling should be simple
    let div_by_zero: f64 = calculator.call("divide", &(10.0, 0.0))?;
    println!("10 / 0 = {} (NaN indicates error)", div_by_zero);
    */
    
    // CURRENT v0.2.0 API - More complex but works
    println!("\n🔧 Current v0.2.0 API (more complex):");
    
    // You would need to manually compile the Rust project to WASM first:
    // cd calculator && wasm-pack build --target web
    
    println!("1. Compile Rust to WASM: cd calculator && wasm-pack build");
    println!("2. Load WASM bytes into sandbox");
    println!("3. Create module and instance");
    println!("4. Call functions with proper serialization");
    println!("\nThis is why we need the simplified API! 🎯");
    
    Ok(())
}

/*
This example shows how wasm-sandbox SHOULD work:

1. **Auto-compilation**: Just point to Rust source, it compiles to WASM
2. **Simple function calls**: Type-safe parameter passing
3. **Reusable sandboxes**: Compile once, call many functions
4. **Secure by default**: Automatic sandboxing with sane limits
5. **Error handling**: Clean error propagation

The goal is that a user with Rust code can go from source to 
secure sandboxed execution in 2 lines of code.
*/
