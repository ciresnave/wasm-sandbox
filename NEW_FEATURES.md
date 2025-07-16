# New Features in v0.4.0

This document summarizes the major new features added in the v0.4.0 release of wasm-sandbox.

## 🧩 WebAssembly Component Model

The WebAssembly Component Model represents the next evolution of WebAssembly, focusing on modularity, composition, and language interoperability.

- **Full Documentation**: [Component Model Guide](./docs/guides/COMPONENT_MODEL.md)
- **Example**: [component_model.rs](./examples/component_model.rs)
- **Feature Flag**: `component-model`

Key capabilities:

- Rich interface types beyond basic WebAssembly types
- Composition of components with clean interfaces
- Multi-language support with interface-first design
- Enhanced security through interface-based access controls

## 🐍 Python Language Bindings

Use wasm-sandbox directly from Python with a native API that mirrors the Rust interface.

- **Full Documentation**: [Python Bindings Guide](./docs/bindings/PYTHON_BINDINGS.md)
- **Example**: [python_bindings.rs](./examples/python_bindings.rs)
- **Feature Flag**: `python-bindings`

Key capabilities:

- Complete Python API matching the Rust API
- Seamless type conversion between Python and WebAssembly
- Asyncio support for non-blocking operations
- Simple installation via pip: `pip install wasm-sandbox-py`

## 📊 Streaming APIs for Large Data

Process datasets larger than available memory with efficient streaming interfaces.

- **Full Documentation**: [Streaming APIs Guide](./docs/guides/STREAMING_APIS.md)
- **Example**: [streaming_data.rs](./examples/streaming_data.rs)
- **Feature Flag**: `streaming-apis`

Key capabilities:

- Memory-efficient processing of large datasets
- Both memory and file-based streaming channels
- Zero-copy optimization where possible
- Transformation pipelines with minimal overhead
- Built-in backpressure handling

## Using These Features

Enable these features in your Cargo.toml:

```toml
[dependencies]
wasm-sandbox = { version = "0.4.0", features = ["component-model", "python-bindings", "streaming-apis"] }
```

Or individually as needed:

```toml
[dependencies]
wasm-sandbox = { version = "0.4.0", features = ["streaming-apis"] }
```
