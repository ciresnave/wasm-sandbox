//! MCP Server template

use std::collections::HashMap;
use crate::error::Result;
use super::TemplateRenderer;

/// MCP Server template renderer
pub struct McpServerTemplate {
    /// Default template
    template: String,
}

impl McpServerTemplate {
    /// Create a new MCP server template renderer
    pub fn new() -> Self {
        Self {
            template: MCP_SERVER_TEMPLATE.to_string(),
        }
    }
}

impl TemplateRenderer for McpServerTemplate {
    fn render(&self, template_name: &str, variables: &HashMap<String, String>) -> Result<String> {
        if template_name != "mcp_server" {
            return Err(crate::error::Error::Template { 
                message: format!("Unknown template: {}", template_name) 
            });
        }
        
        let mut result = self.template.clone();
        
        // Replace template variables
        for (key, value) in variables {
            let placeholder = format!("{{{{ {} }}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        Ok(result)
    }
    
    fn register_template(&mut self, _name: &str, _template: &str) -> Result<()> {
        // For now, don't allow registration of new templates
        Err(crate::error::Error::Template {
            message: "Template registration not supported".to_string()
        })
    }
    
    fn has_template(&self, name: &str) -> bool {
        name == "mcp_server"
    }
    
    fn get_template(&self, name: &str) -> Option<&str> {
        if name == "mcp_server" {
            Some(&self.template)
        } else {
            None
        }
    }
}

/// Template for MCP server wrapper
const MCP_SERVER_TEMPLATE: &str = r#"// Generated MCP Server WASM wrapper
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use anyhow::{Result, anyhow, Context};
use once_cell::sync::Lazy;

// Configuration
const APP_PATH: &str = "{{ app_path }}";

// Arguments
const APP_ARGS: &[&str] = &[
    {{ app_args }}
];

// Global state
static MCP_SERVER: Lazy<Mutex<Option<McpServerState>>> = Lazy::new(|| Mutex::new(None));
static RUNNING: AtomicBool = AtomicBool::new(false);

// MCP server state
struct McpServerState {
    capabilities: Capabilities,
    tools: Vec<Tool>,
    prompts: Vec<Prompt>,
}

// MCP protocol structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Capabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    logging: Option<LoggingCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompts: Option<PromptsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resources: Option<ResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<ToolsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoggingCapability {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PromptsCapability {
    #[serde(rename = "listChanged")]
    list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResourcesCapability {
    subscribe: bool,
    #[serde(rename = "listChanged")]
    list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolsCapability {
    #[serde(rename = "listChanged")]
    list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Prompt {
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<Vec<PromptArgument>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PromptArgument {
    name: String,
    description: String,
    required: bool,
}

// Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// Exported functions
#[no_mangle]
pub extern "C" fn mcp_server_init() -> i32 {
    let mut server = MCP_SERVER.lock().unwrap();
    
    *server = Some(McpServerState {
        capabilities: Capabilities {
            logging: Some(LoggingCapability {}),
            prompts: Some(PromptsCapability { list_changed: true }),
            resources: Some(ResourcesCapability { subscribe: true, list_changed: true }),
            tools: Some(ToolsCapability { list_changed: true }),
        },
        tools: vec![],
        prompts: vec![],
    });
    
    RUNNING.store(true, Ordering::Relaxed);
    0
}

#[no_mangle]
pub extern "C" fn mcp_server_handle_request(
    request_ptr: *const u8,
    request_len: usize,
    response_ptr: *mut u8,
    response_len: *mut usize,
) -> i32 {
    if !RUNNING.load(Ordering::Relaxed) {
        return -1;
    }
    
    // Safety: The caller guarantees the pointer is valid
    let request_data = unsafe {
        std::slice::from_raw_parts(request_ptr, request_len)
    };
    
    match handle_mcp_request(request_data) {
        Ok(response) => {
            let response_bytes = response.as_bytes();
            let copy_len = std::cmp::min(response_bytes.len(), unsafe { *response_len });
            
            unsafe {
                std::ptr::copy_nonoverlapping(
                    response_bytes.as_ptr(),
                    response_ptr,
                    copy_len
                );
                *response_len = copy_len;
            }
            
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn mcp_server_shutdown() -> i32 {
    RUNNING.store(false, Ordering::Relaxed);
    
    let mut server = MCP_SERVER.lock().unwrap();
    *server = None;
    
    0
}

// Internal request handler
fn handle_mcp_request(request_data: &[u8]) -> Result<String> {
    let request_str = std::str::from_utf8(request_data)?;
    let request: JsonRpcRequest = serde_json::from_str(request_str)?;
    
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(request.id, request.params),
        "tools/list" => handle_tools_list(request.id),
        "tools/call" => handle_tools_call(request.id, request.params),
        "prompts/list" => handle_prompts_list(request.id),
        "prompts/get" => handle_prompts_get(request.id, request.params),
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        },
    };
    
    let response_str = serde_json::to_string(&response)?;
    Ok(response_str)
}

fn handle_initialize(id: Value, _params: Option<Value>) -> JsonRpcResponse {
    let server = MCP_SERVER.lock().unwrap();
    
    if let Some(ref state) = *server {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": state.capabilities,
                "serverInfo": {
                    "name": "{{ server_name }}",
                    "version": "{{ server_version }}"
                }
            })),
            error: None,
        }
    } else {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: "Server not initialized".to_string(),
                data: None,
            }),
        }
    }
}

fn handle_tools_list(id: Value) -> JsonRpcResponse {
    let server = MCP_SERVER.lock().unwrap();
    
    if let Some(ref state) = *server {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": state.tools
            })),
            error: None,
        }
    } else {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: "Server not initialized".to_string(),
                data: None,
            }),
        }
    }
}

fn handle_tools_call(id: Value, params: Option<Value>) -> JsonRpcResponse {
    // Placeholder implementation
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(json!({
            "content": [
                {
                    "type": "text",
                    "text": "Tool call not implemented"
                }
            ]
        })),
        error: None,
    }
}

fn handle_prompts_list(id: Value) -> JsonRpcResponse {
    let server = MCP_SERVER.lock().unwrap();
    
    if let Some(ref state) = *server {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "prompts": state.prompts
            })),
            error: None,
        }
    } else {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: "Server not initialized".to_string(),
                data: None,
            }),
        }
    }
}

fn handle_prompts_get(id: Value, params: Option<Value>) -> JsonRpcResponse {
    // Placeholder implementation
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(json!({
            "description": "Prompt not implemented",
            "messages": []
        })),
        error: None,
    }
}
"#;
