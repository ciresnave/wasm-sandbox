//! Filesystem and unsupported-capability enforcement (wasmtime backend).
//!
//! The guest is a hand-written WAT module that calls WASI `path_open`, `path_unlink_file` and
//! `fd_prestat_get` directly and returns the raw errno, so each test observes what the guest can
//! really do, not what the configuration says. Preopened directories start at fd 3.

use std::fs;
use std::path::Path;

use wasm_sandbox::security::Capabilities;
use wasm_sandbox::{Error, FilesystemCapability, InstanceConfig, NetworkCapability, WasmSandbox};

const WASI_ERRNO_SUCCESS: i32 = 0;

const GUEST: &str = r#"
(module
  (import "wasi_snapshot_preview1" "path_open"
    (func $path_open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_unlink_file"
    (func $unlink (param i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_prestat_get"
    (func $prestat (param i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 100) "hello.txt")
  (data (i32.const 120) "new.txt")
  (data (i32.const 140) "../secret.txt")

  ;; open hello.txt read-only (rights: fd_read = 1<<1)
  (func (export "open_ro") (param $fd i32) (result i32)
    (call $path_open (local.get $fd) (i32.const 0) (i32.const 100) (i32.const 9)
      (i32.const 0) (i64.const 2) (i64.const 0) (i32.const 0) (i32.const 200)))

  ;; open existing hello.txt for writing (rights: fd_write = 1<<6)
  (func (export "open_existing_for_write") (param $fd i32) (result i32)
    (call $path_open (local.get $fd) (i32.const 0) (i32.const 100) (i32.const 9)
      (i32.const 0) (i64.const 64) (i64.const 0) (i32.const 0) (i32.const 200)))

  ;; create new.txt (oflags CREAT = 1)
  (func (export "create") (param $fd i32) (result i32)
    (call $path_open (local.get $fd) (i32.const 0) (i32.const 120) (i32.const 7)
      (i32.const 1) (i64.const 64) (i64.const 0) (i32.const 0) (i32.const 200)))

  ;; try to open ../secret.txt relative to the preopened directory
  (func (export "open_escape") (param $fd i32) (result i32)
    (call $path_open (local.get $fd) (i32.const 0) (i32.const 140) (i32.const 13)
      (i32.const 0) (i64.const 2) (i64.const 0) (i32.const 0) (i32.const 200)))

  (func (export "unlink") (param $fd i32) (result i32)
    (call $unlink (local.get $fd) (i32.const 100) (i32.const 9)))

  ;; 0 if this fd is a preopened directory, EBADF (8) if there is no such preopen
  (func (export "prestat") (param $fd i32) (result i32)
    (call $prestat (local.get $fd) (i32.const 300)))
)
"#;

fn call(sandbox: &WasmSandbox, id: wasm_sandbox::InstanceId, name: &str, fd: i32) -> i32 {
    sandbox
        .get_instance(id)
        .expect("instance exists")
        .instance
        .call_simple_function(name, &[fd])
        .unwrap_or_else(|e| panic!("calling {name}: {e}"))
}

fn instantiate(
    capabilities: Capabilities,
) -> (WasmSandbox, Result<wasm_sandbox::InstanceId, Error>) {
    let mut sandbox = WasmSandbox::new().expect("sandbox");
    let module = sandbox.load_module(GUEST.as_bytes()).expect("load guest");
    let result = sandbox.create_instance(
        module,
        Some(InstanceConfig {
            capabilities,
            ..Default::default()
        }),
    );
    (sandbox, result)
}

fn host_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("hello.txt"), "hello").unwrap();
    // A sibling the guest must not reach through `..`.
    fs::write(dir.path().parent().unwrap().join("secret.txt"), "secret").ok();
    dir
}

fn caps_with(fs_cap: FilesystemCapability) -> Capabilities {
    Capabilities {
        filesystem: fs_cap,
        ..Capabilities::minimal()
    }
}

fn readable(dir: &Path) -> FilesystemCapability {
    FilesystemCapability {
        readable_dirs: vec![dir.to_path_buf()],
        ..FilesystemCapability::default()
    }
}

fn writable(dir: &Path, create_and_delete: bool) -> FilesystemCapability {
    FilesystemCapability {
        writable_dirs: vec![dir.to_path_buf()],
        allow_create: create_and_delete,
        allow_delete: create_and_delete,
        ..FilesystemCapability::default()
    }
}

#[test]
fn no_directories_means_no_preopen() {
    let (sandbox, id) = instantiate(Capabilities::minimal());
    let id = id.expect("minimal capabilities still create an instance");
    assert_ne!(
        call(&sandbox, id, "prestat", 3),
        WASI_ERRNO_SUCCESS,
        "no preopen at fd 3"
    );
    assert_ne!(call(&sandbox, id, "open_ro", 3), WASI_ERRNO_SUCCESS);
}

#[test]
fn readable_dir_allows_reading_existing_file() {
    let dir = host_dir();
    let (sandbox, id) = instantiate(caps_with(readable(dir.path())));
    let id = id.expect("instance");
    assert_eq!(
        call(&sandbox, id, "prestat", 3),
        WASI_ERRNO_SUCCESS,
        "readable dir is preopened"
    );
    assert_eq!(call(&sandbox, id, "open_ro", 3), WASI_ERRNO_SUCCESS);
}

#[test]
fn readable_dir_denies_write_create_and_delete() {
    let dir = host_dir();
    let (sandbox, id) = instantiate(caps_with(readable(dir.path())));
    let id = id.expect("instance");
    assert_ne!(
        call(&sandbox, id, "open_existing_for_write", 3),
        WASI_ERRNO_SUCCESS
    );
    assert_ne!(call(&sandbox, id, "create", 3), WASI_ERRNO_SUCCESS);
    assert_ne!(call(&sandbox, id, "unlink", 3), WASI_ERRNO_SUCCESS);
    assert!(!dir.path().join("new.txt").exists());
    assert!(dir.path().join("hello.txt").exists());
}

#[test]
fn writable_dir_with_create_and_delete_allows_both() {
    let dir = host_dir();
    let (sandbox, id) = instantiate(caps_with(writable(dir.path(), true)));
    let id = id.expect("instance");
    assert_eq!(call(&sandbox, id, "create", 3), WASI_ERRNO_SUCCESS);
    assert!(dir.path().join("new.txt").exists());
    assert_eq!(call(&sandbox, id, "unlink", 3), WASI_ERRNO_SUCCESS);
    assert!(!dir.path().join("hello.txt").exists());
}

#[test]
fn guest_cannot_escape_the_preopened_directory() {
    let dir = host_dir();
    let (sandbox, id) = instantiate(caps_with(readable(dir.path())));
    let id = id.expect("instance");
    assert_ne!(call(&sandbox, id, "open_escape", 3), WASI_ERRNO_SUCCESS);
}

#[test]
fn unsupported_or_unsatisfiable_grants_are_loud_errors() {
    let dir = host_dir();

    // Network: core modules have no socket API, so a grant cannot be applied.
    let (_s, r) = instantiate(Capabilities {
        network: NetworkCapability::Loopback,
        ..Capabilities::minimal()
    });
    assert!(
        matches!(r, Err(Error::Capability { .. })),
        "network grant: {r:?}"
    );

    // Process creation cannot be applied.
    let (_s, r) = instantiate(Capabilities {
        process: wasm_sandbox::ProcessCapability::Full,
        ..Capabilities::minimal()
    });
    assert!(
        matches!(r, Err(Error::Capability { .. })),
        "process grant: {r:?}"
    );

    // WASI cannot cap file size.
    let mut with_size = writable(dir.path(), true);
    with_size.max_file_size = Some(1024);
    let (_s, r) = instantiate(caps_with(with_size));
    assert!(
        matches!(r, Err(Error::Capability { .. })),
        "max_file_size: {r:?}"
    );

    // WASI cannot allow create without delete (or vice versa).
    let mut split = writable(dir.path(), false);
    split.allow_create = true;
    let (_s, r) = instantiate(caps_with(split));
    assert!(
        matches!(r, Err(Error::Capability { .. })),
        "create without delete: {r:?}"
    );

    // A directory that does not exist.
    let (_s, r) = instantiate(caps_with(readable(&dir.path().join("missing"))));
    assert!(
        matches!(r, Err(Error::Capability { .. })),
        "missing dir: {r:?}"
    );
}
