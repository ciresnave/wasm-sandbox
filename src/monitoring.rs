//! Resource monitoring and usage tracking

use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::error::ResourceKind;
use crate::InstanceId;

/// Detailed resource usage information with timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedResourceUsage {
    pub memory: MemoryUsage,
    pub cpu: CpuUsage,
    pub io: IoUsage,
    pub timeline: Vec<ResourceSnapshot>,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub current_bytes: usize,
    pub peak_bytes: usize,
    pub allocations: u64,
    pub deallocations: u64,
}

/// CPU usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    pub time_spent: Duration,
    pub instructions_executed: u64,
    pub function_calls: u64,
}

/// I/O usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoUsage {
    pub files_opened: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub network_requests: u64,
}

/// Resource snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub timestamp: u64, // milliseconds since epoch
    pub memory_bytes: usize,
    pub cpu_time_ms: u64,
    pub active_handles: u32,
}

/// Resource monitor for tracking usage patterns
#[derive(Debug)]
pub struct ResourceMonitor {
    _instance_id: Option<InstanceId>,
    start_time: Instant,
    memory_usage: MemoryUsage,
    cpu_usage: CpuUsage,
    io_usage: IoUsage,
    snapshots: Vec<ResourceSnapshot>,
    snapshot_interval: Duration,
    last_snapshot: Instant,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(instance_id: Option<InstanceId>) -> Self {
        let now = Instant::now();
        Self {
            _instance_id: instance_id,
            start_time: now,
            memory_usage: MemoryUsage {
                current_bytes: 0,
                peak_bytes: 0,
                allocations: 0,
                deallocations: 0,
            },
            cpu_usage: CpuUsage {
                time_spent: Duration::from_millis(0),
                instructions_executed: 0,
                function_calls: 0,
            },
            io_usage: IoUsage {
                files_opened: 0,
                bytes_read: 0,
                bytes_written: 0,
                network_requests: 0,
            },
            snapshots: Vec::new(),
            snapshot_interval: Duration::from_secs(1),
            last_snapshot: now,
        }
    }

    /// Update memory usage
    pub fn update_memory(&mut self, current_bytes: usize) {
        self.memory_usage.current_bytes = current_bytes;
        if current_bytes > self.memory_usage.peak_bytes {
            self.memory_usage.peak_bytes = current_bytes;
        }
        self.take_snapshot_if_needed();
    }

    /// Record memory allocation
    pub fn record_allocation(&mut self, _bytes: usize) {
        self.memory_usage.allocations += 1;
    }

    /// Record memory deallocation
    pub fn record_deallocation(&mut self, _bytes: usize) {
        self.memory_usage.deallocations += 1;
    }

    /// Update CPU usage
    pub fn update_cpu_time(&mut self, time_spent: Duration) {
        self.cpu_usage.time_spent = time_spent;
        self.take_snapshot_if_needed();
    }

    /// Record function call
    pub fn record_function_call(&mut self) {
        self.cpu_usage.function_calls += 1;
    }

    /// Record instructions executed
    pub fn record_instructions(&mut self, count: u64) {
        self.cpu_usage.instructions_executed += count;
    }

    /// Record file operation
    pub fn record_file_open(&mut self) {
        self.io_usage.files_opened += 1;
    }

    /// Record file read
    pub fn record_file_read(&mut self, bytes: u64) {
        self.io_usage.bytes_read += bytes;
    }

    /// Record file write
    pub fn record_file_write(&mut self, bytes: u64) {
        self.io_usage.bytes_written += bytes;
    }

    /// Record network request
    pub fn record_network_request(&mut self) {
        self.io_usage.network_requests += 1;
    }

