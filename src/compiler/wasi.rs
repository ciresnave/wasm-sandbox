//! WASI-specific compiler options and tools for WebAssembly

use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;

use crate::error::{Error, Result};
use super::{Compiler, CompilerOptions};

/// WASI target configuration
#[derive(Debug, Clone)]
pub struct WasiConfig {
    /// WASI version
    pub version: String,
    
    /// Additional WASI features to enable
    pub features: Vec<String>,
    
    /// Mapped directories (host path -> guest path)
    pub mapped_dirs: HashMap<PathBuf, String>,
    
    /// Environment variables to pass to the WASI module
    pub env_vars: HashMap<String, String>,
    
    /// Arguments to pass to the WASI module
    pub args: Vec<String>,
    
    /// Preopened directories
    pub preopens: Vec<String>,
}

impl Default for WasiConfig {
    fn default() -> Self {
        Self {
            version: "snapshot1".to_string(),
            features: vec![],
            mapped_dirs: HashMap::new(),
            env_vars: HashMap::new(),
            args: vec![],
            preopens: vec![],
        }
    }
}

/// WASI compiler that wraps another compiler and adds WASI-specific options
pub struct WasiCompiler<C: Compiler> {
    /// Inner compiler
    inner: C,
    
    /// WASI configuration
    wasi_config: WasiConfig,
}

impl<C: Compiler> WasiCompiler<C> {
    /// Create a new WASI compiler
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            wasi_config: WasiConfig::default(),
        }
    }
    
    /// Set the WASI configuration
    pub fn with_wasi_config(mut self, config: WasiConfig) -> Self {
        self.wasi_config = config;
        self
    }
    
    /// Map a directory from host to guest
    pub fn map_directory(mut self, host_path: impl Into<PathBuf>, guest_path: impl Into<String>) -> Self {
        self.wasi_config.mapped_dirs.insert(host_path.into(), guest_path.into());
        self
    }
    
    /// Add an environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.wasi_config.env_vars.insert(key.into(), value.into());
        self
    }
    
    /// Add a command-line argument
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.wasi_config.args.push(arg.into());
        self
    }
    
    /// Add a preopen directory
    pub fn with_preopen(mut self, dir: impl Into<String>) -> Self {
        self.wasi_config.preopens.push(dir.into());
        self
    }
    
    /// Generate a WASI-specific wrapper file for the WASM module
    fn generate_wasi_wrapper(&self, wasm_path: &Path, _options: &CompilerOptions) -> Result<PathBuf> {
        // Create a wrapper directory next to the WASM file
        let parent = wasm_path.parent().ok_or_else(|| {
            Error::FileSystem("Invalid WASM path, no parent directory".to_string())
        })?;
        
        let wrapper_dir = parent.join("wasi_wrapper");
        std::fs::create_dir_all(&wrapper_dir)
            .map_err(|e| Error::FileSystem(format!("Failed to create wrapper directory: {}", e)))?;
            
        // Generate a wrapper.js file that configures WASI
        let wrapper_path = wrapper_dir.join("wrapper.js");
        
        // Generate environment variables JSON
        let env_vars = serde_json::to_string(&self.wasi_config.env_vars)
            .map_err(|e| Error::Generic(format!("Failed to serialize environment variables: {}", e)))?;
        
        // Generate mapped directories JSON
        let mapped_dirs: HashMap<String, String> = self.wasi_config.mapped_dirs.iter()
            .map(|(host, guest)| (guest.clone(), host.to_string_lossy().to_string()))
            .collect();
            
        let mapped_dirs = serde_json::to_string(&mapped_dirs)
            .map_err(|e| Error::Generic(format!("Failed to serialize mapped directories: {}", e)))?;
            
        // Generate arguments JSON
        let args = serde_json::to_string(&self.wasi_config.args)
            .map_err(|e| Error::Generic(format!("Failed to serialize arguments: {}", e)))?;
            
        // Generate preopens JSON
        let preopens = serde_json::to_string(&self.wasi_config.preopens)
            .map_err(|e| Error::Generic(format!("Failed to serialize preopens: {}", e)))?;
            
        // Create wrapper content
        let wrapper_content = format!(
            r#"// WASI wrapper for {wasm_name}
const fs = require('fs');
const path = require('path');
const {{ WASI }} = require('wasi');

// WASI configuration
const wasi = new WASI({{
  version: '{wasi_version}',
  args: {args},
  env: {env_vars},
  preopens: {preopens},
  mappedDirectories: {mapped_dirs},
}});

// Load the WASM module
async function run() {{
  try {{
    const wasmPath = path.resolve(__dirname, '../{wasm_name}');
    const wasmBuffer = fs.readFileSync(wasmPath);
    const wasmModule = await WebAssembly.compile(wasmBuffer);
    
    // Set up imports
    const importObject = {{
      wasi_snapshot_preview1: wasi.wasiImport,
    }};
    
    // Instantiate the module
    const instance = await WebAssembly.instantiate(wasmModule, importObject);
    
    // Start the module
    wasi.start(instance);
  }} catch (error) {{
    console.error('Error running WASI module:', error);
    process.exit(1);
  }}
}}

run();
"#,
            wasm_name = wasm_path.file_name().unwrap().to_string_lossy(),
            wasi_version = self.wasi_config.version,
            args = args,
            env_vars = env_vars,
            preopens = preopens,
            mapped_dirs = mapped_dirs,
        );
        
        // Write the wrapper file
        std::fs::write(&wrapper_path, wrapper_content)
            .map_err(|e| Error::FileSystem(format!("Failed to write wrapper file: {}", e)))?;
            
        Ok(wrapper_path)
    }
    
    /// Generate a WASI configuration file for runtime use
    fn generate_wasi_config_file(&self, wasm_path: &Path) -> Result<PathBuf> {
        let parent = wasm_path.parent().ok_or_else(|| {
            Error::FileSystem("Invalid WASM path, no parent directory".to_string())
        })?;
        
        let config_path = parent.join(format!("{}.wasi.json", 
            wasm_path.file_stem().unwrap().to_string_lossy()));
            
        // Convert mapped directories to strings
        let mapped_dirs: HashMap<String, String> = self.wasi_config.mapped_dirs.iter()
            .map(|(host, guest)| (guest.clone(), host.to_string_lossy().to_string()))
            .collect();
            
        // Create a serializable config
        let mut config = std::collections::HashMap::new();
        config.insert("version", self.wasi_config.version.clone());
        config.insert("features", format!("{:?}", self.wasi_config.features));
        config.insert("mappedDirs", format!("{:?}", mapped_dirs));
        config.insert("env", format!("{:?}", self.wasi_config.env_vars));
        config.insert("args", format!("{:?}", self.wasi_config.args));
        config.insert("preopens", format!("{:?}", self.wasi_config.preopens));
        
        // Write the config file
        // Simple formatted output
        let mut config_str = String::from("{\n");
        for (key, value) in &config {
            config_str.push_str(&format!("  \"{}\": {},\n", key, value));
        }
        config_str.push_str("}");
            
        std::fs::write(&config_path, config_str)
            .map_err(|e| Error::FileSystem(format!("Failed to write WASI config file: {}", e)))?;
            
        Ok(config_path)
    }
}

