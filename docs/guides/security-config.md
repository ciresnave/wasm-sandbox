# Security Configuration Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This guide covers how to configure wasm-sandbox's security features to create safe, isolated environments for running untrusted code.

## Security Philosophy

wasm-sandbox follows a **defense-in-depth** approach with multiple layers of security:

1. **WebAssembly Isolation** - Code runs in a memory-safe WebAssembly sandbox
2. **Capability-Based Security** - Explicit permission grants for system access
3. **Resource Limits** - Prevent resource exhaustion attacks
4. **Runtime Controls** - Monitor and control execution behavior

## Quick Start - Secure Defaults

The simplest configuration provides strong security out of the box:

```rust
use wasm_sandbox::WasmSandbox;

// Secure by default - no filesystem, network, or excessive resource access
let sandbox = WasmSandbox::from_source("untrusted_code.rs").await?;
```

**Default Security Settings:**

- ❌ **No filesystem access** (read or write)
- ❌ **No network access** (inbound or outbound)
- ✅ **Memory limited** to 16MB
- ✅ **Execution timeout** of 30 seconds
- ✅ **Computational limits** via fuel metering

## Capability-Based Security

Grant specific permissions only when needed:

```rust
use wasm_sandbox::{WasmSandbox, Capabilities, FilesystemCapability, NetworkCapability};

let sandbox = WasmSandbox::builder()
    .source("file_processor.rs")
    .capabilities(Capabilities {
        filesystem: vec![
            FilesystemCapability::ReadOnly("/input".into()),
            FilesystemCapability::WriteOnly("/output".into()),
        ],
        network: vec![
            NetworkCapability::Connect("api.example.com:443".into()),
        ],
        ..Capabilities::minimal()
    })
    .build()
    .await?;
```

### Filesystem Capabilities

Control file system access with granular permissions:

```rust
use wasm_sandbox::{FilesystemCapability, PathPattern};

let capabilities = Capabilities {
    filesystem: vec![
        // Read-only access to input directory
        FilesystemCapability::ReadOnly("/app/input".into()),
        
        // Write-only access to output directory
        FilesystemCapability::WriteOnly("/app/output".into()),
        
        // Read-only access to specific configuration file
        FilesystemCapability::ReadOnly("/app/config.json".into()),
        
        // Temporary directory with read-write access
        FilesystemCapability::ReadWrite("/tmp/sandbox".into()),
        
        // Pattern-based access (glob patterns supported)
        FilesystemCapability::ReadOnly("/app/data/*.json".into()),
    ],
    ..Capabilities::minimal()
};
```

### Network Capabilities

Control network access to specific endpoints:

```rust
use wasm_sandbox::NetworkCapability;

let capabilities = Capabilities {
    network: vec![
        // Allow HTTPS connections to API
        NetworkCapability::Connect("api.example.com:443".into()),
        
        // Allow HTTP connections to specific service
        NetworkCapability::Connect("localhost:8080".into()),
        
        // Allow connections to any port on localhost
        NetworkCapability::Connect("localhost:*".into()),
        
        // Allow listening on specific port (for servers)
        NetworkCapability::Listen("0.0.0.0:3000".into()),
    ],
    ..Capabilities::minimal()
};
```

### Environment Variable Access

Control access to environment variables:

```rust
use wasm_sandbox::EnvironmentCapability;

let capabilities = Capabilities {
    environment: vec![
        // Allow reading specific environment variables
        EnvironmentCapability::Read("API_KEY".into()),
        EnvironmentCapability::Read("DATABASE_URL".into()),
        
        // Allow setting temporary variables
        EnvironmentCapability::Write("TEMP_DIR".into()),
    ],
    ..Capabilities::minimal()
};
```

## Resource Limits

Prevent resource exhaustion with configurable limits:

```rust
use wasm_sandbox::ResourceLimits;
use std::time::Duration;

let sandbox = WasmSandbox::builder()
    .source("resource_intensive.rs")
    .resource_limits(ResourceLimits {
        // Memory limits
        memory_bytes: Some(128 * 1024 * 1024), // 128MB max memory
        
        // Execution limits
        execution_timeout: Some(Duration::from_secs(60)), // 60 second timeout
        max_fuel: Some(10_000_000), // Computational limit
        
        // I/O limits
        max_file_size: Some(10 * 1024 * 1024), // 10MB file size limit
        max_open_files: Some(100), // Maximum 100 open files
        
        // Network limits
        max_connections: Some(10), // Maximum 10 concurrent connections
        network_timeout: Some(Duration::from_secs(30)), // 30 second network timeout
        
        ..ResourceLimits::default()
    })
    .build()
    .await?;
```

### Understanding Fuel

Fuel is a mechanism to limit computational resources:

```rust
// High fuel limit for complex operations
.max_fuel(Some(50_000_000))  // ~50M instructions

// Low fuel limit for simple operations
.max_fuel(Some(1_000_000))   // ~1M instructions

// No fuel limit (use with extreme caution)
.max_fuel(None)
```

**Fuel Guidelines:**

