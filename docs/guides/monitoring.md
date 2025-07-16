# Monitoring Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This guide covers comprehensive monitoring, observability, and alerting for wasm-sandbox applications in production environments, including metrics collection, distributed tracing, log aggregation, and health monitoring.

## Monitoring Philosophy

Effective monitoring for wasm-sandbox provides:

1. **Observability** - Understanding system behavior through metrics, logs, and traces
2. **Alerting** - Proactive notification of issues before they impact users
3. **Performance Tracking** - Continuous performance optimization opportunities
4. **Security Monitoring** - Detection of security threats and policy violations
5. **Capacity Planning** - Data-driven scaling decisions

## Quick Start - Basic Monitoring

```rust
use wasm_sandbox::{WasmSandbox, MonitoringConfig, MetricsCollector};
use prometheus::{Registry, Counter, Histogram, Gauge};
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter("wasm_sandbox=info,my_app=info")
        .json()
        .init();

    // Create metrics registry
    let registry = Registry::new();
    let metrics = create_metrics(&registry)?;

    // Configure monitoring
    let monitoring_config = MonitoringConfig {
        enable_metrics: true,
        enable_tracing: true,
        enable_profiling: true,
        metrics_port: 9090,
        health_check_port: 8080,
        log_level: "info".to_string(),
    };

    // Create monitored sandbox
    let sandbox = WasmSandbox::builder()
        .source("my_program.rs")
        .monitoring_config(monitoring_config)
        .metrics_collector(metrics)
        .enable_resource_monitoring(true)
        .enable_security_auditing(true)
        .build()
        .await?;

    info!("Sandbox with monitoring initialized");

    // Start metrics server
    start_metrics_server(registry, 9090).await?;

    Ok(())
}
```

## Metrics Collection

### Core Metrics

```rust
use prometheus::{Counter, Histogram, Gauge, IntGauge, opts, register_counter, register_histogram, register_gauge};

#[derive(Clone)]
pub struct SandboxMetrics {
    // Execution metrics
    pub executions_total: Counter,
    pub executions_successful: Counter,
    pub executions_failed: Counter,
    pub execution_duration: Histogram,
    
    // Resource metrics
    pub memory_usage_bytes: Gauge,
    pub memory_limit_bytes: Gauge,
    pub cpu_usage_percent: Gauge,
    pub fuel_consumed_total: Counter,
    
    // Instance metrics
    pub active_instances: IntGauge,
    pub instance_creation_duration: Histogram,
    pub instance_destruction_total: Counter,
    
    // Cache metrics
    pub module_cache_hits: Counter,
    pub module_cache_misses: Counter,
    pub module_cache_size: IntGauge,
    
    // Security metrics
    pub security_violations: Counter,
    pub capability_requests: Counter,
    pub resource_limit_exceeded: Counter,
    
    // Network metrics
    pub network_requests_total: Counter,
    pub network_bytes_sent: Counter,
    pub network_bytes_received: Counter,
    pub network_errors: Counter,
    
    // I/O metrics
    pub file_operations_total: Counter,
    pub file_bytes_read: Counter,
    pub file_bytes_written: Counter,
    pub file_errors: Counter,
}

impl SandboxMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        Ok(Self {
            executions_total: register_counter!(opts!(
                "wasm_executions_total",
                "Total number of WebAssembly function executions"
            ))?,
            
            executions_successful: register_counter!(opts!(
                "wasm_executions_successful_total",
                "Number of successful WebAssembly executions"
            ))?,
            
            executions_failed: register_counter!(opts!(
                "wasm_executions_failed_total",
                "Number of failed WebAssembly executions"
            ))?,
            
            execution_duration: register_histogram!(prometheus::HistogramOpts::new(
                "wasm_execution_duration_seconds",
                "Time spent executing WebAssembly functions"
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0]))?,
            
            memory_usage_bytes: register_gauge!(opts!(
                "wasm_memory_usage_bytes",
                "Current memory usage in bytes"
            ))?,
            
            memory_limit_bytes: register_gauge!(opts!(
                "wasm_memory_limit_bytes",
                "Memory limit in bytes"
            ))?,
            
            cpu_usage_percent: register_gauge!(opts!(
                "wasm_cpu_usage_percent",
                "CPU usage percentage"
            ))?,
            
            fuel_consumed_total: register_counter!(opts!(
                "wasm_fuel_consumed_total",
                "Total fuel consumed across all executions"
            ))?,
            
            active_instances: register_int_gauge!(opts!(
                "wasm_active_instances",
                "Number of active WebAssembly instances"
            ))?,
            
            instance_creation_duration: register_histogram!(prometheus::HistogramOpts::new(
                "wasm_instance_creation_duration_seconds",
                "Time spent creating WebAssembly instances"
            ).buckets(vec![0.001, 0.01, 0.1, 1.0, 5.0]))?,
            
            instance_destruction_total: register_counter!(opts!(
                "wasm_instance_destruction_total",
                "Total number of destroyed instances"
            ))?,
            
            module_cache_hits: register_counter!(opts!(
                "wasm_module_cache_hits_total",
                "Number of module cache hits"
            ))?,
            
            module_cache_misses: register_counter!(opts!(
                "wasm_module_cache_misses_total",
                "Number of module cache misses"
            ))?,
            
            module_cache_size: register_int_gauge!(opts!(
                "wasm_module_cache_size",
                "Number of modules in cache"
            ))?,
            
            security_violations: register_counter!(opts!(
                "wasm_security_violations_total",
                "Number of security violations detected"
            ))?,
            
            capability_requests: register_counter!(opts!(
                "wasm_capability_requests_total",
                "Number of capability requests"
            ))?,
            
            resource_limit_exceeded: register_counter!(opts!(
                "wasm_resource_limit_exceeded_total",
                "Number of resource limit violations"
            ))?,
            
            network_requests_total: register_counter!(opts!(
                "wasm_network_requests_total",
                "Total number of network requests"
            ))?,
            
            network_bytes_sent: register_counter!(opts!(
                "wasm_network_bytes_sent_total",
                "Total bytes sent over network"
            ))?,
            
            network_bytes_received: register_counter!(opts!(
                "wasm_network_bytes_received_total",
                "Total bytes received over network"
            ))?,
            
            network_errors: register_counter!(opts!(
                "wasm_network_errors_total",
                "Number of network errors"
            ))?,
            
            file_operations_total: register_counter!(opts!(
                "wasm_file_operations_total",
                "Total number of file operations"
            ))?,
            
            file_bytes_read: register_counter!(opts!(
                "wasm_file_bytes_read_total",
                "Total bytes read from files"
            ))?,
            
            file_bytes_written: register_counter!(opts!(
                "wasm_file_bytes_written_total",
                "Total bytes written to files"
            ))?,
            
            file_errors: register_counter!(opts!(
                "wasm_file_errors_total",
                "Number of file operation errors"
            ))?,
        })
    }
}
```

