//! Wasmtime runtime implementation

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::any::Any;

use dashmap::DashMap;
use wasmtime::{Engine, Module, Store, Linker, Config, Val, Memory, Caller, AsContextMut, Extern, Func};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

use crate::error::{Error, Result};
use crate::runtime::{
    ModuleId, RuntimeConfig, RuntimeMetrics, 
    WasmInstance, WasmInstanceState, WasmModule, WasmRuntime
};
use crate::security::{Capabilities, ResourceLimits};

/// Wasmtime module implementation
pub struct WasmtimeModule {
    /// Module ID
    id: ModuleId,
    
    /// Module name
    name: Option<String>,
    
    /// Wasmtime module
    module: Module,
    
    /// Module exports
    exports: Vec<String>,
    
    /// Module size in bytes
    size: usize,
}

impl WasmtimeModule {
    /// Create a new Wasmtime module
    pub fn new(module: Module, wasm_bytes: &[u8]) -> Self {
        // Extract exports using the current wasmtime API
        let exports = module.exports()
            .map(|export| export.name().to_string())
            .collect();
        
        // Extract module name if available
        let name = None; // Current API doesn't expose this
        
        Self {
            id: ModuleId::new(),
            name,
            module,
            exports,
            size: wasm_bytes.len(),
        }
    }
    
    /// Get a reference to the Wasmtime module
    pub fn module(&self) -> &Module {
        &self.module
    }
}

impl WasmModule for WasmtimeModule {
    fn id(&self) -> ModuleId {
        self.id
    }
    
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    fn size(&self) -> usize {
        self.size
    }
    
    fn exports(&self) -> Vec<String> {
        self.exports.clone()
    }
    
    fn clone_module(&self) -> Box<dyn WasmModule> {
        Box::new(Self {
            id: self.id,
            name: self.name.clone(),
            module: self.module.clone(),
            exports: self.exports.clone(),
            size: self.size,
        })
    }
}

/// Store data for Wasmtime instance
pub struct WasmtimeStoreData {
    /// WASI context
    wasi_ctx: WasiCtx,
    
    /// Instance state
    state: WasmInstanceState,
}

/// Wasmtime instance implementation
pub struct WasmtimeInstance {
    /// Store for the instance
    store: Store<WasmtimeStoreData>,
    
    /// Linker for the instance
    linker: Arc<Linker<WasmtimeStoreData>>,
    
    /// Module ID
    module_id: ModuleId,
}

impl WasmtimeInstance {
    /// Create a new Wasmtime instance
    pub fn new(
        store: Store<WasmtimeStoreData>,
        linker: Arc<Linker<WasmtimeStoreData>>,
        module_id: ModuleId,
    ) -> Self {
        Self {
            store,
            linker,
            module_id,
        }
    }
}

impl WasmInstance for WasmtimeInstance {
    fn state(&self) -> WasmInstanceState {
        self.store.data().state
    }
    
    async fn call_function<P, R>(
        &self,
        function_name: &str,
        params: &P,
    ) -> Result<R>
    where
        P: serde::Serialize + ?Sized,
        R: for<'de> serde::Deserialize<'de>,
    {
        // Serialize parameters to JSON
        let params_json = serde_json::to_string(params)
            .map_err(|e| Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: format!("Failed to serialize parameters: {}", e),
            })?;
        
