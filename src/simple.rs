//! Simplified one-liner APIs for common use cases

use std::path::Path;
use serde_json::Value;
use crate::error::Result;
use crate::{WasmSandbox, InstanceConfig};

/// One-liner execution functions for maximum ease of use
pub struct SimpleSandbox;

impl SimpleSandbox {
    /// Execute a single function with minimal setup - the simplest possible API
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use wasm_sandbox::simple;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), wasm_sandbox::SandboxError> {
    ///     // Execute a function from a Rust source file
    ///     let result: i32 = simple::run("./calculator.rs", "add", &[serde_json::Value::from(5), serde_json::Value::from(3)]).await?;
    ///     
    ///     // Execute from pre-compiled WASM
    ///     let result: String = simple::run("./module.wasm", "process", &[serde_json::Value::from("hello")]).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn run<T, P>(
        source_path: P,
        function_name: &str,
        args: &[Value],
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned + 'static,
        P: AsRef<Path>,
    {
        // Use secure defaults
        let config = InstanceConfig::builder()
            .memory_limit(crate::config::MemoryUnit::mb(64))  // 64MB default
            .timeout(30u64)    // 30 second timeout
            .network_deny_all()                               // No network by default
            .build()?;

        Self::run_with_config(source_path, function_name, args, config).await
    }

    /// Execute with custom configuration but still simple
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use wasm_sandbox::{simple, InstanceConfig, MemoryUnit};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), wasm_sandbox::SandboxError> {
    ///     let config = InstanceConfig::builder()
    ///         .memory_limit(128.mb())
    ///         .timeout(60)
    ///         .filesystem_read(&["/data"])
    ///         .build()?;
    ///     
    ///     let result: Vec<String> = simple::run_with_config(
    ///         "./data_processor.rs", 
    ///         "process_files", 
    ///         &["/data/input.txt".into()],
    ///         config
    ///     ).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn run_with_config<T, P>(
        source_path: P,
        function_name: &str,
        args: &[Value],
        config: InstanceConfig,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned + 'static,
        P: AsRef<Path>,
    {
        // Create sandbox and execute
        let mut sandbox = WasmSandbox::new()?;
        let wasm_bytes = crate::compile_source_to_wasm(
            source_path.as_ref().to_str().ok_or_else(|| crate::Error::InvalidInput {
                field: "source_path".to_string(),
                reason: "Path contains invalid UTF-8 characters".to_string(),
                suggestion: Some("Use a path with valid UTF-8 characters".to_string()),
            })?
        ).await?;
        let module_id = sandbox.load_module(&wasm_bytes)?;
        let instance_id = sandbox.create_instance(module_id, Some(config))?;
        
        let result: Value = sandbox.call_function(instance_id, function_name, args.to_vec()).await?;
        let deserialized = serde_json::from_value(result)?;
        Ok(deserialized)
    }

    /// Execute multiple functions in sequence with the same instance
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use wasm_sandbox::simple;
    /// use serde_json::json;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), wasm_sandbox::SandboxError> {
    ///     let calls = vec![
    ///         ("initialize", vec![json!("config.json")]),
    ///         ("process", vec![json!("data1.txt")]),
    ///         ("process", vec![json!("data2.txt")]),
    ///         ("finalize", vec![]),
    ///     ];
    ///     
    ///     let results = simple::run_sequence("./processor.rs", &calls).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn run_sequence<P>(
        source_path: P,
        function_calls: &[(&str, Vec<Value>)],
    ) -> Result<Vec<Value>>
    where
        P: AsRef<Path>,
    {
        let config = InstanceConfig::builder()
            .memory_limit(crate::config::MemoryUnit::mb(64))
            .timeout(30u64)
            .network_deny_all()
            .build()?;

        Self::run_sequence_with_config(source_path, function_calls.iter().map(|(name, args)| (name.to_string(), args.clone())).collect(), config).await
    }

    /// Execute multiple functions with custom configuration
    pub async fn run_sequence_with_config<P>(
        source_path: P,
        function_calls: Vec<(String, Vec<Value>)>,
        config: InstanceConfig,
    ) -> Result<Vec<Value>>
    where
        P: AsRef<Path>,
    {
        let mut sandbox = WasmSandbox::new()?;
        let wasm_bytes = crate::compile_source_to_wasm(
            source_path.as_ref().to_str().ok_or_else(|| crate::Error::InvalidInput {
                field: "source_path".to_string(),
                reason: "Path contains invalid UTF-8 characters".to_string(),
                suggestion: Some("Use a path with valid UTF-8 characters".to_string()),
            })?
        ).await?;
        let module_id = sandbox.load_module(&wasm_bytes)?;
        let instance_id = sandbox.create_instance(module_id, Some(config))?;
        
        let mut results = Vec::new();
        for (function_name, args) in function_calls {
            let result: Value = sandbox.call_function(instance_id, &function_name, args).await?;
            results.push(result);
        }
        
        Ok(results)
    }

    /// Create a reusable sandbox instance for multiple operations
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use wasm_sandbox::simple;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), wasm_sandbox::SandboxError> {
    ///     // Create a reusable sandbox
    ///     let mut sandbox = simple::create_reusable("./calculator.rs").await?;
    ///     
    ///     // Use it multiple times
    ///     let result1: i32 = sandbox.call("add".to_string(), vec![serde_json::Value::from(5), serde_json::Value::from(3)]).await?;
    ///     let result2: i32 = sandbox.call("multiply".to_string(), vec![serde_json::Value::from(4), serde_json::Value::from(6)]).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_reusable<P>(source_path: P) -> Result<ReusableSandbox>
    where
        P: AsRef<Path>,
    {
        let config = InstanceConfig::builder()
            .memory_limit(crate::config::MemoryUnit::mb(64))
            .timeout(30u64)
            .network_deny_all()
            .build()?;

        Self::create_reusable_with_config(source_path, config).await
    }

    /// Create a reusable sandbox with custom configuration
    pub async fn create_reusable_with_config<P>(
        source_path: P,
        config: InstanceConfig,
    ) -> Result<ReusableSandbox>
    where
        P: AsRef<Path>,
    {
        let mut sandbox = WasmSandbox::new()?;
        let wasm_bytes = crate::compile_source_to_wasm(
            source_path.as_ref().to_str().ok_or_else(|| crate::Error::InvalidInput {
                field: "source_path".to_string(),
                reason: "Path contains invalid UTF-8 characters".to_string(),
                suggestion: Some("Use a path with valid UTF-8 characters".to_string()),
            })?
        ).await?;
        let module_id = sandbox.load_module(&wasm_bytes)?;
        let instance_id = sandbox.create_instance(module_id, Some(config))?;
        
        Ok(ReusableSandbox {
            sandbox,
            instance_id,
        })
    }
}