- **Simple calculations**: 1,000 - 10,000 fuel
- **File processing**: 100,000 - 1,000,000 fuel
- **Complex algorithms**: 1,000,000 - 10,000,000 fuel
- **Machine learning**: 10,000,000+ fuel

## Security Profiles

Use predefined security profiles for common scenarios:

### Minimal Security (Maximum Isolation)

```rust
use wasm_sandbox::SecurityProfile;

let sandbox = WasmSandbox::builder()
    .source("untrusted_code.rs")
    .security_profile(SecurityProfile::Minimal)
    .build()
    .await?;
```

**Minimal Profile:**

- No filesystem access
- No network access
- No environment variables
- 16MB memory limit
- 30 second timeout
- 1M fuel limit

### File Processing Profile

```rust
let sandbox = WasmSandbox::builder()
    .source("file_processor.rs")
    .security_profile(SecurityProfile::FileProcessing {
        input_dir: "/input".into(),
        output_dir: "/output".into(),
        max_file_size: 100 * 1024 * 1024, // 100MB
    })
    .build()
    .await?;
```

### Web Service Profile

```rust
let sandbox = WasmSandbox::builder()
    .source("web_service.rs")
    .security_profile(SecurityProfile::WebService {
        allowed_hosts: vec!["api.example.com".into(), "database.local".into()],
        listen_port: 3000,
    })
    .build()
    .await?;
```

## Runtime Security Monitoring

Monitor security events during execution:

```rust
use wasm_sandbox::{SecurityEvent, SecurityViolation};

// Enable security monitoring
let sandbox = WasmSandbox::builder()
    .source("monitored_code.rs")
    .enable_security_monitoring(true)
    .security_callback(|event: SecurityEvent| {
        match event {
            SecurityEvent::Violation(violation) => {
                match violation {
                    SecurityViolation::UnauthorizedFileAccess { path } => {
                        eprintln!("🚨 Unauthorized file access: {}", path);
                    }
                    SecurityViolation::UnauthorizedNetworkAccess { address } => {
                        eprintln!("🚨 Unauthorized network access: {}", address);
                    }
                    SecurityViolation::ResourceLimitExceeded { resource, limit } => {
                        eprintln!("🚨 Resource limit exceeded: {} > {}", resource, limit);
                    }
                }
            }
            SecurityEvent::CapabilityUsed { capability } => {
                println!("ℹ️ Capability used: {:?}", capability);
            }
        }
    })
    .build()
    .await?;
```

## Security Auditing

Enable comprehensive security auditing:

```rust
use wasm_sandbox::AuditConfig;

let sandbox = WasmSandbox::builder()
    .source("audited_code.rs")
    .audit_config(AuditConfig {
        log_all_operations: true,
        log_resource_usage: true,
        log_capability_usage: true,
        audit_file: Some("/var/log/wasm-sandbox-audit.log".into()),
    })
    .build()
    .await?;
```

**Audit Log Format:**

```json
{
  "timestamp": "2025-07-12T10:30:00Z",
  "sandbox_id": "sb-123456",
  "event_type": "filesystem_access",
  "details": {
    "operation": "read",
    "path": "/input/data.json",
    "size": 1024,
    "allowed": true
  }
}
```

## Multi-Tenant Security

Isolate multiple tenants safely:

```rust
use wasm_sandbox::{TenantConfig, TenantId};

// Create tenant-isolated sandboxes
let tenant_a_sandbox = WasmSandbox::builder()
    .source("tenant_code.rs")
    .tenant_config(TenantConfig {
        tenant_id: TenantId::new("tenant-a"),
        isolation_level: IsolationLevel::Strong,
        resource_quota: ResourceQuota::new()
            .memory_mb(64)
            .cpu_percent(25)
            .disk_mb(100),
    })
    .build()
    .await?;
```

## Security Best Practices

### 1. Principle of Least Privilege

```rust
// ❌ Don't grant broad access
let bad_capabilities = Capabilities {
    filesystem: vec![FilesystemCapability::ReadWrite("/".into())], // Too broad!
    ..Capabilities::minimal()
};

// ✅ Grant specific, minimal access
let good_capabilities = Capabilities {
    filesystem: vec![
        FilesystemCapability::ReadOnly("/app/input/specific-file.json".into()),
        FilesystemCapability::WriteOnly("/app/output/result.json".into()),
    ],
    ..Capabilities::minimal()
};
```

### 2. Always Set Resource Limits

```rust
// ❌ Don't leave resources unlimited
let risky_sandbox = WasmSandbox::builder()
    .source("untrusted.rs")
    .build().await?; // Uses defaults, but better to be explicit

// ✅ Explicitly set appropriate limits
let safe_sandbox = WasmSandbox::builder()
    .source("untrusted.rs")
    .memory_limit(32 * 1024 * 1024)        // 32MB
    .timeout_duration(Duration::from_secs(10)) // 10 seconds
    .max_fuel(Some(1_000_000))             // 1M instructions
    .build().await?;
```

### 3. Validate Input and Output

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ValidatedInput {
    #[serde(deserialize_with = "validate_string_length")]
    data: String,
    
    #[serde(deserialize_with = "validate_positive_number")]
    count: u32,
}

