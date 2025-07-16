# Security Model Design

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This design document outlines the comprehensive security model for wasm-sandbox, including capability-based security, sandboxing mechanisms, threat modeling, and defense-in-depth strategies.

## Security Philosophy

The wasm-sandbox security model is built on these core principles:

1. **Principle of Least Privilege** - Grant minimal necessary permissions
2. **Defense in Depth** - Multiple layers of security controls
3. **Capability-Based Security** - Explicit capability delegation
4. **Fail-Safe Defaults** - Secure by default configuration
5. **Complete Mediation** - Every access is checked
6. **Audit Trail** - Comprehensive security logging

## Threat Model

### Threat Actors

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ThreatActor {
    /// Malicious untrusted code running in sandbox
    MaliciousGuest {
        intent: AttackIntent,
        sophistication: SophisticationLevel,
        persistence: bool,
    },
    
    /// Compromised host application
    CompromisedHost {
        compromise_vector: CompromiseVector,
        privileges: HostPrivileges,
    },
    
    /// External network attacker
    NetworkAttacker {
        access_level: NetworkAccess,
        target: NetworkTarget,
    },
    
    /// Malicious or careless administrator
    InsiderThreat {
        privilege_level: AdminPrivileges,
        access_scope: AccessScope,
    },
    
    /// Supply chain compromise
    SupplyChainAttack {
        compromise_point: SupplyChainVector,
        affected_components: Vec<Component>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttackIntent {
    DataExfiltration,
    SystemCompromise,
    DenialOfService,
    PrivilegeEscalation,
    PersistentAccess,
    ResourceAbuse,
    InformationDisclosure,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SophisticationLevel {
    Script,      // Script kiddie attacks
    Intermediate, // Moderately skilled attacker
    Advanced,    // Nation-state level
    Expert,      // Zero-day, custom exploits
}
```

### Attack Vectors

```rust
#[derive(Debug, Clone)]
pub struct AttackVector {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: RiskSeverity,
    pub likelihood: Probability,
    pub attack_surface: AttackSurface,
    pub prerequisites: Vec<String>,
    pub mitigations: Vec<Mitigation>,
}

#[derive(Debug, Clone)]
pub enum AttackSurface {
    // WebAssembly-specific attacks
    WasmBytecodeTampering,
    WasmMemoryCorruption,
    WasmStackOverflow,
    WasmTableOverflow,
    WasmFunctionPointerCorruption,
    
    // Host-guest communication attacks
    ChannelPoisoning,
    SerializationAttacks,
    MemoryChannelExploits,
    RpcProtocolAbuse,
    
    // Resource-based attacks
    MemoryExhaustion,
    CpuExhaustion,
    DiskSpaceExhaustion,
    NetworkFlood,
    FileDescriptorExhaustion,
    
    // Security boundary attacks
    SandboxEscape,
    CapabilityForging,
    PermissionEscalation,
    TimeOfCheckTimeOfUse,
    
    // Infrastructure attacks
    ContainerEscape,
    ProcessInjection,
    SystemCallAbuse,
    KernelExploits,
    
    // Application-level attacks
    LogicBombs,
    ConfigurationTampering,
    DependencyConfusion,
    SupplyChainPoisoning,
}

impl AttackVector {
    pub fn sandbox_escape() -> Self {
        Self {
            id: "AV001".to_string(),
            name: "Sandbox Escape".to_string(),
            description: "Malicious code attempts to break out of the WebAssembly sandbox and access host system resources".to_string(),
            severity: RiskSeverity::Critical,
            likelihood: Probability::Low,
            attack_surface: AttackSurface::SandboxEscape,
            prerequisites: vec![
                "Arbitrary code execution in sandbox".to_string(),
                "Knowledge of sandbox implementation".to_string(),
                "Runtime vulnerability".to_string(),
            ],
            mitigations: vec![
                Mitigation::runtime_isolation(),
                Mitigation::capability_restrictions(),
                Mitigation::memory_protection(),
                Mitigation::system_call_filtering(),
            ],
        }
    }
    
    pub fn memory_corruption() -> Self {
        Self {
            id: "AV002".to_string(),
            name: "Memory Corruption".to_string(),
            description: "Exploit memory safety vulnerabilities to corrupt host memory or achieve code execution".to_string(),
            severity: RiskSeverity::High,
            likelihood: Probability::Medium,
            attack_surface: AttackSurface::WasmMemoryCorruption,
            prerequisites: vec![
                "Memory safety bug in runtime".to_string(),
                "Ability to trigger the bug".to_string(),
                "Knowledge of memory layout".to_string(),
            ],
            mitigations: vec![
                Mitigation::memory_safe_runtime(),
                Mitigation::address_space_randomization(),
                Mitigation::stack_canaries(),
                Mitigation::control_flow_integrity(),
            ],
        }
    }
    
    pub fn resource_exhaustion() -> Self {
        Self {
            id: "AV003".to_string(),
            name: "Resource Exhaustion".to_string(),
            description: "Consume excessive system resources to cause denial of service".to_string(),
            severity: RiskSeverity::Medium,
            likelihood: Probability::High,
            attack_surface: AttackSurface::MemoryExhaustion,
            prerequisites: vec![
                "Ability to execute code in sandbox".to_string(),
                "Insufficient resource limits".to_string(),
            ],
            mitigations: vec![
                Mitigation::resource_limits(),
                Mitigation::resource_monitoring(),
                Mitigation::fair_scheduling(),
                Mitigation::circuit_breakers(),
            ],
        }
    }
}
```

## Capability-Based Security Model

### Capability System

```rust
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Core capability that grants specific permissions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability {
    pub id: CapabilityId,
    pub permissions: HashSet<Permission>,
    pub constraints: CapabilityConstraints,
    pub delegation_rights: DelegationRights,
    pub audit_level: AuditLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // File system permissions
    FileRead { path_pattern: PathPattern },
    FileWrite { path_pattern: PathPattern },
    FileExecute { path_pattern: PathPattern },
    DirectoryList { path_pattern: PathPattern },
    DirectoryCreate { path_pattern: PathPattern },
    
    // Network permissions
    NetworkConnect { 
        host_pattern: HostPattern, 
        port_range: PortRange,
        protocol: NetworkProtocol,
    },
    NetworkListen { 
        interface: NetworkInterface, 
        port_range: PortRange,
        protocol: NetworkProtocol,
    },
    NetworkRaw,
    
    // System permissions
    SystemInfo,
    ProcessSpawn { executable_pattern: PathPattern },
    EnvironmentRead { variable_pattern: VariablePattern },
    EnvironmentWrite { variable_pattern: VariablePattern },
    
    // Resource permissions
    MemoryAllocate { max_bytes: u64 },
    CpuTime { max_milliseconds: u64 },
    DiskSpace { max_bytes: u64 },
    
    // Host communication permissions
    HostFunctionCall { function_pattern: FunctionPattern },
    MemoryShare { size_limit: u64 },
    EventEmit { event_pattern: EventPattern },
    
    // Security permissions
    CryptographicOperations,
    RandomNumberGeneration,
    TimeAccess,
    
    // Administrative permissions
    ConfigurationRead,
    ConfigurationWrite,
    LogAccess,
    MetricsAccess,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityConstraints {
    /// Time-based constraints
    pub time_constraints: Option<TimeConstraints>,
    /// Usage-based constraints
    pub usage_constraints: Option<UsageConstraints>,
    /// Context-based constraints
    pub context_constraints: Option<ContextConstraints>,
    /// Rate limiting
    pub rate_limits: Option<RateLimits>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeConstraints {
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
    pub time_of_day_restrictions: Option<TimeOfDayRestrictions>,
    pub day_of_week_restrictions: Option<HashSet<chrono::Weekday>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageConstraints {
    pub max_invocations: Option<u64>,
    pub max_data_transfer: Option<u64>,
    pub max_file_operations: Option<u64>,
    pub max_network_connections: Option<u64>,
}

/// Capability delegation system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DelegationRights {
    pub can_delegate: bool,
    pub max_delegation_depth: Option<u32>,
    pub delegatable_permissions: HashSet<Permission>,
    pub delegation_constraints: Option<CapabilityConstraints>,
}

impl Capability {
    /// Create a minimal capability for read-only file access
    pub fn file_read_only(path_pattern: PathPattern) -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::FileRead { path_pattern });
        
        Self {
            id: CapabilityId(uuid::Uuid::new_v4().to_string()),
            permissions,
            constraints: CapabilityConstraints::default(),
            delegation_rights: DelegationRights {
                can_delegate: false,
                max_delegation_depth: None,
                delegatable_permissions: HashSet::new(),
                delegation_constraints: None,
            },
            audit_level: AuditLevel::Standard,
        }
    }
    
    /// Create a network capability for HTTP client access
    pub fn http_client(allowed_hosts: Vec<String>) -> Self {
        let mut permissions = HashSet::new();
        
        for host in allowed_hosts {
            permissions.insert(Permission::NetworkConnect {
                host_pattern: HostPattern::Exact(host),
                port_range: PortRange::Single(443), // HTTPS
                protocol: NetworkProtocol::Tcp,
            });
            permissions.insert(Permission::NetworkConnect {
                host_pattern: HostPattern::Exact("*".to_string()),
                port_range: PortRange::Single(80), // HTTP
                protocol: NetworkProtocol::Tcp,
            });
        }
        
        Self {
            id: CapabilityId(uuid::Uuid::new_v4().to_string()),
            permissions,
            constraints: CapabilityConstraints {
                rate_limits: Some(RateLimits {
                    requests_per_second: Some(10.0),
                    bytes_per_second: Some(1024 * 1024), // 1MB/s
                    concurrent_connections: Some(5),
                }),
                ..Default::default()
            },
            delegation_rights: DelegationRights::no_delegation(),
            audit_level: AuditLevel::Enhanced,
        }
    }
    
    /// Create a capability for host function calls
    pub fn host_functions(function_patterns: Vec<FunctionPattern>) -> Self {
        let mut permissions = HashSet::new();
        
        for pattern in function_patterns {
            permissions.insert(Permission::HostFunctionCall { 
                function_pattern: pattern 
            });
        }
        
        Self {
            id: CapabilityId(uuid::Uuid::new_v4().to_string()),
            permissions,
            constraints: CapabilityConstraints::default(),
            delegation_rights: DelegationRights::no_delegation(),
            audit_level: AuditLevel::Standard,
        }
    }
    
    /// Check if this capability grants a specific permission
    pub fn grants_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| p.encompasses(permission))
    }
    
    /// Delegate a subset of this capability to another entity
    pub fn delegate(
        &self,
        permissions: HashSet<Permission>,
        constraints: Option<CapabilityConstraints>,
    ) -> Result<Capability, DelegationError> {
        if !self.delegation_rights.can_delegate {
            return Err(DelegationError::DelegationNotAllowed);
        }
        
        // Check if all requested permissions are delegatable
        for permission in &permissions {
            if !self.delegation_rights.delegatable_permissions.contains(permission) {
                return Err(DelegationError::PermissionNotDelegatable(permission.clone()));
            }
        }
        
        let effective_constraints = match (constraints, &self.delegation_rights.delegation_constraints) {
            (Some(req), Some(base)) => Some(base.intersect(&req)),
            (None, Some(base)) => Some(base.clone()),
            (Some(req), None) => Some(req),
            (None, None) => None,
        };
        
        Ok(Capability {
            id: CapabilityId(uuid::Uuid::new_v4().to_string()),
            permissions,
            constraints: effective_constraints.unwrap_or_default(),
            delegation_rights: DelegationRights::no_delegation(), // Delegated caps can't delegate further by default
            audit_level: self.audit_level.clone(),
        })
    }
}
```

### Capability Store and Management

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CapabilityStore {
    capabilities: Arc<RwLock<HashMap<CapabilityId, Capability>>>,
    delegations: Arc<RwLock<HashMap<CapabilityId, Vec<CapabilityId>>>>,
    revocations: Arc<RwLock<HashSet<CapabilityId>>>,
    audit_log: Arc<RwLock<Vec<CapabilityAuditEvent>>>,
}

impl CapabilityStore {
    pub fn new() -> Self {
        Self {
            capabilities: Arc::new(RwLock::new(HashMap::new())),
            delegations: Arc::new(RwLock::new(HashMap::new())),
            revocations: Arc::new(RwLock::new(HashSet::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Store a new capability
    pub async fn store_capability(&self, capability: Capability) -> CapabilityId {
        let id = capability.id.clone();
        
        self.capabilities.write().await.insert(id.clone(), capability.clone());
        
        self.audit_log.write().await.push(CapabilityAuditEvent {
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::CapabilityCreated,
            capability_id: id.clone(),
            actor: None,
            details: serde_json::to_value(&capability).unwrap(),
        });
        
        id
    }
    
    /// Retrieve a capability by ID
    pub async fn get_capability(&self, id: &CapabilityId) -> Option<Capability> {
        // Check if capability is revoked
        if self.revocations.read().await.contains(id) {
            return None;
        }
        
        self.capabilities.read().await.get(id).cloned()
    }
    
    /// Check if a capability grants a specific permission
    pub async fn check_permission(&self, id: &CapabilityId, permission: &Permission) -> bool {
        if let Some(capability) = self.get_capability(id).await {
            // Check time constraints
            if !self.check_time_constraints(&capability.constraints).await {
                return false;
            }
            
            // Check usage constraints
            if !self.check_usage_constraints(id, &capability.constraints).await {
                return false;
            }
            
            // Check permission
            capability.grants_permission(permission)
        } else {
            false
        }
    }
    
    /// Revoke a capability
    pub async fn revoke_capability(&self, id: &CapabilityId, actor: Option<String>) {
        self.revocations.write().await.insert(id.clone());
        
        // Also revoke all delegated capabilities
        if let Some(delegated_ids) = self.delegations.read().await.get(id) {
            for delegated_id in delegated_ids {
                self.revocations.write().await.insert(delegated_id.clone());
            }
        }
        
        self.audit_log.write().await.push(CapabilityAuditEvent {
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::CapabilityRevoked,
            capability_id: id.clone(),
            actor,
            details: serde_json::json!({}),
        });
    }
    
    /// Delegate a capability
    pub async fn delegate_capability(
        &self,
        parent_id: &CapabilityId,
        permissions: HashSet<Permission>,
        constraints: Option<CapabilityConstraints>,
        actor: Option<String>,
    ) -> Result<CapabilityId, DelegationError> {
        let parent_capability = self.get_capability(parent_id).await
            .ok_or(DelegationError::CapabilityNotFound)?;
        
        let delegated_capability = parent_capability.delegate(permissions, constraints)?;
        let delegated_id = delegated_capability.id.clone();
        
        // Store the delegated capability
        self.capabilities.write().await.insert(delegated_id.clone(), delegated_capability);
        
        // Track delegation relationship
        self.delegations.write().await
            .entry(parent_id.clone())
            .or_insert_with(Vec::new)
            .push(delegated_id.clone());
        
        self.audit_log.write().await.push(CapabilityAuditEvent {
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::CapabilityDelegated,
            capability_id: delegated_id.clone(),
            actor,
            details: serde_json::json!({
                "parent_capability": parent_id,
                "permissions": permissions.len(),
            }),
        });
        
        Ok(delegated_id)
    }
    
    async fn check_time_constraints(&self, constraints: &CapabilityConstraints) -> bool {
        if let Some(time_constraints) = &constraints.time_constraints {
            let now = chrono::Utc::now();
            
            // Check validity window
            if let Some(valid_from) = &time_constraints.valid_from {
                if now < *valid_from {
                    return false;
                }
            }
            
            if let Some(valid_until) = &time_constraints.valid_until {
                if now > *valid_until {
                    return false;
                }
            }
            
            // Check time of day restrictions
            if let Some(time_restrictions) = &time_constraints.time_of_day_restrictions {
                let current_time = now.time();
                if !time_restrictions.allows_time(current_time) {
                    return false;
                }
            }
            
            // Check day of week restrictions
            if let Some(day_restrictions) = &time_constraints.day_of_week_restrictions {
                let current_day = now.weekday();
                if !day_restrictions.contains(&current_day) {
                    return false;
                }
            }
        }
        
        true
    }
    
    async fn check_usage_constraints(&self, _id: &CapabilityId, _constraints: &CapabilityConstraints) -> bool {
        // TODO: Implement usage tracking and constraint checking
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAuditEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub capability_id: CapabilityId,
    pub actor: Option<String>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    CapabilityCreated,
    CapabilityAccessed,
    CapabilityDelegated,
    CapabilityRevoked,
    PermissionDenied,
    ConstraintViolation,
}
```

## Sandboxing Mechanisms

### Multi-Layer Sandboxing

```rust
#[derive(Debug, Clone)]
pub struct SandboxConfiguration {
    /// WebAssembly runtime isolation
    pub wasm_isolation: WasmIsolationConfig,
    /// Process-level isolation
    pub process_isolation: ProcessIsolationConfig,
    /// Container-level isolation
    pub container_isolation: ContainerIsolationConfig,
    /// Network isolation
    pub network_isolation: NetworkIsolationConfig,
    /// File system isolation
    pub filesystem_isolation: FilesystemIsolationConfig,
    /// Memory protection
    pub memory_protection: MemoryProtectionConfig,
}

#[derive(Debug, Clone)]
pub struct WasmIsolationConfig {
    /// Enable WebAssembly bounds checking
    pub bounds_checking: bool,
    /// Stack size limit
    pub stack_size_limit: Option<usize>,
    /// Table size limit
    pub table_size_limit: Option<usize>,
    /// Memory size limit
    pub memory_size_limit: Option<usize>,
    /// Fuel limit for execution metering
    pub fuel_limit: Option<u64>,
    /// Disable dangerous features
    pub disable_bulk_memory: bool,
    pub disable_reference_types: bool,
    pub disable_simd: bool,
    pub disable_multi_value: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessIsolationConfig {
    /// Use separate process for each sandbox
    pub separate_process: bool,
    /// Linux namespaces to use
    pub namespaces: HashSet<LinuxNamespace>,
    /// seccomp-bpf filter
    pub seccomp_filter: Option<SeccompFilter>,
    /// Capabilities to drop
    pub dropped_capabilities: HashSet<LinuxCapability>,
    /// User/group to run as
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    /// chroot/pivot_root
    pub root_directory: Option<String>,
}

#[derive(Debug, Clone)]
pub enum LinuxNamespace {
    User,
    Pid,
    Network,
    Mount,
    Ipc,
    Uts,
    Cgroup,
}

#[derive(Debug, Clone)]
pub struct ContainerIsolationConfig {
    /// Container runtime to use
    pub runtime: ContainerRuntime,
    /// Resource limits
    pub cpu_limit: Option<f64>,
    pub memory_limit: Option<u64>,
    pub pids_limit: Option<u64>,
    /// Security options
    pub read_only_root: bool,
    pub no_new_privileges: bool,
    pub apparmor_profile: Option<String>,
    pub selinux_context: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ContainerRuntime {
    Docker,
    Podman,
    Containerd,
    CriO,
}
```

### Runtime Isolation Implementation

```rust
use nix::sys::prctl;
use nix::unistd::{setuid, setgid, chroot};
use seccomp::{ScmpFilterCtx, ScmpAction, ScmpArgCompare, ScmpCompareOp};

pub struct IsolationManager {
    config: SandboxConfiguration,
    active_isolations: HashMap<String, ActiveIsolation>,
}

impl IsolationManager {
    pub fn new(config: SandboxConfiguration) -> Self {
        Self {
            config,
            active_isolations: HashMap::new(),
        }
    }
    
    /// Create isolated environment for sandbox execution
    pub async fn create_isolation(&mut self, sandbox_id: &str) -> Result<ActiveIsolation, IsolationError> {
        let isolation = ActiveIsolation::new(sandbox_id.to_string());
        
        // Apply process isolation
        if self.config.process_isolation.separate_process {
            self.setup_process_isolation(&isolation).await?;
        }
        
        // Apply container isolation
        if let Some(container_config) = &self.config.container_isolation {
            self.setup_container_isolation(&isolation, container_config).await?;
        }
        
        // Apply network isolation
        self.setup_network_isolation(&isolation).await?;
        
        // Apply filesystem isolation
        self.setup_filesystem_isolation(&isolation).await?;
        
        // Apply memory protection
        self.setup_memory_protection(&isolation).await?;
        
        self.active_isolations.insert(sandbox_id.to_string(), isolation.clone());
        Ok(isolation)
    }
    
    async fn setup_process_isolation(&self, isolation: &ActiveIsolation) -> Result<(), IsolationError> {
        let config = &self.config.process_isolation;
        
        // Create new namespaces
        if config.namespaces.contains(&LinuxNamespace::User) {
            // Create user namespace
            unsafe {
                if libc::unshare(libc::CLONE_NEWUSER) != 0 {
                    return Err(IsolationError::NamespaceCreationFailed("user".to_string()));
                }
            }
        }
        
        if config.namespaces.contains(&LinuxNamespace::Pid) {
            unsafe {
                if libc::unshare(libc::CLONE_NEWPID) != 0 {
                    return Err(IsolationError::NamespaceCreationFailed("pid".to_string()));
                }
            }
        }
        
        if config.namespaces.contains(&LinuxNamespace::Network) {
            unsafe {
                if libc::unshare(libc::CLONE_NEWNET) != 0 {
                    return Err(IsolationError::NamespaceCreationFailed("network".to_string()));
                }
            }
        }
        
        if config.namespaces.contains(&LinuxNamespace::Mount) {
            unsafe {
                if libc::unshare(libc::CLONE_NEWNS) != 0 {
                    return Err(IsolationError::NamespaceCreationFailed("mount".to_string()));
                }
            }
        }
        
        // Set up seccomp filter
        if let Some(filter) = &config.seccomp_filter {
            self.apply_seccomp_filter(filter)?;
        }
        
        // Drop capabilities
        for capability in &config.dropped_capabilities {
            self.drop_capability(capability)?;
        }
        
        // Change user/group
        if let Some(gid) = config.gid {
            setgid(nix::unistd::Gid::from_raw(gid))?;
        }
        
        if let Some(uid) = config.uid {
            setuid(nix::unistd::Uid::from_raw(uid))?;
        }
        
        // Change root directory
        if let Some(root_dir) = &config.root_directory {
            chroot(root_dir)?;
            std::env::set_current_dir("/")?;
        }
        
        Ok(())
    }
    
    fn apply_seccomp_filter(&self, filter: &SeccompFilter) -> Result<(), IsolationError> {
        let mut ctx = ScmpFilterCtx::new_filter(ScmpAction::Errno(libc::EPERM))?;
        
        // Allow basic system calls
        let allowed_syscalls = vec![
            "read", "write", "open", "close", "mmap", "munmap", "brk",
            "rt_sigaction", "rt_sigprocmask", "rt_sigreturn",
            "ioctl", "access", "pipe", "select", "mremap", "msync",
            "mincore", "madvise", "shmget", "shmat", "shmctl",
            "dup", "dup2", "pause", "nanosleep", "getitimer",
            "alarm", "setitimer", "getpid", "sendfile", "socket",
            "connect", "accept", "sendto", "recvfrom", "sendmsg",
            "recvmsg", "shutdown", "bind", "listen", "getsockname",
            "getpeername", "socketpair", "setsockopt", "getsockopt",
            "clone", "fork", "vfork", "execve", "exit", "wait4",
            "kill", "uname", "semget", "semop", "semctl", "shmdt",
            "msgget", "msgsnd", "msgrcv", "msgctl", "fcntl",
            "flock", "fsync", "fdatasync", "truncate", "ftruncate",
            "getdents", "getcwd", "chdir", "fchdir", "rename",
            "mkdir", "rmdir", "creat", "link", "unlink", "symlink",
            "readlink", "chmod", "fchmod", "chown", "fchown",
            "lchown", "umask", "gettimeofday", "getrlimit",
            "getrusage", "sysinfo", "times", "ptrace", "getuid",
            "syslog", "getgid", "setuid", "setgid", "geteuid",
            "getegid", "setpgid", "getppid", "getpgrp", "setsid",
            "setreuid", "setregid", "getgroups", "setgroups",
            "setresuid", "getresuid", "setresgid", "getresgid",
            "getpgid", "setfsuid", "setfsgid", "getsid",
            "capget", "capset", "rt_sigpending", "rt_sigtimedwait",
            "rt_sigqueueinfo", "rt_sigsuspend", "sigaltstack",
            "utime", "mknod", "uselib", "personality", "ustat",
            "statfs", "fstatfs", "sysfs", "getpriority",
            "setpriority", "sched_setparam", "sched_getparam",
            "sched_setscheduler", "sched_getscheduler",
            "sched_get_priority_max", "sched_get_priority_min",
            "sched_rr_get_interval", "mlock", "munlock", "mlockall",
            "munlockall", "vhangup", "modify_ldt", "pivot_root",
            "_sysctl", "prctl", "arch_prctl", "adjtimex",
            "setrlimit", "chroot", "sync", "acct", "settimeofday",
            "mount", "umount2", "swapon", "swapoff", "reboot",
            "sethostname", "setdomainname", "iopl", "ioperm",
            "create_module", "init_module", "delete_module",
            "get_kernel_syms", "query_module", "quotactl",
            "nfsservctl", "getpmsg", "putpmsg", "afs_syscall",
            "tuxcall", "security", "gettid", "readahead",
            "setxattr", "lsetxattr", "fsetxattr", "getxattr",
            "lgetxattr", "fgetxattr", "listxattr", "llistxattr",
            "flistxattr", "removexattr", "lremovexattr",
            "fremovexattr", "tkill", "time", "futex",
            "sched_setaffinity", "sched_getaffinity",
            "set_thread_area", "io_setup", "io_destroy", "io_getevents",
            "io_submit", "io_cancel", "get_thread_area",
            "lookup_dcookie", "epoll_create", "epoll_ctl_old",
            "epoll_wait_old", "remap_file_pages", "getdents64",
            "set_tid_address", "restart_syscall", "semtimedop",
            "fadvise64", "timer_create", "timer_settime",
            "timer_gettime", "timer_getoverrun", "timer_delete",
            "clock_settime", "clock_gettime", "clock_getres",
            "clock_nanosleep", "exit_group", "epoll_wait",
            "epoll_ctl", "tgkill", "utimes", "vserver", "mbind",
            "set_mempolicy", "get_mempolicy", "mq_open", "mq_unlink",
            "mq_timedsend", "mq_timedreceive", "mq_notify",
            "mq_getsetattr", "kexec_load", "waitid", "add_key",
            "request_key", "keyctl", "ioprio_set", "ioprio_get",
            "inotify_init", "inotify_add_watch", "inotify_rm_watch",
            "migrate_pages", "openat", "mkdirat", "mknodat",
            "fchownat", "futimesat", "newfstatat", "unlinkat",
            "renameat", "linkat", "symlinkat", "readlinkat",
            "fchmodat", "faccessat", "pselect6", "ppoll",
            "unshare", "set_robust_list", "get_robust_list",
            "splice", "tee", "sync_file_range", "vmsplice",
            "move_pages", "utimensat", "epoll_pwait", "signalfd",
            "timerfd_create", "eventfd", "fallocate",
            "timerfd_settime", "timerfd_gettime", "accept4",
            "signalfd4", "eventfd2", "epoll_create1", "dup3",
            "pipe2", "inotify_init1", "preadv", "pwritev",
            "rt_tgsigqueueinfo", "perf_event_open", "recvmmsg",
            "fanotify_init", "fanotify_mark", "prlimit64",
            "name_to_handle_at", "open_by_handle_at", "clock_adjtime",
            "syncfs", "sendmmsg", "setns", "getcpu", "process_vm_readv",
            "process_vm_writev", "kcmp", "finit_module",
            "sched_setattr", "sched_getattr", "renameat2",
            "seccomp", "getrandom", "memfd_create", "kexec_file_load",
            "bpf", "execveat", "userfaultfd", "membarrier",
            "mlock2", "copy_file_range", "preadv2", "pwritev2",
            "pkey_mprotect", "pkey_alloc", "pkey_free", "statx",
            "io_pgetevents", "rseq",
        ];
        
        for syscall in allowed_syscalls {
            ctx.add_rule(ScmpAction::Allow, syscall)?;
        }
        
        // Block dangerous system calls
        let blocked_syscalls = vec![
            "ptrace", "process_vm_readv", "process_vm_writev",
            "kexec_load", "kexec_file_load", "reboot", "swapon",
            "swapoff", "mount", "umount2", "pivot_root", "chroot",
            "create_module", "init_module", "delete_module",
            "iopl", "ioperm", "clone",
        ];
        
        for syscall in blocked_syscalls {
            ctx.add_rule(ScmpAction::Errno(libc::EPERM), syscall)?;
        }
        
        ctx.load()?;
        Ok(())
    }
    
    fn drop_capability(&self, capability: &LinuxCapability) -> Result<(), IsolationError> {
        use caps::{CapSet, CapsHashSet};
        
        let mut caps = CapsHashSet::new();
        caps.insert(capability.to_caps_capability());
        
        caps::drop(None, CapSet::Effective, &caps)?;
        caps::drop(None, CapSet::Permitted, &caps)?;
        caps::drop(None, CapSet::Inheritable, &caps)?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ActiveIsolation {
    pub sandbox_id: String,
    pub process_id: Option<u32>,
    pub container_id: Option<String>,
    pub network_namespace: Option<String>,
    pub mount_namespace: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ActiveIsolation {
    pub fn new(sandbox_id: String) -> Self {
        Self {
            sandbox_id,
            process_id: None,
            container_id: None,
            network_namespace: None,
            mount_namespace: None,
            created_at: chrono::Utc::now(),
        }
    }
}
```

## Security Monitoring and Auditing

### Real-time Security Monitoring

```rust
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct SecurityMonitor {
    event_sender: broadcast::Sender<SecurityEvent>,
    threat_detector: Arc<ThreatDetector>,
    incident_responder: Arc<IncidentResponder>,
    audit_logger: Arc<SecurityAuditLogger>,
}

impl SecurityMonitor {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            event_sender,
            threat_detector: Arc::new(ThreatDetector::new()),
            incident_responder: Arc::new(IncidentResponder::new()),
            audit_logger: Arc::new(SecurityAuditLogger::new()),
        }
    }
    
    /// Monitor security events in real-time
    pub async fn start_monitoring(&self) -> Result<(), MonitoringError> {
        let mut event_receiver = self.event_sender.subscribe();
        
        tokio::spawn({
            let threat_detector = Arc::clone(&self.threat_detector);
            let incident_responder = Arc::clone(&self.incident_responder);
            let audit_logger = Arc::clone(&self.audit_logger);
            
            async move {
                while let Ok(event) = event_receiver.recv().await {
                    // Log all security events
                    audit_logger.log_event(&event).await;
                    
                    // Analyze for threats
                    if let Some(threat) = threat_detector.analyze_event(&event).await {
                        // Respond to detected threat
                        incident_responder.respond_to_threat(&threat).await;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Report a security event
    pub async fn report_event(&self, event: SecurityEvent) {
        let _ = self.event_sender.send(event);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_id: String,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub sandbox_id: Option<String>,
    pub execution_id: Option<String>,
    pub actor: Option<String>,
    pub details: serde_json::Value,
    pub source_ip: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    // Access control events
    PermissionDenied,
    CapabilityViolation,
    UnauthorizedAccess,
    PrivilegeEscalation,
    
    // Sandbox security events
    SandboxEscapeAttempt,
    ResourceLimitExceeded,
    MemoryViolation,
    SecurityPolicyViolation,
    
    // Network security events
    NetworkConnectionBlocked,
    SuspiciousNetworkActivity,
    DataExfiltrationAttempt,
    NetworkFlood,
    
    // File system security events
    UnauthorizedFileAccess,
    FileSystemModification,
    SymlinkAttack,
    PathTraversalAttempt,
    
    // Runtime security events
    MaliciousCodeDetected,
    AnomalousExecution,
    RuntimeViolation,
    FunctionCallViolation,
    
    // Administrative events
    ConfigurationChange,
    SecurityPolicyUpdate,
    AdminAccess,
    SystemMaintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Threat detection engine
pub struct ThreatDetector {
    detection_rules: Vec<Box<dyn ThreatDetectionRule>>,
    anomaly_detector: AnomalyDetector,
    threat_intelligence: ThreatIntelligence,
}

impl ThreatDetector {
    pub fn new() -> Self {
        Self {
            detection_rules: vec![
                Box::new(SandboxEscapeDetector::new()),
                Box::new(ResourceAbuseDetector::new()),
                Box::new(AnomalousNetworkDetector::new()),
                Box::new(MaliciousCodeDetector::new()),
                Box::new(PrivilegeEscalationDetector::new()),
            ],
            anomaly_detector: AnomalyDetector::new(),
            threat_intelligence: ThreatIntelligence::new(),
        }
    }
    
    /// Analyze a security event for threats
    pub async fn analyze_event(&self, event: &SecurityEvent) -> Option<ThreatDetection> {
        // Check against detection rules
        for rule in &self.detection_rules {
            if let Some(threat) = rule.detect_threat(event).await {
                return Some(threat);
            }
        }
        
        // Check for anomalies
        if let Some(anomaly) = self.anomaly_detector.detect_anomaly(event).await {
            return Some(ThreatDetection {
                threat_id: uuid::Uuid::new_v4().to_string(),
                threat_type: ThreatType::Anomaly(anomaly),
                severity: SecuritySeverity::Medium,
                confidence: 0.7,
                source_event: event.clone(),
                indicators: vec![],
                recommended_actions: vec![
                    "Monitor for additional anomalous activity".to_string(),
                    "Review event context and user behavior".to_string(),
                ],
            });
        }
        
        None
    }
}

#[async_trait]
pub trait ThreatDetectionRule: Send + Sync {
    async fn detect_threat(&self, event: &SecurityEvent) -> Option<ThreatDetection>;
    fn rule_name(&self) -> &str;
    fn rule_description(&self) -> &str;
}

/// Sandbox escape detection
pub struct SandboxEscapeDetector {
    suspicious_patterns: Vec<String>,
}

impl SandboxEscapeDetector {
    pub fn new() -> Self {
        Self {
            suspicious_patterns: vec![
                "ptrace".to_string(),
                "/proc/self/mem".to_string(),
                "../../../".to_string(),
                "mmap".to_string(),
                "mprotect".to_string(),
            ],
        }
    }
}

#[async_trait]
impl ThreatDetectionRule for SandboxEscapeDetector {
    async fn detect_threat(&self, event: &SecurityEvent) -> Option<ThreatDetection> {
        match &event.event_type {
            SecurityEventType::SandboxEscapeAttempt => {
                Some(ThreatDetection {
                    threat_id: uuid::Uuid::new_v4().to_string(),
                    threat_type: ThreatType::SandboxEscape,
                    severity: SecuritySeverity::Critical,
                    confidence: 0.95,
                    source_event: event.clone(),
                    indicators: vec![
                        "Direct sandbox escape attempt detected".to_string(),
                    ],
                    recommended_actions: vec![
                        "Immediately terminate sandbox".to_string(),
                        "Quarantine the code".to_string(),
                        "Alert security team".to_string(),
                        "Review sandbox configuration".to_string(),
                    ],
                })
            }
            SecurityEventType::MemoryViolation => {
                if let Some(details) = event.details.as_object() {
                    if let Some(violation_type) = details.get("violation_type") {
                        if violation_type.as_str() == Some("memory_corruption") {
                            return Some(ThreatDetection {
                                threat_id: uuid::Uuid::new_v4().to_string(),
                                threat_type: ThreatType::SandboxEscape,
                                severity: SecuritySeverity::High,
                                confidence: 0.8,
                                source_event: event.clone(),
                                indicators: vec![
                                    "Memory corruption detected - potential escape vector".to_string(),
                                ],
                                recommended_actions: vec![
                                    "Terminate sandbox immediately".to_string(),
                                    "Analyze memory dump".to_string(),
                                    "Review runtime security".to_string(),
                                ],
                            });
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
    
    fn rule_name(&self) -> &str {
        "sandbox_escape_detector"
    }
    
    fn rule_description(&self) -> &str {
        "Detects attempts to escape from WebAssembly sandbox"
    }
}

#[derive(Debug, Clone)]
pub struct ThreatDetection {
    pub threat_id: String,
    pub threat_type: ThreatType,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub source_event: SecurityEvent,
    pub indicators: Vec<String>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ThreatType {
    SandboxEscape,
    ResourceAbuse,
    DataExfiltration,
    PrivilegeEscalation,
    MaliciousCode,
    AnomalousNetwork,
    Anomaly(AnomalyType),
}
```

## Compliance and Certification

### Security Standards Compliance

```rust
#[derive(Debug, Clone)]
pub struct ComplianceFramework {
    pub standards: Vec<SecurityStandard>,
    pub auditor: ComplianceAuditor,
    pub reporter: ComplianceReporter,
}

#[derive(Debug, Clone)]
pub enum SecurityStandard {
    /// ISO 27001 Information Security Management
    Iso27001,
    /// NIST Cybersecurity Framework
    NistCsf,
    /// SOC 2 Type II
    Soc2TypeII,
    /// Common Criteria EAL4+
    CommonCriteriaEal4Plus,
    /// FIPS 140-2 Level 3
    Fips140Level3,
    /// PCI DSS
    PciDss,
    /// GDPR Privacy
    Gdpr,
    /// HIPAA
    Hipaa,
}

impl ComplianceFramework {
    pub fn new() -> Self {
        Self {
            standards: vec![
                SecurityStandard::Iso27001,
                SecurityStandard::NistCsf,
                SecurityStandard::Soc2TypeII,
            ],
            auditor: ComplianceAuditor::new(),
            reporter: ComplianceReporter::new(),
        }
    }
    
    /// Check compliance against all configured standards
    pub async fn check_compliance(&self, sandbox_config: &SandboxConfiguration) -> ComplianceReport {
        let mut report = ComplianceReport::new();
        
        for standard in &self.standards {
            let result = self.auditor.audit_standard(standard, sandbox_config).await;
            report.add_standard_result(standard.clone(), result);
        }
        
        report
    }
    
    /// Generate compliance documentation
    pub async fn generate_compliance_docs(&self) -> Result<ComplianceDocumentation, ComplianceError> {
        let mut docs = ComplianceDocumentation::new();
        
        // Generate security architecture documentation
        docs.add_section("Security Architecture", self.generate_security_architecture_doc().await?);
        
        // Generate threat model documentation
        docs.add_section("Threat Model", self.generate_threat_model_doc().await?);
        
        // Generate security controls documentation
        docs.add_section("Security Controls", self.generate_security_controls_doc().await?);
        
        // Generate incident response documentation
        docs.add_section("Incident Response", self.generate_incident_response_doc().await?);
        
        Ok(docs)
    }
}

#[derive(Debug, Clone)]
pub struct ComplianceReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub overall_compliance: ComplianceStatus,
    pub standard_results: HashMap<SecurityStandard, StandardComplianceResult>,
    pub recommendations: Vec<ComplianceRecommendation>,
}

#[derive(Debug, Clone)]
pub enum ComplianceStatus {
    Compliant,
    PartiallyCompliant,
    NonCompliant,
}

#[derive(Debug, Clone)]
pub struct StandardComplianceResult {
    pub standard: SecurityStandard,
    pub status: ComplianceStatus,
    pub score: f64, // 0.0 to 1.0
    pub passed_controls: Vec<String>,
    pub failed_controls: Vec<String>,
    pub gaps: Vec<ComplianceGap>,
}

#[derive(Debug, Clone)]
pub struct ComplianceGap {
    pub control_id: String,
    pub description: String,
    pub severity: GapSeverity,
    pub remediation_effort: RemediationEffort,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum GapSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum RemediationEffort {
    Low,      // < 1 day
    Medium,   // 1-5 days
    High,     // 1-4 weeks
    VeryHigh, // > 1 month
}
```

## Next Steps

This security model design provides:

1. **Comprehensive Threat Modeling** - Identified threat actors, attack vectors, and attack surfaces
2. **Capability-Based Security** - Fine-grained permission system with delegation and constraints
3. **Multi-Layer Sandboxing** - WebAssembly, process, container, and system-level isolation
4. **Real-time Monitoring** - Security event detection and incident response
5. **Compliance Framework** - Support for major security standards and certifications

Continue with:
- **[Error Handling Guide](error-handling.md)** - Handle security-related errors
- **[Monitoring Guide](monitoring.md)** - Monitor security events and metrics
- **[Production Deployment](production.md)** - Deploy with enterprise security

---

**Security Excellence:** This design document establishes the foundation for enterprise-grade security. Implement these patterns progressively, starting with basic capability restrictions and expanding to full multi-layer isolation as your security requirements mature.
