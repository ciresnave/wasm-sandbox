//! Calling a plugin function from async host code.
//!
//! Plugin exports are ordinary synchronous WebAssembly functions: a plugin cannot export an
//! `async fn`. The host side is async, so you `.await` the *call*, not the plugin function.
//! In a Rust plugin the export looks like:
//!
//! ```ignore
//! #[unsafe(no_mangle)]
//! pub extern "C" fn add(a: i32, b: i32) -> i32 { a + b }
//! ```
//!
//! Run with: `cargo run --example sync_plugin_call`

use wasm_sandbox::{Result, WasmSandbox};

const PLUGIN: &[u8] = include_bytes!("../fixtures/test_module.wasm");

#[tokio::main]
async fn main() -> Result<()> {
    let mut sandbox = WasmSandbox::new()?;
    let module_id = sandbox.load_module(PLUGIN)?;
    let instance_id = sandbox.create_instance(module_id, None)?;

    // The plugin function is synchronous; the host call is awaited.
    let sum: i32 = sandbox
        .call_function(instance_id, "add", &(20i32, 22i32))
        .await?;
    println!("add(20, 22) = {sum}");
    Ok(())
}