    /// Take a snapshot if enough time has passed
    fn take_snapshot_if_needed(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_snapshot) >= self.snapshot_interval {
            self.take_snapshot();
            self.last_snapshot = now;
        }
    }

    /// Take a resource snapshot
    pub fn take_snapshot(&mut self) {
        let snapshot = ResourceSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            memory_bytes: self.memory_usage.current_bytes,
            cpu_time_ms: self.cpu_usage.time_spent.as_millis() as u64,
            active_handles: (self.io_usage.files_opened - self.io_usage.bytes_written.min(self.io_usage.files_opened)) as u32,
        };
        
        self.snapshots.push(snapshot);
        
        // Keep only last 100 snapshots to prevent memory growth
        if self.snapshots.len() > 100 {
            self.snapshots.remove(0);
        }
    }

    /// Get detailed resource usage
    pub fn get_detailed_usage(&self) -> DetailedResourceUsage {
        DetailedResourceUsage {
            memory: self.memory_usage.clone(),
            cpu: self.cpu_usage.clone(),
            io: self.io_usage.clone(),
            timeline: self.snapshots.clone(),
        }
    }

    /// Get current resource usage
    pub fn get_current_usage(&self) -> DetailedResourceUsage {
        DetailedResourceUsage {
            memory: self.memory_usage.clone(),
            cpu: self.cpu_usage.clone(),
            io: self.io_usage.clone(),
            timeline: self.snapshots.clone(),
        }
    }

    /// Check if resource limit is exceeded
    pub fn check_resource_limit(&self, kind: &ResourceKind, limit: u64) -> Option<(u64, String)> {
        match kind {
            ResourceKind::Memory => {
                let used = self.memory_usage.current_bytes as u64;
                if used > limit {
                    Some((used, format!("Memory usage {} bytes exceeds limit {} bytes", used, limit)))
                } else {
                    None
                }
            },
            ResourceKind::CpuTime => {
                let used = self.cpu_usage.time_spent.as_millis() as u64;
                if used > limit {
                    Some((used, format!("CPU time {} ms exceeds limit {} ms", used, limit)))
                } else {
                    None
                }
            },
            ResourceKind::ExecutionTime => {
                let used = Instant::now().duration_since(self.start_time).as_millis() as u64;
                if used > limit {
                    Some((used, format!("Execution time {} ms exceeds limit {} ms", used, limit)))
                } else {
                    None
                }
            },
            ResourceKind::FileHandles => {
                let used = self.io_usage.files_opened;
                if used > limit {
                    Some((used, format!("File handles {} exceeds limit {}", used, limit)))
                } else {
                    None
                }
            },
            ResourceKind::NetworkConnections => {
                let used = self.io_usage.network_requests;
                if used > limit {
                    Some((used, format!("Network requests {} exceeds limit {}", used, limit)))
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    /// Get utilization percentage for a resource
    pub fn get_utilization(&self, kind: &ResourceKind, limit: u64) -> f64 {
        if limit == 0 {
            return 0.0;
        }
        
        let used = match kind {
            ResourceKind::Memory => self.memory_usage.current_bytes as u64,
            ResourceKind::CpuTime => self.cpu_usage.time_spent.as_millis() as u64,
            ResourceKind::ExecutionTime => Instant::now().duration_since(self.start_time).as_millis() as u64,
            ResourceKind::FileHandles => self.io_usage.files_opened,
            ResourceKind::NetworkConnections => self.io_usage.network_requests,
            _ => 0,
        };
        
        (used as f64 / limit as f64 * 100.0).min(100.0)
    }

    /// Reset usage statistics
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.memory_usage = MemoryUsage {
            current_bytes: 0,
            peak_bytes: 0,
            allocations: 0,
            deallocations: 0,
        };
        self.cpu_usage = CpuUsage {
            time_spent: Duration::from_millis(0),
            instructions_executed: 0,
            function_calls: 0,
        };
        self.io_usage = IoUsage {
            files_opened: 0,
            bytes_read: 0,
            bytes_written: 0,
            network_requests: 0,
        };
        self.snapshots.clear();
        self.last_snapshot = Instant::now();
    }
}
