# Production Deployment Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This comprehensive guide covers deploying wasm-sandbox applications in production environments, including performance optimization, monitoring, security hardening, and operational best practices.

## Production Architecture Overview

A production wasm-sandbox deployment typically consists of:

1. **Application Layer** - Your Rust application using wasm-sandbox
2. **Runtime Layer** - WebAssembly runtime (Wasmtime/Wasmer)
3. **Security Layer** - Process isolation, resource limits, capability controls
4. **Monitoring Layer** - Metrics, logging, alerting, health checks
5. **Infrastructure Layer** - Containerization, orchestration, networking

## Quick Production Setup

```rust
use wasm_sandbox::{WasmSandbox, ProductionConfig};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize production logging
    tracing_subscriber::fmt()
        .with_env_filter("wasm_sandbox=info,my_app=info")
        .json()
        .init();

    // Load production configuration
    let config = ProductionConfig::from_env()?;
    
    // Create production-ready sandbox
    let sandbox = WasmSandbox::builder()
        .source(&config.wasm_module_path)
        .production_config(config)
        .enable_telemetry(true)
        .health_check_endpoint("/health")
        .metrics_endpoint("/metrics")
        .build()
        .await?;
    
    info!("Production sandbox initialized successfully");
    
    // Start your application server
    start_server(sandbox).await?;
    
    Ok(())
}
```

## Configuration Management

### Environment-Based Configuration

```rust
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize)]
pub struct ProductionConfig {
    // Resource limits
    pub memory_limit_mb: u64,
    pub timeout_seconds: u64,
    pub max_fuel: Option<u64>,
    
    // Security settings
    pub enable_network: bool,
    pub enable_filesystem: bool,
    pub allowed_hosts: Vec<String>,
    
    // Performance settings
    pub worker_threads: usize,
    pub max_concurrent_executions: usize,
    pub cache_size: usize,
    
    // Monitoring settings
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub log_level: String,
    
    // High availability
    pub health_check_interval: u64,
    pub graceful_shutdown_timeout: u64,
}

impl ProductionConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = Self {
            memory_limit_mb: env::var("WASM_MEMORY_LIMIT_MB")
                .unwrap_or_else(|_| "128".to_string())
                .parse()?,
            
            timeout_seconds: env::var("WASM_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            
            max_fuel: env::var("WASM_MAX_FUEL")
                .ok()
                .and_then(|s| s.parse().ok()),
            
            enable_network: env::var("WASM_ENABLE_NETWORK")
                .unwrap_or_else(|_| "false".to_string())
                .parse()?,
            
            enable_filesystem: env::var("WASM_ENABLE_FILESYSTEM")
                .unwrap_or_else(|_| "false".to_string())
                .parse()?,
            
            allowed_hosts: env::var("WASM_ALLOWED_HOSTS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            
            worker_threads: env::var("WASM_WORKER_THREADS")
                .unwrap_or_else(|_| num_cpus::get().to_string())
                .parse()?,
            
            max_concurrent_executions: env::var("WASM_MAX_CONCURRENT")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            
            cache_size: env::var("WASM_CACHE_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()?,
            
            enable_metrics: env::var("WASM_ENABLE_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            
            metrics_port: env::var("WASM_METRICS_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()?,
            
            log_level: env::var("WASM_LOG_LEVEL")
                .unwrap_or_else(|| "info".to_string()),
            
            health_check_interval: env::var("WASM_HEALTH_CHECK_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            
            graceful_shutdown_timeout: env::var("WASM_SHUTDOWN_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
        };
        
        Ok(config)
    }
}
```

### Configuration Validation

