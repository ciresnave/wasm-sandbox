//! Wasmer runtime implementation

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, mpsc};

use wasmer::{Engine, Instance, Module, Store, Value, imports};

use crate::error::{Error, Result};
use crate::runtime::{
    ModuleId, RuntimeConfig, RuntimeMetrics, WasmFunctionCaller, WasmFunctionCallerAsync,
    WasmInstance, WasmInstanceState, WasmModule, WasmRuntime,
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

/// A request to the thread that owns a Wasmer `Store`.
enum StoreRequest {
    MemoryUsage(mpsc::Sender<usize>),
    MemoryPtr(mpsc::Sender<Option<usize>>),
    Call {
        function_name: String,
        params: Vec<i32>,
        reply: mpsc::Sender<Result<i32>>,
    },
}

/// Wasmer instance implementation
///
/// Wasmer 7's `Store` is deliberately neither `Send` nor `Sync`, so it (and the `Instance`
/// and `Memory` handles tied to it) never leaves the thread that created it. This type holds
/// only a channel to that thread, which keeps `WasmInstance: Send + Sync` without `unsafe`.
/// The thread exits when this value is dropped.
pub struct WasmerInstance {
    /// Channel to the thread that owns the store and instance
    requests: mpsc::Sender<StoreRequest>,

    /// Module ID
    #[allow(dead_code)]
    module_id: ModuleId,

    /// Instance state
    state: RwLock<WasmInstanceState>,

    /// Fuel usage tracking
    fuel_usage: RwLock<Option<u64>>,
}

impl WasmerInstance {
    /// Create a new Wasmer instance: spawns the store-owning thread and instantiates `module`
    /// on it, returning once instantiation has succeeded or failed.
    pub fn new(engine: Engine, module: Module, module_id: ModuleId) -> Result<Self> {
        let (requests, inbox) = mpsc::channel::<StoreRequest>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<()>>();

        std::thread::Builder::new()
            .name("wasmer-store".to_string())
            .spawn(move || {
                let mut store = Store::new(engine);
                // Imports are empty for now - WASI imports are not yet wired up
                let imports = imports! {};
                let instance = match Instance::new(&mut store, &module, &imports) {
                    Ok(instance) => instance,
                    Err(e) => {
                        let _ = ready_tx.send(Err(Error::InstanceCreation {
                            reason: e.to_string(),
                            instance_id: Some(module_id.as_uuid()),
                        }));
                        return;
                    }
                };
                let memory = instance.exports.get_memory("memory").ok().cloned();
                let _ = ready_tx.send(Ok(()));

                while let Ok(request) = inbox.recv() {
                    match request {
                        StoreRequest::MemoryUsage(reply) => {
                            let size = memory
                                .as_ref()
                                .map_or(0, |m| m.view(&store).size().bytes().0);
                            let _ = reply.send(size);
                        }
                        StoreRequest::MemoryPtr(reply) => {
                            let ptr = memory.as_ref().map(|m| m.view(&store).data_ptr() as usize);
                            let _ = reply.send(ptr);
                        }
                        StoreRequest::Call {
                            function_name,
                            params,
                            reply,
                        } => {
                            let _ = reply.send(call_i32(
                                &mut store,
                                &instance,
                                &function_name,
                                &params,
                            ));
                        }
                    }
                }
            })
            .map_err(|e| Error::InstanceCreation {
                reason: format!("Failed to spawn Wasmer store thread: {}", e),
                instance_id: Some(module_id.as_uuid()),
            })?;

        ready_rx.recv().map_err(|_| Error::InstanceCreation {
            reason: "Wasmer store thread exited during instantiation".to_string(),
            instance_id: Some(module_id.as_uuid()),
        })??;

        Ok(Self {
            requests,
            module_id,
            state: RwLock::new(WasmInstanceState::Running),
            fuel_usage: RwLock::new(None),
        })
    }

    /// Send a request and wait for the store thread's answer, if it is still running.
    fn ask<T>(&self, make: impl FnOnce(mpsc::Sender<T>) -> StoreRequest) -> Option<T> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.requests.send(make(reply_tx)).ok()?;
        reply_rx.recv().ok()
    }
}

