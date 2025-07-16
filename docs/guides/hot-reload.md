# Hot Reload

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Dynamic module updates and hot reloading capabilities for seamless development and production deployment workflows.

## Overview

Hot reload allows updating WebAssembly modules without stopping the running application, enabling rapid development cycles and zero-downtime deployments in production.

## Development Hot Reload

### File Watcher Setup

```rust
use wasm_sandbox::{WasmSandbox, HotReload, FileWatcher};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct HotReloadableSandbox {
    sandbox: Arc<RwLock<WasmSandbox>>,
    file_watcher: FileWatcher,
    module_path: String,
}

impl HotReloadableSandbox {
    pub async fn new(module_path: &str) -> Result<Self> {
        // Initial sandbox creation
        let sandbox = Arc::new(RwLock::new(
            WasmSandbox::builder()
                .source(module_path)
                .hot_reload_enabled(true)
                .build()
                .await?
        ));

        // Set up file watcher
        let file_watcher = FileWatcher::new(module_path.into())?;
        
        Ok(Self {
            sandbox,
            file_watcher,
            module_path: module_path.to_string(),
        })
    }

    pub async fn start_watching(&mut self) -> Result<()> {
        let sandbox_clone = Arc::clone(&self.sandbox);
        let module_path = self.module_path.clone();

        self.file_watcher.on_change(move |event| {
            let sandbox = Arc::clone(&sandbox_clone);
            let path = module_path.clone();
            
            Box::pin(async move {
                if let Event { kind: EventKind::Modify(_), .. } = event {
                    println!("Module file changed, reloading...");
                    
                    if let Err(e) = Self::reload_module(sandbox, &path).await {
                        eprintln!("Hot reload failed: {}", e);
                    } else {
                        println!("Hot reload successful!");
                    }
                }
            })
        }).await?;

        Ok(())
    }

    async fn reload_module(sandbox: Arc<RwLock<WasmSandbox>>, module_path: &str) -> Result<()> {
        // Graceful reload with state preservation
        let mut sandbox_guard = sandbox.write().await;
        
        // Save current state if module supports it
        let saved_state = sandbox_guard.try_call::<(), serde_json::Value>("save_state", ()).await.ok();
        
        // Create new sandbox instance
        let new_sandbox = WasmSandbox::builder()
            .source(module_path)
            .hot_reload_enabled(true)
            .build()
            .await?;

        // Restore state if available
        if let Some(state) = saved_state {
            if let Err(e) = new_sandbox.try_call::<serde_json::Value, ()>("restore_state", &state).await {
                eprintln!("Warning: Could not restore state: {}", e);
            }
        }

        // Replace the sandbox
        *sandbox_guard = new_sandbox;
        
        Ok(())
    }

    pub async fn call<T, A>(&self, function: &str, args: A) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        A: serde::Serialize,
    {
        let sandbox = self.sandbox.read().await;
        sandbox.call(function, args).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut hot_sandbox = HotReloadableSandbox::new("target/wasm32-wasi/debug/my_module.wasm").await?;
    
    // Start watching for file changes
    hot_sandbox.start_watching().await?;
    
    // Use the sandbox normally
    loop {
        let result: String = hot_sandbox.call("process", "test data").await?;
        println!("Result: {}", result);
        
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
```

### Build Integration

```rust
use cargo_metadata::MetadataCommand;
use std::process::Command;

pub struct AutoBuilder {
    manifest_path: String,
    target_dir: String,
    package_name: String,
}

impl AutoBuilder {
    pub fn new(manifest_path: &str) -> Result<Self> {
        let metadata = MetadataCommand::new()
            .manifest_path(manifest_path)
            .exec()?;

        Ok(Self {
            manifest_path: manifest_path.to_string(),
            target_dir: metadata.target_directory.to_string(),
            package_name: metadata.root_package().unwrap().name.clone(),
        })
    }

    pub async fn build_wasm(&self) -> Result<String> {
        println!("Building WebAssembly module...");
        
        let output = Command::new("cargo")
            .args(&[
                "build",
                "--target", "wasm32-wasi",
                "--release",
                "--manifest-path", &self.manifest_path,
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Build failed: {}", stderr).into());
        }

        let wasm_path = format!(
            "{}/wasm32-wasi/release/{}.wasm",
            self.target_dir,
            self.package_name.replace("-", "_")
        );

        println!("Build successful: {}", wasm_path);
        Ok(wasm_path)
    }

    pub async fn watch_and_build<F>(&self, on_build: F) -> Result<()>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let watcher = FileWatcher::new("src".into())?;
        let builder = self.clone();

        watcher.on_change(move |_event| {
            let builder = builder.clone();
            let callback = on_build.clone();
            
            Box::pin(async move {
                match builder.build_wasm().await {
                    Ok(wasm_path) => callback(wasm_path),
                    Err(e) => eprintln!("Build error: {}", e),
                }
            })
        }).await?;

        Ok(())
    }
}
```