```rust
impl ProductionConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate memory limits
        if self.memory_limit_mb < 16 {
            return Err(ConfigError::InvalidMemoryLimit("Minimum 16MB required"));
        }
        
        if self.memory_limit_mb > 4096 {
            return Err(ConfigError::InvalidMemoryLimit("Maximum 4GB allowed"));
        }
        
        // Validate timeout settings
        if self.timeout_seconds < 1 {
            return Err(ConfigError::InvalidTimeout("Minimum 1 second required"));
        }
        
        if self.timeout_seconds > 3600 {
            return Err(ConfigError::InvalidTimeout("Maximum 1 hour allowed"));
        }
        
        // Validate concurrency settings
        if self.max_concurrent_executions > 10000 {
            return Err(ConfigError::InvalidConcurrency("Maximum 10000 concurrent executions"));
        }
        
        // Validate network settings
        if self.enable_network && self.allowed_hosts.is_empty() {
            return Err(ConfigError::InvalidNetworkConfig("Must specify allowed hosts when network is enabled"));
        }
        
        Ok(())
    }
}
```

## Performance Optimization

### Multi-Threading and Concurrency

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct ProductionSandboxPool {
    semaphore: Arc<Semaphore>,
    sandbox_factory: SandboxFactory,
    metrics: Arc<MetricsCollector>,
}

impl ProductionSandboxPool {
    pub fn new(max_concurrent: usize, factory: SandboxFactory) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            sandbox_factory: factory,
            metrics: Arc::new(MetricsCollector::new()),
        }
    }
    
    pub async fn execute<T, R>(&self, function: &str, input: &T) -> Result<R, SandboxError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        // Acquire semaphore permit
        let permit = self.semaphore.acquire().await?;
        
        // Track execution metrics
        let start = Instant::now();
        self.metrics.executions_started.inc();
        
        // Execute in sandbox
        let result = {
            let sandbox = self.sandbox_factory.create().await?;
            sandbox.call(function, input).await
        };
        
        // Record metrics
        let duration = start.elapsed();
        self.metrics.execution_duration.observe(duration.as_secs_f64());
        
        match &result {
            Ok(_) => self.metrics.executions_successful.inc(),
            Err(_) => self.metrics.executions_failed.inc(),
        }
        
        drop(permit); // Release semaphore
        result
    }
}
```

### Module Caching

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ModuleCache {
    cache: Arc<RwLock<HashMap<String, Arc<CompiledModule>>>>,
    max_size: usize,
    metrics: Arc<CacheMetrics>,
}

impl ModuleCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            metrics: Arc::new(CacheMetrics::new()),
        }
    }
    
    pub async fn get_or_compile(&self, source: &str) -> Result<Arc<CompiledModule>, Error> {
        let cache_key = format!("{:x}", md5::compute(source));
        
        // Try to get from cache first
        {
            let cache = self.cache.read().await;
            if let Some(module) = cache.get(&cache_key) {
                self.metrics.cache_hits.inc();
                return Ok(Arc::clone(module));
            }
        }
        
        self.metrics.cache_misses.inc();
        
        // Compile the module
        let module = Arc::new(compile_module(source).await?);
        
        // Store in cache with LRU eviction
        {
            let mut cache = self.cache.write().await;
            
            // Evict oldest entries if cache is full
            if cache.len() >= self.max_size {
                // Simple LRU: remove random entry (in production, use proper LRU)
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                    self.metrics.cache_evictions.inc();
                }
            }
            
            cache.insert(cache_key, Arc::clone(&module));
        }
        
        Ok(module)
    }
}
```

### Connection Pooling

```rust
use deadpool::managed::{Manager, Pool, PoolError};

pub struct SandboxManager {
    config: ProductionConfig,
}

#[async_trait::async_trait]
impl Manager for SandboxManager {
    type Type = WasmSandbox;
    type Error = SandboxError;
    
    async fn create(&self) -> Result<WasmSandbox, SandboxError> {
        WasmSandbox::builder()
            .memory_limit(self.config.memory_limit_mb * 1024 * 1024)
            .timeout_duration(Duration::from_secs(self.config.timeout_seconds))
            .max_fuel(self.config.max_fuel)
            .enable_network(self.config.enable_network)
            .enable_filesystem(self.config.enable_filesystem)
            .build()
            .await
    }
    
    async fn recycle(&self, sandbox: &mut WasmSandbox) -> Result<(), SandboxError> {
        // Reset sandbox state for reuse
        sandbox.reset().await?;
        Ok(())
    }
}

pub type SandboxPool = Pool<SandboxManager>;

pub fn create_sandbox_pool(config: &ProductionConfig) -> Result<SandboxPool, PoolError<SandboxError>> {
    let manager = SandboxManager {
        config: config.clone(),
    };
    
    Pool::builder(manager)
        .max_size(config.max_concurrent_executions)
        .build()
}
```

