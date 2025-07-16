use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Instance identifier for tracking across the sandbox
pub type InstanceId = uuid::Uuid;

/// Resource types that can be limited
#[derive(Debug, Clone)]
pub enum ResourceKind {
    Memory,
    CpuTime,
    Fuel,
    FileHandles,
    NetworkConnections,
    ExecutionTime,
}

/// Security context providing details about attempted operations
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub attempted_operation: String,
    pub required_capability: String,
    pub available_capabilities: Vec<String>,
}

/// Enhanced error types for the wasm-sandbox crate with detailed context
#[derive(Error, Debug)]
pub enum SandboxError {
    /// Security violation with detailed context
    #[error("Security violation: {violation}")]
    SecurityViolation { 
        violation: String, 
        instance_id: Option<InstanceId>,
        context: SecurityContext,
    },
    
    /// Resource exhausted with specific usage details
    #[error("Resource exhausted: {kind:?} used {used}/{limit}")]
    ResourceExhausted { 
        kind: ResourceKind, 
        limit: u64, 
        used: u64,
        instance_id: Option<InstanceId>,
        suggestion: Option<String>,
    },
    
    /// WASM runtime error with function context
    #[error("WebAssembly runtime error in function '{function}': {message}")]
    WasmRuntime {
        function: String,
        instance_id: Option<InstanceId>,
        message: String,
    },
    
    /// Configuration error with helpful suggestions
    #[error("Configuration error: {message}")]
    Configuration { 
        message: String,
        suggestion: Option<String>,
        field: Option<String>,
    },
    
    /// Module compilation or loading error
    #[error("Module error: {operation} failed - {reason}")]
    Module {
        operation: String,
        reason: String,
        suggestion: Option<String>,
    },
    
    /// Instance management error
    #[error("Instance error: {operation} failed for instance {instance_id:?} - {reason}")]
    Instance {
        operation: String,
        instance_id: Option<InstanceId>,
        reason: String,
    },
    
    /// Communication channel error
    #[error("Communication error: {channel} channel failed - {reason}")]
    Communication {
        channel: String,
        reason: String,
        instance_id: Option<InstanceId>,
    },
    
    /// Timeout error with context
    #[error("Operation timed out after {duration:?}: {operation}")]
    Timeout {
        operation: String,
        duration: Duration,
        instance_id: Option<InstanceId>,
    },

    /// Function call error
    #[error("Failed to call function '{function_name}': {reason}")]
    FunctionCall {
        function_name: String,
        reason: String,
    },

    /// File system error with path context
    #[error("Filesystem error: {operation} failed on '{path:?}' - {reason}")]
    Filesystem {
        operation: String,
        path: PathBuf,
        reason: String,
    },

    /// Network error with context
    #[error("Network error: {operation} failed - {reason}")]
    Network {
        operation: String,
        reason: String,
        endpoint: Option<String>,
    },

    /// Serialization/deserialization error
    #[error("Serialization error: {format} {operation} failed - {reason}")]
    Serialization {
        format: String, // json, messagepack, etc.
        operation: String, // serialize, deserialize
        reason: String,
    },

    /// Invalid input with validation details
    #[error("Invalid input: {field} - {reason}")]
    InvalidInput {
        field: String,
        reason: String,
        suggestion: Option<String>,
    },

    /// Resource not found
    #[error("Resource not found: {resource_type} '{identifier}' not found")]
    NotFound {
        resource_type: String, // instance, module, etc.
        identifier: String,
    },

    /// Operation not supported
    #[error("Unsupported operation: {operation} is not supported in this context")]
    Unsupported {
        operation: String,
        context: String,
        suggestion: Option<String>,
    },

    /// Generic error with a message
    #[error("Generic error: {message}")]
    Generic { message: String },
    
    /// Template rendering error
    #[error("Template error: {message}")]
    Template { message: String },
    
    /// Compilation error
    #[error("Compilation error: {message}")]
    Compilation { message: String },

    /// Instance creation error
    #[error("Instance creation error: {reason}")]
    InstanceCreation {
        reason: String,
        instance_id: Option<InstanceId>,
    },
    