### Custom Metrics Collection

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MetricsCollector {
    metrics: Arc<SandboxMetrics>,
    custom_metrics: Arc<RwLock<HashMap<String, CustomMetric>>>,
}

#[derive(Debug, Clone)]
pub enum CustomMetric {
    Counter(f64),
    Gauge(f64),
    Histogram(Vec<f64>),
}

impl MetricsCollector {
    pub fn new(metrics: Arc<SandboxMetrics>) -> Self {
        Self {
            metrics,
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn record_execution(&self, duration: Duration, success: bool, fuel_used: u64) {
        self.metrics.executions_total.inc();
        
        if success {
            self.metrics.executions_successful.inc();
        } else {
            self.metrics.executions_failed.inc();
        }
        
        self.metrics.execution_duration.observe(duration.as_secs_f64());
        self.metrics.fuel_consumed_total.inc_by(fuel_used);
    }
    
    pub async fn record_memory_usage(&self, used: u64, limit: u64) {
        self.metrics.memory_usage_bytes.set(used as f64);
        self.metrics.memory_limit_bytes.set(limit as f64);
    }
    
    pub async fn record_instance_lifecycle(&self, created: bool, duration: Option<Duration>) {
        if created {
            self.metrics.active_instances.inc();
            if let Some(d) = duration {
                self.metrics.instance_creation_duration.observe(d.as_secs_f64());
            }
        } else {
            self.metrics.active_instances.dec();
            self.metrics.instance_destruction_total.inc();
        }
    }
    
    pub async fn record_cache_operation(&self, hit: bool, cache_size: usize) {
        if hit {
            self.metrics.module_cache_hits.inc();
        } else {
            self.metrics.module_cache_misses.inc();
        }
        self.metrics.module_cache_size.set(cache_size as i64);
    }
    
    pub async fn record_security_event(&self, event_type: SecurityEventType) {
        match event_type {
            SecurityEventType::Violation => self.metrics.security_violations.inc(),
            SecurityEventType::CapabilityRequest => self.metrics.capability_requests.inc(),
            SecurityEventType::ResourceLimit => self.metrics.resource_limit_exceeded.inc(),
        }
    }
    
    pub async fn record_network_operation(&self, bytes_sent: u64, bytes_received: u64, error: bool) {
        self.metrics.network_requests_total.inc();
        self.metrics.network_bytes_sent.inc_by(bytes_sent);
        self.metrics.network_bytes_received.inc_by(bytes_received);
        
        if error {
            self.metrics.network_errors.inc();
        }
    }
    
    pub async fn record_file_operation(&self, bytes_read: u64, bytes_written: u64, error: bool) {
        self.metrics.file_operations_total.inc();
        self.metrics.file_bytes_read.inc_by(bytes_read);
        self.metrics.file_bytes_written.inc_by(bytes_written);
        
        if error {
            self.metrics.file_errors.inc();
        }
    }
    
    pub async fn record_custom_metric(&self, name: &str, value: f64, metric_type: CustomMetricType) {
        let mut custom_metrics = self.custom_metrics.write().await;
        
        match metric_type {
            CustomMetricType::Counter => {
                let entry = custom_metrics.entry(name.to_string()).or_insert(CustomMetric::Counter(0.0));
                if let CustomMetric::Counter(ref mut current) = entry {
                    *current += value;
                }
            }
            CustomMetricType::Gauge => {
                custom_metrics.insert(name.to_string(), CustomMetric::Gauge(value));
            }
            CustomMetricType::Histogram => {
                let entry = custom_metrics.entry(name.to_string()).or_insert(CustomMetric::Histogram(Vec::new()));
                if let CustomMetric::Histogram(ref mut values) = entry {
                    values.push(value);
                }
            }
        }
    }
}
```

## Distributed Tracing

### OpenTelemetry Integration

```rust
use opentelemetry::{global, trace::{TraceError, Tracer}, KeyValue};
use opentelemetry_jaeger::JaegerPipeline;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct TracingConfig {
    pub jaeger_endpoint: String,
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
}

pub fn init_tracing(config: TracingConfig) -> Result<(), TraceError> {
    let tracer = JaegerPipeline::new()
        .with_service_name(&config.service_name)
        .with_tags(vec![
            KeyValue::new("service.version", config.service_version.clone()),
            KeyValue::new("environment", config.environment.clone()),
        ])
        .with_agent_endpoint(&config.jaeger_endpoint)
        .install_batch(opentelemetry::runtime::Tokio)?;

    let opentelemetry_layer = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(opentelemetry_layer)
        .with(tracing_subscriber::fmt::layer().json())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    Ok(())
}

// Instrumented sandbox execution
#[tracing::instrument(skip(sandbox, input), fields(function = %function))]
pub async fn execute_with_tracing<T, R>(
    sandbox: &WasmSandbox,
    function: &str,
    input: &T,
) -> Result<R, Error>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let span = tracing::Span::current();
    
    // Add custom attributes
    span.record("wasm.module", &sandbox.module_name());
    span.record("wasm.function", function);
    
    let start_time = Instant::now();
    
    match sandbox.call(function, input).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            span.record("wasm.duration_ms", duration.as_millis());
            span.record("wasm.success", true);
            
            tracing::info!(
                function = %function,
                duration_ms = duration.as_millis(),
                "WebAssembly function executed successfully"
            );
            
            Ok(result)
        }
        Err(error) => {
            let duration = start_time.elapsed();
            span.record("wasm.duration_ms", duration.as_millis());
            span.record("wasm.success", false);
            span.record("wasm.error", &error.to_string());
            
            tracing::error!(
                function = %function,
                duration_ms = duration.as_millis(),
                error = %error,
                "WebAssembly function execution failed"
            );
            
            Err(error)
        }
    }
}
```

### Span Correlation

```rust
use tracing::{Span, instrument};
use uuid::Uuid;

