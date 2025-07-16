# Error Handling Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This guide covers comprehensive error handling patterns, best practices, and recovery strategies for wasm-sandbox applications, including custom error types, graceful degradation, and fault tolerance.

## Error Handling Philosophy

Effective error handling in wasm-sandbox follows these principles:

1. **Fail Fast** - Detect errors as early as possible
2. **Graceful Degradation** - Continue operation with reduced functionality when possible
3. **Clear Messaging** - Provide actionable error messages
4. **Recovery** - Attempt automatic recovery when safe
5. **Observability** - Log and monitor all error conditions

## Quick Start - Basic Error Handling

```rust
use wasm_sandbox::{WasmSandbox, Error, ErrorKind, Result};
use tracing::{error, warn, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize error reporting
    setup_error_reporting().await?;
    
    // Create sandbox with error handling
    let sandbox = WasmSandbox::builder()
        .source("my_program.rs")
        .error_handler(|error| {
            handle_sandbox_error(&error);
        })
        .build()
        .await?;
    
    // Execute with proper error handling
    match execute_with_retry(&sandbox, "process_data", &input, 3).await {
        Ok(result) => {
            info!("Execution successful: {:?}", result);
        }
        Err(e) => {
            error!("Execution failed after retries: {}", e);
            handle_critical_error(&e).await?;
        }
    }
    
    Ok(())
}

async fn execute_with_retry<T, R>(
    sandbox: &WasmSandbox,
    function: &str,
    input: &T,
    max_retries: usize,
) -> Result<R>
where
    T: serde::Serialize,
    R: for<'de> serde::Deserialize<'de>,
{
    let mut attempts = 0;
    let mut last_error = None;
    
    while attempts <= max_retries {
        match sandbox.call(function, input).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                last_error = Some(e);
                
                if attempts <= max_retries {
                    let delay = Duration::from_millis(100 * attempts as u64);
                    warn!("Attempt {} failed, retrying in {:?}: {}", attempts, delay, last_error.as_ref().unwrap());
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

## Error Types and Categories

### Core Error Types

```rust
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum WasmSandboxError {
    // Compilation errors
    #[error("Compilation failed: {message}")]
    CompilationError {
        message: String,
        source_location: Option<SourceLocation>,
        diagnostics: Vec<CompilationDiagnostic>,
    },
    
    // Runtime errors
    #[error("Runtime error: {message}")]
    RuntimeError {
        message: String,
        error_code: i32,
        function: Option<String>,
        stack_trace: Option<String>,
    },
    
    // Resource limit errors
    #[error("Resource limit exceeded: {resource} - used {used}, limit {limit}")]
    ResourceLimitExceeded {
        resource: ResourceType,
        used: u64,
        limit: u64,
        percentage: f64,
    },
    
    // Security violations
    #[error("Security violation: {violation_type} - {message}")]
    SecurityViolation {
        violation_type: SecurityViolationType,
        message: String,
        capability: Option<String>,
        resource: Option<String>,
    },
    
    // I/O errors
    #[error("I/O error: {operation} - {message}")]
    IoError {
        operation: IoOperation,
        message: String,
        path: Option<String>,
        error_code: Option<i32>,
    },
    
    // Network errors
    #[error("Network error: {operation} - {message}")]
    NetworkError {
        operation: NetworkOperation,
        message: String,
        host: Option<String>,
        status_code: Option<u16>,
    },
    
    // Configuration errors
    #[error("Configuration error: {field} - {message}")]
    ConfigurationError {
        field: String,
        message: String,
        suggested_fix: Option<String>,
    },
    
    // Module errors
    #[error("Module error: {message}")]
    ModuleError {
        message: String,
        module_name: Option<String>,
        validation_errors: Vec<String>,
    },
    
    // Instance errors
    #[error("Instance error: {message}")]
    InstanceError {
        message: String,
        instance_id: Option<String>,
        state: Option<InstanceState>,
    },
    
    // Communication errors
    #[error("Communication error: {channel} - {message}")]
    CommunicationError {
        channel: String,
        message: String,
        direction: CommunicationDirection,
    },
    
    // Timeout errors
    #[error("Timeout after {duration:?}: {operation}")]
    TimeoutError {
        operation: String,
        duration: Duration,
        partial_result: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Help,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Memory,
    Cpu,
    Fuel,
    FileSystem,
    Network,
    Time,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityViolationType {
    CapabilityDenied,
    ResourceAccess,
    SystemCall,
    MemoryViolation,
    NetworkAccess,
}
```

### Error Context and Chain

```rust
use std::fmt;

#[derive(Debug)]
pub struct ErrorContext {
    pub execution_id: String,
    pub sandbox_id: String,
    pub function: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub environment: std::collections::HashMap<String, String>,
    pub stack_trace: Option<String>,
    pub related_errors: Vec<WasmSandboxError>,
}

impl ErrorContext {
    pub fn new(execution_id: String, sandbox_id: String) -> Self {
        Self {
            execution_id,
            sandbox_id,
            function: None,
            timestamp: chrono::Utc::now(),
            environment: std::collections::HashMap::new(),
            stack_trace: None,
            related_errors: Vec::new(),
        }
    }
    
    pub fn with_function(mut self, function: String) -> Self {
        self.function = Some(function);
        self
    }
    
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }
    
    pub fn with_stack_trace(mut self, stack_trace: String) -> Self {
        self.stack_trace = Some(stack_trace);
        self
    }
    
    pub fn add_related_error(mut self, error: WasmSandboxError) -> Self {
        self.related_errors.push(error);
        self
    }
}

#[derive(Debug)]
pub struct ContextualError {
    pub error: WasmSandboxError,
    pub context: ErrorContext,
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (execution: {}, sandbox: {})", 
               self.error, 
               self.context.execution_id, 
               self.context.sandbox_id)
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
```

## Error Recovery Strategies

### Automatic Recovery

```rust
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait RecoveryStrategy: Send + Sync {
    async fn can_recover(&self, error: &WasmSandboxError) -> bool;
    async fn recover(&self, error: &WasmSandboxError, context: &ErrorContext) -> Result<RecoveryResult, RecoveryError>;
    fn strategy_name(&self) -> &str;
    fn max_attempts(&self) -> usize;
}

#[derive(Debug, Clone)]
pub enum RecoveryResult {
    Retry,
    RetryWithDelay(Duration),
    RetryWithModification(SandboxConfig),
    Fallback(serde_json::Value),
    Abort,
}

pub struct ErrorRecoveryManager {
    strategies: Vec<Arc<dyn RecoveryStrategy>>,
    max_recovery_attempts: usize,
    recovery_timeout: Duration,
}

impl ErrorRecoveryManager {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                Arc::new(MemoryRecoveryStrategy::new()),
                Arc::new(TimeoutRecoveryStrategy::new()),
                Arc::new(CompilationRecoveryStrategy::new()),
                Arc::new(NetworkRecoveryStrategy::new()),
                Arc::new(SecurityRecoveryStrategy::new()),
            ],
            max_recovery_attempts: 3,
            recovery_timeout: Duration::from_secs(30),
        }
    }
    
    pub async fn attempt_recovery(
        &self,
        error: &WasmSandboxError,
        context: &ErrorContext,
    ) -> Result<RecoveryResult, RecoveryError> {
        for strategy in &self.strategies {
            if strategy.can_recover(error).await {
                info!(
                    strategy = strategy.strategy_name(),
                    execution_id = %context.execution_id,
                    "Attempting recovery"
                );
                
                match tokio::time::timeout(
                    self.recovery_timeout,
                    strategy.recover(error, context)
                ).await {
                    Ok(Ok(result)) => {
                        info!(
                            strategy = strategy.strategy_name(),
                            execution_id = %context.execution_id,
                            "Recovery successful"
                        );
                        return Ok(result);
                    }
                    Ok(Err(e)) => {
                        warn!(
                            strategy = strategy.strategy_name(),
                            execution_id = %context.execution_id,
                            error = %e,
                            "Recovery strategy failed"
                        );
                    }
                    Err(_) => {
                        warn!(
                            strategy = strategy.strategy_name(),
                            execution_id = %context.execution_id,
                            "Recovery strategy timed out"
                        );
                    }
                }
            }
        }
        
        Err(RecoveryError::NoStrategyAvailable)
    }
}

