# Multi-Language Examples

This directory contains examples showing how to use wasm-sandbox with code written in different programming languages that compile to WebAssembly.

## Available Languages

### ✅ Currently Supported
- **[Rust](rust/)** - Native WASM support, excellent performance
- **[C/C++](c/)** - Using Emscripten toolchain
- **[JavaScript/TypeScript](javascript/)** - Using AssemblyScript (planned)
- **[Python](python/)** - Using PyO3 or similar (planned)
- **[Go](go/)** - Using TinyGo (planned)

### 🔄 Planned Support
- **Zig** - Native WASM support
- **C#** - Using Blazor WebAssembly
- **Java** - Using TeaVM or similar
- **Swift** - Using SwiftWasm

## Universal Pattern

Each language example follows the same pattern to demonstrate the simplicity:

```rust
// The goal: This should work regardless of source language
let result = wasm_sandbox::run("./my_code.{ext}", "function_name", &params)?;
```

## Example Structure

Each language directory contains:
- **Source code** - The actual program to be sandboxed
- **Build configuration** - How to compile to WASM
- **Usage example** - Rust code showing wasm-sandbox usage
- **README** - Language-specific setup and notes

## Getting Started by Language

### If you're a Rust developer
```bash
cd rust/calculator/
cargo run
```

### If you're a C developer
```bash
cd c/math_library/
make run-example
```

### If you're a Python developer
```bash
cd python/text_processor/
python setup.py
cargo run
```

### If you're a JavaScript developer
```bash
cd javascript/data_transformer/
npm install
npm run build:wasm
cargo run
```

## Language-Specific Getting Started

- **[Rust developers](rust/README.md)** - "I have Rust code I want to sandbox"
- **[C/C++ developers](c/README.md)** - "I have C/C++ code I want to sandbox"
- **[Python developers](python/README.md)** - "I have Python code I want to sandbox"
- **[JavaScript developers](javascript/README.md)** - "I have JS/TS code I want to sandbox"

## Design Goal

The examples demonstrate that wasm-sandbox should make it equally easy to sandbox code regardless of the source language. Whether you're coming from systems programming (Rust/C), scripting (Python), or web development (JavaScript), the pattern is the same:

1. **Point to your code** - `WasmSandbox::from_source("./my_project/")`
2. **Call functions** - `sandbox.call("function_name", &params)`
3. **Get results** - Type-safe return values

This universality makes wasm-sandbox valuable to the entire developer ecosystem, not just Rust developers.
