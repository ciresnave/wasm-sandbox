//! Security audit and logging

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use serde::{Serialize, Deserialize};

/// Severity level for audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditSeverity {
    /// Informational message
    Info,
    
    /// Warning message
    Warning,
    
    /// Error message
    Error,
    
    /// Critical security event
    Critical,
}

/// Type of audit event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Module loaded
    ModuleLoaded { 
        /// Module ID
        id: String, 
        
        /// Module size
        size: usize 
    },
    
    /// Instance created
    InstanceCreated { 
        /// Instance ID
        id: String 
    },
    
    /// Instance terminated
    InstanceTerminated { 
        /// Instance ID
        id: String, 
        
        /// Exit code
        exit_code: Option<i32> 
    },
    
    /// Function called
    FunctionCall { 
        /// Instance ID
        instance_id: String, 
        
        /// Function name
        function_name: String 
    },
    
    /// Resource limit reached
    ResourceLimit { 
        /// Instance ID
        instance_id: String, 
        
        /// Resource type
        resource: String, 
        
        /// Limit type
        limit_type: String, 
        
        /// Limit value
        limit: u64,
        
        /// Attempted value
        attempted: u64 
    },
    
    /// Capability violation
    CapabilityViolation { 
        /// Instance ID
        instance_id: String, 
        
        /// Capability domain
        domain: String, 
        
        /// Operation
        operation: String 
    },
    
    /// Host function call
    HostFunctionCall { 
        /// Instance ID
        instance_id: String, 
        
        /// Function name
        function_name: String 
    },
    
    /// Memory access
    MemoryAccess { 
        /// Instance ID
        instance_id: String, 
        
        /// Access type
        access_type: String, 
        
        /// Memory address
        address: u32, 
        
        /// Size in bytes
        size: usize 
    },
    
    /// Custom event
    Custom { 
        /// Event type
        event_type: String, 
        
        /// Event data
        data: String 
    },
}

/// Audit event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Timestamp
    pub timestamp: SystemTime,
    
    /// Severity level
    pub severity: AuditSeverity,
    
    /// Event type
    pub event_type: AuditEventType,
    
    /// Message
    pub message: String,
}

/// Audit logger for the sandbox
#[derive(Debug, Clone)]
pub struct AuditLogger {
    /// Log events
    events: Arc<Mutex<VecDeque<AuditEvent>>>,
    
    /// Maximum number of events to keep
    max_events: usize,
    
    /// Whether to log to stdout
    log_to_stdout: bool,
    
    /// Whether to log to a file
    log_to_file: bool,
    
