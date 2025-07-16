# Development Setup

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Complete development environment setup for contributing to wasm-sandbox, including toolchain configuration, IDE setup, and development workflows.

## Prerequisites

### System Requirements

#### Operating Systems

- **Linux**: Ubuntu 20.04+, Fedora 35+, Arch Linux
- **macOS**: macOS 11.0+ (Big Sur or later)
- **Windows**: Windows 10/11 with WSL2 recommended

#### Hardware Requirements

- **CPU**: 64-bit processor (x86_64 or ARM64)
- **Memory**: 8GB RAM minimum, 16GB recommended
- **Storage**: 10GB free space for development environment
- **Network**: Stable internet connection for downloading dependencies

### Core Dependencies

#### Rust Toolchain

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install required targets and components
rustup target add wasm32-wasi
rustup target add wasm32-unknown-unknown
rustup component add rustfmt
rustup component add clippy
rustup component add rust-src

# Verify installation
rustc --version
cargo --version
```

#### WebAssembly Tools

```bash
# Install wasmtime CLI
curl https://wasmtime.dev/install.sh -sSf | bash

# Install wasmer CLI
curl https://get.wasmer.io -sSf | sh

# Install wasm-pack for building WebAssembly packages
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install cargo-generate for project templates
cargo install cargo-generate

# Install wasm-bindgen CLI
cargo install wasm-bindgen-cli

# Install wabt (WebAssembly Binary Toolkit)
# Linux/WSL
sudo apt install wabt
# macOS
brew install wabt
# Windows (chocolatey)
choco install wabt

# Verify WebAssembly tools
wasmtime --version
wasmer --version
wasm-pack --version
```

#### Additional Tools

```bash
# Install useful development tools
cargo install cargo-watch      # File watching for development
cargo install cargo-expand     # Macro expansion
cargo install cargo-audit      # Security auditing
cargo install cargo-outdated   # Check for outdated dependencies
cargo install cargo-tree       # Dependency tree visualization
cargo install cargo-bloat      # Binary size analysis
cargo install flamegraph       # Performance profiling
cargo install criterion        # Benchmarking (if not using built-in)

# Install git hooks manager
cargo install cargo-husky

# Install documentation tools
cargo install mdbook           # For building documentation
cargo install mdbook-mermaid   # Mermaid diagram support
```

## Project Setup

### Repository Setup

```bash
# Clone the repository
git clone https://github.com/your-org/wasm-sandbox.git
cd wasm-sandbox

# Set up git hooks (if using cargo-husky)
cargo husky install

# Create development branch
git checkout -b development
git push -u origin development
```

### Environment Configuration

#### Environment Variables

Create a `.env` file in the project root:

```bash
# .env file for development
export RUST_LOG=debug
export RUST_BACKTRACE=1
export WASM_SANDBOX_TEST_MODE=1
export WASM_SANDBOX_LOG_LEVEL=debug
export WASM_SANDBOX_CACHE_DIR=./target/wasm_cache
export WASM_SANDBOX_PLUGIN_DIR=./plugins
export WASMTIME_BACKTRACE_DETAILS=1

# Performance testing
export WASM_SANDBOX_PERF_MODE=0
export WASM_SANDBOX_MEMORY_LIMIT=1073741824  # 1GB default

# Feature flags for development
export WASM_SANDBOX_EXPERIMENTAL_FEATURES=1
export WASM_SANDBOX_ENABLE_HOT_RELOAD=1
export WASM_SANDBOX_DEBUG_COMPILATION=1
```

Load environment variables:

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
if [ -f .env ]; then
    export $(cat .env | sed 's/#.*//g' | xargs)
fi

# Or use direnv for automatic loading
echo "dotenv" > .envrc
direnv allow
```

#### Development Configuration

Create `Cargo.toml` development section:

