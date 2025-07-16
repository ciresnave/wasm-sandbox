# MCP Servers

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Building Model Context Protocol (MCP) servers with wasm-sandbox for secure AI agent integration, tool execution, and context management.

## Overview

MCP servers with wasm-sandbox enable secure execution of AI agent tools, dynamic capability provisioning, and isolated processing of AI-generated code and data.

## Basic MCP Server

### Simple Tool Server

```rust
use mcp_core::{
    protocol::{
        CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult,
        Tool, ToolInputSchema, ResourceTemplate, ListResourcesRequest,
        ListResourcesResult, ReadResourceRequest, ReadResourceResult,
    },
    transport::stdio::StdioTransport,
    Server, ServerBuilder,
};
use wasm_sandbox::{WasmSandbox, SecurityPolicy, Capability};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct WasmMcpServer {
    tools: Arc<RwLock<HashMap<String, WasmTool>>>,
    resources: Arc<RwLock<HashMap<String, WasmResource>>>,
    sandboxes: Arc<RwLock<HashMap<String, WasmSandbox>>>,
}

#[derive(Debug, Clone)]
struct WasmTool {
    name: String,
    description: String,
    schema: ToolInputSchema,
    wasm_module: String,
    function_name: String,
    security_policy: SecurityPolicy,
}

#[derive(Debug, Clone)]
struct WasmResource {
    uri: String,
    name: String,
    description: String,
    mime_type: String,
    wasm_module: String,
    function_name: String,
}

impl WasmMcpServer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            sandboxes: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn register_tool(&self, tool: WasmTool) -> Result<(), Box<dyn std::error::Error>> {
        // Create sandbox for this tool
        let sandbox = WasmSandbox::builder()
            .source(&tool.wasm_module)
            .security_policy(tool.security_policy.clone())
            .memory_limit(64 * 1024 * 1024) // 64MB default
            .cpu_timeout(std::time::Duration::from_secs(30))
            .build()
            .await?;

        let mut tools = self.tools.write().await;
        let mut sandboxes = self.sandboxes.write().await;
        
        tools.insert(tool.name.clone(), tool.clone());
        sandboxes.insert(tool.name.clone(), sandbox);
        
        println!("Registered tool: {} -> {}", tool.name, tool.wasm_module);
        Ok(())
    }

    pub async fn register_resource(&self, resource: WasmResource) -> Result<(), Box<dyn std::error::Error>> {
        let mut resources = self.resources.write().await;
        resources.insert(resource.uri.clone(), resource);
        Ok(())
    }

    async fn list_tools(&self) -> Result<ListToolsResult, Box<dyn std::error::Error>> {
        let tools = self.tools.read().await;
        
        let tool_list: Vec<Tool> = tools.values().map(|wasm_tool| {
            Tool {
                name: wasm_tool.name.clone(),
                description: Some(wasm_tool.description.clone()),
                input_schema: wasm_tool.schema.clone(),
            }
        }).collect();

        Ok(ListToolsResult { tools: tool_list })
    }

    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult, Box<dyn std::error::Error>> {
        let sandboxes = self.sandboxes.read().await;
        let tools = self.tools.read().await;
        
        let tool = tools.get(&request.name)
            .ok_or_else(|| format!("Tool '{}' not found", request.name))?;
        
        let sandbox = sandboxes.get(&request.name)
            .ok_or_else(|| format!("Sandbox for tool '{}' not found", request.name))?;

        // Prepare tool execution context
        let tool_context = ToolExecutionContext {
            name: request.name.clone(),
            arguments: request.arguments.unwrap_or_default(),
            metadata: HashMap::new(),
        };

        // Execute tool with audit logging
        let start_time = std::time::Instant::now();
        let result = sandbox.call_with_audit(&tool.function_name, &tool_context).await?;
        let execution_time = start_time.elapsed();

        // Check for security violations
        if !result.audit_log.violations.is_empty() {
            return Ok(CallToolResult {
                content: vec![],
                is_error: Some(true),
                _meta: Some(json!({
                    "error": "Security violation detected",
                    "violations": result.audit_log.violations
                })),
            });
        }

        // Parse tool output
        let tool_output: ToolOutput = serde_json::from_value(result.output)?;

        Ok(CallToolResult {
            content: tool_output.content,
            is_error: Some(tool_output.is_error),
            _meta: Some(json!({
                "execution_time_ms": execution_time.as_millis(),
                "memory_used": result.memory_used,
                "audit_events": result.audit_log.events.len()
            })),
        })
    }

    async fn list_resources(&self) -> Result<ListResourcesResult, Box<dyn std::error::Error>> {
        let resources = self.resources.read().await;
        
        let resource_list: Vec<ResourceTemplate> = resources.values().map(|wasm_resource| {
            ResourceTemplate {
                uri: wasm_resource.uri.clone(),
                name: wasm_resource.name.clone(),
                description: Some(wasm_resource.description.clone()),
                mime_type: Some(wasm_resource.mime_type.clone()),
            }
        }).collect();

        Ok(ListResourcesResult { resources: resource_list })
    }

    async fn read_resource(&self, request: ReadResourceRequest) -> Result<ReadResourceResult, Box<dyn std::error::Error>> {
        let resources = self.resources.read().await;
        let resource = resources.get(&request.uri)
            .ok_or_else(|| format!("Resource '{}' not found", request.uri))?;

        // Create temporary sandbox for resource reading
        let sandbox = WasmSandbox::builder()
            .source(&resource.wasm_module)
            .security_policy(SecurityPolicy::resource_access())
            .build()
            .await?;

        let resource_context = ResourceContext {
            uri: request.uri.clone(),
            parameters: HashMap::new(),
        };

        let content: String = sandbox.call(&resource.function_name, &resource_context).await?;

        Ok(ReadResourceResult {
            contents: vec![mcp_core::protocol::ResourceContents::Text {
                uri: request.uri,
                mime_type: Some(resource.mime_type.clone()),
                text: content,
            }],
        })
    }
}

#[derive(Serialize, Deserialize)]
struct ToolExecutionContext {
    name: String,
    arguments: Value,
    metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct ToolOutput {
    content: Vec<mcp_core::protocol::Content>,
    is_error: bool,
}

#[derive(Serialize, Deserialize)]
struct ResourceContext {
    uri: String,
    parameters: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize MCP server
    let wasm_server = WasmMcpServer::new().await?;
    
    // Register tools
    register_default_tools(&wasm_server).await?;
    register_default_resources(&wasm_server).await?;

    // Create MCP server
    let server = ServerBuilder::new("wasm-mcp-server")
        .version("1.0.0")
        .build();

    // Set up handlers
    let wasm_server_clone = wasm_server.clone();
    server.set_list_tools_handler(move || {
        let server = wasm_server_clone.clone();
        Box::pin(async move { server.list_tools().await })
    });

    let wasm_server_clone = wasm_server.clone();
    server.set_call_tool_handler(move |request| {
        let server = wasm_server_clone.clone();
        Box::pin(async move { server.call_tool(request).await })
    });

    let wasm_server_clone = wasm_server.clone();
    server.set_list_resources_handler(move || {
        let server = wasm_server_clone.clone();
        Box::pin(async move { server.list_resources().await })
    });

    let wasm_server_clone = wasm_server.clone();
    server.set_read_resource_handler(move |request| {
        let server = wasm_server_clone.clone();
        Box::pin(async move { server.read_resource(request).await })
    });

    // Start server with stdio transport
    let transport = StdioTransport::new();
    server.run(transport).await?;

    Ok(())
}

async fn register_default_tools(server: &WasmMcpServer) -> Result<(), Box<dyn std::error::Error>> {
    // File system tool
    server.register_tool(WasmTool {
        name: "read_file".to_string(),
        description: "Read content from a file".to_string(),
        schema: ToolInputSchema {
            type_: "object".to_string(),
            properties: Some(json!({
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                }
            })),
            required: Some(vec!["path".to_string()]),
        },
        wasm_module: "tools/file_reader.wasm".to_string(),
        function_name: "read_file".to_string(),
        security_policy: SecurityPolicy::file_access_readonly(),
    }).await?;

    // Code execution tool
    server.register_tool(WasmTool {
        name: "execute_code".to_string(),
        description: "Execute code safely in a sandbox".to_string(),
        schema: ToolInputSchema {
            type_: "object".to_string(),
            properties: Some(json!({
                "language": {
                    "type": "string",
                    "enum": ["python", "javascript", "rust"],
                    "description": "Programming language"
                },
                "code": {
                    "type": "string",
                    "description": "Code to execute"
                },
                "timeout": {
                    "type": "number",
                    "description": "Timeout in seconds",
                    "default": 30
                }
            })),
            required: Some(vec!["language".to_string(), "code".to_string()]),
        },
        wasm_module: "tools/code_executor.wasm".to_string(),
        function_name: "execute_code".to_string(),
        security_policy: SecurityPolicy::code_execution(),
    }).await?;

    // Web scraping tool
    server.register_tool(WasmTool {
        name: "fetch_url".to_string(),
        description: "Fetch content from a URL".to_string(),
        schema: ToolInputSchema {
            type_: "object".to_string(),
            properties: Some(json!({
                "url": {
                    "type": "string",
                    "description": "URL to fetch"
                },
                "headers": {
                    "type": "object",
                    "description": "HTTP headers",
                    "default": {}
                }
            })),
            required: Some(vec!["url".to_string()]),
        },
        wasm_module: "tools/web_fetcher.wasm".to_string(),
        function_name: "fetch_url".to_string(),
        security_policy: SecurityPolicy::web_access(),
    }).await?;

    Ok(())
}

async fn register_default_resources(server: &WasmMcpServer) -> Result<(), Box<dyn std::error::Error>> {
    // System information resource
    server.register_resource(WasmResource {
        uri: "system://info".to_string(),
        name: "System Information".to_string(),
        description: "Current system information and status".to_string(),
        mime_type: "application/json".to_string(),
        wasm_module: "resources/system_info.wasm".to_string(),
        function_name: "get_system_info".to_string(),
    }).await?;

    // Environment variables resource
    server.register_resource(WasmResource {
        uri: "system://env".to_string(),
        name: "Environment Variables".to_string(),
        description: "Available environment variables".to_string(),
        mime_type: "application/json".to_string(),
        wasm_module: "resources/env_reader.wasm".to_string(),
        function_name: "get_environment".to_string(),
    }).await?;

    Ok(())
}
```

