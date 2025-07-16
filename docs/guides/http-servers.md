# HTTP Servers

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Building secure, scalable HTTP services powered by WebAssembly sandboxes for safe request processing and dynamic business logic.

## Overview

HTTP servers with wasm-sandbox enable secure processing of user requests, plugin-based route handling, and isolated execution of untrusted code in web services.

## Basic HTTP Server

### Simple Request Handler

```rust
use axum::{
    extract::{Path, Query, Json},
    response::{Json as ResponseJson, Html},
    routing::{get, post},
    Router, Server,
};
use wasm_sandbox::{WasmSandbox, SecurityPolicy, Capability};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct ProcessRequest {
    data: serde_json::Value,
    options: HashMap<String, String>,
}

#[derive(Serialize)]
struct ProcessResponse {
    result: serde_json::Value,
    execution_time_ms: u64,
    memory_used: u64,
    status: String,
}

async fn process_data(Json(request): Json<ProcessRequest>) -> Result<ResponseJson<ProcessResponse>, String> {
    let start_time = std::time::Instant::now();
    
    // Create secure sandbox for request processing
    let sandbox = WasmSandbox::builder()
        .source("processors/data_processor.wasm")
        .security_policy(SecurityPolicy::strict())
        .memory_limit(64 * 1024 * 1024) // 64MB
        .cpu_timeout(std::time::Duration::from_secs(30))
        .build()
        .await
        .map_err(|e| format!("Failed to create sandbox: {}", e))?;

    // Process the request
    let result = sandbox
        .call("process_request", (&request.data, &request.options))
        .await
        .map_err(|e| format!("Processing failed: {}", e))?;

    let execution_time = start_time.elapsed();
    let memory_used = sandbox.memory_usage().await.unwrap_or(0);

    Ok(ResponseJson(ProcessResponse {
        result,
        execution_time_ms: execution_time.as_millis() as u64,
        memory_used,
        status: "success".to_string(),
    }))
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/process", post(process_data))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
```

### Route-Based Sandboxes

```rust
use axum::{extract::Path, response::Json, routing::any, Router};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RouteHandler {
    sandboxes: Arc<RwLock<HashMap<String, WasmSandbox>>>,
}

impl RouteHandler {
    pub async fn new() -> Self {
        Self {
            sandboxes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_route(&self, path: &str, wasm_module: &str) -> Result<(), String> {
        let sandbox = WasmSandbox::builder()
            .source(wasm_module)
            .security_policy(SecurityPolicy::web_service())
            .add_capability(Capability::NetworkAccess {
                allowed_hosts: vec!["api.internal.com".to_string()],
                allowed_ports: vec![443, 80],
            })
            .build()
            .await
            .map_err(|e| format!("Failed to create sandbox for route {}: {}", path, e))?;

        let mut sandboxes = self.sandboxes.write().await;
        sandboxes.insert(path.to_string(), sandbox);
        
        println!("Registered route: {} -> {}", path, wasm_module);
        Ok(())
    }

    pub async fn handle_request(
        &self,
        route: &str,
        method: &str,
        headers: &HeaderMap,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let sandboxes = self.sandboxes.read().await;
        let sandbox = sandboxes.get(route)
            .ok_or_else(|| format!("Route not found: {}", route))?;

        let request_data = HttpRequest {
            method: method.to_string(),
            path: route.to_string(),
            headers: headers.iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect(),
            body,
        };

        sandbox.call("handle_http_request", &request_data).await
            .map_err(|e| format!("Handler execution failed: {}", e))
    }
}

#[derive(Serialize, Deserialize)]
struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Option<serde_json::Value>,
}

// Dynamic route handler
async fn dynamic_route_handler(
    Path(route): Path<String>,
    method: axum::http::Method,
    headers: axum::http::HeaderMap,
    body: Option<Json<serde_json::Value>>,
) -> Result<Json<serde_json::Value>, String> {
    // Get global route handler (you'd inject this via app state)
    let handler = get_route_handler(); // Implementation depends on your architecture
    
    let result = handler.handle_request(
        &route,
        method.as_str(),
        &headers,
        body.map(|Json(b)| b),
    ).await?;

    Ok(Json(result))
}
```

## Advanced HTTP Features

### Middleware Integration

