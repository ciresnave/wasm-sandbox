// Basic Usage Example - Simple and functional demonstration
//
// This example demonstrates the basic usage of wasm-sandbox with actual API calls
// that work with the current codebase structure.

use wasm_sandbox::{WasmSandbox, InstanceConfig, InstanceId};
use wasm_sandbox::security::{
    Capabilities, ResourceLimits, FilesystemCapability, 
    NetworkCapability, EnvironmentCapability, ProcessCapability, 
    TimeCapability, RandomCapability
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Basic wasm-sandbox Usage Example");
    println!("This demonstrates core functionality with real API calls");
    
    // Step 1: Create a simple WASM module for testing
    let wasm_bytes = create_test_module()?;
    println!("📦 Created test WASM module ({} bytes)", wasm_bytes.len());
    
    // Step 2: Configure security capabilities
    let capabilities = Capabilities::custom(
        // Filesystem - no access by default
        FilesystemCapability::None,
        
        // Network - no access by default  
        NetworkCapability::None,
        
        // Environment - no access by default
        EnvironmentCapability::None,
        
        // Process - no spawning allowed
        ProcessCapability::None,
        
        // Time - basic time access
        TimeCapability::Basic,
        
        // Random - no cryptographic access
        RandomCapability::None,
    );
    
    // Step 3: Configure resource limits
    let resource_limits = ResourceLimits::default();
    
    let instance_config = InstanceConfig {
        capabilities,
        resource_limits,
        startup_timeout_ms: 5000,
        enable_debug: false,
    };
    
    // Step 4: Create sandbox and load module
    let mut sandbox = WasmSandbox::new()?;
    let module_id = sandbox.load_module(&wasm_bytes)?;
    let instance_id = sandbox.create_instance(module_id, Some(instance_config))?;
    
    println!("🛡️  Sandbox created with minimal security configuration");
    
    // Step 5: Test basic function calls
    println!("\n🧮 Testing Basic Function Calls:");
    
    // Test simple math function
    let result: i32 = sandbox.call_function(instance_id, "add", (5, 7)).await?;
    println!("   ✅ add(5, 7) = {}", result);
    
    // Test multiplication
    let result: i32 = sandbox.call_function(instance_id, "multiply", (6, 8)).await?;
    println!("   ✅ multiply(6, 8) = {}", result);
    
    // Test a function that returns a boolean
    let result: bool = sandbox.call_function(instance_id, "is_positive", 42i32).await?;
    println!("   ✅ is_positive(42) = {}", result);
    
    let result: bool = sandbox.call_function(instance_id, "is_positive", -5i32).await?;
    println!("   ✅ is_positive(-5) = {}", result);
    
    // Step 6: Test JSON serialization/deserialization
    println!("\n📄 Testing JSON Function Calls:");
    
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct Person {
        name: String,
        age: u32,
    }
    
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };
    
    let result: Person = sandbox.call_function(instance_id, "process_person", person).await?;
    println!("   ✅ process_person result: {:?}", result);
    
    // Step 7: Test error handling
    println!("\n🚨 Testing Error Handling:");
    
    // Try calling a function that doesn't exist
    match sandbox.call_function::<(), ()>(instance_id, "non_existent_function", ()).await {
        Ok(_) => println!("   ❌ Unexpected success"),
        Err(e) => println!("   ✅ Expected error: {}", e),
    }
    
    // Step 8: Demonstrate different capability configurations
    println!("\n🔒 Testing Different Security Configurations:");
    
    // Create a more permissive configuration
    let permissive_capabilities = Capabilities::custom(
        FilesystemCapability::None, // Still no filesystem access
        NetworkCapability::Loopback, // Allow localhost connections
        EnvironmentCapability::None,
        ProcessCapability::None,
        TimeCapability::Full, // Full time access
        RandomCapability::Deterministic, // Deterministic random
    );
    
    let permissive_config = InstanceConfig {
        capabilities: permissive_capabilities,
        resource_limits: ResourceLimits::default(),
        startup_timeout_ms: 5000,
        enable_debug: true, // Enable debugging
    };
    
    let permissive_instance = sandbox.create_instance(module_id, Some(permissive_config))?;
    println!("   ✅ Created permissive instance: {}", permissive_instance);
    
    // Test the same function with different security context
    let result: i32 = sandbox.call_function(permissive_instance, "add", (10, 20)).await?;
    println!("   ✅ Permissive instance add(10, 20) = {}", result);
    
    // Step 9: Cleanup
    sandbox.remove_instance(instance_id);
    sandbox.remove_instance(permissive_instance);
    println!("\n🧹 Cleaned up instances");
    
    println!("\n✅ Basic usage example completed successfully!");
    
    Ok(())
}

// Create a simple test WASM module
fn create_test_module() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let wasm_source = r#"
        (module
            (memory (export "memory") 1)
            
            ;; Add two numbers
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
            
            ;; Multiply two numbers
            (func (export "multiply") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.mul
            )
            
            ;; Check if a number is positive
            (func (export "is_positive") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.gt_s
            )
            
            ;; Process person (mock - just returns 1 for success)
            (func (export "process_person") (result i32)
                i32.const 1
            )
        )
    "#;
    
    // Convert WAT to WASM bytes
    Ok(wat::parse_str(wasm_source)?)
}

// Enhanced configuration helpers
impl InstanceConfig {
    /// Create a minimal security configuration
    pub fn minimal_security() -> Self {
        Self {
            capabilities: Capabilities::custom(
                FilesystemCapability::None,
                NetworkCapability::None,
                EnvironmentCapability::None,
                ProcessCapability::None,
                TimeCapability::Basic,
                RandomCapability::None,
            ),
            resource_limits: ResourceLimits::default(),
            startup_timeout_ms: 5000,
            enable_debug: false,
        }
    }
    
    /// Create a development-friendly configuration
    pub fn development() -> Self {
        Self {
            capabilities: Capabilities::custom(
                FilesystemCapability::None,
                NetworkCapability::Loopback,
                EnvironmentCapability::None,
                ProcessCapability::None,
                TimeCapability::Full,
                RandomCapability::Deterministic,
            ),
            resource_limits: ResourceLimits::default(),
            startup_timeout_ms: 10000, // Longer timeout for development
            enable_debug: true,
        }
    }
}
