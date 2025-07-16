//! Configuration builders with ergonomic APIs and human-readable units

use std::path::PathBuf;
use std::time::Duration;

use crate::error::{Result, SandboxError};
use crate::security::Capabilities;
use crate::{InstanceConfig, SandboxConfig};

/// Human-readable memory units
pub trait MemoryUnit {
    fn bytes(self) -> u64;
    fn kb(self) -> u64;
    fn mb(self) -> u64;
    fn gb(self) -> u64;
}

impl MemoryUnit for u64 {
    fn bytes(self) -> u64 { self }
    fn kb(self) -> u64 { self * 1024 }
    fn mb(self) -> u64 { self * 1024 * 1024 }
    fn gb(self) -> u64 { self * 1024 * 1024 * 1024 }
}

/// Human-readable time units
pub trait TimeUnit {
    fn millis(self) -> Duration;
    fn seconds(self) -> Duration;
    fn minutes(self) -> Duration;
}

impl TimeUnit for u64 {
    fn millis(self) -> Duration { Duration::from_millis(self) }
    fn seconds(self) -> Duration { Duration::from_secs(self) }
    fn minutes(self) -> Duration { Duration::from_secs(self * 60) }
}

/// Network policy for advanced capabilities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicy {
    pub allowed_domains: Vec<String>,
    pub max_connections: usize,
    pub allowed_ports: Vec<u16>,
    pub deny_all: bool,
    pub loopback_only: bool,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            allowed_domains: Vec::new(),
            max_connections: 10,
            allowed_ports: Vec::new(),
            deny_all: true,
            loopback_only: false,
        }
    }
}

/// Filesystem policy for advanced capabilities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilesystemPolicy {
    pub read_paths: Vec<PathBuf>,
    pub write_paths: Vec<PathBuf>,
    pub temp_dir_access: bool,
    pub max_file_size: usize,
    pub deny_all: bool,
}

impl Default for FilesystemPolicy {
    fn default() -> Self {
        Self {
            read_paths: Vec::new(),
            write_paths: Vec::new(),
            temp_dir_access: false,
            max_file_size: 10 * 1024 * 1024, // 10MB
            deny_all: true,
        }
    }
}

/// Advanced capabilities with fine-grained controls
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdvancedCapabilities {
    pub network: NetworkPolicy,
    pub filesystem: FilesystemPolicy,
    pub env_vars: Vec<String>,
    pub process_spawn: bool,
    pub max_threads: usize,
}

impl Default for AdvancedCapabilities {
    fn default() -> Self {
        Self {
            network: NetworkPolicy::default(),
            filesystem: FilesystemPolicy::default(),
            env_vars: Vec::new(),
            process_spawn: false,
            max_threads: 1,
        }
    }
}

/// Builder for InstanceConfig with ergonomic APIs
#[derive(Debug, Clone)]
pub struct InstanceConfigBuilder {
    config: InstanceConfig,
    advanced_caps: AdvancedCapabilities,
}

impl InstanceConfigBuilder {
    /// Create a new instance config builder
    pub fn new() -> Self {
        Self {
            config: InstanceConfig::default(),
            advanced_caps: AdvancedCapabilities::default(),
        }
    }

    /// Set memory limit using human-readable units
    pub fn memory_limit<T: MemoryUnit>(mut self, amount: T) -> Self {
        self.config.resource_limits.memory.max_memory_pages = 
            (amount.bytes() / 65536) as u32; // WASM page size is 64KB
        self
    }

    /// Set timeout using human-readable duration
    pub fn timeout<T: TimeUnit>(mut self, duration: T) -> Self {
        self.config.startup_timeout_ms = duration.millis().as_millis() as u64;
        self
    }

    /// Allow filesystem read access to paths
    pub fn filesystem_read<P: AsRef<std::path::Path>>(mut self, paths: &[P]) -> Self {
        self.advanced_caps.filesystem.read_paths = paths.iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();
        self.advanced_caps.filesystem.deny_all = false;
        self
    }

