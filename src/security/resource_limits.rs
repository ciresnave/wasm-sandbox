//! Resource limits implementation for the sandbox

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::error::{Error, Result};
use crate::security::{
    MemoryLimits, CpuLimits, IoLimits, TimeLimits, ResourceLimits
};

/// Memory resource tracker
#[derive(Debug, Clone)]
pub struct MemoryResourceTracker {
    /// Maximum memory pages
    pub max_memory_pages: u32,
    
    /// Current memory pages
    current_pages: Arc<AtomicU64>,
    
    /// Peak memory pages
    peak_pages: Arc<AtomicU64>,
    
    /// Memory growth rate tracking
    growth_tracker: Arc<Mutex<MemoryGrowthTracker>>,
}

/// Tracking for memory growth rate
#[derive(Debug)]
struct MemoryGrowthTracker {
    /// Maximum growth rate
    max_rate: Option<u32>,
    
    /// Last memory size
    last_size: u64,
    
    /// Last check time
    last_check: Instant,
    
    /// Growth events in current window
    growth_events: Vec<(Instant, u64)>,
    
    /// Window duration
    window: Duration,
}

impl MemoryResourceTracker {
    /// Create a new memory resource tracker
    pub fn new(limits: &MemoryLimits) -> Self {
        let growth_tracker = MemoryGrowthTracker {
            max_rate: limits.max_growth_rate,
            last_size: 0,
            last_check: Instant::now(),
            growth_events: Vec::new(),
            window: Duration::from_secs(1), // 1 second window for growth rate
        };
        
        Self {
            max_memory_pages: limits.max_memory_pages,
            current_pages: Arc::new(AtomicU64::new(limits.reserved_memory_pages as u64)),
            peak_pages: Arc::new(AtomicU64::new(limits.reserved_memory_pages as u64)),
            growth_tracker: Arc::new(Mutex::new(growth_tracker)),
        }
    }
    
    /// Check if memory allocation is allowed
    pub fn check_allocation(&self, pages: u32) -> Result<()> {
        let current = self.current_pages.load(Ordering::Acquire);
        let requested = current + pages as u64;
        
        if requested > self.max_memory_pages as u64 {
            return Err(Error::ResourceLimit(
                format!("Memory allocation of {} pages would exceed limit of {} pages", 
                    pages, self.max_memory_pages)
            ));
        }
        
        // Check growth rate
        if let Some(max_rate) = self.growth_tracker.lock().unwrap().max_rate {
            let now = Instant::now();
            let mut tracker = self.growth_tracker.lock().unwrap();
            
            // Clean up old events
            let cutoff = now - tracker.window;
            tracker.growth_events.retain(|(time, _)| *time >= cutoff);
            
            // Add current growth
            tracker.growth_events.push((now, pages as u64));
            
            // Calculate total growth in window
            let total_growth: u64 = tracker.growth_events.iter().map(|(_, size)| *size).sum();
            
            if total_growth > max_rate as u64 {
                return Err(Error::ResourceLimit(
                    format!("Memory growth rate of {} pages/s exceeds limit of {} pages/s",
                        total_growth, max_rate)
                ));
            }
            
            tracker.last_size = requested;
            tracker.last_check = now;
        }
        
        Ok(())
    }
    