pub struct ExecutionContext {
    pub trace_id: String,
    pub span_id: String,
    pub execution_id: String,
    pub sandbox_id: String,
}

impl ExecutionContext {
    pub fn new(sandbox_id: &str) -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            execution_id: Uuid::new_v4().to_string(),
            sandbox_id: sandbox_id.to_string(),
        }
    }
}

#[instrument(
    skip(sandbox, input, context),
    fields(
        trace_id = %context.trace_id,
        execution_id = %context.execution_id,
        sandbox_id = %context.sandbox_id
    )
)]
pub async fn execute_with_context<T, R>(
    sandbox: &WasmSandbox,
    function: &str,
    input: &T,
    context: ExecutionContext,
) -> Result<R, Error>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    // Inject trace context into sandbox execution
    sandbox.set_trace_context(&context).await?;
    
    // Execute with full tracing
    let result = execute_with_tracing(sandbox, function, input).await?;
    
    Ok(result)
}
```

## Log Aggregation

### Structured Logging

```rust
use serde_json::{json, Value};
use tracing::{event, Level, field::{Field, Visit}};

pub struct StructuredLogger {
    app_name: String,
    version: String,
    environment: String,
}

impl StructuredLogger {
    pub fn new(app_name: String, version: String, environment: String) -> Self {
        Self {
            app_name,
            version,
            environment,
        }
    }
    
