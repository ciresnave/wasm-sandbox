//! Code generation for wrapping applications

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub mod cli_tool;
pub mod http_server;
pub mod mcp_server;
pub mod generic;
pub mod http_server_impl;

// Re-export HTTP server generator
pub use http_server_impl::{HttpServerGenerator, HttpServerConfig};

use crate::error::Result;

/// Wrapper generator for sandboxing applications
pub trait WrapperGenerator {
    /// Generate wrapper code for the target application
    fn generate_wrapper(&self, spec: &WrapperSpec) -> Result<String>;
    
    /// Compile the wrapper to WebAssembly
    fn compile_wrapper(&self, code: &str, output_path: &Path) -> Result<()>;
    
    /// Generate and compile in one step
    fn generate_and_compile(
        &self,
        spec: &WrapperSpec,
        output_path: &Path,
    ) -> Result<()> {
        let code = self.generate_wrapper(spec)?;
        self.compile_wrapper(&code, output_path)
    }
}

/// Type of application to wrap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApplicationType {
    /// HTTP server
    HttpServer { port: u16 },
    
    /// MCP server
    McpServer { port: u16, schema_path: Option<PathBuf> },
    
    /// CLI tool
    CliTool { interactive: bool },
    
    /// Generic executable
    Generic,
}

/// Communication specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationSpec {
    /// Communication protocol
    pub protocol: CommunicationProtocol,
    
    /// Bidirectional channels
    pub channels: Vec<String>,
    
    /// RPC functions
    pub rpc_functions: HashMap<String, String>,
}

impl Default for CommunicationSpec {
    fn default() -> Self {
        Self {
            protocol: CommunicationProtocol::Json,
            channels: vec!["default".to_string()],
            rpc_functions: HashMap::new(),
        }
    }
}

/// Communication protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationProtocol {
    /// JSON
    Json,
    
    /// MessagePack
    MessagePack,
    
    /// Custom protocol
    Custom(String),
}

/// Specification for wrapper generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrapperSpec {
    /// Type of application to wrap
    pub app_type: ApplicationType,
    
    /// Path to the application
    pub app_path: PathBuf,
    
    /// Command-line arguments
    pub arguments: Vec<String>,
    
    /// Environment variables
    pub environment: HashMap<String, String>,
    
    /// Working directory
    pub working_directory: Option<PathBuf>,
    
    /// Communication protocol settings
    pub communication: CommunicationSpec,
    
    /// Custom template variables
    pub template_variables: HashMap<String, String>,
}