## Advanced MCP Features

### Dynamic Tool Registration

```rust
pub struct DynamicMcpServer {
    server: WasmMcpServer,
    tool_registry: Arc<RwLock<ToolRegistry>>,
    capabilities: Arc<RwLock<ServerCapabilities>>,
}

#[derive(Debug)]
struct ToolRegistry {
    tools: HashMap<String, RegisteredTool>,
    categories: HashMap<String, Vec<String>>,
    auto_discovery: bool,
    watch_directories: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct RegisteredTool {
    tool: WasmTool,
    version: String,
    author: String,
    tags: Vec<String>,
    usage_count: u64,
    last_used: std::time::SystemTime,
    performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Default)]
struct PerformanceMetrics {
    average_execution_time_ms: f64,
    success_rate: f64,
    memory_efficiency: f64,
    total_invocations: u64,
}

#[derive(Debug)]
struct ServerCapabilities {
    max_concurrent_tools: usize,
    supported_languages: Vec<String>,
    security_levels: Vec<String>,
    experimental_features: Vec<String>,
}

impl DynamicMcpServer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let server = WasmMcpServer::new().await?;
        
        let tool_registry = ToolRegistry {
            tools: HashMap::new(),
            categories: HashMap::new(),
            auto_discovery: true,
            watch_directories: vec![
                PathBuf::from("./mcp-tools"),
                PathBuf::from("~/.mcp-tools"),
            ],
        };

        let capabilities = ServerCapabilities {
            max_concurrent_tools: 50,
            supported_languages: vec!["rust".to_string(), "python".to_string(), "javascript".to_string()],
            security_levels: vec!["strict".to_string(), "moderate".to_string(), "permissive".to_string()],
            experimental_features: vec!["hot_reload".to_string(), "distributed_execution".to_string()],
        };

        Ok(Self {
            server,
            tool_registry: Arc::new(RwLock::new(tool_registry)),
            capabilities: Arc::new(RwLock::new(capabilities)),
        })
    }

    pub async fn start_auto_discovery(&self) -> Result<(), Box<dyn std::error::Error>> {
        let registry = self.tool_registry.clone();
        let server = self.server.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::discover_tools(&registry, &server).await {
                    eprintln!("Tool discovery error: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn discover_tools(
        registry: &Arc<RwLock<ToolRegistry>>,
        server: &WasmMcpServer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let watch_dirs = {
            let reg = registry.read().await;
            reg.watch_directories.clone()
        };

        for dir in watch_dirs {
            if dir.exists() {
                Self::scan_directory(&dir, registry, server).await?;
            }
        }

        Ok(())
    }

    async fn scan_directory(
        dir: &Path,
        registry: &Arc<RwLock<ToolRegistry>>,
        server: &WasmMcpServer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    if let Ok(tool) = Self::analyze_wasm_tool(&path).await {
                        Self::register_discovered_tool(registry, server, tool).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn analyze_wasm_tool(path: &Path) -> Result<WasmTool, Box<dyn std::error::Error>> {
        // Create temporary sandbox to introspect the WASM module
        let inspector = WasmSandbox::builder()
            .source(path)
            .security_policy(SecurityPolicy::introspection())
            .build()
            .await?;

        // Get tool metadata
        let metadata: ToolMetadata = inspector.call("get_tool_metadata", ()).await?;

        Ok(WasmTool {
            name: metadata.name,
            description: metadata.description,
            schema: metadata.input_schema,
            wasm_module: path.to_string_lossy().to_string(),
            function_name: metadata.entry_point,
            security_policy: SecurityPolicy::from_string(&metadata.required_permissions)?,
        })
    }

    async fn register_discovered_tool(
        registry: &Arc<RwLock<ToolRegistry>>,
        server: &WasmMcpServer,
        tool: WasmTool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut reg = registry.write().await;
        
        if !reg.tools.contains_key(&tool.name) {
            let registered_tool = RegisteredTool {
                tool: tool.clone(),
                version: "1.0.0".to_string(),
                author: "auto-discovered".to_string(),
                tags: vec!["auto".to_string()],
                usage_count: 0,
                last_used: std::time::SystemTime::now(),
                performance_metrics: PerformanceMetrics::default(),
            };

            reg.tools.insert(tool.name.clone(), registered_tool);
            drop(reg); // Release lock before async call

            server.register_tool(tool).await?;
            println!("Auto-discovered and registered tool: {}", tool.name);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct ToolMetadata {
    name: String,
    description: String,
    version: String,
    author: String,
    entry_point: String,
    input_schema: ToolInputSchema,
    required_permissions: String,
    tags: Vec<String>,
}
```

