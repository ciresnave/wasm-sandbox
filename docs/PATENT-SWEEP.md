# Patent sweep: wasm-sandbox 0.5.0 @ e2aba5e

Date: 2026-10-25; Run by: wasm-sandbox lane, Claude Sonnet 5.5; Method: AI-assisted search and read; **NOT legal advice; NOT a freedom-to-operate opinion.**
Covers the version line **0.5.x** (swept at `origin/main` e2aba5e, the commit that became 0.5.0). Process: `C:\Projects\PATENT-SWEEP-PROCESS.md`.

## Ideas swept
1. A one-call API that compiles source code to WebAssembly, instantiates it in a sandbox and invokes a named exported function with typed (serialized) parameters and result.
2. A deny-by-default capability set (network, filesystem, environment variables, process, time, randomness) that is translated into the WASI context given to each sandboxed instance, including per-instance environment allow/deny lists.
3. Per-instance resource limits (instruction "fuel", memory, CPU, time, I/O) enforced by the host runtime, with usage monitoring.
4. A runtime-abstraction trait so one sandbox API runs on more than one WebAssembly engine (wasmtime, wasmer), selected by build feature.
5. Host-to-guest function calls whose parameters and results are serialized as JSON or MessagePack, plus shared-linear-memory channels.
6. Templates/wrappers that expose a sandboxed module as an HTTP server, a command-line tool, or an MCP server.
7. Audit/event logging of capability use and violations, and runtime metrics, for sandboxed instances.
8. Keeping a thread-confined (non-`Send`) engine store on one dedicated owner thread and exposing a `Send + Sync` channel handle for calling into it.
9. Reusable (resettable) sandboxes and a progressive-complexity API (one-liner, builder, trait level).

## Queries (source, query, date)
All 2026-10-25, web search restricted to patents.google.com unless noted; claims then read with `tools/patent-claims.py`.
- Q1 "patent compiling untrusted source code to WebAssembly and executing in sandbox with capability-based permissions" (ideas 1, 2)
- Q2 "patent WebAssembly runtime resource limits instruction metering fuel memory limit sandbox host function calls" (3)
- Q3 "patent abstraction layer multiple WebAssembly runtimes engines selectable common interface instance module" (4)
- Q4 "patent host guest communication shared linear memory WebAssembly serialization JSON MessagePack function call plugin" (5)
- Q5 "patent deny-by-default capability set network filesystem environment variables WebAssembly WASI sandbox plugin" (2)
- Q6 "patent generating wrapper exposing sandboxed module as HTTP server command line tool or model context protocol server" (6)
- Q7 "patent receiving source code, compiling to WebAssembly module, instantiating in sandbox with limits, invoking exported function and returning result" (1)
- Q8 "patent logging capability violations audit events sandboxed plugin permission denied monitoring metrics module instance WebAssembly" (7)
- Q9 "patent WebAssembly instance thread-confined store message channel proxy handle non-thread-safe runtime object owner thread" (8)
- Q10 (unrestricted web) title search for US 12683941 to find a readable copy.
Ideas 9 got no dedicated query (generic API ergonomics, no mechanism); not swept beyond Q1/Q7.

## Hits
Status is Google Patents' own (not the official record). "Claims read" = independent claims read in full via `patent-claims.py` unless stated.

