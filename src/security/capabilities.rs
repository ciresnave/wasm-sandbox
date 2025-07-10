//! Implementation of security capabilities for the sandbox

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::net::{IpAddr, SocketAddr};

use crate::error::{Error, Result};
use crate::security::{
    NetworkCapability, FilesystemCapability, 
    EnvironmentCapability, ProcessCapability, TimeCapability, RandomCapability
};

/// Capability verification helper
pub trait CapabilityVerifier {
    /// Verify that an operation is allowed by the capabilities
    fn verify(&self, operation: &str, params: &[&str]) -> Result<()>;
}

/// Network capability verifier
pub struct NetworkVerifier {
    capability: NetworkCapability,
}

impl NetworkVerifier {
    /// Create a new network verifier
    pub fn new(capability: NetworkCapability) -> Self {
        Self { capability }
    }
    
    /// Check if a host is allowed
    pub fn is_host_allowed(&self, host: &str, port: u16, secure: bool) -> bool {
        match &self.capability {
            NetworkCapability::None => false,
            NetworkCapability::Loopback => {
                // Check if host is localhost or 127.0.0.1
                host == "localhost" || host == "127.0.0.1" || host == "::1"
            },
            NetworkCapability::AllowedHosts(hosts) => {
                // Check if host is in the allowed hosts list
                hosts.iter().any(|h| {
                    // Check host
                    if h.host != host {
                        return false;
                    }
                    
                    // Check port if specified
                    if let Some(port_range) = &h.ports {
                        if !port_range.contains(port) {
                            return false;
                        }
                    }
                    
                    // Check secure flag
                    if !h.secure && secure {
                        return false;
                    }
                    
                    true
                })
            },
            NetworkCapability::AllowedPorts(ports) => {
                // Check if port is in any allowed port range
                ports.iter().any(|r| r.contains(port))
            },
            NetworkCapability::Full => true,
        }
    }
    
    /// Check if an IP address is allowed
    pub fn is_ip_allowed(&self, ip: IpAddr, port: u16) -> bool {
        match &self.capability {
            NetworkCapability::None => false,
            NetworkCapability::Loopback => {
                // Check if IP is localhost
                match ip {
                    IpAddr::V4(addr) => addr.is_loopback(),
                    IpAddr::V6(addr) => addr.is_loopback(),
                }
            },
            NetworkCapability::AllowedHosts(_hosts) => {
                // Not implemented: would need to resolve hosts to IPs
                false
            },
            NetworkCapability::AllowedPorts(ports) => {
                // Check if port is in any allowed port range
                ports.iter().any(|r| r.contains(port))
            },
            NetworkCapability::Full => true,
        }
    }
    
    /// Check if a socket address is allowed
    pub fn is_socket_allowed(&self, socket: SocketAddr) -> bool {
        self.is_ip_allowed(socket.ip(), socket.port())
    }
}

impl CapabilityVerifier for NetworkVerifier {
    fn verify(&self, operation: &str, params: &[&str]) -> Result<()> {
        match operation {
            "connect" => {
                if params.len() < 2 {
                    return Err(Error::Capability("Missing host and port for connect".to_string()));
                }
                
                let host = params[0];
                let port = params[1].parse::<u16>().map_err(|_| {
                    Error::Capability(format!("Invalid port: {}", params[1]))
                })?;
                
                let secure = params.get(2).map(|s| *s == "secure").unwrap_or(false);
                
                if !self.is_host_allowed(host, port, secure) {
                    return Err(Error::SecurityViolation(
                        format!("Network access denied to {}:{}", host, port)
                    ));
                }
            }
            "bind" => {
                if params.len() < 2 {
                    return Err(Error::Capability("Missing host and port for bind".to_string()));
                }
                
                let host = params[0];
                let port = params[1].parse::<u16>().map_err(|_| {
                    Error::Capability(format!("Invalid port: {}", params[1]))
                })?;
                
                if !self.is_host_allowed(host, port, false) {
                    return Err(Error::SecurityViolation(
                        format!("Network binding denied to {}:{}", host, port)
                    ));
                }
            }
            "listen" => {
                if params.len() < 1 {
                    return Err(Error::Capability("Missing port for listen".to_string()));
                }
                
                let port = params[0].parse::<u16>().map_err(|_| {
                    Error::Capability(format!("Invalid port: {}", params[0]))
                })?;
                
                // For listen, we check if the loopback address is allowed with this port
                if !self.is_host_allowed("127.0.0.1", port, false) {
                    return Err(Error::SecurityViolation(
                        format!("Network listening denied on port {}", port)
                    ));
                }
            }
            _ => {
                return Err(Error::Capability(format!("Unknown network operation: {}", operation)));
            }
        }
        
        Ok(())
    }
}