## Production Hot Reload

### Zero-Downtime Updates

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use semver::Version;

pub struct ProductionHotReload {
    current_sandbox: Arc<RwLock<WasmSandbox>>,
    staging_sandbox: Arc<RwLock<Option<WasmSandbox>>>,
    version: Arc<RwLock<Version>>,
    update_strategy: UpdateStrategy,
}

#[derive(Clone)]
pub enum UpdateStrategy {
    BlueGreen,
    RollingUpdate { batch_size: usize },
    CanaryDeployment { traffic_percentage: f32 },
}

impl ProductionHotReload {
    pub async fn new(initial_module: &str, strategy: UpdateStrategy) -> Result<Self> {
        let sandbox = WasmSandbox::builder()
            .source(initial_module)
            .production_mode(true)
            .build()
            .await?;

        let version = Version::parse("1.0.0").unwrap();

        Ok(Self {
            current_sandbox: Arc::new(RwLock::new(sandbox)),
            staging_sandbox: Arc::new(RwLock::new(None)),
            version: Arc::new(RwLock::new(version)),
            update_strategy: strategy,
        })
    }

    pub async fn prepare_update(&self, new_module: &str, new_version: Version) -> Result<()> {
        println!("Preparing update to version {}", new_version);

        // Create staging sandbox
        let staging_sandbox = WasmSandbox::builder()
            .source(new_module)
            .production_mode(true)
            .build()
            .await?;

        // Run validation tests
        self.validate_module(&staging_sandbox).await?;

        // Store staging sandbox
        let mut staging = self.staging_sandbox.write().await;
        *staging = Some(staging_sandbox);

        println!("Update prepared successfully");
        Ok(())
    }

    async fn validate_module(&self, sandbox: &WasmSandbox) -> Result<()> {
        // Health check
        let health: String = sandbox.call("health_check", ()).await?;
        if health != "ok" {
            return Err("Health check failed".into());
        }

        // Compatibility check
        let current_version = self.version.read().await;
        let api_version: String = sandbox.call("get_api_version", ()).await?;
        let module_api_version = Version::parse(&api_version)?;

        if module_api_version.major != current_version.major {
            return Err("Breaking API change detected".into());
        }

        // Performance validation
        let start_time = Instant::now();
        let _: String = sandbox.call("benchmark_function", "test_data").await?;
        let duration = start_time.elapsed();

        if duration > Duration::from_millis(100) {
            return Err("Performance regression detected".into());
        }

        Ok(())
    }

    pub async fn execute_update(&self, new_version: Version) -> Result<()> {
        match &self.update_strategy {
            UpdateStrategy::BlueGreen => self.blue_green_update(new_version).await,
            UpdateStrategy::RollingUpdate { batch_size } => {
                self.rolling_update(new_version, *batch_size).await
            }
            UpdateStrategy::CanaryDeployment { traffic_percentage } => {
                self.canary_update(new_version, *traffic_percentage).await
            }
        }
    }

    async fn blue_green_update(&self, new_version: Version) -> Result<()> {
        let staging = self.staging_sandbox.read().await;
        let staging_sandbox = staging.as_ref()
            .ok_or("No staging sandbox prepared")?;

        // Atomic swap
        {
            let mut current = self.current_sandbox.write().await;
            let new_sandbox = staging_sandbox.clone();
            *current = new_sandbox;
        }

        // Update version
        {
            let mut version = self.version.write().await;
            *version = new_version;
        }

        println!("Blue-green update completed");
        Ok(())
    }

    async fn rolling_update(&self, new_version: Version, _batch_size: usize) -> Result<()> {
        // Implementation for rolling updates across multiple instances
        // This would typically coordinate with a load balancer
        println!("Rolling update not implemented in this example");
        Ok(())
    }

    async fn canary_update(&self, new_version: Version, traffic_percentage: f32) -> Result<()> {
        println!("Starting canary deployment with {}% traffic", traffic_percentage * 100.0);

        // Implement gradual traffic shifting
        // Monitor metrics and automatically roll back if issues detected
        
        Ok(())
    }

    pub async fn call<T, A>(&self, function: &str, args: A) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        A: serde::Serialize,
    {
        let sandbox = self.current_sandbox.read().await;
        sandbox.call(function, args).await
    }

