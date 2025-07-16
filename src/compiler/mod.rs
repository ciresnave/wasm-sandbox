//! Wrapper compilation toolchain

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

/// Compiler options
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    /// Target triple (e.g. wasm32-wasi)
    pub target: String,
    
    /// Optimization level
    pub opt_level: OptimizationLevel,
    
    /// Debug info level
    pub debug_level: DebugLevel,
    
    /// Features to enable
    pub features: Vec<String>,
    
    /// No default features flag
    pub no_default_features: bool,
    
    /// Additional compiler arguments
    pub extra_args: Vec<String>,
    
    /// Toolchain version (e.g. "stable", "nightly")
    pub toolchain: String,
    
    /// Build profile
    pub profile: BuildProfile,
    
    /// Target CPU specification (optional)
    pub target_cpu: Option<String>,
    
    /// Additional RUSTFLAGS to pass to the compiler (optional)
    pub rustflags: Option<String>,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            target: "wasm32-wasi".to_string(),
            opt_level: OptimizationLevel::Default,
            debug_level: DebugLevel::None,
            features: Vec::new(),
            no_default_features: false,
            extra_args: Vec::new(),
            toolchain: "stable".to_string(),
            profile: BuildProfile::Release,
            target_cpu: None,
            rustflags: None,
        }
    }
}

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// No optimizations
    None,
    
    /// Basic optimizations
    Basic,
    
    /// Default optimizations
    Default,
    
    /// Size optimizations
    Size,
    
    /// Speed optimizations
    Speed,
}

/// Debug information level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLevel {
    /// No debug information
    None,
    
    /// Basic debug information
    Basic,
    
    /// Full debug information
    Full,
}

/// Build profile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    /// Debug profile
    Debug,
    
    /// Release profile
    Release,
}

/// Compiler abstraction
pub trait Compiler {
    /// Compile a Rust project to WASM
    fn compile(
        &self,
        project_path: &Path,
        output_path: &Path,
        options: &CompilerOptions,
    ) -> Result<PathBuf>;
    
    /// Check if the compiler is available
    fn check_available(&self) -> bool;
    
    /// Get compiler version
    fn version(&self) -> Result<String>;
}

/// Cargo compiler implementation
pub struct CargoCompiler;

impl CargoCompiler {
    /// Create a new cargo compiler
    pub fn new() -> Self {
        Self
    }
}

impl Compiler for CargoCompiler {
    fn compile(
        &self,
        project_path: &Path,
        output_path: &Path,
        options: &CompilerOptions,
    ) -> Result<PathBuf> {
        // Check if cargo is available
        if !self.check_available() {
            return Err(Error::Compilation { message: "Cargo is not available".to_string() });
        }
        
        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_path)
            .map_err(|e| Error::Filesystem { 
                operation: "create_dir_all".to_string(), 
                path: output_path.to_path_buf(), 
                reason: format!("Failed to create output directory: {}", e) 
            })?;
        
        // Build the cargo command
        let mut cmd = Command::new("cargo");
        
        // Add the toolchain if not stable
        if options.toolchain != "stable" {
            cmd.arg(format!("+{}", options.toolchain));
        }
        
        // Basic build command
        cmd.current_dir(project_path)
            .arg("build")
            .arg("--target").arg(&options.target);
        
        // Add profile
        match options.profile {
            BuildProfile::Debug => {
                // Debug is the default, no need to add flags
            },
            BuildProfile::Release => {
                cmd.arg("--release");
            },
        }
        
        // Add optimization level
        match options.opt_level {
            OptimizationLevel::None => {
                cmd.env("RUSTFLAGS", "-C opt-level=0");
            },
            OptimizationLevel::Basic => {
                cmd.env("RUSTFLAGS", "-C opt-level=1");
            },
            OptimizationLevel::Default => {
                // Default, no need to set
            },
            OptimizationLevel::Size => {
                cmd.env("RUSTFLAGS", "-C opt-level=s");
            },
            OptimizationLevel::Speed => {
                cmd.env("RUSTFLAGS", "-C opt-level=3");
            },
        }
        
        // Add debug level
        match options.debug_level {
            DebugLevel::None => {
                // No debug info is the default for release builds
                if options.profile == BuildProfile::Debug {
                    cmd.env("RUSTFLAGS", "-C debuginfo=0");
                }
            },
            DebugLevel::Basic => {
                cmd.env("RUSTFLAGS", "-C debuginfo=1");
            },
            DebugLevel::Full => {
                cmd.env("RUSTFLAGS", "-C debuginfo=2");
            },
        }
        
        // Add features
        if !options.features.is_empty() {
            cmd.arg("--features").arg(options.features.join(","));
        }
        
        // No default features
        if options.no_default_features {
            cmd.arg("--no-default-features");
        }
        
        // Add extra args
        for arg in &options.extra_args {
            cmd.arg(arg);
        }
        
        // Run the build
        let output = cmd.output()
            .map_err(|e| Error::Compilation { message: format!("Failed to execute cargo: {}", e) })?;
        
        // Check for errors
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Compilation { message: format!("Build failed: {}", stderr) });
        }
        
        // Determine the output file path
        let profile_dir = match options.profile {
            BuildProfile::Debug => "debug",
            BuildProfile::Release => "release",
        };
        
        // Get the project name
        let cargo_toml_path = project_path.join("Cargo.toml");
        let cargo_toml = std::fs::read_to_string(&cargo_toml_path)
            .map_err(|e| Error::Filesystem { 
                operation: "read".to_string(), 
                path: cargo_toml_path.clone(), 
                reason: format!("Failed to read Cargo.toml: {}", e) 
            })?;
        
        // Simple parser to extract the package name
        let package_name = cargo_toml
            .lines()
            .find_map(|line| {
                if line.trim().starts_with("name") {
                    line.split('=')
                        .nth(1)
                        .map(|s| s.trim().trim_matches('"').to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| Error::Compilation { message: "Failed to determine package name".to_string() })?;
        
        // Construct the output file path
        let wasm_file = format!("{}.wasm", package_name);
        let wasm_path = project_path
            .join("target")
            .join(&options.target)
            .join(profile_dir)
            .join(wasm_file);
        
        // Copy to the output path
        let output_wasm_path = output_path.join(format!("{}.wasm", package_name));
        std::fs::copy(&wasm_path, &output_wasm_path)
            .map_err(|e| Error::Filesystem { 
                operation: "copy".to_string(), 
                path: wasm_path.clone(), 
                reason: format!("Failed to copy WASM file: {}", e) 
            })?;
        
        Ok(output_wasm_path)
    }
    
    fn check_available(&self) -> bool {
        Command::new("cargo")
            .arg("--version")
            .output()
            .is_ok()
    }
    
    fn version(&self) -> Result<String> {
        let output = Command::new("cargo")
            .arg("--version")
            .output()
            .map_err(|e| Error::Compilation { message: format!("Failed to get cargo version: {}", e) })?;
        
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(version)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(Error::Compilation { message: format!("Failed to get cargo version: {}", stderr) })
        }
    }
}

pub mod cargo;
pub mod wasi;
