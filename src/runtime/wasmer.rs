//! Wasmer runtime implementation

use std::any::Any;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use wasmer::{Engine, Module, Store, Instance, Value, Memory, imports};

use crate::error::{Error, Result};
use crate::runtime::{
    ModuleId, RuntimeConfig, RuntimeMetrics, WasmInstanceState,
    WasmInstance, WasmModule, WasmRuntime, WasmFunctionCaller, WasmFunctionCallerAsync
};
use crate::security::{Capabilities, ResourceLimits};

/// Wasmer module implementation
pub struct WasmerModule {
    /// Module ID
    id: ModuleId,
    
    /// Module name
    name: Option<String>,
    
    /// Wasmer module
    module: Module,
    
    /// Module exports
    exports: Vec<String>,
    
    /// Module size in bytes
    size: usize,
}

impl WasmerModule {
    /// Create a new Wasmer module
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
    
    /// Get a reference to the Wasmer module
    pub fn module(&self) -> &Module {
        &self.module
    }
}

impl WasmModule for WasmerModule {
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Wasmer instance implementation
pub struct WasmerInstance {
    /// Store for the instance (with interior mutability)
    store: RwLock<Store>,
    
    /// Instance
    instance: Instance,
    
    /// Module ID
    #[allow(dead_code)]
    module_id: ModuleId,
    
    /// Instance state
    state: RwLock<WasmInstanceState>,
    
    /// Fuel usage tracking
    fuel_usage: RwLock<Option<u64>>,
}

impl WasmerInstance {
    /// Create a new Wasmer instance
    pub fn new(
        store: Store,
        instance: Instance,
        module_id: ModuleId,
    ) -> Result<Self> {
        let initial_state = WasmInstanceState::Running;
        
        Ok(Self {
            store: RwLock::new(store),
            instance,
            module_id,
            state: RwLock::new(initial_state),
            fuel_usage: RwLock::new(None),
        })
    }
    
    /// Helper method to get the memory instance
    fn get_memory(&self) -> Option<Memory> {
        self.instance.exports.get_memory("memory").ok().cloned()
    }
}

impl WasmInstance for WasmerInstance {
    fn state(&self) -> WasmInstanceState {
        *self.state.read().unwrap()
    }

    fn memory_usage(&self) -> usize {
        if let Some(memory) = self.get_memory() {
            let store = self.store.read().unwrap();
            let size = memory.view(&store).size();
            size.bytes().0
        } else {
            0
        }
    }

    fn fuel_usage(&self) -> Option<u64> {
        *self.fuel_usage.read().unwrap()
    }

    fn reset_fuel(&self) -> Result<()> {
        let mut fuel_usage = self.fuel_usage.write().unwrap();
        *fuel_usage = Some(0);
        Ok(())
    }

    fn add_fuel(&self, fuel: u64) -> Result<()> {
        let mut fuel_usage = self.fuel_usage.write().unwrap();
        match fuel_usage.as_mut() {
            Some(current) => *current += fuel,
            None => *fuel_usage = Some(fuel),
        }
        Ok(())
    }

    unsafe fn memory_ptr(&self) -> Result<*mut u8> {
        if let Some(memory) = self.get_memory() {
            let store = self.store.read().unwrap();
            let view = memory.view(&store);
            Ok(view.data_ptr())
        } else {
            Err(Error::InstanceCreation {
                reason: "Memory export not found".to_string(),
                instance_id: Some(self.module_id.as_uuid()),
            })
        }
    }

    fn memory_size(&self) -> usize {
        self.memory_usage()
    }

    fn function_caller(&self) -> Box<dyn WasmFunctionCaller> {
        Box::new(WasmerFunctionCaller::new(self.module_id))
    }

    fn call_simple_function(&self, function_name: &str, params: &[i32]) -> Result<i32> {
        // Get the function from the instance
        let function = self.instance.exports.get_function(function_name)
            .map_err(|_| Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: "Function not found".to_string(),
            })?;
        
        // Convert parameters to Wasmer values
        let wasmer_params: Vec<Value> = params.iter()
            .map(|&p| Value::I32(p))
            .collect();
        
        // Call the function
        let mut store = self.store.write().unwrap();
        let results = function.call(&mut *store, &wasmer_params)
            .map_err(|e| Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: e.to_string(),
            })?;
        
        // Extract the result
        if let Some(Value::I32(result)) = results.first() {
            Ok(*result)
        } else {
            Err(Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: "Function did not return an i32".to_string(),
            })
        }
    }
}

/// Function caller implementation for Wasmer
pub struct WasmerFunctionCaller {
    #[allow(dead_code)]
    module_id: ModuleId,
}

impl WasmerFunctionCaller {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }
}

