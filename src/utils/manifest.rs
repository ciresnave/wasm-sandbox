//! Manifest parsing and validation

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};
use crate::security::{
    Capabilities, NetworkCapability, FilesystemCapability, 
    EnvironmentCapability, ProcessCapability, PortRange, HostSpec
};
use crate::runtime::RuntimeConfig;

/// Sandbox manifest format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxManifest {
    /// Name of the application
    pub name: String,
    
    /// Version of the application
    pub version: String,
    
    /// Description of the application
    pub description: Option<String>,
    
    /// Runtime configuration
    #[serde(default)]
    pub runtime: ManifestRuntime,
    
    /// Security capabilities
    #[serde(default)]
    pub capabilities: ManifestCapabilities,
    
    /// Resource limits
    #[serde(default)]
    pub resource_limits: ManifestResourceLimits,
}

/// Runtime configuration in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRuntime {
    /// WebAssembly runtime engine
    pub engine: String,
    
    /// Whether to enable debugging
    #[serde(default)]
    pub debug: bool,
    
    /// Whether to cache compiled modules
    #[serde(default = "default_true")]
    pub cache_modules: bool,
    
    /// Number of compilation threads
    #[serde(default = "default_threads")]
    pub compilation_threads: usize,
}

fn default_true() -> bool {
    true
}

fn default_threads() -> usize {
    num_cpus::get()
}

impl Default for ManifestRuntime {
    fn default() -> Self {
        Self {
            engine: "wasmtime".to_string(),
            debug: false,
            cache_modules: true,
            compilation_threads: num_cpus::get(),
        }
    }
}

/// Network capabilities in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestNetworkCapabilities {
    /// Network mode
    #[serde(default)]
    pub mode: String,
    
    /// Allowed hosts
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    
    /// Allowed ports
    #[serde(default)]
    pub allowed_ports: Vec<String>,
}

impl Default for ManifestNetworkCapabilities {
    fn default() -> Self {
        Self {
            mode: "none".to_string(),
            allowed_hosts: Vec::new(),
            allowed_ports: Vec::new(),
        }
    }
}

/// Filesystem capabilities in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFilesystemCapabilities {
    /// Readable directories
    #[serde(default)]
    pub readable_dirs: Vec<String>,
    
    /// Writable directories
    #[serde(default)]
    pub writable_dirs: Vec<String>,
    
    /// Whether to allow file creation
    #[serde(default)]
    pub allow_create: bool,
    
    /// Whether to allow file deletion
    #[serde(default)]
    pub allow_delete: bool,
    
    /// Maximum file size
    pub max_file_size: Option<String>,
}

impl Default for ManifestFilesystemCapabilities {
    fn default() -> Self {
        Self {
            readable_dirs: Vec::new(),
            writable_dirs: Vec::new(),
            allow_create: false,
            allow_delete: false,
            max_file_size: None,
        }
    }
}

/// Environment capabilities in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEnvironmentCapabilities {
    /// Environment mode
    #[serde(default)]
    pub mode: String,
    
    /// Environment variables
    #[serde(default)]
    pub vars: Vec<String>,
}

impl Default for ManifestEnvironmentCapabilities {
    fn default() -> Self {
        Self {
            mode: "none".to_string(),
            vars: Vec::new(),
        }
    }
}

/// Process capabilities in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestProcessCapabilities {
    /// Whether to allow process execution
    #[serde(default)]
    pub allow_execution: bool,
    
    /// Allowed commands
    #[serde(default)]
    pub allowed_commands: Vec<String>,
}

impl Default for ManifestProcessCapabilities {
    fn default() -> Self {
        Self {
            allow_execution: false,
            allowed_commands: Vec::new(),
        }
    }
}

/// Capabilities in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestCapabilities {
    /// Network capabilities
    #[serde(default)]
    pub network: ManifestNetworkCapabilities,
    
    /// Filesystem capabilities
    #[serde(default)]
    pub filesystem: ManifestFilesystemCapabilities,
    
    /// Environment capabilities
    #[serde(default)]
    pub environment: ManifestEnvironmentCapabilities,
    
    /// Process capabilities
    #[serde(default)]
    pub process: ManifestProcessCapabilities,
    
    /// Time capabilities
    #[serde(default)]
    pub time_mode: String,
    
    /// Random number generator capabilities
    #[serde(default)]
    pub random_mode: String,
    
    /// Custom capabilities
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

impl Default for ManifestCapabilities {
    fn default() -> Self {
        Self {
            network: ManifestNetworkCapabilities::default(),
            filesystem: ManifestFilesystemCapabilities::default(),
            environment: ManifestEnvironmentCapabilities::default(),
            process: ManifestProcessCapabilities::default(),
            time_mode: "readonly".to_string(),
            random_mode: "pseudo".to_string(),
            custom: HashMap::new(),
        }
    }
}

