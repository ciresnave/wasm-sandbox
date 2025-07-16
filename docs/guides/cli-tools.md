# CLI Tools

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Building powerful command-line tools with wasm-sandbox for secure script execution, plugin systems, and extensible automation workflows.

## Overview

CLI tools with wasm-sandbox enable secure execution of user scripts, plugin-based command extensions, and isolated processing of user data with comprehensive resource controls.

## Basic CLI Application

### Simple Command Processor

```rust
use clap::{Arg, Command, ArgMatches};
use wasm_sandbox::{WasmSandbox, SecurityPolicy, Capability};
use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct CliRequest {
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    working_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CliResponse {
    exit_code: i32,
    stdout: String,
    stderr: String,
    execution_time_ms: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("wasm-cli")
        .version("1.0")
        .about("WebAssembly-powered CLI tool")
        .arg(Arg::new("script")
            .short('s')
            .long("script")
            .value_name("FILE")
            .help("WebAssembly script to execute")
            .required(true))
        .arg(Arg::new("function")
            .short('f')
            .long("function")
            .value_name("NAME")
            .help("Function to call in the script")
            .default_value("main"))
        .arg(Arg::new("args")
            .long("args")
            .value_name("ARGS")
            .help("Arguments to pass to the function")
            .num_args(0..))
        .arg(Arg::new("timeout")
            .long("timeout")
            .value_name("SECONDS")
            .help("Execution timeout in seconds")
            .default_value("30"))
        .arg(Arg::new("memory-limit")
            .long("memory-limit")
            .value_name("MB")
            .help("Memory limit in megabytes")
            .default_value("64"))
        .arg(Arg::new("allow-network")
            .long("allow-network")
            .action(clap::ArgAction::SetTrue)
            .help("Allow network access"))
        .arg(Arg::new("allow-fs")
            .long("allow-fs")
            .action(clap::ArgAction::SetTrue)
            .help("Allow filesystem access"))
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .action(clap::ArgAction::SetTrue)
            .help("Enable verbose output"))
        .get_matches();

    let result = execute_script(&matches).await?;
    
    if matches.get_flag("verbose") {
        println!("Execution completed in {}ms", result.execution_time_ms);
        if !result.stderr.is_empty() {
            eprintln!("STDERR: {}", result.stderr);
        }
    }
    
    print!("{}", result.stdout);
    std::process::exit(result.exit_code);
}

async fn execute_script(matches: &ArgMatches) -> Result<CliResponse, Box<dyn std::error::Error>> {
    let script_path = matches.get_one::<String>("script").unwrap();
    let function_name = matches.get_one::<String>("function").unwrap();
    let timeout_secs: u64 = matches.get_one::<String>("timeout").unwrap().parse()?;
    let memory_limit_mb: u64 = matches.get_one::<String>("memory-limit").unwrap().parse()?;
    
    let args: Vec<String> = matches.get_many::<String>("args")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();

    let start_time = std::time::Instant::now();

    // Build security policy based on flags
    let mut policy = SecurityPolicy::strict();
    if matches.get_flag("allow-network") {
        policy = policy.add_capability(Capability::NetworkAccess {
            allowed_hosts: vec!["*".to_string()],
            allowed_ports: vec![80, 443],
        });
    }
    if matches.get_flag("allow-fs") {
        policy = policy.add_capability(Capability::FileSystemAccess {
            allowed_paths: vec![std::env::current_dir()?.to_string_lossy().to_string()],
            read_only: false,
        });
    }

    // Create sandbox
    let sandbox = WasmSandbox::builder()
        .source(script_path)
        .security_policy(policy)
        .memory_limit(memory_limit_mb * 1024 * 1024)
        .cpu_timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .await?;

    // Prepare execution context
    let cli_request = CliRequest {
        command: function_name.clone(),
        args: args.clone(),
        env: std::env::vars().collect(),
        working_dir: std::env::current_dir()?.to_string_lossy().to_string(),
    };

    // Execute the function
    let result = match sandbox.call::<CliRequest, CliResponse>(function_name, &cli_request).await {
        Ok(response) => response,
        Err(e) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("Execution error: {}", e),
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        },
    };

    Ok(result)
}
```