// Memory recovery strategy
pub struct MemoryRecoveryStrategy {
    gc_threshold: f64,
    scale_factor: f64,
}

impl MemoryRecoveryStrategy {
    pub fn new() -> Self {
        Self {
            gc_threshold: 0.8, // Trigger GC at 80% memory usage
            scale_factor: 1.5,  // Increase memory limit by 50%
        }
    }
}

#[async_trait]
impl RecoveryStrategy for MemoryRecoveryStrategy {
    async fn can_recover(&self, error: &WasmSandboxError) -> bool {
        matches!(error, WasmSandboxError::ResourceLimitExceeded { 
            resource: ResourceType::Memory, .. 
        })
    }
    
    async fn recover(&self, error: &WasmSandboxError, context: &ErrorContext) -> Result<RecoveryResult, RecoveryError> {
        if let WasmSandboxError::ResourceLimitExceeded { used, limit, .. } = error {
            // Calculate new memory limit
            let new_limit = (*limit as f64 * self.scale_factor) as u64;
            
            // Check if we should trigger garbage collection first
            let usage_percentage = *used as f64 / *limit as f64;
            if usage_percentage < self.gc_threshold {
                // Try garbage collection first
                return Ok(RecoveryResult::Retry);
            }
            
            // Increase memory limit
            let mut new_config = SandboxConfig::default();
            new_config.memory_limit = Some(new_limit);
            
            info!(
                old_limit = limit,
                new_limit = new_limit,
                "Increasing memory limit for recovery"
            );
            
            Ok(RecoveryResult::RetryWithModification(new_config))
        } else {
            Err(RecoveryError::IncompatibleError)
        }
    }
    