    /// Allow filesystem write access to paths
    pub fn filesystem_write<P: AsRef<std::path::Path>>(mut self, paths: &[P]) -> Self {
        self.advanced_caps.filesystem.write_paths = paths.iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();
        self.advanced_caps.filesystem.deny_all = false;
        self
    }

    /// Deny all network access
    pub fn network_deny_all(mut self) -> Self {
        self.advanced_caps.network.deny_all = true;
        self.advanced_caps.network.loopback_only = false;
        self
    }

    /// Allow only loopback network access
    pub fn network_loopback_only(mut self) -> Self {
        self.advanced_caps.network.deny_all = false;
        self.advanced_caps.network.loopback_only = true;
        self
    }

    /// Allow network access to specific domains
    pub fn network_allow_domains(mut self, domains: &[impl AsRef<str>]) -> Self {
        self.advanced_caps.network.allowed_domains = domains.iter()
            .map(|d| d.as_ref().to_string())
            .collect();
        self.advanced_caps.network.deny_all = false;
        self
    }

    /// Allow network access to specific ports
    pub fn network_allow_ports(mut self, ports: &[u16]) -> Self {
        self.advanced_caps.network.allowed_ports = ports.to_vec();
        self.advanced_caps.network.deny_all = false;
        self
    }

    /// Set maximum number of network connections
    pub fn network_max_connections(mut self, max: usize) -> Self {
        self.advanced_caps.network.max_connections = max;
        self
    }

    /// Set CPU time limit
    pub fn cpu_time_limit<T: TimeUnit>(mut self, duration: T) -> Self {
        self.config.resource_limits.cpu.max_execution_time_ms = duration.millis().as_millis() as u64;
        self
    }

    /// Set maximum file operations per second
    pub fn io_ops_limit(mut self, ops_per_sec: u32) -> Self {
        self.config.resource_limits.io.max_read_bytes_per_second = Some(ops_per_sec as u64);
        self
    }

    /// Enable debugging
    pub fn enable_debug(mut self) -> Self {
        self.config.enable_debug = true;
        self
    }

    /// Allow access to environment variables
    pub fn env_vars(mut self, vars: &[impl AsRef<str>]) -> Self {
        self.advanced_caps.env_vars = vars.iter()
            .map(|v| v.as_ref().to_string())
            .collect();
        self
    }

    /// Allow process spawning
    pub fn allow_process_spawn(mut self) -> Self {
        self.advanced_caps.process_spawn = true;
        self
    }

    /// Set maximum number of threads
    pub fn max_threads(mut self, max: usize) -> Self {
        self.advanced_caps.max_threads = max;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<InstanceConfig> {
        // Validate configuration
        self.validate()?;
        
        // Convert advanced capabilities to basic capabilities
        let converted_capabilities = self.convert_capabilities()?;
        let mut config = self.config;
        config.capabilities = converted_capabilities;
        
        Ok(config)
    }

    /// Validate the configuration
    fn validate(&self) -> Result<()> {
        // Check memory limits
        if self.config.resource_limits.memory.max_memory_pages == 0 {
            return Err(SandboxError::config_error(
                "Memory limit cannot be zero",
                Some("Use .memory_limit() to set a valid memory limit".to_string())
            ));
        }

        // Check timeout
        if self.config.startup_timeout_ms == 0 {
            return Err(SandboxError::config_error(
                "Timeout cannot be zero",
                Some("Use .timeout() to set a valid timeout".to_string())
            ));
        }

        // Validate filesystem paths exist
        for path in &self.advanced_caps.filesystem.read_paths {
            if !path.exists() {
                return Err(SandboxError::config_error(
                    format!("Read path does not exist: {:?}", path),
                    Some("Ensure all read paths exist before configuration".to_string())
                ));
            }
        }

        for path in &self.advanced_caps.filesystem.write_paths {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    return Err(SandboxError::config_error(
                        format!("Write path parent directory does not exist: {:?}", parent),
                        Some("Ensure parent directories exist before configuration".to_string())
                    ));
                }
            }
        }

