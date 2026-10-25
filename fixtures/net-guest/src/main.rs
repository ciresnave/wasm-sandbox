//! Network probe used by `tests/component_network_test.rs`.
//!
//! Connects to the address in the `NETTEST_TARGET` environment variable and prints one line to
//! stdout, `RESULT <n>`, then exits normally (WASI preview 2 cannot carry a numeric exit code):
//!
//! - `RESULT 0`: connected.
//! - `RESULT 100+n`: the connect failed with WASI socket error code `n` (`wasi:sockets/network`
//!   `error-code`): 1 access-denied, 2 not-supported, 3 invalid-argument, 5 timeout,
//!   11 address-not-bindable, 13 remote-unreachable, 14 connection-refused, ...
//! - `RESULT 1`: the connect failed with an error that carries no code.

use std::net::TcpStream;

fn main() {
    let target = std::env::var("NETTEST_TARGET").unwrap_or_default();
    let result = match TcpStream::connect(target.as_str()) {
        Ok(_) => 0,
        Err(e) => e.raw_os_error().map_or(1, |code| 100 + code),
    };
    println!("RESULT {result}");
}
