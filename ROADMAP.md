# Roadmap

| Item | Why | Status |
|---|---|---|
| Drop the four `deny.toml` advisory ignores (bincode RUSTSEC-2025-0141, paste RUSTSEC-2024-0436, proc-macro-error2 RUSTSEC-2026-0173, unsafe-libyaml RUSTSEC-2024-0449) | Unmaintained crates, transitive through wasmer/wasmtime; ignored 2026-10-25 | Waiting on upstream; re-check at each dependency refresh |
| ~~Fix `docs/guides/security-config.md` to the real capability API ([#8](https://github.com/ciresnave/wasm-sandbox/issues/8))~~ | Guide documented an API that never existed | Done in this PR (guide rewritten, `examples/security_config.rs`) |
| ~~Tested sync-plugin + host `.await` example ([#7](https://github.com/ciresnave/wasm-sandbox/issues/7))~~ | Guest-side async is unsupported; working pattern was undocumented | Done in this PR (`examples/sync_plugin_call.rs`) |
| ~~Network capability enforcement + Wasmer loud-reject ([#11](https://github.com/ciresnave/wasm-sandbox/issues/11))~~ | Network is enforced for components; core modules and Wasmer reject unenforceable grants | Done in 0.7.0 (this PR); patent re-sweep for the allowlist design before publish |
| WASI for the Wasmer backend (apply env/filesystem/network grants) | Wasmer wires up no WASI today and rejects every grant | Not scheduled |
| Wasmer on Windows | wasmer 7 has no native compiler backend there (Cranelift/Singlepass refuse to build); `wasmer-runtime` is non-Windows only | Revisit when upstream adds one |
