//! Model Context Protocol (MCP) server wrapper implementation

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};
use crate::wrappers::{WrapperGenerator, WrapperSpec, ApplicationType};

/// MCP server wrapper generator
pub struct McpServerGenerator;

impl WrapperGenerator for McpServerGenerator {
    fn generate_wrapper(&self, spec: &WrapperSpec) -> Result<String> {
        // Extract MCP-specific options
        let (port, schema_path) = match &spec.app_type {
            ApplicationType::McpServer { port, schema_path } => (*port, schema_path.clone()),
            _ => return Err(Error::WrapperGeneration(
                "Application type must be McpServer".to_string()
            )),
        };
        
        // Generate the wrapper code
        self.generate_code(spec, port, schema_path.as_ref())
    }
    
    fn compile_wrapper(&self, code: &str, output_path: &Path) -> Result<()> {
        // Create a temporary directory for building
        let temp_dir = tempfile::tempdir()
            .map_err(|e| Error::WrapperGeneration(
                format!("Failed to create temporary directory: {}", e)
            ))?;
        
        // Write code to a temporary file
        let src_path = temp_dir.path().join("src/main.rs");
        std::fs::create_dir_all(src_path.parent().unwrap())
            .map_err(|e| Error::WrapperGeneration(
                format!("Failed to create source directory: {}", e)
            ))?;
            
        std::fs::write(&src_path, code)
            .map_err(|e| Error::WrapperGeneration(
                format!("Failed to write source code: {}", e)
            ))?;
            
        // Create Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "mcp-server-wrapper"
version = "0.1.0"
edition = "2021"

[dependencies]
wasm-sandbox = {{ path = "../.." }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tokio = {{ version = "1.0", features = ["full"] }}
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
model-context-protocol = "0.1.0"
axum = "0.6.0"
tower = "0.4.0"
tower-http = {{ version = "0.4.0", features = ["cors"] }}
jsonschema = "0.17.0"
"#
        );
        
        std::fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)
            .map_err(|e| Error::WrapperGeneration(
                format!("Failed to write Cargo.toml: {}", e)
            ))?;
            
        // Run cargo build
        let status = Command::new("cargo")
            .current_dir(temp_dir.path())
            .args(&["build", "--target", "wasm32-wasi", "--release"])
            .status()
            .map_err(|e| Error::WrapperGeneration(
                format!("Failed to run cargo build: {}", e)
            ))?;
            
        if !status.success() {
            return Err(Error::WrapperGeneration(
                "Cargo build failed".to_string()
            ));
        }
        
        // Copy the output file
        let wasm_path = temp_dir.path()
            .join("target/wasm32-wasi/release/mcp-server-wrapper.wasm");
            
        std::fs::copy(wasm_path, output_path)
            .map_err(|e| Error::WrapperGeneration(
                format!("Failed to copy output file: {}", e)
            ))?;
            
        Ok(())
    }
}

impl McpServerGenerator {
    /// Generate MCP server wrapper code
    fn generate_code(&self, spec: &WrapperSpec, port: u16, schema_path: Option<&PathBuf>) -> Result<String> {
        let app_path = spec.app_path.to_string_lossy();
        let args = spec.arguments.join(", ");
        
        // Format environment variables
        let env_vars = spec.environment.iter()
            .map(|(k, v)| format!("        env_map.insert(\"{}\".to_string(), \"{}\".to_string());", k, v))
            .collect::<Vec<_>>()
            .join("\n");
            
        // Handle schema path
        let schema_code = match schema_path {
            Some(path) => format!(
                r#"    // Load JSON schema
    let schema_path = "{}";
    let schema = std::fs::read_to_string(schema_path)
        .map_err(|e| anyhow::anyhow!("Failed to read schema: {{}}", e))?;
    let schema: serde_json::Value = serde_json::from_str(&schema)
        .map_err(|e| anyhow::anyhow!("Failed to parse schema: {{}}", e))?;"#,
                path.to_string_lossy()
            ),
            None => "    // No schema provided".to_string(),
        };
            
        let code = format!(
            r#"//! MCP server wrapper for WASM sandbox

use std::{{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{{Arc, Mutex}},
}};

use wasm_sandbox::{{
    WasmSandbox, InstanceId, SandboxConfig, InstanceConfig,
    security::{{ResourceLimits, Capabilities}},
    communication::CommunicationChannel,
}};

use axum::{{
    routing::{{get, post}},
    extract::{{Extension, Json}},
    response::{{Response, IntoResponse}},
    http::{{StatusCode, header}},
    Router,
}};
use tower_http::cors::CorsLayer;
use serde_json::Value;
use tracing::{{info, error, debug, warn}};
use anyhow::Result;

/// Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting MCP server on port {port}");
    
{schema_code}
    
    // Create environment map
    let mut env_map = HashMap::new();
{env_vars}
    
    // Create sandbox
    let sandbox = WasmSandbox::new()?;
    let sandbox = Arc::new(Mutex::new(sandbox));
    
    // Create router
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(vec![header::CONTENT_TYPE]);
        
    let router = Router::new()
        .route("/v1/models", get(list_models))
        .route("/v1/models/:model/generate", post(generate))
        .layer(Extension(sandbox.clone()))
        .layer(cors);
        
    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], {port}));
    info!("MCP server listening on {{}}", addr);
    
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;
        
    Ok(())
}}

/// List available models
async fn list_models() -> impl IntoResponse {{
    // In a real implementation, this would return the list of models
    // For now, we just return a simple JSON response
    let response = serde_json::json!({{
        "models": [
            {{
                "id": "default-model",
                "name": "Default Model",
                "version": "1.0.0",
            }}
        ]
    }});
    
    Json(response)
}}

/// Generate text from model
async fn generate(
    Extension(sandbox): Extension<Arc<Mutex<WasmSandbox>>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {{
    // In a real implementation, this would process the request and call the model
    // For now, we just return a simple response
    let response = serde_json::json!({{
        "model": "default-model",
        "created": chrono::Utc::now().timestamp(),
        "response": {{
            "content": "This is a placeholder response from the MCP server wrapper.",
        }}
    }});
    
    Json(response)
}}
"#,
            port = port,
            schema_code = schema_code,
            env_vars = env_vars,
        );
        
        Ok(code)
    }
}
