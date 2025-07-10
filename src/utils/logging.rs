//! Structured logging

use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize logging
pub fn init() {
    init_with_filter("info");
}

/// Initialize logging with a specific filter
pub fn init_with_filter(filter: &str) {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new(filter))
            .with_target(true)
            .init();
    });
}

/// Log a security event
pub fn log_security_event(event_type: &str, message: &str) {
    warn!(
        target: "wasm_sandbox::security",
        event_type = %event_type,
        "Security event: {}",
        message
    );
}

/// Log a resource limit event
pub fn log_resource_limit_event(resource_type: &str, limit: u64, usage: u64) {
    warn!(
        target: "wasm_sandbox::resources",
        resource_type = %resource_type,
        limit = %limit,
        usage = %usage,
        "Resource limit event: {} usage {} / {}",
        resource_type,
        usage,
        limit
    );
}

/// Log a runtime event
pub fn log_runtime_event(event_type: &str, message: &str) {
    info!(
        target: "wasm_sandbox::runtime",
        event_type = %event_type,
        "Runtime event: {}",
        message
    );
}

/// Log a communication event
pub fn log_communication_event(channel: &str, direction: &str, bytes: usize) {
    debug!(
        target: "wasm_sandbox::communication",
        channel = %channel,
        direction = %direction,
        bytes = %bytes,
        "Communication event: {} {} bytes on channel {}",
        direction,
        bytes,
        channel
    );
}

/// Log an error
pub fn log_error(component: &str, error: &crate::error::Error) {
    error!(
        target: "wasm_sandbox::error",
        component = %component,
        "Error in {}: {}",
        component,
        error
    );
}

/// Log a custom event
pub fn log_event(_target: &str, level: tracing::Level, _data: &[(&str, &dyn std::fmt::Display)], message: &str) {
    // Use specific level macros instead of dynamic level
    match level {
        tracing::Level::ERROR => tracing::error!("{}", message),
        tracing::Level::WARN => tracing::warn!("{}", message),
        tracing::Level::INFO => tracing::info!("{}", message),
        tracing::Level::DEBUG => tracing::debug!("{}", message),
        tracing::Level::TRACE => tracing::trace!("{}", message),
    }
}
