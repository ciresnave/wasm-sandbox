//! Cargo build system integration for compiling Rust to WebAssembly

use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;

use crate::error::{Error, Result};
use super::{Compiler, CompilerOptions, BuildProfile};

/// Enhanced Cargo compiler implementation with additional features
pub struct EnhancedCargoCompiler {
    /// Environment variables to set during compilation
    env_vars: HashMap<String, String>,
    
    /// Additional cargo flags
    cargo_flags: Vec<String>,
    
    /// Target directory override
    target_dir: Option<PathBuf>,
    
    /// Rustup toolchain overrides by target
    toolchain_overrides: HashMap<String, String>,
    
    /// Cache directory for build artifacts
    cache_dir: Option<PathBuf>,
}

impl Default for EnhancedCargoCompiler {
    fn default() -> Self {
        Self {
            env_vars: HashMap::new(),
            cargo_flags: Vec::new(),
            target_dir: None,
            toolchain_overrides: HashMap::new(),
            cache_dir: None,
        }
    }
}

impl EnhancedCargoCompiler {
    /// Create a new enhanced cargo compiler
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add an environment variable for the compilation process
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }
    
    /// Add a cargo flag
    pub fn with_cargo_flag(mut self, flag: impl Into<String>) -> Self {
        self.cargo_flags.push(flag.into());
        self
    }
    
    /// Set the target directory
    pub fn with_target_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.target_dir = Some(dir.into());
        self
    }
    
    /// Set a toolchain override for a specific target
    pub fn with_toolchain_override(
        mut self,
        target: impl Into<String>,
        toolchain: impl Into<String>,
    ) -> Self {
        self.toolchain_overrides.insert(target.into(), toolchain.into());
        self
    }
    
    /// Set the cache directory
    pub fn with_cache_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(dir.into());
        self
    }
    
    /// Generate a full RUSTFLAGS string based on compiler options
    fn generate_rustflags(&self, options: &CompilerOptions) -> String {
        let mut flags = Vec::<String>::new();
        
        // Add optimization level
        let opt_flag = match options.opt_level {
            super::OptimizationLevel::None => "-C opt-level=0".to_string(),
            super::OptimizationLevel::Basic => "-C opt-level=1".to_string(),
            super::OptimizationLevel::Default => "-C opt-level=2".to_string(),
            super::OptimizationLevel::Size => "-C opt-level=s".to_string(),
            super::OptimizationLevel::Speed => "-C opt-level=3".to_string(),
        };
        flags.push(opt_flag);
        
        // Add debug level
        let debug_flag = match options.debug_level {
            super::DebugLevel::None => "-C debuginfo=0".to_string(),
            super::DebugLevel::Basic => "-C debuginfo=1".to_string(),
            super::DebugLevel::Full => "-C debuginfo=2".to_string(),
        };
        flags.push(debug_flag);
        
        // Add target CPU if specified
        if let Some(ref cpu) = options.target_cpu {
            flags.push(format!("-C target-cpu={}", cpu));
        }
        
        // Add any additional RUSTFLAGS from options
        if let Some(ref rustflags) = options.rustflags {
            flags.push(rustflags.clone());
        }
        
        flags.join(" ")
    }
}

impl Compiler for EnhancedCargoCompiler {
    fn compile(
        &self,
        project_path: &Path,
        output_path: &Path,
        options: &CompilerOptions,
    ) -> Result<PathBuf> {
        // Check if cargo is available
        if !self.check_available() {
            return Err(Error::Compilation("Cargo is not available".to_string()));
        }
        
        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_path)
            .map_err(|e| Error::FileSystem(format!("Failed to create output directory: {}", e)))?;
        
        // Build the cargo command
        let mut cmd = Command::new("cargo");
        
        // Determine the toolchain to use
        let toolchain = self.toolchain_overrides
            .get(&options.target)
            .unwrap_or(&options.toolchain);
        