/// Memory limits in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMemoryLimits {
    /// Maximum memory
    pub max_memory: Option<String>,
    
    /// Reserved memory
    pub reserved_memory: Option<String>,
}

impl Default for ManifestMemoryLimits {
    fn default() -> Self {
        Self {
            max_memory: Some("64MB".to_string()),
            reserved_memory: Some("16MB".to_string()),
        }
    }
}

/// CPU limits in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestCpuLimits {
    /// Maximum execution time
    pub max_execution_time: Option<String>,
    
    /// CPU usage percentage
    pub cpu_usage_percentage: Option<u8>,
    
    /// Maximum threads
    pub max_threads: Option<u32>,
}

impl Default for ManifestCpuLimits {
    fn default() -> Self {
        Self {
            max_execution_time: Some("10s".to_string()),
            cpu_usage_percentage: Some(50),
            max_threads: Some(1),
        }
    }
}

/// I/O limits in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestIoLimits {
    /// Maximum read bytes
    pub max_read_bytes: Option<String>,
    
    /// Maximum write bytes
    pub max_write_bytes: Option<String>,
    
    /// Maximum open files
    pub max_open_files: Option<u32>,
}

impl Default for ManifestIoLimits {
    fn default() -> Self {
        Self {
            max_read_bytes: Some("10MB".to_string()),
            max_write_bytes: Some("5MB".to_string()),
            max_open_files: Some(10),
        }
    }
}

/// Resource limits in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestResourceLimits {
    /// Memory limits
    #[serde(default)]
    pub memory: ManifestMemoryLimits,
    
    /// CPU limits
    #[serde(default)]
    pub cpu: ManifestCpuLimits,
    
    /// I/O limits
    #[serde(default)]
    pub io: ManifestIoLimits,
}

impl Default for ManifestResourceLimits {
    fn default() -> Self {
        Self {
            memory: ManifestMemoryLimits::default(),
            cpu: ManifestCpuLimits::default(),
            io: ManifestIoLimits::default(),
        }
    }
}