## Security Hardening

### Process Isolation

```rust
use std::process::Command;

pub struct IsolatedSandbox {
    sandbox: WasmSandbox,
    process_isolation: bool,
}

impl IsolatedSandbox {
    pub async fn new(config: &ProductionConfig) -> Result<Self, Error> {
        let sandbox = WasmSandbox::builder()
            // Enable all security features
            .strict_mode(true)
            .disable_deprecated_features(true)
            .enable_control_flow_guard(true)
            .enable_stack_protection(true)
            
            // Minimal capabilities
            .deny_all_capabilities()
            .allow_capability(Capability::Compute)
            
            // Network restrictions
            .network_policy(NetworkPolicy::DenyAll)
            .if_network_enabled(|builder| {
                builder
                    .allowed_hosts(&config.allowed_hosts)
                    .max_connections(10)
                    .timeout_duration(Duration::from_secs(30))
            })
            
            // Filesystem restrictions
            .filesystem_policy(FilesystemPolicy::ReadOnly)
            .temp_directory_size_limit(100 * 1024 * 1024) // 100MB temp
            
            // Resource limits
            .memory_limit(config.memory_limit_mb * 1024 * 1024)
            .max_fuel(config.max_fuel)
            .timeout_duration(Duration::from_secs(config.timeout_seconds))
            
            .build()
            .await?;
        
        Ok(Self {
            sandbox,
            process_isolation: true,
        })
    }
}
```

### Secret Management

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SecretManager {
    vault_client: VaultClient,
    encryption_key: Vec<u8>,
}

impl SecretManager {
    pub async fn new() -> Result<Self, Error> {
        let vault_client = VaultClient::new(
            env::var("VAULT_ADDR")?,
            env::var("VAULT_TOKEN")?,
        ).await?;
        
        let encryption_key = vault_client
            .get_secret("wasm-sandbox/encryption-key")
            .await?;
        
        Ok(Self {
            vault_client,
            encryption_key,
        })
    }
    
    pub async fn inject_secrets(&self, sandbox: &mut WasmSandbox) -> Result<(), Error> {
        // Inject secrets as environment variables
        let secrets = self.vault_client
            .get_secrets("wasm-sandbox/app-secrets")
            .await?;
        
        for (key, value) in secrets {
            let encrypted_value = self.encrypt(&value)?;
            sandbox.set_env_var(&key, &encrypted_value).await?;
        }
        
        Ok(())
    }
    
    fn encrypt(&self, data: &str) -> Result<String, Error> {
        // Implement encryption logic
        todo!("Implement proper encryption")
    }
}
```

## Monitoring and Observability

### Structured Logging

```rust
use tracing::{instrument, info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(config: &ProductionConfig) -> Result<(), Error> {
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(false)
        .with_span_list(true);
    
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level));
    
    tracing_subscriber::registry()
        .with(json_layer)
        .with(filter_layer)
        .init();
    
    Ok(())
}

#[instrument(skip(sandbox, input))]
pub async fn execute_with_logging<T, R>(
    sandbox: &WasmSandbox,
    function: &str,
    input: &T,
) -> Result<R, Error>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    info!(function = %function, "Starting function execution");
    
    let start = Instant::now();
    
    match sandbox.call(function, input).await {
        Ok(result) => {
            let duration = start.elapsed();
            info!(
                function = %function,
                duration_ms = duration.as_millis(),
                "Function execution completed successfully"
            );
            Ok(result)
        }
        Err(error) => {
            let duration = start.elapsed();
            error!(
                function = %function,
                duration_ms = duration.as_millis(),
                error = %error,
                "Function execution failed"
            );
            Err(error)
        }
    }
}
```

### Metrics Collection

```rust
use prometheus::{Counter, Histogram, Gauge, Registry, Opts};

#[derive(Clone)]
pub struct SandboxMetrics {
    pub executions_total: Counter,
    pub executions_successful: Counter,
    pub executions_failed: Counter,
    pub execution_duration: Histogram,
    pub memory_usage: Gauge,
    pub active_instances: Gauge,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
}

