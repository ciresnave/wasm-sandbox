# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-11

### Added

- **Comprehensive trait-based architecture** with dyn-compatible core traits and extension traits
- **Extension traits** for async and generic operations (`WasmInstanceExt`, `WasmRuntimeExt`, `RpcChannelExt`)
- **Downcasting support** via `as_any()` method on all trait objects
- **Enhanced test suite** with 15 tests covering integration and trait structure validation
- **Complete trait design documentation** (`TRAIT_DESIGN.md`)
- **Benchmark suite** for performance validation
- **Production-grade clippy compliance** with appropriate allow directives

### Improved

- **Code quality**: Eliminated all compiler warnings (20 → 0)
- **Architecture**: Refactored to support both concrete types and trait objects
- **Documentation**: Enhanced with comprehensive trait usage patterns
- **Testing**: Added trait structure tests ensuring dyn-compatibility
- **API design**: Better separation between core and advanced functionality

### Fixed

- **Async function warnings** in extension traits (intentionally allowed for internal APIs)
- **Unused variable warnings** throughout codebase
- **Dead code warnings** for API-design fields

### Changed

- **Breaking**: Core traits are now dyn-compatible (generic methods moved to extension traits)
- **API**: Advanced async/generic operations now require importing extension traits
- **Architecture**: Runtime abstraction now supports trait object usage patterns

### Technical Details

- **Trait Objects**: All core traits (`WasmRuntime`, `WasmInstance`, `WasmModule`) support `dyn` usage
- **Extension Pattern**: Advanced features available through `*Ext` traits
- **Migration Path**: Existing concrete type usage continues to work seamlessly
- **Performance**: Zero-cost abstractions maintained for direct usage

## [Unreleased]

## [0.1.0] - 2024-01-XX

### Initial Release

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