    pub async fn rollback(&self, target_version: Version) -> Result<()> {
        println!("Rolling back to version {}", target_version);
        
        // Implementation would restore previous sandbox state
        // This might involve reloading from a backup or registry
        
        Ok(())
    }
}
```

### Module Registry

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ModuleMetadata {
    pub name: String,
    pub version: Version,
    pub checksum: String,
    pub api_version: String,
    pub dependencies: Vec<String>,
    pub size: u64,
    pub upload_time: chrono::DateTime<chrono::Utc>,
}

pub struct ModuleRegistry {
    modules: HashMap<String, Vec<ModuleMetadata>>,
    storage_backend: Box<dyn StorageBackend>,
}

trait StorageBackend: Send + Sync {
    async fn store_module(&self, metadata: &ModuleMetadata, data: &[u8]) -> Result<()>;
    async fn retrieve_module(&self, name: &str, version: &Version) -> Result<Vec<u8>>;
    async fn list_versions(&self, name: &str) -> Result<Vec<Version>>;
}

impl ModuleRegistry {
    pub async fn new(storage: Box<dyn StorageBackend>) -> Self {
        Self {
            modules: HashMap::new(),
            storage_backend: storage,
        }
    }

    pub async fn publish_module(&mut self, name: &str, version: Version, wasm_data: &[u8]) -> Result<()> {
        // Calculate checksum
        let checksum = sha256::digest(wasm_data);
        
        // Extract metadata from module
        let api_version = self.extract_api_version(wasm_data).await?;
        
        let metadata = ModuleMetadata {
            name: name.to_string(),
            version: version.clone(),
            checksum,
            api_version,
            dependencies: vec![], // Extract from module if available
            size: wasm_data.len() as u64,
            upload_time: chrono::Utc::now(),
        };

        // Store module
        self.storage_backend.store_module(&metadata, wasm_data).await?;
        
        // Update registry
        self.modules.entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(metadata);

        println!("Published {} v{}", name, version);
        Ok(())
    }

    pub async fn download_module(&self, name: &str, version: &Version) -> Result<Vec<u8>> {
        // Verify module exists
        let versions = self.modules.get(name)
            .ok_or_else(|| format!("Module '{}' not found", name))?;

        let metadata = versions.iter()
            .find(|m| &m.version == version)
            .ok_or_else(|| format!("Version {} not found for module '{}'", version, name))?;

        // Download from storage
        let data = self.storage_backend.retrieve_module(name, version).await?;
        
        // Verify checksum
        let actual_checksum = sha256::digest(&data);
        if actual_checksum != metadata.checksum {
            return Err("Checksum verification failed".into());
        }

        Ok(data)
    }

    async fn extract_api_version(&self, _wasm_data: &[u8]) -> Result<String> {
        // Parse WebAssembly module to extract API version
        // This would use a WASM parser to read custom sections
        Ok("1.0.0".to_string())
    }
}
```

## Configuration

### Hot Reload Settings

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct HotReloadConfig {
    pub enabled: bool,
    pub watch_paths: Vec<String>,
    pub debounce_ms: u64,
    pub auto_build: bool,
    pub validation_timeout_ms: u64,
    pub rollback_on_failure: bool,
    pub max_rollback_attempts: u32,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            watch_paths: vec!["src".to_string()],
            debounce_ms: 500,
            auto_build: true,
            validation_timeout_ms: 5000,
            rollback_on_failure: true,
            max_rollback_attempts: 3,
        }
    }
}

pub struct HotReloadManager {
    config: HotReloadConfig,
    active_watchers: Vec<FileWatcher>,
    update_history: Vec<UpdateRecord>,
}

#[derive(Clone)]
struct UpdateRecord {
    timestamp: chrono::DateTime<chrono::Utc>,
    from_version: Version,
    to_version: Version,
    success: bool,
    duration: Duration,
    error_message: Option<String>,
}

impl HotReloadManager {
    pub async fn new(config: HotReloadConfig) -> Result<Self> {
        Ok(Self {
            config,
            active_watchers: Vec::new(),
            update_history: Vec::new(),
        })
    }

    pub async fn enable_hot_reload(&mut self, sandbox: &WasmSandbox) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        for path in &self.config.watch_paths {
            let watcher = FileWatcher::new(path.clone())?;
            
            // Configure debouncing
            watcher.set_debounce_duration(Duration::from_millis(self.config.debounce_ms));
            
            self.active_watchers.push(watcher);
        }

