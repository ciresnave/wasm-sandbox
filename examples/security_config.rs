//! Security configuration: build `Capabilities` and `ResourceLimits` by hand and create an
//! instance with them. This is the example the Security Configuration Guide points to.
//!
//! Run with: `cargo run --example security_config`

use wasm_sandbox::security::audit::{AuditEventType, AuditLogger};
use wasm_sandbox::security::{
    Capabilities, FilesystemCapability, HostSpec, NetworkCapability, PortRange, ResourceLimits,
};
use wasm_sandbox::{CpuLimits, EnvironmentCapability, InstanceConfig, Result, WasmSandbox};

const TEST_MODULE: &[u8] = include_bytes!("../fixtures/test_module.wasm");

#[tokio::main]
async fn main() -> Result<()> {
    // Start from nothing and grant only what is needed.
    let capabilities = Capabilities {
        // One enum value, not a list: choose None, Loopback, AllowedHosts, AllowedPorts or Full.
        network: NetworkCapability::AllowedHosts(vec![HostSpec {
            host: "api.example.com".to_string(),
            ports: Some(PortRange::new(443, 443)),
            secure: true,
        }]),
        // Directory lists live inside a single struct.
        filesystem: FilesystemCapability {
            readable_dirs: vec!["./test_input".into()],
            writable_dirs: vec!["./test_output".into()],
            max_file_size: Some(10 * 1024 * 1024),
            allow_create: true,
            allow_delete: false,
        },
        environment: EnvironmentCapability::Allowlist(vec!["API_KEY".to_string()]),
        ..Capabilities::minimal()
    };

    let limits = ResourceLimits {
        fuel: Some(1_000_000),
        cpu: CpuLimits {
            max_execution_time_ms: 2_000,
            ..CpuLimits::default()
        },
        ..ResourceLimits::default()
    };

    let mut sandbox = WasmSandbox::new()?;
    let module_id = sandbox.load_module(TEST_MODULE)?;
    let instance_id = sandbox.create_instance(
        module_id,
        Some(InstanceConfig {
            capabilities,
            resource_limits: limits,
            ..Default::default()
        }),
    )?;

    let sum: i32 = sandbox
        .call_function(instance_id, "add", &(2i32, 40i32))
        .await?;
    println!("add(2, 40) = {sum}");

    // The audit logger records events you give it; the runtime does not write to it for you.
    let logger = AuditLogger::new(1000);
    logger.warning(
        AuditEventType::Custom {
            event_type: "fuel".to_string(),
            data: "instance near fuel limit".to_string(),
        },
        "instance near fuel limit",
    );
    println!("{} audit event(s) recorded", logger.get_events().len());
    Ok(())
}
