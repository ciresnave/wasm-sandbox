//! Generic wrapper implementation

use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};
use crate::wrappers::{WrapperGenerator, WrapperSpec};

/// Generic wrapper generator
pub struct GenericGenerator;

impl WrapperGenerator for GenericGenerator {
    fn generate_wrapper(&self, spec: &WrapperSpec) -> Result<String> {
        // Generate the wrapper code
        self.generate_code(spec)
    }
    
    fn compile_wrapper(&self, code: &str, output_path: &Path) -> Result<()> {
        // Create a temporary directory for building
        let temp_dir = tempfile::tempdir()
            .map_err(|e| Error::WrapperGeneration { 
                reason: format!("Failed to create temporary directory: {}", e),
                wrapper_type: Some("generic".to_string()),
            })?;
        
        // Write code to a temporary file
        let src_path = temp_dir.path().join("src/main.rs");
        std::fs::create_dir_all(src_path.parent().unwrap())
            .map_err(|e| Error::WrapperGeneration { 
                reason: format!("Failed to create source directory: {}", e),
                wrapper_type: Some("generic".to_string()),
            })?;
            
        std::fs::write(&src_path, code)
            .map_err(|e| Error::WrapperGeneration {
                reason: format!("Failed to write source code: {}", e),
                wrapper_type: Some("generic".to_string()),
            })?;
            
        // Create Cargo.toml
        let cargo_toml = r#"[package]
name = "generic-wrapper"
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
                wrapper_type: Some("generic".to_string()),
            })?;
            
        // Run cargo build
        let status = Command::new("cargo")
            .current_dir(temp_dir.path())
            .args(["build", "--target", "wasm32-wasi", "--release"])
            .status()
            .map_err(|e| Error::WrapperGeneration {
                reason: format!("Failed to run cargo build: {}", e),
                wrapper_type: Some("generic".to_string()),
            })?;
            
        if !status.success() {
            return Err(Error::WrapperGeneration {
                reason: "Cargo build failed".to_string(),
                wrapper_type: Some("generic".to_string()),
            });
        }
        
        // Copy the output file
        let wasm_path = temp_dir.path()
            .join("target/wasm32-wasi/release/generic-wrapper.wasm");
            
        std::fs::copy(wasm_path, output_path)
            .map_err(|e| Error::WrapperGeneration {
                reason: format!("Failed to copy output file: {}", e),
                wrapper_type: Some("generic".to_string()),
            })?;
            
        Ok(())
    }
}

impl GenericGenerator {
    /// Generate generic wrapper code
    fn generate_code(&self, spec: &WrapperSpec) -> Result<String> {
        let app_path = spec.app_path.to_string_lossy();
        let args = spec.arguments.join("\", \"");
        
        // Format environment variables
        let env_vars = spec.environment.iter()
            .map(|(k, v)| format!("        env_map.insert(\"{}\".to_string(), \"{}\".to_string());", k, v))
            .collect::<Vec<_>>()
            .join("\n");
            
        // Generate custom template variables
        let mut custom_vars = String::new();
        for (key, value) in &spec.template_variables {
            custom_vars.push_str(&format!("    // Custom variable: {}\n", key));
            custom_vars.push_str(&format!("    let {} = \"{}\";\n", key, value));
        }
            
        let code = format!(
            r#"//! Generic wrapper for WASM sandbox

use std::{{
    collections::HashMap,
    process::{{Command, Stdio}},
    sync::{{Arc, Mutex}},
    io::{{BufRead, BufReader}},
}};

use wasm_sandbox::{{
    WasmSandbox, InstanceId, SandboxConfig, InstanceConfig,
    security::{{ResourceLimits, Capabilities}},
    communication::CommunicationChannel,
    error::Result as SandboxResult,
}};

use tracing::{{info, error, debug, warn}};

/// Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting generic wrapper");
    
{custom_vars}
    
    // Create environment map
    let mut env_map = HashMap::new();
{env_vars}
    
    // Create and run the wrapped application
    let result = run_application(
        "{app_path}",
        &["{args}"],
        env_map,
    ).await;
    
    match result {{
        Ok(exit_code) => {{
            info!("Application exited with code: {{}}", exit_code);
            Ok(())
        }},
        Err(e) => {{
            error!("Application failed: {{}}", e);
            Err(e.into())
        }}
    }}
}}

/// Run the wrapped application
async fn run_application(
    app_path: &str,
    args: &[&str],
    env_vars: HashMap<String, String>,
) -> Result<i32, anyhow::Error> {{
    // Create command
    let mut command = Command::new(app_path)
        .args(args)
        .envs(env_vars)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    // Get stdout and stderr
    let stdout = command.stdout.take().expect("Failed to get stdout");
    let stderr = command.stderr.take().expect("Failed to get stderr");
    
    // Spawn threads to handle output
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);
    
    // Handle stdout
    let stdout_handle = std::thread::spawn(move || {{
        for line in stdout_reader.lines() {{
            if let Ok(line) = line {{
                println!("{{}}", line);
            }}
        }}
    }});
    
    // Handle stderr
    let stderr_handle = std::thread::spawn(move || {{
        for line in stderr_reader.lines() {{
            if let Ok(line) = line {{
                eprintln!("{{}}", line);
            }}
        }}
    }});
    
    // Wait for threads to complete
    stdout_handle.join().unwrap();
    stderr_handle.join().unwrap();
    
    // Wait for the process to exit
    let status = command.wait()?;
    let exit_code = status.code().unwrap_or(1);
    
    Ok(exit_code)
}}
"#,
            app_path = app_path,
            args = args,
            env_vars = env_vars,
            custom_vars = custom_vars,
        );
        
        Ok(code)
    }
}