/// Filesystem capability verifier
pub struct FilesystemVerifier {
    capability: FilesystemCapability,
    normalized_readable: HashSet<PathBuf>,
    normalized_writable: HashSet<PathBuf>,
}

impl FilesystemVerifier {
    /// Create a new filesystem verifier
    pub fn new(capability: FilesystemCapability) -> Self {
        // Normalize paths for better comparison
        let normalized_readable = capability.readable_dirs
            .iter()
            .filter_map(|p| std::fs::canonicalize(p).ok())
            .collect();
            
        let normalized_writable = capability.writable_dirs
            .iter()
            .filter_map(|p| std::fs::canonicalize(p).ok())
            .collect();
        
        Self { 
            capability,
            normalized_readable,
            normalized_writable,
        }
    }
    
    /// Check if a path is readable
    pub fn is_readable(&self, path: &Path) -> bool {
        // Try to canonicalize the path
        let canon_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => return false, // Path doesn't exist or other error
        };
        
        // Check if the path is in any readable directory
        for dir in &self.normalized_readable {
            if is_path_within(dir, &canon_path) {
                return true;
            }
        }
        
        // Also check if it's writable (writable implies readable)
        self.is_writable(path)
    }
    
    /// Check if a path is writable
    pub fn is_writable(&self, path: &Path) -> bool {
        // Try to canonicalize the path
        let canon_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => {
                // If path doesn't exist, check its parent directory
                if let Some(parent) = path.parent() {
                    match std::fs::canonicalize(parent) {
                        Ok(p) => p,
                        Err(_) => return false, // Parent doesn't exist
                    }
                } else {
                    return false; // No parent (root)
                }
            }
        };
        
        // Check if the path is in any writable directory
        for dir in &self.normalized_writable {
            if is_path_within(dir, &canon_path) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if file creation is allowed
    pub fn can_create(&self) -> bool {
        self.capability.allow_create
    }
    
    /// Check if file deletion is allowed
    pub fn can_delete(&self) -> bool {
        self.capability.allow_delete
    }
    
    /// Check if a file size is within limits
    pub fn is_size_allowed(&self, size: u64) -> bool {
        match self.capability.max_file_size {
            Some(limit) => size <= limit,
            None => true,
        }
    }
}

impl CapabilityVerifier for FilesystemVerifier {
    fn verify(&self, operation: &str, params: &[&str]) -> Result<()> {
        match operation {
            "open" | "read" => {
                if params.is_empty() {
                    return Err(Error::Capability("Missing path for open/read".to_string()));
                }
                
                let path = Path::new(params[0]);
                if !self.is_readable(path) {
                    return Err(Error::SecurityViolation(
                        format!("File read access denied: {}", path.display())
                    ));
                }
            }
            "write" | "append" => {
                if params.is_empty() {
                    return Err(Error::Capability("Missing path for write/append".to_string()));
                }
                
                let path = Path::new(params[0]);
                if !self.is_writable(path) {
                    return Err(Error::SecurityViolation(
                        format!("File write access denied: {}", path.display())
                    ));
                }
                
                // Check size limit if provided
                if params.len() > 1 {
                    let size = params[1].parse::<u64>().map_err(|_| {
                        Error::Capability(format!("Invalid size: {}", params[1]))
                    })?;
                    
                    if !self.is_size_allowed(size) {
                        return Err(Error::ResourceLimit(
                            format!("File size limit exceeded: {}", size)
                        ));
                    }
                }
            }
            "create" => {
                if params.is_empty() {
                    return Err(Error::Capability("Missing path for create".to_string()));
                }
                
                if !self.can_create() {
                    return Err(Error::SecurityViolation("File creation is not allowed".to_string()));
                }
                
                let path = Path::new(params[0]);
                if !self.is_writable(path) {
                    return Err(Error::SecurityViolation(
                        format!("File creation access denied: {}", path.display())
                    ));
                }
            }
            "delete" | "remove" => {
                if params.is_empty() {
                    return Err(Error::Capability("Missing path for delete/remove".to_string()));
                }
                
                if !self.can_delete() {
                    return Err(Error::SecurityViolation("File deletion is not allowed".to_string()));
                }
                
                let path = Path::new(params[0]);
                if !self.is_writable(path) {
                    return Err(Error::SecurityViolation(
                        format!("File deletion access denied: {}", path.display())
                    ));
                }
            }
            _ => {
                return Err(Error::Capability(format!("Unknown filesystem operation: {}", operation)));
            }
        }
        
        Ok(())
    }
}