```rust
use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
};
use tower::{Layer, Service};

// Sandbox-based authentication middleware
async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Extract auth token
    let auth_header = request.headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    // Create sandbox for auth verification
    let auth_sandbox = WasmSandbox::builder()
        .source("auth/jwt_verifier.wasm")
        .security_policy(SecurityPolicy::strict())
        .build()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verify token
    let auth_result: AuthResult = auth_sandbox
        .call("verify_token", auth_header)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;

    if !auth_result.valid {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }

    // Add user info to request extensions
    let mut request = request;
    request.extensions_mut().insert(auth_result.user);

    Ok(next.run(request).await)
}

#[derive(Serialize, Deserialize)]
struct AuthResult {
    valid: bool,
    user: UserInfo,
    expires_at: i64,
}

#[derive(Serialize, Deserialize, Clone)]
struct UserInfo {
    id: String,
    email: String,
    roles: Vec<String>,
}

// Rate limiting middleware with sandbox
async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let client_ip = request.headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let rate_limiter = WasmSandbox::builder()
        .source("middleware/rate_limiter.wasm")
        .memory_limit(16 * 1024 * 1024) // 16MB
        .build()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let allowed: bool = rate_limiter
        .call("check_rate_limit", client_ip)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    if !allowed {
        return Err(axum::http::StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

// Apply middleware to routes
fn create_app() -> Router {
    Router::new()
        .route("/api/*path", any(dynamic_route_handler))
        .layer(middleware::from_fn(rate_limit_middleware))
        .route("/protected/*path", any(dynamic_route_handler))
        .layer(middleware::from_fn(auth_middleware))
}
```

### WebSocket Support

```rust
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};

async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, room_id))
}

async fn handle_websocket(socket: WebSocket, room_id: String) {
    let (mut sender, mut receiver) = socket.split();
    
    // Create sandbox for WebSocket message processing
    let ws_sandbox = match WasmSandbox::builder()
        .source("websocket/message_processor.wasm")
        .security_policy(SecurityPolicy::websocket())
        .build()
        .await
    {
        Ok(sandbox) => sandbox,
        Err(e) => {
            eprintln!("Failed to create WebSocket sandbox: {}", e);
            return;
        }
    };

    // Initialize room context
    if let Err(e) = ws_sandbox.call::<String, ()>("join_room", &room_id).await {
        eprintln!("Failed to join room: {}", e);
        return;
    }

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Process message through sandbox
                match ws_sandbox.call::<String, String>("process_message", &text).await {
                    Ok(response) => {
                        if !response.is_empty() {
                            if let Err(e) = sender.send(Message::Text(response)).await {
                                eprintln!("Failed to send response: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Message processing failed: {}", e);
                        let error_response = serde_json::json!({
                            "error": "Message processing failed",
                            "details": e.to_string()
                        });
                        if let Err(e) = sender.send(Message::Text(error_response.to_string())).await {
                            eprintln!("Failed to send error response: {}", e);
                            break;
                        }
                    }
                }
            }
            Ok(Message::Binary(data)) => {
                // Handle binary data
                match ws_sandbox.call::<Vec<u8>, Vec<u8>>("process_binary", &data).await {
                    Ok(response) => {
                        if let Err(e) = sender.send(Message::Binary(response)).await {
                            eprintln!("Failed to send binary response: {}", e);
                            break;
                        }
                    }
                    Err(e) => eprintln!("Binary processing failed: {}", e),
                }
            }
            Ok(Message::Close(_)) => {
                println!("WebSocket closed");
                break;
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    if let Err(e) = ws_sandbox.call::<(), ()>("leave_room", ()).await {
        eprintln!("Failed to leave room: {}", e);
    }
}

// Add WebSocket route to app
fn add_websocket_routes(app: Router) -> Router {
    app.route("/ws/:room_id", axum::routing::get(websocket_handler))
}
```

## Performance Optimization

### Connection Pooling