impl SandboxManifest {
    /// Load a manifest from a file
    pub fn from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::FileSystem(format!("Failed to read manifest: {}", e)))?;
        
        Self::from_str(&content)
    }
    
    /// Load a manifest from a string
    pub fn from_str(content: &str) -> Result<Self> {
        // Try to parse as TOML first
        if let Ok(manifest) = toml::from_str::<SandboxManifest>(content) {
            return Ok(manifest);
        }
        
        // Try to parse as JSON
        serde_json::from_str::<SandboxManifest>(content)
            .map_err(|e| Error::InvalidConfig(format!("Failed to parse manifest: {}", e)))
    }
    
    /// Convert to runtime configuration
    pub fn to_runtime_config(&self) -> RuntimeConfig {
        RuntimeConfig {
            enable_fuel: true,
            enable_memory_limits: true,
            native_stack_trace: self.runtime.debug,
            debug_info: self.runtime.debug,
            compilation_threads: self.runtime.compilation_threads,
            cache_modules: self.runtime.cache_modules,
            cache_directory: None,
        }
    }
    
    /// Convert to capabilities
    pub fn to_capabilities(&self) -> Result<Capabilities> {
        // Parse network capabilities
        let network = match self.capabilities.network.mode.as_str() {
            "none" => NetworkCapability::None,
            "loopback" => NetworkCapability::Loopback,
            "allowed_hosts" => {
                let mut hosts = Vec::new();
                for host_spec in &self.capabilities.network.allowed_hosts {
                    // Parse host spec (format: "hostname:port" or "hostname:port-range")
                    let parts: Vec<&str> = host_spec.split(':').collect();
                    if parts.len() != 2 {
                        return Err(Error::InvalidConfig(format!("Invalid host spec: {}", host_spec)));
                    }
                    
                    let host = parts[0].to_string();
                    let port_spec = parts[1];
                    
                    let ports = if port_spec.contains('-') {
                        let port_parts: Vec<&str> = port_spec.split('-').collect();
                        if port_parts.len() != 2 {
                            return Err(Error::InvalidConfig(format!("Invalid port range: {}", port_spec)));
                        }
                        
                        let start = port_parts[0].parse::<u16>()
                            .map_err(|_| Error::InvalidConfig(format!("Invalid port: {}", port_parts[0])))?;
                        let end = port_parts[1].parse::<u16>()
                            .map_err(|_| Error::InvalidConfig(format!("Invalid port: {}", port_parts[1])))?;
                        
                        Some(PortRange::new(start, end))
                    } else {
                        let port = port_spec.parse::<u16>()
                            .map_err(|_| Error::InvalidConfig(format!("Invalid port: {}", port_spec)))?;
                        
                        Some(PortRange::single(port))
                    };
                    
                    hosts.push(HostSpec {
                        host,
                        ports,
                        secure: true, // Allow secure connections by default
                    });
                }
                
                NetworkCapability::AllowedHosts(hosts)
            },
            "allowed_ports" => {
                let mut ports = Vec::new();
                for port_spec in &self.capabilities.network.allowed_ports {
                    if port_spec.contains('-') {
                        let port_parts: Vec<&str> = port_spec.split('-').collect();
                        if port_parts.len() != 2 {
                            return Err(Error::InvalidConfig(format!("Invalid port range: {}", port_spec)));
                        }
                        
                        let start = port_parts[0].parse::<u16>()
                            .map_err(|_| Error::InvalidConfig(format!("Invalid port: {}", port_parts[0])))?;
                        let end = port_parts[1].parse::<u16>()
                            .map_err(|_| Error::InvalidConfig(format!("Invalid port: {}", port_parts[1])))?;
                        
                        ports.push(PortRange::new(start, end));
                    } else {
                        let port = port_spec.parse::<u16>()
                            .map_err(|_| Error::InvalidConfig(format!("Invalid port: {}", port_spec)))?;
                        
                        ports.push(PortRange::single(port));
                    }
                }
                
                NetworkCapability::AllowedPorts(ports)
            },
            "full" => NetworkCapability::Full,
            _ => {
                return Err(Error::InvalidConfig(format!("Invalid network mode: {}", self.capabilities.network.mode)));
            }
        };
        
        // Parse filesystem capabilities
        let filesystem = FilesystemCapability {
            readable_dirs: self.capabilities.filesystem.readable_dirs.iter()
                .map(|s| PathBuf::from(s))
                .collect(),
            writable_dirs: self.capabilities.filesystem.writable_dirs.iter()
                .map(|s| PathBuf::from(s))
                .collect(),
            max_file_size: self.capabilities.filesystem.max_file_size.as_ref()
                .and_then(|s| parse_size(s).ok()),
            allow_create: self.capabilities.filesystem.allow_create,
            allow_delete: self.capabilities.filesystem.allow_delete,
        };
        
        // Parse environment capabilities
        let environment = match self.capabilities.environment.mode.as_str() {
            "none" => EnvironmentCapability::None,
            "allowlist" => EnvironmentCapability::Allowlist(
                self.capabilities.environment.vars.clone()
            ),
            "denylist" => EnvironmentCapability::Denylist(
                self.capabilities.environment.vars.clone()
            ),
            "full" => EnvironmentCapability::Full,
            _ => {
                return Err(Error::InvalidConfig(format!("Invalid environment mode: {}", self.capabilities.environment.mode)));
            }
        };
        
        // Parse process capabilities
        let process = if self.capabilities.process.allow_execution {
            if self.capabilities.process.allowed_commands.is_empty() {
                ProcessCapability::Full
            } else {
                ProcessCapability::AllowedCommands(
                    self.capabilities.process.allowed_commands.clone()
                )
            }
        } else {
            ProcessCapability::None
        };
        
        Ok(Capabilities {
            network,
            filesystem,
            environment,
            process,
            time: match self.capabilities.time_mode.as_str() {
                "readonly" => crate::security::TimeCapability::ReadOnly,
                "full" => crate::security::TimeCapability::Full,
                _ => crate::security::TimeCapability::ReadOnly,
            },
            random: match self.capabilities.random_mode.as_str() {
                "none" => crate::security::RandomCapability::None,
                "pseudo" => crate::security::RandomCapability::PseudoOnly,
                "full" => crate::security::RandomCapability::Full,
                _ => crate::security::RandomCapability::PseudoOnly,
            },
            custom: HashMap::new(), // Custom capabilities are not supported in the manifest yet
        })
    }
}

/// Parse a size string (e.g. "10MB") into bytes
fn parse_size(size: &str) -> Result<u64> {
    let size = size.trim();
    
    if size.is_empty() {
        return Err(Error::InvalidConfig("Empty size string".to_string()));
    }
    
    let mut num_str = String::new();
    let mut suffix = String::new();
    
    for c in size.chars() {
        if c.is_digit(10) || c == '.' {
            num_str.push(c);
        } else {
            suffix.push(c);
        }
    }
    
    if num_str.is_empty() {
        return Err(Error::InvalidConfig(format!("Invalid size format: {}", size)));
    }
    
    let num: f64 = num_str.parse()
        .map_err(|_| Error::InvalidConfig(format!("Invalid number: {}", num_str)))?;
    
    let multiplier = match suffix.trim().to_uppercase().as_str() {
        "" | "B" => 1,
        "K" | "KB" => 1024,
        "M" | "MB" => 1024 * 1024,
        "G" | "GB" => 1024 * 1024 * 1024,
        _ => return Err(Error::InvalidConfig(format!("Invalid size suffix: {}", suffix))),
    };
    
    Ok((num * multiplier as f64) as u64)
}
