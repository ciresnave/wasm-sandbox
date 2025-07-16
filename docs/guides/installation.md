# Installation Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Complete installation instructions for wasm-sandbox across different platforms and use cases.

## Quick Installation

### Using Cargo

```bash
# Add to your Cargo.toml
[dependencies]
wasm-sandbox = "0.3"

# Or install via cargo add
cargo add wasm-sandbox
```

### Basic Setup

```rust
// src/main.rs
use wasm_sandbox::{WasmSandbox, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let sandbox = WasmSandbox::builder()
        .source("module.wasm")
        .build()
        .await?;
    
    let result: String = sandbox.call("greet", "World").await?;
    println!("{}", result);
    
    Ok(())
}
```

## System Requirements

### Minimum Requirements

- **Rust**: 1.70.0 or later
- **Operating System**:
  - Linux (x86_64, aarch64)
  - macOS (x86_64, Apple Silicon)
  - Windows (x86_64)
- **Memory**: 512MB available RAM
- **Storage**: 100MB free space

### Recommended Requirements

- **Rust**: Latest stable version
- **Memory**: 2GB+ available RAM
- **CPU**: Multi-core processor for parallel execution
- **Storage**: 1GB+ free space for compilation cache

## Platform-Specific Installation

### Linux (Ubuntu/Debian)

```bash
# Install system dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install WebAssembly targets
rustup target add wasm32-wasi
rustup target add wasm32-unknown-unknown

# Install development tools
cargo install wasm-pack
cargo install wasm-opt

# Add wasm-sandbox to your project
cargo add wasm-sandbox
```

### macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Install Rust via Homebrew (alternative to rustup)
brew install rust
# OR use rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install WebAssembly targets
rustup target add wasm32-wasi
rustup target add wasm32-unknown-unknown

# Install development tools
cargo install wasm-pack
brew install binaryen  # Provides wasm-opt

# Add wasm-sandbox to your project
cargo add wasm-sandbox
```

### Windows

#### Using PowerShell (Recommended)

```powershell
# Install Rust
Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
.\rustup-init.exe

# Restart shell or run:
# $env:PATH += ";$env:USERPROFILE\.cargo\bin"

# Install WebAssembly targets
rustup target add wasm32-wasi
rustup target add wasm32-unknown-unknown

# Install development tools
cargo install wasm-pack
cargo install wasm-opt

# Add wasm-sandbox to your project
cargo add wasm-sandbox
```

#### Using Windows Subsystem for Linux (WSL)

```bash
# Follow Linux installation instructions within WSL
# This provides the best development experience on Windows
wsl --install Ubuntu
wsl

# Then follow Ubuntu installation steps above
```

## Development Environment Setup

### VS Code Setup

Install recommended extensions:

```bash
# Install VS Code extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension ms-vscode.vscode-wasm
code --install-extension vadimcn.vscode-lldb
```

Create `.vscode/settings.json`:

```json
{
    "rust-analyzer.cargo.target": "wasm32-wasi",
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "files.watcherExclude": {
        "**/target/**": true,
        "**/.git/**": true
    }
}
```

Create `.vscode/tasks.json`:

```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Build WASM",
            "type": "shell",
            "command": "cargo",
            "args": ["build", "--target", "wasm32-wasi", "--release"],
            "group": {
                "kind": "build",
                "isDefault": true
            }
        },
        {
            "label": "Test Sandbox",
            "type": "shell",
            "command": "cargo",
            "args": ["test"],
            "group": "test"
        }
    ]
}
```

### IntelliJ IDEA / CLion Setup

1. Install the Rust plugin
2. Configure Rust toolchain path
3. Set up WASM target configuration

```toml
# Add to .idea/workspace.xml or similar
[build]
target = "wasm32-wasi"

[env]
RUST_LOG = "debug"
```

## Feature Flags

wasm-sandbox supports optional features that can be enabled in `Cargo.toml`:

```toml
[dependencies]
wasm-sandbox = { version = "0.3", features = [
    "wasmtime",         # Wasmtime runtime (default)
    "wasmer",           # Wasmer runtime support
    "streaming",        # Streaming data processing
    "async-traits",     # Async trait support
    "serde-json",       # JSON serialization
    "monitoring",       # Resource monitoring
    "audit",            # Security auditing
    "jit",              # Just-in-time compilation
    "component-model",  # WebAssembly Component Model
    "multi-tenant",     # Multi-tenant isolation
] }
```

### Feature Descriptions

| Feature | Description | Size Impact |
|---------|-------------|-------------|
| `wasmtime` | Wasmtime runtime backend (default) | +2.5MB |
| `wasmer` | Wasmer runtime backend | +3.1MB |
| `streaming` | Streaming data processing APIs | +0.1MB |
| `async-traits` | Async trait implementations | +0.05MB |
| `serde-json` | JSON serialization support | +0.3MB |
| `monitoring` | Resource usage monitoring | +0.2MB |
| `audit` | Security audit logging | +0.1MB |
| `jit` | JIT compilation optimizations | +0.5MB |
| `component-model` | WASM Component Model support | +1.2MB |
| `multi-tenant` | Multi-tenant isolation features | +0.3MB |

## WebAssembly Toolchain Setup

### Core Tools

```bash
# Essential WebAssembly tools
rustup target add wasm32-wasi          # WASI target
rustup target add wasm32-unknown-unknown  # Core WASM target