    fn strategy_name(&self) -> &str {
        "memory_recovery"
    }
    
    fn max_attempts(&self) -> usize {
        2
    }
}

// Timeout recovery strategy
pub struct TimeoutRecoveryStrategy {
    max_timeout_extension: Duration,
    extension_factor: f64,
}

#[async_trait]
impl RecoveryStrategy for TimeoutRecoveryStrategy {
    async fn can_recover(&self, error: &WasmSandboxError) -> bool {
        matches!(error, WasmSandboxError::TimeoutError { .. })
    }
    
    async fn recover(&self, error: &WasmSandboxError, _context: &ErrorContext) -> Result<RecoveryResult, RecoveryError> {
        if let WasmSandboxError::TimeoutError { duration, .. } = error {
            let new_duration = Duration::from_secs(
                (duration.as_secs() as f64 * self.extension_factor) as u64
            );
            
            if new_duration <= self.max_timeout_extension {
                let mut new_config = SandboxConfig::default();
                new_config.timeout_duration = Some(new_duration);
                
                Ok(RecoveryResult::RetryWithModification(new_config))
            } else {
                Err(RecoveryError::RecoveryLimitExceeded)
            }
        } else {
            Err(RecoveryError::IncompatibleError)
        }
    }
    
    fn strategy_name(&self) -> &str {
        "timeout_recovery"
    }
    
