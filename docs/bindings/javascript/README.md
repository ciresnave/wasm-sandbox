# JavaScript/TypeScript Bindings

📖 **[← Back to Documentation](../../README.md)** | 🏠 **[← Main README](../../../README.md)** | 🚀 **[NPM Package](https://npmjs.com/package/wasm-sandbox-js)**

Native JavaScript and TypeScript bindings for wasm-sandbox, enabling secure WebAssembly execution in Node.js and browser environments.

## Installation

### Node.js

```bash
# NPM
npm install wasm-sandbox-js

# Yarn
yarn add wasm-sandbox-js

# PNPM
pnpm add wasm-sandbox-js
```

### Browser (CDN)

```html
<!-- ES Modules -->
<script type="module">
  import { WasmSandbox } from 'https://cdn.skypack.dev/wasm-sandbox-js';
</script>

<!-- UMD -->
<script src="https://unpkg.com/wasm-sandbox-js/dist/wasm-sandbox.umd.js"></script>
```

## Quick Start

### Basic Usage

```typescript
import { WasmSandbox, SecurityPolicy } from 'wasm-sandbox-js';

async function main() {
  // Create a secure sandbox
  const sandbox = await WasmSandbox.builder()
    .source('./calculator.wasm')
    .securityPolicy(SecurityPolicy.strict())
    .memoryLimit(64 * 1024 * 1024) // 64MB
    .cpuTimeout(30000) // 30 seconds
    .build();

  // Call a function
  const result = await sandbox.call('add', [5, 3]);
  console.log('Result:', result); // 8

  // Cleanup
  await sandbox.dispose();
}

main().catch(console.error);
```

### TypeScript Types

```typescript
interface WasmSandboxBuilder {
  source(path: string | Uint8Array | URL): WasmSandboxBuilder;
  securityPolicy(policy: SecurityPolicy): WasmSandboxBuilder;
  memoryLimit(bytes: number): WasmSandboxBuilder;
  cpuTimeout(milliseconds: number): WasmSandboxBuilder;
  addCapability(capability: Capability): WasmSandboxBuilder;
  instanceId(id: string): WasmSandboxBuilder;
  build(): Promise<WasmSandbox>;
}

interface WasmSandbox {
  call<T = any>(functionName: string, args?: any[]): Promise<T>;
  callWithAudit<T = any>(functionName: string, args?: any[]): Promise<AuditResult<T>>;
  memoryUsage(): Promise<number>;
  dispose(): Promise<void>;
}

interface SecurityPolicy {
  static strict(): SecurityPolicy;
  static moderate(): SecurityPolicy;
  static permissive(): SecurityPolicy;
  addCapability(capability: Capability): SecurityPolicy;
  removeCapability(capabilityType: string): SecurityPolicy;
}
```

## API Reference

### WasmSandbox Class

#### Constructor Methods

```typescript
// Create sandbox from file path
const sandbox = await WasmSandbox.fromFile('./module.wasm');

// Create sandbox from bytes
const wasmBytes = new Uint8Array([...]);
const sandbox = await WasmSandbox.fromBytes(wasmBytes);

// Create sandbox from URL
const sandbox = await WasmSandbox.fromUrl('https://example.com/module.wasm');

// Advanced builder pattern
const sandbox = await WasmSandbox.builder()
  .source('./module.wasm')
  .securityPolicy(SecurityPolicy.strict())
  .memoryLimit(128 * 1024 * 1024)
  .cpuTimeout(60000)
  .addCapability(Capability.networkAccess(['api.example.com']))
  .addCapability(Capability.fileSystemAccess(['/tmp'], { readOnly: true }))
  .instanceId('worker-1')
  .build();
```

#### Function Execution

```typescript
// Simple function call
const result = await sandbox.call('calculate', [10, 20]);

// Typed function call
interface CalculationResult {
  sum: number;
  product: number;
  timestamp: string;
}

const result = await sandbox.call<CalculationResult>('complexCalculation', [10, 20]);

// Function call with timeout
const result = await sandbox.callWithTimeout('slowFunction', [], 5000);

// Batch function calls
const results = await sandbox.callBatch([
  { function: 'add', args: [1, 2] },
  { function: 'multiply', args: [3, 4] },
  { function: 'divide', args: [10, 2] }
]);

// Function call with audit logging
const auditResult = await sandbox.callWithAudit('sensitiveOperation', [data]);
console.log('Violations:', auditResult.violations);
console.log('Events:', auditResult.events);
console.log('Result:', auditResult.output);
```

#### Resource Management

```typescript
// Memory management
const memoryUsage = await sandbox.memoryUsage();
console.log(`Memory used: ${memoryUsage / 1024 / 1024} MB`);

// Garbage collection
await sandbox.collectGarbage();

// Resource monitoring
const stats = await sandbox.getResourceStats();
console.log('CPU time:', stats.cpuTimeMs);
console.log('Peak memory:', stats.peakMemoryBytes);
console.log('Function calls:', stats.functionCalls);

// Reset sandbox state
await sandbox.reset();

// Cleanup
await sandbox.dispose();
```

### Security Policies

#### Predefined Policies

```typescript
// Strict security (default)
const strictPolicy = SecurityPolicy.strict();

// Moderate security
const moderatePolicy = SecurityPolicy.moderate();

// Permissive security
const permissivePolicy = SecurityPolicy.permissive();

// Custom policy
const customPolicy = SecurityPolicy.builder()
  .allowNetworkAccess(['api.trusted.com', 'cdn.example.com'])
  .allowFileSystemAccess(['/data'], { readOnly: true })
  .allowEnvironmentAccess(['NODE_ENV', 'API_KEY'])
  .enableAuditLogging(true)
  .maxExecutionTime(30000)
  .build();
```

#### Capabilities

```typescript
// Network access
const networkCapability = Capability.networkAccess([
  'api.example.com',
  'cdn.example.com'
], {
  allowedPorts: [80, 443],
  requireHttps: true,
  maxRequestSize: 1024 * 1024 // 1MB
});

// File system access
const fsCapability = Capability.fileSystemAccess([
  '/tmp',
  '/data/readonly'
], {
  readOnly: true,
  maxFileSize: 10 * 1024 * 1024 // 10MB
});

// Environment access
const envCapability = Capability.environmentAccess([
  'NODE_ENV',
  'API_KEY',
  'DATABASE_URL'
]);

// Apply capabilities
const sandbox = await WasmSandbox.builder()
  .source('./module.wasm')
  .addCapability(networkCapability)
  .addCapability(fsCapability)
  .addCapability(envCapability)
  .build();
```

## Advanced Features

### HTTP Server Integration

```typescript
import express from 'express';
import { WasmSandbox, SecurityPolicy } from 'wasm-sandbox-js';

const app = express();
app.use(express.json());

// Create sandbox pool for handling requests
class SandboxPool {
  private sandboxes: WasmSandbox[] = [];
  private available: boolean[] = [];

  async initialize(size: number) {
    for (let i = 0; i < size; i++) {
      const sandbox = await WasmSandbox.builder()
        .source('./request-handler.wasm')
        .securityPolicy(SecurityPolicy.moderate())
        .instanceId(`pool-${i}`)
        .build();
      
      this.sandboxes.push(sandbox);
      this.available.push(true);
    }
  }

  async acquire(): Promise<{ sandbox: WasmSandbox; release: () => void }> {
    const index = this.available.findIndex(available => available);
    if (index === -1) {
      throw new Error('No available sandboxes');
    }

    this.available[index] = false;
    const sandbox = this.sandboxes[index];

    const release = () => {
      this.available[index] = true;
    };

    return { sandbox, release };
  }
}

const pool = new SandboxPool();

app.post('/process', async (req, res) => {
  const { sandbox, release } = await pool.acquire();
  
  try {
    const result = await sandbox.call('processRequest', [req.body]);
    res.json({ result, success: true });
  } catch (error) {
    res.status(500).json({ error: error.message, success: false });
  } finally {
    release();
  }
});

// Initialize pool and start server
pool.initialize(10).then(() => {
  app.listen(3000, () => {
    console.log('Server running on port 3000');
  });
});
```

### Browser Environment

```typescript
// Browser-specific optimizations
import { WasmSandbox, BrowserOptimizations } from 'wasm-sandbox-js/browser';

// Enable web workers for isolation
const sandbox = await WasmSandbox.builder()
  .source('./module.wasm')
  .useWebWorker(true)
  .enableSharedArrayBuffer(true)
  .memoryLimit(32 * 1024 * 1024) // Lower limit for browsers
  .build();

// Service Worker integration
if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/wasm-worker.js')
    .then(registration => {
      console.log('WASM Worker registered:', registration);
    });
}

// Streaming execution for large modules
const stream = fetch('./large-module.wasm');
const sandbox = await WasmSandbox.fromStream(stream);
```

### React Integration

```tsx
import React, { useEffect, useState } from 'react';
import { WasmSandbox } from 'wasm-sandbox-js';

interface UseWasmSandboxOptions {
  source: string;
  securityPolicy?: SecurityPolicy;
  autoDispose?: boolean;
}

function useWasmSandbox(options: UseWasmSandboxOptions) {
  const [sandbox, setSandbox] = useState<WasmSandbox | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let disposed = false;

    async function initializeSandbox() {
      try {
        const newSandbox = await WasmSandbox.builder()
          .source(options.source)
          .securityPolicy(options.securityPolicy || SecurityPolicy.strict())
          .build();

        if (!disposed) {
          setSandbox(newSandbox);
          setLoading(false);
        } else {
          // Component unmounted before sandbox was ready
          await newSandbox.dispose();
        }
      } catch (err) {
        if (!disposed) {
          setError(err as Error);
          setLoading(false);
        }
      }
    }

    initializeSandbox();

    return () => {
      disposed = true;
      if (sandbox && options.autoDispose !== false) {
        sandbox.dispose();
      }
    };
  }, [options.source]);

  return { sandbox, loading, error };
}

// Usage in component
function CalculatorComponent() {
  const { sandbox, loading, error } = useWasmSandbox({
    source: './calculator.wasm',
    autoDispose: true
  });

  const [result, setResult] = useState<number | null>(null);

  const calculate = async (a: number, b: number) => {
    if (!sandbox) return;
    
    try {
      const result = await sandbox.call<number>('add', [a, b]);
      setResult(result);
    } catch (err) {
      console.error('Calculation error:', err);
    }
  };

  if (loading) return <div>Loading calculator...</div>;
  if (error) return <div>Error: {error.message}</div>;

  return (
    <div>
      <button onClick={() => calculate(5, 3)}>
        Calculate 5 + 3
      </button>
      {result !== null && <div>Result: {result}</div>}
    </div>
  );
}
```

### Node.js Cluster Support

```typescript
import cluster from 'cluster';
import { cpus } from 'os';
import { WasmSandbox } from 'wasm-sandbox-js';

if (cluster.isPrimary) {
  // Primary process
  const numCPUs = cpus().length;
  
  for (let i = 0; i < numCPUs; i++) {
    cluster.fork();
  }

  cluster.on('exit', (worker, code, signal) => {
    console.log(`Worker ${worker.process.pid} died`);
    cluster.fork(); // Restart worker
  });
} else {
  // Worker process
  async function workerMain() {
    const sandbox = await WasmSandbox.builder()
      .source('./worker-module.wasm')
      .instanceId(`worker-${process.pid}`)
      .build();

    // Process messages from primary
    process.on('message', async (message) => {
      try {
        const result = await sandbox.call(message.function, message.args);
        process.send!({ id: message.id, result, success: true });
      } catch (error) {
        process.send!({ id: message.id, error: error.message, success: false });
      }
    });

    console.log(`Worker ${process.pid} ready`);
  }

  workerMain().catch(console.error);
}
```

## Performance Optimization

### Connection Pooling

```typescript
class OptimizedSandboxPool {
  private pool: WasmSandbox[] = [];
  private inUse: Set<WasmSandbox> = new Set();
  private queue: Array<{ resolve: Function; reject: Function }> = [];

  constructor(private options: {
    minSize: number;
    maxSize: number;
    source: string;
    securityPolicy: SecurityPolicy;
  }) {}

  async initialize() {
    // Create minimum number of sandboxes
    for (let i = 0; i < this.options.minSize; i++) {
      const sandbox = await this.createSandbox();
      this.pool.push(sandbox);
    }
  }

  async acquire(): Promise<WasmSandbox> {
    // Return available sandbox from pool
    if (this.pool.length > 0) {
      const sandbox = this.pool.pop()!;
      this.inUse.add(sandbox);
      return sandbox;
    }

    // Create new sandbox if under max limit
    if (this.inUse.size < this.options.maxSize) {
      const sandbox = await this.createSandbox();
      this.inUse.add(sandbox);
      return sandbox;
    }

    // Wait for available sandbox
    return new Promise((resolve, reject) => {
      this.queue.push({ resolve, reject });
    });
  }

  release(sandbox: WasmSandbox) {
    this.inUse.delete(sandbox);

    // Serve queued requests first
    if (this.queue.length > 0) {
      const { resolve } = this.queue.shift()!;
      this.inUse.add(sandbox);
      resolve(sandbox);
      return;
    }

    // Return to pool if under min size
    if (this.pool.length < this.options.minSize) {
      this.pool.push(sandbox);
    } else {
      // Dispose excess sandboxes
      sandbox.dispose();
    }
  }

  private async createSandbox(): Promise<WasmSandbox> {
    return WasmSandbox.builder()
      .source(this.options.source)
      .securityPolicy(this.options.securityPolicy)
      .build();
  }

  async dispose() {
    // Dispose all sandboxes
    const allSandboxes = [...this.pool, ...this.inUse];
    await Promise.all(allSandboxes.map(s => s.dispose()));
    
    // Reject queued requests
    this.queue.forEach(({ reject }) => {
      reject(new Error('Pool disposed'));
    });
  }
}
```

### Caching and Precompilation

```typescript
import { createHash } from 'crypto';

class SandboxCache {
  private cache = new Map<string, WasmSandbox>();
  private compilationCache = new Map<string, Uint8Array>();

  private getCacheKey(source: string, options: any): string {
    const content = JSON.stringify({ source, options });
    return createHash('sha256').update(content).digest('hex');
  }

  async getOrCreate(
    source: string,
    options: {
      securityPolicy: SecurityPolicy;
      memoryLimit?: number;
      cpuTimeout?: number;
    }
  ): Promise<WasmSandbox> {
    const cacheKey = this.getCacheKey(source, options);
    
    // Return cached sandbox if available
    if (this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey)!;
    }

    // Check for precompiled module
    let moduleBytes: Uint8Array;
    if (this.compilationCache.has(cacheKey)) {
      moduleBytes = this.compilationCache.get(cacheKey)!;
    } else {
      // Load and cache module bytes
      if (source.startsWith('http')) {
        const response = await fetch(source);
        moduleBytes = new Uint8Array(await response.arrayBuffer());
      } else {
        const fs = await import('fs/promises');
        moduleBytes = await fs.readFile(source);
      }
      
      this.compilationCache.set(cacheKey, moduleBytes);
    }

    // Create and cache sandbox
    const sandbox = await WasmSandbox.builder()
      .source(moduleBytes)
      .securityPolicy(options.securityPolicy)
      .memoryLimit(options.memoryLimit)
      .cpuTimeout(options.cpuTimeout)
      .build();

    this.cache.set(cacheKey, sandbox);
    return sandbox;
  }

  clear() {
    // Dispose all cached sandboxes
    this.cache.forEach(sandbox => sandbox.dispose());
    this.cache.clear();
    this.compilationCache.clear();
  }
}

// Global cache instance
export const sandboxCache = new SandboxCache();
```

## Testing

### Jest Integration

```typescript
// jest.config.js
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  setupFilesAfterEnv: ['<rootDir>/src/test-setup.ts'],
  testTimeout: 30000, // Longer timeout for WASM operations
};

// test-setup.ts
import { WasmSandbox } from 'wasm-sandbox-js';

// Global test sandbox
let testSandbox: WasmSandbox;

beforeAll(async () => {
  testSandbox = await WasmSandbox.builder()
    .source('./test-fixtures/test-module.wasm')
    .build();
});

afterAll(async () => {
  if (testSandbox) {
    await testSandbox.dispose();
  }
});

// Test utilities
export async function createTestSandbox(source: string = './test-fixtures/test-module.wasm') {
  return WasmSandbox.builder()
    .source(source)
    .securityPolicy(SecurityPolicy.permissive()) // Relaxed for testing
    .build();
}

export { testSandbox };
```

### Test Examples

```typescript
// sandbox.test.ts
import { WasmSandbox, SecurityPolicy } from 'wasm-sandbox-js';
import { createTestSandbox } from './test-setup';

describe('WasmSandbox', () => {
  test('should execute basic functions', async () => {
    const sandbox = await createTestSandbox();
    
    const result = await sandbox.call('add', [5, 3]);
    expect(result).toBe(8);
    
    await sandbox.dispose();
  });

  test('should enforce memory limits', async () => {
    const sandbox = await WasmSandbox.builder()
      .source('./test-fixtures/memory-intensive.wasm')
      .memoryLimit(1024 * 1024) // 1MB limit
      .build();

    await expect(
      sandbox.call('allocateLargeBuffer', [])
    ).rejects.toThrow('Memory limit exceeded');
    
    await sandbox.dispose();
  });

  test('should respect CPU timeouts', async () => {
    const sandbox = await WasmSandbox.builder()
      .source('./test-fixtures/cpu-intensive.wasm')
      .cpuTimeout(1000) // 1 second timeout
      .build();

    await expect(
      sandbox.call('infiniteLoop', [])
    ).rejects.toThrow('CPU timeout exceeded');
    
    await sandbox.dispose();
  });

  test('should enforce security policies', async () => {
    const sandbox = await WasmSandbox.builder()
      .source('./test-fixtures/network-module.wasm')
      .securityPolicy(SecurityPolicy.strict()) // No network access
      .build();

    await expect(
      sandbox.call('makeHttpRequest', ['https://example.com'])
    ).rejects.toThrow('Network access denied');
    
    await sandbox.dispose();
  });
});
```

## Examples

### Real-world Usage Examples

Check the [`examples/`](./examples/) directory for complete working examples:

- **[Web Server](./examples/web-server/)** - Express.js server with WASM request processing
- **[CLI Tool](./examples/cli-tool/)** - Command-line application with plugin support
- **[Browser App](./examples/browser-app/)** - Frontend application with WASM modules
- **[Microservice](./examples/microservice/)** - Containerized microservice architecture
- **[Data Pipeline](./examples/data-pipeline/)** - ETL pipeline with WASM transformations

---

**JavaScript Excellence**: Production-ready JavaScript/TypeScript bindings with comprehensive APIs, optimization strategies, and framework integrations.
