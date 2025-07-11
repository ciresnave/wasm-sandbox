//! Wasmtime runtime implementation

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use dashmap::DashMap;
use wasmtime::{Engine, Module, Store, Linker, Config, Val, Memory, Instance};
use wasmtime_wasi::{WasiCtx, sync::WasiCtxBuilder};

use crate::error::{Error, Result};
use crate::runtime::{
    ModuleId, RuntimeConfig, RuntimeMetrics, 
    WasmInstance, WasmInstanceState, WasmModule, WasmRuntime, WasmFunctionCaller, WasmFunctionCallerAsync
};
use crate::security::{Capabilities, ResourceLimits};
// Removed unused imports

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
            exports.push(export.name().to_string());
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
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Store data for Wasmtime instance
pub struct WasmtimeStoreData {
    /// WASI context
    pub wasi: WasiCtx,
    
    /// Instance state
    state: WasmInstanceState,
    
    /// Cached memory export
    memory: Option<Memory>,
}

/// Wasmtime instance implementation
pub struct WasmtimeInstance {
    /// Store for the instance (with interior mutability)
    store: RwLock<Store<WasmtimeStoreData>>,
    
    /// Instance
    instance: Instance,
    
    /// Module ID
    #[allow(dead_code)]
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
            store: RwLock::new(store),
            instance,
            module_id,
        })
    }
    
    /// Helper method to get the memory instance
    fn get_memory(&self) -> Option<Memory> {
        self.store.read().unwrap().data().memory
    }
}

/// Function caller implementation for Wasmtime
pub struct WasmtimeFunctionCaller;

impl WasmtimeFunctionCaller {
    pub fn new() -> Self {
        Self
    }
}

impl WasmFunctionCaller for WasmtimeFunctionCaller {
    fn call_function_json(
        &self,
        function_name: &str,
        params_json: &str,
    ) -> Result<String> {
        // For now, implement a simple echo function
        // In a real implementation, this would call the actual WASM function
        // and handle serialization/deserialization properly
        Ok(format!(r#"{{"result": "Function {} called with params: {}", "success": true}}"#, 
                  function_name, params_json))
    }
    
    fn call_function_msgpack(
        &self,
        function_name: &str,
        params_msgpack: &[u8],
    ) -> Result<Vec<u8>> {
        // Convert to JSON for now
        let params_json = String::from_utf8_lossy(params_msgpack);
        let result_json = WasmFunctionCaller::call_function_json(self, function_name, &params_json)?;
        Ok(result_json.into_bytes())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl WasmFunctionCallerAsync for WasmtimeFunctionCaller {
    async fn call_function_json_async(
        &self,
        function_name: &str,
        params_json: &str,
    ) -> Result<String> {
        // For now, just call the sync version
        // In a real implementation, this would be truly async
        self.call_function_json(function_name, params_json)
    }
    
    async fn call_function_msgpack_async(
        &self,
        function_name: &str,
        params_msgpack: &[u8],
    ) -> Result<Vec<u8>> {
        // For now, just call the sync version
        // In a real implementation, this would be truly async
        self.call_function_msgpack(function_name, params_msgpack)
    }
}

impl WasmInstance for WasmtimeInstance {
    fn state(&self) -> WasmInstanceState {
        self.store.read().unwrap().data().state
    }
    
    fn memory_usage(&self) -> usize {
        if let Some(memory) = self.get_memory() {
            let store = self.store.read().unwrap();
            return memory.data_size(&*store);
        }
        
        0
    }
    
    fn fuel_usage(&self) -> Option<u64> {
        // Note: Fuel API has changed in Wasmtime 13.0
        // For now, return None until we implement the correct API
        None
    }
    
    fn reset_fuel(&self) -> Result<()> {
        // Fuel API needs to be updated for Wasmtime 13.0
        Ok(())
    }
    
    fn add_fuel(&self, _fuel: u64) -> Result<()> {
        // Fuel API needs to be updated for Wasmtime 13.0
        Ok(())
    }
    
    unsafe fn memory_ptr(&self) -> Result<*mut u8> {
        let memory = self.get_memory().ok_or_else(|| 
            Error::UnsupportedOperation("No memory exported by the module".to_string())
        )?;
        
        let store = self.store.read().unwrap();
        let ptr = memory.data_ptr(&*store);
        Ok(ptr)
    }
    
    fn memory_size(&self) -> usize {
        self.memory_usage()
    }
    
    fn function_caller(&self) -> Box<dyn WasmFunctionCaller> {
        // Return a simple function caller for now
        Box::new(WasmtimeFunctionCaller::new()) as Box<dyn WasmFunctionCaller>
    }
    
    fn call_simple_function(&self, function_name: &str, params: &[i32]) -> Result<i32> {
        let mut store_guard = self.store.write().unwrap();
        
        // Get the function export
        let func = self.instance
            .get_func(&mut *store_guard, function_name)
            .ok_or_else(|| Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: "Function not found".to_string(),
            })?;
        
        // Convert parameters to wasmtime values
        let args: Vec<Val> = params.iter().map(|&p| Val::I32(p)).collect();
        
        // Call the function
        let mut results = vec![Val::I32(0)]; // Pre-allocate result
        func.call(&mut *store_guard, &args, &mut results)
            .map_err(|e| Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: format!("Call failed: {}", e),
            })?;
        
        // Extract the result
        match &results[0] {
            Val::I32(result) => Ok(*result),
            _ => Err(Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: "Unexpected return type".to_string(),
            }),
        }
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
        if config.enable_memory_limits {
            wasmtime_config.static_memory_maximum_size(4 * 1024 * 1024 * 1024); // 4GB max
        }
        
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
                wasmtime_config.cache_config_load(cache_dir.to_string_lossy().as_ref())
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
        