    fn max_attempts(&self) -> usize {
        2
    }
}
```

### Circuit Breaker Pattern

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: AtomicU64,
    state: Arc<std::sync::RwLock<CircuitState>>,
    failure_threshold: u64,
    recovery_timeout: Duration,
    success_threshold: u64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, recovery_timeout: Duration, success_threshold: u64) -> Self {
        Self {
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: AtomicU64::new(0),
            state: Arc::new(std::sync::RwLock::new(CircuitState::Closed)),
            failure_threshold,
            recovery_timeout,
            success_threshold,
        }
    }
    
    pub async fn call<F, T>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: std::future::Future<Output = Result<T, WasmSandboxError>>,
    {
        // Check circuit state
        match self.get_state() {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    self.set_state(CircuitState::HalfOpen);
                } else {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }
        
        // Execute operation
        let start_time = std::time::Instant::now();
        match operation.await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                let execution_time = start_time.elapsed();
                self.record_failure(&error, execution_time);
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }
    
    fn record_success(&self) {
        let current_state = self.get_state();
        let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        match current_state {
            CircuitState::HalfOpen => {
                if success_count >= self.success_threshold {
                    self.set_state(CircuitState::Closed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            _ => {}
        }
    }
    
    fn record_failure(&self, error: &WasmSandboxError, execution_time: Duration) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.last_failure_time.store(now, Ordering::Relaxed);
        
        // Determine if this failure should trigger circuit opening
        let should_open = match error {
            WasmSandboxError::ResourceLimitExceeded { .. } => true,
            WasmSandboxError::SecurityViolation { .. } => true,
            WasmSandboxError::TimeoutError { .. } => execution_time > Duration::from_secs(10),
            _ => false,
        };
        
        if should_open && failure_count >= self.failure_threshold {
            self.set_state(CircuitState::Open);
            warn!(
                failure_count = failure_count,
                error = %error,
                "Circuit breaker opened due to failures"
            );
        }
    }
    
    fn should_attempt_reset(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let last_failure = self.last_failure_time.load(Ordering::Relaxed);
        now - last_failure > self.recovery_timeout.as_secs()
    }
    
    fn get_state(&self) -> CircuitState {
        self.state.read().unwrap().clone()
    }
    
    fn set_state(&self, new_state: CircuitState) {
        *self.state.write().unwrap() = new_state;
    }
}

#[derive(Error, Debug)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(WasmSandboxError),
}
```

## Graceful Degradation

### Fallback Mechanisms