/// Helper function to check if a path is within a directory
fn is_path_within(dir: &Path, path: &Path) -> bool {
    let dir_str = dir.to_string_lossy();
    let path_str = path.to_string_lossy();
    
    // Check if path starts with dir (and there's either an exact match or a path separator after)
    if path_str == dir_str {
        return true;
    }
    
    path_str.starts_with(&format!("{}{}", dir_str, std::path::MAIN_SEPARATOR))
}

/// Environment capability verifier
pub struct EnvironmentVerifier {
    capability: EnvironmentCapability,
}

impl EnvironmentVerifier {
    /// Create a new environment verifier
    pub fn new(capability: EnvironmentCapability) -> Self {
        Self { capability }
    }
    
    /// Check if a variable is allowed
    pub fn is_var_allowed(&self, var: &str) -> bool {
        match &self.capability {
            EnvironmentCapability::None => false,
            EnvironmentCapability::Allowlist(allowed) => allowed.iter().any(|v| v == var),
            EnvironmentCapability::Denylist(denied) => !denied.iter().any(|v| v == var),
            EnvironmentCapability::Full => true,
        }
    }
}

impl CapabilityVerifier for EnvironmentVerifier {
    fn verify(&self, operation: &str, params: &[&str]) -> Result<()> {
        match operation {
            "get" | "set" => {
                if params.is_empty() {
                    return Err(Error::Capability("Missing variable name".to_string()));
                }
                
                let var = params[0];
                if !self.is_var_allowed(var) {
                    return Err(Error::SecurityViolation(
                        format!("Environment variable access denied: {}", var)
                    ));
                }
                
                // For "set", additionally check if we're in Full mode
                if operation == "set" && !matches!(self.capability, EnvironmentCapability::Full) {
                    return Err(Error::SecurityViolation(
                        format!("Setting environment variables is not allowed: {}", var)
                    ));
                }
            }
            _ => {
                return Err(Error::Capability(format!("Unknown environment operation: {}", operation)));
            }
        }
        
        Ok(())
    }
}

/// Process capability verifier
pub struct ProcessVerifier {
    capability: ProcessCapability,
}

impl ProcessVerifier {
    /// Create a new process verifier
    pub fn new(capability: ProcessCapability) -> Self {
        Self { capability }
    }
    
    /// Check if a command is allowed to be executed
    pub fn is_command_allowed(&self, command: &str) -> bool {
        match &self.capability {
            ProcessCapability::None => false,
            ProcessCapability::AllowedCommands(allowed) => {
                // Check if the command matches any allowed commands
                allowed.iter().any(|cmd| {
                    // Exact match
                    if cmd == command {
                        return true;
                    }
                    
                    // Wildcard match
                    if cmd.ends_with("*") {
                        let prefix = &cmd[..cmd.len() - 1];
                        return command.starts_with(prefix);
                    }
                    
                    false
                })
            }
            ProcessCapability::Full => true,
        }
    }
}

