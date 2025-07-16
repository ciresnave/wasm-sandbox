# Development Tools Integration

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Integration with development tools, IDEs, debuggers, and development workflows for productive wasm-sandbox development.

## IDE Support

### VS Code Extension

```json
{
    "name": "wasm-sandbox",
    "displayName": "WebAssembly Sandbox",
    "description": "Development tools for wasm-sandbox projects",
    "version": "1.0.0",
    "engines": {
        "vscode": "^1.60.0"
    },
    "categories": ["Debuggers", "Other"],
    "activationEvents": [
        "onLanguage:rust",
        "workspaceContains:**/sandbox.toml"
    ],
    "contributes": {
        "commands": [
            {
                "command": "wasm-sandbox.compile",
                "title": "Compile to WebAssembly",
                "category": "WASM Sandbox"
            },
            {
                "command": "wasm-sandbox.run",
                "title": "Run in Sandbox",
                "category": "WASM Sandbox"
            },
            {
                "command": "wasm-sandbox.debug",
                "title": "Debug WebAssembly",
                "category": "WASM Sandbox"
            }
        ],
        "configuration": {
            "title": "WASM Sandbox",
            "properties": {
                "wasm-sandbox.runtime": {
                    "type": "string",
                    "default": "wasmtime",
                    "enum": ["wasmtime", "wasmer"],
                    "description": "WebAssembly runtime to use"
                },
                "wasm-sandbox.optimization": {
                    "type": "string",
                    "default": "speed",
                    "enum": ["none", "speed", "size"],
                    "description": "Optimization level"
                }
            }
        }
    }
}
```

### Language Server Protocol

```rust
use tower_lsp::{LspService, Server};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

pub struct WasmSandboxLanguageServer {
    client: tower_lsp::Client,
    workspace: Arc<RwLock<Workspace>>,
}

#[tower_lsp::async_trait]
impl tower_lsp::LanguageServer for WasmSandboxLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "wasm-sandbox.compile".to_string(),
                        "wasm-sandbox.run".to_string(),
                        "wasm-sandbox.validate".to_string(),
                    ],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = self.provide_completions(&params).await;
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let hover_info = self.provide_hover_info(&params).await;
        Ok(hover_info)
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            "wasm-sandbox.compile" => {
                self.compile_workspace().await?;
                Ok(None)
            }
            "wasm-sandbox.run" => {
                self.run_sandbox().await?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

impl WasmSandboxLanguageServer {
    async fn provide_completions(&self, params: &CompletionParams) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // API completions
        completions.extend(self.get_api_completions());
        
        // Configuration completions
        completions.extend(self.get_config_completions());
        
        // Function completions
        completions.extend(self.get_function_completions());

        completions
    }

    fn get_api_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "WasmSandbox::builder()".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Create a new sandbox builder".to_string()),
                documentation: Some(Documentation::String(
                    "Creates a new WasmSandbox builder with default configuration".to_string()
                )),
                insert_text: Some("WasmSandbox::builder()".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "sandbox.call()".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Call a WebAssembly function".to_string()),
                insert_text: Some("call(\"${1:function_name}\", ${2:args})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }
}
```

## Debugging Support

### Debug Adapter Protocol

```rust
use dap::{DebugAdapter, Event, Request, Response};

pub struct WasmSandboxDebugAdapter {
    sandbox: Option<WasmSandbox>,
    breakpoints: HashMap<String, Vec<Breakpoint>>,
    call_stack: Vec<StackFrame>,
    variables: HashMap<i64, Vec<Variable>>,
}

impl DebugAdapter for WasmSandboxDebugAdapter {
    async fn handle_request(&mut self, request: Request) -> Option<Response> {
        match request.command.as_str() {
            "initialize" => Some(self.handle_initialize(request).await),
            "launch" => Some(self.handle_launch(request).await),
            "setBreakpoints" => Some(self.handle_set_breakpoints(request).await),
            "continue" => Some(self.handle_continue(request).await),
            "next" => Some(self.handle_next(request).await),
            "stepIn" => Some(self.handle_step_in(request).await),
            "stackTrace" => Some(self.handle_stack_trace(request).await),
            "variables" => Some(self.handle_variables(request).await),
            _ => None,
        }
    }

    async fn handle_initialize(&mut self, request: Request) -> Response {
        Response {
            request_seq: request.seq,
            success: true,
            command: request.command,
            body: Some(json!({
                "supportsBreakpointLocationsRequest": true,
                "supportsStepInTargetsRequest": true,
                "supportsConfigurationDoneRequest": true,
                "supportsEvaluateForHovers": true,
                "supportsExceptionInfoRequest": true,
            })),
            ..Default::default()
        }
    }

    async fn handle_launch(&mut self, request: Request) -> Response {
        let args = request.arguments.unwrap_or_default();
        let wasm_file = args["program"].as_str().unwrap_or("program.wasm");

        // Create debug-enabled sandbox
        match WasmSandbox::builder()
            .enable_debugging(true)
            .source_file(wasm_file)
            .build()
            .await
        {
            Ok(sandbox) => {
                self.sandbox = Some(sandbox);
                Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    ..Default::default()
                }
            }
            Err(e) => Response {
                request_seq: request.seq,
                success: false,
                command: request.command,
                message: Some(e.to_string()),
                ..Default::default()
            }
        }
    }

    async fn handle_set_breakpoints(&mut self, request: Request) -> Response {
        let args = request.arguments.unwrap_or_default();
        let source = args["source"]["path"].as_str().unwrap_or("");
        let breakpoints: Vec<_> = args["breakpoints"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|bp| Breakpoint {
                line: bp["line"].as_u64().unwrap_or(0) as usize,
                column: bp.get("column").and_then(|c| c.as_u64()).map(|c| c as usize),
                condition: bp.get("condition").and_then(|c| c.as_str()).map(String::from),
            })
            .collect();

        self.breakpoints.insert(source.to_string(), breakpoints.clone());

        // Set breakpoints in sandbox
        if let Some(sandbox) = &mut self.sandbox {
            for bp in &breakpoints {
                let _ = sandbox.set_breakpoint(BreakpointLocation {
                    file: source.to_string(),
                    line: bp.line,
                    column: bp.column,
                }).await;
            }
        }

        Response {
            request_seq: request.seq,
            success: true,
            command: request.command,
            body: Some(json!({
                "breakpoints": breakpoints.iter().map(|bp| {
                    json!({
                        "verified": true,
                        "line": bp.line,
                        "column": bp.column
                    })
                }).collect::<Vec<_>>()
            })),
            ..Default::default()
        }
    }
}
```