        // Return as Arc<dyn WasmModule>
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
        // Try to downcast the module to a WasmtimeModule using the modules map
        let wasmtime_module = if let Some(id) = self.modules.iter().find_map(|m| {
            if m.id() == module.id() {
                Some(m.id())
            } else {
                None
            }
        }) {
            self.modules.get(&id).unwrap().clone()
        } else {
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
                    if let Err(e) = wasi_builder.env(&k, &v) {
                        return Err(Error::InstanceCreation(
                            format!("Failed to set env var {}: {}", k, e)
                        ));
                    }
                }
            },
            crate::security::EnvironmentCapability::Denylist(vars) => {
                // Allow all except specific variables
                let env_vars = std::env::vars()
                    .filter(|(k, _)| !vars.contains(k))
                    .collect::<HashMap<String, String>>();
                
                for (k, v) in env_vars {
                    if let Err(e) = wasi_builder.env(&k, &v) {
                        return Err(Error::InstanceCreation(
                            format!("Failed to set env var {}: {}", k, e)
                        ));
                    }
                }
            },
            crate::security::EnvironmentCapability::Full => {
                // Allow all environment variables
                for (k, v) in std::env::vars() {
                    if let Err(e) = wasi_builder.env(&k, &v) {
                        return Err(Error::InstanceCreation(
                            format!("Failed to set env var {}: {}", k, e)
                        ));
                    }
                }
            },
        }
        
        // Configure filesystem capabilities - simplified version for now
        // In a real implementation, this would properly handle directory access permissions
        if !capabilities.filesystem.readable_dirs.is_empty() {
            // Just log that we'd be configuring dirs
            log::warn!("Directory access capabilities are not yet fully implemented");
        }
        
        // Build the WASI context
        let wasi_ctx = wasi_builder.build();
        
        // Create the store
        let mut store = Store::new(
            &self.engine, 
            WasmtimeStoreData {
                wasi: wasi_ctx,
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
        wasmtime_wasi::add_to_linker(&mut linker, |s: &mut WasmtimeStoreData| &mut s.wasi)
            .map_err(|e| Error::InstanceCreation(
                format!("Failed to add WASI to linker: {}", e)
            ))?;
        
        // Add "env" memory if needed by the module
        let memory_type = wasmtime::MemoryType::new(1, None);
        let memory = Memory::new(&mut store, memory_type)
            .map_err(|e| Error::InstanceCreation(
                format!("Failed to create memory: {}", e)
            ))?;
        
        linker.define(&mut store, "env", "memory", memory)
            .map_err(|e| Error::InstanceCreation(
                format!("Failed to define env memory: {}", e)
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
        
        Ok(Box::new(instance) as Box<dyn WasmInstance>)
    }
    
    fn get_metrics(&self) -> RuntimeMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    fn get_module_ids(&self) -> Vec<ModuleId> {
        self.modules.iter().map(|entry| *entry.key()).collect()
    }
    
    fn shutdown(&self) -> Result<()> {
        // Nothing specific to do for Wasmtime
        Ok(())
    }
}