        println!("Hot reload enabled for {} paths", self.config.watch_paths.len());
        Ok(())
    }

    pub fn get_update_history(&self) -> &[UpdateRecord] {
        &self.update_history
    }

    pub async fn manual_update(&mut self, new_module_path: &str) -> Result<()> {
        let start_time = Instant::now();
        
        let record = UpdateRecord {
            timestamp: chrono::Utc::now(),
            from_version: Version::parse("1.0.0").unwrap(), // Get from current module
            to_version: Version::parse("1.0.1").unwrap(),   // Get from new module
            success: true, // Will be updated based on result
            duration: start_time.elapsed(),
            error_message: None,
        };

        self.update_history.push(record);
        Ok(())
    }
}
```

## Integration Examples

### VS Code Extension

```typescript
// VS Code extension for hot reload
import * as vscode from 'vscode';
import * as path from 'path';
import { spawn, ChildProcess } from 'child_process';

export class HotReloadManager {
    private watchers: vscode.FileSystemWatcher[] = [];
    private buildProcess: ChildProcess | null = null;

    public activate() {
        // Watch for Rust file changes
        const rustWatcher = vscode.workspace.createFileSystemWatcher('**/*.rs');
        rustWatcher.onDidChange(this.onFileChange.bind(this));
        this.watchers.push(rustWatcher);

        // Watch for Cargo.toml changes
        const cargoWatcher = vscode.workspace.createFileSystemWatcher('**/Cargo.toml');
        cargoWatcher.onDidChange(this.onFileChange.bind(this));
        this.watchers.push(cargoWatcher);

        // Register commands
        vscode.commands.registerCommand('wasm-sandbox.hotReload', this.manualReload.bind(this));
    }

    private async onFileChange(uri: vscode.Uri) {
        if (this.buildProcess) {
            this.buildProcess.kill();
        }

        // Debounce rapid changes
        setTimeout(() => {
            this.buildAndReload();
        }, 500);
    }

    private async buildAndReload() {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) return;

        vscode.window.showInformationMessage('Building WebAssembly module...');

        this.buildProcess = spawn('cargo', ['build', '--target', 'wasm32-wasi'], {
            cwd: workspaceFolder.uri.fsPath,
            stdio: 'pipe'
        });

        this.buildProcess.on('close', (code) => {
            if (code === 0) {
                vscode.window.showInformationMessage('Hot reload successful!');
            } else {
                vscode.window.showErrorMessage('Build failed - check terminal output');
            }
            this.buildProcess = null;
        });
    }

    public dispose() {
        this.watchers.forEach(watcher => watcher.dispose());
        if (this.buildProcess) {
            this.buildProcess.kill();
        }
    }
}
```

### Web Interface

```html
<!DOCTYPE html>
<html>
<head>
    <title>Hot Reload Dashboard</title>
    <style>
        .status { padding: 10px; margin: 5px; border-radius: 5px; }
        .success { background-color: #d4edda; color: #155724; }
        .error { background-color: #f8d7da; color: #721c24; }
        .warning { background-color: #fff3cd; color: #856404; }
    </style>
</head>
<body>
    <h1>WebAssembly Hot Reload Dashboard</h1>
    
    <div id="status" class="status">Checking status...</div>
    
    <button onclick="manualReload()">Manual Reload</button>
    <button onclick="toggleAutoReload()">Toggle Auto Reload</button>
    
    <h2>Update History</h2>
    <div id="history"></div>

    <script>
        let autoReloadEnabled = true;
        
        async function checkStatus() {
            try {
                const response = await fetch('/api/hot-reload/status');
                const status = await response.json();
                
                const statusDiv = document.getElementById('status');
                statusDiv.textContent = `Status: ${status.state} | Version: ${status.version} | Last Update: ${status.lastUpdate}`;
                statusDiv.className = `status ${status.state === 'healthy' ? 'success' : 'warning'}`;
            } catch (error) {
                const statusDiv = document.getElementById('status');
                statusDiv.textContent = 'Error: Unable to connect to server';
                statusDiv.className = 'status error';
            }
        }
        
        async function manualReload() {
            try {
                const response = await fetch('/api/hot-reload/trigger', { method: 'POST' });
                const result = await response.json();
                
                if (result.success) {
                    alert('Reload triggered successfully');
                } else {
                    alert(`Reload failed: ${result.error}`);
                }
            } catch (error) {
                alert('Error triggering reload');
            }
        }
        
        function toggleAutoReload() {
            autoReloadEnabled = !autoReloadEnabled;
            document.querySelector('button[onclick="toggleAutoReload()"]').textContent = 
                `${autoReloadEnabled ? 'Disable' : 'Enable'} Auto Reload`;
        }
        
        // Check status every 5 seconds
        setInterval(checkStatus, 5000);
        checkStatus();
    </script>
</body>
</html>
```

Next: **[HTTP Servers](http-servers.md)** - Building HTTP services with wasm-sandbox

---

**Hot Reload Excellence:** Seamless development workflows and zero-downtime production updates with comprehensive hot reload capabilities.