### Plugin System

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde_json::Value;

pub struct PluginManager {
    plugins: HashMap<String, Plugin>,
    plugin_dir: PathBuf,
}

#[derive(Debug)]
pub struct Plugin {
    name: String,
    version: String,
    description: String,
    commands: Vec<String>,
    sandbox: WasmSandbox,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginManifest {
    name: String,
    version: String,
    description: String,
    wasm_file: String,
    commands: Vec<CommandDef>,
    permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandDef {
    name: String,
    description: String,
    function: String,
    args: Vec<ArgDef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ArgDef {
    name: String,
    description: String,
    required: bool,
    arg_type: String,
}

impl PluginManager {
    pub async fn new<P: AsRef<Path>>(plugin_dir: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut manager = Self {
            plugins: HashMap::new(),
            plugin_dir: plugin_dir.as_ref().to_path_buf(),
        };
        
        manager.load_plugins().await?;
        Ok(manager)
    }

    async fn load_plugins(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = tokio::fs::read_dir(&self.plugin_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Err(e) = self.load_plugin_from_dir(entry.path()).await {
                    eprintln!("Failed to load plugin from {:?}: {}", entry.path(), e);
                }
            }
        }
        
        println!("Loaded {} plugins", self.plugins.len());
        Ok(())
    }

    async fn load_plugin_from_dir(&mut self, dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let manifest_path = dir.join("plugin.json");
        let manifest_content = tokio::fs::read_to_string(&manifest_path).await?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)?;

        let wasm_path = dir.join(&manifest.wasm_file);
        
        // Build security policy from permissions
        let mut policy = SecurityPolicy::strict();
        for permission in &manifest.permissions {
            match permission.as_str() {
                "network" => {
                    policy = policy.add_capability(Capability::NetworkAccess {
                        allowed_hosts: vec!["*".to_string()],
                        allowed_ports: vec![80, 443],
                    });
                }
                "filesystem" => {
                    policy = policy.add_capability(Capability::FileSystemAccess {
                        allowed_paths: vec![".".to_string()],
                        read_only: false,
                    });
                }
                "env" => {
                    policy = policy.add_capability(Capability::EnvironmentAccess);
                }
                _ => {
                    eprintln!("Unknown permission: {}", permission);
                }
            }
        }

        let sandbox = WasmSandbox::builder()
            .source(&wasm_path)
            .security_policy(policy)
            .memory_limit(128 * 1024 * 1024) // 128MB default
            .build()
            .await?;

        let plugin = Plugin {
            name: manifest.name.clone(),
            version: manifest.version,
            description: manifest.description,
            commands: manifest.commands.iter().map(|c| c.name.clone()).collect(),
            sandbox,
        };

        self.plugins.insert(manifest.name, plugin);
        Ok(())
    }

    pub fn list_plugins(&self) -> Vec<&Plugin> {
        self.plugins.values().collect()
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Plugin> {
        self.plugins.get(name)
    }

    pub async fn execute_command(
        &self,
        plugin_name: &str,
        command: &str,
        args: Vec<String>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let plugin = self.get_plugin(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        if !plugin.commands.contains(&command.to_string()) {
            return Err(format!("Command '{}' not found in plugin '{}'", command, plugin_name).into());
        }

        let result = plugin.sandbox.call(command, &args).await?;
        Ok(result)
    }
}

// CLI integration
async fn handle_plugin_command(
    plugin_manager: &PluginManager,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("list", _)) => {
            println!("Available plugins:");
            for plugin in plugin_manager.list_plugins() {
                println!("  {} v{} - {}", plugin.name, plugin.version, plugin.description);
                for command in &plugin.commands {
                    println!("    - {}", command);
                }
            }
        }
        Some(("run", sub_matches)) => {
            let plugin_name = sub_matches.get_one::<String>("plugin").unwrap();
            let command = sub_matches.get_one::<String>("command").unwrap();
            let args: Vec<String> = sub_matches.get_many::<String>("args")
                .map(|vals| vals.cloned().collect())
                .unwrap_or_default();

            let result = plugin_manager.execute_command(plugin_name, command, args).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => {
            eprintln!("Invalid plugin command");
        }
    }
    Ok(())
}
```

## Advanced CLI Features

### Interactive Shell

```rust
use rustyline::{Editor, Result as RustyResult};
use rustyline::error::ReadlineError;

pub struct InteractiveShell {
    editor: Editor<()>,
    sandbox: WasmSandbox,
    context: HashMap<String, Value>,
}

impl InteractiveShell {
    pub async fn new(script_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let sandbox = WasmSandbox::builder()
            .source(script_path)
            .security_policy(SecurityPolicy::interactive())
            .persistent_state(true) // Maintain state between calls
            .build()
            .await?;

        // Initialize sandbox context
        sandbox.call::<(), ()>("init_interactive", ()).await?;

        Ok(Self {
            editor: Editor::new()?,
            sandbox,
            context: HashMap::new(),
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("WASM Interactive Shell v1.0");
        println!("Type 'help' for commands, 'exit' to quit");

        loop {
            let readline = self.editor.readline("wasm> ");
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    self.editor.add_history_entry(line);

                    match line {
                        "exit" | "quit" => break,
                        "help" => self.show_help(),
                        "context" => self.show_context(),
                        "reset" => self.reset_context().await?,
                        _ => {
                            if let Err(e) = self.execute_command(line).await {
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn execute_command(&mut self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // Parse command
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let command = parts[0];
        let args = parts.get(1).unwrap_or(&"");

        let request = InteractiveRequest {
            command: command.to_string(),
            args: args.to_string(),
            context: self.context.clone(),
        };

        match self.sandbox.call::<InteractiveRequest, InteractiveResponse>("execute_interactive", &request).await {
            Ok(response) => {
                if !response.output.is_empty() {
                    println!("{}", response.output);
                }
                
                // Update context with any changes
                self.context.extend(response.context_updates);
                
                if response.error {
                    eprintln!("Command failed with exit code: {}", response.exit_code);
                }

                let duration = start_time.elapsed();
                if duration.as_millis() > 100 {
                    println!("(execution time: {}ms)", duration.as_millis());
                }
            }
            Err(e) => {
                eprintln!("Execution failed: {}", e);
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("Available commands:");
        println!("  help     - Show this help message");
        println!("  context  - Show current context variables");
        println!("  reset    - Reset the execution context");
        println!("  exit     - Exit the shell");
        println!();
        println!("Any other input will be executed as a command in the WebAssembly module.");
    }

    fn show_context(&self) {
        if self.context.is_empty() {
            println!("Context is empty");
        } else {
            println!("Current context:");
            for (key, value) in &self.context {
                println!("  {} = {}", key, serde_json::to_string(value).unwrap_or_else(|_| "?".to_string()));
            }
        }
    }

    async fn reset_context(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.context.clear();
        self.sandbox.call::<(), ()>("reset_interactive", ()).await?;
        println!("Context reset");
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct InteractiveRequest {
    command: String,
    args: String,
    context: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct InteractiveResponse {
    output: String,
    error: bool,
    exit_code: i32,
    context_updates: HashMap<String, Value>,
}
```

### Batch Processing

```rust
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};

pub struct BatchProcessor {
    sandbox_pool: Vec<WasmSandbox>,
    concurrency: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct BatchJob {
    id: String,
    input: Value,
    function: String,
    timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BatchResult {
    job_id: String,
    output: Option<Value>,
    error: Option<String>,
    execution_time_ms: u64,
    memory_used: u64,
}

impl BatchProcessor {
    pub async fn new(
        wasm_module: &str,
        concurrency: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut sandbox_pool = Vec::new();
        
        for i in 0..concurrency {
            let sandbox = WasmSandbox::builder()
                .source(wasm_module)
                .instance_id(format!("batch-{}", i))
                .security_policy(SecurityPolicy::batch_processing())
                .build()
                .await?;
            sandbox_pool.push(sandbox);
        }

        Ok(Self {
            sandbox_pool,
            concurrency,
        })
    }

    pub async fn process_jobs(&self, jobs: Vec<BatchJob>) -> Vec<BatchResult> {
        let progress_bar = ProgressBar::new(jobs.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-")
        );

        let results = stream::iter(jobs)
            .map(|job| {
                let sandbox_index = fastrand::usize(..self.sandbox_pool.len());
                let sandbox = &self.sandbox_pool[sandbox_index];
                self.process_single_job(sandbox, job)
            })
            .buffer_unordered(self.concurrency)
            .inspect(|_| {
                progress_bar.inc(1);
            })
            .collect::<Vec<_>>()
            .await;

        progress_bar.finish_with_message("Batch processing completed");
        results
    }

    async fn process_single_job(&self, sandbox: &WasmSandbox, job: BatchJob) -> BatchResult {
        let start_time = std::time::Instant::now();
        
        match tokio::time::timeout(
            std::time::Duration::from_millis(job.timeout_ms),
            sandbox.call::<Value, Value>(&job.function, &job.input)
        ).await {
            Ok(Ok(output)) => BatchResult {
                job_id: job.id,
                output: Some(output),
                error: None,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                memory_used: sandbox.memory_usage().await.unwrap_or(0),
            },
            Ok(Err(e)) => BatchResult {
                job_id: job.id,
                output: None,
                error: Some(e.to_string()),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                memory_used: 0,
            },
            Err(_) => BatchResult {
                job_id: job.id,
                output: None,
                error: Some("Timeout".to_string()),
                execution_time_ms: job.timeout_ms,
                memory_used: 0,
            },
        }
    }

    pub fn save_results(&self, results: &[BatchResult], output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = serde_json::to_string_pretty(results)?;
        std::fs::write(output_file, output)?;
        println!("Results saved to {}", output_file);
        Ok(())
    }

    pub fn print_summary(&self, results: &[BatchResult]) {
        let total = results.len();
        let successful = results.iter().filter(|r| r.error.is_none()).count();
        let failed = total - successful;
        
        let total_time: u64 = results.iter().map(|r| r.execution_time_ms).sum();
        let avg_time = if total > 0 { total_time / total as u64 } else { 0 };
        
        let total_memory: u64 = results.iter().map(|r| r.memory_used).sum();
        let avg_memory = if total > 0 { total_memory / total as u64 } else { 0 };

        println!("\nBatch Processing Summary:");
        println!("  Total jobs: {}", total);
        println!("  Successful: {} ({:.1}%)", successful, (successful as f64 / total as f64) * 100.0);
        println!("  Failed: {} ({:.1}%)", failed, (failed as f64 / total as f64) * 100.0);
        println!("  Average execution time: {}ms", avg_time);
        println!("  Average memory usage: {:.2}MB", avg_memory as f64 / (1024.0 * 1024.0));
        println!("  Total processing time: {:.2}s", total_time as f64 / 1000.0);
    }
}
```

## Configuration Management

### Configuration Files

```rust
use config::{Config, ConfigError, File, Environment};

#[derive(Debug, Deserialize)]
pub struct CliConfig {
    pub sandbox: SandboxConfig,
    pub plugins: PluginConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct SandboxConfig {
    pub default_memory_limit_mb: u64,
    pub default_timeout_seconds: u64,
    pub max_instances: usize,
    pub enable_persistence: bool,
}

#[derive(Debug, Deserialize)]
pub struct PluginConfig {
    pub plugin_dir: String,
    pub auto_load: bool,
    pub max_plugins: usize,
    pub update_check_interval_hours: u64,
}

#[derive(Debug, Deserialize)]
pub struct SecurityConfig {
    pub default_policy: String,
    pub allow_network_by_default: bool,
    pub allow_filesystem_by_default: bool,
    pub enable_audit_logging: bool,
    pub max_execution_time_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub max_file_size_mb: u64,
    pub max_files: u32,
}

impl CliConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Config::builder();

        // Default configuration
        config = config.set_default("sandbox.default_memory_limit_mb", 64)?;
        config = config.set_default("sandbox.default_timeout_seconds", 30)?;
        config = config.set_default("sandbox.max_instances", 10)?;
        config = config.set_default("sandbox.enable_persistence", false)?;

        config = config.set_default("plugins.plugin_dir", "./plugins")?;
        config = config.set_default("plugins.auto_load", true)?;
        config = config.set_default("plugins.max_plugins", 50)?;
        config = config.set_default("plugins.update_check_interval_hours", 24)?;

        config = config.set_default("security.default_policy", "strict")?;
        config = config.set_default("security.allow_network_by_default", false)?;
        config = config.set_default("security.allow_filesystem_by_default", false)?;
        config = config.set_default("security.enable_audit_logging", true)?;
        config = config.set_default("security.max_execution_time_seconds", 300)?;

        config = config.set_default("logging.level", "info")?;
        config = config.set_default("logging.format", "pretty")?;
        config = config.set_default("logging.output", "stdout")?;
        config = config.set_default("logging.max_file_size_mb", 10)?;
        config = config.set_default("logging.max_files", 5)?;

        // Load from files
        let config_dir = dirs::config_dir()
            .map(|d| d.join("wasm-cli"))
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        config = config.add_source(File::from(config_dir.join("config")).required(false));
        config = config.add_source(File::with_name("wasm-cli").required(false));
        config = config.add_source(File::with_name(".wasm-cli").required(false));

        // Environment variables (WASM_CLI_*)
        config = config.add_source(Environment::with_prefix("WASM_CLI").separator("_"));

        config.build()?.try_deserialize()
    }

    pub fn security_policy(&self) -> SecurityPolicy {
        match self.security.default_policy.as_str() {
            "permissive" => SecurityPolicy::permissive(),
            "moderate" => SecurityPolicy::moderate(),
            "paranoid" => SecurityPolicy::paranoid(),
            _ => SecurityPolicy::strict(),
        }
        .with_network_access(self.security.allow_network_by_default)
        .with_filesystem_access(self.security.allow_filesystem_by_default)
        .with_audit_enabled(self.security.enable_audit_logging)
    }
}

// Example configuration file (wasm-cli.toml)
const DEFAULT_CONFIG: &str = r#"
[sandbox]
default_memory_limit_mb = 64
default_timeout_seconds = 30
max_instances = 10
enable_persistence = false

[plugins]
plugin_dir = "./plugins"
auto_load = true
max_plugins = 50
update_check_interval_hours = 24

[security]
default_policy = "strict"
allow_network_by_default = false
allow_filesystem_by_default = false
enable_audit_logging = true
max_execution_time_seconds = 300

[logging]
level = "info"
format = "pretty"
output = "stdout"
max_file_size_mb = 10
max_files = 5
"#;
```

## Testing Framework

### CLI Testing

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_basic_script_execution() {
    let mut cmd = Command::cargo_bin("wasm-cli").unwrap();
    
    cmd.arg("--script")
       .arg("fixtures/test_module.wasm")
       .arg("--function")
       .arg("add")
       .arg("--args")
       .arg("5")
       .arg("3");

    cmd.assert()
       .success()
       .stdout(predicate::str::contains("8"));
}

#[tokio::test]
async fn test_timeout_handling() {
    let mut cmd = Command::cargo_bin("wasm-cli").unwrap();
    
    cmd.arg("--script")
       .arg("fixtures/slow_module.wasm")
       .arg("--timeout")
       .arg("1");

    cmd.assert()
       .failure()
       .stderr(predicate::str::contains("timeout"));
}

#[tokio::test]
async fn test_plugin_system() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().join("plugins");
    
    // Create test plugin
    create_test_plugin(&plugin_dir).await;
    
    let mut cmd = Command::cargo_bin("wasm-cli").unwrap();
    
    cmd.arg("plugin")
       .arg("list")
       .env("WASM_CLI_PLUGINS_PLUGIN_DIR", plugin_dir);

    cmd.assert()
       .success()
       .stdout(predicate::str::contains("test-plugin"));
}

#[tokio::test]
async fn test_interactive_shell() {
    let mut cmd = Command::cargo_bin("wasm-cli").unwrap();
    
    cmd.arg("--interactive")
       .arg("--script")
       .arg("fixtures/test_module.wasm")
       .write_stdin("add 2 3\nexit\n");

    cmd.assert()
       .success()
       .stdout(predicate::str::contains("5"));
}

async fn create_test_plugin(plugin_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let test_plugin_dir = plugin_dir.join("test-plugin");
    tokio::fs::create_dir_all(&test_plugin_dir).await?;
    
    let manifest = serde_json::json!({
        "name": "test-plugin",
        "version": "1.0.0",
        "description": "Test plugin for CLI",
        "wasm_file": "plugin.wasm",
        "commands": [
            {
                "name": "greet",
                "description": "Greet someone",
                "function": "greet",
                "args": [
                    {
                        "name": "name",
                        "description": "Name to greet",
                        "required": true,
                        "type": "string"
                    }
                ]
            }
        ],
        "permissions": ["env"]
    });
    
    tokio::fs::write(
        test_plugin_dir.join("plugin.json"),
        serde_json::to_string_pretty(&manifest)?
    ).await?;
    
    // Copy test WASM file
    tokio::fs::copy(
        "fixtures/test_module.wasm",
        test_plugin_dir.join("plugin.wasm")
    ).await?;
    
    Ok(())
}
```

## Example Applications

### File Processor CLI

```rust
// Complete file processing CLI tool
use glob::glob;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("file-processor")
        .version("1.0")
        .about("Process files with WebAssembly scripts")
        .arg(Arg::new("pattern")
            .short('p')
            .long("pattern")
            .value_name("GLOB")
            .help("File pattern to process")
            .required(true))
        .arg(Arg::new("script")
            .short('s')
            .long("script")
            .value_name("FILE")
            .help("Processing script")
            .required(true))
        .arg(Arg::new("output-dir")
            .short('o')
            .long("output")
            .value_name("DIR")
            .help("Output directory")
            .required(true))
        .arg(Arg::new("parallel")
            .long("parallel")
            .value_name("N")
            .help("Number of parallel workers")
            .default_value("4"))
        .get_matches();

    let pattern = matches.get_one::<String>("pattern").unwrap();
    let script_path = matches.get_one::<String>("script").unwrap();
    let output_dir = matches.get_one::<String>("output-dir").unwrap();
    let parallel: usize = matches.get_one::<String>("parallel").unwrap().parse()?;

    // Create output directory
    tokio::fs::create_dir_all(output_dir).await?;

    // Find files to process
    let files: Vec<_> = glob(pattern)?
        .filter_map(|entry| entry.ok())
        .collect();

    if files.is_empty() {
        eprintln!("No files found matching pattern: {}", pattern);
        return Ok(());
    }

    println!("Found {} files to process", files.len());

    // Create batch processor
    let processor = BatchProcessor::new(script_path, parallel).await?;

    // Create jobs
    let jobs: Vec<BatchJob> = files.into_iter().enumerate().map(|(i, file_path)| {
        BatchJob {
            id: format!("file-{}", i),
            input: serde_json::json!({
                "input_file": file_path.to_string_lossy(),
                "output_dir": output_dir
            }),
            function: "process_file".to_string(),
            timeout_ms: 60000, // 60 seconds per file
        }
    }).collect();

    // Process files
    let results = processor.process_jobs(jobs).await;
    
    // Save results
    processor.save_results(&results, &format!("{}/processing_results.json", output_dir))?;
    processor.print_summary(&results);

    Ok(())
}
```

Next: **[MCP Servers](mcp-servers.md)** - Model Context Protocol server integration with wasm-sandbox

---

**CLI Excellence:** Powerful command-line tools with secure WebAssembly execution, plugin systems, and comprehensive automation capabilities.