        // Add the toolchain if not stable
        if toolchain != "stable" {
            cmd.arg(format!("+{}", toolchain));
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
        
        // Set RUSTFLAGS environment variable
        cmd.env("RUSTFLAGS", self.generate_rustflags(options));
        
        // Add custom target directory if specified
        if let Some(ref target_dir) = self.target_dir {
            cmd.arg("--target-dir").arg(target_dir);
        }
        
        // Add features
        if !options.features.is_empty() {
            cmd.arg("--features").arg(options.features.join(","));
        }
        
        // No default features
        if options.no_default_features {
            cmd.arg("--no-default-features");
        }
        
        // Add cargo flags
        for flag in &self.cargo_flags {
            cmd.arg(flag);
        }
        
        // Add extra args from options
        for arg in &options.extra_args {
            cmd.arg(arg);
        }
        
        // Add environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }
        
        // Run the build
        let output = cmd.output()
            .map_err(|e| Error::Compilation(format!("Failed to execute cargo: {}", e)))?;
        
        // Check for errors
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Compilation(format!("Build failed: {}", stderr)));
        }
        
        // Determine the output file path
        let profile_dir = match options.profile {
            BuildProfile::Debug => "debug",
            BuildProfile::Release => "release",
        };
        
        // Get the project name from Cargo.toml
        let package_name = self.get_package_name(project_path)?;
        
        // Construct the path to the wasm file
        let default_target_dir = project_path.join("target");
        let target_dir = self.target_dir
            .as_ref()
            .unwrap_or(&default_target_dir);
            
        let wasm_file = format!("{}.wasm", package_name);
        let wasm_path = target_dir
            .join(&options.target)
            .join(profile_dir)
            .join(wasm_file);
        
        // Copy to the output path
        let output_wasm_path = output_path.join(format!("{}.wasm", package_name));
        std::fs::copy(&wasm_path, &output_wasm_path)
            .map_err(|e| Error::FileSystem(format!("Failed to copy WASM file: {}", e)))?;
        
        // Copy .d.ts file if available (useful for WASM-bindgen projects)
        let dts_path = wasm_path.with_extension("d.ts");
        if dts_path.exists() {
            let output_dts_path = output_wasm_path.with_extension("d.ts");
            std::fs::copy(&dts_path, &output_dts_path)
                .map_err(|e| Error::FileSystem(format!("Failed to copy .d.ts file: {}", e)))?;
        }
        
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
            .map_err(|e| Error::Compilation(format!("Failed to get cargo version: {}", e)))?;
        
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(version)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(Error::Compilation(format!("Failed to get cargo version: {}", stderr)))
        }
    }
}

impl EnhancedCargoCompiler {
    /// Get the package name from Cargo.toml
    fn get_package_name(&self, project_path: &Path) -> Result<String> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        let cargo_toml = std::fs::read_to_string(cargo_toml_path)
            .map_err(|e| Error::FileSystem(format!("Failed to read Cargo.toml: {}", e)))?;
        
        // Simple parser to extract the package name
        cargo_toml
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
            .ok_or_else(|| Error::Compilation("Failed to determine package name".to_string()))
    }
}

/// Caching cargo compiler which stores build results to avoid recompilation
pub struct CachingCargoCompiler {
    /// Inner compiler
    inner: EnhancedCargoCompiler,
    
    /// Cache directory
    cache_dir: PathBuf,
}

impl CachingCargoCompiler {
    /// Create a new caching cargo compiler
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        let cache_dir = cache_dir.into();
        
        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_dir).ok();
        
        Self {
            inner: EnhancedCargoCompiler::default().with_cache_dir(&cache_dir),
            cache_dir,
        }
    }
    
    /// Calculate a hash for the project and options to use as cache key
    fn cache_key(&self, project_path: &Path, options: &CompilerOptions) -> Result<String> {
        // Read Cargo.toml and Cargo.lock for hashing
        let cargo_toml = std::fs::read_to_string(project_path.join("Cargo.toml"))
            .map_err(|e| Error::FileSystem(format!("Failed to read Cargo.toml: {}", e)))?;
            
        let cargo_lock = std::fs::read_to_string(project_path.join("Cargo.lock"))
            .unwrap_or_default();
            
        // Create a string with all the inputs that affect the build
        let inputs = format!(
            "{}:{}:{}:{}:{}:{}",
            cargo_toml,
            cargo_lock,
            options.target,
            format!("{:?}", options.opt_level),
            format!("{:?}", options.profile),
            options.features.join(",")
        );
        
        // Hash the inputs
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        inputs.hash(&mut hasher);
        let hash = hasher.finish();
        
        Ok(format!("{:016x}", hash))
    }
}

impl Compiler for CachingCargoCompiler {
    fn compile(
        &self,
        project_path: &Path,
        output_path: &Path,
        options: &CompilerOptions,
    ) -> Result<PathBuf> {
        // Calculate cache key
        let key = self.cache_key(project_path, options)?;
        
        // Get the project name
        let package_name = self.inner.get_package_name(project_path)?;
        
        // Check if we have a cached version
        let cached_path = self.cache_dir.join(format!("{}-{}.wasm", package_name, key));
        
        if cached_path.exists() {
            // Copy from cache to output path
            let output_wasm_path = output_path.join(format!("{}.wasm", package_name));
            std::fs::copy(&cached_path, &output_wasm_path)
                .map_err(|e| Error::FileSystem(format!("Failed to copy cached WASM file: {}", e)))?;
                
            return Ok(output_wasm_path);
        }
        
        // Not in cache, compile
        let result = self.inner.compile(project_path, output_path, options)?;
        
        // Store in cache
        std::fs::copy(&result, &cached_path)
            .map_err(|e| Error::FileSystem(format!("Failed to cache WASM file: {}", e)))?;
            
        Ok(result)
    }
    
    fn check_available(&self) -> bool {
        self.inner.check_available()
    }
    
    fn version(&self) -> Result<String> {
        self.inner.version()
    }
}
