//! HTTP Server wrapper generator implementation

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::wrappers::{WrapperGenerator, WrapperSpec, ApplicationType};
use crate::templates::HTTP_SERVER_TEMPLATE;
use crate::compiler::{CompilerOptions, BuildProfile, OptimizationLevel};
use crate::compiler::{CargoCompiler, Compiler};

/// HTTP Server wrapper configuration
#[derive(Clone)]
pub struct HttpServerConfig {
    /// Server port
    pub port: u16,
    
    /// Server host
    pub host: String,
    
    /// Maximum request body size in bytes
    pub max_body_size: usize,
    
    /// Request timeout in seconds
    pub request_timeout: u64,
    
    /// CORS configuration
    pub cors_enabled: bool,
    
    /// CORS allowed origins
    pub cors_allowed_origins: Vec<String>,
    
    /// CORS allowed methods
    pub cors_allowed_methods: Vec<String>,
    
    /// CORS allowed headers
    pub cors_allowed_headers: Vec<String>,
    
    /// Static file serving configuration
    pub serve_static: bool,
    
    /// Static file directory
    pub static_dir: Option<PathBuf>,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            max_body_size: 10 * 1024 * 1024, // 10MB
            request_timeout: 30,
            cors_enabled: false,
            cors_allowed_origins: vec!["*".to_string()],
            cors_allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            cors_allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            serve_static: false,
            static_dir: None,
        }
    }
}

/// HTTP Server wrapper generator
pub struct HttpServerGenerator {
    /// Configuration
    config: HttpServerConfig,
    
    /// Cargo compiler
    compiler: CargoCompiler,
}

impl HttpServerGenerator {
    /// Create a new HTTP Server wrapper generator with default configuration
    pub fn new() -> Self {
        Self {
            config: HttpServerConfig::default(),
            compiler: CargoCompiler::new(),
        }
    }
    
    /// Create a new HTTP Server wrapper generator with custom configuration
    pub fn with_config(config: HttpServerConfig) -> Self {
        Self {
            config,
            compiler: CargoCompiler::new(),
        }
    }
    
    /// Generate Cargo.toml for the wrapper project
    fn generate_cargo_toml(&self, app_name: &str) -> String {
        format!(
            r#"[package]
name = "{}-http-server-wrapper"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2.84"
wasm-bindgen-futures = "0.4.34"
js-sys = "0.3.61"
console_error_panic_hook = "0.1.7"
serde = {{ version = "1.0.152", features = ["derive"] }}
serde_json = "1.0.93"
rmp-serde = "1.1.1"
log = "0.4.17"
futures = "0.3.28"
once_cell = "1.17.1"
anyhow = "1.0.70"
hyper = {{ version = "0.14.25", features = ["full"] }}
tokio = {{ version = "1.27.0", features = ["full"] }}
http = "0.2.9"
bytes = "1.4.0"
base64 = "0.21.0"

[dependencies.web-sys]
version = "0.3.61"
features = [
    "console",
    "Document",
    "Element",
    "HtmlElement",
    "Window",
]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
"#,
            app_name
        )
    }
    
    /// Render the wrapper code using the template
    fn render_wrapper_code(&self, spec: &WrapperSpec) -> Result<String> {
        match &spec.app_type {
            ApplicationType::HttpServer { port } => {
                let config = HttpServerConfig {
                    port: *port,
                    ..self.config.clone()
                };
                
                // Generate the template context
                let mut context = HashMap::new();
                context.insert("app_path".to_string(), spec.app_path.to_string_lossy().to_string());
                context.insert("app_args".to_string(), serde_json::to_string(&spec.arguments)?);
                context.insert("port".to_string(), port.to_string());
                context.insert("host".to_string(), config.host);
                context.insert("max_body_size".to_string(), config.max_body_size.to_string());
                context.insert("request_timeout".to_string(), config.request_timeout.to_string());
                context.insert("cors_enabled".to_string(), config.cors_enabled.to_string());
                context.insert("cors_allowed_origins".to_string(), serde_json::to_string(&config.cors_allowed_origins)?);
                context.insert("cors_allowed_methods".to_string(), serde_json::to_string(&config.cors_allowed_methods)?);
                context.insert("cors_allowed_headers".to_string(), serde_json::to_string(&config.cors_allowed_headers)?);
                
                // Render the template
                let mut template = HTTP_SERVER_TEMPLATE.to_string();
                
                // Replace template variables
                for (key, value) in context {
                    let placeholder = format!("{{{{ {} }}}}", key);
                    template = template.replace(&placeholder, &value);
                }
                
                Ok(template)
            }
            _ => Err(Error::WrapperGeneration {
                reason: "Not an HTTP server application".to_string(),
                wrapper_type: Some("http_server".to_string()),
            }),
        }
    }
}

impl WrapperGenerator for HttpServerGenerator {
    fn generate_wrapper(&self, spec: &WrapperSpec) -> Result<String> {
        self.render_wrapper_code(spec)
    }
    
    fn compile_wrapper(&self, code: &str, output_path: &Path) -> Result<()> {
        // Create a temporary directory for the project
        let temp_dir = tempfile::tempdir()?;
        let project_dir = temp_dir.path();
        
        // Create src directory
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir)?;
        
        // Write the code to lib.rs
        fs::write(src_dir.join("lib.rs"), code)?;
        
        // Write Cargo.toml
        let app_name = "http_server_app";
        let cargo_toml = self.generate_cargo_toml(app_name);
        fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
        
        // Configure compiler options
        let compiler_options = CompilerOptions {
            target: "wasm32-wasi".to_string(),
            opt_level: OptimizationLevel::Speed,
            profile: BuildProfile::Release,
            ..CompilerOptions::default()
        };
        
        // Compile the project
        self.compiler.compile(project_dir, output_path, &compiler_options)?;
        
        Ok(())
    }
}