        // Get the function from the instance
        let func = self.linker.get(&mut self.store, "", function_name)
            .ok_or_else(|| Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: "Function not found in instance".to_string(),
            })?;
        
        // Call the function with parameters
        let result = func.call_async(&mut self.store, [wasmtime::Val::String(self.store.as_context_mut(), params_json.into())])
            .await
            .map_err(|e| Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: format!("Function call failed: {}", e),
            })?;
            
        // Get the result as a string
        let result_string = if let Some(val) = result.first() {
            if let wasmtime::Val::String(s) = val {
                s.to_string().map_err(|e| Error::FunctionCall {
                    function_name: function_name.to_string(),
                    reason: format!("Failed to convert result to string: {}", e),
                })?
            } else {
                return Err(Error::FunctionCall {
                    function_name: function_name.to_string(),
                    reason: "Function returned a non-string value".to_string(),
                });
            }
        } else {
            return Err(Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: "Function returned no values".to_string(),
            });
        };
        
        // Deserialize the result
        serde_json::from_str(&result_string)
            .map_err(|e| Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: format!("Failed to deserialize result: {}", e),
            })
    }
    
    fn memory_usage(&self) -> usize {
        // Get memory instance from the store
        if let Ok(memory) = self.get_memory() {
            let data_size = memory.data_size(&self.store);
            return data_size;
        }
        
        0
    }
    
    fn fuel_usage(&self) -> Option<u64> {
        // Get fuel consumption if enabled
        if let Ok(fuel_consumed) = wasmtime::Fuel::get_consumed(&mut self.store.as_context_mut()) {
            return Some(fuel_consumed);
        }
        
        None
    }
    
    fn reset_fuel(&self) -> Result<()> {
        // Reset fuel counter if enabled
        if let Err(e) = wasmtime::Fuel::reset(&mut self.store.as_context_mut(), 0) {
            return Err(Error::ResourceLimit(format!("Failed to reset fuel: {}", e)));
        }
        
        Ok(())
    }
    
    fn add_fuel(&self, fuel: u64) -> Result<()> {
        // Add fuel if enabled
        if let Err(e) = wasmtime::Fuel::add(&mut self.store.as_context_mut(), fuel) {
            return Err(Error::ResourceLimit(format!("Failed to add fuel: {}", e)));
        }
        
        Ok(())
    }
    
    unsafe fn memory_ptr(&self) -> Result<*mut u8> {
        // Get memory pointer from the instance
        if let Ok(memory) = self.get_memory() {
            let ptr = memory.data_ptr(&self.store);
            return Ok(ptr);
        }
        
        Err(Error::UnsupportedOperation("No memory exported by the module".to_string()))
    }
    
    fn memory_size(&self) -> usize {
        self.memory_usage()
    }
    
    // Helper method to get the memory instance
    fn get_memory(&self) -> Result<wasmtime::Memory> {
        let memory = self.linker
            .get(&mut self.store, "", "memory")
            .and_then(|ext| ext.into_memory())
            .ok_or_else(|| {
                Error::UnsupportedOperation("No memory exported by the module".to_string())
            })?;
            
        Ok(memory)
    }
}

/// Wasmtime runtime implementation
pub struct WasmtimeRuntime {
    /// Wasmtime engine
    engine: Engine,
    
    /// Configuration
    config: RuntimeConfig,
    
    /// Loaded modules
    modules: DashMap<ModuleId, Arc<WasmtimeModule>>,
    
    /// Runtime metrics
    metrics: Mutex<RuntimeMetrics>,
}

impl WasmtimeRuntime {
    /// Create a new Wasmtime runtime
    pub fn new(config: &RuntimeConfig) -> Result<Self> {
        // Create Wasmtime configuration
        let mut wasmtime_config = Config::new();
        
        // Configure features
        if config.enable_fuel {
            wasmtime_config.consume_fuel(true);
        }
        
        if config.enable_memory_limits {
            wasmtime_config.memory_init_cow(true);
        }
        
        if config.native_stack_trace {
            wasmtime_config.native_stack_trace(true);
        }
        
        if config.debug_info {
            wasmtime_config.debug_info(true);
        }
        
        // Configure compilation threads
        wasmtime_config.cranelift_set_compilation_threads(config.compilation_threads);
        
        // Configure caching
        if config.cache_modules {
            if let Some(cache_dir) = &config.cache_directory {
                wasmtime_config.cache_config_load(&cache_dir.to_string_lossy())
                    .map_err(|e| Error::RuntimeInitialization(
                        format!("Failed to configure cache: {}", e)
                    ))?;
            } else {
                // Use default cache location
                wasmtime_config.cache_config_load_default()
                    .map_err(|e| Error::RuntimeInitialization(
                        format!("Failed to configure default cache: {}", e)
                    ))?;
            }
        }
        
        // Create engine
        let engine = Engine::new(&wasmtime_config)
            .map_err(|e| Error::RuntimeInitialization(
                format!("Failed to create Wasmtime engine: {}", e)
            ))?;
        
        Ok(Self {
            engine,
            config: config.clone(),
            modules: DashMap::new(),
            metrics: Mutex::new(RuntimeMetrics {
                compiled_modules: 0,
                active_instances: 0,
                total_memory_usage: 0,
                peak_memory_usage: 0,
                fuel_consumption_rate: None,
                cache_hit_rate: None,
                last_compilation_time_ms: None,
            }),
        })
    }
}

impl WasmRuntime for WasmtimeRuntime {
    fn initialize(&mut self, config: RuntimeConfig) -> Result<()> {
        // Update configuration
        self.config = config;
        
        Ok(())
    }
    