impl SandboxMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let executions_total = Counter::with_opts(
            Opts::new("wasm_executions_total", "Total number of function executions")
        )?;
        
        let executions_successful = Counter::with_opts(
            Opts::new("wasm_executions_successful_total", "Number of successful executions")
        )?;
        
        let executions_failed = Counter::with_opts(
            Opts::new("wasm_executions_failed_total", "Number of failed executions")
        )?;
        
        let execution_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "wasm_execution_duration_seconds",
                "Time spent executing WebAssembly functions"
            ).buckets(vec![0.001, 0.01, 0.1, 1.0, 10.0, 60.0])
        )?;
        
        let memory_usage = Gauge::with_opts(
            Opts::new("wasm_memory_usage_bytes", "Current memory usage in bytes")
        )?;
        
        let active_instances = Gauge::with_opts(
            Opts::new("wasm_active_instances", "Number of active WebAssembly instances")
        )?;
        
        let cache_hits = Counter::with_opts(
            Opts::new("wasm_cache_hits_total", "Number of module cache hits")
        )?;
        
        let cache_misses = Counter::with_opts(
            Opts::new("wasm_cache_misses_total", "Number of module cache misses")
        )?;
        
        Ok(Self {
            executions_total,
            executions_successful,
            executions_failed,
            execution_duration,
            memory_usage,
            active_instances,
            cache_hits,
            cache_misses,
        })
    }
    
    pub fn register(&self, registry: &Registry) -> Result<(), prometheus::Error> {
        registry.register(Box::new(self.executions_total.clone()))?;
        registry.register(Box::new(self.executions_successful.clone()))?;
        registry.register(Box::new(self.executions_failed.clone()))?;
        registry.register(Box::new(self.execution_duration.clone()))?;
        registry.register(Box::new(self.memory_usage.clone()))?;
        registry.register(Box::new(self.active_instances.clone()))?;
        registry.register(Box::new(self.cache_hits.clone()))?;
        registry.register(Box::new(self.cache_misses.clone()))?;
        Ok(())
    }
}
```

### Health Checks

```rust
use serde_json::json;

pub struct HealthChecker {
    sandbox_pool: Arc<SandboxPool>,
    metrics: Arc<SandboxMetrics>,
}

impl HealthChecker {
    pub async fn check(&self) -> HealthStatus {
        let mut checks = Vec::new();
        
        // Check sandbox pool health
        checks.push(self.check_sandbox_pool().await);
        
        // Check memory usage
        checks.push(self.check_memory_usage().await);
        
        // Check response time
        checks.push(self.check_response_time().await);
        
        // Check external dependencies
        checks.push(self.check_dependencies().await);
        
        let all_healthy = checks.iter().all(|check| check.status == "healthy");
        
        HealthStatus {
            status: if all_healthy { "healthy" } else { "unhealthy" },
            checks,
            timestamp: Utc::now(),
        }
    }
    
    async fn check_sandbox_pool(&self) -> HealthCheck {
        match self.sandbox_pool.get().await {
            Ok(_) => HealthCheck {
                name: "sandbox_pool".to_string(),
                status: "healthy".to_string(),
                message: "Sandbox pool is responsive".to_string(),
            },
            Err(e) => HealthCheck {
                name: "sandbox_pool".to_string(),
                status: "unhealthy".to_string(),
                message: format!("Sandbox pool error: {}", e),
            },
        }
    }
    
    async fn check_memory_usage(&self) -> HealthCheck {
        let usage = get_memory_usage();
        let threshold = 0.9; // 90% threshold
        
        if usage < threshold {
            HealthCheck {
                name: "memory".to_string(),
                status: "healthy".to_string(),
                message: format!("Memory usage: {:.1}%", usage * 100.0),
            }
        } else {
            HealthCheck {
                name: "memory".to_string(),
                status: "unhealthy".to_string(),
                message: format!("High memory usage: {:.1}%", usage * 100.0),
            }
        }
    }
    