| # | patent | title | assignee | status | claim(s) read | overlap | our difference | verdict |
|---|---|---|---|---|---|---|---|---|
| 1 | US8272048B2 | Restriction of program process capabilities | Apple | Expired - Fee Related (GP) | 1 | Per-program security profile decides whether an operation is allowed | Claim 1 requires monitoring in the **operating-system kernel** at the point kernel-operation arguments are resolved; we configure a WASI context in a user-space library and never intercept kernel operations | clear |
| 2 | US20130160115A1 | Sandboxing for multi-tenancy | Microsoft | Active (GP), application | 1, 13, 17 (all independent) | Isolates applications and controls/detects resource overuse | All claims require a **worker process manager spawning worker processes on a server**, one application per process; we are an in-process library that instantiates Wasm modules, with no server and no worker processes | clear |
| 3 | US8005950B1 | Application server scalability through runtime restrictions enforcement | Google | Active (GP) | 1 (opening 1,100 chars of the independent claim; the remainder was not read) | Terminates an application that breaks time or memory limits | Claim 1 is a multi-server application execution system (stored apps, request/response handling, servers); we are a library with trap-on-exhaustion fuel/limits and no request-serving fleet | clear (see Limitations: claim read in part) |
| 4 | US20190207867A1 | Platform with multiple execution engines | GE | Active (GP), application | 1, 9, 17 | Picks one of several execution back-ends behind a common interface | Claims select among **cloud services platforms** and fetch platform-specific API info to launch on that platform; we pick a local Wasm engine at compile time via a cargo feature | clear |
| 5 | US11392357B2 | Delegating bytecode runtime compilation to serverless environment | SAP | Active (GP) | 1 (opening of claim set) | Runs WASM modules in a sandbox | Claim 1 is a host sending bytecode plus a directed acyclic graph to a WASM module that compiles it; we compile source with the system toolchain and run the result | clear |
| 6 | US11301562B2 | Function execution based on data locality | SAP | Active (GP) | 1 (opening of claim set) | WASM runtime executing in a sandbox | Requires an orchestration layer selecting a VM by data store location and read/write policy; we have none | clear |
| 7 | US8825748B2 | Sandboxed daemon process invocation through HTTP | Sharp | Active (GP) | 1 (opening of claim set) | Applications run under a rule set in a restricted environment, started by HTTP | Claim 1 assigns a URL per initiated application and runs it in the background **without returning a response**; our HTTP wrapper template serves requests and returns responses | clear |
| 8 | US20210089348A1 | Multi-tenant actor systems with web assembly | SAP | Active (GP), application | 1, 8, 15 | A thread selected for a Wasm module in a sandbox (nearest to our owner-thread design, idea 8) | Claims require a **dispatcher platform, a tenant identifier, a thread chosen from a pool, and a tenant actor created in the sandbox**; we spawn one dedicated thread per wasmer instance with no dispatcher, pool or tenant identity (the security guide's "Multi-Tenant Security" section is a usage pattern, not an implemented dispatcher) | clear (re-sweep if a pool/tenant dispatcher is ever added) |
| 9 | US20200026865A1 | Application security policy management agent | Cisco | Active (GP), application | 1, 8, 15 | Permission requests approved/denied against a policy | Claims require running in a development environment, a custom security manager, and **updating the policy to approve the denied permission** (policy learning); we do static policy only | clear |
| 10 | US9785489B1 | Secure script execution using sandboxed environments | Amazon | Active (GP) | 1, 5, 13 | Sandboxed thread/isolated process runs a script | Claims centre on a **user interface element that modifies the script** and a scripting service round-trip; we have neither | clear |
| 11 | US20250028585A1 | Detecting webassembly-based browser exploits | Palo Alto Networks | Pending (GP) | 1, 9, 15 | Concerns Wasm instances and exported functions | Detects exploits by **hooking a browser engine API and comparing RWX memory contents**; we do not detect exploits | clear |
| 12 | US 12683941 B1 | Secure tunnel access to locally executed tools for AI agents | Airia LLC | Granted 2026-07-14 (face of the patent); 20 claims | 1, 19, 20 (all independent), read off rendered page images of the USPTO PDF (pp. 40-42 of the document), not machine text | Starts an MCP server inside an isolated "nano sandbox" at the remote device (nearest to our MCP wrapper template, idea 6) | Every independent claim requires: downloading a **cloud connector installer from an AI platform**, authenticating the connector to a **connection server**, a **secure tunnel opened without a firewall port**, an **SSE channel** initialised by that server, a nano sandbox (isolated execution environment in a kernel) instantiated from a definition in the request, and a **session identifier** exchanged with the connection server. This crate has no connector, connection server, tunnel, SSE relay or session-identifier protocol; the MCP wrapper template only generates a module that speaks MCP over the host's own transport | clear |
| 13 | US 12592823 B1 | Secure tunnel access to remote client resources for AI agents | (not recorded; same family as hit 12) | Granted (number form B1; date not recorded) | 1, 19, 20 (all independent), read off rendered page images of the USPTO PDF | Secure tunnel from a connector to a platform; AI agent access to client resources | Every independent claim requires a **UI selection creating a managed cloud connector object with a key**, a connector authenticating to a connection manager on an AI-platform server, a **keyed encrypted tunnel opened with no firewall ports**, and **model guardrails detecting prompt injection and PII** on inputs and outputs of an AI model at the remote device; this crate has none of these | clear |

Not read first-hand and therefore not counted: CN110489942B (encrypted wasm loading), US6842906B1 and US9507648 (proxy/handle patterns, general), US8997225B2/US6275938B1 (native-code sandboxes), US20040237064A1/US7418718B2 (.NET runtime hosting). They surfaced in searches as background only; none mentions the mechanisms above in the snippets returned.

## Needs review (to the PM)
None. No hit is `needs-review` and none is `unread`. (US 12683941 and its family member US 12592823 were first unreadable: the Google Patents number form 404ed in the tool and the USPTO copies are scanned images. Both were then read from rendered page images of the USPTO PDFs; see hits 12 and 13.)

## Limitations
- AI-assisted search with a general web search engine, restricted to Google Patents for most queries; ten queries, dated above. Not a systematic classification-code search (CPC G06F 21/53, G06F 9/455) and no USPTO/EPO/WIPO native search; coverage is partial and results are search-engine ranked.
- Claim text is Google Patents' machine conversion; "status" is Google's, not the official record. No assignee, family, continuation or maintenance-fee check was done.
- Hits 3 to 7: only the opening of the independent claim (about 1,100 characters) was read for the verdict, not the whole claim set; dependent claims were not analysed.
- Idea 9 was not searched as a separate mechanism.
- Not legal advice and not a freedom-to-operate opinion. Re-sweep when a new family of capability is added and at least yearly.
