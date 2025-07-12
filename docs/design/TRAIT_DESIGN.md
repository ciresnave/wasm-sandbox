# WASM Sandbox Trait Design Documentation

## Overview

This document describes the new trait design patterns implemented in the wasm-sandbox crate to achieve dyn-compatibility while preserving type-safe and generic APIs. This refactoring was necessary to enable the use of trait objects (`dyn`) while maintaining flexibility for advanced use cases.

## Core Design Principles

### 1. Dyn-Compatible Core Traits

The core traits (`WasmInstance`, `WasmModule`, `WasmRuntime`, `RpcChannel`) are designed to be dyn-compatible, meaning they can be used as trait objects (`dyn WasmInstance`, `dyn WasmRuntime`, etc.).

**Key Characteristics:**

**Key Characteristics:**

- No generic methods with non-concrete types
- No associated types beyond basic ones  
- No async functions in the core trait (async is available in extension traits)
- All methods return concrete types or basic generic bounds
- No associated types beyond basic ones
- No async functions in the core trait
- All methods return concrete types or basic generic bounds

### 2. Extension Traits for Advanced Features

Advanced functionality that requires generics or async operations is provided through extension traits that extend the core dyn-compatible traits.

**Extension Trait Patterns:**

- `WasmInstanceExt` for generic/async operations on instances
- `WasmRuntimeExt` for advanced runtime operations
- `RpcChannelExt` for type-safe communication

### 3. Downcasting Support

All trait objects support downcasting to concrete implementations through the `as_any()` method.

## Trait Architecture

### Core Traits (Dyn-Compatible)

#### `WasmInstance`

```rust
pub trait WasmInstance: Send + Sync {
    // Basic state and resource management
    fn state(&self) -> WasmInstanceState;
    fn memory_usage(&self) -> usize;
    fn fuel_usage(&self) -> Option<u64>;
    
    // Resource control
    fn reset_fuel(&self) -> Result<()>;
    fn add_fuel(&self, fuel: u64) -> Result<()>;
    
    // Memory access
    unsafe fn memory_ptr(&self) -> Result<*mut u8>;
    fn memory_size(&self) -> usize;
    
    // Function calling (basic)
    fn function_caller(&self) -> Box<dyn WasmFunctionCaller>;
    fn call_simple_function(&self, function_name: &str, params: &[i32]) -> Result<i32>;
    
    // Downcasting support
    fn as_any(&self) -> &dyn Any;
}
```

#### `WasmRuntime`

```rust
pub trait WasmRuntime: Send + Sync {
    // Module management
    fn load_module(&self, wasm_bytes: &[u8]) -> Result<Box<dyn WasmModule>>;
    fn get_module(&self, id: ModuleId) -> Result<Box<dyn WasmModule>>;
    
    // Instance management
    fn create_instance(
        &self, 
        module: &dyn WasmModule, 
        resources: ResourceLimits,
        capabilities: Capabilities,
    ) -> Result<Box<dyn WasmInstance>>;
    
    // Runtime info
    fn runtime_type(&self) -> &'static str;
    fn capabilities(&self) -> RuntimeCapabilities;
    
    // Downcasting support
    fn as_any(&self) -> &dyn Any;
}
```

### Extension Traits (Generic/Async)

Extension traits provide advanced functionality including async operations and generic methods. These traits use `#[allow(async_fn_in_trait)]` to suppress compiler warnings about async functions in traits, as this is an intentional design choice for extension traits within the crate.

#### `WasmInstanceExt`

```rust
pub trait WasmInstanceExt: WasmInstance {
    // Generic function calling
    async fn call_function_json_async(
        &self,
        function_name: &str,
        params_json: &str,
    ) -> Result<String>;
    
    async fn call_function_msgpack_async(
        &self,
        function_name: &str,
        params_msgpack: &[u8],
    ) -> Result<Vec<u8>>;
    
    // Type-safe function calling
    async fn call_function<P, R>(
        &self,
        function_name: &str,
        params: &P,
    ) -> Result<R>
    where
        P: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de> + Send + Sync;
}
```

#### `WasmRuntimeExt`

```rust
pub trait WasmRuntimeExt: WasmRuntime {
    // Advanced module loading
    async fn load_module_async(&self, wasm_bytes: &[u8]) -> Result<Box<dyn WasmModule>>;
    
    // Batch operations
    async fn create_instances_batch<I>(&self, specs: I) -> Result<Vec<Box<dyn WasmInstance>>>
    where
        I: Iterator<Item = InstanceSpec> + Send;
}
```

### Communication Traits

#### `RpcChannel` (Dyn-Compatible)

```rust
pub trait RpcChannel: Send + Sync {
    // Basic message passing
    fn send_raw(&self, data: &[u8]) -> Result<()>;
    fn receive_raw(&self) -> Result<Option<Vec<u8>>>;
    
    // Connection management
    fn is_connected(&self) -> bool;
    fn close(&self) -> Result<()>;
    
    // Downcasting support
    fn as_any(&self) -> &dyn Any;
}
```

#### `RpcChannelExt` (Generic)

```rust
pub trait RpcChannelExt: RpcChannel {
    // Type-safe communication
    async fn send<T: Serialize + Send>(&self, message: &T) -> Result<()>;
    async fn receive<T: for<'de> Deserialize<'de> + Send>(&self) -> Result<Option<T>>;
    
    // Request-response pattern
    async fn call<P, R>(&self, method: &str, params: &P) -> Result<R>
    where
        P: Serialize + Send,
        R: for<'de> Deserialize<'de> + Send;
}
```

