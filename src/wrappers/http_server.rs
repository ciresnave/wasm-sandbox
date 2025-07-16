//! HTTP server wrapper implementation

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use crate::error::Result;
use crate::wrappers::{WrapperGenerator, WrapperSpec};

/// HTTP request method
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    /// GET request
    GET,
    
    /// POST request
    POST,
    
    /// PUT request
    PUT,
    
    /// DELETE request
    DELETE,
    
    /// PATCH request
    PATCH,
    
    /// HEAD request
    HEAD,
    
    /// OPTIONS request
    OPTIONS,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GET => write!(f, "GET"),
            Self::POST => write!(f, "POST"),
            Self::PUT => write!(f, "PUT"),
            Self::DELETE => write!(f, "DELETE"),
            Self::PATCH => write!(f, "PATCH"),
            Self::HEAD => write!(f, "HEAD"),
            Self::OPTIONS => write!(f, "OPTIONS"),
        }
    }
}

/// HTTP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// Request method
    pub method: HttpMethod,
    
    /// Request path
    pub path: String,
    
    /// Query parameters
    pub query: Option<String>,
    
    /// Request headers
    pub headers: Vec<(String, String)>,
    
    /// Request body
    pub body: Option<Vec<u8>>,
    
    /// Remote address
    pub remote_addr: Option<SocketAddr>,
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// Status code
    pub status: u16,
    
    /// Response headers
    pub headers: Vec<(String, String)>,
    
    /// Response body
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Create a new HTTP response
    pub fn new(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body,
        }
    }
    
    /// Create a new HTTP response with headers
    pub fn with_headers(status: u16, headers: Vec<(String, String)>, body: Vec<u8>) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }
    
    /// Create an OK response with text
    pub fn text(body: &str) -> Self {
        Self {
            status: 200,
            headers: vec![
                ("Content-Type".to_string(), "text/plain; charset=utf-8".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.as_bytes().to_vec(),
        }
    }
    
    /// Create an OK response with JSON
    pub fn json<T: Serialize>(body: &T) -> Self {
        let json = serde_json::to_vec(body).unwrap_or_default();
        Self {
            status: 200,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Content-Length".to_string(), json.len().to_string()),
            ],
            body: json,
        }
    }
    
    /// Create a not found response
    pub fn not_found() -> Self {
        Self::text("Not Found")
            .with_status(404)
    }
    
    /// Create an internal server error response
    pub fn server_error(message: &str) -> Self {
        Self::text(message)
            .with_status(500)
    }
    
    /// Set the status code
    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }
    
    /// Add a header
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpServerConfig {
    /// Bind address
    pub address: IpAddr,
    
    /// Port
    pub port: u16,
    
    /// Thread count
    pub threads: Option<usize>,
    
    /// Connection timeout in seconds
    pub timeout_seconds: Option<u64>,
    
    /// Maximum request size in bytes
    pub max_request_size: Option<usize>,
    
    /// CORS configuration
    pub cors: Option<CorsConfig>,
    
    /// TLS configuration
    pub tls: Option<TlsConfig>,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    
    /// Allow credentials
    pub allow_credentials: bool,
    
    /// Max age in seconds
    pub max_age: Option<u32>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate path
    pub cert_path: PathBuf,
    
    /// Key path
    pub key_path: PathBuf,
}

/// HTTP server wrapper generator
pub struct HttpServerGenerator;

impl WrapperGenerator for HttpServerGenerator {
    fn generate_wrapper(&self, spec: &WrapperSpec) -> Result<String> {
        // Parse the spec template variables for HTTP server options
        let config = HttpServerConfig {
            address: spec.template_variables.get("address")
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| "127.0.0.1".parse().unwrap()),
            port: spec.template_variables.get("port")
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
            threads: spec.template_variables.get("threads")
                .and_then(|s| s.parse().ok()),
            timeout_seconds: spec.template_variables.get("timeout_seconds")
                .and_then(|s| s.parse().ok()),
            max_request_size: spec.template_variables.get("max_request_size")
                .and_then(|s| s.parse().ok()),
            cors: None,
            tls: None,
        };
        
