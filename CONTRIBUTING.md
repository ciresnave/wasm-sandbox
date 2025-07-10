# Contributing to wasm-sandbox

We welcome contributions to the `wasm-sandbox` project! This document provides guidelines for contributing.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/wasm-sandbox.git
   cd wasm-sandbox
   ```
3. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo
- Git

### Building

```bash
# Build the project
cargo build

# Build with all features
cargo build --all-features

# Run tests
cargo test --all-features

# Run benchmarks
cargo bench

# Check code style
cargo clippy --all-features
cargo fmt --check
```

### Testing

Before submitting a pull request, make sure all tests pass:

```bash
# Run all tests
cargo test --all-features

# Run integration tests
cargo test --test integration_test

# Run trait structure tests
cargo test --test trait_structure_test

# Run benchmarks
cargo bench
```

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use `cargo clippy` to catch common issues
- Add documentation for all public APIs
- Include examples in documentation where appropriate
- Write tests for new functionality

## Pull Request Process

1. **Update documentation** if you're changing APIs
2. **Add tests** for new functionality
3. **Run the full test suite** to ensure nothing is broken
4. **Update the changelog** if your changes are user-facing
5. **Create a pull request** with a clear description of your changes

### Pull Request Checklist

- [ ] Code follows the project's style guidelines
- [ ] All tests pass (`cargo test --all-features`)
- [ ] New tests are added for new functionality
- [ ] Documentation is updated for API changes
- [ ] Changelog is updated for user-facing changes
- [ ] Benchmarks still pass (`cargo bench`)
- [ ] No new warnings from `cargo clippy`

## Issues

When reporting issues:

1. **Use the issue template** if available
2. **Provide minimal reproduction** case
3. **Include relevant system information** (OS, Rust version, etc.)
4. **Check existing issues** to avoid duplicates

## Security

For security-related issues, please email the maintainers directly rather than opening a public issue.

## Code of Conduct

This project adheres to the Rust Code of Conduct. By participating, you are expected to uphold this code.

## License

By contributing to this project, you agree that your contributions will be licensed under the MIT License.

## Architecture Overview

Before contributing, please review:

- [`TRAIT_DESIGN.md`](TRAIT_DESIGN.md) - Trait architecture and design decisions
- [`API.md`](API.md) - API documentation and usage examples
- [`README.md`](README.md) - Project overview and quick start

## Areas for Contribution

We welcome contributions in these areas:

- **Runtime backends**: Additional WebAssembly runtime support
- **Security features**: Enhanced sandbox isolation and security
- **Performance optimizations**: Faster host-guest communication
- **Documentation**: Better examples and guides
- **Testing**: More comprehensive test coverage
- **Benchmarks**: Performance regression detection

## Questions?

If you have questions about contributing, feel free to:

- Open an issue for discussion
- Check existing documentation
- Look at existing code for patterns

Thank you for your interest in contributing to wasm-sandbox!