impl CapabilityVerifier for ProcessVerifier {
    fn verify(&self, operation: &str, params: &[&str]) -> Result<()> {
        match operation {
            "exec" | "spawn" => {
                if params.is_empty() {
                    return Err(Error::Capability("Missing command".to_string()));
                }
                
                let command = params[0];
                if !self.is_command_allowed(command) {
                    return Err(Error::SecurityViolation(
                        format!("Process execution denied: {}", command)
                    ));
                }
            }
            _ => {
                return Err(Error::Capability(format!("Unknown process operation: {}", operation)));
            }
        }
        
        Ok(())
    }
}

/// Time capability verifier
pub struct TimeVerifier {
    capability: TimeCapability,
}

impl TimeVerifier {
    /// Create a new time verifier
    pub fn new(capability: TimeCapability) -> Self {
        Self { capability }
    }
    
    /// Check if setting time is allowed
    pub fn can_set_time(&self) -> bool {
        matches!(self.capability, TimeCapability::Full)
    }
}

impl CapabilityVerifier for TimeVerifier {
    fn verify(&self, operation: &str, params: &[&str]) -> Result<()> {
        match operation {
            "get" => {
                // Reading time is always allowed
                Ok(())
            }
            "set" => {
                if !self.can_set_time() {
                    return Err(Error::SecurityViolation("Setting time is not allowed".to_string()));
                }
                Ok(())
            }
            _ => {
                Err(Error::Capability(format!("Unknown time operation: {}", operation)))
            }
        }
    }
}

/// Random capability verifier
pub struct RandomVerifier {
    capability: RandomCapability,
}

impl RandomVerifier {
    /// Create a new random verifier
    pub fn new(capability: RandomCapability) -> Self {
        Self { capability }
    }
    
    /// Check if secure random generation is allowed
    pub fn can_secure_random(&self) -> bool {
        matches!(self.capability, RandomCapability::Full)
    }
    
    /// Check if any random generation is allowed
    pub fn can_pseudo_random(&self) -> bool {
        !matches!(self.capability, RandomCapability::None)
    }
}

impl CapabilityVerifier for RandomVerifier {
    fn verify(&self, operation: &str, params: &[&str]) -> Result<()> {
        match operation {
            "pseudo" => {
                if !self.can_pseudo_random() {
                    return Err(Error::SecurityViolation("Pseudo-random generation is not allowed".to_string()));
                }
                Ok(())
            }
            "secure" => {
                if !self.can_secure_random() {
                    return Err(Error::SecurityViolation("Secure random generation is not allowed".to_string()));
                }
                Ok(())
            }
            _ => {
                Err(Error::Capability(format!("Unknown random operation: {}", operation)))
            }
        }
    }
}

/// Central capability manager that combines all verifiers
pub struct CapabilityManager {
    /// Network verifier
    pub network: NetworkVerifier,
    
    /// Filesystem verifier
    pub filesystem: FilesystemVerifier,
    
    /// Environment verifier
    pub environment: EnvironmentVerifier,
    
    /// Process verifier
    pub process: ProcessVerifier,
    
    /// Time verifier
    pub time: TimeVerifier,
    
    /// Random verifier
    pub random: RandomVerifier,
}

impl CapabilityManager {
    /// Create a new capability manager
    pub fn new(
        network: NetworkCapability,
        filesystem: FilesystemCapability,
        environment: EnvironmentCapability,
        process: ProcessCapability,
        time: TimeCapability,
        random: RandomCapability,
    ) -> Self {
        Self {
            network: NetworkVerifier::new(network),
            filesystem: FilesystemVerifier::new(filesystem),
            environment: EnvironmentVerifier::new(environment),
            process: ProcessVerifier::new(process),
            time: TimeVerifier::new(time),
            random: RandomVerifier::new(random),
        }
    }
    
    /// Check if an operation is allowed based on its capability domain
    pub fn verify(&self, domain: &str, operation: &str, params: &[&str]) -> Result<()> {
        match domain {
            "network" => self.network.verify(operation, params),
            "filesystem" | "fs" => self.filesystem.verify(operation, params),
            "environment" | "env" => self.environment.verify(operation, params),
            "process" | "proc" => self.process.verify(operation, params),
            "time" => self.time.verify(operation, params),
            "random" | "rand" => self.random.verify(operation, params),
            _ => Err(Error::Capability(format!("Unknown capability domain: {}", domain))),
        }
    }
}
