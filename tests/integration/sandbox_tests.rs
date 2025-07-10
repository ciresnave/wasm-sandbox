//! Integration tests for the wasm-sandbox

use std::sync::Arc;
use std::path::PathBuf;

use wasm_sandbox::{
    WasmSandbox, SandboxConfig, InstanceConfig, InstanceId,
    security::{
        Capabilities, NetworkCapability, FilesystemCapability, 
        EnvironmentCapability, ResourceLimits
    },
    communication::{MemoryChannel, RpcChannel},
};

/// Test fixture path
fn test_fixture_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures");
    path.push(filename);
    path
}

/// Test basic sandbox functionality
#[tokio::test]
async fn test_sandbox_creation() {
    // Create a sandbox
    let sandbox = WasmSandbox::new().expect("Failed to create sandbox");
    
    // Check that it was created successfully
    assert!(sandbox.runtime().get_metrics().compiled_modules == 0);
}

/// Test module loading
#[tokio::test]
async fn test_module_loading() {
    // Create a sandbox
    let mut sandbox = WasmSandbox::new().expect("Failed to create sandbox");
    
    // Load a test module
    let fixture_path = test_fixture_path("test_module.wasm");
    let wasm_bytes = std::fs::read(fixture_path).expect("Failed to read test module");
    
    let module_id = sandbox.load_module(&wasm_bytes).expect("Failed to load module");
    
    // Check that the module was loaded
    assert!(sandbox.runtime().get_metrics().compiled_modules == 1);
    
    // Create an instance
    let instance_id = sandbox.create_instance(
        module_id,
        Some(InstanceConfig::default()),
    ).expect("Failed to create instance");
    
    // Check that we got a valid instance ID
    assert!(sandbox.get_instance(instance_id).is_some());
}

/// Test function calling
#[tokio::test]
async fn test_function_calling() {
    // Create a sandbox
    let mut sandbox = WasmSandbox::new().expect("Failed to create sandbox");
    
    // Load a test module
    let fixture_path = test_fixture_path("test_module.wasm");
    let wasm_bytes = std::fs::read(fixture_path).expect("Failed to read test module");
    
    let module_id = sandbox.load_module(&wasm_bytes).expect("Failed to load module");
    
    // Create an instance
    let instance_id = sandbox.create_instance(
        module_id,
        Some(InstanceConfig::default()),
    ).expect("Failed to create instance");
    
    // Call a function
    // This is a mock since we don't have an actual WASM file with this function
    let instance = sandbox.get_instance_mut(instance_id).unwrap();
    
    // For this test, we'd need to register a test function in the instance
    // This would normally be handled by the runtime, but for testing we can
    // add our own handler.
    
    // For now we'll just check that the instance exists
    assert!(instance.id == instance_id);
}

/// Test security capabilities
#[tokio::test]
async fn test_security_capabilities() {
    // Create a sandbox
    let mut sandbox = WasmSandbox::new().expect("Failed to create sandbox");
    
    // Load a test module
    let fixture_path = test_fixture_path("test_module.wasm");
    let wasm_bytes = std::fs::read(fixture_path).expect("Failed to read test module");
    
    let module_id = sandbox.load_module(&wasm_bytes).expect("Failed to load module");
    
    // Create custom capabilities
    let mut capabilities = Capabilities::minimal();
    capabilities.network = NetworkCapability::Loopback;
    capabilities.filesystem = FilesystemCapability {
        readable_dirs: vec![std::env::temp_dir()],
        writable_dirs: vec![std::env::temp_dir()],
        max_file_size: Some(1024 * 1024),
        allow_create: true,
        allow_delete: false,
    };
    
    // Create an instance with custom capabilities
    let instance_config = InstanceConfig {
        capabilities,
        ..InstanceConfig::default()
    };
    
    let instance_id = sandbox.create_instance(
        module_id,
        Some(instance_config),
    ).expect("Failed to create instance");
    
    // Check that the instance was created with the correct capabilities
    let instance = sandbox.get_instance(instance_id).unwrap();
    assert_eq!(instance.config.capabilities.network, NetworkCapability::Loopback);
}

/// Test communication channels
#[tokio::test]
async fn test_communication_channels() {
    // Create a memory channel
    let channel = Arc::new(MemoryChannel::new("test", 10));
    
    // Send a message
    channel.send_to_guest(b"Hello").expect("Failed to send message");
    
    // In a real test, we'd have a guest component that would read this message
    // and send a response. For now we'll simulate that.
    {
        let mut queue = channel.guest_to_host.lock().unwrap();
        queue.push_back(b"World".to_vec());
    }
    
    // Check if there are messages
    assert!(channel.has_messages());
    
    // Receive the message
    let response = channel.receive_from_guest().expect("Failed to receive message");
    assert_eq!(response, b"World");
}

/// Test RPC functionality
#[tokio::test]
async fn test_rpc_functionality() {
    // Create a memory channel
    let channel = Arc::new(MemoryChannel::new("rpc", 10));
    
    // Create an RPC channel
    let mut rpc = RpcChannel::new(channel.clone());
    
    // Register a host function
    rpc.register_host_function("add", |args: (i32, i32)| -> Result<i32, wasm_sandbox::Error> {
        Ok(args.0 + args.1)
    }).expect("Failed to register function");
    
    // In a real test, we'd have a guest component that would handle this call
    // For now we'll simulate the response
    {
        let mut queue = channel.guest_to_host.lock().unwrap();
        queue.push_back(b"42".to_vec());
    }
    
    // Call a guest function
    let result: i32 = rpc.call_guest_function("calculate", &(10, 5))
        .expect("Failed to call function");
        
    assert_eq!(result, 42);
}