    pub fn log_execution(&self, event: ExecutionEvent) {
        let log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": event.level.as_str(),
            "app": self.app_name,
            "version": self.version,
            "environment": self.environment,
            "event_type": "wasm_execution",
            "trace_id": event.trace_id,
            "execution_id": event.execution_id,
            "sandbox_id": event.sandbox_id,
            "function": event.function,
            "duration_ms": event.duration.as_millis(),
            "success": event.success,
            "memory_used": event.memory_used,
            "fuel_consumed": event.fuel_consumed,
            "error": event.error,
            "custom_fields": event.custom_fields
        });
        
        match event.level {
            Level::ERROR => tracing::error!("{}", log_entry),
            Level::WARN => tracing::warn!("{}", log_entry),
            Level::INFO => tracing::info!("{}", log_entry),
            Level::DEBUG => tracing::debug!("{}", log_entry),
            Level::TRACE => tracing::trace!("{}", log_entry),
        }
    }
    
    pub fn log_security_event(&self, event: SecurityEvent) {
        let log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": "warn",
            "app": self.app_name,
            "version": self.version,
            "environment": self.environment,
            "event_type": "security_event",
            "trace_id": event.trace_id,
            "sandbox_id": event.sandbox_id,
            "violation_type": event.violation_type,
            "capability": event.capability,
            "resource": event.resource,
            "threshold": event.threshold,
            "actual_value": event.actual_value,
            "action_taken": event.action_taken
        });
        
        tracing::warn!("{}", log_entry);
    }
    
    pub fn log_resource_event(&self, event: ResourceEvent) {
        let log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": event.level.as_str(),
            "app": self.app_name,
            "version": self.version,
            "environment": self.environment,
            "event_type": "resource_event",
            "trace_id": event.trace_id,
            "sandbox_id": event.sandbox_id,
            "resource_type": event.resource_type,
            "current_usage": event.current_usage,
            "limit": event.limit,
            "percentage": event.percentage,
            "trend": event.trend
        });
        
        match event.level {
            Level::ERROR => tracing::error!("{}", log_entry),
            Level::WARN => tracing::warn!("{}", log_entry),
            Level::INFO => tracing::info!("{}", log_entry),
            _ => tracing::debug!("{}", log_entry),
        }
    }
}

#[derive(Debug)]
pub struct ExecutionEvent {
    pub level: Level,
    pub trace_id: String,
    pub execution_id: String,
    pub sandbox_id: String,
    pub function: String,
    pub duration: Duration,
    pub success: bool,
    pub memory_used: u64,
    pub fuel_consumed: u64,
    pub error: Option<String>,
    pub custom_fields: Value,
}

#[derive(Debug)]
pub struct SecurityEvent {
    pub trace_id: String,
    pub sandbox_id: String,
    pub violation_type: String,
    pub capability: Option<String>,
    pub resource: Option<String>,
    pub threshold: Option<u64>,
    pub actual_value: Option<u64>,
    pub action_taken: String,
}

#[derive(Debug)]
pub struct ResourceEvent {
    pub level: Level,
    pub trace_id: String,
    pub sandbox_id: String,
    pub resource_type: String,
    pub current_usage: u64,
    pub limit: u64,
    pub percentage: f64,
    pub trend: String,
}
```

## Health Monitoring

### Health Check System

```rust
use std::collections::HashMap;
use tokio::time::{interval, Duration};

pub struct HealthMonitor {
    checks: HashMap<String, Box<dyn HealthCheck + Send + Sync>>,
    check_interval: Duration,
    unhealthy_threshold: u32,
    failure_counts: HashMap<String, u32>,
}