## Build Integration

### Cargo Integration

```toml
# Cargo.toml
[package]
name = "my-wasm-project"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-sandbox = "0.3.0"
serde = { version = "1.0", features = ["derive"] }

[package.metadata.wasm-sandbox]
runtime = "wasmtime"
optimization = "speed"
security = "strict"
resource_limits = { memory = "128MB", cpu = "1.0" }

[[package.metadata.wasm-sandbox.wrapper]]
type = "http_server"
port = 8080
routes = [
    { path = "/api/process", function = "process_data" },
    { path = "/api/health", function = "health_check" }
]
```

### Custom Build Script

```rust
// build.rs
use std::process::Command;

fn main() {
    // Compile to WebAssembly
    let output = Command::new("cargo")
        .args(&[
            "build",
            "--target", "wasm32-wasi",
            "--release"
        ])
        .output()
        .expect("Failed to compile to WebAssembly");

    if !output.status.success() {
        panic!("WebAssembly compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    // Optimize with wasm-opt
    Command::new("wasm-opt")
        .args(&[
            "-O3",
            "--enable-bulk-memory",
            "--enable-reference-types",
            "target/wasm32-wasi/release/my_wasm_project.wasm",
            "-o",
            "optimized.wasm"
        ])
        .output()
        .expect("Failed to optimize WebAssembly");

    // Generate wrapper
    wasm_sandbox::build::generate_wrapper(
        "optimized.wasm",
        &wasm_sandbox::build::WrapperConfig {
            wrapper_type: wasm_sandbox::build::WrapperType::HttpServer,
            output_file: "src/generated_wrapper.rs".into(),
            ..Default::default()
        }
    ).expect("Failed to generate wrapper");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
```

## Testing Framework

```rust
use wasm_sandbox::testing::{SandboxTest, TestBuilder};

#[wasm_sandbox::test]
async fn test_basic_function() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = TestBuilder::new()
        .source("test_module.wasm")
        .timeout(Duration::from_secs(5))
        .build()
        .await?;

    let result: i32 = sandbox.call("add", (2, 3)).await?;
    assert_eq!(result, 5);

    Ok(())
}

#[wasm_sandbox::test]
async fn test_memory_limits() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = TestBuilder::new()
        .source("memory_test.wasm")
        .memory_limit(1024 * 1024) // 1MB
        .build()
        .await?;

    // This should fail due to memory limit
    let result = sandbox.call("allocate_large", 2 * 1024 * 1024).await;
    assert!(result.is_err());

    Ok(())
}

#[wasm_sandbox::benchmark]
async fn bench_function_call(b: &mut Bencher) -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = TestBuilder::new()
        .source("bench_module.wasm")
        .build()
        .await?;

    b.iter(|| async {
        let _result: i32 = sandbox.call("fast_function", 42).await.unwrap();
    });

    Ok(())
}
```

## Performance Profiling

```rust
pub struct SandboxProfiler {
    profiling_data: Arc<RwLock<ProfilingData>>,
    sample_rate: Duration,
}

impl SandboxProfiler {
    pub async fn profile_execution<T, R>(&self, sandbox: &WasmSandbox, function: &str, args: &T) -> Result<ProfiledResult<R>, ProfileError>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        let start_time = Instant::now();
        let start_memory = sandbox.get_memory_usage().await?;
        let start_fuel = sandbox.get_fuel_remaining().await?;

        // Start sampling
        let _sampler = self.start_sampling(sandbox).await?;

        // Execute function
        let result = sandbox.call(function, args).await?;

        // Collect final metrics
        let end_time = Instant::now();
        let end_memory = sandbox.get_memory_usage().await?;
        let end_fuel = sandbox.get_fuel_remaining().await?;

        let profile = ExecutionProfile {
            duration: end_time.duration_since(start_time),
            memory_delta: end_memory.saturating_sub(start_memory),
            fuel_consumed: start_fuel.saturating_sub(end_fuel),
            call_count: self.get_call_count().await,
            hotspots: self.analyze_hotspots().await,
        };

        Ok(ProfiledResult {
            result,
            profile,
        })
    }
}
```

Next: **[Performance Guide](performance.md)** - Optimization strategies

---

**Development Excellence:** Integrate wasm-sandbox seamlessly into your development workflow with powerful tooling support.
