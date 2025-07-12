# Ease of Use Improvements for wasm-sandbox

## Problem Statement

The current wasm-sandbox API requires too much ceremony for simple use cases. Users should be able to go from "I have some code" to "sandboxed execution" with minimal steps and zero configuration for common scenarios.

## Core Principle: Progressive Complexity

The API should follow a "pit of success" design:
1. **Dead simple** for basic use cases (auto-compile, sane defaults)
2. **Progressively more control** as needs become complex
3. **Full customization** available but not required

## Proposed Simplified API

### Level 1: One-Line Execution

```rust
// Compile and run a single function
let result: i32 = wasm_sandbox::run("./calculator.rs", "add", &(5, 3))?;

// Run with timeout (still dead simple)
let result: String = wasm_sandbox::run_with_timeout(
    "./text_processor.py", 
    "process", 
    &"Hello, World!",
    Duration::from_secs(30)
)?;

// Run entire program
let output = wasm_sandbox::execute("./my_program.rs", &["arg1", "arg2"])?;
```

### Level 2: Reusable Sandbox

```rust
// Auto-compile from source directory
let sandbox = WasmSandbox::from_source("./my_project/")?;

// Call functions (sync by default, async available)
let result1 = sandbox.call("function1", &params1)?;
let result2 = sandbox.call("function2", &params2)?;

// Or async when needed
let result = sandbox.call_async("slow_function", &params).await?;
```

### Level 3: Basic Configuration

```rust
// Simple builder pattern with sane defaults
let sandbox = WasmSandbox::builder()
    .source("./my_project/")
    .memory_limit("64MB")          // Human-readable units
    .timeout("30s")                // Human-readable duration
    .allow_network(false)          // Boolean for simple cases
    .build()?;

let result = sandbox.call("process_data", &input)?;
```

### Level 4: Advanced Configuration (Current API)

```rust
// Full control when needed
let config = SandboxConfig {
    runtime: RuntimeConfig::wasmtime(),
    security: SecurityConfig {
        capabilities: Capabilities {
            filesystem: FilesystemCapability::ReadOnly(vec!["/data".into()]),
            network: NetworkCapability::Loopback,
            // ... detailed configuration
        },
        resource_limits: ResourceLimits { /* ... */ },
    },
};

let sandbox = WasmSandbox::with_config(config)?;
// ... existing detailed API
```

## Auto-Compilation Support

The crate should automatically detect and compile source code:

### Rust Projects
```rust
// Detects Cargo.toml, compiles to WASM
let sandbox = WasmSandbox::from_source("./rust_project/")?;

// Or single file
let sandbox = WasmSandbox::from_file("./calculator.rs")?;
```

### Python Scripts
```rust
// Uses PyO3 or similar to compile Python to WASM
let sandbox = WasmSandbox::from_source("./python_script.py")?;
```

### C/C++ Projects
```rust
// Uses Emscripten toolchain
let sandbox = WasmSandbox::from_source("./c_project/")?;
```

### JavaScript/TypeScript
```rust
// Uses tools like AssemblyScript
let sandbox = WasmSandbox::from_source("./js_project/")?;
```

## Implementation Strategy

### 1. Auto-Detection System

```rust
impl WasmSandbox {
    pub fn from_source<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Auto-detect project type
        let project_type = detect_project_type(path)?;
        
        match project_type {
            ProjectType::Rust => compile_rust_project(path),
            ProjectType::Python => compile_python_script(path),
            ProjectType::C => compile_c_project(path),
            ProjectType::JavaScript => compile_js_project(path),
            ProjectType::PrecompiledWasm => load_wasm_file(path),
        }
    }
}

fn detect_project_type(path: &Path) -> Result<ProjectType> {
    if path.join("Cargo.toml").exists() {
        Ok(ProjectType::Rust)
    } else if path.extension() == Some(OsStr::new("py")) {
        Ok(ProjectType::Python)
    } else if path.extension() == Some(OsStr::new("wasm")) {
        Ok(ProjectType::PrecompiledWasm)
    } else if path.join("package.json").exists() {
        Ok(ProjectType::JavaScript)
    } else if path.join("Makefile").exists() || has_c_files(path) {
        Ok(ProjectType::C)
    } else {
        Err(Error::UnknownProjectType)
    }
}
```

### 2. Smart Defaults

```rust
impl Default for SandboxDefaults {
    fn default() -> Self {
        Self {
            memory_limit: 64 * 1024 * 1024,  // 64MB - reasonable for most tasks
            timeout: Duration::from_secs(30), // 30s - prevents runaway processes
            filesystem: FilesystemCapability::None, // Secure by default
            network: NetworkCapability::None,      // Secure by default
            allow_stdio: true,                     // Useful for most programs
        }
    }
}
```

