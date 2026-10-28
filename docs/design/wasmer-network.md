# Plan: network capability on the Wasmer backend

Status: **implemented in `src/runtime/wasmer_net.rs` (0.7.6).** Written 2026-10-28 at `main` 288eb26 (0.7.4) for the PM's task `do=A`;
the sections below are the plan as approved. Not published until the PM publishes. The patent addendum that must precede any publish is
`docs/PATENT-SWEEP.md`, Addendum 3.

## Where we are

Since 0.7.4 the Wasmer backend (`wasmer-runtime`, non-Windows) runs WASI modules through wasix and
applies the environment and filesystem grants. A `NetworkCapability` other than `None` is still
rejected with `Error::Capability`, and the guest has no sockets: the wasix runtime is built with
`UnsupportedVirtualNetworking` because wasix's default (`host-vnet`) hands the guest real host
sockets.

Components (Wasmtime, 0.7.0) already enforce all five `NetworkCapability` variants through
`runtime::network_policy::NetworkPolicy`. This plan makes Wasmer enforce the same five, with the
same meaning, by reusing that policy unchanged.

## Design

One new type, `PolicyNetworking`, implementing wasix's `VirtualNetworking` trait. It owns a
`NetworkPolicy` (built once at instance creation by `NetworkPolicy::from_capability`, which already
rejects `HostSpec.secure` and resolves allowed host names once) and an inner
`virtual_net::host::LocalNetworking`. `instantiate_wasi` installs it with
`set_networking_implementation` instead of the unsupported stub. For `NetworkCapability::None` the
stub stays, so the default instance is unchanged.

Grounding (checked in `wasmer-wasix` 0.705.0 / `virtual-net` 0.705.0): wasix never opens a socket
itself. A search of `wasmer-wasix/src` finds no `tokio::net`, `std::net::Tcp*` or `UdpSocket` use; the
only references to the runtime's networking are `Runtime::networking()` and its use from
`state/env.rs`. So wrapping `VirtualNetworking` covers every socket the guest can make.

Per method of `VirtualNetworking` and the sockets it returns:

| Call | Rule |
|---|---|
| `connect_tcp(addr, peer)` | `permits(peer, Remote)`; also `permits(addr, Bind)` for the local side (the wildcard + port 0 implicit bind is allowed by the existing policy) |
| `bind_tcp` / `listen_tcp(addr)` | `permits(addr, Bind)` / `permits(addr, Listen)` |
| `bind_udp(addr)` | `permits(addr, Bind)`; the returned socket is wrapped |
| accepted connection (`try_accept` on the wrapped listener) | `permits(peer, Remote)`; a denied peer's connection is closed and the next one is awaited |
| UDP `try_send_to` / `try_recv_from` on the wrapped socket (also covers wasix's emulated connected UDP) | per datagram, `permits(peer, Remote)`; a datagram from a disallowed source is dropped, a send to one is an error |
| UDP `join_multicast_*`, `set_broadcast(true)` | denied unless the policy is `Any` |
| `resolve` | denied unless `name_lookup_enabled()` (only `Any`), exactly as for components |
| `bind_raw`, `bind_icmp` | always denied: raw and ICMP sockets are not part of any `NetworkCapability` |
| `bridge`, `dhcp_acquire`, `ip_*`, `route_*`, `gateway_set`, `mac` | always denied: they configure the host's virtual network, not the guest's traffic |

Why a wrapper and not a firewall: it is the same shape as the components path (a host-supplied check
on every address the guest uses), needs no OS privilege, and is identical on Linux and macOS.

## What does not change

- `ProcessCapability` other than `None` stays rejected. wasix process spawn needs registered commands
  and none are provided, but enabling it is a separate capability family with its own sweep.
- `HostSpec.secure` stays rejected (cannot be enforced at the socket level), as on components.
- No public API change. A network grant on Wasmer goes from `Err(Error::Capability)` to success, so
  this is additive: by the portfolio's pre-1.0 rule it is a third-digit bump (0.7.5 if nothing else
  lands first). The PM allocates.
- The README enforcement text and the `create_wasi_env` error message change from "not supported yet".

## Tests (all observe the guest, as `tests/wasmer_wasi_test.rs` does)

Real sockets on loopback, each denial paired with an allowed control so a broken test cannot pass
vacuously:
1. `Full`: connect to a host listener succeeds (the positive control for every denial below).
2. `Loopback`: loopback connect succeeds; the same guest bound to the machine's non-loopback address
   is refused.
3. `AllowedPorts`: a port inside the range connects, outside is refused.
4. `AllowedHosts`: the named address connects, another loopback address does not (`127.0.0.2`).
5. `None`: unchanged (the existing no-sockets test).
6. `listen` / explicit `bind` denied unless the capability allows it.
7. IPv4-mapped IPv6 form of an allowed address behaves like the IPv4 form.

Unit tests of `PolicyNetworking` against a fake inner network for what cannot be produced locally:
accept from a disallowed peer is closed, a UDP datagram from a disallowed source is dropped, raw/ICMP
and multicast are denied, `resolve` is denied unless `Any`.

## Risks and open points

- **Every method has a default.** In `virtual-net` 0.705.0 every `VirtualNetworking` method has a
  default body, so a method the wrapper forgets to override silently falls through to the default
  (and to the inner network only if the wrapper delegates it). The wrapper must therefore override
  **every** method explicitly, either checking or denying it, and a unit test must list them so a
  new upstream method is noticed.
- **UDP surface.** A UDP socket's data path in `virtual-net` is `try_send_to` / `try_recv_from` on
  `VirtualConnectionlessSocket`; wasix emulates a "connected" UDP socket on top of those. The check
  goes on those two calls, so it covers both forms. Methods with defaults on the socket traits
  (`set_ttl`, broadcast, multicast) must be reviewed one by one.
- **`Full` and raw sockets.** `Full` means unrestricted TCP/UDP here, not raw. This differs from a
  literal reading of "full"; document it.
- **Resolve once.** Same limit as components: an allowed name that later resolves elsewhere is
  denied, never allowed.
- **Wasix upgrades.** The method list above is for 0.705.0. A new `VirtualNetworking` method with a
  permissive default would be a hole; the test suite should assert the wrapper overrides every
  method (a unit test that lists them).
- **Windows.** Not applicable: `wasmer-runtime` does not build there.

## Decision requested

Approve this design (Option A of the PM's [ASK] list is scope: env/fs done, network next) so the
implementation can start after the patent addendum is accepted. Process stays out of scope.
