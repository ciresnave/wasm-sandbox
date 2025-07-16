# Runtime Abstraction Design

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This design document outlines the runtime abstraction layer that enables wasm-sandbox to support multiple WebAssembly runtime engines while providing a unified, high-level API.

## Design Goals

The runtime abstraction layer aims to:

1. **Unified Interface** - Single API regardless of underlying runtime
2. **Runtime Agnostic** - Support multiple WebAssembly engines seamlessly
3. **Feature Parity** - Expose common features across all runtimes
4. **Performance** - Minimize abstraction overhead
5. **Extensibility** - Easy to add new runtime backends
6. **Configuration** - Runtime-specific optimizations when needed

## Architecture Overview

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Main runtime abstraction trait
#[async_trait]
pub trait WasmRuntime: Send + Sync + 'static {
    type Module: WasmModule;
    type Instance: WasmInstance;
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Runtime identification
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn capabilities(&self) -> RuntimeCapabilities;
    
    /// Module compilation
    async fn compile_module(&self, code: &[u8]) -> Result<Self::Module, Self::Error>;
    async fn compile_module_from_file(&self, path: &std::path::Path) -> Result<Self::Module, Self::Error>;
    async fn validate_module(&self, code: &[u8]) -> Result<(), Self::Error>;
    
    /// Instance creation and management
    async fn instantiate(&self, module: &Self::Module) -> Result<Self::Instance, Self::Error>;
    async fn instantiate_with_config(
        &self, 
        module: &Self::Module, 
        config: &InstanceConfig
    ) -> Result<Self::Instance, Self::Error>;
    
    /// Resource management
    async fn set_resource_limits(&self, limits: &ResourceLimits) -> Result<(), Self::Error>;
    async fn get_resource_usage(&self) -> Result<ResourceUsage, Self::Error>;
    
    /// Runtime configuration
    async fn configure(&mut self, config: &RuntimeConfig) -> Result<(), Self::Error>;
    async fn get_configuration(&self) -> RuntimeConfig;
    
    /// Runtime lifecycle
    async fn initialize(&mut self) -> Result<(), Self::Error>;
    async fn shutdown(&mut self) -> Result<(), Self::Error>;
    
    /// Debugging and introspection
    async fn get_runtime_info(&self) -> RuntimeInfo;
    async fn collect_metrics(&self) -> RuntimeMetrics;
}

/// WebAssembly module abstraction
#[async_trait]
pub trait WasmModule: Send + Sync + Clone {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Module metadata
    fn module_id(&self) -> &str;
    fn module_hash(&self) -> &[u8];
    fn size(&self) -> usize;
    fn creation_time(&self) -> chrono::DateTime<chrono::Utc>;
    
    /// Module introspection
    fn get_exports(&self) -> Vec<Export>;
    fn get_imports(&self) -> Vec<Import>;
    fn get_memory_info(&self) -> MemoryInfo;
    fn get_table_info(&self) -> Vec<TableInfo>;
    
    /// Module validation
    async fn validate(&self) -> Result<ValidationResult, Self::Error>;
    async fn analyze_security(&self) -> SecurityAnalysisResult;
    