        // Generate a basic HTTP server wrapper
        let code = self.generate_code(&config, spec)?;
        
        Ok(code)
    }
    
    fn compile_wrapper(&self, code: &str, output_path: &std::path::Path) -> Result<()> {
        // For now, just write the code to the output path
        std::fs::write(output_path, code)
            .map_err(|e| crate::error::Error::WrapperGeneration {
                reason: format!("Failed to write wrapper code: {}", e),
                wrapper_type: Some("http_server".to_string()),
            })?;
        Ok(())
    }
}

impl HttpServerGenerator {
    /// Generate HTTP server wrapper code
    fn generate_code(&self, config: &HttpServerConfig, spec: &WrapperSpec) -> Result<String> {
        let address = format!("{}:{}", config.address, config.port);
        let _threads = config.threads.unwrap_or(4);
        let _timeout = config.timeout_seconds.unwrap_or(30);
        
        // Generate the server code
        let mut code = format!(
            r#"//! HTTP server wrapper for WASM sandbox

use std::{{
    net::{{IpAddr, SocketAddr}},
    sync::{{Arc, Mutex}},
    time::Duration,
}};

use tokio::{{
    net::TcpListener,
    runtime::Runtime,
    io::{{AsyncReadExt, AsyncWriteExt}},
}};
use hyper::{{
    Body, Request, Response, Server, StatusCode,
    service::{{make_service_fn, service_fn}},
    header::{{HeaderValue, CONTENT_TYPE}},
}};
use serde_json::{{Value, json}};
use tracing::{{info, error, debug, warn}};

use wasm_sandbox::{{
    WasmSandbox, InstanceId, SandboxConfig, InstanceConfig,
    security::{{ResourceLimits, Capabilities}},
}};

/// Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting HTTP server on {address}");
    
    // Create sandbox
    let mut sandbox = WasmSandbox::new()?;
    
    // Load the WASM module
    let wasm_bytes = include_bytes!("{wasm_path}");
    let module_id = sandbox.load_module(wasm_bytes)?;
    
    info!("Loaded WASM module");
    
    // Create instance config
    let instance_config = {instance_config};
    
    // Create the instance
    let instance_id = sandbox.create_instance(module_id, Some(instance_config))?;
    info!("Created WASM instance: {{instance_id}}");
    
    // Store the instance ID
    let instance_id = Arc::new(instance_id);
    
    // Create HTTP server
    let make_svc = make_service_fn(move |conn| {{
        let remote_addr = conn.remote_addr();
        let instance_id = instance_id.clone();
        let sandbox = sandbox.clone();
        
        async move {{
            Ok::<_, hyper::Error>(service_fn(move |req| {{
                let instance_id = instance_id.clone();
                let sandbox = sandbox.clone();
                handle_request(req, sandbox, instance_id, remote_addr)
            }}))
        }}
    }});
    
    let addr = "{address}".parse()?;
    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(shutdown_signal());
        
    info!("Server running on http://{address}");
    
    if let Err(e) = server.await {{
        error!("Server error: {{}}", e);
    }}
    
    Ok(())
}}

/// Handle HTTP request
async fn handle_request(
    req: Request<Body>, 
    sandbox: Arc<Mutex<WasmSandbox>>,
    instance_id: Arc<InstanceId>,
    remote_addr: SocketAddr,
) -> Result<Response<Body>, hyper::Error> {{
    // Convert hyper request to our format
    let (parts, body) = req.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await?;
    
    // Create request object
    let request = json!({{
        "method": parts.method.as_str(),
        "path": parts.uri.path(),
        "query": parts.uri.query(),
        "headers": parts.headers.iter().map(|(k, v)| {{
            (k.as_str(), v.to_str().unwrap_or_default())
        }}).collect::<Vec<_>>(),
        "body": body_bytes.to_vec(),
        "remote_addr": remote_addr.to_string(),
    }});
    
    // Call handler function in sandbox
    let response = match sandbox.lock().unwrap().call_function::<_, Value>(
        *instance_id,
        "handle_http_request",
        &request,
    ).await {{
        Ok(res) => res,
        Err(e) => {{
            error!("Error calling WASM function: {{}}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal Server Error"))?);
        }}
    }};
    
