//! Host-guest communication mechanisms

use std::sync::Arc;

use crate::error::Result;

/// Communication channel between host and guest
pub trait CommunicationChannel: Send + Sync {
    /// Send a message to the guest
    fn send_to_guest(&self, message: &[u8]) -> Result<()>;
    
    /// Receive a message from the guest
    fn receive_from_guest(&self) -> Result<Vec<u8>>;
    
    /// Check if there are pending messages
    fn has_messages(&self) -> bool;
    
    /// Close the communication channel
    fn close(&self) -> Result<()>;
}

/// RPC mechanism between host and guest (dyn-compatible part)
pub trait RpcChannel: Send + Sync {
    /// Register a host function with JSON serialization
    fn register_host_function_json(
        &mut self,
        name: &str,
        function: Box<dyn Fn(&str) -> Result<String> + Send + Sync + 'static>,
    ) -> Result<()>;
    
    /// Call a function in the guest with JSON serialization
    fn call_guest_function_json(
        &self,
        function_name: &str,
        params_json: &str,
    ) -> Result<String>;
    
    /// Register a host function with MessagePack serialization
    fn register_host_function_msgpack(
        &mut self,
        name: &str,
        function: Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Send + Sync + 'static>,
    ) -> Result<()>;
    
    /// Call a function in the guest with MessagePack serialization
    fn call_guest_function_msgpack(
        &self,
        function_name: &str,
        params_msgpack: &[u8],
    ) -> Result<Vec<u8>>;
}

/// Extension trait for type-safe RPC operations (generic, not dyn-compatible)
pub trait RpcChannelExt {
    /// Register a host function that can be called from the guest
    fn register_host_function<F, Params, Return>(
        &mut self,
        name: &str,
        function: F,
    ) -> Result<()>
    where
        F: Fn(Params) -> Result<Return> + Send + Sync + 'static,
        Params: serde::de::DeserializeOwned + 'static,
        Return: serde::Serialize + 'static;
    
    /// Call a function in the guest
    fn call_guest_function<Params, Return>(
        &self,
        function_name: &str,
        params: &Params,
    ) -> Result<Return>
    where
        Params: serde::Serialize + ?Sized,
        Return: serde::de::DeserializeOwned + 'static;
}

/// Automatic implementation for all RPC channels
impl<T: RpcChannel> RpcChannelExt for T {
    fn register_host_function<F, Params, Return>(
        &mut self,
        name: &str,
        function: F,
    ) -> Result<()>
    where
        F: Fn(Params) -> Result<Return> + Send + Sync + 'static,
        Params: serde::de::DeserializeOwned + 'static,
        Return: serde::Serialize + 'static,
    {
        let wrapped_function = Box::new(move |params_json: &str| -> Result<String> {
            let params: Params = serde_json::from_str(params_json)?;
            let result = function(params)?;
            let result_json = serde_json::to_string(&result)?;
            Ok(result_json)
        });
        
        self.register_host_function_json(name, wrapped_function)
    }
    
    fn call_guest_function<Params, Return>(
        &self,
        function_name: &str,
        params: &Params,
    ) -> Result<Return>
    where
        Params: serde::Serialize + ?Sized,
        Return: serde::de::DeserializeOwned + 'static,
    {
        let params_json = serde_json::to_string(params)?;
        let result_json = self.call_guest_function_json(function_name, &params_json)?;
        let result = serde_json::from_str(&result_json)?;
        Ok(result)
    }
}

/// Communication channel factory
pub trait CommunicationFactory: Send + Sync {
    /// Create a new communication channel
    fn create_channel(&self) -> Result<Arc<dyn CommunicationChannel>>;
    
    /// Create a new RPC channel
    fn create_rpc_channel(&self) -> Result<Arc<dyn RpcChannel>>;
}

pub mod channels;
pub mod io;
pub mod rpc;
pub mod memory;
pub mod memory_channel;

// Re-export memory channel for easier usage
pub use memory_channel::{MemoryChannel, MemoryRpcChannel, MemoryChannelConfig};