impl WasmFunctionCaller for WasmerFunctionCaller {
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
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl WasmFunctionCallerAsync for WasmerFunctionCaller {
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

/// Wasmer runtime implementation
pub struct WasmerRuntime {
    /// Wasmer engine
    engine: Engine,
    
    /// Module cache
    modules: RwLock<HashMap<ModuleId, Arc<WasmerModule>>>,
    
    /// Runtime metrics
    metrics: RwLock<RuntimeMetrics>,
    
    /// Runtime configuration
    config: RwLock<Option<RuntimeConfig>>,
}

impl WasmerRuntime {
    /// Create a new Wasmer runtime
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        
        let metrics = RuntimeMetrics {
            compiled_modules: 0,
            active_instances: 0,
            total_memory_usage: 0,
            peak_memory_usage: 0,
            fuel_consumption_rate: None,
            cache_hit_rate: None,
            last_compilation_time_ms: None,
        };
        
        Ok(Self {
            engine,
            modules: RwLock::new(HashMap::new()),
            metrics: RwLock::new(metrics),
            config: RwLock::new(None),
        })
    }
    
    /// Create a WASI environment with the given capabilities
    fn create_wasi_env(&self, _capabilities: &Capabilities) -> Result<()> {
        // For now, just return Ok(()) - we'll implement full WASI later
        // when we have a better understanding of the wasmer-wasix API
        Ok(())
    }
}

impl WasmRuntime for WasmerRuntime {
    fn initialize(&mut self, config: RuntimeConfig) -> Result<()> {
        *self.config.write().unwrap() = Some(config);
        Ok(())
    }
    
    fn load_module(&self, wasm_bytes: &[u8]) -> Result<Box<dyn WasmModule>> {
        let start_time = std::time::Instant::now();
        
        // Compile the module
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| Error::Module {
                operation: "compilation".to_string(),
                reason: e.to_string(),
                suggestion: Some("Check that the WebAssembly binary is valid".to_string()),
            })?;
        
        // Create our wrapper
        let wasmer_module = WasmerModule::new(module, wasm_bytes);
        let module_id = wasmer_module.id();
        
        // Store in cache
        {
            let mut modules = self.modules.write().unwrap();
            modules.insert(module_id, Arc::new(wasmer_module));
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.compiled_modules += 1;
            metrics.last_compilation_time_ms = Some(start_time.elapsed().as_millis() as u64);
        }
        
        // Return the module
        self.get_module(module_id)
            .map(|arc_module| {
                let module: &WasmerModule = arc_module.as_any().downcast_ref().unwrap();
                module.clone_module()
            })
    }
    
    fn get_module(&self, id: ModuleId) -> Result<Arc<dyn WasmModule>> {
        let modules = self.modules.read().unwrap();
        modules.get(&id)
            .cloned()
            .map(|m| m as Arc<dyn WasmModule>)
            .ok_or(Error::NotFound {
                resource_type: "module".to_string(),
                identifier: id.to_string(),
            })
    }
    
    fn get_module_ids(&self) -> Vec<ModuleId> {
        let modules = self.modules.read().unwrap();
        modules.keys().cloned().collect()
    }
    
    fn create_instance(
        &self,
        module: &dyn WasmModule,
        _resource_limits: ResourceLimits,
        capabilities: Capabilities,
    ) -> Result<Box<dyn WasmInstance>> {
        // Get the actual Wasmer module
        let wasmer_module = module.as_any()
            .downcast_ref::<WasmerModule>()
            .ok_or(Error::InstanceCreation {
                reason: "Module is not a WasmerModule".to_string(),
                instance_id: None,
            })?;
        
        // Create store
        let mut store = Store::new(self.engine.clone());
        
        // Create WASI environment (simplified for now)
        self.create_wasi_env(&capabilities)?;
        
        // Create imports (empty for now - we'll add WASI imports later)
        let imports = imports! {};
        
        // Create instance
        let instance = Instance::new(&mut store, &wasmer_module.module, &imports)
            .map_err(|e| Error::InstanceCreation {
                reason: e.to_string(),
                instance_id: Some(module.id().as_uuid()),
            })?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.active_instances += 1;
        }
        
        // Create our instance wrapper
        let wasmer_instance = WasmerInstance::new(store, instance, module.id())?;
        
        Ok(Box::new(wasmer_instance))
    }
    
    fn get_metrics(&self) -> RuntimeMetrics {
        self.metrics.read().unwrap().clone()
    }
    
    fn shutdown(&self) -> Result<()> {
        // Clear module cache
        {
            let mut modules = self.modules.write().unwrap();
            modules.clear();
        }
        
        // Reset metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.compiled_modules = 0;
            metrics.active_instances = 0;
            metrics.total_memory_usage = 0;
            metrics.peak_memory_usage = 0;
            metrics.fuel_consumption_rate = None;
            metrics.cache_hit_rate = None;
            metrics.last_compilation_time_ms = None;
        }
        
        Ok(())
    }
}