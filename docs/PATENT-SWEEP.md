# Patent sweep: wasm-sandbox 0.5.0 @ e2aba5e

Date: 2026-10-25; Run by: wasm-sandbox lane, Claude Sonnet 5.5; Method: AI-assisted search and read; **NOT legal advice; NOT a freedom-to-operate opinion.**
Covers the version line **0.5.x** (swept at `origin/main` e2aba5e, the commit that became 0.5.0). The 0.6.x line is covered by the first addendum (swept at 1173e3e, the head of the 0.6.0 PR) and the 0.7.x network design by the second (swept at 6bb48e3, the 0.7.0 merge). Process: `C:\Projects\PATENT-SWEEP-PROCESS.md`.

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

---

# Addendum: 0.6.0 filesystem-enforcement design

Date: 2026-10-25; Run by: wasm-sandbox lane, Claude Sonnet 5.5; Method and caveats as above (AI-assisted; **NOT legal advice; NOT a freedom-to-operate opinion**). Swept at 1173e3e (head of the 0.6.0 PR, #14); required by the PM before the 0.6.0 publish. Hits 1-13 above stand; this adds the new mechanisms.

## Ideas swept (new in 0.6.0)
10. Mapping a capability struct's `readable_dirs` / `writable_dirs` onto WASI preopened directory handles (read-only or read-write) so the guest sees exactly those directories and cannot leave them.
11. Rejecting, at sandbox-instance creation, a requested capability the runtime cannot enforce (an error, not a silent no-op), including combinations the underlying runtime cannot express (writable without create/delete, file-size cap).
12. Running blocking WASI-using instance creation and guest calls on a short-lived scoped thread when invoked from an asynchronous runtime thread, to avoid a nested-runtime failure.

## Queries (source, query, date)
All 2026-10-25, web search restricted to patents.google.com, claims then read with `tools/patent-claims.py`.
- Q11 "patent mapping granted directories to preopened read-only or read-write directory handles for WebAssembly guest WASI filesystem sandbox" (idea 10)
- Q12 "patent rejecting sandbox instance creation when requested permission cannot be enforced by runtime, fail closed configuration validation" (idea 11)
- Q13 "patent executing blocking runtime call on separate thread when invoked from asynchronous event loop thread to avoid nested runtime deadlock" (idea 12)

## Hits
| # | patent | title | assignee | status | claim(s) read | overlap | our difference | verdict |
|---|---|---|---|---|---|---|---|---|
| 14 | US9626507B2 | Hosted application sandboxing | Google | Active (GP) | 1, 12 (opening ~1,400 chars of each independent claim) | Per-directory read/write permission for sandboxed workers (from the search snippet) | The claims read are a global-level manager, cluster-level managers and process-level managers distributing jobs from work orders referencing application files; we are an in-process library with no work orders, managers or cluster | clear |
| 15 | US6792537B1 | Mechanism for determining restrictions to impose on an implementation of a service | Sun / Oracle | Expired - Lifetime (GP, priority 1999) | 1, 15, 29 (opening of each) | Derives restrictions from the permissions granted to an application; falls back to default limitations when none are granted (nearest to idea 11) | Claim 1 derives restrictions on an **implementation of a service an application requests**, from granted permissions, with default limits merged from policies; we map capability grants to directory preopens and **refuse** grants we cannot enforce, with no service-implementation negotiation. Expired per Google (not the official record) | clear |
| 16 | US8756604B2 | Async wrapper handling execution of asynchronous operations for synchronous and asynchronous routines | Microsoft | Expired - Fee Related (GP) | 1, 7, 13 (opening of each) | Handles a sync routine calling async work by running it elsewhere and blocking for completion (idea 12) | Claims require an async wrapper that **sets a state flag distinguishing a request from an async routine vs. from a sync-over-async wrapper** and marshals results accordingly; we only test whether a runtime is current and, if so, run the call on a scoped thread, with no flag and no marshaling. Expired per Google | clear |
| 17 | US9197417B2 | Hosted application sandbox model | Microsoft | Active (GP) | 1, 19, 20 | Hosted application instances | Claims allocate per-instance **subdomains mapped through a routing-table rule** and serve a UI through the selected subdomain; unrelated to this crate | clear |
| 18 | US8280908B2 | Merging file system directories | Microsoft | Expired - Fee Related (GP) | 1, 7 (opening of each) | Per-silo views of file system directories, some read-only/read-write (from the search snippet) | Claims require an **operating-system silo with a filter driver** that resolves names across silo-specific directory views; we preopen host directories into a WASI guest with no OS-level silo or filter driver | clear |

Not read (background from snippets only): US20080005133A1 (same family as hit 18), US9524396B2 (per-client directories on a network filesystem), WO2016131830A1, US8799862B2, US8719215B2, US20070261124A1, US8060878B2, US6012081A. None names WASI, preopens or Wasm in the snippets returned.

## Needs review (to the PM)
None: no hit overlaps on an independent claim read, and none is `unread` among the numbered hits.

## Limitations
Same as the main record. In addition: hits 14-16 and 18 were read only in the opening of each independent claim (about 1,400 characters), not the whole claim set; three queries only for three ideas; Google Patents returned HTTP 503 to the claims tool for several minutes during this sweep (rate limiting), so the claim reads were retried after a pause. The Google "status" values are not the official record (expiry dates not verified).

---

# Addendum 2: 0.7.0 network-allowlist design

Date: 2026-10-25; Run by: wasm-sandbox lane, Claude Sonnet 5.5; Method and caveats as above (AI-assisted; **NOT legal advice; NOT a freedom-to-operate opinion**). Swept at 6bb48e3 (`main` after #16 merged, the code of 0.7.0); required by the PM before the 0.7.0 publish. Hits 1-18 above stand.

## Ideas swept (new in 0.7.0)
13. A per-address socket policy for a sandboxed guest, default deny, derived from a capability value (none / loopback / port ranges / named hosts / full) and applied by a host-supplied check on every address the guest uses, with name lookups disabled unless unrestricted.
14. Resolving the allowed host names **once**, when the sandbox instance is created, into a fixed IP allow-list (a later change in the DNS answer is denied, never allowed).
15. Comparing addresses after normalising an IPv4-mapped IPv6 form to the IPv4 address, so a rule written for one form covers the other and cannot be bypassed with the other.
16. Classifying each use of an address as remote-peer, bind or listen, permitting the OS's implicit wildcard bind that precedes an outgoing connection while denying explicit binds and listening unless granted.

## Queries (source, query, date)
All 2026-10-25, web search restricted to patents.google.com, claims then read with `tools/patent-claims.py` (Google Patents returned 503 for about 40 minutes; retried every 5 minutes until all four fetched).
- Q14 "patent sandbox network access control resolving allowed hostnames to IP address allowlist at sandbox creation, deny connections to addresses not in list" (ideas 13, 14)
- Q15 "patent normalizing IPv4-mapped IPv6 address to IPv4 before matching against firewall rule or access control list" (idea 15)
- Q16 "patent WebAssembly guest socket address callback permit or deny connect bind listen per address policy host-supplied check" (ideas 13, 16)

## Hits
| # | patent | title | assignee | status | claim(s) read | overlap | our difference | verdict |
|---|---|---|---|---|---|---|---|---|
| 19 | US20200366648A1 | Configuring hostname based firewall policies | ColorTokens | Active (GP; this is the published application US 2020/0366648 A1, Nov. 19, 2020; whether a patent was granted from it was not checked) | **1, 8 and 15 (all three independent claims) read in full**, from the official USPTO publication (rendered pages 10-12 of the PDF, read as images; the machine text from `patent-claims.py` was compared and matches) | Hostname-based access rules turned into IP rules (ideas 13, 14) | Every independent claim requires **all** of: a hostname-based policy added to a **host-based firewall** and "non-implementable ... until said computing device generates a DNS query"; **intercepting the DNS query**; passing it to a DNS server and **intercepting the DNS response**; deriving the hostname-to-IP mapping **from the intercepted response**; and **transforming the firewall policy by replacing the hostname with that IP**. We use no firewall, intercept no DNS query or response, and resolve each allowed name once with the system resolver when the instance is created; the dependent claims (TTL cache, periodic re-verification, regex hostnames, source IP) add elements we also lack | clear |
| 20 | US7739401B2 | Restricting communication of selected processes to a set of specific network addresses | (individual) / Longhorn HD | Expired - Fee Related (GP) | 1, 2 (first two independent claims; 6 claim blocks) | A set of network addresses tied to a process | Claims associate a process with **multiple addresses with a separate communication channel per address** and allow reception of an incoming request on the channel that is ready; we have a per-address allow/deny check, no per-address channels, no reception handling. Expired per Google | clear |
| 21 | US8782739B2 | Access list key compression | Cisco | Expired - Fee Related (GP) | 1, 50 | Access lists keyed on IP addresses, with special handling of IPv4-compatible / IPv4-mapped IPv6 (idea 15) | Claims **compose a compressed access-list key** by replacing a subset of address bits with a **Zone_ID** (claim 1) or by protocol-dependent key formats (claim 50) and route packets with it; we convert a mapped address to its IPv4 equivalent for comparison, with no key compression, zone identifiers or routing. Expired per Google | clear |
| 22 | US20220286431A1 | DNS resolver-assisted domain name-based access control | Cisco | Active (GP) | 1, 7 | Domain-name-based access control (ideas 13, 14) | Claims are a **DNS resolver that sends the resolved IP, with the client's identity, to a separate firewall device** (claim 1) which builds a domain-name registry entry and matches packets (claim 7); we have no resolver, firewall device or client identification | clear |

Not read (background from snippets only): US7623518B2, US8719450B2, US6985485B2/US6920136B2, US8582599B2, EP3503505B1/US20190199616A1, US20060080444A1, US9225684B2, US20030061510A1, TWI400920B, US20080222292A1. None names WASI, a sandboxed guest or the mechanisms above in the snippets returned.

## Needs review (to the PM)
None: no hit overlaps on an independent claim read, and none of the numbered hits is `unread`.

## Limitations
Same as the main record. In addition: hit 19 was read in full (all independent claims) from the official publication; hits 20-22 were read in the first independent claims only; three queries for four ideas (idea 16 has no dedicated query beyond Q16); Google "status" values are not the official record and expiry dates were not verified.

---

# Addendum 3: 0.7.4 confining filesystem, and the planned Wasmer network design

Date: 2026-10-28; Run by: wasm-sandbox lane, Claude Sonnet 5.5; Method and caveats as above (AI-assisted; **NOT legal advice; NOT a freedom-to-operate opinion**). Swept at 288eb26 (`main` after #21, the code of 0.7.4) for idea 20, and against `docs/design/wasmer-network.md` (a plan, no code yet) for ideas 17-19. Idea 20 shipped in 0.7.4 without its own sweep; it is recorded here. Required by the PM before any Wasmer-network publish. Hits 1-22 above stand.

## Ideas swept (new since addendum 2)
17. A decorator over a host virtual-network provider: every socket the guest creates or uses is obtained through a wrapper that applies the address policy of idea 13 on each call (connect to a peer, bind, listen, per-datagram send and receive) before delegating to the real network.
18. Applying the policy to **incoming** traffic too: closing an accepted connection whose peer address is not allowed, and dropping a received datagram whose source address is not allowed.
19. Withholding, regardless of the grant, the sockets and operations outside plain TCP/UDP: raw and ICMP sockets, interface, address, route and DHCP configuration, multicast and broadcast (unless the grant is unrestricted).
20. (0.7.4, shipped) A confining filesystem decorator over a host directory for a sandboxed guest: it refuses every mutation when the grant is read-only, and refuses any path whose canonical real location is outside the granted directory (so a symlink cannot lead out).

## Queries (source, query, date)
All 2026-10-28, web search restricted to patents.google.com; claims then read with `tools/patent-claims.py` (Google Patents returned 503 for about 30 minutes; retried every 5 minutes until all four fetched).
- Q17 "patent virtual network wrapper decorator sandbox socket connect bind check destination address allow list each call guest virtual machine" (ideas 17, 13)
- Q18 "patent filtering incoming datagrams and accepted connections by source address allow list sandboxed application host network proxy" (idea 18)
- Q19 "patent sandbox file system access path resolved symbolic link real path outside permitted directory deny, read-only mount enforcement" (idea 20)
- Q20 "patent sandbox deny raw socket ICMP network interface configuration routing DHCP guest, provide only TCP UDP sockets restricted" (idea 19)

## Hits
| # | patent | title | assignee | status | claim(s) read | overlap | our difference | verdict |
|---|---|---|---|---|---|---|---|---|
| 23 | US7216225B2 | Filtered application-to-application communication | Voltaire / Mellanox | Expired - Lifetime (GP; estimate 2023-12-02) | 1, 8, 15, 18 (all independent claims) | A filter applying a policy to a process's communication (ideas 17, 18) | Every independent claim requires a communication interface that **bypasses a portion of the operating-system kernel** and hands filtered data **directly to communication hardware** (multi-channel NIC, InfiniBand in dependents); we filter addresses in user space and delegate to the ordinary kernel network stack. Expired per Google | clear |
| 24 | US20160191413A1 | Introspection method and apparatus for network access filtering | Nicira / VMware | Active (GP; published application, grant not checked; estimate 2036) | 1 and 13 (both independent claims) | Allow/deny of a guest's network access by policy, decided on the host (ideas 17, 18) | Both independent claims require a **guest introspector installed on the data compute node (a virtual machine) that captures data about the access attempt**, and policies stored on the host examined with that captured data; the dependent claims add filters in the guest network stack, website and file identification. We install nothing in the guest, capture no guest data, and decide from the socket address alone in the host process that serves the guest's calls | clear |
| 25 | US6003084A | Secure network proxy for connecting entities | Secure Computing / McAfee | Expired - Lifetime (GP) | 1, 9, 10, 16, 22 (all independent claims) | Checking a requester's address against a list of authorized addresses (dependent 11, 17) | Every independent claim requires a **session manager/proxy that establishes two independent connections** (to the requester and to the responder), **simultaneously at a plurality of protocol layers**, with a relay between them; the address check appears only as a dependent step. We terminate no connection, relay nothing and keep one connection per socket. Expired per Google | clear |
| 26 | US8930346B2 | Symbolic-link identifying | IBM | Expired - Fee Related (GP) | 1 (the only independent claim) | Detecting a symbolic link in a path to keep access inside an export (idea 20) | Claim 1 is a **network file server** that determines an absolute path with no symlinks and **traverses it with `openat()` on each component's file descriptor**, with a parameter that makes the open report a symlink, to find one inserted *after* the path was determined. We resolve the real path once with the standard library (`canonicalize`) and compare it with the granted directory; no network file server, no per-component descriptor traversal, no `openat`. Expired per Google | clear |
| 27 | US20130340050A1 | Zero configuration communications between a sandboxed program and a networked service | Samba TV (original assignee: an individual) | Active (GP; published application, Dec. 19, 2013; whether a patent was granted from it was not checked; estimate 2032) | **1 and 22 (both independent claims) read in full from the official USPTO publication** (rendered pages 17-19 of the PDF, read as images); the machine text from `patent-claims.py` was compared and matches | A program in a security sandbox communicating with a network service (ideas 13, 17) | Both independent claims require **a networked device that announces a networked service to a discovery service, or performs the discovery service for a private network, and a client device that automatically instantiates a connection between the sandboxed program and that device or service**; the dependents add discovery agents, GUIDs, relays, NAT traversal and access tokens. We have no discovery service, no announcing, no automatic connection and no relay: the sandbox only decides, per socket address, whether a connection the guest itself initiates may proceed | clear |

Not read (background from snippets only): US20210397467A1 (socket impersonation on VMs), US11113086B1, US11057385B2, US10608881B2, US11336619B2, US11374906B2 (sandbox behind a web proxy), US5802320A, US5915087A, US8363650B2, US6591306B1, US9542403B2 and US20120203927A1 (same family as hit 26, so likely similar claims), US7809779B2, US6321219B1, US7197516B1. None names WASI, a WebAssembly guest or the decorator/policy mechanisms above in the snippets returned.

## Needs review (to the PM)
None of hits 23-27 overlaps on an independent claim read. The "not read" list above is background only. Hit 27 (the nearest by subject) was added at the PM's request on 2026-10-28 and read from the official publication.

## Limitations
Same as the main record. In addition: four queries for four ideas; each hit's independent claims were read in full (hits 23-26 from Google Patents' machine-converted text, hit 27 from the official publication), but the "not read" patents were not; Google "status" values are not the official record and expiry dates were not verified; the machine-converted claim text of hits 23-26 was not compared with the official publication; idea 20 was swept after it shipped.
