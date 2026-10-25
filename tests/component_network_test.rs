//! Network capability enforcement on the component-model path.
//!
//! The guest is `fixtures/net_guest.wasm` (source and build steps in `fixtures/net-guest/`): a
//! WASI preview-2 command that connects to `$NETTEST_TARGET` and prints `RESULT <n>`: 0 connected,
//! `100 + k` for WASI socket error code `k`. `RESULT 101` (access-denied) is what proves the sandbox,
//! not the operating system, refused the connection.

use std::net::TcpListener;
use std::sync::Mutex;

use sha2::{Digest, Sha256};
use wasm_sandbox::runtime::WasmRuntime;
use wasm_sandbox::runtime::component::ComponentRuntime;
use wasm_sandbox::security::{
    Capabilities, HostSpec, NetworkCapability, PortRange, ResourceLimits,
};
use wasm_sandbox::{EnvironmentCapability, Error, ProcessCapability};

const GUEST: &[u8] = include_bytes!("../fixtures/net_guest.wasm");
const GUEST_SHA256: &str = "3c06f74d6e8ea156c6765ba7acd7a76e0d56abcbdf7bf623ba54d2d510a04cca";

const CONNECTED: i32 = 0;
/// `100 + k` for the WASI socket error codes wasmtime-wasi returns when the sandbox refuses:
/// access-denied (1, an address the policy rejects) and not-supported (2, sockets not enabled).
/// Anything else (connected, refused, unreachable) means the policy did not refuse.
const DENIED: [i32; 2] = [101, 102];

/// The guest reads one environment variable, so scenarios must not overlap.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn caps(network: NetworkCapability) -> Capabilities {
    Capabilities {
        network,
        environment: EnvironmentCapability::Allowlist(vec!["NETTEST_TARGET".to_string()]),
        ..Capabilities::minimal()
    }
}

fn run_guest(network: NetworkCapability, target: &str) -> Result<i32, Error> {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // SAFETY: serialised by ENV_LOCK; no other test in this binary touches this variable.
    unsafe { std::env::set_var("NETTEST_TARGET", target) };

    let runtime = ComponentRuntime::new(Capabilities::minimal(), ResourceLimits::default())?;
    let module = runtime.load_module(GUEST)?;
    let instance = runtime.create_command_instance(module.as_ref(), caps(network))?;
    let output = instance.run_command()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result = stdout
        .trim()
        .strip_prefix("RESULT ")
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| {
            panic!(
                "no RESULT line; stdout={stdout:?} stderr={:?}",
                output.stderr
            )
        });
    Ok(result)
}

fn assert_denied(code: i32, label: &str) {
    assert!(
        DENIED.contains(&code),
        "{label}: expected the sandbox to refuse (one of {DENIED:?}), got {code}"
    );
}

fn host(ip: &str, ports: Option<(u16, u16)>) -> HostSpec {
    HostSpec {
        host: ip.to_string(),
        ports: ports.map(|(a, b)| PortRange::new(a, b)),
        secure: false,
    }
}

#[test]
fn fixture_digest_is_pinned() {
    let digest: String = Sha256::digest(GUEST)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        digest, GUEST_SHA256,
        "fixtures/net_guest.wasm changed; update the digest and fixtures/net-guest/README.md"
    );
}

#[test]
fn network_is_denied_unless_granted_and_grants_are_enforced() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let local = format!("127.0.0.1:{port}");
    let other = port.wrapping_add(1).max(1024);

    // Default deny: no grant, no connection (and the refusal comes from the sandbox).
    assert_denied(run_guest(NetworkCapability::None, &local).unwrap(), "None");

    // Positive control for everything below: an allowed connection really connects.
    assert_eq!(
        run_guest(NetworkCapability::Loopback, &local).unwrap(),
        CONNECTED,
        "Loopback"
    );
    assert_eq!(
        run_guest(NetworkCapability::Full, &local).unwrap(),
        CONNECTED,
        "Full"
    );

    // Loopback refuses a non-loopback address, including its IPv4-mapped IPv6 form.
    assert_denied(
        run_guest(NetworkCapability::Loopback, "8.8.8.8:80").unwrap(),
        "denied",
    );
    // The mapped form never reaches the policy: wasmtime-wasi itself rejects IPv4-mapped IPv6
    // addresses with an invalid-argument error (128). The policy's own normalisation of mapped
    // forms is covered by the unit tests in `src/runtime/network_policy.rs`.
    let mapped = run_guest(NetworkCapability::Loopback, "[::ffff:8.8.8.8]:80").unwrap();
    assert!(
        DENIED.contains(&mapped) || mapped == 128,
        "mapped form must not connect, got {mapped}"
    );

    // Ports.
    let ports = |a, b| NetworkCapability::AllowedPorts(vec![PortRange::new(a, b)]);
    assert_eq!(
        run_guest(ports(port, port), &local).unwrap(),
        CONNECTED,
        "port allowed"
    );
    assert_denied(
        run_guest(ports(other, other), &local).unwrap(),
        "other port",
    );

    // Hosts (resolved once at instance creation).
    let hosts = |h: HostSpec| NetworkCapability::AllowedHosts(vec![h]);
    assert_eq!(
        run_guest(hosts(host("127.0.0.1", None)), &local).unwrap(),
        CONNECTED,
        "host any port"
    );
    assert_eq!(
        run_guest(hosts(host("127.0.0.1", Some((port, port)))), &local).unwrap(),
        CONNECTED,
        "host+port"
    );
    assert_denied(
        run_guest(hosts(host("127.0.0.1", Some((other, other)))), &local).unwrap(),
        "host, wrong port",
    );
    assert_denied(
        run_guest(hosts(host("10.0.0.1", None)), &local).unwrap(),
        "other host",
    );
}

#[test]
fn unsatisfiable_network_grants_are_loud_errors() {
    // `secure` cannot be enforced at the socket level.
    let secure = HostSpec {
        secure: true,
        ..host("127.0.0.1", None)
    };
    let r = run_guest(NetworkCapability::AllowedHosts(vec![secure]), "127.0.0.1:1");
    assert!(
        matches!(r, Err(Error::Capability { .. })),
        "secure host: {r:?}"
    );

    // A host name that does not resolve cannot be turned into an allow-list.
    let r = run_guest(
        NetworkCapability::AllowedHosts(vec![host("no-such-host.invalid", None)]),
        "127.0.0.1:1",
    );
    assert!(
        matches!(r, Err(Error::Capability { .. })),
        "unresolvable host: {r:?}"
    );

    // Process creation is not available to components either.
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let runtime =
        ComponentRuntime::new(Capabilities::minimal(), ResourceLimits::default()).unwrap();
    let module = runtime.load_module(GUEST).unwrap();
    let r = runtime.create_instance(
        module.as_ref(),
        ResourceLimits::default(),
        Capabilities {
            process: ProcessCapability::Full,
            ..Capabilities::minimal()
        },
    );
    assert!(matches!(r, Err(Error::Capability { .. })), "process grant");
}
