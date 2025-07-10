//! Wasmtime runtime implementation

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::any::Any;

use dashmap::DashMap;
use wasmtime::{Engine, Module, Store, Linker, Config, Val, Memory, Instance};
use wasmtime_wasi::{WasiCtx, sync::WasiCtxBuilder};

use crate::error::{Error, Result};
use crate::runtime::{
    ModuleId, RuntimeConfig, RuntimeMetrics, 
    WasmInstance, WasmInstanceState, WasmModule, WasmRuntime
};
use crate::security::{Capabilities, ResourceLimits};
use crate::runtime::wasm_common::{ToWasmValues, FromWasmValues};

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
        // Extract exports
        let mut exports = Vec::new();
        
        // Get all export names from the module
        for export in module.exports() {
            if let Some(name) = export.name() {
                exports.push(name.to_string());
            }
        }
        
        Self {
            id: ModuleId::new(),
            name: None,
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
    
    /// Cached memory export
    memory: Option<Memory>,
}

/// Wasmtime instance implementation
pub struct WasmtimeInstance {
    /// Store for the instance
    store: Store<WasmtimeStoreData>,
    
    /// Instance
    instance: Instance,
    
    /// Module ID
    module_id: ModuleId,
}

impl WasmtimeInstance {
    /// Create a new Wasmtime instance
    pub fn new(
        mut store: Store<WasmtimeStoreData>,
        instance: Instance,
        module_id: ModuleId,
    ) -> Result<Self> {
        // Cache the memory export if available
        let memory = instance
            .get_export(&mut store, "memory")
            .and_then(|ext| ext.into_memory());
        
        store.data_mut().memory = memory;
        
        // Update instance state
        store.data_mut().state = WasmInstanceState::Running;
        
        Ok(Self {
            store,
            instance,
            module_id,
        })
    }
    
    /// Helper method to get the memory instance
    fn get_memory(&self) -> Option<Memory> {
        self.store.data().memory
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
        // Get the function from the instance
        let func = match self.instance.get_func(&mut self.store.as_context_mut(), function_name) {
            Some(f) => f,
            None => {
                return Err(Error::FunctionCall {
                    function_name: function_name.to_string(),
                    reason: "Function not found in instance".to_string(),
                })
            }
        };
        
        // Serialize parameters to binary using msgpack
        let param_bytes = params.to_wasm_values();
        
        // Prepare for function call
        let mut results = vec![Val::I32(0)]; // Pointer to result data
        
        // Call the function
        match func.call(&mut self.store.as_context_mut(), &[Val::I32(param_bytes.len() as i32)], &mut results) {
            Ok(_) => {
                // Get result pointer from the returned value
                let result_ptr = match results[0] {
                    Val::I32(ptr) => ptr as u32,
                    _ => {
                        return Err(Error::FunctionCall {
                            function_name: function_name.to_string(),
                            reason: "Function returned an invalid result type".to_string(),
                        });
                    }
                };
                
                // Read result data from memory
                let memory = match self.get_memory() {
                    Some(mem) => mem,
                    None => {
                        return Err(Error::FunctionCall {
                            function_name: function_name.to_string(),
                            reason: "No memory available to read result".to_string(),
                        });
                    }
                };
                
                // Read length prefix (first 4 bytes)
                let mut length_bytes = [0u8; 4];
                memory.data(&self.store)
                    .get(result_ptr as usize..(result_ptr as usize + 4))
                    .ok_or_else(|| Error::FunctionCall {
                        function_name: function_name.to_string(),
                        reason: "Failed to read result length from memory".to_string(),
                    })?
                    .copy_to_slice(&mut length_bytes);
                
                let result_len = u32::from_le_bytes(length_bytes) as usize;
                
                // Read the actual result data
                let mut result_data = vec![0u8; result_len];
                memory.data(&self.store)
                    .get((result_ptr as usize + 4)..(result_ptr as usize + 4 + result_len))
                    .ok_or_else(|| Error::FunctionCall {
                        function_name: function_name.to_string(),
                        reason: "Failed to read result data from memory".to_string(),
                    })?
                    .copy_to_slice(&mut result_data);
                
                // Deserialize the result
                let result: R = FromWasmValues::from_wasm_values(&result_data);
                Ok(result)
            },
            Err(e) => {
                Err(Error::FunctionCall {
                    function_name: function_name.to_string(),
                    reason: format!("Function call failed: {}", e),
                })
            }
        }
    }
    
    fn memory_usage(&self) -> usize {
        if let Some(memory) = self.get_memory() {
            return memory.data_size(&self.store);
        }
        
        0
    }
    
    fn fuel_usage(&self) -> Option<u64> {
        match wasmtime::Fuel::get(&mut self.store.as_context_mut()) {
            Ok(Some(fuel_consumed)) => Some(fuel_consumed),
            _ => None,
        }
    }
    
