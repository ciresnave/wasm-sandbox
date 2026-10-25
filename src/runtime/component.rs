//! WebAssembly Component Model implementation
//!
//! This module provides support for the WebAssembly Component Model,
//! which enables more complex and type-safe interactions between host and guest.
//!
//! The Component Model allows for:
//! - Interface-driven development
//! - Multi-language component authoring
//! - Structured data exchange
//! - Explicit imports and exports

use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use wasmtime::component::{Component, Instance, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::p2::bindings::sync::Command;
use wasmtime_wasi::p2::pipe::MemoryOutputPipe;
use wasmtime_wasi::sockets::SocketAddrUse;
use wasmtime_wasi::{I32Exit, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::error::{Error, Result};
use crate::runtime::network_policy::{NetUse, NetworkPolicy};
use crate::runtime::wasmtime::{preopen_plan, validate_filesystem};
use crate::runtime::{
    ModuleId, RuntimeConfig, RuntimeMetrics, WasmFunctionCaller, WasmInstance, WasmInstanceState,
    WasmModule, WasmRuntime,
};
use crate::security::{Capabilities, EnvironmentCapability, ProcessCapability, ResourceLimits};

/// A WebAssembly Component Module
pub struct ComponentModule {
    /// Unique identifier for this module
    id: ModuleId,

    /// The compiled component
    component: Component,

    /// Component name if available
    name: Option<String>,

    /// Component metadata
    metadata: serde_json::Value,
}

impl ComponentModule {
    /// Create a new Component Module from WebAssembly bytes
    pub fn new(engine: &Engine, bytes: &[u8]) -> Result<Self> {
        let component = Component::new(engine, bytes).map_err(|e| Error::ModuleLoad {
            message: e.to_string(),
        })?;

        Ok(Self {
            id: ModuleId::new(),
            component,
            name: None,
            metadata: serde_json::Value::Null,
        })
    }

    /// Get the component
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Set component name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set component metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

impl WasmModule for ComponentModule {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn size(&self) -> usize {
        // Components don't expose raw size easily, return 0 for now
        0
    }

    fn exports(&self) -> Vec<String> {
        // Component exports would need to be introspected
        // For now return empty vector
        Vec::new()
    }

    fn clone_module(&self) -> Box<dyn WasmModule> {
        Box::new(ComponentModule {
            id: self.id,
            component: self.component.clone(),
            name: self.name.clone(),
            metadata: self.metadata.clone(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A WebAssembly Component Instance
pub struct ComponentInstance<T: 'static> {
    /// Unique identifier for this instance
    #[allow(dead_code)]
    id: Uuid,

    /// The module this instance was created from
    #[allow(dead_code)]
    module_id: Uuid,

    /// The wasmtime component instance
    instance: Instance,

    /// Store containing the instance
    store: Store<T>,

    /// Instance capabilities
    #[allow(dead_code)]
    capabilities: Capabilities,

    /// Resource limits
    #[allow(dead_code)]
    resource_limits: ResourceLimits,
}

impl<T: Send + 'static> ComponentInstance<T> {
    /// Create a new Component Instance
    pub fn new(
        module: &ComponentModule,
        mut store: Store<T>,
        linker: &Linker<T>,
        capabilities: Capabilities,
        resource_limits: ResourceLimits,
    ) -> Result<Self> {
        let instance = linker
            .instantiate(&mut store, &module.component)
            .map_err(|e| Error::InstanceCreation {
                reason: e.to_string(),
                instance_id: None,
            })?;

        Ok(Self {
            id: Uuid::new_v4(),
            module_id: module.id.as_uuid(),
            instance,
            store,
            capabilities,
            resource_limits,
        })
    }

    /// Get a reference to the store
    pub fn store(&self) -> &Store<T> {
        &self.store
    }

    /// Get a mutable reference to the store
    pub fn store_mut(&mut self) -> &mut Store<T> {
        &mut self.store
    }

    /// Get a reference to the instance
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
}

impl<T: Send + Sync + 'static> WasmInstance for ComponentInstance<T> {
    fn state(&self) -> WasmInstanceState {
        WasmInstanceState::Running // Components are always running if created
    }

    fn memory_usage(&self) -> usize {
        // Component memory usage would need specific wasmtime API calls
        0
    }

    fn fuel_usage(&self) -> Option<u64> {
        // Fuel tracking would need to be implemented in store config
        None
    }

    fn reset_fuel(&self) -> Result<()> {
        // Would need store.set_fuel() call
        Ok(())
    }

    fn add_fuel(&self, _fuel: u64) -> Result<()> {
        // Would need store.add_fuel() call
        Ok(())
    }

    unsafe fn memory_ptr(&self) -> Result<*mut u8> {
        // Direct memory access for components would be complex
        Err(Error::UnsupportedOperation {
            message: "Direct memory access not supported for components".to_string(),
        })
    }

    fn memory_size(&self) -> usize {
        // Component memory size would need specific API
        0
    }

    fn function_caller(&self) -> Box<dyn WasmFunctionCaller> {
        Box::new(ComponentFunctionCaller {})
    }

    fn call_simple_function(&self, _function_name: &str, _params: &[i32]) -> Result<i32> {
        // Component function calling is different from core modules
        Err(Error::UnsupportedOperation {
            message: "Simple function calls not supported for components".to_string(),
        })
    }
}

/// Function caller implementation for WebAssembly Components
pub struct ComponentFunctionCaller {}

impl WasmFunctionCaller for ComponentFunctionCaller {
    fn call_function_json(&self, _function_name: &str, _params_json: &str) -> Result<String> {
        Err(Error::UnsupportedOperation {
            message: "Component function calling not yet implemented".to_string(),
        })
    }

    fn call_function_msgpack(
        &self,
        _function_name: &str,
        _params_msgpack: &[u8],
    ) -> Result<Vec<u8>> {
        Err(Error::UnsupportedOperation {
            message: "Component function calling not yet implemented".to_string(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Component Model runtime manager
pub struct ComponentRuntime {
    /// Wasmtime engine
    engine: Engine,

    /// Default capabilities for new instances
    #[allow(dead_code)]
    default_capabilities: Capabilities,

    /// Default resource limits for new instances
    #[allow(dead_code)]
    default_resource_limits: ResourceLimits,
}

impl ComponentRuntime {
    /// Create a new Component Model runtime
    pub fn new(
        default_capabilities: Capabilities,
        default_resource_limits: ResourceLimits,
    ) -> Result<Self> {
        let mut config = wasmtime::Config::new();

        // Configure for component model support
        config.wasm_component_model(true);

        // Note: memory limits are not a wasm *stack* limit. This used to call `max_wasm_stack` with
        // the memory size, which wasmtime 50 rejects whenever it exceeds `async_stack_size`, so a
        // runtime could not be created with the default limits.

        let engine = Engine::new(&config).map_err(|e| Error::RuntimeInitialization {
            message: e.to_string(),
        })?;

        Ok(Self {
            engine,
            default_capabilities,
            default_resource_limits,
        })
    }

    /// Create a new component module from bytes
    pub fn create_component_module(&self, bytes: &[u8]) -> Result<ComponentModule> {
        ComponentModule::new(&self.engine, bytes)
    }

    /// Create a new component module from a file
    pub fn create_component_module_from_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<ComponentModule> {
        let bytes = std::fs::read(path).map_err(|e| Error::IoError {
            message: e.to_string(),
        })?;
        self.create_component_module(&bytes)
    }

    /// Create a WASI command instance with its capabilities applied, keeping the concrete type so
    /// the command can be run with [`WasiComponentInstance::run_command`] to capture its output.
    ///
    /// Grants the component runtime cannot honour (a process capability, a writable directory
    /// without create and delete, a file-size cap, a `secure` or unresolvable host) are
    /// [`Error::Capability`] here rather than being ignored.
    pub fn create_command_instance(
        &self,
        module: &dyn WasmModule,
        capabilities: Capabilities,
    ) -> Result<WasiComponentInstance> {
        let component = module
            .as_any()
            .downcast_ref::<ComponentModule>()
            .ok_or_else(|| Error::InstanceCreation {
                reason: "Module is not a ComponentModule".to_string(),
                instance_id: None,
            })?;

        if capabilities.process != ProcessCapability::None {
            return Err(Error::Capability {
                message: "a process capability is not supported for components: the guest cannot                           start processes; use ProcessCapability::None"
                    .to_string(),
            });
        }
        validate_filesystem(&capabilities.filesystem)?;
        let network = NetworkPolicy::from_capability(&capabilities.network)?;

        Ok(WasiComponentInstance {
            engine: self.engine.clone(),
            component: component.component.clone(),
            module_id: component.id,
            capabilities,
            network: Arc::new(network),
        })
    }

    /// Create a default store for components
    pub fn create_store<T>(&self, data: T) -> Store<T> {
        Store::new(&self.engine, data)
    }

    /// Create a default linker for components
    pub fn create_linker<T>(&self) -> Linker<T> {
        Linker::new(&self.engine)
    }
}

impl WasmRuntime for ComponentRuntime {
    fn initialize(&mut self, _config: RuntimeConfig) -> Result<()> {
        // Component runtime initialization would configure the engine
        Ok(())
    }

    fn load_module(&self, bytes: &[u8]) -> Result<Box<dyn WasmModule>> {
        let module = self.create_component_module(bytes)?;
        Ok(Box::new(module))
    }

    fn get_module(&self, _id: ModuleId) -> Result<Arc<dyn WasmModule>> {
        // Would need to store modules in a map to retrieve by ID
        Err(Error::UnsupportedOperation {
            message: "Module retrieval not implemented for components yet".to_string(),
        })
    }

    fn get_module_ids(&self) -> Vec<ModuleId> {
        // Would return IDs from stored modules
        Vec::new()
    }

    fn create_instance(
        &self,
        module: &dyn WasmModule,
        _resources: ResourceLimits,
        capabilities: Capabilities,
    ) -> Result<Box<dyn WasmInstance>> {
        Ok(Box::new(
            self.create_command_instance(module, capabilities)?,
        ))
    }

    fn get_metrics(&self) -> RuntimeMetrics {
        RuntimeMetrics {
            compiled_modules: 0,
            active_instances: 0,
            total_memory_usage: 0,
            peak_memory_usage: 0,
            fuel_consumption_rate: None,
            cache_hit_rate: None,
            last_compilation_time_ms: None,
        }
    }

    fn shutdown(&self) -> Result<()> {
        // Component runtime shutdown
        Ok(())
    }
}

/// Capture limit for a command's stdout and stderr, in bytes.
const OUTPUT_LIMIT: usize = 64 * 1024;

/// WASI state for a component command: the context built from the instance's capabilities.
struct WasiState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for WasiState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

fn net_use(usage: SocketAddrUse) -> NetUse {
    match usage {
        SocketAddrUse::TcpBind | SocketAddrUse::UdpBind => NetUse::Bind,
        SocketAddrUse::TcpListen => NetUse::Listen,
        SocketAddrUse::TcpAccept
        | SocketAddrUse::TcpConnect
        | SocketAddrUse::UdpSend
        | SocketAddrUse::UdpReceive => NetUse::Remote,
    }
}

/// Environment variables the capability lets through, read from the host process.
fn env_pairs(environment: &EnvironmentCapability) -> Vec<(String, String)> {
    match environment {
        EnvironmentCapability::None => Vec::new(),
        EnvironmentCapability::Allowlist(vars) => {
            std::env::vars().filter(|(k, _)| vars.contains(k)).collect()
        }
        EnvironmentCapability::Denylist(vars) => std::env::vars()
            .filter(|(k, _)| !vars.contains(k))
            .collect(),
        EnvironmentCapability::Full => std::env::vars().collect(),
    }
}

/// A WASI preview-2 *command* component with the instance's capabilities applied.
///
/// Create it with [`ComponentRuntime::create_instance`]. Each call of `call_simple_function("run",
/// &[])` runs the component's `wasi:cli/run` in a fresh WASI context (so a command can be run
/// repeatedly) and returns its exit code: `0` on success or `1` on failure. Directory grants, the environment allow-list and the network policy
/// apply to every run; the default is no sockets, no name lookups and no directories.
pub struct WasiComponentInstance {
    engine: Engine,
    component: Component,
    module_id: ModuleId,
    capabilities: Capabilities,
    network: Arc<NetworkPolicy>,
}

/// What a command component did when run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// `0` if `run` succeeded, `1` if it returned an error or called `exit` with a failure
    /// (WASI preview 2 carries no other exit code), or the code of an `I32Exit` trap.
    pub exit_code: i32,
    /// Everything the guest wrote to stdout (at most 64 KiB; the guest gets an error beyond that).
    pub stdout: Vec<u8>,
    /// Everything the guest wrote to stderr (same limit).
    pub stderr: Vec<u8>,
}

impl WasiComponentInstance {
    /// Run the component's `wasi:cli/run` once, in a fresh WASI context, and return its exit code
    /// and captured output.
    pub fn run_command(&self) -> Result<CommandOutput> {
        // WASI preview 2's sync adapter blocks on a runtime of its own, which panics on a Tokio
        // worker thread; run on a plain thread when called from one.
        crate::runtime::wasmtime::off_runtime(|| self.run_command_blocking())
    }

    fn run_command_blocking(&self) -> Result<CommandOutput> {
        let cap_err = |what: &str, e: &dyn std::fmt::Display| Error::Capability {
            message: format!("{what}: {e}"),
        };

        let stdout = MemoryOutputPipe::new(OUTPUT_LIMIT);
        let stderr = MemoryOutputPipe::new(OUTPUT_LIMIT);
        let mut builder = WasiCtxBuilder::new();
        builder.stdout(stdout.clone()).stderr(stderr.clone());
        for (k, v) in env_pairs(&self.capabilities.environment) {
            builder.env(&k, &v);
        }
        for dir in preopen_plan(&self.capabilities.filesystem) {
            builder
                .preopened_dir(&dir.host, &dir.guest, dir.perms)
                .map_err(|e| {
                    cap_err(
                        &format!(
                            "cannot preopen directory '{}' for the guest",
                            dir.host.display()
                        ),
                        &e,
                    )
                })?;
        }
        // Default deny: sockets and name lookups stay off unless the capability asks for them.
        builder
            .allow_tcp(self.network.sockets_enabled())
            .allow_udp(self.network.sockets_enabled())
            .allow_ip_name_lookup(self.network.name_lookup_enabled());
        let policy = Arc::clone(&self.network);
        builder.socket_addr_check(move |addr, usage| {
            let allowed = policy.permits(addr, net_use(usage));
            Box::pin(async move { allowed })
        });

        let mut store = Store::new(
            &self.engine,
            WasiState {
                ctx: builder.build(),
                table: ResourceTable::new(),
            },
        );
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_err(|e| {
            Error::InstanceCreation {
                reason: format!("Failed to add WASI to linker: {e}"),
                instance_id: Some(self.module_id.as_uuid()),
            }
        })?;
        let command = Command::instantiate(&mut store, &self.component, &linker).map_err(|e| {
            Error::InstanceCreation {
                reason: e.to_string(),
                instance_id: Some(self.module_id.as_uuid()),
            }
        })?;

        let exit_code = match command.wasi_cli_run().call_run(&mut store) {
            Ok(Ok(())) => 0,
            Ok(Err(())) => 1,
            Err(e) => match e.downcast_ref::<I32Exit>() {
                Some(I32Exit(code)) => *code,
                None => {
                    return Err(Error::FunctionCall {
                        function_name: "run".to_string(),
                        reason: e.to_string(),
                    });
                }
            },
        };
        Ok(CommandOutput {
            exit_code,
            stdout: stdout.contents().to_vec(),
            stderr: stderr.contents().to_vec(),
        })
    }
}

impl WasmInstance for WasiComponentInstance {
    fn state(&self) -> WasmInstanceState {
        WasmInstanceState::Running
    }

    fn memory_usage(&self) -> usize {
        0
    }

    fn fuel_usage(&self) -> Option<u64> {
        None
    }

    fn reset_fuel(&self) -> Result<()> {
        Ok(())
    }

    fn add_fuel(&self, _fuel: u64) -> Result<()> {
        Ok(())
    }

    unsafe fn memory_ptr(&self) -> Result<*mut u8> {
        Err(Error::UnsupportedOperation {
            message: "Direct memory access not supported for components".to_string(),
        })
    }

    fn memory_size(&self) -> usize {
        0
    }

    fn function_caller(&self) -> Box<dyn WasmFunctionCaller> {
        Box::new(ComponentFunctionCaller {})
    }

    fn call_simple_function(&self, function_name: &str, params: &[i32]) -> Result<i32> {
        if function_name != "run" || !params.is_empty() {
            return Err(Error::UnsupportedOperation {
                message: "components expose only `run` (no parameters), which runs the command \
                          and returns its exit code"
                    .to_string(),
            });
        }
        Ok(self.run_command()?.exit_code)
    }
}
