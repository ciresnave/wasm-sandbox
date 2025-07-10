//! Memory-based communication channel

use std::sync::{Arc, Mutex};
use std::collections::{VecDeque, HashMap};

use serde::{Serialize, de::DeserializeOwned};

use crate::error::{Error, Result};
use crate::communication::{CommunicationChannel, RpcChannel};
use crate::utils::logging;

/// Memory channel for communication between host and guest
pub struct MemoryChannel {
    /// Channel name
    name: String,
    
    /// Messages from host to guest
    host_to_guest: Mutex<VecDeque<Vec<u8>>>,
    
    /// Messages from guest to host
    guest_to_host: Mutex<VecDeque<Vec<u8>>>,
    
    /// Channel capacity
    capacity: usize,
    
    /// Is closed
    closed: Mutex<bool>,
}

impl MemoryChannel {
    /// Create a new memory channel
    pub fn new(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            host_to_guest: Mutex::new(VecDeque::with_capacity(capacity)),
            guest_to_host: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            closed: Mutex::new(false),
        }
    }
    
    /// Get the channel name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the channel capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl CommunicationChannel for MemoryChannel {
    fn send_to_guest(&self, message: &[u8]) -> Result<()> {
        // Check if closed
        if *self.closed.lock().unwrap() {
            return Err(Error::Communication("Channel is closed".to_string()));
        }
        
        // Send the message
        let mut queue = self.host_to_guest.lock().unwrap();
        if queue.len() >= self.capacity {
            return Err(Error::Communication("Channel is full".to_string()));
        }
        
        queue.push_back(message.to_vec());
        
        // Log the event
        logging::log_communication_event(&self.name, "sent", message.len());
        
        Ok(())
    }
    
    fn receive_from_guest(&self) -> Result<Vec<u8>> {
        // Check if closed
        if *self.closed.lock().unwrap() {
            return Err(Error::Communication("Channel is closed".to_string()));
        }
        
        // Receive the message
        let mut queue = self.guest_to_host.lock().unwrap();
        
        if let Some(message) = queue.pop_front() {
            // Log the event
            logging::log_communication_event(&self.name, "received", message.len());
            
            Ok(message)
        } else {
            Err(Error::Communication("No messages available".to_string()))
        }
    }
    
    fn has_messages(&self) -> bool {
        // Check if closed
        if *self.closed.lock().unwrap() {
            return false;
        }
        
        // Check if there are messages
        !self.guest_to_host.lock().unwrap().is_empty()
    }
    
    fn close(&self) -> Result<()> {
        // Set closed flag
        *self.closed.lock().unwrap() = true;
        
        Ok(())
    }
}

/// RPC channel for function calls between host and guest
pub struct RpcChannel {
    /// Communication channel
    channel: Arc<dyn CommunicationChannel>,
    
    /// Host functions
    host_functions: Mutex<HashMap<String, Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Send + Sync>>>,
}

impl RpcChannel {
    /// Create a new RPC channel
    pub fn new(channel: Arc<dyn CommunicationChannel>) -> Self {
        Self {
            channel,
            host_functions: Mutex::new(HashMap::new()),
        }
    }
}

impl RpcChannel for RpcChannel {
    fn register_host_function<F, Params, Return>(
        &mut self,
        name: &str,
        function: F,
    ) -> Result<()>
    where
        F: Fn(Params) -> Result<Return> + Send + Sync + 'static,
        Params: DeserializeOwned + 'static,
        Return: Serialize + 'static,
    {
        // Create a wrapper function
        let wrapper = move |data: &[u8]| -> Result<Vec<u8>> {
            // Deserialize parameters
            let params: Params = serde_json::from_slice(data)
                .map_err(|e| Error::Communication(format!("Failed to deserialize parameters: {}", e)))?;
            
            // Call the function
            let result = function(params)?;
            
            // Serialize the result
            let result_data = serde_json::to_vec(&result)
                .map_err(|e| Error::Communication(format!("Failed to serialize result: {}", e)))?;
            
            Ok(result_data)
        };
        
        // Register the function
        let mut host_functions = self.host_functions.lock().unwrap();
        host_functions.insert(name.to_string(), Box::new(wrapper));
        
        Ok(())
    }
    
    fn call_guest_function<Params, Return>(
        &self,
        function_name: &str,
        params: &Params,
    ) -> Result<Return>
    where
        Params: Serialize + ?Sized,
        Return: DeserializeOwned + 'static,
    {
        // Prepare the function call message
        let mut message = Vec::new();
        
        // Add function name length (u16)
        let name_len = function_name.len() as u16;
        message.extend_from_slice(&name_len.to_le_bytes());
        
        // Add function name
        message.extend_from_slice(function_name.as_bytes());
        
        // Serialize parameters
        let params_data = serde_json::to_vec(params)
            .map_err(|e| Error::Communication(format!("Failed to serialize parameters: {}", e)))?;
        
        // Add parameters length (u32)
        let params_len = params_data.len() as u32;
        message.extend_from_slice(&params_len.to_le_bytes());
        
        // Add parameters
        message.extend_from_slice(&params_data);
        
        // Send the message
        self.channel.send_to_guest(&message)?;
        
        // Wait for response
        let response_data = self.channel.receive_from_guest()?;
        
        // Deserialize response
        let response: Return = serde_json::from_slice(&response_data)
            .map_err(|e| Error::Communication(format!("Failed to deserialize response: {}", e)))?;
        
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_channel() {
        // Create a channel
        let channel = MemoryChannel::new("test", 10);
        
        // Send a message
        let message = b"Hello, world!";
        assert!(channel.send_to_guest(message).is_ok());
        
        // Check if there are messages
        assert!(!channel.has_messages());
        
        // Push a message to simulate guest sending
        {
            let mut queue = channel.guest_to_host.lock().unwrap();
            queue.push_back(b"Response".to_vec());
        }
        
        // Check if there are messages
        assert!(channel.has_messages());
        
        // Receive the message
        let response = channel.receive_from_guest().unwrap();
        assert_eq!(response, b"Response");
        
        // Close the channel
        assert!(channel.close().is_ok());
        
        // Try to send after closing
        assert!(channel.send_to_guest(message).is_err());
    }
    
    #[test]
    fn test_rpc_channel() {
        // Create a memory channel
        let memory_channel = Arc::new(MemoryChannel::new("rpc", 10));
        
        // Create an RPC channel
        let mut rpc_channel = RpcChannel::new(memory_channel.clone());
        
        // Register a function
        let result = rpc_channel.register_host_function("echo", |s: String| -> Result<String> {
            Ok(s)
        });
        assert!(result.is_ok());
        
        // Simulate a function call response
        {
            let mut queue = memory_channel.guest_to_host.lock().unwrap();
            queue.push_back(br#""Hello, world!""#.to_vec());
        }
        
        // Call a function
        let result: String = rpc_channel.call_guest_function("echo", &"Hello").unwrap();
        assert_eq!(result, "Hello, world!");
    }
}