## Usage Patterns

### Basic Usage (Dyn-Compatible)

```rust
// Create runtime as trait object
let runtime: Box<dyn WasmRuntime> = create_runtime(&config)?;

// Load module
let module = runtime.load_module(&wasm_bytes)?;

// Create instance
let instance = runtime.create_instance(
    module.as_ref(),
    ResourceLimits::default(),
    Capabilities::minimal(),
)?;

// Basic function call
let result = instance.call_simple_function("add", &[5, 7])?;
```

### Advanced Usage (Extension Traits)

```rust
// Create runtime with specific type for extension traits
let runtime = WasmtimeRuntime::new(&config)?;

// Use extension trait for async operations
let module = runtime.load_module_async(&wasm_bytes).await?;

// Type-safe function calling
let result: ComplexResult = instance
    .call_function("complex_function", &complex_params)
    .await?;
```

### Mixed Usage Pattern

```rust
// Start with trait object for flexibility
let runtime: Box<dyn WasmRuntime> = create_runtime(&config)?;

// Downcast when needed for specific features
if let Some(wasmtime_runtime) = runtime.as_any().downcast_ref::<WasmtimeRuntime>() {
    // Use Wasmtime-specific features
    wasmtime_runtime.enable_debug_info()?;
}

// Continue with trait object for common operations
let instance = runtime.create_instance(module.as_ref(), limits, caps)?;
```

### Communication Patterns

```rust
// Basic RPC communication
let channel: Box<dyn RpcChannel> = create_memory_channel(1024)?;

// Type-safe communication with extension trait
async fn send_request<T: RpcChannelExt>(channel: &T, data: &MyData) -> Result<MyResponse> {
    channel.call("process", data).await
}
```

## Implementation Guidelines

### For Runtime Implementers

1. **Implement the Core Trait First**

   ```rust
   impl WasmRuntime for MyRuntime {
       fn load_module(&self, wasm_bytes: &[u8]) -> Result<Box<dyn WasmModule>> {
           // Implementation
       }
       
       fn as_any(&self) -> &dyn Any {
           self
       }
   }
   ```

2. **Add Extension Trait Implementation**

   ```rust
   impl WasmRuntimeExt for MyRuntime {
       async fn load_module_async(&self, wasm_bytes: &[u8]) -> Result<Box<dyn WasmModule>> {
           // Async implementation
       }
   }
   ```

3. **Provide Blanket Implementation for Trait Objects**

   ```rust
   impl<T: WasmRuntime + ?Sized> WasmRuntimeExt for T {
       // Default implementations that delegate to sync methods
   }
   ```

### For Library Users

1. **Use Trait Objects for Flexibility**
   - Store runtimes and instances as `Box<dyn WasmRuntime>` and `Box<dyn WasmInstance>`
   - This allows switching between different runtime implementations

2. **Use Extension Traits for Advanced Features**
   - Import extension traits: `use wasm_sandbox::runtime::WasmInstanceExt;`
   - Call async and generic methods on trait objects

3. **Downcast When Necessary**
   - Use `as_any()` to access implementation-specific features
   - Handle downcasting failures gracefully

## Benefits of This Design

### 1. **Flexibility**

- Can use any runtime implementation through trait objects
- Easy to switch between Wasmtime, Wasmer, or custom runtimes
- Plugin-style architecture support

### 2. **Type Safety**

- Extension traits provide full type safety for generic operations
- Compile-time guarantees for serialization/deserialization

### 3. **Performance**

- Zero-cost abstractions for direct usage
- Minimal overhead for trait object usage
- Async operations properly supported

### 4. **Backward Compatibility**

- Existing code using concrete types continues to work
- Gradual migration path to trait objects

### 5. **Extensibility**

- Easy to add new extension traits
- Custom runtime implementations can provide additional features
- Future-proof design

## Migration Guide

### From Concrete Types to Trait Objects

**Before:**

```rust
let runtime = WasmtimeRuntime::new(&config)?;
let instance = runtime.create_instance(/* ... */)?;
```

**After:**

```rust
let runtime: Box<dyn WasmRuntime> = Box::new(WasmtimeRuntime::new(&config)?);
let instance = runtime.create_instance(/* ... */)?;
```

### From Generic Methods to Extension Traits

**Before:**

```rust
let result: T = instance.call_function(name, params)?; // Not dyn-compatible
```

**After:**

```rust
use wasm_sandbox::runtime::WasmInstanceExt;
let result: T = instance.call_function(name, params).await?; // Works with trait objects
```

## Testing the Design

The trait structure is comprehensively tested in `tests/trait_structure_test.rs`:

- **Dyn-compatibility tests**: Verify trait objects work correctly
- **Extension trait tests**: Ensure generic and async methods function
- **Downcasting tests**: Validate `as_any()` implementations
- **Error handling tests**: Check error propagation through trait boundaries
- **Concurrent access tests**: Verify thread safety

## Conclusion

This trait design provides a robust foundation for the wasm-sandbox crate that balances:

- **Usability**: Simple APIs for common use cases
- **Flexibility**: Support for advanced scenarios through extension traits
- **Performance**: Minimal overhead and zero-cost abstractions
- **Safety**: Strong type checking and memory safety guarantees

The design is extensible and future-proof, allowing for new runtime backends and communication channels while maintaining a consistent API surface.
