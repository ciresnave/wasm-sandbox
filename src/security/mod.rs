//! Security policies and resource limitations

use std::collections::HashMap;
use std::path::PathBuf;

pub mod audit;
pub mod capabilities;
pub mod resource_limits;
pub mod audit_impl;

/// Host specification for network access
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostSpec {
    /// Hostname or IP address
    pub host: String,
    
    /// Port range (inclusive)
    pub ports: Option<PortRange>,
    
    /// Whether to allow secure connections (HTTPS)
    pub secure: bool,
}

/// Port range specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortRange {
    /// Start port (inclusive)
    pub start: u16,
    
    /// End port (inclusive)
    pub end: u16,
}

impl PortRange {
    /// Create a new port range
    pub fn new(start: u16, end: u16) -> Self {
        assert!(start <= end, "Start port must be less than or equal to end port");
        Self { start, end }
    }
    
    /// Create a single port range
    pub fn single(port: u16) -> Self {
        Self {
            start: port,
            end: port,
        }
    }
    
    /// Check if a port is in the range
    pub fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }
}

/// Network access capabilities
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkCapability {
    /// No network access allowed
    None,
    
    /// Loopback only (localhost)
    Loopback,
    
    /// Specific hosts only
    AllowedHosts(Vec<HostSpec>),
    
    /// Specific ports only
    AllowedPorts(Vec<PortRange>),
    
    /// Full network access
    Full,
}

impl Default for NetworkCapability {
    fn default() -> Self {
        Self::None
    }
}

/// Filesystem access capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct FilesystemCapability {
    /// Directories that can be read from
    pub readable_dirs: Vec<PathBuf>,
    
    /// Directories that can be written to
    pub writable_dirs: Vec<PathBuf>,
    
    /// Maximum file size for writing
    pub max_file_size: Option<u64>,
    
    /// Allow file creation
    pub allow_create: bool,
    
    /// Allow file deletion
    pub allow_delete: bool,
}

impl Default for FilesystemCapability {
    fn default() -> Self {
        Self {
            readable_dirs: Vec::new(),
            writable_dirs: Vec::new(),
            max_file_size: None,
            allow_create: false,
            allow_delete: false,
        }
    }
}

/// Environment variable access capabilities
#[derive(Debug, Clone, PartialEq)]
pub enum EnvironmentCapability {
    /// No environment variable access
    None,
    
    /// Allow only specific variables
    Allowlist(Vec<String>),
    
    /// Deny specific variables
    Denylist(Vec<String>),
    
    /// Full environment variable access
    Full,
}

impl Default for EnvironmentCapability {
    fn default() -> Self {
        Self::None
    }
}

/// Process creation capability
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessCapability {
    /// No process creation allowed
    None,
    
    /// Allow only specific commands
    AllowedCommands(Vec<String>),
    
    /// Full process creation capability
    Full,
}

impl Default for ProcessCapability {
    fn default() -> Self {
        Self::None
    }
}

/// Time access capability
#[derive(Debug, Clone, PartialEq)]
pub enum TimeCapability {
    /// Read-only time access
    ReadOnly,
    
    /// Full time access (can set system time)
    Full,
}

impl Default for TimeCapability {
    fn default() -> Self {
        Self::ReadOnly
    }
}

/// Random number generation capability
#[derive(Debug, Clone, PartialEq)]
pub enum RandomCapability {
    /// No random number generation
    None,
    
    /// Pseudo-random number generation only
    PseudoOnly,
    
    /// Full random number generation (includes secure random)
    Full,
}

impl Default for RandomCapability {
    fn default() -> Self {
        Self::PseudoOnly
    }
}

/// Custom capability type
#[derive(Debug, Clone, PartialEq)]
pub enum CustomCapability {
    /// Boolean capability (enabled/disabled)
    Boolean(bool),
    
    /// Numeric capability with limits
    Numeric {
        /// Current value
        value: i64,
        /// Minimum allowed value
        min: i64,
        /// Maximum allowed value
        max: i64,
    },
    
    /// String capability
    String(String),
    
    /// String list capability
    StringList(Vec<String>),
}

/// Security capabilities for the sandbox
#[derive(Debug, Clone, PartialEq)]
pub struct Capabilities {
    /// Network access permissions
    pub network: NetworkCapability,
    
    /// Filesystem access permissions
    pub filesystem: FilesystemCapability,
    
    /// Environment variable access
    pub environment: EnvironmentCapability,
    
    /// Process creation capability
    pub process: ProcessCapability,
    
    /// Time access capability (for limiting time manipulation)
    pub time: TimeCapability,
    
    /// Random number generation capability
    pub random: RandomCapability,
    
    /// Custom capabilities map
    pub custom: HashMap<String, CustomCapability>,
}