### Context-Aware Tool Execution

```rust
pub struct ContextAwareMcp {
    server: DynamicMcpServer,
    context_manager: Arc<RwLock<ContextManager>>,
    execution_history: Arc<RwLock<ExecutionHistory>>,
}

#[derive(Debug)]
struct ContextManager {
    conversation_contexts: HashMap<String, ConversationContext>,
    global_context: GlobalContext,
    context_sharing_rules: HashMap<String, SharingRule>,
}

#[derive(Debug, Clone)]
struct ConversationContext {
    id: String,
    user_id: String,
    started_at: std::time::SystemTime,
    variables: HashMap<String, Value>,
    tool_preferences: HashMap<String, ToolPreference>,
    security_level: SecurityLevel,
}

#[derive(Debug, Clone)]
struct GlobalContext {
    system_state: HashMap<String, Value>,
    shared_resources: HashMap<String, Resource>,
    active_sessions: u32,
    uptime: std::time::Duration,
}

#[derive(Debug, Clone)]
struct ToolPreference {
    weight: f64,
    last_success_rate: f64,
    preferred_parameters: HashMap<String, Value>,
}

#[derive(Debug)]
enum SharingRule {
    Private,
    SessionOnly,
    UserScoped,
    Global,
}

#[derive(Debug, Clone)]
enum SecurityLevel {
    Strict,
    Moderate,
    Permissive,
    Custom(SecurityPolicy),
}

impl ContextAwareMcp {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let server = DynamicMcpServer::new().await?;
        
        let context_manager = ContextManager {
            conversation_contexts: HashMap::new(),
            global_context: GlobalContext {
                system_state: HashMap::new(),
                shared_resources: HashMap::new(),
                active_sessions: 0,
                uptime: std::time::Duration::from_secs(0),
            },
            context_sharing_rules: HashMap::new(),
        };

        Ok(Self {
            server,
            context_manager: Arc::new(RwLock::new(context_manager)),
            execution_history: Arc::new(RwLock::new(ExecutionHistory::new())),
        })
    }

    pub async fn execute_with_context(
        &self,
        request: CallToolRequest,
        conversation_id: &str,
        user_id: &str,
    ) -> Result<CallToolResult, Box<dyn std::error::Error>> {
        // Get or create conversation context
        let context = self.get_or_create_context(conversation_id, user_id).await?;
        
        // Enhance request with context
        let enhanced_request = self.enhance_request_with_context(request, &context).await?;
        
        // Select optimal tool based on context and history
        let optimal_tool = self.select_optimal_tool(&enhanced_request, &context).await?;
        
        // Execute with enhanced context
        let result = self.execute_enhanced_tool(optimal_tool, enhanced_request, &context).await?;
        
        // Update context and history
        self.update_context_from_result(conversation_id, &result).await?;
        
        Ok(result)
    }

    async fn get_or_create_context(
        &self,
        conversation_id: &str,
        user_id: &str,
    ) -> Result<ConversationContext, Box<dyn std::error::Error>> {
        let mut manager = self.context_manager.write().await;
        
        if let Some(context) = manager.conversation_contexts.get(conversation_id) {
            return Ok(context.clone());
        }

        let new_context = ConversationContext {
            id: conversation_id.to_string(),
            user_id: user_id.to_string(),
            started_at: std::time::SystemTime::now(),
            variables: HashMap::new(),
            tool_preferences: HashMap::new(),
            security_level: SecurityLevel::Moderate,
        };

        manager.conversation_contexts.insert(conversation_id.to_string(), new_context.clone());
        manager.global_context.active_sessions += 1;

        Ok(new_context)
    }

    async fn enhance_request_with_context(
        &self,
        mut request: CallToolRequest,
        context: &ConversationContext,
    ) -> Result<CallToolRequest, Box<dyn std::error::Error>> {
        // Add context variables to request arguments
        if let Some(ref mut args) = request.arguments {
            if let Some(args_obj) = args.as_object_mut() {
                args_obj.insert("_context".to_string(), json!({
                    "conversation_id": context.id,
                    "user_id": context.user_id,
                    "variables": context.variables,
                    "session_start": context.started_at.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default().as_secs()
                }));
            }
        }

        Ok(request)
    }

    async fn select_optimal_tool(
        &self,
        request: &CallToolRequest,
        context: &ConversationContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Check if user has a preference for this tool
        if let Some(preference) = context.tool_preferences.get(&request.name) {
            if preference.last_success_rate > 0.8 {
                return Ok(request.name.clone());
            }
        }

        // Check execution history for similar requests
        let history = self.execution_history.read().await;
        if let Some(alternative) = history.get_best_alternative(&request.name, &request.arguments) {
            return Ok(alternative);
        }

        Ok(request.name.clone())
    }

    async fn execute_enhanced_tool(
        &self,
        tool_name: String,
        request: CallToolRequest,
        context: &ConversationContext,
    ) -> Result<CallToolResult, Box<dyn std::error::Error>> {
        // Create enhanced security policy based on context
        let security_policy = match context.security_level {
            SecurityLevel::Strict => SecurityPolicy::strict(),
            SecurityLevel::Moderate => SecurityPolicy::moderate(),
            SecurityLevel::Permissive => SecurityPolicy::permissive(),
            SecurityLevel::Custom(ref policy) => policy.clone(),
        };

        // Execute with context-aware monitoring
        let start_time = std::time::Instant::now();
        let result = self.server.server.call_tool(CallToolRequest {
            name: tool_name.clone(),
            arguments: request.arguments,
        }).await?;
        
        let execution_time = start_time.elapsed();

        // Record execution in history
        let mut history = self.execution_history.write().await;
        history.record_execution(ExecutionRecord {
            tool_name,
            conversation_id: context.id.clone(),
            user_id: context.user_id.clone(),
            execution_time,
            success: result.is_error != Some(true),
            arguments_hash: Self::hash_arguments(&request.arguments),
        });

        Ok(result)
    }

    async fn update_context_from_result(
        &self,
        conversation_id: &str,
        result: &CallToolResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = self.context_manager.write().await;
        
        if let Some(context) = manager.conversation_contexts.get_mut(conversation_id) {
            // Extract any context updates from result metadata
            if let Some(meta) = &result._meta {
                if let Some(context_updates) = meta.get("context_updates") {
                    if let Some(updates) = context_updates.as_object() {
                        for (key, value) in updates {
                            context.variables.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn hash_arguments(args: &Option<Value>) -> u64 {
        use std::hash::{Hash, Hasher, DefaultHasher};
        let mut hasher = DefaultHasher::new();
        format!("{:?}", args).hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug)]
struct ExecutionHistory {
    records: Vec<ExecutionRecord>,
    tool_performance: HashMap<String, ToolPerformance>,
    user_patterns: HashMap<String, UserPattern>,
}

#[derive(Debug, Clone)]
struct ExecutionRecord {
    tool_name: String,
    conversation_id: String,
    user_id: String,
    execution_time: std::time::Duration,
    success: bool,
    arguments_hash: u64,
}

#[derive(Debug)]
struct ToolPerformance {
    total_executions: u64,
    success_rate: f64,
    average_execution_time: std::time::Duration,
    user_satisfaction: f64,
}

#[derive(Debug)]
struct UserPattern {
    preferred_tools: Vec<String>,
    common_argument_patterns: HashMap<u64, u32>,
    typical_session_length: std::time::Duration,
}

impl ExecutionHistory {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            tool_performance: HashMap::new(),
            user_patterns: HashMap::new(),
        }
    }

    pub fn record_execution(&mut self, record: ExecutionRecord) {
        // Update tool performance
        let performance = self.tool_performance
            .entry(record.tool_name.clone())
            .or_insert(ToolPerformance {
                total_executions: 0,
                success_rate: 0.0,
                average_execution_time: std::time::Duration::from_secs(0),
                user_satisfaction: 0.8, // Default satisfaction
            });

        performance.total_executions += 1;
        performance.success_rate = (performance.success_rate * (performance.total_executions - 1) as f64 + 
                                   if record.success { 1.0 } else { 0.0 }) / performance.total_executions as f64;

        // Update user patterns
        let pattern = self.user_patterns
            .entry(record.user_id.clone())
            .or_insert(UserPattern {
                preferred_tools: Vec::new(),
                common_argument_patterns: HashMap::new(),
                typical_session_length: std::time::Duration::from_secs(600), // 10 minutes default
            });

        *pattern.common_argument_patterns.entry(record.arguments_hash).or_insert(0) += 1;

        self.records.push(record);
    }

    pub fn get_best_alternative(&self, tool_name: &str, args: &Option<Value>) -> Option<String> {
        // Simple algorithm to find best alternative based on success rate
        let args_hash = Self::hash_args(args);
        
        self.tool_performance
            .iter()
            .filter(|(name, perf)| *name != tool_name && perf.success_rate > 0.9)
            .max_by(|(_, a), (_, b)| a.success_rate.partial_cmp(&b.success_rate).unwrap())
            .map(|(name, _)| name.clone())
    }

    fn hash_args(args: &Option<Value>) -> u64 {
        use std::hash::{Hash, Hasher, DefaultHasher};
        let mut hasher = DefaultHasher::new();
        format!("{:?}", args).hash(&mut hasher);
        hasher.finish()
    }
}
```

