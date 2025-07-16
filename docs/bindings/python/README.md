# Python Bindings

📖 **[← Back to Documentation](../../README.md)** | 🏠 **[← Main README](../../../README.md)** | 🐍 **[PyPI Package](https://pypi.org/project/wasm-sandbox-py/)**

Native Python bindings for wasm-sandbox, enabling secure WebAssembly execution in Python applications with full asyncio support.

## Installation

### Using pip

```bash
# Install from PyPI
pip install wasm-sandbox-py

# Install with async extras
pip install wasm-sandbox-py[async]

# Install with all extras
pip install wasm-sandbox-py[async,numpy,pandas]
```

### Using conda

```bash
conda install -c conda-forge wasm-sandbox-py
```

### From Source

```bash
git clone https://github.com/your-org/wasm-sandbox-py
cd wasm-sandbox-py
pip install -e .
```

## Quick Start

### Basic Usage

```python
import asyncio
from wasm_sandbox import WasmSandbox, SecurityPolicy

async def main():
    # Create a secure sandbox
    sandbox = await WasmSandbox.builder() \
        .source('./calculator.wasm') \
        .security_policy(SecurityPolicy.strict()) \
        .memory_limit(64 * 1024 * 1024) \
        .cpu_timeout(30.0) \
        .build()

    # Call a function
    result = await sandbox.call('add', [5, 3])
    print(f'Result: {result}')  # 8

    # Cleanup
    await sandbox.dispose()

if __name__ == '__main__':
    asyncio.run(main())
```

### Synchronous API

```python
from wasm_sandbox.sync import WasmSandbox, SecurityPolicy

# Synchronous version (uses asyncio internally)
sandbox = WasmSandbox.builder() \
    .source('./calculator.wasm') \
    .security_policy(SecurityPolicy.strict()) \
    .build()

result = sandbox.call('add', [5, 3])
print(f'Result: {result}')  # 8

sandbox.dispose()
```

## API Reference

### WasmSandbox Class

#### Builder Pattern

```python
from wasm_sandbox import WasmSandbox, SecurityPolicy, Capability

# Basic builder
sandbox = await WasmSandbox.builder() \
    .source('./module.wasm') \
    .build()

# Advanced configuration
sandbox = await WasmSandbox.builder() \
    .source('./module.wasm') \
    .security_policy(SecurityPolicy.moderate()) \
    .memory_limit(128 * 1024 * 1024) \
    .cpu_timeout(60.0) \
    .add_capability(Capability.network_access(['api.example.com'])) \
    .add_capability(Capability.filesystem_access(['/tmp'], read_only=True)) \
    .instance_id('worker-1') \
    .build()

# From different sources
sandbox_from_bytes = await WasmSandbox.from_bytes(wasm_bytes)
sandbox_from_url = await WasmSandbox.from_url('https://example.com/module.wasm')
sandbox_from_file = await WasmSandbox.from_file('./module.wasm')
```

#### Function Execution

```python
# Simple function call
result = await sandbox.call('calculate', [10, 20])

# Typed function call with type hints
from typing import Dict, Any

result: Dict[str, Any] = await sandbox.call('complex_calculation', [10, 20])

# Function call with timeout
result = await sandbox.call_with_timeout('slow_function', [], timeout=5.0)

# Batch function calls
results = await sandbox.call_batch([
    {'function': 'add', 'args': [1, 2]},
    {'function': 'multiply', 'args': [3, 4]},
    {'function': 'divide', 'args': [10, 2]}
])

# Function call with audit logging
audit_result = await sandbox.call_with_audit('sensitive_operation', [data])
print(f'Violations: {audit_result.violations}')
print(f'Events: {audit_result.events}')
print(f'Result: {audit_result.output}')
```

#### Resource Management

```python
# Memory management
memory_usage = await sandbox.memory_usage()
print(f'Memory used: {memory_usage / 1024 / 1024:.2f} MB')

# Garbage collection
await sandbox.collect_garbage()

# Resource monitoring
stats = await sandbox.get_resource_stats()
print(f'CPU time: {stats.cpu_time_ms}ms')
print(f'Peak memory: {stats.peak_memory_bytes} bytes')
print(f'Function calls: {stats.function_calls}')

# Reset sandbox state
await sandbox.reset()

# Cleanup
await sandbox.dispose()
```

### Security Policies

#### Predefined Policies

```python
from wasm_sandbox import SecurityPolicy

# Strict security (default)
strict_policy = SecurityPolicy.strict()

# Moderate security
moderate_policy = SecurityPolicy.moderate()

# Permissive security
permissive_policy = SecurityPolicy.permissive()

# Custom policy
custom_policy = SecurityPolicy.builder() \
    .allow_network_access(['api.trusted.com', 'cdn.example.com']) \
    .allow_filesystem_access(['/data'], read_only=True) \
    .allow_environment_access(['HOME', 'USER']) \
    .enable_audit_logging(True) \
    .max_execution_time(30.0) \
    .build()
```

#### Capabilities

```python
from wasm_sandbox import Capability

# Network access
network_capability = Capability.network_access(
    domains=['api.example.com', 'cdn.example.com'],
    allowed_ports=[80, 443],
    require_https=True,
    max_request_size=1024 * 1024  # 1MB
)

# File system access
fs_capability = Capability.filesystem_access(
    paths=['/tmp', '/data/readonly'],
    read_only=True,
    max_file_size=10 * 1024 * 1024  # 10MB
)

# Environment access
env_capability = Capability.environment_access([
    'HOME',
    'USER',
    'API_KEY'
])

# Apply capabilities
sandbox = await WasmSandbox.builder() \
    .source('./module.wasm') \
    .add_capability(network_capability) \
    .add_capability(fs_capability) \
    .add_capability(env_capability) \
    .build()
```

## Framework Integrations

### Django Integration

```python
# settings.py
INSTALLED_APPS = [
    # ... other apps
    'wasm_sandbox.django',
]

WASM_SANDBOX = {
    'DEFAULT_POLICY': 'strict',
    'MODULE_PATH': BASE_DIR / 'wasm_modules',
    'POOL_SIZE': 10,
    'MEMORY_LIMIT': 64 * 1024 * 1024,
    'CPU_TIMEOUT': 30.0,
}

# views.py
from django.http import JsonResponse
from django.views.decorators.csrf import csrf_exempt
from wasm_sandbox.django import get_sandbox_pool
import json

@csrf_exempt
async def process_request(request):
    if request.method != 'POST':
        return JsonResponse({'error': 'Method not allowed'}, status=405)
    
    try:
        data = json.loads(request.body)
        
        # Get sandbox from pool
        async with get_sandbox_pool().acquire() as sandbox:
            result = await sandbox.call('process_data', [data])
            
        return JsonResponse({'result': result, 'success': True})
    
    except Exception as e:
        return JsonResponse({'error': str(e), 'success': False}, status=500)

# urls.py
from django.urls import path
from . import views

urlpatterns = [
    path('process/', views.process_request, name='process'),
]
```

### FastAPI Integration

```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from wasm_sandbox import WasmSandbox, SecurityPolicy
from typing import Any, Dict
import asyncio

app = FastAPI()

# Sandbox pool for handling requests
class SandboxPool:
    def __init__(self, size: int = 10):
        self.size = size
        self.sandboxes = []
        self.available = asyncio.Queue()
        
    async def initialize(self):
        for i in range(self.size):
            sandbox = await WasmSandbox.builder() \
                .source('./request-handler.wasm') \
                .security_policy(SecurityPolicy.moderate()) \
                .instance_id(f'pool-{i}') \
                .build()
            
            self.sandboxes.append(sandbox)
            await self.available.put(sandbox)
    
    async def acquire(self):
        return await self.available.get()
    
    async def release(self, sandbox: WasmSandbox):
        await self.available.put(sandbox)

# Global pool
pool = SandboxPool()

@app.on_event("startup")
async def startup_event():
    await pool.initialize()

@app.on_event("shutdown")
async def shutdown_event():
    for sandbox in pool.sandboxes:
        await sandbox.dispose()

class ProcessRequest(BaseModel):
    data: Dict[str, Any]
    function: str = 'process_data'

class ProcessResponse(BaseModel):
    result: Any
    success: bool
    error: str = None

@app.post("/process", response_model=ProcessResponse)
async def process_data(request: ProcessRequest):
    sandbox = await pool.acquire()
    
    try:
        result = await sandbox.call(request.function, [request.data])
        return ProcessResponse(result=result, success=True)
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
    
    finally:
        await pool.release(sandbox)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

### Flask Integration

```python
from flask import Flask, request, jsonify
from wasm_sandbox.flask import WasmSandboxFlask
import asyncio

app = Flask(__name__)

# Initialize WASM sandbox extension
wasm = WasmSandboxFlask(app)

# Configure sandbox
app.config['WASM_SANDBOX_MODULE'] = './request-handler.wasm'
app.config['WASM_SANDBOX_POLICY'] = 'moderate'
app.config['WASM_SANDBOX_POOL_SIZE'] = 5

@app.route('/process', methods=['POST'])
def process_request():
    try:
        data = request.get_json()
        
        # Use the sandbox pool from extension
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        
        try:
            result = loop.run_until_complete(
                wasm.call_function('process_data', [data])
            )
            return jsonify({'result': result, 'success': True})
        finally:
            loop.close()
    
    except Exception as e:
        return jsonify({'error': str(e), 'success': False}), 500

if __name__ == '__main__':
    app.run(debug=True)
```

### Celery Integration

```python
from celery import Celery
from wasm_sandbox import WasmSandbox, SecurityPolicy
import asyncio

app = Celery('wasm_tasks', broker='redis://localhost:6379')

# Shared sandbox configuration
SANDBOX_CONFIG = {
    'source': './worker-module.wasm',
    'security_policy': SecurityPolicy.strict(),
    'memory_limit': 32 * 1024 * 1024,
    'cpu_timeout': 60.0
}

@app.task
def process_data_sync(data):
    """Synchronous wrapper for async WASM execution"""
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    
    try:
        return loop.run_until_complete(process_data_async(data))
    finally:
        loop.close()

async def process_data_async(data):
    """Async WASM execution"""
    sandbox = await WasmSandbox.builder() \
        .source(SANDBOX_CONFIG['source']) \
        .security_policy(SANDBOX_CONFIG['security_policy']) \
        .memory_limit(SANDBOX_CONFIG['memory_limit']) \
        .cpu_timeout(SANDBOX_CONFIG['cpu_timeout']) \
        .build()
    
    try:
        result = await sandbox.call('process_data', [data])
        return result
    finally:
        await sandbox.dispose()

# Usage
from celery import group

# Process multiple items in parallel
job = group(process_data_sync.s(item) for item in data_items)
result = job.apply_async()
```

## Data Science Integration

### NumPy Integration

```python
import numpy as np
from wasm_sandbox import WasmSandbox
from wasm_sandbox.numpy import numpy_to_wasm, wasm_to_numpy

async def main():
    sandbox = await WasmSandbox.from_file('./math-module.wasm')
    
    # Convert NumPy array to WASM-compatible format
    data = np.array([[1, 2, 3], [4, 5, 6]], dtype=np.float32)
    wasm_data = numpy_to_wasm(data)
    
    # Call WASM function with NumPy data
    result_wasm = await sandbox.call('matrix_multiply', [wasm_data, wasm_data])
    
    # Convert result back to NumPy
    result = wasm_to_numpy(result_wasm)
    print(f'Result shape: {result.shape}')
    print(f'Result:\n{result}')
    
    await sandbox.dispose()

asyncio.run(main())
```

### Pandas Integration

```python
import pandas as pd
from wasm_sandbox import WasmSandbox
from wasm_sandbox.pandas import dataframe_to_wasm, wasm_to_dataframe

async def process_dataframe():
    sandbox = await WasmSandbox.from_file('./data-processor.wasm')
    
    # Create sample DataFrame
    df = pd.DataFrame({
        'id': [1, 2, 3, 4, 5],
        'value': [10.5, 20.3, 30.1, 40.8, 50.2],
        'category': ['A', 'B', 'A', 'C', 'B']
    })
    
    # Convert to WASM format
    wasm_data = dataframe_to_wasm(df)
    
    # Process in WASM
    processed_wasm = await sandbox.call('process_dataframe', [wasm_data])
    
    # Convert back to DataFrame
    processed_df = wasm_to_dataframe(processed_wasm)
    print(processed_df)
    
    await sandbox.dispose()

asyncio.run(process_dataframe())
```

### Jupyter Notebook Integration

```python
# Install in Jupyter
!pip install wasm-sandbox-py[jupyter]

# Import and setup
from wasm_sandbox.jupyter import WasmSandboxMagic
get_ipython().register_magic_function(WasmSandboxMagic)

# Use magic commands
%%wasm_sandbox --source ./calculator.wasm --policy strict
result = add(5, 3)
print(f"5 + 3 = {result}")

# Interactive widgets
from wasm_sandbox.jupyter import SandboxWidget

widget = SandboxWidget(source='./interactive-module.wasm')
widget.display()
```

## Advanced Features

### Connection Pooling

```python
from wasm_sandbox import SandboxPool, SecurityPolicy
import asyncio

class OptimizedSandboxPool:
    def __init__(self, min_size: int = 2, max_size: int = 10, 
                 source: str = None, security_policy = None):
        self.min_size = min_size
        self.max_size = max_size
        self.source = source
        self.security_policy = security_policy or SecurityPolicy.strict()
        
        self.available = asyncio.Queue()
        self.in_use = set()
        self.total_created = 0
        
    async def initialize(self):
        """Create minimum number of sandboxes"""
        for _ in range(self.min_size):
            sandbox = await self._create_sandbox()
            await self.available.put(sandbox)
    
    async def acquire(self) -> WasmSandbox:
        """Acquire a sandbox from the pool"""
        try:
            # Try to get available sandbox (non-blocking)
            sandbox = self.available.get_nowait()
            self.in_use.add(sandbox)
            return sandbox
        except asyncio.QueueEmpty:
            # Create new sandbox if under max limit
            if self.total_created < self.max_size:
                sandbox = await self._create_sandbox()
                self.in_use.add(sandbox)
                return sandbox
            
            # Wait for available sandbox
            sandbox = await self.available.get()
            self.in_use.add(sandbox)
            return sandbox
    
    async def release(self, sandbox: WasmSandbox):
        """Release a sandbox back to the pool"""
        self.in_use.discard(sandbox)
        
        # Reset sandbox state
        await sandbox.reset()
        
        # Return to pool if under min size, otherwise dispose
        if self.available.qsize() < self.min_size:
            await self.available.put(sandbox)
        else:
            await sandbox.dispose()
            self.total_created -= 1
    
    async def _create_sandbox(self) -> WasmSandbox:
        """Create a new sandbox instance"""
        sandbox = await WasmSandbox.builder() \
            .source(self.source) \
            .security_policy(self.security_policy) \
            .build()
        
        self.total_created += 1
        return sandbox
    
    async def dispose_all(self):
        """Dispose all sandboxes in the pool"""
        # Dispose available sandboxes
        while not self.available.empty():
            sandbox = await self.available.get()
            await sandbox.dispose()
        
        # Dispose in-use sandboxes
        for sandbox in self.in_use:
            await sandbox.dispose()
        
        self.in_use.clear()
        self.total_created = 0

# Usage with context manager
class PooledSandbox:
    def __init__(self, pool: OptimizedSandboxPool):
        self.pool = pool
        self.sandbox = None
    
    async def __aenter__(self):
        self.sandbox = await self.pool.acquire()
        return self.sandbox
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.sandbox:
            await self.pool.release(self.sandbox)

# Example usage
async def main():
    pool = OptimizedSandboxPool(
        min_size=2,
        max_size=10,
        source='./calculator.wasm'
    )
    
    await pool.initialize()
    
    try:
        # Use pool with context manager
        async with PooledSandbox(pool) as sandbox:
            result = await sandbox.call('add', [5, 3])
            print(f'Result: {result}')
        
        # Multiple concurrent operations
        async def worker(worker_id: int):
            async with PooledSandbox(pool) as sandbox:
                result = await sandbox.call('multiply', [worker_id, 2])
                return result
        
        tasks = [worker(i) for i in range(20)]
        results = await asyncio.gather(*tasks)
        print(f'Worker results: {results}')
    
    finally:
        await pool.dispose_all()

asyncio.run(main())
```

### Caching and Persistence

```python
import pickle
import hashlib
from pathlib import Path
from wasm_sandbox import WasmSandbox, SecurityPolicy

class SandboxCache:
    def __init__(self, cache_dir: str = './sandbox_cache'):
        self.cache_dir = Path(cache_dir)
        self.cache_dir.mkdir(exist_ok=True)
        self.memory_cache = {}
    
    def _get_cache_key(self, source: str, options: dict) -> str:
        """Generate cache key from source and options"""
        content = f"{source}:{str(sorted(options.items()))}"
        return hashlib.sha256(content.encode()).hexdigest()
    
    async def get_or_create(self, source: str, **options) -> WasmSandbox:
        """Get cached sandbox or create new one"""
        cache_key = self._get_cache_key(source, options)
        
        # Check memory cache first
        if cache_key in self.memory_cache:
            return self.memory_cache[cache_key]
        
        # Check disk cache
        cache_file = self.cache_dir / f"{cache_key}.pkl"
        if cache_file.exists():
            try:
                with open(cache_file, 'rb') as f:
                    cached_data = pickle.load(f)
                
                # Reconstruct sandbox from cached data
                sandbox = await WasmSandbox.builder() \
                    .source(cached_data['module_bytes']) \
                    .security_policy(cached_data['security_policy']) \
                    .memory_limit(cached_data.get('memory_limit')) \
                    .cpu_timeout(cached_data.get('cpu_timeout')) \
                    .build()
                
                self.memory_cache[cache_key] = sandbox
                return sandbox
            
            except Exception as e:
                print(f"Cache load failed: {e}")
                # Fall through to create new sandbox
        
        # Create new sandbox
        sandbox = await WasmSandbox.builder() \
            .source(source) \
            .security_policy(options.get('security_policy', SecurityPolicy.strict())) \
            .memory_limit(options.get('memory_limit')) \
            .cpu_timeout(options.get('cpu_timeout')) \
            .build()
        
        # Cache the sandbox data
        try:
            # Read module bytes for caching
            if isinstance(source, str) and source.endswith('.wasm'):
                with open(source, 'rb') as f:
                    module_bytes = f.read()
            else:
                module_bytes = source  # Assume it's already bytes
            
            cache_data = {
                'module_bytes': module_bytes,
                'security_policy': options.get('security_policy', SecurityPolicy.strict()),
                'memory_limit': options.get('memory_limit'),
                'cpu_timeout': options.get('cpu_timeout')
            }
            
            with open(cache_file, 'wb') as f:
                pickle.dump(cache_data, f)
        
        except Exception as e:
            print(f"Cache save failed: {e}")
        
        self.memory_cache[cache_key] = sandbox
        return sandbox
    
    def clear_memory_cache(self):
        """Clear in-memory cache"""
        for sandbox in self.memory_cache.values():
            asyncio.create_task(sandbox.dispose())
        self.memory_cache.clear()
    
    def clear_disk_cache(self):
        """Clear disk cache"""
        for cache_file in self.cache_dir.glob("*.pkl"):
            cache_file.unlink()

# Global cache instance
sandbox_cache = SandboxCache()

# Usage
async def main():
    # First call - creates and caches sandbox
    sandbox1 = await sandbox_cache.get_or_create(
        './calculator.wasm',
        security_policy=SecurityPolicy.strict(),
        memory_limit=64 * 1024 * 1024
    )
    
    # Second call - returns cached sandbox
    sandbox2 = await sandbox_cache.get_or_create(
        './calculator.wasm',
        security_policy=SecurityPolicy.strict(),
        memory_limit=64 * 1024 * 1024
    )
    
    assert sandbox1 is sandbox2  # Same instance
    
    result = await sandbox1.call('add', [5, 3])
    print(f'Result: {result}')

asyncio.run(main())
```

## Testing

### Pytest Integration

```python
# conftest.py
import pytest
import asyncio
from wasm_sandbox import WasmSandbox, SecurityPolicy

@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()

@pytest.fixture
async def test_sandbox():
    """Provide a test sandbox for each test function."""
    sandbox = await WasmSandbox.builder() \
        .source('./test-fixtures/test-module.wasm') \
        .security_policy(SecurityPolicy.permissive()) \
        .build()
    
    yield sandbox
    
    await sandbox.dispose()

@pytest.fixture
async def calculator_sandbox():
    """Specialized sandbox for calculator tests."""
    sandbox = await WasmSandbox.from_file('./test-fixtures/calculator.wasm')
    yield sandbox
    await sandbox.dispose()

# Helper functions
async def create_test_sandbox(source: str = './test-fixtures/test-module.wasm'):
    """Create a sandbox for testing with relaxed security."""
    return await WasmSandbox.builder() \
        .source(source) \
        .security_policy(SecurityPolicy.permissive()) \
        .build()
```

### Test Examples

```python
# test_sandbox.py
import pytest
import asyncio
from wasm_sandbox import WasmSandbox, SecurityPolicy, SecurityViolation

class TestWasmSandbox:
    
    @pytest.mark.asyncio
    async def test_basic_function_execution(self, calculator_sandbox):
        """Test basic function execution."""
        result = await calculator_sandbox.call('add', [5, 3])
        assert result == 8
        
        result = await calculator_sandbox.call('multiply', [4, 7])
        assert result == 28
    
    @pytest.mark.asyncio
    async def test_memory_limit_enforcement(self):
        """Test memory limit enforcement."""
        sandbox = await WasmSandbox.builder() \
            .source('./test-fixtures/memory-intensive.wasm') \
            .memory_limit(1024 * 1024) \
            .build()
        
        try:
            with pytest.raises(Exception, match="Memory limit exceeded"):
                await sandbox.call('allocate_large_buffer', [])
        finally:
            await sandbox.dispose()
    
    @pytest.mark.asyncio
    async def test_cpu_timeout_enforcement(self):
        """Test CPU timeout enforcement."""
        sandbox = await WasmSandbox.builder() \
            .source('./test-fixtures/cpu-intensive.wasm') \
            .cpu_timeout(1.0) \
            .build()
        
        try:
            with pytest.raises(Exception, match="CPU timeout exceeded"):
                await sandbox.call('infinite_loop', [])
        finally:
            await sandbox.dispose()
    
    @pytest.mark.asyncio
    async def test_security_policy_enforcement(self):
        """Test security policy enforcement."""
        sandbox = await WasmSandbox.builder() \
            .source('./test-fixtures/network-module.wasm') \
            .security_policy(SecurityPolicy.strict()) \
            .build()
        
        try:
            with pytest.raises(SecurityViolation, match="Network access denied"):
                await sandbox.call('make_http_request', ['https://example.com'])
        finally:
            await sandbox.dispose()
    
    @pytest.mark.asyncio
    async def test_batch_function_calls(self, calculator_sandbox):
        """Test batch function execution."""
        calls = [
            {'function': 'add', 'args': [1, 2]},
            {'function': 'multiply', 'args': [3, 4]},
            {'function': 'subtract', 'args': [10, 5]}
        ]
        
        results = await calculator_sandbox.call_batch(calls)
        
        assert len(results) == 3
        assert results[0] == 3  # 1 + 2
        assert results[1] == 12  # 3 * 4
        assert results[2] == 5   # 10 - 5
    
    @pytest.mark.asyncio
    async def test_resource_monitoring(self, test_sandbox):
        """Test resource monitoring capabilities."""
        # Initial state
        initial_stats = await test_sandbox.get_resource_stats()
        assert initial_stats.function_calls == 0
        
        # Execute some functions
        await test_sandbox.call('simple_function', [])
        await test_sandbox.call('another_function', [42])
        
        # Check updated stats
        updated_stats = await test_sandbox.get_resource_stats()
        assert updated_stats.function_calls == 2
        assert updated_stats.cpu_time_ms > initial_stats.cpu_time_ms
    
    @pytest.mark.asyncio
    async def test_sandbox_reset(self, test_sandbox):
        """Test sandbox state reset."""
        # Set some state
        await test_sandbox.call('set_state', [42])
        state = await test_sandbox.call('get_state', [])
        assert state == 42
        
        # Reset sandbox
        await test_sandbox.reset()
        
        # State should be cleared
        state = await test_sandbox.call('get_state', [])
        assert state == 0  # Default state

# Performance tests
class TestPerformance:
    
    @pytest.mark.asyncio
    async def test_concurrent_execution(self):
        """Test concurrent sandbox execution."""
        sandbox = await create_test_sandbox('./test-fixtures/calculator.wasm')
        
        try:
            # Create many concurrent tasks
            tasks = [
                sandbox.call('add', [i, i+1])
                for i in range(100)
            ]
            
            results = await asyncio.gather(*tasks)
            
            # Verify results
            for i, result in enumerate(results):
                expected = i + (i + 1)
                assert result == expected
        
        finally:
            await sandbox.dispose()
    
    @pytest.mark.asyncio
    async def test_startup_performance(self):
        """Test sandbox startup performance."""
        import time
        
        start_time = time.time()
        
        sandbox = await WasmSandbox.from_file('./test-fixtures/calculator.wasm')
        
        startup_time = time.time() - start_time
        
        try:
            # Should start up quickly (under 100ms)
            assert startup_time < 0.1
            
            # Should be able to execute immediately
            result = await sandbox.call('add', [1, 1])
            assert result == 2
        
        finally:
            await sandbox.dispose()

# Integration tests
class TestIntegration:
    
    @pytest.mark.asyncio
    async def test_django_integration(self):
        """Test Django integration."""
        # This would test the Django-specific functionality
        # when running in a Django test environment
        pass
    
    @pytest.mark.asyncio
    async def test_numpy_integration(self):
        """Test NumPy data exchange."""
        import numpy as np
        from wasm_sandbox.numpy import numpy_to_wasm, wasm_to_numpy
        
        sandbox = await create_test_sandbox('./test-fixtures/math-module.wasm')
        
        try:
            # Create test data
            data = np.array([[1, 2], [3, 4]], dtype=np.float32)
            wasm_data = numpy_to_wasm(data)
            
            # Process in WASM
            result_wasm = await sandbox.call('matrix_transpose', [wasm_data])
            result = wasm_to_numpy(result_wasm)
            
            # Verify result
            expected = np.array([[1, 3], [2, 4]], dtype=np.float32)
            np.testing.assert_array_equal(result, expected)
        
        finally:
            await sandbox.dispose()
```

## Examples

### Real-world Usage Examples

Check the [`examples/`](./examples/) directory for complete working examples:

- **[Flask Web App](./examples/flask-app/)** - Web application with WASM processing
- **[Django REST API](./examples/django-api/)** - REST API with secure computation
- **[Data Pipeline](./examples/data-pipeline/)** - ETL pipeline with pandas integration
- **[CLI Tool](./examples/cli-tool/)** - Command-line application with plugin support
- **[Jupyter Notebook](./examples/jupyter-demo/)** - Interactive data science workflows
- **[Microservice](./examples/microservice/)** - Containerized Python microservice

---

**Python Excellence**: Production-ready Python bindings with comprehensive async support, framework integrations, and data science capabilities.
