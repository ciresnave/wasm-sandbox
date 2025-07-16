# Multi-Tenant Isolation Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This guide covers implementing secure multi-tenant isolation in wasm-sandbox deployments, ensuring complete separation between different tenants' code and data.

## Overview

Multi-tenant isolation provides:

- **Complete Separation** - Zero data leakage between tenants
- **Resource Isolation** - Fair resource allocation per tenant
- **Security Boundaries** - Independent security contexts
- **Operational Isolation** - Separate monitoring and management

## Quick Start

```rust
use wasm_sandbox::{MultiTenantSandbox, TenantConfig, IsolationLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = MultiTenantSandbox::builder()
        .isolation_level(IsolationLevel::Strong)
        .enable_tenant_monitoring(true)
        .resource_quotas_per_tenant(ResourceQuotas {
            memory: 128 * 1024 * 1024,  // 128MB per tenant
            cpu_cores: 1.0,
            storage: 1024 * 1024 * 1024, // 1GB per tenant
        })
        .build()
        .await?;

    // Create isolated tenant environments
    let tenant_a = sandbox.create_tenant("tenant-a", TenantConfig {
        isolation_level: IsolationLevel::Strong,
        resource_limits: ResourceLimits::default(),
        security_policy: SecurityPolicy::strict(),
    }).await?;

    let tenant_b = sandbox.create_tenant("tenant-b", TenantConfig {
        isolation_level: IsolationLevel::Strong,
        resource_limits: ResourceLimits::default(),
        security_policy: SecurityPolicy::strict(),
    }).await?;

    // Execute code in isolated tenants
    let result_a = tenant_a.execute("process_data", &input_a).await?;
    let result_b = tenant_b.execute("process_data", &input_b).await?;

    Ok(())
}
```

## Isolation Levels

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum IsolationLevel {
    /// Basic process-level isolation
    Basic,
    /// Container-based isolation
    Container,
    /// VM-level isolation (strongest)
    VirtualMachine,
    /// Strong isolation with multiple layers
    Strong,
}

impl IsolationLevel {
    pub fn security_features(&self) -> Vec<SecurityFeature> {
        match self {
            IsolationLevel::Basic => vec![
                SecurityFeature::ProcessSeparation,
                SecurityFeature::MemoryIsolation,
            ],
            IsolationLevel::Container => vec![
                SecurityFeature::ProcessSeparation,
                SecurityFeature::MemoryIsolation,
                SecurityFeature::FilesystemIsolation,
                SecurityFeature::NetworkIsolation,
                SecurityFeature::Namespaces,
            ],
            IsolationLevel::VirtualMachine => vec![
                SecurityFeature::HardwareVirtualization,
                SecurityFeature::KernelIsolation,
                SecurityFeature::CompleteResourceIsolation,
            ],
            IsolationLevel::Strong => vec![
                SecurityFeature::ProcessSeparation,
                SecurityFeature::MemoryIsolation,
                SecurityFeature::FilesystemIsolation,
                SecurityFeature::NetworkIsolation,
                SecurityFeature::Namespaces,
                SecurityFeature::Seccomp,
                SecurityFeature::AppArmor,
                SecurityFeature::CapabilityDropping,
            ],
        }
    }
}
```

## Tenant Management

```rust
pub struct TenantManager {
    tenants: HashMap<TenantId, TenantContext>,
    isolation_config: IsolationConfig,
    resource_allocator: ResourceAllocator,
    security_monitor: SecurityMonitor,
}

impl TenantManager {
    pub async fn create_tenant(&mut self, tenant_id: TenantId, config: TenantConfig) -> Result<TenantHandle, TenantError> {
        // Validate tenant doesn't exist
        if self.tenants.contains_key(&tenant_id) {
            return Err(TenantError::TenantAlreadyExists(tenant_id));
        }

        // Allocate resources
        let resources = self.resource_allocator.allocate(&config.resource_limits).await?;

        // Create isolated environment
        let isolation = self.create_isolation_environment(&tenant_id, &config).await?;

        // Set up security context
        let security_context = SecurityContext::new(tenant_id.clone(), config.security_policy);

        // Create tenant context
        let tenant_context = TenantContext {
            id: tenant_id.clone(),
            config,
            resources,
            isolation,
            security_context,
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            status: TenantStatus::Active,
        };

        // Store tenant
        self.tenants.insert(tenant_id.clone(), tenant_context);

        // Start monitoring
        self.security_monitor.start_tenant_monitoring(tenant_id.clone()).await?;

        Ok(TenantHandle::new(tenant_id))
    }