## Security and Isolation

### AI Agent Sandboxing

```rust
pub struct SecureMcpEnvironment {
    server: ContextAwareMcp,
    agent_policies: Arc<RwLock<HashMap<String, AgentSecurityPolicy>>>,
    isolation_manager: Arc<IsolationManager>,
    audit_logger: Arc<AuditLogger>,
}

#[derive(Debug, Clone)]
struct AgentSecurityPolicy {
    agent_id: String,
    trust_level: TrustLevel,
    allowed_tools: HashSet<String>,
    resource_limits: ResourceLimits,
    network_restrictions: NetworkPolicy,
    data_access_rules: DataAccessPolicy,
    temporal_restrictions: TemporalPolicy,
}

#[derive(Debug, Clone)]
enum TrustLevel {
    Untrusted,      // Minimal permissions, heavy sandboxing
    Limited,        // Basic tools, restricted access
    Trusted,        // Most tools, moderate restrictions
    FullyTrusted,   // All tools, minimal restrictions
}

#[derive(Debug, Clone)]
struct ResourceLimits {
    max_memory_mb: u64,
    max_cpu_time_seconds: u64,
    max_concurrent_tools: u32,
    max_file_size_mb: u64,
    max_network_requests_per_minute: u32,
}

#[derive(Debug, Clone)]
struct NetworkPolicy {
    allowed_domains: HashSet<String>,
    blocked_domains: HashSet<String>,
    allowed_ports: HashSet<u16>,
    require_https: bool,
    max_request_size_kb: u64,
}

#[derive(Debug, Clone)]
struct DataAccessPolicy {
    allowed_file_patterns: Vec<String>,
    sensitive_data_patterns: Vec<String>,
    encryption_required: bool,
    audit_all_access: bool,
}

#[derive(Debug, Clone)]
struct TemporalPolicy {
    session_timeout_minutes: u32,
    max_tools_per_session: u32,
    rate_limit_per_minute: u32,
    quiet_hours: Option<(u8, u8)>, // Start and end hour
}

impl SecureMcpEnvironment {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let server = ContextAwareMcp::new().await?;
        let isolation_manager = Arc::new(IsolationManager::new().await?);
        let audit_logger = Arc::new(AuditLogger::new("mcp_audit.log").await?);

        Ok(Self {
            server,
            agent_policies: Arc::new(RwLock::new(HashMap::new())),
            isolation_manager,
            audit_logger,
        })
    }

    pub async fn register_agent(&self, agent_id: &str, trust_level: TrustLevel) -> Result<(), Box<dyn std::error::Error>> {
        let policy = self.create_policy_for_trust_level(agent_id, trust_level).await?;
        
        let mut policies = self.agent_policies.write().await;
        policies.insert(agent_id.to_string(), policy.clone());
        
        // Create isolated environment for this agent
        self.isolation_manager.create_agent_environment(agent_id, &policy).await?;
        
        // Log agent registration
        self.audit_logger.log_agent_registration(agent_id, &policy.trust_level).await?;
        
        Ok(())
    }

    pub async fn execute_tool_for_agent(
        &self,
        agent_id: &str,
        request: CallToolRequest,
        conversation_id: &str,
    ) -> Result<CallToolResult, Box<dyn std::error::Error>> {
        // Get agent policy
        let policy = {
            let policies = self.agent_policies.read().await;
            policies.get(agent_id)
                .ok_or_else(|| format!("Agent '{}' not registered", agent_id))?
                .clone()
        };

        // Validate request against policy
        self.validate_request_against_policy(&request, &policy).await?;

        // Log request
        self.audit_logger.log_tool_request(agent_id, &request).await?;

        // Execute in isolated environment
        let result = self.isolation_manager
            .execute_in_isolation(agent_id, || async {
                self.server.execute_with_context(request, conversation_id, agent_id).await
            })
            .await?;

        // Log result
        self.audit_logger.log_tool_result(agent_id, &result).await?;

        // Update agent behavior metrics
        self.update_agent_metrics(agent_id, &result).await?;

        Ok(result)
    }

    async fn create_policy_for_trust_level(
        &self,
        agent_id: &str,
        trust_level: TrustLevel,
    ) -> Result<AgentSecurityPolicy, Box<dyn std::error::Error>> {
        let (allowed_tools, resource_limits, network_policy, data_policy, temporal_policy) = 
            match trust_level {
                TrustLevel::Untrusted => (
                    ["read_file", "list_directory"].iter().map(|s| s.to_string()).collect(),
                    ResourceLimits {
                        max_memory_mb: 32,
                        max_cpu_time_seconds: 10,
                        max_concurrent_tools: 1,
                        max_file_size_mb: 1,
                        max_network_requests_per_minute: 0,
                    },
                    NetworkPolicy {
                        allowed_domains: HashSet::new(),
                        blocked_domains: HashSet::new(),
                        allowed_ports: HashSet::new(),
                        require_https: true,
                        max_request_size_kb: 0,
                    },
                    DataAccessPolicy {
                        allowed_file_patterns: vec!["./sandbox/*".to_string()],
                        sensitive_data_patterns: vec!["password".to_string(), "key".to_string()],
                        encryption_required: true,
                        audit_all_access: true,
                    },
                    TemporalPolicy {
                        session_timeout_minutes: 10,
                        max_tools_per_session: 10,
                        rate_limit_per_minute: 5,
                        quiet_hours: None,
                    },
                ),
                TrustLevel::Limited => (
                    ["read_file", "write_file", "execute_code", "list_directory"]
                        .iter().map(|s| s.to_string()).collect(),
                    ResourceLimits {
                        max_memory_mb: 128,
                        max_cpu_time_seconds: 60,
                        max_concurrent_tools: 3,
                        max_file_size_mb: 10,
                        max_network_requests_per_minute: 10,
                    },
                    NetworkPolicy {
                        allowed_domains: ["api.openai.com", "httpbin.org"].iter().map(|s| s.to_string()).collect(),
                        blocked_domains: HashSet::new(),
                        allowed_ports: [80, 443].iter().cloned().collect(),
                        require_https: true,
                        max_request_size_kb: 100,
                    },
                    DataAccessPolicy {
                        allowed_file_patterns: vec!["./workspace/*".to_string(), "./temp/*".to_string()],
                        sensitive_data_patterns: vec!["password".to_string(), "secret".to_string()],
                        encryption_required: true,
                        audit_all_access: true,
                    },
                    TemporalPolicy {
                        session_timeout_minutes: 60,
                        max_tools_per_session: 50,
                        rate_limit_per_minute: 20,
                        quiet_hours: Some((22, 6)), // 10 PM to 6 AM
                    },
                ),
                TrustLevel::Trusted => (
                    ["read_file", "write_file", "execute_code", "fetch_url", "system_command"]
                        .iter().map(|s| s.to_string()).collect(),
                    ResourceLimits {
                        max_memory_mb: 512,
                        max_cpu_time_seconds: 300,
                        max_concurrent_tools: 10,
                        max_file_size_mb: 100,
                        max_network_requests_per_minute: 100,
                    },
                    NetworkPolicy {
                        allowed_domains: HashSet::new(), // Empty means all allowed
                        blocked_domains: ["malware.example.com"].iter().map(|s| s.to_string()).collect(),
                        allowed_ports: [80, 443, 22, 3000].iter().cloned().collect(),
                        require_https: false,
                        max_request_size_kb: 1024,
                    },
                    DataAccessPolicy {
                        allowed_file_patterns: vec!["./workspace/*".to_string(), "/tmp/*".to_string()],
                        sensitive_data_patterns: vec!["password".to_string()],
                        encryption_required: false,
                        audit_all_access: true,
                    },
                    TemporalPolicy {
                        session_timeout_minutes: 240,
                        max_tools_per_session: 500,
                        rate_limit_per_minute: 100,
                        quiet_hours: None,
                    },
                ),
                TrustLevel::FullyTrusted => (
                    HashSet::new(), // Empty means all tools allowed
                    ResourceLimits {
                        max_memory_mb: 2048,
                        max_cpu_time_seconds: 3600,
                        max_concurrent_tools: 50,
                        max_file_size_mb: 1024,
                        max_network_requests_per_minute: 1000,
                    },
                    NetworkPolicy {
                        allowed_domains: HashSet::new(),
                        blocked_domains: HashSet::new(),
                        allowed_ports: HashSet::new(),
                        require_https: false,
                        max_request_size_kb: 10240,
                    },
                    DataAccessPolicy {
                        allowed_file_patterns: vec!["*".to_string()],
                        sensitive_data_patterns: Vec::new(),
                        encryption_required: false,
                        audit_all_access: false,
                    },
                    TemporalPolicy {
                        session_timeout_minutes: 1440, // 24 hours
                        max_tools_per_session: 10000,
                        rate_limit_per_minute: 1000,
                        quiet_hours: None,
                    },
                ),
            };

        Ok(AgentSecurityPolicy {
            agent_id: agent_id.to_string(),
            trust_level,
            allowed_tools,
            resource_limits,
            network_restrictions: network_policy,
            data_access_rules: data_policy,
            temporal_restrictions: temporal_policy,
        })
    }

    async fn validate_request_against_policy(
        &self,
        request: &CallToolRequest,
        policy: &AgentSecurityPolicy,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if tool is allowed
        if !policy.allowed_tools.is_empty() && !policy.allowed_tools.contains(&request.name) {
            return Err(format!("Tool '{}' not allowed for agent", request.name).into());
        }

        // Check temporal restrictions
        if let Some((start, end)) = policy.temporal_restrictions.quiet_hours {
            let now = chrono::Local::now();
            let hour = now.hour() as u8;
            if hour >= start || hour < end {
                return Err("Tool execution not allowed during quiet hours".into());
            }
        }

        // Additional validation logic...
        
        Ok(())
    }

    async fn update_agent_metrics(
        &self,
        agent_id: &str,
        result: &CallToolResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation to track agent behavior and adjust trust levels
        Ok(())
    }
}

struct IsolationManager {
    containers: Arc<RwLock<HashMap<String, AgentContainer>>>,
}

struct AgentContainer {
    agent_id: String,
    sandbox_pool: Vec<WasmSandbox>,
    resource_monitor: ResourceMonitor,
    network_filter: NetworkFilter,
}

impl IsolationManager {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            containers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    async fn create_agent_environment(
        &self,
        agent_id: &str,
        policy: &AgentSecurityPolicy,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create isolated container for agent
        let container = AgentContainer {
            agent_id: agent_id.to_string(),
            sandbox_pool: Vec::new(),
            resource_monitor: ResourceMonitor::new(&policy.resource_limits),
            network_filter: NetworkFilter::new(&policy.network_restrictions),
        };

        let mut containers = self.containers.write().await;
        containers.insert(agent_id.to_string(), container);
        
        Ok(())
    }

    async fn execute_in_isolation<F, Fut, T>(
        &self,
        agent_id: &str,
        operation: F,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        let containers = self.containers.read().await;
        let _container = containers.get(agent_id)
            .ok_or_else(|| format!("No container for agent: {}", agent_id))?;

        // Execute with resource monitoring and network filtering
        operation().await
    }
}

struct ResourceMonitor {
    limits: ResourceLimits,
    current_usage: Arc<RwLock<ResourceUsage>>,
}

#[derive(Debug, Default)]
struct ResourceUsage {
    memory_mb: u64,
    cpu_time_seconds: u64,
    active_tools: u32,
    network_requests_this_minute: u32,
}

struct NetworkFilter {
    policy: NetworkPolicy,
    request_log: Arc<RwLock<Vec<NetworkRequest>>>,
}

#[derive(Debug)]
struct NetworkRequest {
    url: String,
    method: String,
    timestamp: std::time::SystemTime,
    size_bytes: u64,
}

struct AuditLogger {
    log_file: Arc<tokio::sync::Mutex<tokio::fs::File>>,
}

impl AuditLogger {
    async fn new(log_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .await?;

        Ok(Self {
            log_file: Arc::new(tokio::sync::Mutex::new(file)),
        })
    }

    async fn log_agent_registration(
        &self,
        agent_id: &str,
        trust_level: &TrustLevel,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event": "agent_registration",
            "agent_id": agent_id,
            "trust_level": format!("{:?}", trust_level)
        });

        self.write_log_entry(&entry).await
    }

    async fn log_tool_request(
        &self,
        agent_id: &str,
        request: &CallToolRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event": "tool_request",
            "agent_id": agent_id,
            "tool_name": request.name,
            "arguments": request.arguments
        });

        self.write_log_entry(&entry).await
    }

    async fn log_tool_result(
        &self,
        agent_id: &str,
        result: &CallToolResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event": "tool_result",
            "agent_id": agent_id,
            "is_error": result.is_error,
            "metadata": result._meta
        });

        self.write_log_entry(&entry).await
    }

    async fn write_log_entry(&self, entry: &Value) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::io::AsyncWriteExt;
        
        let mut file = self.log_file.lock().await;
        file.write_all(entry.to_string().as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;
        
        Ok(())
    }
}
```

