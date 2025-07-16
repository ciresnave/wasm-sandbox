//! Memory-based communication channel between host and guest

use std::sync::{Arc, Mutex};
use std::collections::{VecDeque, HashMap};

use crate::error::{Error, Result};
use crate::communication::{CommunicationChannel, RpcChannel, StringHandlerFunction, ByteHandlerFunction};
use crate::communication::memory::SharedMemoryRegion;
use crate::utils::logging;

/// Memory channel configuration
pub struct MemoryChannelConfig {
    /// Channel name
    pub name: String,
    
    /// Data region size
    pub data_size: usize,
    
    /// Control region size
    pub control_size: usize,
    
    /// Message queue capacity
    pub queue_capacity: usize,
}

impl Default for MemoryChannelConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            data_size: 64 * 1024, // 64KB data region
            control_size: 1024,   // 1KB control region
            queue_capacity: 32,   // 32 messages in queue
        }
    }
}

/// Channel control flags
#[repr(u8)]
#[derive(PartialEq)]
pub enum ControlFlag {
    /// Ready for reading
    Ready = 1,
    
    /// Reading in progress
    Reading = 2,
    
    /// Ready for writing
    WritingReady = 3,
    
    /// Writing in progress
    Writing = 4,
    
    /// Channel closed
    Closed = 5,
}

/// Memory-based communication channel
pub struct MemoryChannel {
    /// Channel name
    name: String,
    
    /// Data region for message data
    data_region: Arc<SharedMemoryRegion>,
    
    /// Control region for synchronization
    control_region: Arc<SharedMemoryRegion>,
    
    /// Message queue for bookkeeping
    message_queue: Mutex<VecDeque<(usize, usize)>>, // (offset, length)
    
    /// Is channel closed
    closed: Mutex<bool>,
    
    /// Queue capacity
    capacity: usize,
}

impl MemoryChannel {
    /// Create a new memory channel with shared memory regions
    pub fn new(
        config: &MemoryChannelConfig,
        data_region: Arc<SharedMemoryRegion>,
        control_region: Arc<SharedMemoryRegion>,
    ) -> Self {
        Self {
            name: config.name.clone(),
            data_region,
            control_region,
            message_queue: Mutex::new(VecDeque::with_capacity(config.queue_capacity)),
            closed: Mutex::new(false),
            capacity: config.queue_capacity,
        }
    }
    
    /// Get the channel name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Set control flag
    fn set_control_flag(&self, flag: ControlFlag) -> Result<()> {
        let mut buffer = [0u8; 1];
        buffer[0] = flag as u8;
        self.control_region.write(0, &buffer)?;
        Ok(())
    }
    
    /// Get control flag
    fn get_control_flag(&self) -> Result<ControlFlag> {
        let mut buffer = [0u8; 1];
        self.control_region.read(0, &mut buffer)?;
        match buffer[0] {
            1 => Ok(ControlFlag::Ready),
            2 => Ok(ControlFlag::Reading),
            3 => Ok(ControlFlag::WritingReady),
            4 => Ok(ControlFlag::Writing),
            5 => Ok(ControlFlag::Closed),
            _ => Err(Error::Communication {
                channel: "memory_channel".to_string(),
                reason: "Invalid control flag".to_string(),
                instance_id: None,
            }),
        }
    }
    
    /// Write message length to control region
    fn write_message_length(&self, length: usize) -> Result<()> {
        let length_bytes = (length as u32).to_le_bytes();
        self.control_region.write(1, &length_bytes)?;
        Ok(())
    }
    
    /// Read message length from control region
    fn read_message_length(&self) -> Result<usize> {
        let mut length_bytes = [0u8; 4];
        self.control_region.read(1, &mut length_bytes)?;
        Ok(u32::from_le_bytes(length_bytes) as usize)
    }
}

impl CommunicationChannel for MemoryChannel {
    fn send_to_guest(&self, message: &[u8]) -> Result<()> {
        // Check if channel is closed
        if *self.closed.lock().unwrap() {
            return Err(Error::Communication {
                channel: "memory_channel".to_string(),
                reason: "Channel is closed".to_string(),
                instance_id: None,
            });
        }
        
        // Check message queue capacity
        let mut queue = self.message_queue.lock().unwrap();
        if queue.len() >= self.capacity {
            return Err(Error::Communication {
                channel: "memory_channel".to_string(),
                reason: "Channel is full".to_string(),
                instance_id: None,
            });
        }
        
        // Wait for control region to be ready for writing
        let mut retries = 0;
        while self.get_control_flag()? != ControlFlag::WritingReady && retries < 100 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            retries += 1;
        }
        
        if retries >= 100 {
            return Err(Error::Communication {
                channel: "memory_channel".to_string(),
                reason: "Timeout waiting for channel to be ready".to_string(),
                instance_id: None,
            });
        }
        
        // Set control flag to writing
        self.set_control_flag(ControlFlag::Writing)?;
        
        // Write message length
        self.write_message_length(message.len())?;
        