```rust
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock};

pub struct SandboxPool {
    sandboxes: Vec<Arc<WasmSandbox>>,
    semaphore: Arc<Semaphore>,
    metrics: Arc<RwLock<PoolMetrics>>,
}

#[derive(Default)]
struct PoolMetrics {
    total_requests: u64,
    active_connections: u64,
    average_response_time: f64,
    pool_utilization: f64,
}

impl SandboxPool {
    pub async fn new(module_path: &str, pool_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut sandboxes = Vec::new();
        
        for i in 0..pool_size {
            let sandbox = Arc::new(
                WasmSandbox::builder()
                    .source(module_path)
                    .instance_id(format!("pool-{}", i))
                    .build()
                    .await?
            );
            sandboxes.push(sandbox);
        }

        Ok(Self {
            sandboxes,
            semaphore: Arc::new(Semaphore::new(pool_size)),
            metrics: Arc::new(RwLock::new(PoolMetrics::default())),
        })
    }

    pub async fn execute<T, A>(&self, function: &str, args: A) -> Result<T, Box<dyn std::error::Error>>
    where
        T: serde::de::DeserializeOwned,
        A: serde::Serialize,
    {
        let start_time = std::time::Instant::now();
        
        // Acquire permit from semaphore
        let _permit = self.semaphore.acquire().await?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
            metrics.active_connections += 1;
        }

        // Get available sandbox (round-robin)
        let sandbox_index = fastrand::usize(..self.sandboxes.len());
        let sandbox = &self.sandboxes[sandbox_index];
        
        // Execute function
        let result = sandbox.call(function, args).await;
        
        // Update metrics
        let duration = start_time.elapsed();
        {
            let mut metrics = self.metrics.write().await;
            metrics.active_connections -= 1;
            
            // Update running average
            let alpha = 0.1; // Smoothing factor
            metrics.average_response_time = 
                alpha * duration.as_secs_f64() + (1.0 - alpha) * metrics.average_response_time;
        }

        result.map_err(|e| e.into())
    }

    pub async fn get_metrics(&self) -> PoolMetrics {
        let metrics = self.metrics.read().await;
        PoolMetrics {
            total_requests: metrics.total_requests,
            active_connections: metrics.active_connections,
            average_response_time: metrics.average_response_time,
            pool_utilization: metrics.active_connections as f64 / self.sandboxes.len() as f64,
        }
    }
}

// Use pool in HTTP handlers
async fn pooled_handler(
    Json(request): Json<ProcessRequest>,
    Extension(pool): Extension<Arc<SandboxPool>>,
) -> Result<ResponseJson<ProcessResponse>, String> {
    let result = pool.execute("process_request", &request).await
        .map_err(|e| format!("Pool execution failed: {}", e))?;

    Ok(ResponseJson(ProcessResponse {
        result,
        execution_time_ms: 0, // Pool handles timing
        memory_used: 0,       // Pool manages resources
        status: "success".to_string(),
    }))
}
```

### Caching Layer

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher, DefaultHasher};
use std::time::{Duration, Instant};

pub struct ResponseCache {
    cache: Arc<RwLock<HashMap<u64, CacheEntry>>>,
    ttl: Duration,
    max_size: usize,
}

struct CacheEntry {
    data: serde_json::Value,
    created_at: Instant,
    access_count: u64,
}

impl ResponseCache {
    pub fn new(ttl: Duration, max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            max_size,
        }
    }

    pub async fn get_or_compute<F, Fut>(&self, key: &str, compute: F) -> Result<serde_json::Value, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error>>>,
    {
        let cache_key = self.hash_key(key);
        
        // Try cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(entry) = cache.get_mut(&cache_key) {
                if entry.created_at.elapsed() < self.ttl {
                    entry.access_count += 1;
                    return Ok(entry.data.clone());
                } else {
                    // Remove expired entry
                    cache.remove(&cache_key);
                }
            }
        }

        // Compute new value
        let result = compute().await?;
        
        // Store in cache
        {
            let mut cache = self.cache.write().await;
            
            // Evict old entries if cache is full
            if cache.len() >= self.max_size {
                self.evict_lru(&mut cache);
            }
            
            cache.insert(cache_key, CacheEntry {
                data: result.clone(),
                created_at: Instant::now(),
                access_count: 1,
            });
        }

        Ok(result)
    }

    fn hash_key(&self, key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn evict_lru(&self, cache: &mut HashMap<u64, CacheEntry>) {
        if let Some((&lru_key, _)) = cache.iter()
            .min_by_key(|(_, entry)| entry.access_count) {
            cache.remove(&lru_key);
        }
    }
}

// Cached handler
async fn cached_handler(
    Json(request): Json<ProcessRequest>,
    Extension(cache): Extension<Arc<ResponseCache>>,
    Extension(pool): Extension<Arc<SandboxPool>>,
) -> Result<ResponseJson<serde_json::Value>, String> {
    let cache_key = format!("{:?}", request);
    
    let result = cache.get_or_compute(&cache_key, || async {
        pool.execute("process_request", &request).await
    }).await.map_err(|e| format!("Cached execution failed: {}", e))?;

    Ok(ResponseJson(result))
}
```

## Security Hardening

### Input Validation

```rust
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
struct ValidatedRequest {
    #[validate(length(min = 1, max = 1000))]
    data: String,
    