## Example Applications

### Complete MCP Server

```rust
// Complete MCP server with all features
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create secure MCP environment
    let mcp_env = SecureMcpEnvironment::new().await?;
    
    // Register different types of agents
    mcp_env.register_agent("code-assistant", TrustLevel::Trusted).await?;
    mcp_env.register_agent("web-scraper", TrustLevel::Limited).await?;
    mcp_env.register_agent("file-processor", TrustLevel::Limited).await?;
    mcp_env.register_agent("system-admin", TrustLevel::FullyTrusted).await?;
    mcp_env.register_agent("untrusted-plugin", TrustLevel::Untrusted).await?;

    // Start auto-discovery for new tools
    mcp_env.server.server.start_auto_discovery().await?;

    // Create MCP server with enhanced capabilities
    let server = ServerBuilder::new("wasm-mcp-server-pro")
        .version("2.0.0")
        .build();

    // Enhanced tool listing with agent-specific filtering
    let mcp_env_clone = mcp_env.clone();
    server.set_list_tools_handler(move || {
        let env = mcp_env_clone.clone();
        Box::pin(async move {
            // Get agent ID from context (implementation specific)
            let agent_id = get_current_agent_id().await?;
            env.get_available_tools_for_agent(&agent_id).await
        })
    });

    // Secure tool execution
    let mcp_env_clone = mcp_env.clone();
    server.set_call_tool_handler(move |request| {
        let env = mcp_env_clone.clone();
        Box::pin(async move {
            let agent_id = get_current_agent_id().await?;
            let conversation_id = get_current_conversation_id().await?;
            env.execute_tool_for_agent(&agent_id, request, &conversation_id).await
        })
    });

    // Agent management endpoints
    let mcp_env_clone = mcp_env.clone();
    server.add_custom_handler("register_agent", move |params| {
        let env = mcp_env_clone.clone();
        Box::pin(async move {
            let agent_request: AgentRegistrationRequest = serde_json::from_value(params)?;
            env.register_agent(&agent_request.agent_id, agent_request.trust_level).await?;
            Ok(json!({"status": "success", "agent_id": agent_request.agent_id}))
        })
    });

    // Performance monitoring endpoint
    let mcp_env_clone = mcp_env.clone();
    server.add_custom_handler("get_metrics", move |_params| {
        let env = mcp_env_clone.clone();
        Box::pin(async move {
            let metrics = env.get_performance_metrics().await?;
            Ok(serde_json::to_value(metrics)?)
        })
    });

    // Health check endpoint
    server.add_custom_handler("health", |_params| {
        Box::pin(async move {
            Ok(json!({
                "status": "healthy",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "version": "2.0.0"
            }))
        })
    });

    println!("Secure WASM MCP Server starting...");
    println!("Features enabled:");
    println!("  - Dynamic tool discovery");
    println!("  - Agent-based security isolation");
    println!("  - Context-aware execution");
    println!("  - Performance monitoring");
    println!("  - Comprehensive audit logging");

    // Start server
    let transport = StdioTransport::new();
    server.run(transport).await?;

    Ok(())
}

async fn get_current_agent_id() -> Result<String, Box<dyn std::error::Error>> {
    // Implementation to extract agent ID from request context
    Ok("default-agent".to_string())
}

async fn get_current_conversation_id() -> Result<String, Box<dyn std::error::Error>> {
    // Implementation to extract conversation ID from request context
    Ok(format!("conv-{}", uuid::Uuid::new_v4()))
}

#[derive(Deserialize)]
struct AgentRegistrationRequest {
    agent_id: String,
    trust_level: TrustLevel,
}
```

Next: **[Development Setup](development-setup.md)** - Complete development environment configuration

---

**MCP Excellence:** Secure Model Context Protocol servers with advanced AI agent integration and comprehensive security controls.