        // Find next free offset in data region
        let offset = if let Some((last_offset, last_len)) = queue.back() {
            last_offset + last_len
        } else {
            0
        };
        
        // Write message to data region
        self.data_region.write(offset, message)?;
        
        // Add message to queue
        queue.push_back((offset, message.len()));
        
        // Set control flag to ready
        self.set_control_flag(ControlFlag::Ready)?;
        
        // Log the event
        logging::log_communication_event(&self.name, "sent", message.len());
        
        Ok(())
    }
    
    fn receive_from_guest(&self) -> Result<Vec<u8>> {
        // Check if channel is closed
        if *self.closed.lock().unwrap() {
            return Err(Error::Communication {
                channel: "memory_channel".to_string(),
                reason: "Channel is closed".to_string(),
                instance_id: None,
            });
        }
        
        // Wait for control region to be ready for reading
        let mut retries = 0;
        while self.get_control_flag()? != ControlFlag::Ready && retries < 100 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            retries += 1;
        }
        
        if retries >= 100 {
            return Err(Error::Communication { 
                channel: "memory_channel".to_string(), 
                reason: "Timeout waiting for message".to_string(),
                instance_id: None 
            });
        }
        
        // Set control flag to reading
        self.set_control_flag(ControlFlag::Reading)?;
        
        // Read message length
        let length = self.read_message_length()?;
        
        // Read message from data region
        let mut message = vec![0u8; length];
        self.data_region.read(0, &mut message)?;
        
        // Set control flag back to writing ready
        self.set_control_flag(ControlFlag::WritingReady)?;
        
        // Log the event
        logging::log_communication_event(&self.name, "received", message.len());
        
        Ok(message)
    }
    
    fn has_messages(&self) -> bool {
        // Check if channel is closed
        if *self.closed.lock().unwrap() {
            return false;
        }
        
        // Check if control region is ready for reading
        if let Ok(flag) = self.get_control_flag() {
            return flag == ControlFlag::Ready;
        }
        
        false
    }
    
    fn close(&self) -> Result<()> {
        // Set closed flag
        *self.closed.lock().unwrap() = true;
        
        // Set control flag to closed
        self.set_control_flag(ControlFlag::Closed)?;
        
        Ok(())
    }
}

/// Memory RPC channel
pub struct MemoryRpcChannel {
    /// Communication channel for RPC
    channel: Arc<MemoryChannel>,
    
    /// RPC function registry
    functions: Mutex<HashMap<String, ByteHandlerFunction>>,
}

impl MemoryRpcChannel {
    /// Create a new memory RPC channel
    pub fn new(channel: Arc<MemoryChannel>) -> Self {
        Self {
            channel,
            functions: Mutex::new(HashMap::new()),
        }
    }
}

impl RpcChannel for MemoryRpcChannel {
    fn register_host_function_json(
        &mut self,
        name: &str,
        function: StringHandlerFunction,
    ) -> Result<()> {
        let name = name.to_string();
        let func = Box::new(move |data: &[u8]| -> Result<Vec<u8>> {
            // Convert bytes to string
            let params_json = String::from_utf8_lossy(data);
            
            // Call the function
            let result_json = function(&params_json)?;
            
            // Convert result back to bytes
            Ok(result_json.into_bytes())
        });
        
        // Register the function
        self.functions.lock().unwrap().insert(name, func);
        
        Ok(())
    }
    
    fn call_guest_function_json(
        &self,
        function_name: &str,
        params_json: &str,
    ) -> Result<String> {
        // Create RPC message
        let mut message = Vec::with_capacity(function_name.len() + params_json.len() + 5);
        
        // Add function name length (u8)
        message.push(function_name.len() as u8);
        
        // Add function name
        message.extend_from_slice(function_name.as_bytes());
        
        // Add parameters as bytes
        message.extend_from_slice(params_json.as_bytes());
        
        // Send message to guest
        self.channel.send_to_guest(&message)?;
        
        // Receive response from guest
        let response_bytes = self.channel.receive_from_guest()?;
        
        // Convert response back to string
        let response = String::from_utf8_lossy(&response_bytes).to_string();
        
        Ok(response)
    }
    
    fn register_host_function_msgpack(
        &mut self,
        name: &str,
        function: ByteHandlerFunction,
    ) -> Result<()> {
        let name = name.to_string();
        
        // Register the function directly
        self.functions.lock().unwrap().insert(name, function);
        
        Ok(())
    }
    
    fn call_guest_function_msgpack(
        &self,
        function_name: &str,
        params_msgpack: &[u8],
    ) -> Result<Vec<u8>> {
        // Create RPC message
        let mut message = Vec::with_capacity(function_name.len() + params_msgpack.len() + 5);
        
        // Add function name length (u8)
        message.push(function_name.len() as u8);
        
        // Add function name
        message.extend_from_slice(function_name.as_bytes());
        
        // Add parameters
        message.extend_from_slice(params_msgpack);
        
        // Send message to guest
        self.channel.send_to_guest(&message)?;
        
        // Receive response from guest
        let response_bytes = self.channel.receive_from_guest()?;
        
        Ok(response_bytes)
    }
}
