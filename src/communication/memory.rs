//! Shared memory abstractions for host-guest communication

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::error::{Error, Result};

/// Shared memory region
pub struct SharedMemoryRegion {
    /// Region name
    name: String,
    
    /// Memory buffer
    buffer: Mutex<Vec<u8>>,
    
    /// Region size
    size: usize,
}

impl SharedMemoryRegion {
    /// Create a new shared memory region
    pub fn new(name: &str, size: usize) -> Self {
        Self {
            name: name.to_string(),
            buffer: Mutex::new(vec![0; size]),
            size,
        }
    }
    
    /// Read from the shared memory region
    pub fn read(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        // Get the buffer
        let buffer = self.buffer.lock().unwrap();
        
        // Check if the offset is valid
        if offset >= self.size {
            return Err(Error::Communication(format!("Invalid offset: {}", offset)));
        }
        
        // Calculate the number of bytes to read
        let n = std::cmp::min(buf.len(), self.size - offset);
        
        // Copy the data
        buf[..n].copy_from_slice(&buffer[offset..offset+n]);
        
        Ok(n)
    }
    
    /// Write to the shared memory region
    pub fn write(&self, offset: usize, data: &[u8]) -> Result<usize> {
        // Get the buffer
        let mut buffer = self.buffer.lock().unwrap();
        
        // Check if the offset is valid
        if offset >= self.size {
            return Err(Error::Communication(format!("Invalid offset: {}", offset)));
        }
        
        // Calculate the number of bytes to write
        let n = std::cmp::min(data.len(), self.size - offset);
        
        // Copy the data
        buffer[offset..offset+n].copy_from_slice(&data[..n]);
        
        Ok(n)
    }
    
    /// Get the region size
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Get the region name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Shared memory manager
pub struct SharedMemoryManager {
    /// Shared memory regions
    regions: Mutex<HashMap<String, Arc<SharedMemoryRegion>>>,
}

impl SharedMemoryManager {
    /// Create a new shared memory manager
    pub fn new() -> Self {
        Self {
            regions: Mutex::new(HashMap::new()),
        }
    }
    
    /// Create a new shared memory region
    pub fn create_region(&self, name: &str, size: usize) -> Result<Arc<SharedMemoryRegion>> {
        // Check if the region already exists
        let mut regions = self.regions.lock().unwrap();
        if regions.contains_key(name) {
            return Err(Error::Communication(format!("Region already exists: {}", name)));
        }
        
        // Create the region
        let region = Arc::new(SharedMemoryRegion::new(name, size));
        
        // Register the region
        regions.insert(name.to_string(), region.clone());
        
        Ok(region)
    }
    
    /// Get a shared memory region
    pub fn get_region(&self, name: &str) -> Option<Arc<SharedMemoryRegion>> {
        let regions = self.regions.lock().unwrap();
        regions.get(name).cloned()
    }
    
    /// Delete a shared memory region
    pub fn delete_region(&self, name: &str) -> Result<()> {
        let mut regions = self.regions.lock().unwrap();
        if regions.remove(name).is_none() {
            return Err(Error::Communication(format!("Region not found: {}", name)));
        }
        
        Ok(())
    }
    
    /// List all shared memory regions
    pub fn list_regions(&self) -> Vec<String> {
        let regions = self.regions.lock().unwrap();
        regions.keys().cloned().collect()
    }
}
