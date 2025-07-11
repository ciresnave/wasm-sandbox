//! RPC mechanisms between host and guest

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize, de::DeserializeOwned};

use crate::error::{Error, Result};
use crate::communication::{RpcChannel, CommunicationChannel};

/// JSON-RPC implementation
pub struct JsonRpcChannel {
    /// Communication channel
    #[allow(dead_code)]
    channel: Arc<dyn CommunicationChannel>,
    
    /// Host functions
    host_functions: Mutex<HashMap<String, Box<dyn Fn(&str) -> Result<String> + Send + Sync>>>,
    
    /// Function call ID counter
    #[allow(dead_code)]
    call_id: Mutex<u64>,
}

impl JsonRpcChannel {
    /// Create a new JSON-RPC channel
    pub fn new(channel: Arc<dyn CommunicationChannel>) -> Self {
        Self {
            channel,
            host_functions: Mutex::new(HashMap::new()),
            call_id: Mutex::new(0),
        }
    }
    
    /// Get the next call ID
    #[allow(dead_code)]
    fn next_call_id(&self) -> u64 {
        let mut id = self.call_id.lock().unwrap();
        *id += 1;
        *id
    }
    
    /// Serialize a value to JSON
    #[allow(dead_code)]
    fn serialize<T: Serialize>(&self, value: &T) -> Result<String> {
        serde_json::to_string(value)
            .map_err(|e| Error::Json(e))
    }
    
    /// Deserialize a value from JSON
    #[allow(dead_code)]
    fn deserialize<T: DeserializeOwned>(&self, data: &str) -> Result<T> {
        serde_json::from_str(data)
            .map_err(|e| Error::Json(e))
    }
}

/// RPC request
#[derive(Debug, Serialize, Deserialize)]
struct RpcRequest {
    /// Request ID
    id: u64,
    
    /// Function name
    method: String,
    
    /// Function parameters (JSON string)
    params: String,
}

/// RPC response
#[derive(Debug, Serialize, Deserialize)]
struct RpcResponse {
    /// Request ID
    id: u64,
    
    /// Response result (JSON string)
    result: Option<String>,
    
    /// Error message
    error: Option<String>,
}

impl RpcChannel for JsonRpcChannel {
    fn register_host_function_json(
        &mut self,
        name: &str,
        function: Box<dyn Fn(&str) -> Result<String> + Send + Sync + 'static>,
    ) -> Result<()> {
        let mut functions = self.host_functions.lock().unwrap();
        functions.insert(name.to_string(), function);
        Ok(())
    }
    
    fn call_guest_function_json(
        &self,
        function_name: &str,
        params_json: &str,
    ) -> Result<String> {
        // Create JSON-RPC request
        let _request = RpcRequest {
            id: 1,
            method: function_name.to_string(),
            params: serde_json::from_str(params_json).map_err(|e| Error::Json(e))?,
        };
        
        // For now, return a mock response
        // In a real implementation, this would send the request through the communication channel
        let response = RpcResponse {
            id: 1,
            result: Some("mock_result".to_string()),
            error: None,
        };
        
        serde_json::to_string(&response).map_err(|e| Error::Json(e))
    }
    
    fn register_host_function_msgpack(
        &mut self,
        name: &str,
        function: Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Send + Sync + 'static>,
    ) -> Result<()> {
        // Convert to JSON function for now
        let json_function = move |params_json: &str| -> Result<String> {
            let params_bytes = params_json.as_bytes();
            let result_bytes = function(params_bytes)?;
            Ok(String::from_utf8_lossy(&result_bytes).to_string())
        };
        
        self.register_host_function_json(name, Box::new(json_function))
    }
    
    fn call_guest_function_msgpack(
        &self,
        function_name: &str,
        params_msgpack: &[u8],
    ) -> Result<Vec<u8>> {
        let params_json = String::from_utf8_lossy(params_msgpack);
        let result_json = self.call_guest_function_json(function_name, &params_json)?;
        Ok(result_json.into_bytes())
    }
}

/// RPC channel factory
pub struct RpcFactory {
    /// Communication channel factory
    channel_factory: Arc<dyn crate::communication::CommunicationFactory>,
}

impl RpcFactory {
    /// Create a new RPC factory
    pub fn new(channel_factory: Arc<dyn crate::communication::CommunicationFactory>) -> Self {
        Self {
            channel_factory,
        }
    }
    
    /// Create a new RPC channel
    pub fn create_channel(&self, _name: &str) -> Result<Arc<dyn RpcChannel>> {
        // Create the communication channel
        let channel = self.channel_factory.create_channel()?;
        
        // Create the RPC channel
        let rpc_channel = JsonRpcChannel::new(channel);
        
        Ok(Arc::new(rpc_channel))
    }
}
