//! WebAssembly runtime abstraction

use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::Result;
use crate::security::{Capabilities, ResourceLimits};

/// Metrics for the WebAssembly runtime
#[derive(Debug, Clone)]
pub struct RuntimeMetrics {
    /// Number of compiled modules
    pub compiled_modules: usize,
    
    /// Number of active instances
    pub active_instances: usize,
    
    /// Total memory used by all instances (in bytes)
    pub total_memory_usage: usize,
    
    /// Peak memory usage (in bytes)
    pub peak_memory_usage: usize,
    
    /// Fuel consumption rate (instructions per second)
    pub fuel_consumption_rate: Option<f64>,
    
    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: Option<f64>,
    
    /// Compilation time for the last module (in milliseconds)
    pub last_compilation_time_ms: Option<u64>,
}

/// Configuration for the WebAssembly runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Enable fuel-based resource metering
    pub enable_fuel: bool,
    
    /// Enable memory limits
    pub enable_memory_limits: bool,
    
    /// Enable native stack trace support
    pub native_stack_trace: bool,
    
    /// Enable debug information
    pub debug_info: bool,
    
    /// Thread pool size for module compilation
    pub compilation_threads: usize,
    
    /// Cache compiled modules
    pub cache_modules: bool,
    
    /// Cache directory for compiled modules
    pub cache_directory: Option<PathBuf>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            enable_fuel: true,
            enable_memory_limits: true,
            native_stack_trace: false,
            debug_info: false,
            compilation_threads: num_cpus::get(),
            cache_modules: true,
            cache_directory: None,
        }
    }
}

/// Unique identifier for a WebAssembly module
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(Uuid);

impl ModuleId {
    /// Create a new random module ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ModuleId {
    fn default() -> Self {
        Self::new()
    }
}

/// State of a WebAssembly instance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmInstanceState {
    /// Instance is created but not started
    Created,
    
    /// Instance is running
    Running,
    
    /// Instance is paused
    Paused,
    
    /// Instance has exited with a status code
    Exited(i32),
    
    /// Instance has crashed
    Crashed,
}

/// WebAssembly module abstraction
pub trait WasmModule: Send + Sync {
    /// Get the module ID
    fn id(&self) -> ModuleId;
    
    /// Get the module name
    fn name(&self) -> Option<&str>;
    
    /// Get the module size in bytes
    fn size(&self) -> usize;
    
    /// Get the list of exported functions
    fn exports(&self) -> Vec<String>;
    
    /// Clone the module
    fn clone_module(&self) -> Box<dyn WasmModule>;
    
    /// Get a reference to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// WebAssembly instance abstraction (dyn-compatible part)
pub trait WasmInstance: Send + Sync {
    /// Get the instance state
    fn state(&self) -> WasmInstanceState;
    
    /// Get memory usage in bytes
    fn memory_usage(&self) -> usize;
    
    /// Get fuel usage (if enabled)
    fn fuel_usage(&self) -> Option<u64>;
    
    /// Reset fuel counter (if enabled)
    fn reset_fuel(&self) -> Result<()>;
    
    /// Add fuel (if enabled)
    fn add_fuel(&self, fuel: u64) -> Result<()>;
    
    /// Get a raw pointer to the instance memory
    /// 
    /// # Safety
    /// 
    /// This is unsafe because it allows direct access to the instance's memory.
    /// The caller must ensure that they do not corrupt memory or violate WebAssembly's
    /// memory safety guarantees.
    unsafe fn memory_ptr(&self) -> Result<*mut u8>;
    
    /// Get memory size in bytes
    fn memory_size(&self) -> usize;
    
    /// Get the function caller for this instance
    fn function_caller(&self) -> Box<dyn WasmFunctionCaller>;
    
    /// Simple function call for basic cases (add two i32s)
    /// This is a convenience method for testing and simple operations
    fn call_simple_function(&self, function_name: &str, params: &[i32]) -> Result<i32>;
}

/// Separate trait for generic/async function calling (dyn-compatible)
pub trait WasmFunctionCaller: Send + Sync {
    /// Call a function in the instance with JSON serialization
    fn call_function_json(
        &self,
        function_name: &str,
        params_json: &str,
    ) -> Result<String>;
    