impl<C: Compiler> Compiler for WasiCompiler<C> {
    fn compile(
        &self,
        project_path: &Path,
        output_path: &Path,
        options: &CompilerOptions,
    ) -> Result<PathBuf> {
        // First, compile with the inner compiler
        let wasm_path = self.inner.compile(project_path, output_path, options)?;
        
        // Generate a WASI wrapper
        let _wrapper_path = self.generate_wasi_wrapper(&wasm_path, options)?;
        
        // Generate a WASI configuration file
        let _config_path = self.generate_wasi_config_file(&wasm_path)?;
        
        // Return the original WASM path
        Ok(wasm_path)
    }
    
    fn check_available(&self) -> bool {
        // Check if both the inner compiler and wasm32-wasi target are available
        if !self.inner.check_available() {
            return false;
        }
        
        // Check if wasm32-wasi target is installed
        let output = Command::new("rustc")
            .args(["--print", "target-list"])
            .output();
            
        match output {
            Ok(output) if output.status.success() => {
                let targets = String::from_utf8_lossy(&output.stdout);
                targets.contains("wasm32-wasi")
            }
            _ => false,
        }
    }
    
    fn version(&self) -> Result<String> {
        // Combine the inner compiler version with WASI info
        let inner_version = self.inner.version()?;
        
        Ok(format!("{} with WASI {}", inner_version, self.wasi_config.version))
    }
}

/// Utility to check if WASI target is installed and install if needed
pub fn ensure_wasi_target() -> Result<()> {
    // Check if the target is already installed
    let output = Command::new("rustc")
        .args(["--print", "target-list"])
        .output()
        .map_err(|e| Error::Compilation(format!("Failed to execute rustc: {}", e)))?;
        
    if output.status.success() {
        let targets = String::from_utf8_lossy(&output.stdout);
        if targets.contains("wasm32-wasi") {
            return Ok(());
        }
    }
    
    // Target not installed, install it
    let output = Command::new("rustup")
        .args(["target", "add", "wasm32-wasi"])
        .output()
        .map_err(|e| Error::Compilation(format!("Failed to execute rustup: {}", e)))?;
        
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Compilation(format!(
            "Failed to install wasm32-wasi target: {}", stderr
        )));
    }
    
    Ok(())
}

/// Check if a Rust project is compatible with WASI
pub fn check_wasi_compatibility(project_path: &Path) -> Result<bool> {
    // Check if Cargo.toml exists
    let cargo_toml_path = project_path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(Error::FileSystem(format!(
            "Cargo.toml not found at {}", cargo_toml_path.display()
        )));
    }
    
    // Just check that we can read Cargo.toml
    std::fs::read_to_string(cargo_toml_path)
        .map_err(|e| Error::FileSystem(format!("Failed to read Cargo.toml: {}", e)))?;
        
    // Look for potentially incompatible dependencies
    let incompatible_deps = [
        "std::fs::File", "std::net::TcpStream", "std::process::Command",
        "tokio::fs::File", "tokio::net::TcpStream", "tokio::process::Command",
        "async_std::fs::File", "async_std::net::TcpStream", "async_std::process::Command",
    ];
    
    // Check all .rs files in src directory
    let src_dir = project_path.join("src");
    if !src_dir.exists() || !src_dir.is_dir() {
        return Err(Error::FileSystem(format!(
            "src directory not found at {}", src_dir.display()
        )));
    }
    
    let mut compatible = true;
    let mut check_file = |file_path: &Path| -> std::io::Result<()> {
        if !file_path.extension().map_or(false, |ext| ext == "rs") {
            return Ok(());
        }
        
        let content = std::fs::read_to_string(file_path)?;
        
        for dep in &incompatible_deps {
            if content.contains(dep) {
                compatible = false;
                println!("Warning: Potentially incompatible code in {}: {}", 
                    file_path.display(), dep);
            }
        }
        
        Ok(())
    };
    
    // Recursively walk the src directory
    fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path) -> std::io::Result<()>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    cb(&path)?;
                }
            }
        }
        Ok(())
    }
    
    visit_dirs(&src_dir, &mut check_file).map_err(|e| {
        Error::FileSystem(format!("Failed to check source files: {}", e))
    })?;
    
    Ok(compatible)
}
