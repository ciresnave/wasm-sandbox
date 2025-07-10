# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-01-XX

### Added
- Initial release of `wasm-sandbox` crate
- Support for Wasmtime and Wasmer WebAssembly runtimes
- Dyn-compatible trait system for runtime abstraction
- Capability-based security model for fine-grained access control
- Resource limits and monitoring (memory, CPU, I/O)
- Flexible host-guest communication channels
- Support for JSON and MessagePack serialization
- Application wrappers for HTTP servers, MCP servers, and CLI tools
- Comprehensive examples and documentation
- Async/await support with tokio
- Benchmarks for performance testing
- Memory-based communication channels
- RPC abstraction layer
- Template system for code generation
- WASI support for filesystem and network operations
- Error handling and logging infrastructure

### Features
- **Security**: Sandbox isolation with configurable capabilities
- **Performance**: Efficient host-guest communication (>1GB/s throughput)
- **Flexibility**: Multiple runtime backends and extensible architecture
- **Ease of Use**: High-level APIs with sensible defaults
- **Monitoring**: Resource usage tracking and limits
- **Async Support**: Full async/await compatibility with tokio

### Dependencies
- `wasmtime` (13.0.0) - Primary WebAssembly runtime
- `wasmer` (3.1.1) - Optional secondary WebAssembly runtime
- `tokio` (1.32.0) - Async runtime
- `serde` (1.0.188) - Serialization framework
- `tracing` (0.1.37) - Logging and instrumentation
- `anyhow` (1.0.75) - Error handling
- `cap-std` (2.0.0) - Capability-based security

### Documentation
- Complete API documentation
- Usage examples for all major features
- Architecture overview
- Security guidelines
- Performance benchmarks
- Contributing guidelines
