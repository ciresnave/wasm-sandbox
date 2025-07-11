//! Communication channels between host and guest

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::communication::CommunicationChannel;
use crate::utils::logging;

/// Message channel implementation
pub struct MessageChannel {
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

impl MessageChannel {
    /// Create a new message channel
    pub fn new(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            host_to_guest: Mutex::new(VecDeque::with_capacity(capacity)),
            guest_to_host: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            closed: Mutex::new(false),
        }
    }
    
    /// Get the guest-side interface
    pub fn get_guest_interface(&self) -> GuestChannelInterface {
        GuestChannelInterface {
            name: self.name.clone(),
            host_to_guest: self.host_to_guest.lock().unwrap().clone(),
            guest_to_host: self.guest_to_host.lock().unwrap().clone(),
            capacity: self.capacity,
            closed: *self.closed.lock().unwrap(),
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

impl CommunicationChannel for MessageChannel {
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
        if *self.closed.lock().unwrap() {
            return false;
        }
        
        let queue = self.guest_to_host.lock().unwrap();
        !queue.is_empty()
    }
    
    fn close(&self) -> Result<()> {
        let mut closed = self.closed.lock().unwrap();
        *closed = true;
        
        // Log the event
        logging::log_communication_event(&self.name, "closed", 0);
        
        Ok(())
    }
}

/// Guest-side channel interface
pub struct GuestChannelInterface {
    /// Channel name
    name: String,
    
    /// Messages from host to guest
    #[allow(dead_code)]
    host_to_guest: VecDeque<Vec<u8>>,
    
    /// Messages from guest to host
    #[allow(dead_code)]
    guest_to_host: VecDeque<Vec<u8>>,
    
    /// Channel capacity
    #[allow(dead_code)]
    capacity: usize,
    
    /// Is closed
    closed: bool,
}

impl GuestChannelInterface {
    /// Send a message to the host
    pub fn send(&self, _message: &[u8]) -> Result<()> {
        // Check if closed
        if self.closed {
            return Err(Error::Communication("Channel is closed".to_string()));
        }
        
        // In a real implementation, we would need to use thread-safe mechanisms
        // This is a simplified implementation
        
        Ok(())
    }
    
    /// Receive a message from the host
    pub fn receive(&self) -> Result<Vec<u8>> {
        // Check if closed
        if self.closed {
            return Err(Error::Communication("Channel is closed".to_string()));
        }
        
        // In a real implementation, we would need to use thread-safe mechanisms
        // This is a simplified implementation
        
        Err(Error::Communication("No messages available".to_string()))
    }
    
    /// Check if there are pending messages
    pub fn has_messages(&self) -> bool {
        if self.closed {
            return false;
        }
        
        // In a real implementation, we would need to use thread-safe mechanisms
        false
    }
    
    /// Close the channel
    pub fn close(&mut self) -> Result<()> {
        self.closed = true;
        Ok(())
    }
    
    /// Get the channel name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Communication channel factory
pub struct ChannelFactory {
    /// Default channel capacity
    default_capacity: usize,
}

impl ChannelFactory {
    /// Create a new channel factory
    pub fn new(default_capacity: usize) -> Self {
        Self {
            default_capacity,
        }
    }
    
    /// Create a message channel with default capacity
    pub fn create_channel(&self, name: &str) -> Arc<MessageChannel> {
        Arc::new(MessageChannel::new(name, self.default_capacity))
    }
    
    /// Create a message channel with custom capacity
    pub fn create_channel_with_capacity(&self, name: &str, capacity: usize) -> Arc<MessageChannel> {
        Arc::new(MessageChannel::new(name, capacity))
    }
}

impl Default for ChannelFactory {
    fn default() -> Self {
        Self {
            default_capacity: 100,
        }
    }
}
