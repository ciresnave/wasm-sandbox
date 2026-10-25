# net-guest: network probe fixture

A tiny WASI preview-2 command component used by `tests/component_network_test.rs`. It reads the
`NETTEST_TARGET` environment variable (an `ip:port`), tries `TcpStream::connect`, and exits with a
code for the outcome (table in `src/main.rs`): 0 connected, 3 permission denied (the host refused the
address), 4 connection refused, and a few more.

The committed binary is `../net_guest.wasm`. The test asserts its SHA-256, so it cannot change
without the test (and this file) being updated.

## Provenance

- Source: `src/main.rs` in this directory (original code, MIT, same licence as the crate; no third-party code, no dependencies).
- The binary also contains the Rust standard library for `wasm32-wasip2` (MIT OR Apache-2.0), linked statically by rustc.
- Built with: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, target `wasm32-wasip2`.
- Command (from this directory): `rustup target add wasm32-wasip2 && cargo build --release --target wasm32-wasip2`
  then `cp target/wasm32-wasip2/release/net-guest.wasm ../net_guest.wasm`.
- SHA-256 of `../net_guest.wasm`: `3c06f74d6e8ea156c6765ba7acd7a76e0d56abcbdf7bf623ba54d2d510a04cca`

A different rustc may produce a different binary; if you rebuild, update the digest in the test and here.
