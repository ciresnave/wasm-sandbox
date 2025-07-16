# Documentation Index

This directory contains comprehensive documentation for wasm-sandbox. All documentation is organized by category for easy navigation.

## Quick Navigation

- **[Getting Started](#getting-started)** - New to wasm-sandbox? Start here
- **[API Reference](#api-reference)** - Detailed API documentation
- **[Design Documents](#design-documents)** - Architecture and design decisions
- **[Guides](#guides)** - Step-by-step tutorials and migration guides
- **[Examples](#examples)** - Working code examples
- **[Community](#community)** - Feedback, contributions, and discussions

## Getting Started

### New Users

1. **[README.md](../README.md)** - Project overview and quick start
2. **[Installation Guide](guides/installation.md)** - Detailed installation instructions
3. **[Basic Tutorial](guides/basic-tutorial.md)** - Your first sandbox application
4. **[Examples Overview](../examples/README.md)** - Working examples to learn from

### Migration

- **[Migration Guide](guides/MIGRATION.md)** - Upgrading from previous versions
- **[Breaking Changes](../CHANGELOG.md)** - Version-specific changes

## API Reference

### Core APIs

- **[API Overview](api/API.md)** - Complete API documentation
- **[Planned Improvements](api/API_IMPROVEMENTS.md)** - Upcoming API changes

### Generated Documentation

- **[docs.rs](https://docs.rs/wasm-sandbox)** - Always up-to-date API reference
- **Local docs**: Run `cargo doc --open` for offline documentation

## Design Documents

### Architecture

- **[Trait Design](design/TRAIT_DESIGN.md)** - Core trait architecture and patterns
- **[Generic Plugin System](design/GENERIC_PLUGIN_DESIGN.md)** - Plugin system design
- **[Ease of Use Improvements](design/EASE_OF_USE_IMPROVEMENTS.md)** - Progressive complexity and simple APIs
- **[Security Model](design/security-model.md)** - Security architecture (planned)
- **[Runtime Abstraction](design/runtime-abstraction.md)** - WebAssembly runtime layer (planned)

### Performance

- **[Performance Guide](design/performance.md)** - Optimization strategies (planned)
- **[Benchmarks](../benches/README.md)** - Performance measurements (planned)

## Guides

### Usage Patterns

- **[Basic Usage](guides/basic-usage.md)** - Common usage patterns (planned)
- **[Security Configuration](guides/security-config.md)** - Configuring capabilities and limits (planned)
- **[Resource Management](guides/resource-management.md)** - Memory, CPU, and I/O limits (planned)
- **[Error Handling](guides/error-handling.md)** - Best practices for error handling (planned)

### Advanced Topics

- **[Plugin Development](guides/plugin-development.md)** - Creating secure plugins (planned)
- **[Hot Reload](guides/hot-reload.md)** - Dynamic module updates (planned)
- **[Streaming Data](guides/streaming.md)** - Large data processing patterns (planned)
- **[Production Deployment](guides/production.md)** - Production considerations (planned)

### Integration Guides

- **[HTTP Servers](guides/http-servers.md)** - Building HTTP services (planned)
- **[CLI Tools](guides/cli-tools.md)** - Command-line applications (planned)
- **[MCP Servers](guides/mcp-servers.md)** - Model Context Protocol integration (planned)

## Examples

All examples are in the [`examples/`](../examples/) directory:

- **[Basic Usage](../examples/basic_usage.rs)** - Simple function calling
- **[File Processor](../examples/file_processor.rs)** - Secure file processing
- **[HTTP Server](../examples/http_server.rs)** - Web server in sandbox
- **[MCP Server](../examples/mcp_server.rs)** - Model Context Protocol server
- **[CLI Wrapper](../examples/cli_wrapper.rs)** - Command-line tool wrapper
- **[Plugin Ecosystem](../examples/plugin_ecosystem.rs)** - Generic plugin system

See [`examples/README.md`](../examples/README.md) for detailed descriptions and usage instructions.

## Community

### Feedback and Discussions

- **[PUP Integration Feedback](feedback/PUP_FEEDBACK_RESPONSE.md)** - Real-world integration experience
- **[GitHub Discussions](https://github.com/ciresnave/wasm-sandbox/discussions)** - Community discussions
- **[GitHub Issues](https://github.com/ciresnave/wasm-sandbox/issues)** - Bug reports and feature requests

### Contributing

- **[Contributing Guide](../CONTRIBUTING.md)** - How to contribute to the project
- **[Code of Conduct](../CODE_OF_CONDUCT.md)** - Community guidelines (planned)
- **[Development Setup](guides/development.md)** - Setting up development environment (planned)

### Releases

- **[Changelog](../CHANGELOG.md)** - Version history and changes
- **[Roadmap](design/roadmap.md)** - Future development plans (planned)

## Documentation for Different Audiences

### For Application Developers

- Start with [README.md](../README.md) and [Basic Tutorial](guides/basic-tutorial.md)
- Check [examples/](../examples/) for patterns matching your use case
- Refer to [docs.rs](https://docs.rs/wasm-sandbox) for API details

### For Plugin Developers  

- Read [Generic Plugin Design](design/GENERIC_PLUGIN_DESIGN.md)
- Follow [Plugin Development Guide](guides/plugin-development.md) (planned)
- Study [Plugin Ecosystem Example](../examples/plugin_ecosystem.rs)

### For Contributors

- Read [Contributing Guide](../CONTRIBUTING.md)
- Understand [Trait Design](design/TRAIT_DESIGN.md)
- Check [API Improvements](api/API_IMPROVEMENTS.md) for development priorities

### For Security Auditors

- Review [Security Model](design/security-model.md) (planned)
- Check [Security Configuration Guide](guides/security-config.md) (planned)
- Examine capability system in [API docs](https://docs.rs/wasm-sandbox)

## Documentation Standards

All documentation follows these standards:

- **Markdown format** with consistent formatting
- **Code examples** that compile and run
- **Links** to related documentation
- **Table of contents** for long documents
- **Clear structure** with headers and sections

## Feedback

Found missing documentation or have suggestions?

- **[Open an issue](https://github.com/ciresnave/wasm-sandbox/issues)** for documentation bugs
- **[Start a discussion](https://github.com/ciresnave/wasm-sandbox/discussions)** for documentation improvements
- **[Submit a PR](https://github.com/ciresnave/wasm-sandbox/pulls)** to contribute documentation

---

*Last updated: July 2025*
