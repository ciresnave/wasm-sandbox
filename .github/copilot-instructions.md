# Wasm Sandbox - Secure WebAssembly Runtime for Untrusted Code

## Project Overview

This project, `wasm-sandbox`, is a Rust crate that provides secure WebAssembly-based sandboxing for untrusted code execution. The goal is to create a high-level, user-friendly API for running untrusted code in a secure environment with flexible host-guest communication and comprehensive resource limits.

## Architecture

The crate is organized into several key modules:

- **Runtime**: Abstracts over different WebAssembly runtimes (Wasmtime, Wasmer)
- **Security**: Handles capabilities, resource limits, and security auditing
- **Communication**: Manages host-guest communication through various channels
- **Wrappers**: Provides code generators for common application types
- **Compiler**: Compiles Rust projects to WebAssembly
- **Templates**: Renders templates for wrappers
- **Utils**: Utility functions, manifest parsing, logging, etc.

## Key Features

1. **Multiple Runtime Support**: Abstract over different WebAssembly runtimes
2. **Security Controls**: Fine-grained capability-based security model
3. **Resource Limits**: Memory, CPU, I/O, and network limits
4. **Host-Guest Communication**: Multiple communication channels
5. **Application Wrappers**: Generate code for HTTP servers, MCP servers, CLI tools
6. **Manifest Support**: Declarative configuration
7. **Type-safe API**: Serialize/deserialize function parameters and return values

## Development Guidelines

When working on this project:

1. **Security First**: All features must prioritize security by default
2. **Flexible APIs**: Provide both high-level and low-level APIs
3. **Error Handling**: Comprehensive error types and helpful messages
4. **Documentation**: Well-documented public API with examples
5. **Testing**: Unit tests, integration tests, and security boundary tests
6. **Performance**: Minimize overhead, especially for host-guest communication

## Key Concepts

- **Sandbox**: The main controller that manages WebAssembly modules and instances
- **Module**: A compiled WebAssembly module
- **Instance**: A runtime instance of a WebAssembly module
- **Capabilities**: Permissions granted to an instance
- **Resource Limits**: Constraints on resource usage
- **Communication Channels**: Methods for host-guest communication

## Code Style

- Follow Rust idioms and naming conventions
- Use the builder pattern for complex configurations
- Prefer strong typing over stringly-typed interfaces
- Use async/await for I/O operations
- Provide both sync and async APIs where appropriate
- Use generics and traits for flexibility
- Minimize unsafe code and document when used
- Use feature flags for optional functionality

## Future Directions

- WebAssembly Component Model support
- Advanced caching and optimization
- JIT compilation
- More runtime backends
- Language bindings for Python, JavaScript, etc.
- More application templates
- Advanced security features (seccomp, syscall filtering)
- Streaming APIs for large data handling