```toml
# Development-specific configuration
[profile.dev]
opt-level = 0
debug = true
split-debuginfo = "unpacked"
debug-assertions = true
overflow-checks = true
lto = false
panic = "unwind"
incremental = true
codegen-units = 256

[profile.dev.package."*"]
opt-level = 1

# Fast compilation profile
[profile.dev-fast]
inherits = "dev"
opt-level = 1
incremental = true
debug-assertions = false
overflow-checks = false

# Performance testing profile
[profile.perf-test]
inherits = "release"
debug = true
lto = "thin"
```

### IDE Configuration

#### Visual Studio Code

Install recommended extensions:

```json
// .vscode/extensions.json
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml",
        "serayuzgur.crates",
        "vadimcn.vscode-lldb",
        "ms-vscode.hexeditor",
        "ms-vscode.cmake-tools",
        "webassemblyjs.wasm",
        "ms-vscode.test-adapter-converter",
        "hbenl.vscode-test-explorer",
        "GitHub.copilot",
        "GitHub.copilot-chat"
    ]
}
```

Configure settings:

```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.check.allTargets": true,
    "rust-analyzer.cargo.loadOutDirsFromCheck": true,
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.completion.autoimport.enable": true,
    "rust-analyzer.experimental.procAttrMacros": true,
    "rust-analyzer.lens.enable": true,
    "rust-analyzer.lens.run": true,
    "rust-analyzer.lens.debug": true,
    "rust-analyzer.lens.implementations": true,
    "rust-analyzer.lens.references": true,
    "rust-analyzer.inlayHints.enable": true,
    "rust-analyzer.inlayHints.parameterHints": true,
    "rust-analyzer.inlayHints.typeHints": true,
    "rust-analyzer.workspace.symbol.search.scope": "workspace_and_dependencies",
    "files.watcherExclude": {
        "**/target/**": true,
        "**/.git/**": true
    },
    "search.exclude": {
        "**/target": true,
        "**/Cargo.lock": true
    },
    "files.associations": {
        "*.wast": "lisp",
        "*.wat": "lisp"
    },
    "terminal.integrated.env.linux": {
        "RUST_LOG": "debug",
        "RUST_BACKTRACE": "1"
    }
}
```

Configure tasks:

```json
// .vscode/tasks.json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "cargo build",
            "type": "cargo",
            "command": "build",
            "group": "build",
            "problemMatcher": ["$rustc"]
        },
        {
            "label": "cargo build --release",
            "type": "cargo",
            "command": "build",
            "args": ["--release"],
            "group": "build",
            "problemMatcher": ["$rustc"]
        },
        {
            "label": "cargo test",
            "type": "cargo",
            "command": "test",
            "group": "test",
            "problemMatcher": ["$rustc"]
        },
        {
            "label": "cargo test -- --nocapture",
            "type": "cargo",
            "command": "test",
            "args": ["--", "--nocapture"],
            "group": "test",
            "problemMatcher": ["$rustc"]
        },
        {
            "label": "cargo clippy",
            "type": "cargo",
            "command": "clippy",
            "args": ["--all-targets", "--all-features"],
            "group": "build",
            "problemMatcher": ["$rustc"]
        },
        {
            "label": "cargo fmt",
            "type": "cargo",
            "command": "fmt",
            "group": "build"
        },
        {
            "label": "cargo watch",
            "type": "shell",
            "command": "cargo",
            "args": ["watch", "-x", "check", "-x", "test"],
            "group": "build",
            "isBackground": true,
            "problemMatcher": ["$rustc"]
        },
        {
            "label": "build example module",
            "type": "shell",
            "command": "cargo",
            "args": [
                "build",
                "--target", "wasm32-wasi",
                "--manifest-path", "examples/wasm_modules/Cargo.toml"
            ],
            "group": "build",
            "problemMatcher": ["$rustc"]
        },
        {
            "label": "run benchmarks",
            "type": "cargo",
            "command": "bench",
            "group": "test",
            "problemMatcher": ["$rustc"]
        }
    ]
}
```

Configure launch configurations:

```json
// .vscode/launch.json
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Debug wasm-sandbox",
            "type": "lldb",
            "request": "launch",
            "program": "${workspaceFolder}/target/debug/wasm-sandbox",
            "args": [],
            "cwd": "${workspaceFolder}",
            "environment": [
                {"name": "RUST_LOG", "value": "debug"},
                {"name": "RUST_BACKTRACE", "value": "1"}
            ]
        },
        {
            "name": "Debug tests",
            "type": "lldb",
            "request": "launch",
            "program": "${workspaceFolder}/target/debug/deps/wasm_sandbox-${input:testExecutable}",
            "args": ["--nocapture"],
            "cwd": "${workspaceFolder}",
            "environment": [
                {"name": "RUST_LOG", "value": "debug"},
                {"name": "RUST_BACKTRACE", "value": "1"}
            ]
        },
        {
            "name": "Debug example",
            "type": "lldb",
            "request": "launch",
            "program": "${workspaceFolder}/target/debug/examples/${input:exampleName}",
            "args": [],
            "cwd": "${workspaceFolder}",
            "environment": [
                {"name": "RUST_LOG", "value": "debug"}
            ]
        }
    ],
    "inputs": [
        {
            "id": "testExecutable",
            "description": "Test executable suffix",
            "default": "",
            "type": "promptString"
        },
        {
            "id": "exampleName",
            "description": "Example name",
            "default": "basic_usage",
            "type": "promptString"
        }
    ]
}
```

#### JetBrains IntelliJ IDEA / CLion

Install Rust plugin and configure:

```xml
<!-- .idea/runConfigurations/Cargo_Build.xml -->
<component name="ProjectRunConfigurationManager">
  <configuration default="false" name="Cargo Build" type="CargoCommandRunConfiguration" factoryName="Cargo Command">
    <option name="command" value="build --all-features" />
    <option name="workingDirectory" value="file://$PROJECT_DIR$" />
    <option name="channel" value="DEFAULT" />
    <option name="requiredFeatures" value="true" />
    <option name="allFeatures" value="true" />
    <option name="emulateTerminal" value="false" />
    <option name="withSudo" value="false" />
    <option name="buildTarget" value="REMOTE" />
    <option name="backtrace" value="SHORT" />
    <envs>
      <env name="RUST_LOG" value="debug" />
    </envs>
  </configuration>
</component>
```

#### Vim/Neovim

Configure with rust-analyzer:

```lua
-- lua/config/rust.lua
local lspconfig = require('lspconfig')

lspconfig.rust_analyzer.setup({
    settings = {
        ["rust-analyzer"] = {
            cargo = {
                features = "all",
                loadOutDirsFromCheck = true,
            },
            procMacro = {
                enable = true,
            },
            check = {
                command = "clippy",
                allTargets = true,
            },
            completion = {
                autoimport = {
                    enable = true,
                },
            },
            inlayHints = {
                enable = true,
                parameterHints = true,
                typeHints = true,
            },
        },
    },
})
```

## Development Workflow

### Project Structure

Understanding the project layout:

```
wasm-sandbox/
├── src/                           # Main source code
│   ├── lib.rs                    # Library entry point
│   ├── error.rs                  # Error types
│   ├── runtime/                  # Runtime implementations
│   │   ├── mod.rs
│   │   ├── wasmtime.rs          # Wasmtime backend
│   │   ├── wasmer.rs            # Wasmer backend
│   │   └── wasm_common.rs       # Common runtime traits
│   ├── security/                 # Security and capabilities
│   │   ├── mod.rs
│   │   ├── capabilities.rs      # Capability definitions
│   │   ├── resource_limits.rs   # Resource limiting
│   │   └── audit.rs             # Security auditing
│   ├── communication/            # Host-guest communication
│   │   ├── mod.rs
│   │   ├── channels.rs          # Communication channels
│   │   ├── memory.rs            # Memory-based communication
│   │   └── rpc.rs               # RPC system
│   ├── compiler/                 # WebAssembly compilation
│   │   ├── mod.rs
│   │   ├── cargo.rs             # Cargo integration
│   │   └── wasi.rs              # WASI support
│   ├── wrappers/                 # High-level wrappers
│   │   ├── mod.rs
│   │   ├── http_server.rs       # HTTP server wrapper
│   │   ├── cli_tool.rs          # CLI tool wrapper
│   │   └── mcp_server.rs        # MCP server wrapper
│   ├── templates/                # Code generation templates
│   │   ├── mod.rs
│   │   └── *.rs                 # Template implementations
│   └── utils/                    # Utilities
│       ├── mod.rs
│       ├── logging.rs           # Logging setup
│       └── manifest.rs          # Manifest parsing
├── examples/                      # Example applications
│   ├── basic_usage.rs           # Basic API usage
│   ├── http_server.rs           # HTTP server example
│   ├── cli_wrapper.rs           # CLI wrapper example
│   ├── mcp_server.rs            # MCP server example
│   └── wasm_modules/            # Example WASM modules
│       ├── Cargo.toml           # WASM module build config
│       └── src/                 # WASM module source
├── tests/                        # Integration tests
│   ├── integration_test.rs      # Main integration tests
│   └── integration/             # Test modules
├── benches/                      # Benchmarks
│   ├── startup.rs               # Startup benchmarks
│   └── communication.rs        # Communication benchmarks
├── fixtures/                     # Test fixtures
│   ├── *.wasm                   # Pre-compiled test modules
│   └── *.wat                    # WebAssembly text files
├── docs/                         # Documentation
│   ├── guides/                  # User guides
│   └── design/                  # Design documents
├── scripts/                      # Development scripts
│   ├── build-examples.sh        # Build example modules
│   ├── run-benchmarks.sh        # Run performance tests
│   └── check-security.sh        # Security checks
└── Cargo.toml                   # Main project configuration
```

### Daily Development Commands

#### Building and Testing

```bash
# Quick development build
cargo build

# Build with all features
cargo build --all-features

# Build for release
cargo build --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_sandbox_creation

# Run integration tests only
cargo test --test integration_test

# Run benchmarks
cargo bench

# Check code without building
cargo check

# Fast check with all features
cargo check --all-features

# Format code
cargo fmt

# Run clippy lints
cargo clippy --all-targets --all-features

# Audit dependencies for security issues
cargo audit

# Check for outdated dependencies
cargo outdated
```

#### WebAssembly Module Development

```bash
# Build example WASM modules
cd examples/wasm_modules
cargo build --target wasm32-wasi --release

# Build specific module
cargo build --target wasm32-wasi --bin simple_calculator

# Optimize WASM module
wasm-opt -Oz target/wasm32-wasi/release/simple_calculator.wasm \
         -o optimized_calculator.wasm

# Inspect WASM module
wasm-objdump -x target/wasm32-wasi/release/simple_calculator.wasm

# Convert WASM to WAT (text format)
wasm2wat target/wasm32-wasi/release/simple_calculator.wasm \
         -o calculator.wat

# Validate WASM module
wasmtime --invoke add calculator.wasm 5 3
```

#### Development Automation

Create development scripts in `scripts/` directory:

```bash
#!/bin/bash
# scripts/dev-setup.sh - Development environment setup

set -e

echo "Setting up wasm-sandbox development environment..."

# Build project
echo "Building project..."
cargo build --all-features

# Build example WASM modules
echo "Building example WASM modules..."
cd examples/wasm_modules
cargo build --target wasm32-wasi --release
cd ../..

# Run basic tests
echo "Running tests..."
cargo test --lib

# Check formatting and lints
echo "Checking code quality..."
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings

echo "Development environment ready!"
```

```bash
#!/bin/bash
# scripts/watch-dev.sh - Continuous development

# Watch for changes and rebuild
cargo watch -x "build --all-features" \
           -x "test --lib" \
           -x "clippy --all-targets --all-features"
```

```bash
#!/bin/bash
# scripts/benchmark-compare.sh - Compare benchmark results

# Run benchmarks and save results
cargo bench -- --save-baseline before

# Make your changes, then run:
# cargo bench -- --baseline before
```

### Testing Strategy