```rust
pub struct FallbackHandler {
    fallback_strategies: Vec<Box<dyn FallbackStrategy>>,
    enabled: bool,
}

#[async_trait]
pub trait FallbackStrategy: Send + Sync {
    async fn can_handle(&self, error: &WasmSandboxError) -> bool;
    async fn execute_fallback(&self, input: &serde_json::Value) -> Result<serde_json::Value, FallbackError>;
    fn strategy_name(&self) -> &str;
    fn quality_score(&self) -> f64; // 0.0 to 1.0, where 1.0 is equivalent to normal operation
}

impl FallbackHandler {
    pub fn new() -> Self {
        Self {
            fallback_strategies: vec![
                Box::new(CachedResultFallback::new()),
                Box::new(SimplifiedProcessingFallback::new()),
                Box::new(DefaultValueFallback::new()),
                Box::new(ExternalServiceFallback::new()),
            ],
            enabled: true,
        }
    }
    
    pub async fn handle_with_fallback<T, R>(
        &self,
        sandbox: &WasmSandbox,
        function: &str,
        input: &T,
    ) -> Result<FallbackResult<R>, WasmSandboxError>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        // Try normal execution first
        match sandbox.call(function, input).await {
            Ok(result) => Ok(FallbackResult::Normal(result)),
            Err(error) => {
                if !self.enabled {
                    return Err(error);
                }
                
                // Convert input to JSON for fallback strategies
                let json_input = serde_json::to_value(input)
                    .map_err(|e| WasmSandboxError::ConfigurationError {
                        field: "input_serialization".to_string(),
                        message: e.to_string(),
                        suggested_fix: None,
                    })?;
                
                // Try fallback strategies
                for strategy in &self.fallback_strategies {
                    if strategy.can_handle(&error).await {
                        match strategy.execute_fallback(&json_input).await {
                            Ok(fallback_result) => {
                                let typed_result: R = serde_json::from_value(fallback_result)
                                    .map_err(|e| WasmSandboxError::ConfigurationError {
                                        field: "output_deserialization".to_string(),
                                        message: e.to_string(),
                                        suggested_fix: None,
                                    })?;
                                
                                warn!(
                                    strategy = strategy.strategy_name(),
                                    quality = strategy.quality_score(),
                                    original_error = %error,
                                    "Using fallback strategy"
                                );
                                
                                return Ok(FallbackResult::Fallback {
                                    result: typed_result,
                                    strategy: strategy.strategy_name().to_string(),
                                    quality: strategy.quality_score(),
                                    original_error: error,
                                });
                            }
                            Err(fallback_error) => {
                                debug!(
                                    strategy = strategy.strategy_name(),
                                    error = %fallback_error,
                                    "Fallback strategy failed"
                                );
                            }
                        }
                    }
                }
                
                // No fallback strategy worked
                Err(error)
            }
        }
    }
}

#[derive(Debug)]
pub enum FallbackResult<T> {
    Normal(T),
    Fallback {
        result: T,
        strategy: String,
        quality: f64,
        original_error: WasmSandboxError,
    },
}

// Cached result fallback strategy
pub struct CachedResultFallback {
    cache: Arc<std::sync::RwLock<HashMap<String, (serde_json::Value, std::time::Instant)>>>,
    cache_ttl: Duration,
}

impl CachedResultFallback {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }
    
    pub fn store_result(&self, input: &serde_json::Value, output: &serde_json::Value) {
        let key = format!("{:x}", md5::compute(serde_json::to_string(input).unwrap()));
        let mut cache = self.cache.write().unwrap();
        cache.insert(key, (output.clone(), std::time::Instant::now()));
    }
}

#[async_trait]
impl FallbackStrategy for CachedResultFallback {
    async fn can_handle(&self, _error: &WasmSandboxError) -> bool {
        true // Can handle any error if cached result is available
    }
    
    async fn execute_fallback(&self, input: &serde_json::Value) -> Result<serde_json::Value, FallbackError> {
        let key = format!("{:x}", md5::compute(serde_json::to_string(input).unwrap()));
        let cache = self.cache.read().unwrap();
        
        if let Some((cached_result, cached_time)) = cache.get(&key) {
            if cached_time.elapsed() < self.cache_ttl {
                return Ok(cached_result.clone());
            }
        }
        
        Err(FallbackError::NoFallbackAvailable)
    }
    
    fn strategy_name(&self) -> &str {
        "cached_result"
    }
    
    fn quality_score(&self) -> f64 {
        0.9 // High quality as it's the same computation
    }
}
```

## Error Reporting and Monitoring

### Structured Error Reporting

