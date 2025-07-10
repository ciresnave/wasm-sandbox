//! Integration test for basic sandbox functionality

use wasm_sandbox::{
    WasmSandbox, InstanceConfig, 
    security::{Capabilities, ResourceLimits},
};

// WASM module for testing - simple add function
const TEST_MODULE: &[u8] = include_bytes!("../fixtures/test_module.wasm");

#[test]
fn test_sandbox_creation() {
    // Create a sandbox with default configuration
    let result = WasmSandbox::new();
    assert!(result.is_ok());
}

#[test]
fn test_module_loading() {
    // Create a sandbox
    let sandbox = WasmSandbox::new().expect("Failed to create sandbox");
    
    // Load a module
    let module_id = sandbox.load_module(TEST_MODULE).expect("Failed to load module");
    
    // Check that we can get the module
    let module = sandbox.runtime().get_module(module_id);
    assert!(module.is_ok());
}

#[test]
fn test_instance_creation() {
    // Create a sandbox
    let mut sandbox = WasmSandbox::new().expect("Failed to create sandbox");
    
    // Load a module
    let module_id = sandbox.load_module(TEST_MODULE).expect("Failed to load module");
    
    // Create an instance with default configuration
    let instance_id = sandbox.create_instance(module_id, None).expect("Failed to create instance");
    
    // Check that we can get the instance
    let instance = sandbox.get_instance(instance_id);
    assert!(instance.is_some());
}

#[tokio::test]
async fn test_function_call() {
    // Create a sandbox
    let mut sandbox = WasmSandbox::new().expect("Failed to create sandbox");
    
    // Load a module
    let module_id = sandbox.load_module(TEST_MODULE).expect("Failed to load module");
    
    // Create an instance with minimal capabilities
    let instance_config = InstanceConfig {
        capabilities: Capabilities::minimal(),
        ..Default::default()
    };
    
    let instance_id = sandbox.create_instance(module_id, Some(instance_config))
        .expect("Failed to create instance");
    
    // Call the add function using async API
    let result: i32 = sandbox.call_function(instance_id, "add", &(5, 7))
        .await
        .expect("Failed to call function");
    
    // Check the result
    assert_eq!(result, 12);
}

#[tokio::test]
async fn test_resource_limits() {
    // Create a sandbox
    let mut sandbox = WasmSandbox::new().expect("Failed to create sandbox");
    
    // Load a module
    let module_id = sandbox.load_module(TEST_MODULE).expect("Failed to load module");
    
    // Create an instance with strict resource limits
    let mut resource_limits = ResourceLimits::default();
    resource_limits.memory.max_memory_pages = 16; // 1MB memory limit (64KB per page)
    
    let instance_config = InstanceConfig {
        resource_limits,
        ..Default::default()
    };
    
    let instance_id = sandbox.create_instance(module_id, Some(instance_config))
        .expect("Failed to create instance");
    
    // Call a simple function that should work within limits
    let result: i32 = sandbox.call_function(instance_id, "add", &(5, 7))
        .await
        .expect("Failed to call function");
    
    // Check the result
    assert_eq!(result, 12);
}

#[test]
fn test_sandbox_cleanup() {
    // Create a sandbox
    let mut sandbox = WasmSandbox::new().expect("Failed to create sandbox");
    
    // Load a module
    let module_id = sandbox.load_module(TEST_MODULE).expect("Failed to load module");
    
    // Create an instance
    let instance_id = sandbox.create_instance(module_id, None).expect("Failed to create instance");
    
    // Remove the instance
    let instance = sandbox.remove_instance(instance_id);
    assert!(instance.is_some());
    
    // Check that the instance is gone
    let instance = sandbox.get_instance(instance_id);
    assert!(instance.is_none());
}
