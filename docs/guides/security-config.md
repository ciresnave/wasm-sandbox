# Security Configuration Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

This guide covers how to configure wasm-sandbox's security settings: capabilities, resource limits and audit logging.

> **Corrected 2026-10-25.** Earlier versions of this guide documented an API that was never implemented
> (`NetworkCapability::Connect`/`Listen`, list-valued `Capabilities` fields, `FilesystemCapability::ReadOnly(..)`,
> `SecurityProfile`, `TenantConfig`, `AuditConfig` on the builder, security callbacks). Those are gone from
> this page; see [#8](https://github.com/ciresnave/wasm-sandbox/issues/8). Everything below is checked against
> the code and the runnable [`examples/security_config.rs`](../../examples/security_config.rs), which CI compiles.

## What is enforced

WebAssembly itself isolates a module's memory and gives it no host access unless the host provides some.
wasm-sandbox builds a WASI context per instance and adds only what your `Capabilities` grant. With the
wasmtime backend (0.6.x):

| Setting | Behaviour |
|---|---|
| `environment` | **Applied.** Only the variables your `EnvironmentCapability` allows are passed to the guest. |
| `filesystem.readable_dirs` | **Applied.** Each directory is preopened read-only. |
| `filesystem.writable_dirs` | **Applied.** Each directory is preopened read-write (create, delete and modify together). Requires `allow_create` **and** `allow_delete` to be `true` and `max_file_size` to be `None`; anything else is an `Error::Capability`. |
| `network` | **Not supported.** Anything other than `NetworkCapability::None` makes `create_instance` fail with `Error::Capability`. Core WebAssembly modules have no socket API, so there is nothing to grant. |
| `process` | **Not supported.** Anything other than `ProcessCapability::None` is an `Error::Capability`. |
| `time`, `random`, `custom` | Stored only; not applied. |
| `ResourceLimits::fuel` | **Applied** (wasmtime backend): fuel is set when the instance is created. |

A grant this backend cannot honour is an error, not a silent no-op, so a configuration that creates an
instance does what it says. The wasmer backend does not apply filesystem grants yet ([#11](https://github.com/ciresnave/wasm-sandbox/issues/11)).
The other `ResourceLimits` fields exist in the type; this guide does not claim they are enforced.

## Quick start - secure defaults

```rust
use wasm_sandbox::WasmSandbox;

let mut sandbox = WasmSandbox::new()?;
let module_id = sandbox.load_module(&wasm_bytes)?;
// `None` means the default `InstanceConfig`, which uses `Capabilities::minimal()`.
let instance_id = sandbox.create_instance(module_id, None)?;
```

`Capabilities::minimal()` is the most restrictive set: no network, no filesystem directories, no environment
variables, no process creation, read-only time, pseudo-random only.
`Capabilities::development()` is a looser preset (loopback network, the current directory readable, the
temp directory writable, a short environment allow-list) meant for local development, not production.

## Capabilities

`Capabilities` has one field per capability *kind*. Each field is a single value (an enum or a struct), not a
list. Build it from `Capabilities::minimal()` and override only what you need:

```rust
use wasm_sandbox::security::Capabilities;
use wasm_sandbox::EnvironmentCapability;

let capabilities = Capabilities {
    environment: EnvironmentCapability::Allowlist(vec!["API_KEY".to_string()]),
    ..Capabilities::minimal()
};
```

Pass it to an instance through `InstanceConfig`:

```rust
use wasm_sandbox::InstanceConfig;

let instance_id = sandbox.create_instance(
    module_id,
    Some(InstanceConfig { capabilities, ..Default::default() }),
)?;
```

### Network (`NetworkCapability`)

A single enum value:

| Variant | Meaning |
|---|---|
| `None` | No network access (the default, and the only value the wasmtime backend accepts today). |
| `Loopback` | Localhost only. |
| `AllowedHosts(Vec<HostSpec>)` | Specific hosts; `HostSpec { host, ports: Option<PortRange>, secure }`. |
| `AllowedPorts(Vec<PortRange>)` | Specific ports; `PortRange::new(start, end)`. |
| `Full` | Unrestricted. |

Only `None` can be applied: with any other variant, `create_instance` returns `Error::Capability`. There is
no separate "connect" and "listen" permission (`NetworkCapability::Connect`/`::Listen` do not exist).

### Filesystem (`FilesystemCapability`)

A struct, not a list of rules:

```rust
use wasm_sandbox::FilesystemCapability;

let filesystem = FilesystemCapability {
    readable_dirs: vec!["./input".into()],
    writable_dirs: vec!["./output".into()],
    max_file_size: None,   // WASI cannot cap file size
    allow_create: true,    // a writable directory needs both flags
    allow_delete: true,
};
```

Each directory must exist. The guest sees each directory as a WASI preopen mounted at the path you gave
(with `\` written as `/`); preopens start at file descriptor 3, readable directories first, in the order
listed. A directory listed as both readable and writable is mounted once, read-write. The guest cannot
leave a preopened directory (for example with `..`). There are no per-file rules and no glob patterns.

### Environment variables (`EnvironmentCapability`)

`None`, `Allowlist(Vec<String>)`, `Denylist(Vec<String>)` or `Full`. This one is applied: with `Allowlist`,
only the named variables that exist in the host environment are passed to the guest.

## Resource limits

`ResourceLimits` groups `memory`, `cpu`, `io`, `time` and `fuel`:

```rust
use wasm_sandbox::security::ResourceLimits;

use wasm_sandbox::CpuLimits;

let limits = ResourceLimits {
    fuel: Some(1_000_000), // instruction budget (applied, wasmtime backend)
    cpu: CpuLimits { max_execution_time_ms: 2_000, ..CpuLimits::default() },
    ..ResourceLimits::default()
};
```

Set it on `InstanceConfig::resource_limits`. Fuel counts executed WebAssembly instructions, so it bounds
computation independent of wall-clock time; `None` disables the budget. The default is 10,000,000.

The high-level builder (`WasmSandbox::builder()`) has only a few simple switches: `source`, `timeout_duration`,
`memory_limit`, `enable_file_access` and `enable_network`. It has no `.capabilities(..)` or
`.resource_limits(..)` method; use `InstanceConfig` as shown above for full control.

## Audit logging

`wasm_sandbox::security::audit` provides an in-memory `AuditLogger` you can use to record your own events:

```rust
use wasm_sandbox::security::audit::{AuditEventType, AuditLogger};

let logger = AuditLogger::new(1000).with_stdout(); // keep the last 1000 events
logger.warning(
    AuditEventType::Custom {
        event_type: "fuel".to_string(),
        data: "instance near fuel limit".to_string(),
    },
    "instance near fuel limit",
);
let events = logger.get_events();
```

`AuditEventType` also has variants for module loads, instance creation, function calls, resource-limit hits and capability violations. The runtime does not currently write
audit events for you, and there is no security callback or tenant isolation API.

## Handling errors

Failures are `wasm_sandbox::Error` (an alias of `SandboxError`); match the variants you care about, for
example `SecurityViolation { .. }`, `ResourceExhausted { kind, limit, used, .. }`, `Timeout { .. }` and
`FunctionCall { .. }`:

```rust
use wasm_sandbox::SandboxError;

match sandbox.call_function::<_, i32>(instance_id, "process", &input).await {
    Ok(result) => handle(result),
    Err(SandboxError::ResourceExhausted { kind, used, limit, .. }) => {
        eprintln!("{kind:?} exhausted: {used}/{limit}");
    }
    Err(e) => return Err(e),
}
```

## Best practices

1. **Least privilege.** Start from `Capabilities::minimal()` and name only what the module needs.
2. **Set a fuel budget** you can justify; never leave untrusted code on `fuel: None`.
3. **Validate input and output** of every call; the sandbox protects the host, not your data.
4. **Test what the guest can do**, not just what the configuration says: the wasmer backend still ignores
   filesystem grants, and `time`, `random` and `custom` are stored only.

## Checklist before production

- [ ] Capabilities start from `minimal()` and each override is justified
- [ ] A fuel budget is set on every instance running untrusted code
- [ ] Inputs and outputs are validated
- [ ] You tested what the guest can and cannot do (not just what the config says)

## Security updates

Subscribe to repository security advisories, read the [CHANGELOG](../../CHANGELOG.md) for security-related
changes, and re-test your security boundaries after upgrades.

## Next steps

- **[Error Handling Guide](error-handling.md)**
- **[API reference](https://docs.rs/wasm-sandbox)**