    fn reset_fuel(&self) -> Result<()> {
        match wasmtime::Fuel::set(&mut self.store.as_context_mut(), 0) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::ResourceLimit(format!("Failed to reset fuel: {}", e))),
        }
    }
    
    fn add_fuel(&self, fuel: u64) -> Result<()> {
        match wasmtime::Fuel::add(&mut self.store.as_context_mut(), fuel) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::ResourceLimit(format!("Failed to add fuel: {}", e))),
        }
    }
    
    unsafe fn memory_ptr(&self) -> Result<*mut u8> {
        if let Some(memory) = self.get_memory() {
            let ptr = memory.data_ptr(&self.store);
            return Ok(ptr);
        }
        
        Err(Error::UnsupportedOperation("No memory exported by the module".to_string()))
    }
    
    fn memory_size(&self) -> usize {
        self.memory_usage()
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
        
        // Configure memory limits
        wasmtime_config.static_memory_maximum_size(config.enable_memory_limits.then_some(0x1_0000_0000)); // 4GB max
        
        if config.debug_info {
            wasmtime_config.debug_info(true);
        }
        
        // Configure compilation
        if config.compilation_threads > 0 {
            wasmtime_config.cranelift_opt_level(wasmtime::OptLevel::Speed);
            wasmtime_config.parallel_compilation(true);
        }
        
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
        let module = Arc::new(WasmtimeModule::new(module, wasm_bytes));
        let id = module.id();
        
        // Store in the modules map
        self.modules.insert(id, module.clone());
        
        Ok(Box::new(WasmtimeModule {
            id,
            name: module.name.clone(),
            module: module.module.clone(),
            exports: module.exports.clone(),
            size: module.size,
        }))
    }
    
    fn get_module(&self, id: ModuleId) -> Result<Arc<dyn WasmModule>> {
        // Get the module
        let module = self.modules.get(&id)
            .ok_or_else(|| Error::ModuleNotFound(format!("Module not found: {}", id)))?;
        
        // Return as Box<dyn WasmModule>
        // Note: This requires a bit of manual casting to satisfy the trait bounds
        let clone: Box<dyn WasmModule> = Box::new(WasmtimeModule {
            id: module.id,
            name: module.name.clone(),
            module: module.module.clone(),
            exports: module.exports.clone(),
            size: module.size,
        });
        
        Ok(Arc::from(clone))
    }
    
    fn create_instance(
        &self, 
        module: &dyn WasmModule, 
        resources: ResourceLimits,
        capabilities: Capabilities,
    ) -> Result<Box<dyn WasmInstance>> {
        // Try to downcast the module to a WasmtimeModule
        // This is a bit cumbersome with the trait object, but workable
        let wasmtime_module = if let Some(id) = self.modules.iter().find_map(|m| {
            if m.id() == module.id() {
                Some(m.id())
            } else {
                None
            }
        }) {
            self.modules.get(&id).unwrap().clone()
        } else {
            // If not found in our modules map, try to compile it again
            return Err(Error::InstanceCreation(
                "Module is not a valid Wasmtime module".to_string()
            ));
        };
        
        // Create WASI context builder
        let mut wasi_builder = WasiCtxBuilder::new();
        
        // Configure environment variables based on capabilities
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
                    wasi_builder = wasi_builder.env(&k, &v);
                }
            },
            crate::security::EnvironmentCapability::Denylist(vars) => {
                // Allow all except specific variables
                let env_vars = std::env::vars()
                    .filter(|(k, _)| !vars.contains(k))
                    .collect::<HashMap<String, String>>();
                
                for (k, v) in env_vars {
                    wasi_builder = wasi_builder.env(&k, &v);
                }
            },
            crate::security::EnvironmentCapability::Full => {
                // Allow all environment variables
                for (k, v) in std::env::vars() {
                    wasi_builder = wasi_builder.env(&k, &v);
                }
            },
        }
        
        // Configure filesystem capabilities
        for dir in &capabilities.filesystem.readable_dirs {
            wasi_builder = wasi_builder.preopened_dir(dir, "/")
                .map_err(|e| Error::InstanceCreation(
                    format!("Failed to preopen directory: {}", e)
                ))?;
        }
        
        // Build the WASI context
        let wasi_ctx = wasi_builder.build();
        
        // Create the store
        let mut store = Store::new(
            &self.engine, 
            WasmtimeStoreData {
                wasi_ctx,
                state: WasmInstanceState::Created,
                memory: None,
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
        
        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &wasmtime_module.module)
            .map_err(|e| Error::InstanceCreation(
                format!("Failed to instantiate module: {}", e)
            ))?;
        
        // Create the instance
        let instance = WasmtimeInstance::new(
            store,
            instance,
            wasmtime_module.id,
        )?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.active_instances += 1;
        }
        
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
