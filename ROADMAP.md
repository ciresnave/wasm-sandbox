# Roadmap

| Item | Why | Status |
|---|---|---|
| Drop the four `deny.toml` advisory ignores (bincode RUSTSEC-2025-0141, paste RUSTSEC-2024-0436, proc-macro-error2 RUSTSEC-2026-0173, unsafe-libyaml RUSTSEC-2024-0449) | Unmaintained crates, transitive through wasmer/wasmtime; ignored 2026-10-25 | Waiting on upstream; re-check at each dependency refresh |
| Fix `docs/guides/security-config.md` to the real capability API ([#8](https://github.com/ciresnave/wasm-sandbox/issues/8)) | Guide documents `NetworkCapability::Connect/Listen`, `FilesystemCapability::ReadOnly(..)` etc., which do not exist | Decide by 2026-11-08 |
| Tested sync-plugin + host `.await` example ([#7](https://github.com/ciresnave/wasm-sandbox/issues/7)) | Guest-side async is unsupported; the working pattern is undocumented | Decide by 2026-11-08 |
| Wasmer on Windows | wasmer 7 has no native compiler backend there (Cranelift/Singlepass refuse to build); `wasmer-runtime` is non-Windows only | Revisit when upstream adds one |