    /// Wrapper generation error
    #[error("Wrapper generation error: {reason}")]
    WrapperGeneration {
        reason: String,
        wrapper_type: Option<String>,
    },

    /// Module load error
    #[error("Module load error: {message}")]
    ModuleLoad { message: String },

    /// Runtime initialization error
    #[error("Runtime initialization error: {message}")]
    RuntimeInitialization { message: String },

    /// IO error with context
    #[error("IO error: {message}")]
    IoError { message: String },

    /// Capability error
    #[error("Capability error: {message}")]
    Capability { message: String },

    /// Resource limit error
    #[error("Resource limit error: {message}")]
    ResourceLimit { message: String },

    /// Unsupported operation error
    #[error("Unsupported operation: {message}")]
    UnsupportedOperation { message: String },

    // Wrapped errors from external sources
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("MessagePack encode error: {0}")]
    MessagePack(#[from] rmp_serde::encode::Error),

    #[error("MessagePack decode error: {0}")]
    MessagePackDecode(#[from] rmp_serde::decode::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Legacy aliases for backward compatibility
impl SandboxError {
    /// Create a security violation error
    pub fn security_violation(violation: impl Into<String>, context: SecurityContext) -> Self {
        Self::SecurityViolation {
            violation: violation.into(),
            instance_id: None,
            context,
        }
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted(
        kind: ResourceKind,
        used: u64,
        limit: u64,
        suggestion: Option<String>,
    ) -> Self {
        Self::ResourceExhausted {
            kind,
            limit,
            used,
            instance_id: None,
            suggestion,
        }
    }

    /// Create a configuration error with suggestion
    pub fn config_error(message: impl Into<String>, suggestion: Option<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            suggestion,
            field: None,
        }
    }

    /// Create a module loading error
    pub fn module_load_error(reason: impl Into<String>) -> Self {
        Self::Module {
            operation: "load".to_string(),
            reason: reason.into(),
            suggestion: Some("Check that the WASM file is valid and accessible".to_string()),
        }
    }

    /// Create a function call error
    pub fn function_call_error(function_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::FunctionCall {
            function_name: function_name.into(),
            reason: reason.into(),
        }
    }
}

impl Clone for SandboxError {
    fn clone(&self) -> Self {
        match self {
            SandboxError::SecurityViolation { violation, instance_id, context } => {
                SandboxError::SecurityViolation {
                    violation: violation.clone(),
                    instance_id: *instance_id,
                    context: context.clone(),
                }
            }
            SandboxError::ResourceExhausted { kind, limit, used, instance_id, suggestion } => {
                SandboxError::ResourceExhausted {
                    kind: kind.clone(),
                    limit: *limit,
                    used: *used,
                    instance_id: *instance_id,
                    suggestion: suggestion.clone(),
                }
            }
            SandboxError::WasmRuntime { function, instance_id, message } => {
                SandboxError::WasmRuntime {
                    function: function.clone(),
                    instance_id: *instance_id,
                    message: message.clone(),
                }
            }
            SandboxError::Configuration { message, suggestion, field } => {
                SandboxError::Configuration {
                    message: message.clone(),
                    suggestion: suggestion.clone(),
                    field: field.clone(),
                }
            }
            SandboxError::Module { operation, reason, suggestion } => {
                SandboxError::Module {
                    operation: operation.clone(),
                    reason: reason.clone(),
                    suggestion: suggestion.clone(),
                }
            }
            SandboxError::Instance { operation, instance_id, reason } => {
                SandboxError::Instance {
                    operation: operation.clone(),
                    instance_id: *instance_id,
                    reason: reason.clone(),
                }
            }
            SandboxError::Communication { channel, reason, instance_id } => {
                SandboxError::Communication {
                    channel: channel.clone(),
                    reason: reason.clone(),
                    instance_id: *instance_id,
                }
            }
            SandboxError::Timeout { operation, duration, instance_id } => {
                SandboxError::Timeout {
                    operation: operation.clone(),
                    duration: *duration,
                    instance_id: *instance_id,
                }
            }
            SandboxError::FunctionCall { function_name, reason } => {
                SandboxError::FunctionCall {
                    function_name: function_name.clone(),
                    reason: reason.clone(),
                }
            }
            SandboxError::Filesystem { operation, path, reason } => {
                SandboxError::Filesystem {
                    operation: operation.clone(),
                    path: path.clone(),
                    reason: reason.clone(),
                }
            }
            SandboxError::Network { operation, reason, endpoint } => {
                SandboxError::Network {
                    operation: operation.clone(),
                    reason: reason.clone(),
                    endpoint: endpoint.clone(),
                }
            }
            SandboxError::Serialization { format, operation, reason } => {
                SandboxError::Serialization {
                    format: format.clone(),
                    operation: operation.clone(),
                    reason: reason.clone(),
                }
            }
            SandboxError::InvalidInput { field, reason, suggestion } => {
                SandboxError::InvalidInput {
                    field: field.clone(),
                    reason: reason.clone(),
                    suggestion: suggestion.clone(),
                }
            }
            SandboxError::NotFound { resource_type, identifier } => {
                SandboxError::NotFound {
                    resource_type: resource_type.clone(),
                    identifier: identifier.clone(),
                }
            }
            SandboxError::Unsupported { operation, context, suggestion } => {
                SandboxError::Unsupported {
                    operation: operation.clone(),
                    context: context.clone(),
                    suggestion: suggestion.clone(),
                }
            }
            SandboxError::Generic { message } => {
                SandboxError::Generic {
                    message: message.clone(),
                }
            }
            SandboxError::Template { message } => {
                SandboxError::Template {
                    message: message.clone(),
                }
            }
            SandboxError::Compilation { message } => {
                SandboxError::Compilation {
                    message: message.clone(),
                }
            }
            SandboxError::InstanceCreation { reason, instance_id } => {
                SandboxError::InstanceCreation {
                    reason: reason.clone(),
                    instance_id: *instance_id,
                }
            }
            SandboxError::WrapperGeneration { reason, wrapper_type } => {
                SandboxError::WrapperGeneration {
                    reason: reason.clone(),
                    wrapper_type: wrapper_type.clone(),
                }
            }
            SandboxError::ModuleLoad { message } => {
                SandboxError::ModuleLoad {
                    message: message.clone(),
                }
            }
            SandboxError::RuntimeInitialization { message } => {
                SandboxError::RuntimeInitialization {
                    message: message.clone(),
                }
            }
            SandboxError::IoError { message } => {
                SandboxError::IoError {
                    message: message.clone(),
                }
            }
            SandboxError::Capability { message } => {
                SandboxError::Capability {
                    message: message.clone(),
                }
            }
            SandboxError::ResourceLimit { message } => {
                SandboxError::ResourceLimit {
                    message: message.clone(),
                }
            }
            SandboxError::UnsupportedOperation { message } => {
                SandboxError::UnsupportedOperation {
                    message: message.clone(),
                }
            }
            // For wrapped errors that don't implement Clone, we create a new error with the string representation
            SandboxError::Io(e) => SandboxError::Filesystem {
                operation: "io".to_string(),
                path: "unknown".into(),
                reason: e.to_string(),
            },
            SandboxError::Json(e) => SandboxError::Serialization {
                format: "json".to_string(),
                operation: "parse".to_string(),
                reason: e.to_string(),
            },
            SandboxError::MessagePack(e) => SandboxError::Serialization {
                format: "messagepack".to_string(),
                operation: "encode".to_string(),
                reason: e.to_string(),
            },
            SandboxError::MessagePackDecode(e) => SandboxError::Serialization {
                format: "messagepack".to_string(),
                operation: "decode".to_string(),
                reason: e.to_string(),
            },
            SandboxError::Other(e) => SandboxError::Configuration {
                message: e.to_string(),
                suggestion: None,
                field: None,
            },
        }
    }
}

/// Result type for the wasm-sandbox crate using the enhanced error type
pub type Result<T> = std::result::Result<T, SandboxError>;

/// Legacy error type alias for backward compatibility  
pub type Error = SandboxError;