    /// Call a function in the instance with MessagePack serialization
    fn call_function_msgpack(
        &self,
        function_name: &str,
        params_msgpack: &[u8],
    ) -> Result<Vec<u8>>;
    
    /// Get a reference to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Extension trait for async function calling (not dyn-compatible)
#[allow(async_fn_in_trait)]
pub trait WasmFunctionCallerAsync {
    /// Call a function in the instance with JSON serialization (async version)
    async fn call_function_json_async(
        &self,
        function_name: &str,
        params_json: &str,
    ) -> Result<String>;
    
    /// Call a function in the instance with MessagePack serialization (async version)
    async fn call_function_msgpack_async(
        &self,
        function_name: &str,
        params_msgpack: &[u8],
    ) -> Result<Vec<u8>>;
}

/// Extension trait for type-safe function calling (generic, not dyn-compatible)
#[allow(async_fn_in_trait)]
pub trait WasmFunctionCallerExt {
    /// Call a function with compile-time type safety
    async fn call_function<P, R>(
        &self,
        function_name: &str,
        params: &P,
    ) -> Result<R>
    where
        P: serde::Serialize + Send + Sync,
        R: for<'de> serde::Deserialize<'de> + Send;
}

/// Automatic implementation for all function callers
impl<T: WasmFunctionCaller + Send + Sync> WasmFunctionCallerExt for T {
    async fn call_function<P, R>(
        &self,
        function_name: &str,
        params: &P,
    ) -> Result<R>
    where
        P: serde::Serialize + Send + Sync,
        R: for<'de> serde::Deserialize<'de> + Send,
    {
        let params_json = serde_json::to_string(params)?;
        let result_json = self.call_function_json(function_name, &params_json)?;
        let result = serde_json::from_str(&result_json)?;
        Ok(result)
    }
}

/// WebAssembly runtime abstraction
pub trait WasmRuntime: Send + Sync {
    /// Initialize the runtime with configuration
    fn initialize(&mut self, config: RuntimeConfig) -> Result<()>;
    
    /// Load a WASM module from bytes
    fn load_module(&self, wasm_bytes: &[u8]) -> Result<Box<dyn WasmModule>>;
    
    /// Get a module by ID
    fn get_module(&self, id: ModuleId) -> Result<Arc<dyn WasmModule>>;
    
    /// Get all module IDs
    fn get_module_ids(&self) -> Vec<ModuleId>;
    
    /// Create a new instance with resource limits
    fn create_instance(
        &self, 
        module: &dyn WasmModule, 
        resources: ResourceLimits,
        capabilities: Capabilities,
    ) -> Result<Box<dyn WasmInstance>>;
    
    /// Get runtime metrics
    fn get_metrics(&self) -> RuntimeMetrics;
    
    /// Shutdown the runtime
    fn shutdown(&self) -> Result<()>;
}

/// Create a runtime with the given configuration
pub fn create_runtime(config: &RuntimeConfig) -> Result<Box<dyn WasmRuntime>> {
    #[cfg(feature = "wasmtime-runtime")]
    {
        Ok(Box::new(crate::runtime::wasmtime::WasmtimeRuntime::new(config)?))
    }

    #[cfg(all(feature = "wasmer-runtime", not(feature = "wasmtime-runtime")))]
    {
        return Ok(Box::new(crate::runtime::wasmer::WasmerRuntime::new(config)?));
    }

    #[cfg(not(any(feature = "wasmtime-runtime", feature = "wasmer-runtime")))]
    {
        return Err(crate::error::Error::RuntimeInitialization(
            "No WebAssembly runtime feature is enabled".to_string(),
        ));
    }
}

pub mod wasmtime;
// Wasmer runtime (re-enabled with new trait structure)
#[cfg(feature = "wasmer-runtime")]
pub mod wasmer;
pub mod wasm_common;

// Re-export runtimes for convenience
#[cfg(feature = "wasmtime-runtime")]
pub use self::wasmtime::WasmtimeRuntime;

#[cfg(feature = "wasmer-runtime")]
pub use self::wasmer::WasmerRuntime;