    // Convert JSON response to hyper response
    let status = response["status"].as_u64().unwrap_or(500) as u16;
    let headers = response["headers"].as_array().unwrap_or(&Vec::new());
    let body = response["body"].as_array().unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_u64().map(|n| n as u8))
        .collect::<Vec<_>>();
        
    // Build response
    let mut builder = Response::builder().status(status);
    
    // Add headers
    for header in headers {{
        if let (Some(name), Some(value)) = (
            header[0].as_str(),
            header[1].as_str(),
        ) {{
            builder = builder.header(name, value);
        }}
    }}
    
    // Add default content type if not present
    if !headers.iter().any(|h| h[0].as_str() == Some("content-type")) {{
        builder = builder.header(CONTENT_TYPE, "text/plain");
    }}
    
    Ok(builder.body(Body::from(body))?)
}}

/// Shutdown signal handler
async fn shutdown_signal() {{
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
        
    info!("Shutdown signal received, stopping server...");
}}
"#,
            address = address,
            wasm_path = spec.app_path.display(),
            instance_config = "InstanceConfig::default()",  // Simplified for now
        );
        
        // Add TLS configuration if enabled
        if let Some(_tls_config) = &config.cors {
            // Add TLS imports and config
            code = code.replace(
                "use hyper::",
                r#"use rustls::{{Certificate, PrivateKey, ServerConfig}};
use tokio_rustls::TlsAcceptor;
use std::fs::File;
use std::io::BufReader;
use rustls_pemfile::{certs, rsa_private_keys};
use hyper::"#
            );
            
            // This is a simplified placeholder - real TLS configuration would be more complex
            code = code.replace(
                "let addr = \"{address}\".parse()?;",
                r#"let addr = "{address}".parse()?;
    
    // Configure TLS
    let tls_config = {
        // Load certificate and private key
        let cert_file = File::open("certificate.pem")?;
        let mut reader = BufReader::new(cert_file);
        let certs = certs(&mut reader)?;
        let certs = certs.into_iter().map(Certificate).collect();
        
        let key_file = File::open("private_key.pem")?;
        let mut reader = BufReader::new(key_file);
        let keys = rsa_private_keys(&mut reader)?;
        let key = PrivateKey(keys[0].clone());
        
        let mut config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;
            
        Arc::new(config)
    };"#
            );
        }
        
        // Add CORS configuration if enabled
        if let Some(cors_config) = &config.cors {
            let _allowed_origins = cors_config.allowed_origins
                .iter()
                .map(|o| format!("\"{}\"", o))
                .collect::<Vec<_>>()
                .join(", ");
                
            let _allowed_methods = cors_config.allowed_methods
                .iter()
                .map(|m| format!("\"{}\"", m))
                .collect::<Vec<_>>()
                .join(", ");
                
            let _allowed_headers = cors_config.allowed_headers
                .iter()
                .map(|h| format!("\"{}\"", h))
                .collect::<Vec<_>>()
                .join(", ");
                
            // Add CORS middleware
            code = code.replace(
                "/// Handle HTTP request",
                r#"/// Apply CORS headers
fn apply_cors(builder: &mut http::response::Builder) {
    builder
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .header("Access-Control-Allow-Credentials", "true")
        .header("Access-Control-Max-Age", "3600");
}

/// Handle HTTP request"#
            );
            
            // Use CORS in the handler
            code = code.replace(
                "// Add default content type if not present",
                r#"// Add CORS headers
    apply_cors(&mut builder);
    
    // Add default content type if not present"#
            );
        }
        
        Ok(code)
    }
}
