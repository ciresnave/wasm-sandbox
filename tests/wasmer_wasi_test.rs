//! WASI enforcement on the Wasmer backend (wasix): environment and filesystem grants are applied,
//! network and process grants are rejected, and the guest gets no sockets.
//!
//! Same method as `fs_capability_test.rs`: a hand-written WAT guest calls WASI directly and
//! returns the raw errno, so each test observes what the guest can really do. Wasmer builds on
//! non-Windows only, and `WasmSandbox` prefers Wasmtime when both are enabled, so this drives
//! `WasmerRuntime` directly.
#![cfg(feature = "wasmer-runtime")]

use std::fs;
use std::path::Path;

use wasm_sandbox::runtime::wasmer::WasmerRuntime;
use wasm_sandbox::runtime::{WasmInstance, WasmRuntime};
use wasm_sandbox::security::{Capabilities, ResourceLimits};
use wasm_sandbox::{Error, FilesystemCapability, NetworkCapability};

const WASI_ERRNO_SUCCESS: i32 = 0;

const GUEST: &str = r#"
(module
  (import "wasi_snapshot_preview1" "path_open"
    (func $path_open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_unlink_file"
    (func $unlink (param i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_prestat_get"
    (func $prestat (param i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "fd_write"
    (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "environ_sizes_get"
    (func $environ_sizes_get (param i32 i32) (result i32)))
  (import "wasix_32v1" "sock_open"
    (func $sock_open (param i32 i32 i32 i32) (result i32)))
  (import "wasix_32v1" "sock_connect"
    (func $sock_connect (param i32 i32) (result i32)))
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

  ;; open hello.txt for writing, then write 5 bytes to it; returns the first errno that fails
  (func (export "write_existing") (param $fd i32) (result i32)
    (local $e i32)
    (local.set $e (call $path_open (local.get $fd) (i32.const 0) (i32.const 100) (i32.const 9)
      (i32.const 0) (i64.const 64) (i64.const 0) (i32.const 0) (i32.const 200)))
    (if (local.get $e) (then (return (local.get $e))))
    (i32.store (i32.const 400) (i32.const 140))
    (i32.store (i32.const 404) (i32.const 5))
    (call $fd_write (i32.load (i32.const 200)) (i32.const 400) (i32.const 1) (i32.const 408)))

  ;; open a TCP socket and connect it to 127.0.0.1:$port; returns the first errno that fails
  (func (export "connect_loopback") (param $port i32) (result i32)
    (local $e i32)
    (local.set $e (call $sock_open (i32.const 1) (i32.const 1) (i32.const 6) (i32.const 600)))
    (if (local.get $e) (then (return (local.get $e))))
    (i32.store8 (i32.const 700) (i32.const 1))
    (i32.store16 (i32.const 701) (local.get $port))
    (i32.store8 (i32.const 703) (i32.const 127))
    (i32.store8 (i32.const 706) (i32.const 1))
    (call $sock_connect (i32.load (i32.const 600)) (i32.const 700)))

  ;; number of environment variables the guest can see
  (func (export "env_count") (param i32) (result i32)
    (drop (call $environ_sizes_get (i32.const 500) (i32.const 504)))
    (i32.load (i32.const 500)))

  (func (export "unlink") (param $fd i32) (result i32)
    (call $unlink (local.get $fd) (i32.const 100) (i32.const 9)))

  ;; 0 if this fd is a preopened directory, EBADF (8) if there is no such preopen
  (func (export "prestat") (param $fd i32) (result i32)
    (call $prestat (local.get $fd) (i32.const 300)))
)
"#;

fn instantiate(capabilities: Capabilities) -> Result<Box<dyn WasmInstance>, Error> {
    let runtime = WasmerRuntime::new().expect("runtime");
    let module = runtime.load_module(GUEST.as_bytes()).expect("load guest");
    runtime.create_instance(module.as_ref(), ResourceLimits::default(), capabilities)
}

fn call(instance: &dyn WasmInstance, name: &str, fd: i32) -> i32 {
    instance
        .call_simple_function(name, &[fd])
        .unwrap_or_else(|e| panic!("calling {name}: {e}"))
}

/// Every fd in 3..16 that is a preopened directory.
fn preopen_fds(instance: &dyn WasmInstance) -> Vec<i32> {
    (3..16)
        .filter(|&fd| call(instance, "prestat", fd) == WASI_ERRNO_SUCCESS)
        .collect()
}

fn host_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("hello.txt"), "hello").unwrap();
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

/// The granted directory. fd 3 is wasix's empty in-memory root, so the first grant is fd 4.
const GRANT: i32 = 4;

#[test]
fn no_directories_means_no_host_access() {
    let instance = instantiate(Capabilities::minimal()).expect("minimal capabilities instantiate");
    assert_eq!(
        preopen_fds(instance.as_ref()),
        vec![3],
        "only the in-memory root"
    );
    assert_ne!(call(instance.as_ref(), "open_ro", 3), WASI_ERRNO_SUCCESS);
}

#[test]
fn readable_dir_allows_reading_and_denies_everything_else() {
    let dir = host_dir();
    let instance = instantiate(caps_with(readable(dir.path()))).expect("instance");
    let i = instance.as_ref();
    assert_eq!(preopen_fds(i), vec![3, GRANT]);
    assert_eq!(call(i, "open_ro", GRANT), WASI_ERRNO_SUCCESS);
    assert_ne!(call(i, "write_existing", GRANT), WASI_ERRNO_SUCCESS);
    assert_ne!(call(i, "create", GRANT), WASI_ERRNO_SUCCESS);
    assert_ne!(call(i, "unlink", GRANT), WASI_ERRNO_SUCCESS);
    assert_ne!(call(i, "open_escape", GRANT), WASI_ERRNO_SUCCESS);
    assert_eq!(
        fs::read_to_string(dir.path().join("hello.txt")).unwrap(),
        "hello"
    );
    assert!(!dir.path().join("new.txt").exists());
}

#[test]
fn writable_dir_allows_write_create_and_delete() {
    let dir = host_dir();
    let instance = instantiate(caps_with(writable(dir.path(), true))).expect("instance");
    let i = instance.as_ref();
    assert_eq!(call(i, "write_existing", GRANT), WASI_ERRNO_SUCCESS);
    assert_eq!(
        fs::read_to_string(dir.path().join("hello.txt")).unwrap(),
        "../se"
    );
    assert_eq!(call(i, "create", GRANT), WASI_ERRNO_SUCCESS);
    assert!(dir.path().join("new.txt").exists());
    assert_eq!(call(i, "unlink", GRANT), WASI_ERRNO_SUCCESS);
    assert!(!dir.path().join("hello.txt").exists());
    assert_ne!(call(i, "open_escape", GRANT), WASI_ERRNO_SUCCESS);
}

/// A symlink inside the granted directory must not lead the guest outside it.
#[cfg(unix)]
#[test]
fn symlink_out_of_the_granted_directory_is_refused() {
    let outer = tempfile::tempdir().expect("tempdir");
    let granted = outer.path().join("granted");
    fs::create_dir(&granted).unwrap();
    fs::write(outer.path().join("secret.txt"), "secret").unwrap();
    std::os::unix::fs::symlink(outer.path().join("secret.txt"), granted.join("hello.txt")).unwrap();

    let instance = instantiate(caps_with(readable(&granted))).expect("instance");
    assert_ne!(
        call(instance.as_ref(), "open_ro", GRANT),
        WASI_ERRNO_SUCCESS,
        "hello.txt is a symlink to a file outside the grant"
    );

    // Positive control: with a regular file in place the same open succeeds
    // (`readable_dir_allows_reading_and_denies_everything_else`). wasix refuses to follow
    // symlinks at all, even inside the grant, which is safe and is why there is no in-grant case.
}

#[test]
fn environment_grants_are_applied() {
    use wasm_sandbox::EnvironmentCapability;
    let with_env = |environment| {
        let instance = instantiate(Capabilities {
            environment,
            ..Capabilities::minimal()
        })
        .expect("instance");
        call(instance.as_ref(), "env_count", 0)
    };
    assert_eq!(with_env(EnvironmentCapability::None), 0);
    assert_eq!(
        with_env(EnvironmentCapability::Allowlist(vec!["PATH".into()])),
        1
    );
    assert_eq!(
        with_env(EnvironmentCapability::Allowlist(vec![
            "NO_SUCH_VARIABLE_X".into()
        ])),
        0
    );
    assert_eq!(
        with_env(EnvironmentCapability::Full),
        std::env::vars().count() as i32
    );
}

/// With no network grant the guest still must not reach the host's network: wasix's default
/// runtime would let it, so this is the test that the stub networking is in place.
#[test]
fn guest_cannot_reach_the_host_network() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();

    let instance = instantiate(Capabilities::minimal()).expect("instance");
    let errno = instance
        .call_simple_function("connect_loopback", &[i32::from(port)])
        .expect("call");
    eprintln!("connect errno: {errno}");
    assert_ne!(errno, WASI_ERRNO_SUCCESS, "connect must fail");
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(
        listener.accept().is_err(),
        "the host listener must not see a connection"
    );
}

#[test]
fn unsupported_or_unsatisfiable_grants_are_loud_errors() {
    let dir = host_dir();
    let err = |caps: Capabilities| matches!(instantiate(caps), Err(Error::Capability { .. }));

    assert!(err(Capabilities {
        network: NetworkCapability::Loopback,
        ..Capabilities::minimal()
    }));
    assert!(err(Capabilities {
        process: wasm_sandbox::ProcessCapability::Full,
        ..Capabilities::minimal()
    }));
    let mut with_size = writable(dir.path(), true);
    with_size.max_file_size = Some(1024);
    assert!(err(caps_with(with_size)));
    let mut split = writable(dir.path(), false);
    split.allow_create = true;
    assert!(err(caps_with(split)));
    assert!(err(caps_with(readable(&dir.path().join("missing")))));
}
