# Rust Calculator Example

This example demonstrates how to use wasm-sandbox with Rust code. It shows both the **future simplified API** (v0.3.0) and the **current API** (v0.2.0).

## 🎯 The Goal: Dead Simple

```rust
// This is what we're working towards:
let result: i32 = wasm_sandbox::run("./calculator/", "add", &(5, 3))?;
```

## 📁 Project Structure

```
calculator/
├── Cargo.toml          # WASM-enabled Rust project
├── src/
│   └── lib.rs          # Calculator functions
├── example.rs          # How to use with wasm-sandbox
└── README.md           # This file
```

## 🚀 Running the Example

### Future API (v0.3.0 - Not Yet Implemented)

```bash
# This is the vision - one command:
cargo run --bin example
```

The example would:
1. Auto-detect this is a Rust project
2. Auto-compile to WebAssembly
3. Create a secure sandbox
4. Call functions with type safety

### Current API (v0.2.0 - Manual Steps)

```bash
# 1. Compile Rust to WASM
cd calculator
wasm-pack build --target web

# 2. Run the current (more complex) example
cargo run --bin current_example
```

## 🧮 Calculator Functions

The calculator provides these functions:

```rust
pub fn add(a: i32, b: i32) -> i32
pub fn subtract(a: i32, b: i32) -> i32  
pub fn multiply(a: i32, b: i32) -> i32
pub fn divide(a: f64, b: f64) -> f64
pub fn factorial(n: u32) -> u64
pub fn sum_array(numbers: &[i32]) -> i32
pub fn reverse_string(input: &str) -> String
```

## ✨ Why This Matters

This example demonstrates the core value proposition:

### Before (Complex)
```rust
// Manual WASM compilation
// Complex sandbox setup  
// Manual module loading
// Manual instance creation
// Complex function calling
// Manual error handling
```

### After (Simple)
```rust
let result = wasm_sandbox::run("./calculator/", "add", &(5, 3))?;
```

## 🔒 Security Benefits

Even with the simple API, you get:

- **Memory isolation** - Calculator can't access host memory
- **Resource limits** - Automatic limits on CPU and memory usage
- **Capability control** - No filesystem or network access by default
- **Error containment** - WASM errors don't crash the host

## 🎓 Learning Path

1. **Try this example** - See how simple it could be
2. **Check the current API** - Understand what exists today
3. **Read the design docs** - See how we'll get there
4. **Contribute!** - Help build the simple API

## 💡 For Rust Developers

If you're a Rust developer who wants to sandbox Rust code:

1. **This is your target experience** - Rust-to-WASM should be invisible
2. **Your code doesn't change** - Same Rust, just sandboxed
3. **You get security for free** - Automatic isolation and limits
4. **Performance stays high** - WASM is near-native speed

This is why wasm-sandbox focusing on ease of use will unlock WASM sandboxing for every Rust developer, not just WASM experts.