```rust
use serde_json::json;

pub struct ErrorReporter {
    sentry_client: Option<sentry::ClientInitGuard>,
    metrics_collector: Arc<MetricsCollector>,
    logger: StructuredLogger,
}

impl ErrorReporter {
    pub fn new() -> Self {
        let sentry_client = if let Ok(dsn) = std::env::var("SENTRY_DSN") {
            Some(sentry::init(sentry::ClientOptions {
                dsn: Some(dsn.parse().unwrap()),
                ..Default::default()
            }))
        } else {
            None
        };
        
        Self {
            sentry_client,
            metrics_collector: Arc::new(MetricsCollector::new()),
            logger: StructuredLogger::new(),
        }
    }
    
    pub async fn report_error(&self, error: &WasmSandboxError, context: &ErrorContext) {
        // Update metrics
        self.update_error_metrics(error).await;
        
        // Log structured error
        self.log_error(error, context).await;
        
        // Report to external services
        if let Some(_) = &self.sentry_client {
            self.report_to_sentry(error, context).await;
        }
        
        // Trigger alerts if necessary
        self.check_alert_conditions(error, context).await;
    }
    
    async fn update_error_metrics(&self, error: &WasmSandboxError) {
        self.metrics_collector.error_count_total.inc();
        
        match error {
            WasmSandboxError::CompilationError { .. } => {
                self.metrics_collector.compilation_errors.inc();
            }
            WasmSandboxError::RuntimeError { .. } => {
                self.metrics_collector.runtime_errors.inc();
            }
            WasmSandboxError::ResourceLimitExceeded { resource, .. } => {
                self.metrics_collector.resource_limit_errors.inc();
                match resource {
                    ResourceType::Memory => self.metrics_collector.memory_limit_errors.inc(),
                    ResourceType::Cpu => self.metrics_collector.cpu_limit_errors.inc(),
                    _ => {}
                }
            }
            WasmSandboxError::SecurityViolation { .. } => {
                self.metrics_collector.security_violations.inc();
            }
            WasmSandboxError::TimeoutError { .. } => {
                self.metrics_collector.timeout_errors.inc();
            }
            _ => {}
        }
    }
    
    async fn log_error(&self, error: &WasmSandboxError, context: &ErrorContext) {
        let error_log = json!({
            "timestamp": context.timestamp.to_rfc3339(),
            "level": "error",
            "event_type": "wasm_sandbox_error",
            "execution_id": context.execution_id,
            "sandbox_id": context.sandbox_id,
            "function": context.function,
            "error": {
                "type": error_type_name(error),
                "message": error.to_string(),
                "details": serialize_error_details(error),
            },
            "context": {
                "environment": context.environment,
                "stack_trace": context.stack_trace,
                "related_errors": context.related_errors.len(),
            }
        });
        
        error!("{}", error_log);
    }
    
    async fn report_to_sentry(&self, error: &WasmSandboxError, context: &ErrorContext) {
        sentry::configure_scope(|scope| {
            scope.set_tag("execution_id", &context.execution_id);
            scope.set_tag("sandbox_id", &context.sandbox_id);
            scope.set_tag("error_type", error_type_name(error));
            
            if let Some(function) = &context.function {
                scope.set_tag("function", function);
            }
            
            for (key, value) in &context.environment {
                scope.set_extra(key, value.clone().into());
            }
        });
        
        sentry::capture_error(error);
    }
    
    async fn check_alert_conditions(&self, error: &WasmSandboxError, context: &ErrorContext) {
        // Check if this error type is occurring frequently
        let error_rate = self.calculate_error_rate(error_type_name(error)).await;
        
        if error_rate > 0.1 { // 10% error rate threshold
            self.trigger_alert(AlertSeverity::High, format!(
                "High error rate detected: {} - {:.1}%",
                error_type_name(error),
                error_rate * 100.0
            )).await;
        }
        
        // Check for critical errors
        match error {
            WasmSandboxError::SecurityViolation { .. } => {
                self.trigger_alert(AlertSeverity::Critical, format!(
                    "Security violation: {}",
                    error
                )).await;
            }
            WasmSandboxError::ResourceLimitExceeded { resource: ResourceType::Memory, .. } => {
                self.trigger_alert(AlertSeverity::High, format!(
                    "Memory limit exceeded: {}",
                    error
                )).await;
            }
            _ => {}
        }
    }
}

fn error_type_name(error: &WasmSandboxError) -> &'static str {
    match error {
        WasmSandboxError::CompilationError { .. } => "compilation_error",
        WasmSandboxError::RuntimeError { .. } => "runtime_error",
        WasmSandboxError::ResourceLimitExceeded { .. } => "resource_limit_exceeded",
        WasmSandboxError::SecurityViolation { .. } => "security_violation",
        WasmSandboxError::IoError { .. } => "io_error",
        WasmSandboxError::NetworkError { .. } => "network_error",
        WasmSandboxError::ConfigurationError { .. } => "configuration_error",
        WasmSandboxError::ModuleError { .. } => "module_error",
        WasmSandboxError::InstanceError { .. } => "instance_error",
        WasmSandboxError::CommunicationError { .. } => "communication_error",
        WasmSandboxError::TimeoutError { .. } => "timeout_error",
    }
}
```

## Error Testing