### 3. Human-Readable Configuration

```rust
impl WasmSandboxBuilder {
    pub fn memory_limit<S: AsRef<str>>(mut self, limit: S) -> Self {
        self.config.memory_limit = parse_memory_size(limit.as_ref());
        self
    }
    
    pub fn timeout<S: AsRef<str>>(mut self, duration: S) -> Self {
        self.config.timeout = parse_duration(duration.as_ref());
        self
    }
}

fn parse_memory_size(s: &str) -> usize {
    // Parse "64MB", "1GB", "512KB", etc.
    // Panic with helpful error for invalid formats
}

fn parse_duration(s: &str) -> Duration {
    // Parse "30s", "5m", "1h", etc.
    // Panic with helpful error for invalid formats
}
```

## Multi-Language Examples

You're absolutely right about needing examples for all major WASM-compatible languages:

### Rust Example
```rust
// examples/languages/rust_calculator/
// - Cargo.toml (WASM target)
// - src/lib.rs (calculator functions)
// - example.rs (showing wasm-sandbox usage)

// Usage:
let result = wasm_sandbox::run(
    "./examples/languages/rust_calculator/", 
    "add", 
    &(5, 3)
)?;
```

### Python Example
```rust
// examples/languages/python_text_processor/
// - requirements.txt (if needed)
// - text_processor.py (text processing functions)
// - example.rs (showing wasm-sandbox usage)

// Usage:
let result = wasm_sandbox::run(
    "./examples/languages/python_text_processor/text_processor.py", 
    "process_text", 
    &"Hello, World!"
)?;
```

### C Example
```rust
// examples/languages/c_math_library/
// - Makefile (WASM compilation)
// - math_lib.c (math functions)
// - example.rs (showing wasm-sandbox usage)
```

### JavaScript/AssemblyScript Example
```rust
// examples/languages/js_data_transformer/
// - package.json (AssemblyScript setup)
// - transformer.ts (data transformation)
// - example.rs (showing wasm-sandbox usage)
```

### Go Example
```rust
// examples/languages/go_web_scraper/
// - go.mod (TinyGo for WASM)
// - scraper.go (web scraping logic)
// - example.rs (showing wasm-sandbox usage)
```

## Benefits of This Approach

### 1. **Lower Barrier to Entry**
- New users can be productive immediately
- No need to understand WASM compilation
- No need to understand complex security models initially

### 2. **Better Developer Experience**
- Auto-compilation removes build system knowledge requirement
- Human-readable configuration (e.g., "64MB" instead of `67108864`)
- Sensible defaults prevent security misconfigurations

### 3. **Broader Adoption**
- Developers from any language can try wasm-sandbox
- Examples in their native language make it approachable
- Progressive complexity allows growth with needs

### 4. **Production Ready**
- Simple API still provides security by default
- Advanced configuration available when needed
- Auto-compilation can be cached for performance

## Implementation Plan

### Phase 1: Simple API Layer
1. Implement one-line `wasm_sandbox::run()` function
2. Add `WasmSandbox::from_source()` with Rust support
3. Create builder pattern with human-readable options

### Phase 2: Multi-Language Support
1. Add Python compilation support
2. Add C/C++ compilation support  
3. Add JavaScript/AssemblyScript support
4. Add Go (TinyGo) support

### Phase 3: Examples and Documentation
1. Create working examples for each language
2. Add tutorials for each language
3. Create "language-specific getting started" guides

### Phase 4: Advanced Features
1. Caching for repeated compilations
2. Hot reload for development
3. IDE integration helpers

## API Backwards Compatibility

The new simple API can coexist with the current detailed API:

```rust
// Simple API (new)
let result = wasm_sandbox::run("./code.rs", "func", &params)?;

// Detailed API (existing, unchanged)
let sandbox = WasmSandbox::new()?;
let module_id = sandbox.load_module(&wasm_bytes)?;
// ... existing complex API
```

This allows:
- Existing users keep working code
- New users get simple onboarding
- Complex needs still fully supported
- Migration path from simple to complex

## Example Documentation Structure

```
docs/
├── getting-started/
│   ├── rust-users.md          # "I have Rust code"
│   ├── python-users.md        # "I have Python code"  
│   ├── c-users.md             # "I have C/C++ code"
│   ├── javascript-users.md    # "I have JS/TS code"
│   └── go-users.md            # "I have Go code"
├── tutorials/
│   ├── first-sandbox.md       # 5-minute tutorial
│   ├── adding-security.md     # When you need more control
│   └── production-deployment.md
└── examples/
    ├── languages/             # Working examples per language
    └── use-cases/             # Real-world scenarios
```

This approach transforms wasm-sandbox from "a WASM runtime abstraction" to "the easiest way to safely run untrusted code" - which is much more compelling for adoption!
