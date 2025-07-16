//! CLI tool wrapper implementation

use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};
use crate::wrappers::{WrapperGenerator, WrapperSpec, ApplicationType};

/// CLI tool wrapper generator
pub struct CliToolGenerator;

impl WrapperGenerator for CliToolGenerator {
    fn generate_wrapper(&self, spec: &WrapperSpec) -> Result<String> {
        // Extract CLI-specific options
        let interactive = match &spec.app_type {
            ApplicationType::CliTool { interactive } => *interactive,
            _ => return Err(Error::WrapperGeneration {
                reason: "Application type must be CliTool".to_string(),
                wrapper_type: Some("cli_tool".to_string()),
            }),
        };
        
        // Generate the wrapper code
        self.generate_code(spec, interactive)
    }
    
    fn compile_wrapper(&self, code: &str, output_path: &Path) -> Result<()> {
        // Create a temporary directory for building
        let temp_dir = tempfile::tempdir()
            .map_err(|e| Error::WrapperGeneration {
                reason: format!("Failed to create temporary directory: {}", e),
                wrapper_type: Some("cli_tool".to_string()),
            })?;
        
        // Write code to a temporary file
        let src_path = temp_dir.path().join("src/main.rs");
        std::fs::create_dir_all(src_path.parent().unwrap())
            .map_err(|e| Error::WrapperGeneration {
                reason: format!("Failed to create source directory: {}", e),
                wrapper_type: Some("cli_tool".to_string()),
            })?;
            
        std::fs::write(&src_path, code)
            .map_err(|e| Error::WrapperGeneration {
                reason: format!("Failed to write source code: {}", e),
                wrapper_type: Some("cli_tool".to_string()),
            })?;
            
        // Create Cargo.toml
        let cargo_toml = r#"[package]
name = "cli-wrapper"
version = "0.1.0"
edition = "2021"

[dependencies]
wasm-sandbox = { path = "../.." }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
"#.to_string();
        
        std::fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)
            .map_err(|e| Error::WrapperGeneration {
                reason: format!("Failed to write Cargo.toml: {}", e),
                wrapper_type: Some("cli_tool".to_string()),
            })?;
            
        // Run cargo build
        let status = Command::new("cargo")
            .current_dir(temp_dir.path())
            .args(["build", "--target", "wasm32-wasi", "--release"])
            .status()
            .map_err(|e| Error::WrapperGeneration {
                reason: format!("Failed to run cargo build: {}", e),
                wrapper_type: Some("cli_tool".to_string()),
            })?;
            
        if !status.success() {
            return Err(Error::WrapperGeneration {
                reason: "Cargo build failed".to_string(),
                wrapper_type: Some("cli_tool".to_string()),
            });
        }
        
        // Copy the output file
        let wasm_path = temp_dir.path()
            .join("target/wasm32-wasi/release/cli-wrapper.wasm");
            
        std::fs::copy(wasm_path, output_path)
            .map_err(|e| Error::WrapperGeneration {
                reason: format!("Failed to copy output file: {}", e),
                wrapper_type: Some("cli_tool".to_string()),
            })?;
            
        Ok(())
    }
}

impl CliToolGenerator {
    /// Generate CLI tool wrapper code
    fn generate_code(&self, spec: &WrapperSpec, interactive: bool) -> Result<String> {
        let app_path = spec.app_path.to_string_lossy();
        let args = spec.arguments.join(", ");
        
        // Format environment variables
        let env_vars = spec.environment.iter()
            .map(|(k, v)| format!("\"{}\" => \"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(",\n        ");
            
        // Decide between interactive and non-interactive mode
        let mut code = format!(
            r#"//! CLI tool wrapper for WASM sandbox

use std::{{
    process::{{Command, Stdio}},
    io::{{BufRead, BufReader, Write}},
    sync::{{Arc, Mutex}},
    collections::HashMap,
}};

use wasm_sandbox::{{
    WasmSandbox, InstanceId, SandboxConfig, InstanceConfig,
    security::{{ResourceLimits, Capabilities}},
    communication::CommunicationChannel,
}};
use serde_json::Value;
use tracing::{{info, error, debug}};

/// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create sandbox
    let mut sandbox = WasmSandbox::new()?;
    
    // Create command
    let mut command = Command::new("{app_path}")
        .args([{args}])
        .env_clear() // Start with a clean environment
        .envs(vec![
            {env_vars}
        ])
"#,
            app_path = app_path,
            args = args,
            env_vars = env_vars,
        );
        
        // Add stdio configuration based on interactive mode
        if interactive {
            code.push_str(
                r#"        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    // Get stdio handles
    let mut stdin = command.stdin.take().expect("Failed to get stdin");
    let stdout = command.stdout.take().expect("Failed to get stdout");
    let stderr = command.stderr.take().expect("Failed to get stderr");
    
    // Create readers for stdout and stderr
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);
    
    // Create communication channels
    let stdin_channel = Arc::new(Mutex::new(Vec::new()));
    let stdout_channel = Arc::new(Mutex::new(Vec::new()));
    let stderr_channel = Arc::new(Mutex::new(Vec::new()));
    
    // Spawn stdout reader thread
    let stdout_channel_clone = stdout_channel.clone();
    std::thread::spawn(move || {
        let mut lines = stdout_reader.lines();
        while let Some(Ok(line)) = lines.next() {
            println!("{}", line);
            stdout_channel_clone.lock().unwrap().push(line);
        }
    });
    
    // Spawn stderr reader thread
    let stderr_channel_clone = stderr_channel.clone();
    std::thread::spawn(move || {
        let mut lines = stderr_reader.lines();
        while let Some(Ok(line)) = lines.next() {
            eprintln!("{}", line);
            stderr_channel_clone.lock().unwrap().push(line);
        }
    });
    
    // Main input loop
    let mut buffer = String::new();
    loop {
        // Read input
        std::io::stdin().read_line(&mut buffer)?;
        
        // Check for exit command
        if buffer.trim() == "exit" {
            break;
        }
        
        // Send input to process
        stdin.write_all(buffer.as_bytes())?;
        stdin.flush()?;
        
        // Store in channel
        stdin_channel.lock().unwrap().push(buffer.clone());
        
        buffer.clear();
    }
    
    // Wait for process to exit
    let status = command.wait()?;
    info!("Process exited with status: {}", status);
    
    Ok(())
"#
            );
        } else {
            // Non-interactive mode
            code.push_str(
                r#"        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    // Get stdio handles
    let stdout = command.stdout.take().expect("Failed to get stdout");
    let stderr = command.stderr.take().expect("Failed to get stderr");
    
    // Create readers
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);
    
    // Read all stdout
    let stdout_lines: Vec<String> = stdout_reader.lines()
        .filter_map(|line| line.ok())
        .collect();
    
    // Read all stderr
    let stderr_lines: Vec<String> = stderr_reader.lines()
        .filter_map(|line| line.ok())
        .collect();
    
    // Print output
    for line in &stdout_lines {
        println!("{}", line);
    }
    
    for line in &stderr_lines {
        eprintln!("{}", line);
    }
    
    // Wait for process to exit
    let status = command.wait()?;
    info!("Process exited with status: {}", status);
    
    Ok(())
"#
            );
        }
        
        code.push_str("}\n");
        
        Ok(code)
    }
}