// Always validate data going into and coming out of the sandbox
fn validate_string_length<'de, D>(deserializer: D) -> Result<String, D::Error> 
where 
    D: serde::Deserializer<'de> 
{
    let s = String::deserialize(deserializer)?;
    if s.len() > 1000 {
        return Err(serde::de::Error::custom("String too long"));
    }
    Ok(s)
}
```

### 4. Handle Security Violations Gracefully

```rust
match sandbox.call("process_data", &input).await {
    Ok(result) => {
        // Success - process result
        handle_success(result).await
    }
    Err(Error::Security(SecurityViolation::UnauthorizedFileAccess { path })) => {
        // Log security incident
        security_logger.log_incident(&path).await;
        
        // Return safe error to user
        Err("Operation not permitted".into())
    }
    Err(Error::ResourceLimit { kind, used, limit }) => {
        // Log resource exhaustion
        metrics.record_resource_limit_hit(&kind, used, limit).await;
        
        // Possibly retry with different limits or reject
        Err("Resource limit exceeded".into())
    }
    Err(e) => {
        // Handle other errors
        Err(e.into())
    }
}
```

## Security Checklist

Before deploying to production:

### ✅ Capability Configuration

- [ ] Filesystem access limited to specific paths
- [ ] Network access limited to required endpoints
- [ ] Environment variables limited to necessary ones
- [ ] No unnecessary capabilities granted

### ✅ Resource Limits

- [ ] Memory limits appropriate for workload
- [ ] Execution timeouts prevent infinite loops
- [ ] File size limits prevent DoS attacks
- [ ] Network timeouts configured

### ✅ Monitoring and Auditing

- [ ] Security monitoring enabled
- [ ] Audit logging configured
- [ ] Violation handling implemented
- [ ] Metrics collection enabled

### ✅ Testing

- [ ] Security boundaries tested
- [ ] Resource limits tested
- [ ] Violation scenarios tested
- [ ] Performance under limits tested

## Common Security Patterns

### Pattern 1: File Processing Service

```rust
async fn create_file_processor_sandbox(
    input_dir: &str,
    output_dir: &str,
) -> Result<WasmSandbox> {
    WasmSandbox::builder()
        .source("file_processor.rs")
        .capabilities(Capabilities {
            filesystem: vec![
                FilesystemCapability::ReadOnly(input_dir.into()),
                FilesystemCapability::WriteOnly(output_dir.into()),
            ],
            ..Capabilities::minimal()
        })
        .resource_limits(ResourceLimits {
            memory_bytes: Some(128 * 1024 * 1024), // 128MB
            execution_timeout: Some(Duration::from_secs(300)), // 5 minutes
            max_file_size: Some(50 * 1024 * 1024), // 50MB files
            max_open_files: Some(50),
            ..ResourceLimits::default()
        })
        .build()
        .await
}
```

### Pattern 2: API Client Sandbox

```rust
async fn create_api_client_sandbox(allowed_hosts: Vec<String>) -> Result<WasmSandbox> {
    let network_capabilities = allowed_hosts
        .into_iter()
        .map(|host| NetworkCapability::Connect(format!("{}:443", host).into()))
        .collect();

    WasmSandbox::builder()
        .source("api_client.rs")
        .capabilities(Capabilities {
            network: network_capabilities,
            environment: vec![
                EnvironmentCapability::Read("API_KEY".into()),
            ],
            ..Capabilities::minimal()
        })
        .resource_limits(ResourceLimits {
            memory_bytes: Some(64 * 1024 * 1024), // 64MB
            execution_timeout: Some(Duration::from_secs(60)), // 1 minute
            network_timeout: Some(Duration::from_secs(30)), // 30 second requests
            max_connections: Some(5),
            ..ResourceLimits::default()
        })
        .build()
        .await
}
```

## Troubleshooting Security Issues

### Common Security Errors

**Unauthorized File Access:**

```
Error: Security violation: Unauthorized file access to '/etc/passwd'
```

**Solution:** Add appropriate filesystem capability or review code for unauthorized access.

**Resource Limit Exceeded:**

```
Error: Memory limit exceeded: used 134217728, limit 67108864
```

**Solution:** Increase memory limit or optimize code to use less memory.

**Network Access Denied:**

```
Error: Security violation: Unauthorized network access to 'malicious.com:80'
```

**Solution:** Review code for unauthorized network calls or add necessary network capabilities.

## Security Updates

Stay informed about security updates:

- **Subscribe to security advisories** in the GitHub repository
- **Review changelog** for security-related changes
- **Update regularly** to get latest security patches
- **Test security boundaries** after updates

---

**Remember:** Security is a process, not a destination. Regularly review and update your security configuration as your application evolves.

## Next Steps

- **[Resource Management Guide](resource-management.md)** - Optimize resource usage
- **[Production Deployment](production.md)** - Deploy securely to production
- **[Error Handling Guide](error-handling.md)** - Handle security violations gracefully
- **[Plugin Development](plugin-development.md)** - Build secure plugin systems