    /// File path for logging
    file_path: Option<String>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::with_capacity(max_events))),
            max_events,
            log_to_stdout: false,
            log_to_file: false,
            file_path: None,
        }
    }
    
    /// Enable logging to stdout
    pub fn with_stdout(mut self) -> Self {
        self.log_to_stdout = true;
        self
    }
    
    /// Enable logging to a file
    pub fn with_file(mut self, file_path: &str) -> Self {
        self.log_to_file = true;
        self.file_path = Some(file_path.to_string());
        self
    }
    
    /// Log an event
    pub fn log(&self, severity: AuditSeverity, event_type: AuditEventType, message: &str) {
        let event = AuditEvent {
            timestamp: SystemTime::now(),
            severity,
            event_type,
            message: message.to_string(),
        };
        
        // Log to stdout if enabled
        if self.log_to_stdout {
            let timestamp = chrono::DateTime::<chrono::Utc>::from(event.timestamp)
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string();
                
            let level = match event.severity {
                AuditSeverity::Info => "INFO",
                AuditSeverity::Warning => "WARN",
                AuditSeverity::Error => "ERROR",
                AuditSeverity::Critical => "CRITICAL",
            };
            
            println!("[{}] {} - {} - {:?}", timestamp, level, event.message, event.event_type);
        }
        
        // Log to file if enabled
        if self.log_to_file {
            if let Some(file_path) = &self.file_path {
                // Open the file in append mode
                let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
                
                // Append to file
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)
                    .map(|mut file| {
                        use std::io::Write;
                        let _ = writeln!(file, "{}", json);
                    })
                    .ok();
            }
        }
        
        // Store in memory
        let mut events = self.events.lock().unwrap();
        events.push_back(event);
        
        // Remove oldest events if over capacity
        while events.len() > self.max_events {
            events.pop_front();
        }
    }
    
    /// Log an info event
    pub fn info(&self, event_type: AuditEventType, message: &str) {
        self.log(AuditSeverity::Info, event_type, message);
    }
    
    /// Log a warning event
    pub fn warning(&self, event_type: AuditEventType, message: &str) {
        self.log(AuditSeverity::Warning, event_type, message);
    }
    
    /// Log an error event
    pub fn error(&self, event_type: AuditEventType, message: &str) {
        self.log(AuditSeverity::Error, event_type, message);
    }
    
    /// Log a critical event
    pub fn critical(&self, event_type: AuditEventType, message: &str) {
        self.log(AuditSeverity::Critical, event_type, message);
    }
    
    /// Get all events
    pub fn get_events(&self) -> Vec<AuditEvent> {
        self.events.lock().unwrap().iter().cloned().collect()
    }
    
    /// Get events by severity
    pub fn get_events_by_severity(&self, severity: AuditSeverity) -> Vec<AuditEvent> {
        self.events.lock().unwrap()
            .iter()
            .filter(|e| e.severity == severity)
            .cloned()
            .collect()
    }
    
    /// Get events in a time range
    pub fn get_events_in_range(&self, start: SystemTime, end: SystemTime) -> Vec<AuditEvent> {
        self.events.lock().unwrap()
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }
    
    /// Clear all events
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

/// Security audit configuration
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Whether to enable auditing
    pub enabled: bool,
    
    /// Log to stdout
    pub log_to_stdout: bool,
    
    /// Log to file
    pub log_to_file: bool,
    
    /// File path for logging
    pub file_path: Option<String>,
    
    /// Maximum number of events to keep in memory
    pub max_events: usize,
    
    /// Minimum severity to log
    pub min_severity: AuditSeverity,
    
    /// Whether to log module loads
    pub log_module_loads: bool,
    
    /// Whether to log instance creation
    pub log_instance_creation: bool,
    
    /// Whether to log function calls
    pub log_function_calls: bool,
    
    /// Whether to log resource limit events
    pub log_resource_limits: bool,
    
    /// Whether to log capability violations
    pub log_capability_violations: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_to_stdout: false,
            log_to_file: false,
            file_path: None,
            max_events: 1000,
            min_severity: AuditSeverity::Info,
            log_module_loads: true,
            log_instance_creation: true,
            log_function_calls: true,
            log_resource_limits: true,
            log_capability_violations: true,
        }
    }
}

/// Security threat level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatLevel {
    /// No threat detected
    None,
    
    /// Low threat level
    Low,
    
    /// Medium threat level
    Medium,
    
    /// High threat level
    High,
    
    /// Critical threat level
    Critical,
}

/// Security scanner for audit logs
pub struct SecurityScanner {
    /// Audit logger
    logger: AuditLogger,
    
    /// Scan configuration
    config: ScanConfig,
}

/// Security scan configuration
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Threshold for capability violations to trigger a warning
    pub capability_violation_threshold: usize,
    
    /// Threshold for resource limit violations to trigger a warning
    pub resource_limit_threshold: usize,
    
    /// Scan interval
    pub scan_interval: Duration,
    
    /// Whether to detect memory access patterns
    pub detect_memory_access_patterns: bool,
    
    /// Whether to detect network access patterns
    pub detect_network_access_patterns: bool,
    
    /// Whether to detect filesystem access patterns
    pub detect_filesystem_access_patterns: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            capability_violation_threshold: 3,
            resource_limit_threshold: 5,
            scan_interval: Duration::from_secs(60),
            detect_memory_access_patterns: true,
            detect_network_access_patterns: true,
            detect_filesystem_access_patterns: true,
        }
    }
}