#### Unit Tests

```rust
// src/runtime/wasmtime.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SecurityPolicy;

    #[tokio::test]
    async fn test_wasmtime_runtime_creation() {
        let runtime = WasmtimeRuntime::new().await.unwrap();
        assert!(runtime.is_initialized());
    }

    #[tokio::test]
    async fn test_module_loading() {
        let runtime = WasmtimeRuntime::new().await.unwrap();
        let module = runtime.load_module("fixtures/test_module.wasm").await.unwrap();
        assert_eq!(module.name(), "test_module");
    }

    #[tokio::test]
    async fn test_security_policy_enforcement() {
        let policy = SecurityPolicy::strict();
        let runtime = WasmtimeRuntime::with_policy(policy).await.unwrap();
        
        // Test that network access is denied
        let result = runtime.test_network_access().await;
        assert!(result.is_err());
    }
}
```

#### Integration Tests

```rust
// tests/integration_test.rs
use wasm_sandbox::{WasmSandbox, SecurityPolicy};

#[tokio::test]
async fn test_end_to_end_execution() {
    let sandbox = WasmSandbox::builder()
        .source("fixtures/test_module.wasm")
        .security_policy(SecurityPolicy::strict())
        .build()
        .await
        .unwrap();

    let result: i32 = sandbox.call("add", (5, 3)).await.unwrap();
    assert_eq!(result, 8);
}

#[tokio::test]
async fn test_resource_limits() {
    let sandbox = WasmSandbox::builder()
        .source("fixtures/memory_intensive.wasm")
        .memory_limit(1024 * 1024) // 1MB
        .build()
        .await
        .unwrap();

    let result = sandbox.call("allocate_large_buffer", ()).await;
    assert!(result.is_err()); // Should fail due to memory limit
}
```

#### Performance Tests

```rust
// benches/startup.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use wasm_sandbox::WasmSandbox;

fn bench_sandbox_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("sandbox_creation", |b| {
        b.to_async(&rt).iter(|| async {
            WasmSandbox::builder()
                .source("fixtures/test_module.wasm")
                .build()
                .await
                .unwrap()
        });
    });
}

fn bench_function_calls(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let sandbox = rt.block_on(async {
        WasmSandbox::builder()
            .source("fixtures/test_module.wasm")
            .build()
            .await
            .unwrap()
    });

    c.bench_function("function_call", |b| {
        b.to_async(&rt).iter(|| async {
            let _: i32 = sandbox.call("add", (5, 3)).await.unwrap();
        });
    });
}

criterion_group!(benches, bench_sandbox_creation, bench_function_calls);
criterion_main!(benches);
```

### Debugging Techniques

#### Environment Variables for Debugging

```bash
# Enable detailed logging
export RUST_LOG=wasm_sandbox=trace,wasmtime=debug

# Enable backtraces
export RUST_BACKTRACE=full

# Enable Wasmtime debugging
export WASMTIME_BACKTRACE_DETAILS=1

# Enable compilation debugging
export WASM_SANDBOX_DEBUG_COMPILATION=1
```

#### LLDB/GDB Debugging

```bash
# Debug with LLDB
lldb target/debug/examples/basic_usage
(lldb) b wasm_sandbox::runtime::wasmtime::WasmtimeRuntime::new
(lldb) r
(lldb) bt

# Debug with GDB
gdb target/debug/examples/basic_usage
(gdb) break wasm_sandbox::runtime::wasmtime::WasmtimeRuntime::new
(gdb) run
(gdb) backtrace
```

#### Memory Debugging

```bash
# Install valgrind (Linux)
sudo apt install valgrind

# Run with valgrind
valgrind --leak-check=full \
         --show-leak-kinds=all \
         --track-origins=yes \
         target/debug/examples/basic_usage

# Use AddressSanitizer
export RUSTFLAGS="-Z sanitizer=address"
cargo build --target x86_64-unknown-linux-gnu
```

#### Performance Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile application
cargo flamegraph --example basic_usage