# Development tools
cargo install wasm-pack                # WASM packaging
cargo install wasm-bindgen-cli         # JS bindings
cargo install wasm-opt                 # WASM optimization
```

### Advanced Tools

```bash
# Debugging and analysis
cargo install twiggy                   # WASM size profiler
cargo install wasm-objdump             # WASM disassembler
cargo install wasmprof                 # WASM profiler

# Component Model (experimental)
cargo install cargo-component          # Component tooling
cargo install wasm-tools               # Component analysis
```

### Tool Configuration

Create `~/.cargo/config.toml`:

```toml
[build]
target = "wasm32-wasi"

[target.wasm32-wasi]
runner = "wasmtime"

[target.wasm32-unknown-unknown]
runner = "node"

[env]
RUST_LOG = { value = "debug", relative = true }
WASM_INTERFACE_TYPES = { value = "1", relative = true }
```

## Docker Setup

### Development Container

```dockerfile
# Dockerfile.dev
FROM rust:1.75

# Install WebAssembly tools
RUN rustup target add wasm32-wasi wasm32-unknown-unknown
RUN cargo install wasm-pack wasm-opt twiggy

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /workspace

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Development setup
EXPOSE 3000 8080
CMD ["bash"]
```

Build and run:

```bash
docker build -f Dockerfile.dev -t wasm-sandbox-dev .
docker run -it -v $(pwd):/workspace wasm-sandbox-dev
```

### Production Container

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/my-app /usr/local/bin/my-app
CMD ["my-app"]
```

## CI/CD Setup

### GitHub Actions

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        targets: wasm32-wasi
        
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/bin/
          ~/.cargo/registry/index/
          ~/.cargo/registry/cache/
          ~/.cargo/git/db/
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        
    - name: Install WASM tools
      run: |
        cargo install wasm-pack
        cargo install wasm-opt
        
    - name: Run tests
      run: cargo test --all-features
      
    - name: Build WASM modules
      run: |
        cargo build --target wasm32-wasi --release
        wasm-opt -Oz target/wasm32-wasi/release/*.wasm -o optimized.wasm
        
    - name: Run clippy
      run: cargo clippy --all-features -- -D warnings
      
    - name: Check formatting
      run: cargo fmt -- --check
```

### GitLab CI

Create `.gitlab-ci.yml`:

```yaml
stages:
  - test
  - build

variables:
  CARGO_HOME: "$CI_PROJECT_DIR/.cargo"

cache:
  paths:
    - .cargo/
    - target/

before_script:
  - apt-get update && apt-get install -y build-essential pkg-config libssl-dev
  - rustup target add wasm32-wasi
  - cargo install wasm-pack wasm-opt

test:
  stage: test
  script:
    - cargo test --all-features
    - cargo clippy --all-features -- -D warnings
    - cargo fmt -- --check

build:
  stage: build
  script:
    - cargo build --target wasm32-wasi --release
    - wasm-opt -Oz target/wasm32-wasi/release/*.wasm -o optimized.wasm
  artifacts:
    paths:
      - optimized.wasm
    expire_in: 1 week
```

## Troubleshooting Installation

### Common Issues

#### 1. Linker Errors on Linux

```bash
# Install missing dependencies
sudo apt install build-essential pkg-config libssl-dev

# Or for older systems
sudo apt install gcc libc6-dev pkg-config libssl1.0-dev
```

#### 2. WASM Target Not Found

```bash
# Reinstall WASM targets
rustup target remove wasm32-wasi
rustup target add wasm32-wasi

# Verify installation
rustup target list --installed | grep wasm
```

#### 3. Permission Errors on Windows

```powershell
# Run PowerShell as Administrator
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Or use alternative installation
winget install Rustlang.Rust.MSVC
```

#### 4. macOS Compilation Issues

```bash
# Install Xcode command line tools
xcode-select --install

# Update macOS if needed
softwareupdate --install --recommended

# Install missing tools via Homebrew
brew install pkg-config openssl
```

### Verification Steps

```bash
# Verify Rust installation
rustc --version
cargo --version

# Verify WASM targets
rustup target list --installed

# Verify wasm-sandbox installation
cargo new test-sandbox
cd test-sandbox
cargo add wasm-sandbox
cargo check
```

### Performance Tuning

#### Compilation Speed

```toml
# Add to Cargo.toml
[profile.dev]
incremental = true
debug = 1

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
```

#### Runtime Performance

```bash
# Use faster linker
cargo install -f cargo-binutils
rustup component add llvm-tools-preview

# Enable LTO for dependencies
export CARGO_PROFILE_RELEASE_LTO=true
```

## Next Steps

After successful installation:

1. **[Basic Tutorial](basic-tutorial.md)** - Your first sandbox application
2. **[Security Configuration](security-config.md)** - Set up secure execution
3. **[Performance Guide](../design/performance.md)** - Optimize for your use case
4. **[Examples](examples.md)** - Explore real-world applications

---

**Installation Complete:** You're ready to build secure WebAssembly applications with wasm-sandbox!
