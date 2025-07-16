# Python Language Bindings

The `wasm-sandbox` library provides Python language bindings to allow Python developers to leverage the power of WebAssembly sandboxing in their applications.

## Installation

### From PyPI (Coming Soon)

```bash
pip install wasm-sandbox
```

### From Source

```bash
# Clone the repository
git clone https://github.com/ciresnave/wasm-sandbox.git
cd wasm-sandbox

# Generate Python bindings
cargo run --example python_bindings

# Build and install the Python package
cd target/bindings/python_c_bindings
./build.sh
cd ../python
pip install -e .
```

## Quick Start

### Running Code in a Sandbox

```python
from wasm_sandbox import run

# One-line execution
result = run("calculator.py", "add", [5, 3])
print(f"5 + 3 = {result}")  # Output: 5 + 3 = 8
```

### With Timeout Protection

```python
from wasm_sandbox import run_with_timeout

# Run with a 30-second timeout
result = run_with_timeout(
    "processor.py", 
    "process", 
    {"data": "Hello, world!"}, 
    timeout_seconds=30
)
print(f"Result: {result}")
```

### Builder Pattern for Custom Configuration

```python
from wasm_sandbox import WasmSandbox

sandbox = WasmSandbox.builder() \
    .source("my_program.py") \
    .timeout_duration(60) \
    .memory_limit(64 * 1024 * 1024) \
    .enable_file_access(False) \
    .enable_network(False) \
    .build()

# Call multiple functions
add_result = sandbox.call("add", [10, 20])
print(f"10 + 20 = {add_result}")  # Output: 10 + 20 = 30

subtract_result = sandbox.call("subtract", [20, 5])
print(f"20 - 5 = {subtract_result}")  # Output: 20 - 5 = 15
```

## Security Controls

### Configuring Resource Limits

```python
from wasm_sandbox import WasmSandbox, MemoryLimits, CpuLimits

# Create a sandbox with custom resource limits
sandbox = WasmSandbox.builder() \
    .source("my_program.py") \
    .memory_limit(32 * 1024 * 1024)  # 32 MB memory limit \
    .cpu_limit(10000)  # 10 seconds max execution time \
    .build()
```

### Managing Capabilities

```python
from wasm_sandbox import WasmSandbox, Capabilities, FilesystemCapability, NetworkCapability

# Configure security capabilities
sandbox = WasmSandbox.builder() \
    .source("my_program.py") \
    .enable_file_access(True) \
    .enable_network(False) \
    .build()

# With more detailed control
capabilities = Capabilities(
    filesystem=FilesystemCapability(
        readable_dirs=["/tmp/data"],
        writable_dirs=["/tmp/output"],
        allow_create=True,
        allow_delete=False
    ),
    network=NetworkCapability(
        allow_outbound=True,
        allow_inbound=False,
        allowed_hosts=["api.example.com"],
        allowed_ports=[443]
    )
)

sandbox = WasmSandbox.builder() \
    .source("my_program.py") \
    .with_capabilities(capabilities) \
    .build()
```

## Error Handling

```python
from wasm_sandbox import run

try:
    result = run("calculator.py", "divide", [10, 0])
    print(result)
except Exception as e:
    print(f"Error executing code: {e}")
```

## Advanced Usage

### Loading WebAssembly Directly

```python
import wasm_sandbox

# Create a sandbox
sandbox = wasm_sandbox.WasmSandbox()

# Load a WebAssembly module from file
with open("my_module.wasm", "rb") as f:
    wasm_bytes = f.read()
    
module_id = sandbox.load_module(wasm_bytes)
instance_id = sandbox.create_instance(module_id)

# Call a function
result = sandbox.call(instance_id, "process_data", {"input": "test data"})
print(result)
```

### Asynchronous Execution (Future)

```python
import asyncio
from wasm_sandbox.async_api import run_async

async def main():
    result = await run_async("processor.py", "process", {"data": "test"})
    print(f"Result: {result}")

asyncio.run(main())
```

## Compatibility

- Python 3.8+
- Platforms: Windows, macOS, Linux
- Dependencies: CFFI 1.15.0+, Pydantic 2.0.0+

## Performance Considerations

- The Python bindings add a small overhead compared to using the Rust API directly
- For high-performance applications, consider using the Rust API
- For most applications, the convenience of the Python API outweighs the small performance cost

## Examples

See the [examples directory](../../examples/python/) for more complete examples.