# Profile specific function
cargo flamegraph --example basic_usage -- --function-name add

# Use perf (Linux)
perf record target/debug/examples/basic_usage
perf report
```

### Code Quality Tools

#### Pre-commit Hooks

Create `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt
        language: system
        files: \.rs$
        args: [--check]
      
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy
        language: system
        files: \.rs$
        args: [--all-targets, --all-features, --, -D, warnings]
      
      - id: cargo-test
        name: cargo test
        entry: cargo test
        language: system
        files: \.rs$
        pass_filenames: false
      
      - id: cargo-audit
        name: cargo audit
        entry: cargo audit
        language: system
        pass_filenames: false
```

#### Continuous Integration

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [ main, development ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta, nightly]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        override: true
        components: rustfmt, clippy
        target: wasm32-wasi
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Cache cargo build
      uses: actions/cache@v3
      with:
        path: target
        key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Install WebAssembly tools
      run: |
        curl https://wasmtime.dev/install.sh -sSf | bash
        echo "$HOME/.wasmtime/bin" >> $GITHUB_PATH
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Build
      run: cargo build --verbose --all-features
    
    - name: Build WASM modules
      run: |
        cd examples/wasm_modules
        cargo build --target wasm32-wasi --release
    
    - name: Run tests
      run: cargo test --verbose --all-features
    
    - name: Run benchmarks
      run: cargo bench --no-run
    
    - name: Security audit
      run: |
        cargo install cargo-audit
        cargo audit

  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        target: wasm32-wasi
    
    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin
    
    - name: Generate coverage
      run: cargo tarpaulin --out xml --all-features
    
    - name: Upload coverage
      uses: codecov/codecov-action@v3
```

## Contributing Guidelines

### Code Style

Follow Rust standard formatting:

```rust
// Use consistent naming
struct SandboxBuilder { ... }
enum SecurityLevel { ... }
trait RuntimeBackend { ... }

// Document public APIs
/// Creates a new WebAssembly sandbox with secure execution environment.
/// 
/// # Examples
/// 
/// ```rust
/// use wasm_sandbox::{WasmSandbox, SecurityPolicy};
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let sandbox = WasmSandbox::builder()
///     .source("module.wasm")
///     .security_policy(SecurityPolicy::strict())
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub async fn new() -> Result<Self, Error> { ... }

// Use descriptive error messages
return Err(Error::ModuleLoadFailed {
    path: module_path.to_string(),
    reason: "Invalid WebAssembly bytecode".to_string(),
});

// Prefer explicit types for public APIs
pub fn set_memory_limit(&mut self, limit: u64) -> &mut Self { ... }
```

### Pull Request Process

1. **Create Feature Branch**

   ```bash
   git checkout -b feature/awesome-new-feature
   ```

2. **Make Changes**
   - Write tests first (TDD approach)
   - Implement feature
   - Update documentation
   - Add benchmarks if performance-critical

3. **Quality Checks**

   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features
   cargo test --all-features
   cargo audit
   ```

4. **Commit Changes**

   ```bash
   git add .
   git commit -m "feat: add awesome new feature
   
   - Implement XYZ functionality
   - Add comprehensive tests
   - Update documentation
   - Add performance benchmarks"
   ```

5. **Push and Create PR**

   ```bash
   git push origin feature/awesome-new-feature
   ```

### Documentation Standards

- **API Documentation**: All public functions must have doc comments
- **Examples**: Include working examples in doc comments
- **Architecture Documentation**: Update design docs for architectural changes
- **Changelog**: Update CHANGELOG.md following [Keep a Changelog](https://keepachangelog.com/)

### Testing Requirements

- **Unit Tests**: 80%+ coverage for new code
- **Integration Tests**: End-to-end test scenarios
- **Performance Tests**: Benchmarks for performance-critical code
- **Security Tests**: Security boundary validation

Next: **[Roadmap](../design/roadmap.md)** - Project roadmap and future directions

---

**Development Excellence:** Complete development environment setup with modern tooling, comprehensive testing, and quality assurance workflows.