        Ok(())
    }

    /// Convert advanced capabilities to basic capabilities
    fn convert_capabilities(&self) -> Result<Capabilities> {
        use crate::security::{NetworkCapability, EnvironmentCapability, ProcessCapability, FilesystemCapability};
        
        let mut capabilities = Capabilities::minimal();
        
        // Convert filesystem capabilities
        if !self.advanced_caps.filesystem.deny_all && 
           (!self.advanced_caps.filesystem.read_paths.is_empty() || 
            !self.advanced_caps.filesystem.write_paths.is_empty()) {
            capabilities.filesystem = FilesystemCapability {
                readable_dirs: self.advanced_caps.filesystem.read_paths.clone(),
                writable_dirs: self.advanced_caps.filesystem.write_paths.clone(),
                max_file_size: Some(self.advanced_caps.filesystem.max_file_size as u64),
                allow_create: !self.advanced_caps.filesystem.write_paths.is_empty(),
                allow_delete: !self.advanced_caps.filesystem.write_paths.is_empty(),
            };
        }
        
        // Convert network capabilities
        if !self.advanced_caps.network.deny_all {
            if self.advanced_caps.network.loopback_only {
                capabilities.network = NetworkCapability::Loopback;
            } else if !self.advanced_caps.network.allowed_domains.is_empty() {
                use crate::security::HostSpec;
                let hosts = self.advanced_caps.network.allowed_domains.iter()
                    .map(|domain| HostSpec {
                        host: domain.clone(),
                        ports: None,
                        secure: false,
                    })
                    .collect();
                capabilities.network = NetworkCapability::AllowedHosts(hosts);
            } else if !self.advanced_caps.network.allowed_ports.is_empty() {
                use crate::security::PortRange;
                let ports = self.advanced_caps.network.allowed_ports.iter()
                    .map(|&port| PortRange::single(port))
                    .collect();
                capabilities.network = NetworkCapability::AllowedPorts(ports);
            } else {
                capabilities.network = NetworkCapability::Full;
            }
        }
        
        // Convert environment capabilities
        if !self.advanced_caps.env_vars.is_empty() {
            capabilities.environment = EnvironmentCapability::Allowlist(self.advanced_caps.env_vars.clone());
        }
        
        // Convert process capabilities
        if self.advanced_caps.process_spawn {
            capabilities.process = ProcessCapability::Full;
        }
        
        Ok(capabilities)
    }
}

impl Default for InstanceConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for InstanceConfig to add builder functionality
pub trait InstanceConfigExt {
    /// Create a new builder
    fn builder() -> InstanceConfigBuilder;
}

impl InstanceConfigExt for InstanceConfig {
    fn builder() -> InstanceConfigBuilder {
        InstanceConfigBuilder::new()
    }
}

/// Builder for SandboxConfig
#[derive(Debug, Clone)]
pub struct SandboxConfigBuilder {
    config: SandboxConfig,
}

impl SandboxConfigBuilder {
    /// Create a new sandbox config builder
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
        }
    }

    /// Set the default instance configuration
    pub fn default_instance_config(mut self, config: InstanceConfig) -> Self {
        self.config.default_instance_config = config;
        self
    }

    /// Set runtime to use Wasmtime
    /// 
    /// Note: Runtime selection is determined at compile time by feature flags.
    /// This method is provided for API compatibility and will return self unchanged.
    /// To use Wasmtime, ensure the "wasmtime-runtime" feature is enabled (default).
    pub fn use_wasmtime(self) -> Self {
        // Runtime selection is handled by feature flags at compile time
        self
    }

    /// Set runtime to use Wasmer
    /// 
    /// Note: Runtime selection is determined at compile time by feature flags.
    /// This method is provided for API compatibility and will return self unchanged.
    /// To use Wasmer, enable the "wasmer-runtime" feature and disable "wasmtime-runtime".
    pub fn use_wasmer(self) -> Self {
        // Runtime selection is handled by feature flags at compile time
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<SandboxConfig> {
        Ok(self.config)
    }
}

impl Default for SandboxConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for SandboxConfig to add builder functionality
pub trait SandboxConfigExt {
    /// Create a new builder
    fn builder() -> SandboxConfigBuilder;
}

impl SandboxConfigExt for SandboxConfig {
    fn builder() -> SandboxConfigBuilder {
        SandboxConfigBuilder::new()
    }
}