    async fn check_response_time(&self) -> HealthCheck {
        let start = Instant::now();
        
        // Execute a simple test function
        let result = {
            let sandbox = self.sandbox_pool.get().await;
            match sandbox {
                Ok(sb) => sb.call("health_check", &()).await,
                Err(e) => Err(e.into()),
            }
        };
        
        let duration = start.elapsed();
        let threshold = Duration::from_millis(500); // 500ms threshold
        
        if duration < threshold && result.is_ok() {
            HealthCheck {
                name: "response_time".to_string(),
                status: "healthy".to_string(),
                message: format!("Response time: {}ms", duration.as_millis()),
            }
        } else {
            HealthCheck {
                name: "response_time".to_string(),
                status: "unhealthy".to_string(),
                message: format!("Slow response: {}ms", duration.as_millis()),
            }
        }
    }
}

#[derive(Serialize)]
pub struct HealthStatus {
    status: &'static str,
    checks: Vec<HealthCheck>,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct HealthCheck {
    name: String,
    status: String,
    message: String,
}
```

## Error Handling and Recovery

### Circuit Breaker Pattern

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

pub struct CircuitBreaker {
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: AtomicU64,
    state: AtomicBool, // true = open, false = closed
    failure_threshold: u64,
    recovery_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, recovery_timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: AtomicU64::new(0),
            state: AtomicBool::new(false),
            failure_threshold,
            recovery_timeout,
        }
    }
    
    pub async fn call<F, T>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: Future<Output = Result<T, Error>>,
    {
        // Check if circuit is open
        if self.is_open() {
            return Err(CircuitBreakerError::CircuitOpen);
        }
        
        match operation.await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                self.record_failure();
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }
    
    fn is_open(&self) -> bool {
        if !self.state.load(Ordering::Relaxed) {
            return false; // Circuit is closed
        }
        
        // Check if recovery timeout has passed
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let last_failure = self.last_failure_time.load(Ordering::Relaxed);
        
        if now - last_failure > self.recovery_timeout.as_secs() {
            // Try to close the circuit
            self.state.store(false, Ordering::Relaxed);
            false
        } else {
            true
        }
    }
    
    fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        // Reset failure count on success
        self.failure_count.store(0, Ordering::Relaxed);
    }
    
    fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.last_failure_time.store(now, Ordering::Relaxed);
        
        // Open circuit if failure threshold is reached
        if failures >= self.failure_threshold {
            self.state.store(true, Ordering::Relaxed);
        }
    }
}
```

### Graceful Shutdown

```rust
use tokio::signal;
use tokio::sync::broadcast;

pub struct GracefulShutdown {
    shutdown_tx: broadcast::Sender<()>,
    sandbox_pool: Arc<SandboxPool>,
    active_requests: Arc<AtomicU64>,
    timeout: Duration,
}

impl GracefulShutdown {
    pub fn new(sandbox_pool: Arc<SandboxPool>, timeout: Duration) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        
        Self {
            shutdown_tx,
            sandbox_pool,
            active_requests: Arc::new(AtomicU64::new(0)),
            timeout,
        }
    }
    
    pub async fn run(&self) {
        // Wait for shutdown signal
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        
        info!("Received shutdown signal, initiating graceful shutdown");
        
        // Send shutdown signal to all components
        let _ = self.shutdown_tx.send(());
        
        // Wait for active requests to complete
        let start = Instant::now();
        while self.active_requests.load(Ordering::Relaxed) > 0 {
            if start.elapsed() > self.timeout {
                warn!("Shutdown timeout reached, forcing exit");
                break;
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        info!("Graceful shutdown completed");
    }
    
    pub fn shutdown_receiver(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }
    
    pub fn track_request<F, T>(&self, future: F) -> impl Future<Output = T>
    where
        F: Future<Output = T>,
    {
        let active_requests = Arc::clone(&self.active_requests);
        
        async move {
            active_requests.fetch_add(1, Ordering::Relaxed);
            let result = future.await;
            active_requests.fetch_sub(1, Ordering::Relaxed);
            result
        }
    }
}
```

## Containerization

### Dockerfile

```dockerfile
# Multi-stage build for optimal size
FROM rust:1.75 as builder

WORKDIR /app

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -u 1000 -m wasmapp

# Copy binary
COPY --from=builder /app/target/release/wasm-sandbox-app /usr/local/bin/

# Set ownership and permissions
RUN chown wasmapp:wasmapp /usr/local/bin/wasm-sandbox-app
RUN chmod +x /usr/local/bin/wasm-sandbox-app

# Switch to non-root user
USER wasmapp

# Set working directory
WORKDIR /home/wasmapp

# Expose ports
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Start the application
CMD ["wasm-sandbox-app"]
```

### Docker Compose for Development

```yaml
version: '3.8'

services:
  wasm-sandbox:
    build: .
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - WASM_MEMORY_LIMIT_MB=256
      - WASM_TIMEOUT_SECONDS=60
      - WASM_ENABLE_NETWORK=true
      - WASM_ENABLE_METRICS=true
      - WASM_LOG_LEVEL=info
      - RUST_LOG=wasm_sandbox=info
    volumes:
      - ./modules:/app/modules:ro
    depends_on:
      - prometheus
      - grafana
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=200h'
      - '--web.enable-lifecycle'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./grafana/datasources:/etc/grafana/provisioning/datasources

volumes:
  prometheus_data:
  grafana_data:
```

## Kubernetes Deployment

### Deployment YAML

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wasm-sandbox
  labels:
    app: wasm-sandbox
spec:
  replicas: 3
  selector:
    matchLabels:
      app: wasm-sandbox
  template:
    metadata:
      labels:
        app: wasm-sandbox
    spec:
      containers:
      - name: wasm-sandbox
        image: wasm-sandbox:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: WASM_MEMORY_LIMIT_MB
          value: "256"
        - name: WASM_TIMEOUT_SECONDS
          value: "60"
        - name: WASM_MAX_CONCURRENT
          value: "100"
        - name: WASM_ENABLE_METRICS
          value: "true"
        - name: WASM_LOG_LEVEL
          value: "info"
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          capabilities:
            drop:
            - ALL
          readOnlyRootFilesystem: true
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: modules
          mountPath: /app/modules
          readOnly: true
      volumes:
      - name: tmp
        emptyDir: {}
      - name: modules
        configMap:
          name: wasm-modules
      securityContext:
        fsGroup: 1000
---
apiVersion: v1
kind: Service
metadata:
  name: wasm-sandbox-service
spec:
  selector:
    app: wasm-sandbox
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: wasm-sandbox-ingress
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - api.yourdomain.com
    secretName: wasm-sandbox-tls
  rules:
  - host: api.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: wasm-sandbox-service
            port:
              number: 80
```

## Load Testing

### Performance Testing

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;

fn benchmark_production_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let pool = rt.block_on(async {
        create_sandbox_pool(&ProductionConfig::from_env().unwrap()).unwrap()
    });
    
    let mut group = c.benchmark_group("production_load");
    
    for concurrent_users in [1, 10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_executions", concurrent_users),
            concurrent_users,
            |b, &concurrent_users| {
                b.to_async(&rt).iter(|| async {
                    let futures: Vec<_> = (0..concurrent_users)
                        .map(|_| {
                            let pool = pool.clone();
                            async move {
                                let sandbox = pool.get().await.unwrap();
                                sandbox.call("test_function", &()).await
                            }
                        })
                        .collect();
                    
                    futures::future::join_all(futures).await
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_production_load);
criterion_main!(benches);
```

### Load Testing Script

```bash
#!/bin/bash

# Load testing with Apache Bench
echo "Starting load tests..."

# Test 1: Low load
echo "Test 1: Low load (10 concurrent, 1000 requests)"
ab -n 1000 -c 10 -H "Content-Type: application/json" \
   -p test_payload.json \
   http://localhost:8080/api/execute

# Test 2: Medium load
echo "Test 2: Medium load (50 concurrent, 5000 requests)"
ab -n 5000 -c 50 -H "Content-Type: application/json" \
   -p test_payload.json \
   http://localhost:8080/api/execute

# Test 3: High load
echo "Test 3: High load (100 concurrent, 10000 requests)"
ab -n 10000 -c 100 -H "Content-Type: application/json" \
   -p test_payload.json \
   http://localhost:8080/api/execute

# Test 4: Stress test
echo "Test 4: Stress test (200 concurrent, 20000 requests)"
ab -n 20000 -c 200 -H "Content-Type: application/json" \
   -p test_payload.json \
   http://localhost:8080/api/execute

echo "Load tests completed"
```

## Disaster Recovery

### Backup Strategy

```rust
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct BackupManager {
    backup_dir: PathBuf,
    retention_days: u32,
}

impl BackupManager {
    pub fn new(backup_dir: impl AsRef<Path>, retention_days: u32) -> Self {
        Self {
            backup_dir: backup_dir.as_ref().to_path_buf(),
            retention_days,
        }
    }
    
    pub async fn backup_modules(&self) -> Result<(), Error> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = self.backup_dir.join(format!("modules_{}", timestamp));
        
        // Create backup directory
        fs::create_dir_all(&backup_path).await?;
        
        // Copy module files
        let mut entries = fs::read_dir("./modules").await?;
        while let Some(entry) = entries.next_entry().await? {
            let source = entry.path();
            let dest = backup_path.join(entry.file_name());
            fs::copy(&source, &dest).await?;
        }
        
        info!("Modules backed up to {:?}", backup_path);
        
        // Clean up old backups
        self.cleanup_old_backups().await?;
        
        Ok(())
    }
    
    async fn cleanup_old_backups(&self) -> Result<(), Error> {
        let cutoff = Utc::now() - chrono::Duration::days(self.retention_days as i64);
        
        let mut entries = fs::read_dir(&self.backup_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if let Ok(modified) = metadata.modified() {
                let modified: DateTime<Utc> = modified.into();
                if modified < cutoff {
                    if metadata.is_dir() {
                        fs::remove_dir_all(entry.path()).await?;
                    } else {
                        fs::remove_file(entry.path()).await?;
                    }
                    info!("Removed old backup: {:?}", entry.path());
                }
            }
        }
        
        Ok(())
    }
}
```

## Operational Best Practices

### 1. Resource Planning

```rust
// Plan for 3x peak load
let production_config = ProductionConfig {
    memory_limit_mb: 512,          // 3x typical usage
    max_concurrent_executions: 300, // 3x expected load
    timeout_seconds: 60,           // Conservative timeout
    cache_size: 5000,              // Large cache for performance
    ..Default::default()
};
```

### 2. Monitoring Alerts

```yaml
# Prometheus alerting rules
groups:
- name: wasm-sandbox
  rules:
  - alert: HighMemoryUsage
    expr: wasm_memory_usage_bytes / wasm_memory_limit_bytes > 0.8
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High memory usage detected"
      
  - alert: HighErrorRate
    expr: rate(wasm_executions_failed_total[5m]) > 0.1
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
      
  - alert: SlowResponseTime
    expr: histogram_quantile(0.95, wasm_execution_duration_seconds) > 5
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Slow response times detected"
```

### 3. Security Checklist

- [ ] Process isolation enabled
- [ ] Minimal capabilities granted
- [ ] Resource limits configured
- [ ] Network access restricted
- [ ] Filesystem access limited
- [ ] Secrets properly managed
- [ ] Audit logging enabled
- [ ] Regular security updates
- [ ] Vulnerability scanning
- [ ] Penetration testing

### 4. Performance Checklist

- [ ] Module caching enabled
- [ ] Connection pooling configured
- [ ] Resource limits optimized
- [ ] Load testing completed
- [ ] Monitoring dashboards created
- [ ] Alerting rules configured
- [ ] Circuit breaker implemented
- [ ] Graceful shutdown handling

## Next Steps

- **[Monitoring Guide](monitoring.md)** - Set up comprehensive monitoring
- **[Security Configuration](security-config.md)** - Harden security settings
- **[Resource Management](resource-management.md)** - Optimize resource usage
- **[Troubleshooting Guide](troubleshooting.md)** - Debug production issues

---

**Production Readiness:** This guide provides a comprehensive foundation for running wasm-sandbox in production. Adapt the configurations to your specific requirements and always test thoroughly before deployment.