impl SecurityScanner {
    /// Create a new security scanner
    pub fn new(logger: AuditLogger, config: ScanConfig) -> Self {
        Self {
            logger,
            config,
        }
    }
    
    /// Scan audit logs for security threats
    pub fn scan(&self) -> Vec<SecurityThreat> {
        let mut threats = Vec::new();
        let events = self.logger.get_events();
        
        // Count capability violations
        let capability_violations = events.iter()
            .filter(|e| matches!(e.event_type, AuditEventType::CapabilityViolation { .. }))
            .count();
            
        if capability_violations >= self.config.capability_violation_threshold {
            threats.push(SecurityThreat {
                level: ThreatLevel::Medium,
                description: format!(
                    "High number of capability violations detected: {} (threshold: {})",
                    capability_violations,
                    self.config.capability_violation_threshold
                ),
                events: events.iter()
                    .filter(|e| matches!(e.event_type, AuditEventType::CapabilityViolation { .. }))
                    .cloned()
                    .collect(),
            });
        }
        
        // Count resource limit violations
        let resource_violations = events.iter()
            .filter(|e| matches!(e.event_type, AuditEventType::ResourceLimit { .. }))
            .count();
            
        if resource_violations >= self.config.resource_limit_threshold {
            threats.push(SecurityThreat {
                level: ThreatLevel::Medium,
                description: format!(
                    "High number of resource limit violations detected: {} (threshold: {})",
                    resource_violations,
                    self.config.resource_limit_threshold
                ),
                events: events.iter()
                    .filter(|e| matches!(e.event_type, AuditEventType::ResourceLimit { .. }))
                    .cloned()
                    .collect(),
            });
        }
        
        // Check for memory access patterns
        if self.config.detect_memory_access_patterns {
            let memory_accesses = events.iter()
                .filter(|e| matches!(e.event_type, AuditEventType::MemoryAccess { .. }))
                .collect::<Vec<_>>();
                
            // Detect potential buffer overflow attempts
            // (This is a simplified heuristic and should be more sophisticated in a real system)
            let mut suspicious_addresses = std::collections::HashSet::new();
            for event in &memory_accesses {
                if let AuditEventType::MemoryAccess { 
                    address, size, access_type, .. 
                } = &event.event_type {
                    if access_type == "write" && *size > 1024 && *address > 0xFFFF0000 {
                        suspicious_addresses.insert(*address);
                    }
                }
            }
            
            if suspicious_addresses.len() > 2 {
                threats.push(SecurityThreat {
                    level: ThreatLevel::High,
                    description: format!(
                        "Potential buffer overflow attempt detected: {} suspicious memory writes",
                        suspicious_addresses.len()
                    ),
                    events: memory_accesses.into_iter().cloned().collect(),
                });
            }
        }
        
        threats
    }
    
    /// Start a background scanning thread
    pub fn start_scanner(&self) -> std::thread::JoinHandle<()> {
        let logger = self.logger.clone();
        let config = self.config.clone();
        
        std::thread::spawn(move || {
            let scanner = SecurityScanner::new(logger.clone(), config);
            
            loop {
                // Sleep for the scan interval
                std::thread::sleep(scanner.config.scan_interval);
                
                // Scan for threats
                let threats = scanner.scan();
                
                // Log threats
                for threat in threats {
                    logger.log(
                        match threat.level {
                            ThreatLevel::None | ThreatLevel::Low => AuditSeverity::Info,
                            ThreatLevel::Medium => AuditSeverity::Warning,
                            ThreatLevel::High | ThreatLevel::Critical => AuditSeverity::Critical,
                        },
                        AuditEventType::Custom {
                            event_type: "security_threat".to_string(),
                            data: threat.description.clone(),
                        },
                        &format!("Security threat detected: {}", threat.description),
                    );
                }
            }
        })
    }
}

/// Security threat detection result
#[derive(Debug, Clone)]
pub struct SecurityThreat {
    /// Threat level
    pub level: ThreatLevel,
    
    /// Description of the threat
    pub description: String,
    
    /// Related audit events
    pub events: Vec<AuditEvent>,
}
