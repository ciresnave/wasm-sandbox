//! Common utilities for WebAssembly runtimes

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Serialize, Deserialize};

/// Function parameter conversion
pub trait ToWasmValues {
    /// Convert to WebAssembly values
    fn to_wasm_values(&self) -> Vec<u8>;
}

/// Function result conversion
pub trait FromWasmValues {
    /// Convert from WebAssembly values
    fn from_wasm_values(values: &[u8]) -> crate::Result<Self>
    where
        Self: Sized;
}

impl<T: Serialize> ToWasmValues for T {
    fn to_wasm_values(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap_or_default()
    }
}

impl<T: for<'de> Deserialize<'de>> FromWasmValues for T {
    fn from_wasm_values(values: &[u8]) -> crate::Result<Self> {
        rmp_serde::from_slice(values).map_err(|e| {
            crate::Error::Serialization {
                format: "messagepack".to_string(),
                operation: "deserialize".to_string(),
                reason: e.to_string(),
            }
        })
    }
}

/// Helper for tracking memory allocations
pub struct MemoryTracker {
    /// Current memory usage in bytes
    current: AtomicU64,
    
    /// Peak memory usage in bytes
    peak: AtomicU64,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        Self {
            current: AtomicU64::new(0),
            peak: AtomicU64::new(0),
        }
    }
    
    /// Allocate memory and track it
    pub fn allocate(&self, size: u64) {
        let current = self.current.fetch_add(size, Ordering::SeqCst) + size;
        let mut peak = self.peak.load(Ordering::SeqCst);
        
        while current > peak {
            match self.peak.compare_exchange_weak(
                peak,
                current,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(actual_peak) => peak = actual_peak,
            }
        }
    }
    
    /// Deallocate memory and track it
    pub fn deallocate(&self, size: u64) {
        self.current.fetch_sub(size, Ordering::SeqCst);
    }
    
    /// Get current memory usage in bytes
    pub fn current(&self) -> u64 {
        self.current.load(Ordering::SeqCst)
    }
    
    /// Get peak memory usage in bytes
    pub fn peak(&self) -> u64 {
        self.peak.load(Ordering::SeqCst)
    }
    
    /// Reset peak memory usage
    pub fn reset_peak(&self) {
        self.peak.store(self.current.load(Ordering::SeqCst), Ordering::SeqCst);
    }
}

/// Helper for finding WASI files
pub fn locate_wasi_modules() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Check common paths for WASI modules
    let common_paths = [
        // Local project paths
        "./wasi",
        "./target/wasm32-wasi",
        
        // System paths
        "/usr/local/lib/wasi",
        "/usr/lib/wasi",
    ];
    
    for path in common_paths.iter() {
        let path = PathBuf::from(path);
        if path.exists() {
            paths.push(path);
        }
    }
    
    // Check environment variables
    if let Ok(path) = std::env::var("WASI_PATH") {
        for p in path.split(':') {
            let path = PathBuf::from(p);
            if path.exists() {
                paths.push(path);
            }
        }
    }
    
    paths
}

/// WebAssembly compilation options
#[derive(Debug, Clone)]
pub struct CompilationOptions {
    /// Optimization level (0-3)
    pub optimization_level: u8,
    
    /// Whether to generate debug information
    pub debug_info: bool,
    
    /// Whether to perform WASM validation
    pub validate: bool,
    
    /// Memory alignment in bytes
    pub memory_alignment: Option<u64>,
    
    /// Whether to consume fuel for metering
    pub consume_fuel: bool,
}

impl Default for CompilationOptions {
    fn default() -> Self {
        Self {
            optimization_level: 1,
            debug_info: false,
            validate: true,
            memory_alignment: None,
            consume_fuel: true,
        }
    }
}

/// Helper for working with memory regions
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    /// Base address
    pub base: u32,
    
    /// Size in bytes
    pub size: u32,
}

impl MemoryRegion {
    /// Create a new memory region
    pub fn new(base: u32, size: u32) -> Self {
        Self { base, size }
    }
    
    /// Check if an address is within this region
    pub fn contains(&self, addr: u32) -> bool {
        addr >= self.base && addr < self.base + self.size
    }
    
    /// Get the end address (exclusive)
    pub fn end(&self) -> u32 {
        self.base + self.size
    }
}

/// Utility for mapping memory regions
pub struct MemoryMap {
    /// Allocated regions
    regions: Vec<MemoryRegion>,
    
    /// Total size
    size: u32,
}

impl MemoryMap {
    /// Create a new memory map
    pub fn new(size: u32) -> Self {
        Self {
            regions: Vec::new(),
            size,
        }
    }
    
    /// Allocate a region of the given size
    /// 
    /// Returns the base address of the allocated region, or None if allocation failed
    pub fn allocate(&mut self, size: u32, align: u32) -> Option<u32> {
        if size == 0 {
            return Some(0);
        }
        
        let align_mask = align - 1;
        let mut base = 0;
        
        // Find a free region
        while base < self.size {
            // Align the base address
            let aligned_base = (base + align_mask) & !align_mask;
            
            // Check if we have enough space
            if aligned_base + size > self.size {
                return None;
            }
            
            // Check if the region is free
            let mut is_free = true;
            for region in &self.regions {
                // Check for overlap
                if region.contains(aligned_base) || 
                   region.contains(aligned_base + size - 1) ||
                   (aligned_base <= region.base && aligned_base + size >= region.end()) {
                    is_free = false;
                    base = region.end();
                    break;
                }
            }
            
            if is_free {
                // Allocate the region
                let region = MemoryRegion::new(aligned_base, size);
                self.regions.push(region);
                return Some(aligned_base);
            }
        }
        
        None
    }
    
    /// Free a region at the given base address
    pub fn free(&mut self, base: u32) {
        self.regions.retain(|region| region.base != base);
    }
    
    /// Check if an address is allocated
    pub fn is_allocated(&self, addr: u32) -> bool {
        self.regions.iter().any(|region| region.contains(addr))
    }
    
    /// Get the region containing an address
    pub fn get_region(&self, addr: u32) -> Option<MemoryRegion> {
        self.regions.iter()
            .find(|region| region.contains(addr))
            .copied()
    }
}

/// WASM imports configuration
#[derive(Debug, Clone, Default)]
pub struct ImportConfig {
    /// Host functions to import
    pub host_functions: Vec<HostFunction>,
    
    /// Memory configuration
    pub memory: Option<MemoryConfig>,
}

/// Memory configuration for imports
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Minimum number of pages
    pub min_pages: u32,
    
    /// Maximum number of pages
    pub max_pages: Option<u32>,
    
    /// Whether memory is shared
    pub shared: bool,
}

/// Host function definition
#[derive(Debug, Clone)]
pub struct HostFunction {
    /// Module name
    pub module: String,
    
    /// Function name
    pub name: String,
    
    /// Parameter types
    pub params: Vec<WasmValueType>,
    
    /// Result types
    pub results: Vec<WasmValueType>,
}

/// WebAssembly value types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmValueType {
    /// 32-bit integer
    I32,
    
    /// 64-bit integer
    I64,
    
    /// 32-bit float
    F32,
    
    /// 64-bit float
    F64,
    
    /// 128-bit vector
    V128,
    
    /// Function reference
    FuncRef,
    
    /// External reference
    ExternRef,
}