### Error Injection for Testing

```rust
#[cfg(test)]
mod error_tests {
    use super::*;
    
    pub struct ErrorInjector {
        error_scenarios: HashMap<String, WasmSandboxError>,
        injection_rate: f64,
    }
    
    impl ErrorInjector {
        pub fn new() -> Self {
            Self {
                error_scenarios: HashMap::new(),
                injection_rate: 0.0,
            }
        }
        
        pub fn add_scenario(&mut self, name: String, error: WasmSandboxError) {
            self.error_scenarios.insert(name, error);
        }
        
        pub fn set_injection_rate(&mut self, rate: f64) {
            self.injection_rate = rate.clamp(0.0, 1.0);
        }
        
        pub fn should_inject_error(&self) -> bool {
            rand::random::<f64>() < self.injection_rate
        }
        
        pub fn get_random_error(&self) -> Option<&WasmSandboxError> {
            if self.error_scenarios.is_empty() {
                return None;
            }
            
            let keys: Vec<_> = self.error_scenarios.keys().collect();
            let random_key = keys[rand::random::<usize>() % keys.len()];
            self.error_scenarios.get(random_key)
        }
    }
    
    #[tokio::test]
    async fn test_memory_error_recovery() {
        let mut injector = ErrorInjector::new();
        injector.add_scenario(
            "memory_limit".to_string(),
            WasmSandboxError::ResourceLimitExceeded {
                resource: ResourceType::Memory,
                used: 128 * 1024 * 1024,
                limit: 64 * 1024 * 1024,
                percentage: 200.0,
            }
        );
        
        let recovery_manager = ErrorRecoveryManager::new();
        let context = ErrorContext::new("test_exec".to_string(), "test_sandbox".to_string());
        
        if let Some(error) = injector.get_random_error() {
            let result = recovery_manager.attempt_recovery(error, &context).await;
            assert!(result.is_ok());
            
            match result.unwrap() {
                RecoveryResult::RetryWithModification(config) => {
                    assert!(config.memory_limit.unwrap() > 64 * 1024 * 1024);
                }
                _ => panic!("Expected memory limit increase"),
            }
        }
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let circuit_breaker = CircuitBreaker::new(3, Duration::from_secs(60), 2);
        
        // Simulate failures
        for _ in 0..3 {
            let result = circuit_breaker.call(async {
                Err(WasmSandboxError::RuntimeError {
                    message: "Test error".to_string(),
                    error_code: -1,
                    function: None,
                    stack_trace: None,
                })
            }).await;
            
            assert!(matches!(result, Err(CircuitBreakerError::OperationFailed(_))));
        }
        
        // Circuit should be open now
        let result = circuit_breaker.call(async { Ok(()) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen)));
    }
    
    #[tokio::test]
    async fn test_fallback_mechanisms() {
        let fallback_handler = FallbackHandler::new();
        
        // Create a mock sandbox that always fails
        let failing_sandbox = MockSandbox::new()
            .with_error(WasmSandboxError::TimeoutError {
                operation: "test_function".to_string(),
                duration: Duration::from_secs(30),
                partial_result: None,
            });
        
        let input = json!({"test": "data"});
        let result = fallback_handler
            .handle_with_fallback(&failing_sandbox, "test_function", &input)
            .await;
        
        match result {
            Ok(FallbackResult::Fallback { strategy, quality, .. }) => {
                assert!(!strategy.is_empty());
                assert!(quality > 0.0 && quality <= 1.0);
            }
            _ => panic!("Expected fallback result"),
        }
    }
}
```

## Best Practices

### 1. Error Classification