#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthResult;
    fn name(&self) -> &str;
    fn critical(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct HealthResult {
    pub healthy: bool,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
    pub response_time: Duration,
}

impl HealthMonitor {
    pub fn new(check_interval: Duration, unhealthy_threshold: u32) -> Self {
        Self {
            checks: HashMap::new(),
            check_interval,
            unhealthy_threshold,
            failure_counts: HashMap::new(),
        }
    }
    
    pub fn add_check(&mut self, check: Box<dyn HealthCheck + Send + Sync>) {
        let name = check.name().to_string();
        self.checks.insert(name.clone(), check);
        self.failure_counts.insert(name, 0);
    }
    
    pub async fn start_monitoring(&mut self) {
        let mut interval = interval(self.check_interval);
        
        loop {
            interval.tick().await;
            self.run_all_checks().await;
        }
    }
    
    async fn run_all_checks(&mut self) {
        for (name, check) in &self.checks {
            let start_time = Instant::now();
            let result = check.check().await;
            let check_duration = start_time.elapsed();
            
            if result.healthy {
                self.failure_counts.insert(name.clone(), 0);
                tracing::debug!(
                    check = %name,
                    duration_ms = check_duration.as_millis(),
                    "Health check passed"
                );
            } else {
                let failure_count = self.failure_counts.get(name).unwrap_or(&0) + 1;
                self.failure_counts.insert(name.clone(), failure_count);
                
                let level = if check.critical() || failure_count >= self.unhealthy_threshold {
                    Level::ERROR
                } else {
                    Level::WARN
                };
                
                tracing::event!(
                    level,
                    check = %name,
                    failure_count = failure_count,
                    message = %result.message,
                    duration_ms = check_duration.as_millis(),
                    "Health check failed"
                );
                
                // Trigger alerts for critical failures
                if check.critical() && failure_count >= self.unhealthy_threshold {
                    self.trigger_alert(name, &result).await;
                }
            }
        }
    }
    
    async fn trigger_alert(&self, check_name: &str, result: &HealthResult) {
        // Implementation depends on alerting system (Slack, PagerDuty, etc.)
        tracing::error!(
            check = %check_name,
            message = %result.message,
            "CRITICAL: Health check failure threshold exceeded"
        );
    }
    
    pub async fn get_health_status(&self) -> OverallHealthStatus {
        let mut check_results = HashMap::new();
        let mut all_healthy = true;
        let mut critical_failure = false;
        
        for (name, check) in &self.checks {
            let result = check.check().await;
            
            if !result.healthy {
                all_healthy = false;
                if check.critical() {
                    critical_failure = true;
                }
            }
            
            check_results.insert(name.clone(), result);
        }
        
        let status = if all_healthy {
            "healthy"
        } else if critical_failure {
            "critical"
        } else {
            "degraded"
        };
        
        OverallHealthStatus {
            status: status.to_string(),
            checks: check_results,
            timestamp: chrono::Utc::now(),
        }
    }
}

// Health check implementations
pub struct SandboxHealthCheck {
    sandbox_pool: Arc<SandboxPool>,
}

#[async_trait::async_trait]
impl HealthCheck for SandboxHealthCheck {
    async fn check(&self) -> HealthResult {
        let start_time = Instant::now();
        
        match self.sandbox_pool.get().await {
            Ok(sandbox) => {
                // Try to execute a simple health check function
                match sandbox.call("health_check", &()).await {
                    Ok(_) => {
                        HealthResult {
                            healthy: true,
                            message: "Sandbox pool is healthy".to_string(),
                            details: HashMap::new(),
                            response_time: start_time.elapsed(),
                        }
                    }
                    Err(e) => {
                        HealthResult {
                            healthy: false,
                            message: format!("Sandbox execution failed: {}", e),
                            details: HashMap::new(),
                            response_time: start_time.elapsed(),
                        }
                    }
                }
            }
            Err(e) => {
                HealthResult {
                    healthy: false,
                    message: format!("Cannot acquire sandbox from pool: {}", e),
                    details: HashMap::new(),
                    response_time: start_time.elapsed(),
                }
            }
        }
    }
    
    fn name(&self) -> &str {
        "sandbox_pool"
    }
    
    fn critical(&self) -> bool {
        true
    }
}

pub struct MemoryHealthCheck {
    memory_threshold: f64, // percentage
}

#[async_trait::async_trait]
impl HealthCheck for MemoryHealthCheck {
    async fn check(&self) -> HealthResult {
        let start_time = Instant::now();
        let memory_usage = get_system_memory_usage();
        
        let healthy = memory_usage.percentage < self.memory_threshold;
        
        let mut details = HashMap::new();
        details.insert("used_bytes".to_string(), json!(memory_usage.used));
        details.insert("total_bytes".to_string(), json!(memory_usage.total));
        details.insert("percentage".to_string(), json!(memory_usage.percentage));
        details.insert("threshold".to_string(), json!(self.memory_threshold));
        
        HealthResult {
            healthy,
            message: if healthy {
                format!("Memory usage is normal: {:.1}%", memory_usage.percentage)
            } else {
                format!("High memory usage: {:.1}% (threshold: {:.1}%)", 
                       memory_usage.percentage, self.memory_threshold)
            },
            details,
            response_time: start_time.elapsed(),
        }
    }
    
    fn name(&self) -> &str {
        "memory_usage"
    }
    
    fn critical(&self) -> bool {
        false
    }
}
```

## Alerting System

### Alert Configuration

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub rules: Vec<AlertRule>,
    pub channels: Vec<AlertChannel>,
    pub escalation_policies: Vec<EscalationPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub channels: Vec<String>,
    pub escalation_policy: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    MetricThreshold {
        metric: String,
        operator: ComparisonOperator,
        threshold: f64,
        duration: Duration,
    },
    HealthCheckFailure {
        check_name: String,
        consecutive_failures: u32,
    },
    ErrorRate {
        threshold_percentage: f64,
        window_duration: Duration,
    },
    ResourceUsage {
        resource: String,
        threshold_percentage: f64,
        duration: Duration,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

pub struct AlertManager {
    config: AlertConfig,
    active_alerts: HashMap<String, ActiveAlert>,
    notification_handlers: HashMap<String, Box<dyn NotificationHandler + Send + Sync>>,
}

impl AlertManager {
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config,
            active_alerts: HashMap::new(),
            notification_handlers: HashMap::new(),
        }
    }
    
    pub fn add_notification_handler(&mut self, name: String, handler: Box<dyn NotificationHandler + Send + Sync>) {
        self.notification_handlers.insert(name, handler);
    }
    
    pub async fn evaluate_rules(&mut self, metrics: &SandboxMetrics) {
        for rule in &self.config.rules {
            if !rule.enabled {
                continue;
            }
            
            let should_alert = self.evaluate_condition(&rule.condition, metrics).await;
            
            if should_alert {
                self.trigger_alert(rule).await;
            } else {
                self.resolve_alert(&rule.name).await;
            }
        }
    }
    
    async fn evaluate_condition(&self, condition: &AlertCondition, metrics: &SandboxMetrics) -> bool {
        match condition {
            AlertCondition::MetricThreshold { metric, operator, threshold, duration: _ } => {
                let value = self.get_metric_value(metric, metrics).await;
                self.compare_values(value, *threshold, operator)
            }
            AlertCondition::HealthCheckFailure { check_name: _, consecutive_failures: _ } => {
                // Implementation depends on health check results
                false
            }
            AlertCondition::ErrorRate { threshold_percentage, window_duration: _ } => {
                let total_executions = metrics.executions_total.get();
                let failed_executions = metrics.executions_failed.get();
                
                if total_executions > 0.0 {
                    let error_rate = (failed_executions / total_executions) * 100.0;
                    error_rate > *threshold_percentage
                } else {
                    false
                }
            }
            AlertCondition::ResourceUsage { resource, threshold_percentage, duration: _ } => {
                match resource.as_str() {
                    "memory" => {
                        let usage = metrics.memory_usage_bytes.get();
                        let limit = metrics.memory_limit_bytes.get();
                        if limit > 0.0 {
                            let percentage = (usage / limit) * 100.0;
                            percentage > *threshold_percentage
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
        }
    }
    
    async fn trigger_alert(&mut self, rule: &AlertRule) {
        let alert_id = format!("{}_{}", rule.name, chrono::Utc::now().timestamp());
        
        if self.active_alerts.contains_key(&rule.name) {
            // Alert already active, update it
            return;
        }
        
        let alert = ActiveAlert {
            id: alert_id.clone(),
            rule_name: rule.name.clone(),
            severity: rule.severity.clone(),
            triggered_at: chrono::Utc::now(),
            escalated: false,
        };
        
        self.active_alerts.insert(rule.name.clone(), alert);
        
        // Send notifications
        for channel_name in &rule.channels {
            if let Some(handler) = self.notification_handlers.get(channel_name) {
                let notification = Notification {
                    alert_id: alert_id.clone(),
                    title: format!("Alert: {}", rule.name),
                    message: rule.description.clone(),
                    severity: rule.severity.clone(),
                    timestamp: chrono::Utc::now(),
                };
                
                if let Err(e) = handler.send_notification(notification).await {
                    tracing::error!(
                        channel = %channel_name,
                        alert = %rule.name,
                        error = %e,
                        "Failed to send alert notification"
                    );
                }
            }
        }
        
        tracing::warn!(
            alert = %rule.name,
            severity = ?rule.severity,
            "Alert triggered"
        );
    }
    
    async fn resolve_alert(&mut self, rule_name: &str) {
        if let Some(alert) = self.active_alerts.remove(rule_name) {
            tracing::info!(
                alert = %rule_name,
                duration_seconds = (chrono::Utc::now() - alert.triggered_at).num_seconds(),
                "Alert resolved"
            );
        }
    }
}

#[async_trait::async_trait]
pub trait NotificationHandler: Send + Sync {
    async fn send_notification(&self, notification: Notification) -> Result<(), NotificationError>;
}

// Slack notification handler
pub struct SlackNotificationHandler {
    webhook_url: String,
    client: reqwest::Client,
}

#[async_trait::async_trait]
impl NotificationHandler for SlackNotificationHandler {
    async fn send_notification(&self, notification: Notification) -> Result<(), NotificationError> {
        let emoji = match notification.severity {
            AlertSeverity::Critical => "🚨",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Info => "ℹ️",
        };
        
        let payload = serde_json::json!({
            "text": format!("{} {} - {}", emoji, notification.title, notification.message),
            "attachments": [
                {
                    "color": match notification.severity {
                        AlertSeverity::Critical => "danger",
                        AlertSeverity::Warning => "warning", 
                        AlertSeverity::Info => "good",
                    },
                    "fields": [
                        {
                            "title": "Alert ID",
                            "value": notification.alert_id,
                            "short": true
                        },
                        {
                            "title": "Timestamp",
                            "value": notification.timestamp.to_rfc3339(),
                            "short": true
                        }
                    ]
                }
            ]
        });
        
        let response = self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(NotificationError::SendFailed(
                format!("Slack API error: {}", response.status())
            ));
        }
        
        Ok(())
    }
}
```

## Performance Monitoring

### Application Performance Monitoring (APM)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PerformanceMonitor {
    metrics: Arc<SandboxMetrics>,
    performance_data: Arc<RwLock<PerformanceData>>,
    sampling_rate: f64,
}