/// Call an exported `i32` function on the store-owning thread.
fn call_i32(
    store: &mut Store,
    instance: &Instance,
    function_name: &str,
    params: &[i32],
) -> Result<i32> {
    let function =
        instance
            .exports
            .get_function(function_name)
            .map_err(|_| Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: "Function not found".to_string(),
            })?;

    let wasmer_params: Vec<Value> = params.iter().map(|&p| Value::I32(p)).collect();
    let results = function
        .call(store, &wasmer_params)
        .map_err(|e| Error::FunctionCall {
            function_name: function_name.to_string(),
            reason: e.to_string(),
        })?;

    if let Some(Value::I32(result)) = results.first() {
        Ok(*result)
    } else {
        Err(Error::FunctionCall {
            function_name: function_name.to_string(),
            reason: "Function did not return an i32".to_string(),
        })
    }
}

impl WasmInstance for WasmerInstance {
    fn state(&self) -> WasmInstanceState {
        *self.state.read().unwrap()
    }

    fn memory_usage(&self) -> usize {
        self.ask(StoreRequest::MemoryUsage).unwrap_or(0)
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
        match self.ask(StoreRequest::MemoryPtr).flatten() {
            Some(ptr) => Ok(ptr as *mut u8),
            None => Err(Error::InstanceCreation {
                reason: "Memory export not found".to_string(),
                instance_id: Some(self.module_id.as_uuid()),
            }),
        }
    }

    fn memory_size(&self) -> usize {
        self.memory_usage()
    }

    fn function_caller(&self) -> Box<dyn WasmFunctionCaller> {
        Box::new(WasmerFunctionCaller::new(self.module_id))
    }

    fn call_simple_function(&self, function_name: &str, params: &[i32]) -> Result<i32> {
        self.ask(|reply| StoreRequest::Call {
            function_name: function_name.to_string(),
            params: params.to_vec(),
            reply,
        })
        .unwrap_or_else(|| {
            Err(Error::FunctionCall {
                function_name: function_name.to_string(),
                reason: "Wasmer store thread is no longer running".to_string(),
            })
        })
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
    fn call_function_json(&self, function_name: &str, params_json: &str) -> Result<String> {
        // For now, implement a simple echo function
        // In a real implementation, this would call the actual WASM function
        // and handle serialization/deserialization properly
        Ok(format!(
            r#"{{"result": "Function {} called with params: {}", "success": true}}"#,
            function_name, params_json
        ))
    }

    fn call_function_msgpack(&self, function_name: &str, params_msgpack: &[u8]) -> Result<Vec<u8>> {
        // Convert to JSON for now
        let params_json = String::from_utf8_lossy(params_msgpack);
        let result_json =
            WasmFunctionCaller::call_function_json(self, function_name, &params_json)?;
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
    ///
    /// The Wasmer backend does not wire up WASI yet (instances get empty imports), so no
    /// capability beyond the defaults can be applied. A grant that would be ignored is rejected
    /// rather than silently accepted.
    fn create_wasi_env(&self, capabilities: &Capabilities) -> Result<()> {
        use crate::security::{EnvironmentCapability, NetworkCapability, ProcessCapability};

        let fs = &capabilities.filesystem;
        let unsupported = capabilities.network != NetworkCapability::None
            || capabilities.process != ProcessCapability::None
            || capabilities.environment != EnvironmentCapability::None
            || !fs.readable_dirs.is_empty()
            || !fs.writable_dirs.is_empty();
        if unsupported {
            return Err(Error::Capability {
                message: "the Wasmer backend does not apply network, process, environment or                           filesystem capabilities yet; use Capabilities::minimal() or the Wasmtime                           backend"
                    .to_string(),
            });
        }
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
        let module = Module::new(&self.engine, wasm_bytes).map_err(|e| Error::Module {
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
        self.get_module(module_id).map(|arc_module| {
            let module: &WasmerModule = arc_module.as_any().downcast_ref().unwrap();
            module.clone_module()
        })
    }

    fn get_module(&self, id: ModuleId) -> Result<Arc<dyn WasmModule>> {
        let modules = self.modules.read().unwrap();
        modules
            .get(&id)
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
        let wasmer_module =
            module
                .as_any()
                .downcast_ref::<WasmerModule>()
                .ok_or(Error::InstanceCreation {
                    reason: "Module is not a WasmerModule".to_string(),
                    instance_id: None,
                })?;

        // Create WASI environment (simplified for now)
        self.create_wasi_env(&capabilities)?;

        // Instantiate on the thread that will own the (non-Send) store
        let wasmer_instance = WasmerInstance::new(
            self.engine.clone(),
            wasmer_module.module.clone(),
            module.id(),
        )?;

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.active_instances += 1;
        }

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