    async fn create_isolation_environment(&self, tenant_id: &TenantId, config: &TenantConfig) -> Result<IsolationEnvironment, IsolationError> {
        match config.isolation_level {
            IsolationLevel::Container => {
                self.create_container_isolation(tenant_id, config).await
            }
            IsolationLevel::VirtualMachine => {
                self.create_vm_isolation(tenant_id, config).await
            }
            IsolationLevel::Strong => {
                self.create_strong_isolation(tenant_id, config).await
            }
            _ => {
                self.create_basic_isolation(tenant_id, config).await
            }
        }
    }
}
```

## Resource Isolation

```rust
pub struct ResourceAllocator {
    total_resources: SystemResources,
    allocated_resources: HashMap<TenantId, AllocatedResources>,
    allocation_strategy: AllocationStrategy,
}

#[derive(Debug, Clone)]
pub struct ResourceQuotas {
    pub memory: u64,
    pub cpu_cores: f64,
    pub storage: u64,
    pub network_bandwidth: Option<u64>,
    pub file_descriptors: Option<u32>,
    pub processes: Option<u32>,
}

impl ResourceAllocator {
    pub async fn allocate(&mut self, limits: &ResourceLimits) -> Result<AllocatedResources, AllocationError> {
        // Check if resources are available
        self.validate_resource_availability(limits)?;

        // Reserve resources
        let allocation = AllocatedResources {
            memory: self.allocate_memory(limits.memory)?,
            cpu: self.allocate_cpu(limits.cpu_cores)?,
            storage: self.allocate_storage(limits.storage)?,
            network: self.allocate_network(limits.network_bandwidth)?,
        };

        Ok(allocation)
    }

    fn allocate_memory(&mut self, requested: u64) -> Result<MemoryAllocation, AllocationError> {
        // Use memory namespaces and control groups
        let cgroup_path = format!("/sys/fs/cgroup/memory/tenant-{}", uuid::Uuid::new_v4());
        
        // Set memory limit
        std::fs::write(format!("{}/memory.limit_in_bytes", cgroup_path), requested.to_string())?;
        
        // Enable OOM killing
        std::fs::write(format!("{}/memory.oom_control", cgroup_path), "0")?;

        Ok(MemoryAllocation {
            limit: requested,
            cgroup_path,
        })
    }
}
```

## Security Boundaries

```rust
pub struct SecurityBoundary {
    tenant_id: TenantId,
    security_context: SecurityContext,
    access_controls: AccessControls,
    audit_logger: AuditLogger,
}

impl SecurityBoundary {
    pub async fn enforce_access(&self, operation: &Operation) -> Result<(), SecurityViolation> {
        // Check tenant permissions
        if !self.access_controls.is_allowed(&self.tenant_id, operation) {
            self.audit_logger.log_access_denied(&self.tenant_id, operation).await;
            return Err(SecurityViolation::AccessDenied);
        }

        // Check cross-tenant access
        if let Some(target_tenant) = operation.target_tenant() {
            if target_tenant != self.tenant_id {
                self.audit_logger.log_cross_tenant_access_attempt(&self.tenant_id, &target_tenant).await;
                return Err(SecurityViolation::CrossTenantAccess);
            }
        }

        Ok(())
    }
}
```

## Monitoring and Management

```rust
pub struct TenantMonitor {
    metrics_collector: MetricsCollector,
    health_checker: HealthChecker,
    alert_manager: AlertManager,
}

impl TenantMonitor {
    pub async fn monitor_tenant(&self, tenant_id: &TenantId) -> TenantMetrics {
        TenantMetrics {
            resource_usage: self.collect_resource_usage(tenant_id).await,
            security_events: self.collect_security_events(tenant_id).await,
            performance_metrics: self.collect_performance_metrics(tenant_id).await,
            health_status: self.check_tenant_health(tenant_id).await,
        }
    }
}
```

## Next Steps

Continue with:

- **[Streaming Execution Guide](streaming-execution.md)** - Handle large data processing
- **[Development Tools Integration](development-tools.md)** - IDE and tooling support

---

**Multi-Tenant Excellence:** This guide provides enterprise-grade isolation for secure multi-tenant deployments.