#[derive(Debug, Default)]
pub struct PerformanceData {
    pub function_timings: HashMap<String, FunctionTimings>,
    pub memory_profile: MemoryProfile,
    pub cpu_profile: CpuProfile,
    pub hotspots: Vec<PerformanceHotspot>,
}

#[derive(Debug, Default)]
pub struct FunctionTimings {
    pub call_count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub p50_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
    pub recent_samples: Vec<Duration>,
}

impl PerformanceMonitor {
    pub fn new(metrics: Arc<SandboxMetrics>, sampling_rate: f64) -> Self {
        Self {
            metrics,
            performance_data: Arc::new(RwLock::new(PerformanceData::default())),
            sampling_rate,
        }
    }
    
    pub async fn record_function_call(&self, function: &str, duration: Duration, memory_used: u64) {
        // Sample based on sampling rate
        if rand::random::<f64>() > self.sampling_rate {
            return;
        }
        
        let mut data = self.performance_data.write().await;
        
        let timings = data.function_timings
            .entry(function.to_string())
            .or_insert_with(FunctionTimings::default);
        
        timings.call_count += 1;
        timings.total_duration += duration;
        
        if timings.call_count == 1 || duration < timings.min_duration {
            timings.min_duration = duration;
        }
        
        if duration > timings.max_duration {
            timings.max_duration = duration;
        }
        
        // Keep recent samples for percentile calculation
        timings.recent_samples.push(duration);
        if timings.recent_samples.len() > 1000 {
            timings.recent_samples.remove(0);
        }
        
        // Update percentiles
        self.update_percentiles(timings);
        
        // Update memory profile
        data.memory_profile.record_usage(memory_used);
        
        // Detect performance hotspots
        if duration > Duration::from_millis(1000) {
            data.hotspots.push(PerformanceHotspot {
                function: function.to_string(),
                duration,
                memory_used,
                timestamp: chrono::Utc::now(),
            });
            
            // Keep only recent hotspots
            data.hotspots.retain(|h| {
                chrono::Utc::now().signed_duration_since(h.timestamp).num_minutes() < 60
            });
        }
    }
    
