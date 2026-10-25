//! Security configuration: build `Capabilities` and `ResourceLimits` by hand and create an
//! instance with them. This is the example the Security Configuration Guide points to.
//!
//! Run with: `cargo run --example security_config`

use wasm_sandbox::security::audit::{AuditEventType, AuditLogger};
use wasm_sandbox::security::{Capabilities, FilesystemCapability, ResourceLimits};
use wasm_sandbox::{CpuLimits, EnvironmentCapability, InstanceConfig, Result, WasmSandbox};

const TEST_MODULE: &[u8] = include_bytes!("../fixtures/test_module.wasm");

#[tokio::main]
async fn main() -> Result<()> {
    // The granted directories must exist.
    std::fs::create_dir_all("./test_input")?;
    std::fs::create_dir_all("./test_output")?;

    // Start from nothing and grant only what is needed. Network and process grants are not
    // supported by the Wasmtime backend: anything but `None` fails at `create_instance`.
    let capabilities = Capabilities {
        // Directory lists live inside a single struct.
        filesystem: FilesystemCapability {
            readable_dirs: vec!["./test_input".into()],
            writable_dirs: vec!["./test_output".into()],
            // Writable directories need both flags and cannot cap file size.
            max_file_size: None,
            allow_create: true,
            allow_delete: true,
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
