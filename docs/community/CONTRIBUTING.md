# Contributing to wasm-sandbox

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)**

Thank you for your interest in contributing to wasm-sandbox! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation Guidelines](#documentation-guidelines)
- [Issue Reporting Guidelines](#issue-reporting-guidelines)
- [Feature Request Guidelines](#issue-reporting-guidelines)
- [Security Vulnerability Reporting](#security-vulnerability-reporting)
- [Community Resources](#community-resources)

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](GUIDELINES.md#code-of-conduct). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

### Finding Issues to Work On

- Check the [GitHub issues](https://github.com/your-org/wasm-sandbox/issues) for tasks labeled with `good first issue` or `help wanted`
- Look at the [roadmap](../design/roadmap.md) to see what features are planned
- Join the community discussion on [Discord](https://discord.gg/your-discord-invite)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:

   ```bash
   git clone https://github.com/your-username/wasm-sandbox.git
   cd wasm-sandbox
   ```

3. Add the original repository as an upstream remote:

   ```bash
   git remote add upstream https://github.com/your-org/wasm-sandbox.git
   ```

## Development Environment

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable, minimum version 1.65)
- [WebAssembly toolchain](https://rustwasm.github.io/wasm-pack/installer/)
- [cargo-make](https://github.com/sagiegurari/cargo-make) (optional but recommended)

### Setting Up the Development Environment

1. Install the dependencies:

   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Add WebAssembly target
   rustup target add wasm32-wasi
   rustup target add wasm32-unknown-unknown
   
   # Install wasm-pack
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   
   # Install cargo-make
   cargo install cargo-make
   ```

2. Build the project:

   ```bash
   # Using cargo
   cargo build
   
   # Using cargo-make
   cargo make build
   ```

3. Run the tests:

   ```bash
   # Using cargo
   cargo test
   
   # Using cargo-make
   cargo make test
   ```

### Development Commands

```bash
# Build the project
cargo build

# Run all tests
cargo test

# Run specific tests
cargo test communication

# Run benchmarks
cargo bench

# Check code formatting
cargo fmt --check

# Apply code formatting
cargo fmt

# Run clippy lints
cargo clippy

# Build documentation
cargo doc --no-deps --open

# Clean build artifacts
cargo clean
```

## Pull Request Process

### Branch Naming

Use descriptive branch names with a prefix indicating the type of change:

- `feature/` for new features
- `fix/` for bug fixes
- `docs/` for documentation changes
- `refactor/` for code refactoring
- `test/` for adding or modifying tests
- `chore/` for maintenance tasks

Example: `feature/add-wasmer-backend` or `fix/memory-leak-in-channels`

### Development Workflow

1. Ensure you're working from the latest code:

   ```bash
   git checkout main
   git pull upstream main
   ```

2. Create a new branch:

   ```bash
   git checkout -b feature/your-feature-name
   ```

3. Make your changes and commit them with descriptive messages:

   ```bash
   git add .
   git commit -m "feat: add support for custom communication channels"
   ```

4. Push your branch to your fork:

   ```bash
   git push origin feature/your-feature-name
   ```

5. Create a pull request from your fork to the main repository

### Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification for commit messages:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Types include:

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools

Examples:

- `feat(runtime): add support for Wasmer runtime`
- `fix(security): properly validate filesystem capabilities`
- `docs: improve installation instructions`

### Pull Request Template

When you submit a pull request, please include:

1. A description of the changes
2. Any related issues that are fixed (e.g., "Fixes #123")
3. A summary of the approach taken
4. Any additional context that might be helpful
5. Screenshots or code samples, if applicable

### Code Review Process

1. Maintainers will review your PR and provide feedback
2. Address any requested changes and push to your branch
3. Once approved, a maintainer will merge your PR
4. Your changes will be included in the next release

## Coding Standards

### Rust Code Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` to format code (run `cargo fmt` before committing)
- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/README.html)
- Use `clippy` to catch common mistakes (run `cargo clippy` before committing)

### API Design Principles

- Make APIs hard to misuse
- Follow the principle of least surprise
- Provide both high-level and low-level APIs
- Maintain backward compatibility
- Document public APIs thoroughly

### Error Handling

- Use `Result<T, Error>` for operations that can fail
- Create specific error types for different error conditions
- Provide helpful error messages
- Avoid panicking in library code

### Commenting Guidelines

- Use [rustdoc](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html) comments for public APIs
- Include examples in documentation
- Document all public functions, types, and modules
- Explain complex code with inline comments

Example:

```rust
/// Executes a WebAssembly function with the given arguments.
///
/// # Arguments
///
/// * `function_name` - The name of the function to execute
/// * `args` - The arguments to pass to the function
///
/// # Returns
///
/// The function result or an error if execution failed
///
/// # Examples
///
/// ```
/// use wasm_sandbox::Sandbox;
///
/// let sandbox = Sandbox::new("path/to/module.wasm").unwrap();
/// let result: i32 = sandbox.call("add", &[1, 2]).unwrap();
/// assert_eq!(result, 3);
/// ```
pub fn call<T, R>(&self, function_name: &str, args: &T) -> Result<R>
where
    T: Serialize + ?Sized,
    R: for<'de> Deserialize<'de>,
{
    // Implementation
}
```

## Testing Requirements

### Test Types

- **Unit tests**: Test individual functions and methods
- **Integration tests**: Test complete features and components
- **Security boundary tests**: Test security isolation and capabilities
- **Performance tests**: Benchmarks for critical operations
- **Documentation tests**: Ensure examples in documentation work

### Test Coverage

- Aim for high test coverage, especially for security-critical code
- All new features should include tests
- Bug fixes should include tests that reproduce the issue

### Writing Tests

Unit tests should be placed in the same file as the code they test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_function() {
        // Test implementation
    }
}
```

Integration tests should be placed in the `tests/` directory:

```rust
// tests/integration_test.rs
use wasm_sandbox::{Sandbox, Error};

#[test]
fn test_sandbox_creation() {
    let sandbox = Sandbox::new("fixtures/test_module.wasm").unwrap();
    // Test assertions
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with logs
RUST_LOG=debug cargo test

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test
```

## Documentation Guidelines

### Code Documentation

- Document all public APIs with rustdoc comments
- Include examples for non-trivial functions
- Explain the purpose and behavior of types and functions
- Document error conditions and panics

### User Documentation

- Place user documentation in the `docs/` directory
- Use Markdown for documentation files
- Include examples for common use cases
- Provide troubleshooting information

### Project Documentation

- Update README.md with significant changes
- Keep CHANGELOG.md up to date
- Document design decisions in `docs/design/`
- Add usage examples to `docs/examples/`

### Documentation Structure

- **API Documentation**: Generated from code comments
- **User Guides**: Step-by-step instructions for users
- **Reference Documentation**: Detailed information about features
- **Examples**: Practical usage examples
- **Design Documents**: Architectural and design decisions

## Issue Reporting Guidelines

### Bug Reports

When reporting a bug, please include:

1. A clear, descriptive title
2. The exact steps to reproduce the issue
3. The expected behavior
4. The actual behavior
5. Your environment details (OS, Rust version, etc.)
6. Relevant logs or error messages
7. A minimal code example if possible

### Feature Requests

When requesting a new feature, please include:

1. A clear, descriptive title
2. A detailed description of the proposed feature
3. The problem it solves
4. Example use cases
5. Any alternatives you've considered
6. Any references or prior art

### Issue Labels

- `bug`: Something isn't working as expected
- `enhancement`: New feature or improvement
- `documentation`: Documentation improvements
- `good first issue`: Good for newcomers
- `help wanted`: Extra attention needed
- `security`: Security-related issues
- `performance`: Performance improvements

## Security Vulnerability Reporting

Security vulnerabilities should be reported privately to the maintainers:

1. Email <security@your-org.com> with details of the vulnerability
2. Do not disclose the vulnerability publicly until it has been addressed
3. Include steps to reproduce, impact assessment, and any potential mitigations

## Community Resources

- **GitHub Issues**: For bug reports and feature requests
- **Discussions**: For questions and community support
- **Discord**: For real-time chat and community building
- **Website**: For official documentation and releases
- **Twitter**: For announcements and updates

---

Thank you for contributing to wasm-sandbox! Your efforts help improve the project for everyone.