    fn update_percentiles(&self, timings: &mut FunctionTimings) {
        if timings.recent_samples.is_empty() {
            return;
        }
        
        let mut samples = timings.recent_samples.clone();
        samples.sort();
        
        let len = samples.len();
        timings.p50_duration = samples[len * 50 / 100];
        timings.p95_duration = samples[len * 95 / 100];
        timings.p99_duration = samples[len * 99 / 100];
    }
    
    pub async fn get_performance_report(&self) -> PerformanceReport {
        let data = self.performance_data.read().await;
        
        PerformanceReport {
            function_timings: data.function_timings.clone(),
            memory_profile: data.memory_profile.clone(),
            cpu_profile: data.cpu_profile.clone(),
            hotspots: data.hotspots.clone(),
            generated_at: chrono::Utc::now(),
        }
    }
    
    pub async fn detect_anomalies(&self) -> Vec<PerformanceAnomaly> {
        let data = self.performance_data.read().await;
        let mut anomalies = Vec::new();
        
        for (function, timings) in &data.function_timings {
            // Detect slow functions
            if timings.p95_duration > Duration::from_millis(5000) {
                anomalies.push(PerformanceAnomaly {
                    anomaly_type: AnomalyType::SlowFunction,
                    function: Some(function.clone()),
                    description: format!(
                        "Function {} has high P95 latency: {}ms",
                        function,
                        timings.p95_duration.as_millis()
                    ),
                    severity: if timings.p95_duration > Duration::from_millis(10000) {
                        AnomalySeverity::High
                    } else {
                        AnomalySeverity::Medium
                    },
                    detected_at: chrono::Utc::now(),
                });
            }
            
            // Detect high variance
            let avg_duration = timings.total_duration / timings.call_count as u32;
            let variance_ratio = timings.max_duration.as_millis() as f64 / avg_duration.as_millis() as f64;
            
            if variance_ratio > 10.0 {
                anomalies.push(PerformanceAnomaly {
                    anomaly_type: AnomalyType::HighVariance,
                    function: Some(function.clone()),
                    description: format!(
                        "Function {} has high performance variance: {}x",
                        function,
                        variance_ratio
                    ),
                    severity: AnomalySeverity::Low,
                    detected_at: chrono::Utc::now(),
                });
            }
        }
        
        anomalies
    }
}
```

## Dashboard Integration

### Grafana Dashboard Configuration

```json
{
  "dashboard": {
    "title": "WebAssembly Sandbox Monitoring",
    "tags": ["wasm", "sandbox", "monitoring"],
    "refresh": "30s",
    "panels": [
      {
        "title": "Execution Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(wasm_executions_total[5m])",
            "legendFormat": "Executions/sec"
          }
        ]
      },
      {
        "title": "Success Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(wasm_executions_successful_total[5m]) / rate(wasm_executions_total[5m]) * 100",
            "legendFormat": "Success Rate %"
          }
        ]
      },
      {
        "title": "Execution Duration",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(wasm_execution_duration_seconds_bucket[5m]))",
            "legendFormat": "P50"
          },
          {
            "expr": "histogram_quantile(0.95, rate(wasm_execution_duration_seconds_bucket[5m]))",
            "legendFormat": "P95"
          },
          {
            "expr": "histogram_quantile(0.99, rate(wasm_execution_duration_seconds_bucket[5m]))",
            "legendFormat": "P99"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "wasm_memory_usage_bytes",
            "legendFormat": "Used"
          },
          {
            "expr": "wasm_memory_limit_bytes",
            "legendFormat": "Limit"
          }
        ]
      },
      {
        "title": "Active Instances",
        "type": "stat",
        "targets": [
          {
            "expr": "wasm_active_instances",
            "legendFormat": "Instances"
          }
        ]
      },
      {
        "title": "Security Violations",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(wasm_security_violations_total[5m])",
            "legendFormat": "Violations/sec"
          }
        ]
      }
    ]
  }
}
```

## Next Steps

- **[Troubleshooting Guide](troubleshooting.md)** - Debug monitoring issues
- **[Production Deployment](production.md)** - Deploy monitoring in production
- **[Security Configuration](security-config.md)** - Monitor security events
- **[Performance Guide](../design/performance.md)** - Optimize performance

---

**Monitoring Excellence:** This guide provides enterprise-grade monitoring capabilities. Start with basic metrics and gradually add more sophisticated observability as your system grows.