    #[validate(range(min = 1, max = 100))]
    priority: u32,
    
    #[validate(email)]
    user_email: String,
    
    #[validate(custom = "validate_json_structure")]
    payload: serde_json::Value,
}

fn validate_json_structure(value: &serde_json::Value) -> Result<(), ValidationError> {
    // Custom validation logic
    if value.is_object() && value.as_object().unwrap().len() <= 50 {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid JSON structure"))
    }
}

async fn validated_handler(
    Json(request): Json<ValidatedRequest>,
) -> Result<ResponseJson<ProcessResponse>, (axum::http::StatusCode, String)> {
    // Validate input
    if let Err(errors) = request.validate() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!("Validation failed: {:?}", errors)
        ));
    }

    // Create sandbox with strict security
    let sandbox = WasmSandbox::builder()
        .source("validators/strict_processor.wasm")
        .security_policy(SecurityPolicy::paranoid())
        .memory_limit(32 * 1024 * 1024)
        .cpu_timeout(Duration::from_secs(10))
        .network_isolation(true)
        .filesystem_isolation(true)
        .build()
        .await
        .map_err(|e| (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Sandbox creation failed: {}", e)
        ))?;

    // Process with audit
    let result = sandbox.call_with_audit("process_validated", &request).await
        .map_err(|e| (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Processing failed: {}", e)
        ))?;

    // Check for security violations
    if !result.audit_log.violations.is_empty() {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            "Security policy violation detected".to_string()
        ));
    }

    Ok(ResponseJson(ProcessResponse {
        result: result.output,
        execution_time_ms: result.execution_time.as_millis() as u64,
        memory_used: result.memory_used,
        status: "success".to_string(),
    }))
}
```

### Request Sanitization

```rust
async fn sanitization_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Create sanitization sandbox
    let sanitizer = WasmSandbox::builder()
        .source("security/input_sanitizer.wasm")
        .security_policy(SecurityPolicy::strict())
        .build()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract and sanitize body if present
    if let Some(content_type) = request.headers().get("content-type") {
        if content_type.to_str().unwrap_or("").contains("application/json") {
            let body = axum::body::to_bytes(request.into_body(), usize::MAX).await
                .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
            
            let body_str = String::from_utf8(body.to_vec())
                .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

            let sanitized: String = sanitizer.call("sanitize_json", &body_str).await
                .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

            let new_body = axum::body::Body::from(sanitized);
            request = Request::from_parts(request.into_parts().0, new_body);
        }
    }

    Ok(next.run(request).await)
}
```

## Example Applications

### REST API Server

```rust
// Complete REST API example
#[tokio::main]
async fn main() {
    let sandbox_pool = Arc::new(
        SandboxPool::new("api/rest_handler.wasm", 10).await.unwrap()
    );
    
    let cache = Arc::new(
        ResponseCache::new(Duration::from_secs(300), 1000)
    );

    let app = Router::new()
        .route("/api/v1/users", get(list_users).post(create_user))
        .route("/api/v1/users/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/health", get(health_check))
        .layer(Extension(sandbox_pool))
        .layer(Extension(cache))
        .layer(middleware::from_fn(sanitization_middleware))
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("REST API server running on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn list_users(
    Query(params): Query<HashMap<String, String>>,
    Extension(pool): Extension<Arc<SandboxPool>>,
) -> Result<Json<Vec<User>>, String> {
    let users = pool.execute("list_users", &params).await
        .map_err(|e| format!("Failed to list users: {}", e))?;
    Ok(Json(users))
}

async fn create_user(
    Json(user): Json<CreateUserRequest>,
    Extension(pool): Extension<Arc<SandboxPool>>,
) -> Result<(axum::http::StatusCode, Json<User>), String> {
    let created_user = pool.execute("create_user", &user).await
        .map_err(|e| format!("Failed to create user: {}", e))?;
    Ok((axum::http::StatusCode::CREATED, Json(created_user)))
}
```

Next: **[CLI Tools](cli-tools.md)** - Command-line applications with wasm-sandbox

---

**HTTP Server Excellence:** Secure, scalable web services with WebAssembly-powered request processing and comprehensive security features.
