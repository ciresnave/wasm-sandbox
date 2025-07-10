//! HTTP Server template constants

/// Template for HTTP server wrapper
pub const HTTP_SERVER_TEMPLATE: &str = r#"// Generated HTTP Server WASM wrapper
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;
use anyhow::{Result, anyhow, Context};

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::server::conn::AddrIncoming;
use http::{HeaderMap, HeaderValue, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::sync::oneshot;
use futures::channel::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use futures::StreamExt;

// Configuration
const APP_PATH: &str = "{{ app_path }}";
const PORT: u16 = {{ port }};
const HOST: &str = "{{ host }}";
const MAX_BODY_SIZE: usize = {{ max_body_size }};
const REQUEST_TIMEOUT: u64 = {{ request_timeout }};
const CORS_ENABLED: bool = {{ cors_enabled }};

// Global state
static SERVER: Lazy<Mutex<Option<ServerState>>> = Lazy::new(|| Mutex::new(None));
static RUNNING: AtomicBool = AtomicBool::new(false);

// Arguments
const APP_ARGS: &[&str] = &[
    {{ app_args }}
];

// Server state
struct ServerState {
    port: u16,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

// Request handler
struct RequestHandler {
    request_tx: UnboundedSender<HttpExchange>,
    response_rx_map: Arc<Mutex<HashMap<String, oneshot::Receiver<HttpResponse>>>>,
}

// HTTP exchange
struct HttpExchange {
    id: String,
    request: HttpRequest,
    response_tx: oneshot::Sender<HttpResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HttpRequest {
    method: String,
    path: String,
    query: Option<String>,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    remote: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl RequestHandler {
    fn new() -> (Self, UnboundedReceiver<HttpExchange>) {
        let (tx, rx) = unbounded();
        let handler = RequestHandler {
            request_tx: tx,
            response_rx_map: Arc::new(Mutex::new(HashMap::new())),
        };
        (handler, rx)
    }

    async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>> {
        // Generate a unique ID for this request
        let id = uuid::Uuid::new_v4().to_string();
        
        // Convert Hyper request to our HttpRequest
        let mut headers = HashMap::new();
        for (name, value) in req.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.to_string(), v.to_string());
            }
        }
        
        let method = req.method().to_string();
        let uri = req.uri();
        let path = uri.path().to_string();
        let query = uri.query().map(|q| q.to_string());
        
        let remote = req.extensions()
            .get::<hyper::server::conn::RemoteAddr>()
            .map(|addr| addr.to_string());
        
        // Read the body
        let body_bytes = hyper::body::to_bytes(req.into_body())
            .await
            .context("Failed to read request body")?;
            
        let body = if !body_bytes.is_empty() {
            Some(body_bytes.to_vec())
        } else {
            None
        };
        
        // Create HttpRequest
        let http_request = HttpRequest {
            method,
            path,
            query,
            headers,
            body,
            remote,
        };
        
        // Create a channel for the response
        let (response_tx, response_rx) = oneshot::channel();
        
        // Store the response receiver
        self.response_rx_map.lock().unwrap().insert(id.clone(), response_rx);
        
        // Send the request to the handler
        self.request_tx.unbounded_send(HttpExchange {
            id: id.clone(),
            request: http_request,
            response_tx,
        }).context("Failed to send request to handler")?;
        
        // Wait for the response
        let response = tokio::time::timeout(
            Duration::from_secs(REQUEST_TIMEOUT),
            async {
                let mut map = self.response_rx_map.lock().unwrap();
                if let Some(rx) = map.remove(&id) {
                    rx.await.context("Failed to receive response")
                } else {
                    Err(anyhow!("Response receiver not found"))
                }
            }
        ).await??;
        
        // Convert to Hyper response
        let mut builder = Response::builder()
            .status(StatusCode::from_u16(response.status)?);
            
        for (name, value) in response.headers {
            builder = builder.header(&name, &value);
        }
        
        if CORS_ENABLED {
            builder = builder
                .header("Access-Control-Allow-Origin", "{{ cors_allowed_origins }}")
                .header("Access-Control-Allow-Methods", "{{ cors_allowed_methods }}")
                .header("Access-Control-Allow-Headers", "{{ cors_allowed_headers }}");
        }
        
        let hyper_response = builder
            .body(Body::from(response.body))
            .context("Failed to build response")?;
            
        Ok(hyper_response)
    }
}

// Start the server
#[wasm_bindgen]
pub fn start(port: u16) -> u16 {
    console_error_panic_hook::set_once();
    
    // Check if already running
    if RUNNING.load(Ordering::SeqCst) {
        let server = SERVER.lock().unwrap();
        return server.as_ref().map_or(0, |s| s.port);
    }
    
    let actual_port = if port > 0 { port } else { PORT };
    
    // Create the runtime
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build() {
        Ok(rt) => rt,
        Err(_) => return 0,
    };
    
    // Create the request handler
    let (handler, mut rx) = RequestHandler::new();
    let handler = Arc::new(handler);
    
    // Create server
    let addr = format!("{}:{}", HOST, actual_port).parse::<SocketAddr>().unwrap();
    
    let handler_clone = handler.clone();
    let make_service = make_service_fn(move |_conn| {
        let handler = handler_clone.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let handler = handler.clone();
                async move {
                    match handler.handle_request(req).await {
                        Ok(resp) => Ok(resp),
                        Err(_) => {
                            let mut resp = Response::new(Body::from("Internal Server Error"));
                            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            Ok(resp)
                        }
                    }
                }
            }))
        }
    });
    
    // Start the server
    let server = Server::bind(&addr)
        .serve(make_service);
    
    let (tx, rx) = oneshot::channel::<()>();
    let graceful = server.with_graceful_shutdown(async {
        rx.await.ok();
    });
    
    // Spawn the server task
    runtime.spawn(async move {
        if let Err(e) = graceful.await {
            eprintln!("Server error: {}", e);
        }
        RUNNING.store(false, Ordering::SeqCst);
    });
    
    // Spawn the request handler task
    runtime.spawn(async move {
        while let Some(exchange) = rx.next().await {
            // Process the request and send response
            let response = handle_app_request(&exchange.request).await;
            let _ = exchange.response_tx.send(response);
        }
    });
    
    // Update server state
    let mut server_state = SERVER.lock().unwrap();
    *server_state = Some(ServerState {
        port: actual_port,
        shutdown_tx: Some(tx),
    });
    
    RUNNING.store(true, Ordering::SeqCst);
    actual_port
}

// Stop the server
#[wasm_bindgen]
pub fn stop() {
    let mut server = SERVER.lock().unwrap();
    if let Some(state) = server.as_mut() {
        if let Some(tx) = state.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
    
    RUNNING.store(false, Ordering::SeqCst);
}

// Check if server is running
#[wasm_bindgen]
pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

// Get server port
#[wasm_bindgen]
pub fn get_port() -> u16 {
    let server = SERVER.lock().unwrap();
    server.as_ref().map_or(0, |s| s.port)
}

// Handle requests by forwarding to the application
async fn handle_app_request(request: &HttpRequest) -> HttpResponse {
    // In a real implementation, we would communicate with the actual application process
    // For now, we return a simple response
    HttpResponse {
        status: 200,
        headers: HashMap::from([
            ("Content-Type".to_string(), "text/plain".to_string()),
        ]),
        body: format!("Request to {} handled by WASM sandbox", request.path).into_bytes(),
    }
}
"#;