impl Capabilities {
    /// Create capabilities with minimal permissions (most restrictive)
    pub fn minimal() -> Self {
        Self {
            network: NetworkCapability::None,
            filesystem: FilesystemCapability::default(),
            environment: EnvironmentCapability::None,
            process: ProcessCapability::None,
            time: TimeCapability::ReadOnly,
            random: RandomCapability::PseudoOnly,
            custom: HashMap::new(),
        }
    }
    
    /// Create capabilities for development (least restrictive)
    pub fn development() -> Self {
        Self {
            network: NetworkCapability::Loopback,
            filesystem: FilesystemCapability {
                readable_dirs: vec![std::env::current_dir().unwrap_or_default()],
                writable_dirs: vec![std::env::temp_dir()],
                max_file_size: Some(10 * 1024 * 1024), // 10MB
                allow_create: true,
                allow_delete: false,
            },
            environment: EnvironmentCapability::Allowlist(vec![
                "PATH".to_string(),
                "TEMP".to_string(),
                "TMP".to_string(),
            ]),
            process: ProcessCapability::None,
            time: TimeCapability::ReadOnly,
            random: RandomCapability::Full,
            custom: HashMap::new(),
        }
    }
    
    /// Add a custom capability
    pub fn add_custom(&mut self, name: &str, capability: CustomCapability) {
        self.custom.insert(name.to_string(), capability);
    }
    
    /// Get a custom capability
    pub fn get_custom(&self, name: &str) -> Option<&CustomCapability> {
        self.custom.get(name)
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        Self::minimal()
    }
}

/// Memory resource limits
#[derive(Debug, Clone)]
pub struct MemoryLimits {
    /// Maximum memory pages (64KB each)
    pub max_memory_pages: u32,
    
    /// Reserved memory pages
    pub reserved_memory_pages: u32,
    
    /// Growth rate limiting
    pub max_growth_rate: Option<u32>,
    
    /// Maximum memory addresses
    pub max_tables: u32,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_memory_pages: 160, // 10MB (160 * 64KB)
            reserved_memory_pages: 16, // 1MB (16 * 64KB)
            max_growth_rate: Some(10),
            max_tables: 1,
        }
    }
}

/// CPU resource limits
#[derive(Debug, Clone)]
pub struct CpuLimits {
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    
    /// CPU usage percentage (0-100)
    pub cpu_usage_percentage: Option<u8>,
    
    /// Thread limit
    pub max_threads: Option<u32>,
}

impl Default for CpuLimits {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 5000, // 5 seconds
            cpu_usage_percentage: Some(50), // 50%
            max_threads: Some(1), // Single-threaded by default
        }
    }
}

/// I/O resource limits
#[derive(Debug, Clone)]
pub struct IoLimits {
    /// Maximum number of open files
    pub max_open_files: u32,
    
    /// Maximum read bytes per second
    pub max_read_bytes_per_second: Option<u64>,
    
    /// Maximum write bytes per second
    pub max_write_bytes_per_second: Option<u64>,
    
    /// Maximum total read bytes
    pub max_total_read_bytes: Option<u64>,
    
    /// Maximum total write bytes
    pub max_total_write_bytes: Option<u64>,
}

impl Default for IoLimits {
    fn default() -> Self {
        Self {
            max_open_files: 10,
            max_read_bytes_per_second: Some(1024 * 1024), // 1MB/s
            max_write_bytes_per_second: Some(1024 * 1024), // 1MB/s
            max_total_read_bytes: Some(10 * 1024 * 1024), // 10MB
            max_total_write_bytes: Some(5 * 1024 * 1024), // 5MB
        }
    }
}

/// Time limits
#[derive(Debug, Clone)]
pub struct TimeLimits {
    /// Maximum execution time in milliseconds
    pub max_total_time_ms: u64,
    
    /// Maximum idle time in milliseconds
    pub max_idle_time_ms: Option<u64>,
}

impl Default for TimeLimits {
    fn default() -> Self {
        Self {
            max_total_time_ms: 30000, // 30 seconds
            max_idle_time_ms: Some(5000), // 5 seconds
        }
    }
}

/// Resource limits for the sandbox
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Memory limits in bytes
    pub memory: MemoryLimits,
    
    /// CPU limits
    pub cpu: CpuLimits,
    
    /// I/O limits
    pub io: IoLimits,
    
    /// Time limits
    pub time: TimeLimits,
    
    /// Wasmtime fuel limits (instruction counting)
    pub fuel: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory: MemoryLimits::default(),
            cpu: CpuLimits::default(),
            io: IoLimits::default(),
            time: TimeLimits::default(),
            fuel: Some(10_000_000), // 10M instructions by default
        }
    }
}