```rust
// Classify errors by severity and recovery potential
pub enum ErrorSeverity {
    Low,      // Warning level, can be ignored temporarily
    Medium,   // Needs attention but doesn't block operation
    High,     // Blocks operation, needs immediate attention
    Critical, // System integrity at risk, needs immediate action
}

pub enum RecoveryPotential {
    Recoverable,    // Can be automatically recovered
    UserAction,     // Requires user intervention
    SystemAction,   // Requires system administrator action
    NonRecoverable, // Cannot be recovered, needs restart/replacement
}

impl WasmSandboxError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            WasmSandboxError::SecurityViolation { .. } => ErrorSeverity::Critical,
            WasmSandboxError::ResourceLimitExceeded { .. } => ErrorSeverity::High,
            WasmSandboxError::CompilationError { .. } => ErrorSeverity::Medium,
            WasmSandboxError::ConfigurationError { .. } => ErrorSeverity::Medium,
            WasmSandboxError::TimeoutError { .. } => ErrorSeverity::Medium,
            WasmSandboxError::RuntimeError { .. } => ErrorSeverity::High,
            _ => ErrorSeverity::Low,
        }
    }
    
    pub fn recovery_potential(&self) -> RecoveryPotential {
        match self {
            WasmSandboxError::ResourceLimitExceeded { .. } => RecoveryPotential::Recoverable,
            WasmSandboxError::TimeoutError { .. } => RecoveryPotential::Recoverable,
            WasmSandboxError::NetworkError { .. } => RecoveryPotential::Recoverable,
            WasmSandboxError::SecurityViolation { .. } => RecoveryPotential::SystemAction,
            WasmSandboxError::ConfigurationError { .. } => RecoveryPotential::UserAction,
            WasmSandboxError::CompilationError { .. } => RecoveryPotential::UserAction,
            _ => RecoveryPotential::NonRecoverable,
        }
    }
}
```

### 2. Progressive Error Handling

```rust
// Handle errors with increasing levels of intervention
pub async fn progressive_error_handling<T>(
    operation: impl Fn() -> Result<T, WasmSandboxError>,
    max_attempts: usize,
) -> Result<T, WasmSandboxError> {
    let mut attempts = 0;
    let mut last_error = None;
    
    while attempts < max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                attempts += 1;
                last_error = Some(error.clone());
                
                // Progressive intervention based on attempt number
                match attempts {
                    1 => {
                        // First failure: simple retry
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    2 => {
                        // Second failure: try recovery strategies
                        let recovery_manager = ErrorRecoveryManager::new();
                        let context = ErrorContext::new("prog_exec".to_string(), "prog_sandbox".to_string());
                        
                        if let Ok(RecoveryResult::Retry) = recovery_manager.attempt_recovery(&error, &context).await {
                            continue;
                        }
                    }
                    3 => {
                        // Third failure: try fallback mechanisms
                        if let Ok(_) = attempt_fallback(&error).await {
                            continue;
                        }
                    }
                    _ => {
                        // Additional failures: escalate
                        break;
                    }
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

### 3. Error Context Enrichment

```rust
// Always provide rich context for errors
pub async fn execute_with_context<T, R>(
    sandbox: &WasmSandbox,
    function: &str,
    input: &T,
) -> Result<R, ContextualError>
where
    T: serde::Serialize,
    R: for<'de> serde::Deserialize<'de>,
{
    let execution_id = uuid::Uuid::new_v4().to_string();
    let context = ErrorContext::new(execution_id.clone(), sandbox.id().to_string())
        .with_function(function.to_string())
        .with_env("input_size".to_string(), 
                  serde_json::to_string(input).map(|s| s.len().to_string()).unwrap_or_default());
    
    match sandbox.call(function, input).await {
        Ok(result) => Ok(result),
        Err(error) => Err(ContextualError { error, context }),
    }
}
```

## Next Steps

- **[Monitoring Guide](monitoring.md)** - Monitor error patterns and rates
- **[Troubleshooting Guide](troubleshooting.md)** - Debug specific error conditions
- **[Security Configuration](security-config.md)** - Handle security-related errors
- **[Production Deployment](production.md)** - Deploy with robust error handling

---

**Error Handling Excellence:** This guide provides comprehensive strategies for handling errors gracefully. Start with basic error handling and gradually implement more sophisticated recovery and fallback mechanisms as your system matures.
