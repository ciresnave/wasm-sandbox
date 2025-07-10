use std::path::PathBuf;
use thiserror::Error;

/// Error types for the wasm-sandbox crate
#[derive(Error, Debug)]
pub enum Error {
    /// Error initializing the runtime
    #[error("Failed to initialize WebAssembly runtime: {0}")]
    RuntimeInitialization(String),

    /// Error loading a WASM module
    #[error("Failed to load WASM module: {0}")]
    ModuleLoad(String),

    /// Error creating a WASM instance
    #[error("Failed to create WASM instance: {0}")]
    InstanceCreation(String),

    /// Error calling a WASM function
    #[error("Failed to call function '{function_name}': {reason}")]
    FunctionCall {
        function_name: String,
        reason: String,
    },

    /// Error with runtime capabilities
    #[error("Capability error: {0}")]
    Capability(String),

    /// Error with resource limits
    #[error("Resource limit error: {0}")]
    ResourceLimit(String),

    /// Error with host-guest communication
    #[error("Communication error: {0}")]
    Communication(String),

    /// Error with wrapper generation
    #[error("Wrapper generation error: {0}")]
    WrapperGeneration(String),

    /// Error with template processing
    #[error("Template error: {0}")]
    Template(String),

    /// Error with WASI configuration
    #[error("WASI configuration error: {0}")]
    WasiConfig(String),

    /// Error compiling code
    #[error("Compilation error: {0}")]
    Compilation(String),

    /// File system error
    #[error("File system error: {0}")]
    FileSystem(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// MessagePack serialization/deserialization error
    #[error("MessagePack error: {0}")]
    MessagePack(#[from] rmp_serde::encode::Error),

    /// MessagePack deserialization error
    #[error("MessagePack decode error: {0}")]
    MessagePackDecode(#[from] rmp_serde::decode::Error),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Invalid path
    #[error("Invalid path: {0:?}")]
    InvalidPath(PathBuf),

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Security violation
    #[error("Security violation: {0}")]
    SecurityViolation(String),

    /// Instance not found
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    /// Module not found
    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    /// Timeout
    #[error("Operation timed out after {0} ms")]
    Timeout(u64),

    /// Generic error
    #[error("{0}")]
    Generic(String),

    /// Wrapped errors from other sources
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type for the wasm-sandbox crate
pub type Result<T> = std::result::Result<T, Error>;