    fn load_module(&self, wasm_bytes: &[u8]) -> Result<Box<dyn WasmModule>> {
        // Compile the module
        let start_time = std::time::Instant::now();
        
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| Error::ModuleLoad(format!("Failed to compile module: {}", e)))?;
        
        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.compiled_modules += 1;
            metrics.last_compilation_time_ms = Some(elapsed_ms);
        }
        
        // Create the module
        let module = WasmtimeModule::new(module, wasm_bytes);
        let id = module.id();
        
        // Store in the modules map
        let module_arc = Arc::new(module);
        self.modules.insert(id, module_arc.clone());
        
        Ok(Box::new(WasmtimeModule {
            id,
            name: module_arc.name.clone(),
            module: module_arc.module.clone(),
            exports: module_arc.exports.clone(),
            size: module_arc.size,
        }))
    }
    
    fn get_module(&self, id: ModuleId) -> Result<Arc<dyn WasmModule>> {
        // Get the module
        let module = self.modules.get(&id)
            .ok_or_else(|| Error::ModuleNotFound(format!("Module not found: {}", id)))?;
        
        // Return a clone of the Arc
        let module_clone: Arc<dyn WasmModule> = module.clone();
        Ok(module_clone)
    }
    
    fn create_instance(
        &self, 
        module: &dyn WasmModule, 
        resources: ResourceLimits,
        capabilities: Capabilities,
    ) -> Result<Box<dyn WasmInstance>> {
        // Get the module as a Wasmtime module
        let wasmtime_module = match module.clone_module().downcast::<WasmtimeModule>() {
            Ok(m) => m,
            Err(_) => return Err(Error::InstanceCreation(
                "Module is not a Wasmtime module".to_string()
            )),
        };
        
        // Create WASI context
        let mut wasi_builder = WasiCtxBuilder::new();
        
        // Configure capabilities
        match &capabilities.environment {
            crate::security::EnvironmentCapability::None => {
                // No environment variables
            },
            crate::security::EnvironmentCapability::Allowlist(vars) => {
                // Only allow specific variables
                let env_vars = std::env::vars()
                    .filter(|(k, _)| vars.contains(k))
                    .collect::<HashMap<String, String>>();
                
                for (k, v) in env_vars {
                    wasi_builder.env(&k, &v);
                }
            },
            crate::security::EnvironmentCapability::Denylist(vars) => {
                // Allow all except specific variables
                let env_vars = std::env::vars()
                    .filter(|(k, _)| !vars.contains(k))
                    .collect::<HashMap<String, String>>();
                
                for (k, v) in env_vars {
                    wasi_builder.env(&k, &v);
                }
            },
            crate::security::EnvironmentCapability::Full => {
                // Allow all environment variables
                for (k, v) in std::env::vars() {
                    wasi_builder.env(&k, &v);
                }
            },
        }
        
        // Configure filesystem capabilities
        for dir in &capabilities.filesystem.readable_dirs {
            wasi_builder.preopened_dir(dir, "/").map_err(|e| Error::InstanceCreation(
                format!("Failed to preopen directory: {}", e)
            ))?;
        }
        
        // Create the WASI context
        let wasi_ctx = wasi_builder.build();
        
        // Create the store
        let mut store = Store::new(
            &self.engine, 
            WasmtimeStoreData {
                wasi_ctx,
                state: WasmInstanceState::Created,
            }
        );
        
        // Set fuel if enabled
        if self.config.enable_fuel {
            if let Some(fuel) = resources.fuel {
                store.add_fuel(fuel).map_err(|e| Error::InstanceCreation(
                    format!("Failed to add fuel: {}", e)
                ))?;
            }
        }
        
        // Create linker
        let mut linker = Linker::new(&self.engine);
        
        // Add WASI to the linker
        wasmtime_wasi::add_to_linker(&mut linker, |s| &mut s.wasi_ctx)
            .map_err(|e| Error::InstanceCreation(
                format!("Failed to add WASI to linker: {}", e)
            ))?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.active_instances += 1;
        }
        
        // Create the instance
        let instance = WasmtimeInstance::new(
            store,
            Arc::new(linker),
            wasmtime_module.id(),
        );
        
        Ok(Box::new(instance))
    }
    
    fn get_metrics(&self) -> RuntimeMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    fn shutdown(&self) -> Result<()> {
        // Nothing to do for Wasmtime
        Ok(())
    }
}