    /// Serialization
    async fn serialize(&self) -> Result<Vec<u8>, Self::Error>;
    async fn deserialize(data: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// WebAssembly instance abstraction
#[async_trait]
pub trait WasmInstance: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Instance metadata
    fn instance_id(&self) -> &str;
    fn module_id(&self) -> &str;
    fn creation_time(&self) -> chrono::DateTime<chrono::Utc>;
    fn state(&self) -> InstanceState;
    
    /// Function calls
    async fn call_function<T, R>(&mut self, name: &str, args: T) -> Result<R, Self::Error>
    where
        T: Serialize + Send,
        R: for<'de> Deserialize<'de> + Send;
    
    async fn call_function_raw(&mut self, name: &str, args: &[WasmValue]) -> Result<Vec<WasmValue>, Self::Error>;
    
    /// Memory management
    async fn read_memory(&self, offset: u32, size: u32) -> Result<Vec<u8>, Self::Error>;
    async fn write_memory(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error>;
    async fn get_memory_size(&self) -> Result<u32, Self::Error>;
    async fn grow_memory(&mut self, pages: u32) -> Result<u32, Self::Error>;
    
    /// Global variables
    async fn get_global(&self, name: &str) -> Result<WasmValue, Self::Error>;
    async fn set_global(&mut self, name: &str, value: WasmValue) -> Result<(), Self::Error>;
    
    /// Table operations
    async fn get_table_element(&self, table: &str, index: u32) -> Result<WasmValue, Self::Error>;
    async fn set_table_element(&mut self, table: &str, index: u32, value: WasmValue) -> Result<(), Self::Error>;
    
    /// Resource monitoring
    async fn get_resource_usage(&self) -> Result<ResourceUsage, Self::Error>;
    async fn get_execution_stats(&self) -> ExecutionStats;
    
    /// Instance lifecycle
    async fn pause(&mut self) -> Result<(), Self::Error>;
    async fn resume(&mut self) -> Result<(), Self::Error>;
    async fn reset(&mut self) -> Result<(), Self::Error>;
    async fn terminate(&mut self) -> Result<(), Self::Error>;
    
    /// Debugging
    async fn get_stack_trace(&self) -> Result<Vec<StackFrame>, Self::Error>;
    async fn set_breakpoint(&mut self, location: BreakpointLocation) -> Result<BreakpointId, Self::Error>;
    async fn remove_breakpoint(&mut self, id: BreakpointId) -> Result<(), Self::Error>;
}
```

## Runtime Capabilities

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeCapabilities {
    /// Core WebAssembly features
    pub wasm_features: WasmFeatures,
    /// Memory management capabilities
    pub memory_management: MemoryCapabilities,
    /// Security features
    pub security_features: SecurityFeatures,
    /// Performance features
    pub performance_features: PerformanceFeatures,
    /// Debugging capabilities
    pub debugging_capabilities: DebuggingCapabilities,
    /// Host integration features
    pub host_integration: HostIntegrationFeatures,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WasmFeatures {
    pub mvp: bool,                    // WebAssembly MVP
    pub bulk_memory: bool,            // Bulk memory operations
    pub reference_types: bool,        // Reference types
    pub simd: bool,                   // SIMD operations
    pub multi_value: bool,            // Multi-value returns
    pub tail_call: bool,              // Tail calls
    pub threads: bool,                // Threading support
    pub exception_handling: bool,     // Exception handling
    pub memory64: bool,               // 64-bit memory
    pub component_model: bool,        // WebAssembly Component Model
    pub gc: bool,                     // Garbage collection
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryCapabilities {
    pub linear_memory: bool,          // Linear memory support
    pub shared_memory: bool,          // Shared memory between instances
    pub memory_protection: bool,      // Memory protection mechanisms
    pub copy_on_write: bool,          // Copy-on-write memory
    pub memory_mapping: bool,         // Memory mapping from host
    pub custom_allocators: bool,      // Custom memory allocators
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityFeatures {
    pub capability_system: bool,      // Capability-based security
    pub resource_limits: bool,        // Resource limiting
    pub execution_metering: bool,     // Fuel/gas metering
    pub stack_overflow_protection: bool,
    pub control_flow_integrity: bool,
    pub address_space_randomization: bool,
    pub secure_compilation: bool,     // Spectre/Meltdown mitigations
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceFeatures {
    pub jit_compilation: bool,        // Just-in-time compilation
    pub aot_compilation: bool,        // Ahead-of-time compilation
    pub optimization_levels: Vec<OptimizationLevel>,
    pub profiling: bool,              // Performance profiling
    pub parallel_compilation: bool,   // Parallel module compilation
    pub caching: bool,                // Compiled module caching
    pub streaming_compilation: bool,  // Streaming compilation
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Speed,
    Size,
    Aggressive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebuggingCapabilities {
    pub breakpoints: bool,            // Breakpoint support
    pub stepping: bool,               // Step-by-step execution
    pub stack_inspection: bool,       // Stack frame inspection
    pub variable_inspection: bool,    // Variable value inspection
    pub memory_inspection: bool,      // Memory content inspection
    pub dwarf_debugging: bool,        // DWARF debug info support
    pub source_mapping: bool,         // Source map support
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostIntegrationFeatures {
    pub host_functions: bool,         // Host function calls
    pub memory_sharing: bool,         // Shared memory with host
    pub callback_support: bool,       // Callbacks from WASM to host
    pub async_support: bool,          // Async function calls
    pub streaming_io: bool,           // Streaming I/O operations
    pub serialization_formats: Vec<SerializationFormat>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SerializationFormat {
    Json,
    MessagePack,
    Bincode,
    Protobuf,
    Custom(String),
}
```

## Runtime Implementations

### Wasmtime Implementation

```rust
use wasmtime::*;

pub struct WasmtimeRuntime {
    engine: Engine,
    config: RuntimeConfig,
    store_builder: StoreBuilder,
}

impl WasmtimeRuntime {
    pub fn new() -> Result<Self, WasmtimeError> {
        let mut config = Config::new();
        config.wasm_backtrace_details(WasmBacktraceDetails::Enable);
        config.wasm_multi_memory(true);
        config.wasm_bulk_memory(true);
        config.wasm_reference_types(true);
        config.wasm_simd(true);
        config.consume_fuel(true);
        
        let engine = Engine::new(&config)?;
        
        Ok(Self {
            engine,
            config: RuntimeConfig::default(),
            store_builder: StoreBuilder::new(),
        })
    }
    
    pub fn with_config(runtime_config: RuntimeConfig) -> Result<Self, WasmtimeError> {
        let mut config = Config::new();
        
        // Apply runtime configuration
        config.consume_fuel(runtime_config.enable_fuel_metering);
        config.epoch_interruption(runtime_config.enable_epoch_interruption);
        config.max_wasm_stack(runtime_config.max_wasm_stack);
        
        // Security configurations
        if runtime_config.security.disable_parallel_compilation {
            config.parallel_compilation(false);
        }
        
        // Optimization configurations
        match runtime_config.optimization_level {
            OptimizationLevel::None => config.cranelift_opt_level(wasmtime::OptLevel::None),
            OptimizationLevel::Speed => config.cranelift_opt_level(wasmtime::OptLevel::Speed),
            OptimizationLevel::Size => config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize),
            OptimizationLevel::Aggressive => {
                config.cranelift_opt_level(wasmtime::OptLevel::Speed);
                config.cranelift_nan_canonicalization(true);
            }
        }
        
        let engine = Engine::new(&config)?;
        
        Ok(Self {
            engine,
            config: runtime_config,
            store_builder: StoreBuilder::new(),
        })
    }
}

#[async_trait]
impl WasmRuntime for WasmtimeRuntime {
    type Module = WasmtimeModule;
    type Instance = WasmtimeInstance;
    type Error = WasmtimeError;
    
    fn name(&self) -> &'static str {
        "wasmtime"
    }
    
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    
    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            wasm_features: WasmFeatures {
                mvp: true,
                bulk_memory: true,
                reference_types: true,
                simd: true,
                multi_value: true,
                tail_call: false, // Not yet supported in stable wasmtime
                threads: true,
                exception_handling: false, // Experimental
                memory64: false,
                component_model: true,
                gc: false,
            },
            memory_management: MemoryCapabilities {
                linear_memory: true,
                shared_memory: true,
                memory_protection: true,
                copy_on_write: false,
                memory_mapping: true,
                custom_allocators: true,
            },
            security_features: SecurityFeatures {
                capability_system: false, // Implemented at sandbox level
                resource_limits: true,
                execution_metering: true,
                stack_overflow_protection: true,
                control_flow_integrity: true,
                address_space_randomization: false,
                secure_compilation: true,
            },
            performance_features: PerformanceFeatures {
                jit_compilation: true,
                aot_compilation: false,
                optimization_levels: vec![
                    OptimizationLevel::None,
                    OptimizationLevel::Speed,
                    OptimizationLevel::Size,
                ],
                profiling: true,
                parallel_compilation: true,
                caching: true,
                streaming_compilation: false,
            },
            debugging_capabilities: DebuggingCapabilities {
                breakpoints: false,
                stepping: false,
                stack_inspection: true,
                variable_inspection: false,
                memory_inspection: true,
                dwarf_debugging: true,
                source_mapping: false,
            },
            host_integration: HostIntegrationFeatures {
                host_functions: true,
                memory_sharing: true,
                callback_support: true,
                async_support: true,
                streaming_io: false,
                serialization_formats: vec![
                    SerializationFormat::Json,
                    SerializationFormat::MessagePack,
                    SerializationFormat::Bincode,
                ],
            },
        }
    }
    
    async fn compile_module(&self, code: &[u8]) -> Result<Self::Module, Self::Error> {
        let module = Module::from_binary(&self.engine, code)
            .map_err(WasmtimeError::CompilationFailed)?;
        
        Ok(WasmtimeModule::new(module))
    }
    
    async fn compile_module_from_file(&self, path: &std::path::Path) -> Result<Self::Module, Self::Error> {
        let module = Module::from_file(&self.engine, path)
            .map_err(WasmtimeError::CompilationFailed)?;
        
        Ok(WasmtimeModule::new(module))
    }
    
    async fn validate_module(&self, code: &[u8]) -> Result<(), Self::Error> {
        Module::validate(&self.engine, code)
            .map_err(WasmtimeError::ValidationFailed)?;
        Ok(())
    }
    
    async fn instantiate(&self, module: &Self::Module) -> Result<Self::Instance, Self::Error> {
        let mut store = Store::new(&self.engine, ());
        
        // Apply fuel limits if configured
        if self.config.enable_fuel_metering {
            store.add_fuel(self.config.fuel_limit.unwrap_or(1_000_000))?;
        }
        
        let instance = Instance::new(&mut store, &module.inner, &[])
            .map_err(WasmtimeError::InstantiationFailed)?;
        
        Ok(WasmtimeInstance::new(instance, store))
    }
    
    async fn instantiate_with_config(
        &self,
        module: &Self::Module,
        config: &InstanceConfig,
    ) -> Result<Self::Instance, Self::Error> {
        let mut store = Store::new(&self.engine, ());
        
        // Apply instance-specific configuration
        if let Some(fuel_limit) = config.fuel_limit {
            store.add_fuel(fuel_limit)?;
        }
        
        if let Some(memory_limit) = config.memory_limit {
            // Note: Wasmtime doesn't have direct memory limits,
            // this would need to be implemented through a custom allocator
        }
        
        // Create linker for imports
        let mut linker = Linker::new(&self.engine);
        
        // Add host functions if specified
        for host_function in &config.host_functions {
            self.add_host_function(&mut linker, host_function)?;
        }
        
        let instance = linker.instantiate(&mut store, &module.inner)
            .map_err(WasmtimeError::InstantiationFailed)?;
        
        Ok(WasmtimeInstance::new(instance, store))
    }
    
    async fn set_resource_limits(&self, limits: &ResourceLimits) -> Result<(), Self::Error> {
        // Wasmtime resource limits are set at store creation time
        // This would require recreating the store
        Ok(())
    }
    
    async fn get_resource_usage(&self) -> Result<ResourceUsage, Self::Error> {
        // TODO: Implement resource usage tracking
        Ok(ResourceUsage {
            memory_used: 0,
            fuel_consumed: 0,
            cpu_time: std::time::Duration::ZERO,
            function_calls: 0,
        })
    }
    
    async fn configure(&mut self, config: &RuntimeConfig) -> Result<(), Self::Error> {
        self.config = config.clone();
        Ok(())
    }
    
    async fn get_configuration(&self) -> RuntimeConfig {
        self.config.clone()
    }
    
    async fn initialize(&mut self) -> Result<(), Self::Error> {
        // Wasmtime doesn't require explicit initialization
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        // Wasmtime doesn't require explicit shutdown
        Ok(())
    }
    
    async fn get_runtime_info(&self) -> RuntimeInfo {
        RuntimeInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            capabilities: self.capabilities(),
            uptime: std::time::Duration::ZERO, // TODO: track uptime
            memory_usage: 0, // TODO: track memory
            active_instances: 0, // TODO: track instances
        }
    }
    
    async fn collect_metrics(&self) -> RuntimeMetrics {
        RuntimeMetrics {
            compilation_time: std::time::Duration::ZERO,
            instantiation_time: std::time::Duration::ZERO,
            execution_time: std::time::Duration::ZERO,
            memory_peak: 0,
            fuel_consumed: 0,
            function_calls: 0,
            errors: 0,
        }
    }
}

impl WasmtimeRuntime {
    fn add_host_function(&self, linker: &mut Linker<()>, function: &HostFunction) -> Result<(), WasmtimeError> {
        // TODO: Implement host function registration
        Ok(())
    }
}

pub struct WasmtimeModule {
    inner: Module,
    metadata: ModuleMetadata,
}

impl WasmtimeModule {
    pub fn new(module: Module) -> Self {
        let metadata = ModuleMetadata {
            id: uuid::Uuid::new_v4().to_string(),
            hash: vec![], // TODO: compute hash
            size: 0, // TODO: get actual size
            creation_time: chrono::Utc::now(),
        };
        
        Self {
            inner: module,
            metadata,
        }
    }
}

#[async_trait]
impl WasmModule for WasmtimeModule {
    type Error = WasmtimeError;
    
    fn module_id(&self) -> &str {
        &self.metadata.id
    }
    
    fn module_hash(&self) -> &[u8] {
        &self.metadata.hash
    }
    
    fn size(&self) -> usize {
        self.metadata.size
    }
    
    fn creation_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.creation_time
    }
    
    fn get_exports(&self) -> Vec<Export> {
        self.inner.exports()
            .map(|export| Export {
                name: export.name().to_string(),
                export_type: match export.ty() {
                    wasmtime::ExternType::Func(_) => ExportType::Function,
                    wasmtime::ExternType::Global(_) => ExportType::Global,
                    wasmtime::ExternType::Table(_) => ExportType::Table,
                    wasmtime::ExternType::Memory(_) => ExportType::Memory,
                },
            })
            .collect()
    }
    
    fn get_imports(&self) -> Vec<Import> {
        self.inner.imports()
            .map(|import| Import {
                module: import.module().to_string(),
                name: import.name().to_string(),
                import_type: match import.ty() {
                    wasmtime::ExternType::Func(_) => ImportType::Function,
                    wasmtime::ExternType::Global(_) => ImportType::Global,
                    wasmtime::ExternType::Table(_) => ImportType::Table,
                    wasmtime::ExternType::Memory(_) => ImportType::Memory,
                },
            })
            .collect()
    }
    
    fn get_memory_info(&self) -> MemoryInfo {
        // TODO: Extract memory information from module
        MemoryInfo {
            initial_pages: 0,
            maximum_pages: None,
            shared: false,
        }
    }
    
    fn get_table_info(&self) -> Vec<TableInfo> {
        // TODO: Extract table information from module
        vec![]
    }
    
    async fn validate(&self) -> Result<ValidationResult, Self::Error> {
        // Wasmtime validates during compilation
        Ok(ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
        })
    }
    
    async fn analyze_security(&self) -> SecurityAnalysisResult {
        SecurityAnalysisResult {
            risk_level: SecurityRiskLevel::Low,
            vulnerabilities: vec![],
            recommendations: vec![],
        }
    }
    
    async fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        self.inner.serialize()
            .map_err(WasmtimeError::SerializationFailed)
    }
    
    async fn deserialize(data: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        // Note: Deserialization requires an Engine, which we don't have here
        // This would need to be handled differently in practice
        Err(WasmtimeError::DeserializationFailed(
            "Deserialization requires an Engine".into()
        ))
    }
}

pub struct WasmtimeInstance {
    instance: Instance,
    store: Store<()>,
    metadata: InstanceMetadata,
}

impl WasmtimeInstance {
    pub fn new(instance: Instance, store: Store<()>) -> Self {
        let metadata = InstanceMetadata {
            id: uuid::Uuid::new_v4().to_string(),
            module_id: "unknown".to_string(), // TODO: link to module
            creation_time: chrono::Utc::now(),
            state: InstanceState::Running,
        };
        
        Self {
            instance,
            store,
            metadata,
        }
    }
}

#[async_trait]
impl WasmInstance for WasmtimeInstance {
    type Error = WasmtimeError;
    
    fn instance_id(&self) -> &str {
        &self.metadata.id
    }
    
    fn module_id(&self) -> &str {
        &self.metadata.module_id
    }
    
    fn creation_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.creation_time
    }
    
    fn state(&self) -> InstanceState {
        self.metadata.state
    }
    
    async fn call_function<T, R>(&mut self, name: &str, args: T) -> Result<R, Self::Error>
    where
        T: Serialize + Send,
        R: for<'de> Deserialize<'de> + Send,
    {
        // Serialize arguments
        let serialized_args = serde_json::to_vec(&args)
            .map_err(|e| WasmtimeError::SerializationFailed(e.into()))?;
        
        // Get the function
        let func = self.instance.get_func(&mut self.store, name)
            .ok_or_else(|| WasmtimeError::FunctionNotFound(name.to_string()))?;
        
        // TODO: Implement proper argument marshaling and function calling
        // This would involve converting serialized args to WASM values,
        // calling the function, and deserializing the result
        
        todo!("Implement function calling with serialization")
    }
    
    async fn call_function_raw(&mut self, name: &str, args: &[WasmValue]) -> Result<Vec<WasmValue>, Self::Error> {
        let func = self.instance.get_func(&mut self.store, name)
            .ok_or_else(|| WasmtimeError::FunctionNotFound(name.to_string()))?;
        
        // Convert WasmValue to wasmtime::Val
        let wasmtime_args: Result<Vec<wasmtime::Val>, _> = args.iter()
            .map(|val| self.convert_wasm_value_to_wasmtime(val))
            .collect();
        
        let wasmtime_args = wasmtime_args?;
        
        // Prepare result buffer
        let mut results = vec![wasmtime::Val::I32(0); func.ty(&self.store).results().len()];
        
        // Call function
        func.call(&mut self.store, &wasmtime_args, &mut results)
            .map_err(WasmtimeError::ExecutionFailed)?;
        
        // Convert results back
        let wasm_results: Result<Vec<WasmValue>, _> = results.iter()
            .map(|val| self.convert_wasmtime_value_to_wasm(val))
            .collect();
        
        wasm_results
    }
    
    async fn read_memory(&self, offset: u32, size: u32) -> Result<Vec<u8>, Self::Error> {
        let memory = self.instance.get_memory(&self.store, "memory")
            .ok_or_else(|| WasmtimeError::MemoryNotFound)?;
        
        let data = memory.data(&self.store);
        let start = offset as usize;
        let end = start + size as usize;
        
        if end > data.len() {
            return Err(WasmtimeError::MemoryOutOfBounds);
        }
        
        Ok(data[start..end].to_vec())
    }
    
    async fn write_memory(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
        let memory = self.instance.get_memory(&mut self.store, "memory")
            .ok_or_else(|| WasmtimeError::MemoryNotFound)?;
        
        let memory_data = memory.data_mut(&mut self.store);
        let start = offset as usize;
        let end = start + data.len();
        
        if end > memory_data.len() {
            return Err(WasmtimeError::MemoryOutOfBounds);
        }
        
        memory_data[start..end].copy_from_slice(data);
        Ok(())
    }
    
    async fn get_memory_size(&self) -> Result<u32, Self::Error> {
        let memory = self.instance.get_memory(&self.store, "memory")
            .ok_or_else(|| WasmtimeError::MemoryNotFound)?;
        
        Ok(memory.size(&self.store) as u32)
    }
    
    async fn grow_memory(&mut self, pages: u32) -> Result<u32, Self::Error> {
        let memory = self.instance.get_memory(&mut self.store, "memory")
            .ok_or_else(|| WasmtimeError::MemoryNotFound)?;
        
        let old_size = memory.grow(&mut self.store, pages as u64)
            .map_err(WasmtimeError::MemoryGrowFailed)?;
        
        Ok(old_size as u32)
    }
    
    // TODO: Implement remaining methods
    async fn get_global(&self, name: &str) -> Result<WasmValue, Self::Error> {
        todo!("Implement get_global")
    }
    
    async fn set_global(&mut self, name: &str, value: WasmValue) -> Result<(), Self::Error> {
        todo!("Implement set_global")
    }
    
    async fn get_table_element(&self, table: &str, index: u32) -> Result<WasmValue, Self::Error> {
        todo!("Implement get_table_element")
    }
    
    async fn set_table_element(&mut self, table: &str, index: u32, value: WasmValue) -> Result<(), Self::Error> {
        todo!("Implement set_table_element")
    }
    
    async fn get_resource_usage(&self) -> Result<ResourceUsage, Self::Error> {
        Ok(ResourceUsage {
            memory_used: self.get_memory_size().await? as u64 * 65536, // Pages to bytes
            fuel_consumed: self.store.fuel_consumed().unwrap_or(0),
            cpu_time: std::time::Duration::ZERO, // TODO: track CPU time
            function_calls: 0, // TODO: track function calls
        })
    }
    
    async fn get_execution_stats(&self) -> ExecutionStats {
        ExecutionStats {
            instructions_executed: 0,
            function_calls: 0,
            memory_accesses: 0,
            execution_time: std::time::Duration::ZERO,
        }
    }
    
    async fn pause(&mut self) -> Result<(), Self::Error> {
        // Wasmtime doesn't have built-in pause/resume
        self.metadata.state = InstanceState::Paused;
        Ok(())
    }
    
    async fn resume(&mut self) -> Result<(), Self::Error> {
        self.metadata.state = InstanceState::Running;
        Ok(())
    }
    
    async fn reset(&mut self) -> Result<(), Self::Error> {
        // Would need to recreate the instance
        todo!("Implement reset")
    }
    
    async fn terminate(&mut self) -> Result<(), Self::Error> {
        self.metadata.state = InstanceState::Terminated;
        Ok(())
    }
    
    async fn get_stack_trace(&self) -> Result<Vec<StackFrame>, Self::Error> {
        // TODO: Implement stack trace extraction
        Ok(vec![])
    }
    
    async fn set_breakpoint(&mut self, location: BreakpointLocation) -> Result<BreakpointId, Self::Error> {
        // Wasmtime doesn't have built-in debugging support
        todo!("Implement breakpoint support")
    }
    
    async fn remove_breakpoint(&mut self, id: BreakpointId) -> Result<(), Self::Error> {
        todo!("Implement breakpoint removal")
    }
}

impl WasmtimeInstance {
    fn convert_wasm_value_to_wasmtime(&self, value: &WasmValue) -> Result<wasmtime::Val, WasmtimeError> {
        match value {
            WasmValue::I32(val) => Ok(wasmtime::Val::I32(*val)),
            WasmValue::I64(val) => Ok(wasmtime::Val::I64(*val)),
            WasmValue::F32(val) => Ok(wasmtime::Val::F32(*val)),
            WasmValue::F64(val) => Ok(wasmtime::Val::F64(*val)),
            WasmValue::FuncRef(_) => todo!("Implement funcref conversion"),
            WasmValue::ExternRef(_) => todo!("Implement externref conversion"),
        }
    }
    
    fn convert_wasmtime_value_to_wasm(&self, value: &wasmtime::Val) -> Result<WasmValue, WasmtimeError> {
        match value {
            wasmtime::Val::I32(val) => Ok(WasmValue::I32(*val)),
            wasmtime::Val::I64(val) => Ok(WasmValue::I64(*val)),
            wasmtime::Val::F32(val) => Ok(WasmValue::F32(*val)),
            wasmtime::Val::F64(val) => Ok(WasmValue::F64(*val)),
            wasmtime::Val::FuncRef(_) => todo!("Implement funcref conversion"),
            wasmtime::Val::ExternRef(_) => todo!("Implement externref conversion"),
            _ => Err(WasmtimeError::UnsupportedValueType),
        }
    }
}
```

### Wasmer Implementation

```rust
use wasmer::*;

pub struct WasmerRuntime {
    store: Store,
    config: RuntimeConfig,
}

impl WasmerRuntime {
    pub fn new() -> Result<Self, WasmerError> {
        let engine = wasmer::Universal::new(wasmer::Cranelift::default()).into();
        let store = Store::new(&engine);
        
        Ok(Self {
            store,
            config: RuntimeConfig::default(),
        })
    }
    
    pub fn with_config(config: RuntimeConfig) -> Result<Self, WasmerError> {
        let mut compiler_config = wasmer::Cranelift::default();
        
        // Apply optimization level
        match config.optimization_level {
            OptimizationLevel::None => compiler_config.opt_level(wasmer::CraneliftOptLevel::None),
            OptimizationLevel::Speed => compiler_config.opt_level(wasmer::CraneliftOptLevel::Speed),
            OptimizationLevel::Size => compiler_config.opt_level(wasmer::CraneliftOptLevel::SpeedAndSize),
            OptimizationLevel::Aggressive => {
                compiler_config.opt_level(wasmer::CraneliftOptLevel::Speed);
                compiler_config.nan_canonicalization(true);
            }
        }
        
        let engine = wasmer::Universal::new(compiler_config).into();
        let store = Store::new(&engine);
        
        Ok(Self {
            store,
            config,
        })
    }
}

#[async_trait]
impl WasmRuntime for WasmerRuntime {
    type Module = WasmerModule;
    type Instance = WasmerInstance;
    type Error = WasmerError;
    
    fn name(&self) -> &'static str {
        "wasmer"
    }
    
    fn version(&self) -> &'static str {
        wasmer::VERSION
    }
    
    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            wasm_features: WasmFeatures {
                mvp: true,
                bulk_memory: true,
                reference_types: true,
                simd: true,
                multi_value: true,
                tail_call: false,
                threads: false, // Not yet stable in Wasmer
                exception_handling: false,
                memory64: false,
                component_model: false,
                gc: false,
            },
            memory_management: MemoryCapabilities {
                linear_memory: true,
                shared_memory: false,
                memory_protection: true,
                copy_on_write: false,
                memory_mapping: false,
                custom_allocators: false,
            },
            security_features: SecurityFeatures {
                capability_system: false,
                resource_limits: false, // Limited support
                execution_metering: false,
                stack_overflow_protection: true,
                control_flow_integrity: true,
                address_space_randomization: false,
                secure_compilation: true,
            },
            performance_features: PerformanceFeatures {
                jit_compilation: true,
                aot_compilation: true,
                optimization_levels: vec![
                    OptimizationLevel::None,
                    OptimizationLevel::Speed,
                    OptimizationLevel::Size,
                ],
                profiling: false,
                parallel_compilation: false,
                caching: true,
                streaming_compilation: false,
            },
            debugging_capabilities: DebuggingCapabilities {
                breakpoints: false,
                stepping: false,
                stack_inspection: false,
                variable_inspection: false,
                memory_inspection: true,
                dwarf_debugging: false,
                source_mapping: false,
            },
            host_integration: HostIntegrationFeatures {
                host_functions: true,
                memory_sharing: true,
                callback_support: true,
                async_support: false,
                streaming_io: false,
                serialization_formats: vec![
                    SerializationFormat::Json,
                    SerializationFormat::MessagePack,
                ],
            },
        }
    }
    
    async fn compile_module(&self, code: &[u8]) -> Result<Self::Module, Self::Error> {
        let module = wasmer::Module::new(&self.store, code)
            .map_err(WasmerError::CompilationFailed)?;
        
        Ok(WasmerModule::new(module))
    }
    
    // ... implement remaining methods similar to Wasmtime
}

// Similar implementations for WasmerModule and WasmerInstance
```

## Runtime Selection and Factory

```rust
use std::collections::HashMap;

pub struct RuntimeFactory {
    available_runtimes: HashMap<String, Box<dyn RuntimeBuilder>>,
    default_runtime: String,
}

impl RuntimeFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            available_runtimes: HashMap::new(),
            default_runtime: "wasmtime".to_string(),
        };
        
        // Register available runtimes
        factory.register_runtime("wasmtime", Box::new(WasmtimeBuilder::new()));
        factory.register_runtime("wasmer", Box::new(WasmerBuilder::new()));
        
        factory
    }
    
    pub fn register_runtime(&mut self, name: &str, builder: Box<dyn RuntimeBuilder>) {
        self.available_runtimes.insert(name.to_string(), builder);
    }
    
    pub fn create_runtime(&self, name: Option<&str>) -> Result<Box<dyn WasmRuntime<Module = Box<dyn WasmModule<Error = RuntimeError>>, Instance = Box<dyn WasmInstance<Error = RuntimeError>>, Error = RuntimeError>>, RuntimeError> {
        let runtime_name = name.unwrap_or(&self.default_runtime);
        
        let builder = self.available_runtimes.get(runtime_name)
            .ok_or_else(|| RuntimeError::RuntimeNotFound(runtime_name.to_string()))?;
        
        builder.build()
    }
    
    pub fn create_runtime_with_config(
        &self,
        name: Option<&str>,
        config: &RuntimeConfig,
    ) -> Result<Box<dyn WasmRuntime<Module = Box<dyn WasmModule<Error = RuntimeError>>, Instance = Box<dyn WasmInstance<Error = RuntimeError>>, Error = RuntimeError>>, RuntimeError> {
        let runtime_name = name.unwrap_or(&self.default_runtime);
        
        let builder = self.available_runtimes.get(runtime_name)
            .ok_or_else(|| RuntimeError::RuntimeNotFound(runtime_name.to_string()))?;
        
        builder.build_with_config(config)
    }
    
    pub fn list_available_runtimes(&self) -> Vec<String> {
        self.available_runtimes.keys().cloned().collect()
    }
    
    pub fn get_runtime_capabilities(&self, name: &str) -> Option<RuntimeCapabilities> {
        self.available_runtimes.get(name)
            .map(|builder| builder.get_capabilities())
    }
    
    pub fn select_best_runtime(&self, requirements: &RuntimeRequirements) -> Option<String> {
        let mut best_runtime = None;
        let mut best_score = 0.0;
        
        for (name, builder) in &self.available_runtimes {
            let capabilities = builder.get_capabilities();
            let score = self.calculate_compatibility_score(&capabilities, requirements);
            
            if score > best_score {
                best_score = score;
                best_runtime = Some(name.clone());
            }
        }
        
        best_runtime
    }
    
    fn calculate_compatibility_score(
        &self,
        capabilities: &RuntimeCapabilities,
        requirements: &RuntimeRequirements,
    ) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;
        
        // Score WASM features
        if let Some(required_features) = &requirements.wasm_features {
            let feature_score = self.score_wasm_features(&capabilities.wasm_features, required_features);
            score += feature_score * 0.3;
            total_weight += 0.3;
        }
        
        // Score security features
        if let Some(required_security) = &requirements.security_features {
            let security_score = self.score_security_features(&capabilities.security_features, required_security);
            score += security_score * 0.25;
            total_weight += 0.25;
        }
        
        // Score performance features
        if let Some(required_performance) = &requirements.performance_features {
            let performance_score = self.score_performance_features(&capabilities.performance_features, required_performance);
            score += performance_score * 0.25;
            total_weight += 0.25;
        }
        
        // Score debugging capabilities
        if let Some(required_debugging) = &requirements.debugging_capabilities {
            let debugging_score = self.score_debugging_capabilities(&capabilities.debugging_capabilities, required_debugging);
            score += debugging_score * 0.2;
            total_weight += 0.2;
        }
        
        if total_weight > 0.0 {
            score / total_weight
        } else {
            0.0
        }
    }
    
    fn score_wasm_features(&self, capabilities: &WasmFeatures, requirements: &WasmFeatures) -> f64 {
        let mut satisfied = 0;
        let mut total = 0;
        
        macro_rules! check_feature {
            ($field:ident) => {
                if requirements.$field {
                    total += 1;
                    if capabilities.$field {
                        satisfied += 1;
                    }
                }
            };
        }
        
        check_feature!(mvp);
        check_feature!(bulk_memory);
        check_feature!(reference_types);
        check_feature!(simd);
        check_feature!(multi_value);
        check_feature!(tail_call);
        check_feature!(threads);
        check_feature!(exception_handling);
        check_feature!(memory64);
        check_feature!(component_model);
        check_feature!(gc);
        
        if total > 0 {
            satisfied as f64 / total as f64
        } else {
            1.0
        }
    }
    
    // Similar scoring methods for other feature categories...
}

pub trait RuntimeBuilder: Send + Sync {
    fn build(&self) -> Result<Box<dyn WasmRuntime<Module = Box<dyn WasmModule<Error = RuntimeError>>, Instance = Box<dyn WasmInstance<Error = RuntimeError>>, Error = RuntimeError>>, RuntimeError>;
    fn build_with_config(&self, config: &RuntimeConfig) -> Result<Box<dyn WasmRuntime<Module = Box<dyn WasmModule<Error = RuntimeError>>, Instance = Box<dyn WasmInstance<Error = RuntimeError>>, Error = RuntimeError>>, RuntimeError>;
    fn get_capabilities(&self) -> RuntimeCapabilities;
}

pub struct WasmtimeBuilder;

impl WasmtimeBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeBuilder for WasmtimeBuilder {
    fn build(&self) -> Result<Box<dyn WasmRuntime<Module = Box<dyn WasmModule<Error = RuntimeError>>, Instance = Box<dyn WasmInstance<Error = RuntimeError>>, Error = RuntimeError>>, RuntimeError> {
        let runtime = WasmtimeRuntime::new()
            .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
        Ok(Box::new(runtime))
    }
    
    fn build_with_config(&self, config: &RuntimeConfig) -> Result<Box<dyn WasmRuntime<Module = Box<dyn WasmModule<Error = RuntimeError>>, Instance = Box<dyn WasmInstance<Error = RuntimeError>>, Error = RuntimeError>>, RuntimeError> {
        let runtime = WasmtimeRuntime::with_config(config.clone())
            .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
        Ok(Box::new(runtime))
    }
    
    fn get_capabilities(&self) -> RuntimeCapabilities {
        // Return static capabilities for Wasmtime
        RuntimeCapabilities {
            // ... (same as in WasmtimeRuntime::capabilities())
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeRequirements {
    pub wasm_features: Option<WasmFeatures>,
    pub security_features: Option<SecurityFeatures>,
    pub performance_features: Option<PerformanceFeatures>,
    pub debugging_capabilities: Option<DebuggingCapabilities>,
    pub host_integration: Option<HostIntegrationFeatures>,
}
```

## Usage Examples

### Basic Runtime Usage

```rust
use wasm_sandbox::runtime::{RuntimeFactory, RuntimeConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let factory = RuntimeFactory::new();
    
    // Create default runtime
    let mut runtime = factory.create_runtime(None)?;
    
    // Compile module
    let wasm_code = std::fs::read("program.wasm")?;
    let module = runtime.compile_module(&wasm_code).await?;
    
    // Create instance
    let mut instance = runtime.instantiate(&module).await?;
    
    // Call function
    let result: i32 = instance.call_function("add", (5, 3)).await?;
    println!("Result: {}", result);
    
    Ok(())
}
```

### Runtime Selection

```rust
use wasm_sandbox::runtime::{RuntimeFactory, RuntimeRequirements, WasmFeatures};

async fn select_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let factory = RuntimeFactory::new();
    
    // Define requirements
    let requirements = RuntimeRequirements {
        wasm_features: Some(WasmFeatures {
            mvp: true,
            simd: true,
            threads: true,
            ..Default::default()
        }),
        security_features: Some(SecurityFeatures {
            execution_metering: true,
            resource_limits: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    
    // Select best runtime
    let runtime_name = factory.select_best_runtime(&requirements)
        .ok_or("No compatible runtime found")?;
    
    println!("Selected runtime: {}", runtime_name);
    
    // Create runtime with custom config
    let config = RuntimeConfig {
        optimization_level: OptimizationLevel::Speed,
        enable_fuel_metering: true,
        fuel_limit: Some(1_000_000),
        ..Default::default()
    };
    
    let runtime = factory.create_runtime_with_config(Some(&runtime_name), &config)?;
    
    Ok(())
}
```

## Next Steps

This runtime abstraction design provides:

1. **Unified Interface** - Single API for multiple WebAssembly runtimes
2. **Feature Detection** - Runtime capability detection and selection
3. **Performance** - Minimal abstraction overhead
4. **Extensibility** - Easy addition of new runtime backends
5. **Configuration** - Runtime-specific optimizations

Continue with:

- **[Performance Guide](performance.md)** - Optimize runtime performance
- **[Security Model](security-model.md)** - Understand security implications
- **[Monitoring Guide](monitoring.md)** - Monitor runtime metrics

---

**Runtime Excellence:** This abstraction layer enables wasm-sandbox to leverage the best features of multiple WebAssembly runtimes while maintaining a consistent, high-level API. Choose runtimes based on your specific requirements for security, performance, and feature support.