/// Reusable sandbox instance for multiple function calls
pub struct ReusableSandbox {
    sandbox: WasmSandbox,
    instance_id: crate::InstanceId,
}

impl ReusableSandbox {
    /// Call a function on this sandbox instance
    pub async fn call<T>(&mut self, function_name: String, args: Vec<Value>) -> Result<T>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        self.sandbox.call_function(self.instance_id, &function_name, args).await
    }

    /// Get resource usage for this instance
    pub fn get_resource_usage(&self) -> Result<crate::monitoring::DetailedResourceUsage> {
        self.sandbox.get_instance_resource_usage(self.instance_id)
    }

    /// Reset the instance (clear memory, restart)
    pub async fn reset(&mut self) -> Result<()> {
        self.sandbox.reset_instance(self.instance_id)
    }
}

/// Module-level convenience functions (can be used as `wasm_sandbox::run()`)
pub use SimpleSandbox as simple;

/// Top-level convenience function for the absolute simplest usage
/// 
/// # Examples
/// 
/// ```rust,no_run
/// #[tokio::main]
/// async fn main() -> Result<(), wasm_sandbox::SandboxError> {
///     // The simplest possible WebAssembly execution:
///     let result: i32 = wasm_sandbox::run("./calculator.rs", "add", &[serde_json::Value::from(5), serde_json::Value::from(3)]).await?;
///     Ok(())
/// }
/// ```
pub async fn run<T, P>(
    source_path: P,
    function_name: &str,
    args: &[Value],
) -> Result<T>
where
    T: serde::de::DeserializeOwned + 'static,
    P: AsRef<Path>,
{
    SimpleSandbox::run(source_path, function_name, args).await
}

/// Top-level function to create a reusable sandbox
/// 
/// # Examples
/// 
/// ```rust,no_run
/// #[tokio::main]
/// async fn main() -> Result<(), wasm_sandbox::SandboxError> {
///     let mut sandbox = wasm_sandbox::from_source("./my_module.rs").await?;
///     let result1: () = sandbox.call("function1".to_string(), vec![]).await?;
///     let result2: i32 = sandbox.call("function2".to_string(), vec![serde_json::Value::from(42)]).await?;
///     Ok(())
/// }
/// ```
pub async fn from_source<P>(source_path: P) -> Result<ReusableSandbox>
where
    P: AsRef<Path>,
{
    SimpleSandbox::create_reusable(source_path).await
}

/// Module-level run_with_config function
pub async fn run_with_config<T, P>(
    source_path: P,
    function_name: &str,
    args: &[Value],
    config: InstanceConfig,
) -> Result<T>
where
    T: serde::de::DeserializeOwned + 'static,
    P: AsRef<Path>,
{
    SimpleSandbox::run_with_config(source_path, function_name, args, config).await
}

/// Module-level run_sequence function
pub async fn run_sequence<P>(
    source_path: P,
    function_calls: &[(&str, Vec<Value>)],
) -> Result<Vec<Value>>
where
    P: AsRef<Path>,
{
    SimpleSandbox::run_sequence(source_path, function_calls).await
}

/// Module-level create_reusable function
pub async fn create_reusable<P>(source_path: P) -> Result<ReusableSandbox>
where
    P: AsRef<Path>,
{
    SimpleSandbox::create_reusable(source_path).await
}