    /// Update memory usage
    pub fn update(&self, pages: u32) {
        let current = self.current_pages.fetch_add(pages as u64, Ordering::AcqRel) + pages as u64;
        let mut peak = self.peak_pages.load(Ordering::Acquire);
        
        while current > peak {
            match self.peak_pages.compare_exchange_weak(
                peak,
                current,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
    }
    
    /// Get current memory usage in pages
    pub fn current_pages(&self) -> u64 {
        self.current_pages.load(Ordering::Acquire)
    }
    
    /// Get peak memory usage in pages
    pub fn peak_pages(&self) -> u64 {
        self.peak_pages.load(Ordering::Acquire)
    }
    
    /// Reset peak memory usage
    pub fn reset_peak(&self) {
        self.peak_pages.store(
            self.current_pages.load(Ordering::Acquire),
            Ordering::Release
        );
    }
}

/// CPU resource tracker
#[derive(Debug, Clone)]
pub struct CpuResourceTracker {
    /// Maximum execution time
    pub max_execution_time: Duration,
    
    /// Target CPU usage percentage
    pub cpu_usage_percentage: Option<u8>,
    
    /// Maximum number of threads
    pub max_threads: Option<u32>,
    
    /// Execution start time
    start_time: Arc<Mutex<Option<Instant>>>,
    
    /// Total execution time
    total_time: Arc<AtomicU64>,
    
    /// Number of active threads
    active_threads: Arc<AtomicU64>,
}

impl CpuResourceTracker {
    /// Create a new CPU resource tracker
    pub fn new(limits: &CpuLimits) -> Self {
        Self {
            max_execution_time: Duration::from_millis(limits.max_execution_time_ms),
            cpu_usage_percentage: limits.cpu_usage_percentage,
            max_threads: limits.max_threads,
            start_time: Arc::new(Mutex::new(None)),
            total_time: Arc::new(AtomicU64::new(0)),
            active_threads: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Start execution tracking
    pub fn start_execution(&self) {
        let mut start = self.start_time.lock().unwrap();
        if start.is_none() {
            *start = Some(Instant::now());
        }
    }
    
    /// Stop execution tracking
    pub fn stop_execution(&self) {
        let mut start_lock = self.start_time.lock().unwrap();
        if let Some(start) = *start_lock {
            let elapsed = start.elapsed();
            self.total_time.fetch_add(elapsed.as_millis() as u64, Ordering::AcqRel);
            *start_lock = None;
        }
    }
    
    /// Check if execution time limit has been exceeded
    pub fn check_time_limit(&self) -> Result<()> {
        let total = self.total_time.load(Ordering::Acquire);
        
        // Add current execution time if running
        let mut current_total = total;
        let start_lock = self.start_time.lock().unwrap();
        if let Some(start) = *start_lock {
            current_total += start.elapsed().as_millis() as u64;
        }
        
        if current_total > self.max_execution_time.as_millis() as u64 {
            return Err(Error::Timeout(current_total));
        }
        
        Ok(())
    }
    
    /// Register a new thread
    pub fn register_thread(&self) -> Result<()> {
        if let Some(max) = self.max_threads {
            let current = self.active_threads.fetch_add(1, Ordering::AcqRel) + 1;
            if current > max as u64 {
                // Rollback the increment
                self.active_threads.fetch_sub(1, Ordering::AcqRel);
                return Err(Error::ResourceLimit(
                    format!("Thread limit of {} exceeded", max)
                ));
            }
        } else {
            self.active_threads.fetch_add(1, Ordering::AcqRel);
        }
        
        Ok(())
    }
    
    /// Unregister a thread
    pub fn unregister_thread(&self) {
        self.active_threads.fetch_sub(1, Ordering::AcqRel);
    }
    
    /// Get total execution time in milliseconds
    pub fn total_time_ms(&self) -> u64 {
        let total = self.total_time.load(Ordering::Acquire);
        
        // Add current execution time if running
        let mut current_total = total;
        let start_lock = self.start_time.lock().unwrap();
        if let Some(start) = *start_lock {
            current_total += start.elapsed().as_millis() as u64;
        }
        
        current_total
    }
    
    /// Get number of active threads
    pub fn active_threads(&self) -> u32 {
        self.active_threads.load(Ordering::Acquire) as u32
    }
    
    /// Apply CPU usage throttling
    pub fn apply_throttling(&self) {
        if let Some(percentage) = self.cpu_usage_percentage {
            if percentage >= 100 {
                return; // No throttling needed
            }
            
            // Simple throttling: sleep proportionally to target usage
            if percentage > 0 {
                // Example: For 50% CPU usage, sleep for 1ms after every 1ms of execution
                let sleep_time_ns = (100 - percentage) as u64 * 10_000; // convert to nanoseconds
                std::thread::sleep(Duration::from_nanos(sleep_time_ns));
            }
        }
    }
}

/// I/O resource tracker
#[derive(Debug, Clone)]
pub struct IoResourceTracker {
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
    
    /// Current number of open files
    open_files: Arc<AtomicU64>,
    
    /// Total bytes read
    total_read: Arc<AtomicU64>,
    
    /// Total bytes written
    total_write: Arc<AtomicU64>,
    
    /// Rate tracking
    rate_tracker: Arc<Mutex<IoRateTracker>>,
}

/// I/O rate tracking
#[derive(Debug)]
struct IoRateTracker {
    /// Read events in the current window
    read_events: Vec<(Instant, u64)>,
    
    /// Write events in the current window
    write_events: Vec<(Instant, u64)>,
    
    /// Window duration
    window: Duration,
}

impl IoResourceTracker {
    /// Create a new I/O resource tracker
    pub fn new(limits: &IoLimits) -> Self {
        let rate_tracker = IoRateTracker {
            read_events: Vec::new(),
            write_events: Vec::new(),
            window: Duration::from_secs(1), // 1 second window
        };
        
        Self {
            max_open_files: limits.max_open_files,
            max_read_bytes_per_second: limits.max_read_bytes_per_second,
            max_write_bytes_per_second: limits.max_write_bytes_per_second,
            max_total_read_bytes: limits.max_total_read_bytes,
            max_total_write_bytes: limits.max_total_write_bytes,
            open_files: Arc::new(AtomicU64::new(0)),
            total_read: Arc::new(AtomicU64::new(0)),
            total_write: Arc::new(AtomicU64::new(0)),
            rate_tracker: Arc::new(Mutex::new(rate_tracker)),
        }
    }
    
    /// Register a file open
    pub fn register_open(&self) -> Result<()> {
        let current = self.open_files.fetch_add(1, Ordering::AcqRel) + 1;
        if current > self.max_open_files as u64 {
            // Rollback the increment
            self.open_files.fetch_sub(1, Ordering::AcqRel);
            return Err(Error::ResourceLimit(
                format!("Open file limit of {} exceeded", self.max_open_files)
            ));
        }
        
        Ok(())
    }
    
    /// Register a file close
    pub fn register_close(&self) {
        self.open_files.fetch_sub(1, Ordering::AcqRel);
    }
    
    /// Register a read operation
    pub fn register_read(&self, bytes: u64) -> Result<()> {
        // Update total
        let total = self.total_read.fetch_add(bytes, Ordering::AcqRel) + bytes;
        
        // Check total limit
        if let Some(limit) = self.max_total_read_bytes {
            if total > limit {
                return Err(Error::ResourceLimit(
                    format!("Total read limit of {} bytes exceeded", limit)
                ));
            }
        }
        
        // Check rate limit
        if let Some(rate_limit) = self.max_read_bytes_per_second {
            let now = Instant::now();
            let mut tracker = self.rate_tracker.lock().unwrap();
            
            // Clean up old events
            let cutoff = now - tracker.window;
            tracker.read_events.retain(|(time, _)| *time >= cutoff);
            
            // Add current read
            tracker.read_events.push((now, bytes));
            
            // Calculate total in window
            let window_total: u64 = tracker.read_events.iter().map(|(_, size)| *size).sum();
            
            if window_total > rate_limit {
                return Err(Error::ResourceLimit(
                    format!("Read rate limit of {} bytes/s exceeded", rate_limit)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Register a write operation
    pub fn register_write(&self, bytes: u64) -> Result<()> {
        // Update total
        let total = self.total_write.fetch_add(bytes, Ordering::AcqRel) + bytes;
        
        // Check total limit
        if let Some(limit) = self.max_total_write_bytes {
            if total > limit {
                return Err(Error::ResourceLimit(
                    format!("Total write limit of {} bytes exceeded", limit)
                ));
            }
        }
        
        // Check rate limit
        if let Some(rate_limit) = self.max_write_bytes_per_second {
            let now = Instant::now();
            let mut tracker = self.rate_tracker.lock().unwrap();
            
            // Clean up old events
            let cutoff = now - tracker.window;
            tracker.write_events.retain(|(time, _)| *time >= cutoff);
            
            // Add current write
            tracker.write_events.push((now, bytes));
            
            // Calculate total in window
            let window_total: u64 = tracker.write_events.iter().map(|(_, size)| *size).sum();
            
            if window_total > rate_limit {
                return Err(Error::ResourceLimit(
                    format!("Write rate limit of {} bytes/s exceeded", rate_limit)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Get number of open files
    pub fn open_files(&self) -> u32 {
        self.open_files.load(Ordering::Acquire) as u32
    }
    
    /// Get total bytes read
    pub fn total_read(&self) -> u64 {
        self.total_read.load(Ordering::Acquire)
    }
    
    /// Get total bytes written
    pub fn total_write(&self) -> u64 {
        self.total_write.load(Ordering::Acquire)
    }
    
    /// Get current read rate in bytes per second
    pub fn read_rate(&self) -> u64 {
        let tracker = self.rate_tracker.lock().unwrap();
        let now = Instant::now();
        let cutoff = now - tracker.window;
        
        // Sum bytes in current window
        tracker.read_events
            .iter()
            .filter(|(time, _)| *time >= cutoff)
            .map(|(_, size)| *size)
            .sum()
    }
    
    /// Get current write rate in bytes per second
    pub fn write_rate(&self) -> u64 {
        let tracker = self.rate_tracker.lock().unwrap();
        let now = Instant::now();
        let cutoff = now - tracker.window;
        
        // Sum bytes in current window
        tracker.write_events
            .iter()
            .filter(|(time, _)| *time >= cutoff)
            .map(|(_, size)| *size)
            .sum()
    }
}

/// Time resource tracker
#[derive(Debug, Clone)]
pub struct TimeResourceTracker {
    /// Maximum total time
    pub max_total_time: Duration,
    
    /// Maximum idle time
    pub max_idle_time: Option<Duration>,
    
    /// Start time
    start_time: Arc<Mutex<Instant>>,
    
    /// Last activity time
    last_activity: Arc<Mutex<Instant>>,
}

impl TimeResourceTracker {
    /// Create a new time resource tracker
    pub fn new(limits: &TimeLimits) -> Self {
        let now = Instant::now();
        
        Self {
            max_total_time: Duration::from_millis(limits.max_total_time_ms),
            max_idle_time: limits.max_idle_time_ms.map(Duration::from_millis),
            start_time: Arc::new(Mutex::new(now)),
            last_activity: Arc::new(Mutex::new(now)),
        }
    }
    
    /// Register activity to reset idle timer
    pub fn register_activity(&self) {
        *self.last_activity.lock().unwrap() = Instant::now();
    }
    
    /// Check if time limits have been exceeded
    pub fn check_limits(&self) -> Result<()> {
        let now = Instant::now();
        
        // Check total time
        let elapsed = now.duration_since(*self.start_time.lock().unwrap());
        if elapsed > self.max_total_time {
            return Err(Error::Timeout(elapsed.as_millis() as u64));
        }
        
        // Check idle time
        if let Some(idle_limit) = self.max_idle_time {
            let idle_time = now.duration_since(*self.last_activity.lock().unwrap());
            if idle_time > idle_limit {
                return Err(Error::ResourceLimit(
                    format!("Idle time limit of {}ms exceeded", idle_limit.as_millis())
                ));
            }
        }
        
        Ok(())
    }
    
    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        let now = Instant::now();
        now.duration_since(*self.start_time.lock().unwrap()).as_millis() as u64
    }
    
    /// Get idle time in milliseconds
    pub fn idle_ms(&self) -> u64 {
        let now = Instant::now();
        now.duration_since(*self.last_activity.lock().unwrap()).as_millis() as u64
    }
}

/// Main resource limit manager
#[derive(Debug, Clone)]
pub struct ResourceLimitManager {
    /// Memory resource tracker
    pub memory: MemoryResourceTracker,
    
    /// CPU resource tracker
    pub cpu: CpuResourceTracker,
    
    /// I/O resource tracker
    pub io: IoResourceTracker,
    
    /// Time resource tracker
    pub time: TimeResourceTracker,
    
    /// Fuel limit and usage
    pub fuel: Option<Arc<AtomicU64>>,
}

impl ResourceLimitManager {
    /// Create a new resource limit manager
    pub fn new(limits: &ResourceLimits) -> Self {
        let fuel = limits.fuel.map(|f| Arc::new(AtomicU64::new(f)));
        
        Self {
            memory: MemoryResourceTracker::new(&limits.memory),
            cpu: CpuResourceTracker::new(&limits.cpu),
            io: IoResourceTracker::new(&limits.io),
            time: TimeResourceTracker::new(&limits.time),
            fuel,
        }
    }
    
    /// Check all resource limits
    pub fn check_all_limits(&self) -> Result<()> {
        // Check time limits
        self.time.check_limits()?;
        
        // Check CPU limits
        self.cpu.check_time_limit()?;
        
        // Check fuel limits
        if let Some(fuel) = &self.fuel {
            if fuel.load(Ordering::Acquire) == 0 {
                return Err(Error::ResourceLimit("Fuel limit exceeded".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Consume fuel
    pub fn consume_fuel(&self, amount: u64) -> Result<()> {
        if let Some(fuel) = &self.fuel {
            let current = fuel.load(Ordering::Acquire);
            if current < amount {
                return Err(Error::ResourceLimit(
                    format!("Not enough fuel: requested {}, available {}", amount, current)
                ));
            }
            
            fuel.fetch_sub(amount, Ordering::AcqRel);
        }
        
        Ok(())
    }
    
    /// Add fuel
    pub fn add_fuel(&self, amount: u64) -> Result<()> {
        if let Some(fuel) = &self.fuel {
            fuel.fetch_add(amount, Ordering::AcqRel);
            Ok(())
        } else {
            Err(Error::UnsupportedOperation("Fuel metering is not enabled".to_string()))
        }
    }
    
    /// Reset fuel
    pub fn reset_fuel(&self, amount: u64) -> Result<()> {
        if let Some(fuel) = &self.fuel {
            fuel.store(amount, Ordering::Release);
            Ok(())
        } else {
            Err(Error::UnsupportedOperation("Fuel metering is not enabled".to_string()))
        }
    }
    
    /// Get remaining fuel
    pub fn get_remaining_fuel(&self) -> Option<u64> {
        self.fuel.as_ref().map(|f| f.load(Ordering::Acquire))
    }
    
    /// Start a background monitor thread for resource limits
    pub fn start_monitor(&self) -> std::thread::JoinHandle<()> {
        // Clone the trackers
        let cpu_tracker = self.cpu.clone();
        let time_tracker = self.time.clone();
        
        std::thread::spawn(move || {
            let check_interval = Duration::from_millis(100); // Check every 100ms
            
            loop {
                // Sleep
                std::thread::sleep(check_interval);
                
                // Register activity for time tracker (monitor itself counts as activity)
                time_tracker.register_activity();
                
                // Apply CPU throttling
                cpu_tracker.apply_throttling();
                
                // Check limits
                // NOTE: We don't handle errors here because the monitor thread
                // doesn't have a way to signal the main thread directly.
                // This is just a background check - the main thread should also
                // check limits at critical points.
                let _ = time_tracker.check_limits();
                let _ = cpu_tracker.check_time_limit();
            }
        })
    }
}
