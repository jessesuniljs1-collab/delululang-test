# Sandbox architecture — the execution-isolation layer under Delulu Authority

**Written:** 2026-09-17. **Status:** proposed design, integrated into `MASTER_PLAN.md` §12 and
`IMPLEMENTATION_ROADMAP.md` (phases PS-A…PS-E). **Nothing here is built.**
**Constraints honoured:** Authority and the Guard are not redefined; the sandbox is an enforcement
layer *beneath* them and invents no permission model of its own; no profile ever silently
downgrades (`DL1408` semantics are kept); the machine channel stays first-class.

---

## 1. The decision in one page

**Sandboxing becomes a cross-cutting architecture introduced early, with the strongest backend
gated on hosts.** Concretely:

1. **The guest holds nothing.** A sandboxed program runs in a guest (a process, a microVM, or an
   external environment) that owns **no capability material, no filesystem view beyond its own
   code, no network stack, and no broker address**. Every effectful primitive the interpreter
   reaches is forwarded over one **effect channel** to the host `delulu` process, which holds the
   capabilities, runs the custody checks, the Guard, the containment resolver and the audit log
   exactly as it does today, performs the effect, and returns the result. This generalizes three
   things the project already does: the WASM host performs effects for a guest that only holds
   opaque handles; the foreign worker is the server on a private pipe and is never handed the
   broker; Stage 5 §6 gives the microVM guest a vsock proxy holding one node (invariant 25).
2. **Policy is derived from authority, never declared beside it.** `delulu authority` already
   computes what the program may do; the sandbox policy is a pure function of that report, the
   operator's grant, and a *profile* that selects the backend and resource limits. There is no
   sandbox allowlist to get out of sync with the authority — the OS policy for the guest is
   always "nothing but the channel", and the scopes live where they live today, host-side.
3. **A provider is a launcher plus a channel, not an enum of policies.** The backend abstraction
   is: *launch a guest runtime with these bytes, connect the effect channel, enforce these
   resource limits, report lifecycle events, kill on demand*. Process, microVM and external
   providers implement that; none of them interprets authority.
4. **Levels are honest labels for boundaries that exist**, probed at run time and reported in
   `--json` and `doctor`; a profile that needs a boundary the host lacks is refused, never
   approximated.
5. **The microVM stays gated on Linux + KVM as an implementation**, but its design changes: the
   guest runs the **interpreter** (the whole language), not the WASM fragment; it has **no
   filesystem device and no network device** — only vsock for the effect channel; the program and
   its authority arrive over the channel; the image is a pinned kernel plus a static `delulu`
   guest runtime as `/init`. That removes `virtiofsd`, the path-resolution class, DNS and metadata
   from the guest's reach by construction.

The rest of this document specifies the layers, the provider seam, the UX and machine interface,
the microVM, the integrations, and what must not be built yet.

## 2. The layers, top to bottom

```
  program text ──► delulu check / authority  (static: rows, kinds, requested scopes)
                        │
  operator grant ───────┼──► GRANT  (⊑ manifest ceiling; today's --grant / lease)
                        │
  profile ──────────────┼──► SANDBOX POLICY = derive(authority, grant, profile)
                        │        backend level, resource limits, ttl, channel rules,
                        │        what the guest may never do (everything not brokered)
                        ▼
  HOST LAUNCHER (provider) — spawns the guest, applies OS/VMM confinement, connects the channel
                        │
  GUEST RUNTIME — the interpreter in "brokered" mode: capabilities are opaque handles,
                  every effect is a channel request; no OS effects; no broker address
                        │  effect channel (framed, versioned, bounded)
                        ▼
  HOST EFFECT PROXY — custody (embedded or daemon), Guard permits, containment resolver,
                      host allowlist, secret store, device envelopes, audit — the existing code
                        │
  OS / device / adapter — where the effect actually happens, as today
                        │
  AUDIT + TELEMETRY — lifecycle (launch, limit, kill, death), every brokered effect, verdicts
```

Read the arrows as *attenuation*: nothing below can widen what was decided above. The sandbox
adds four things and only four: **denial of everything not brokered** (an OS or hardware
boundary), **resource limits**, **identity separation** (a different uid, token or VM — the
category-7 answer `ROOT_ISSUANCE_TRUST_BOUNDARY.md` §2(a) names), and **lifecycle audit**.

### 2.1 Why the guest performs no effects (the load-bearing choice)

Alternative A — the guest holds the capabilities and performs effects, and the OS policy is
translated from the scopes (Landlock paths, proxy allowlists, virtio-fs mounts). Alternative B —
the guest holds nothing and the host performs every effect over a channel.

| | A: guest performs | B: host performs (chosen) |
|---|---|---|
| OS policy | must mirror scopes per backend: Landlock path rules, proxy domain lists, mount lists — three encodings of one authority, each a place for a spelling defect (the 2026-08-10 search key) | one policy for every backend: *nothing but the channel*; scopes enforced once, by the code that already does |
| Network | the guest needs a NIC or proxy; DNS and metadata must be blocked separately (AgentCore) | the guest has no network at all; `http.get` is a channel request the host performs under the existing host check — and since the runtime has **no network client today** (NE-17), the host-side egress proxy of PS-B-02 will be the *first* one, built with name allowlisting, resolved-address pinning and special-use refusal from its first line |
| Files | a shared filesystem in the guest (virtio-fs, bind mounts) — the C84/SYMLINK-DANGLE class inside the boundary | no shared filesystem; reads and writes stream over the channel, resolved by `contains_on_disk` host-side |
| Secrets | plaintext may need to enter the guest for `verify`/`map` | secret bytes stay host-side; `verify` is brokered; `map` is the one tension (§8.3) |
| Revocation | the guest can cache a capability | the host re-checks every request; revocation is host-side as today |
| Broker | the guest must reach the broker (or a proxy) | the guest never learns a broker address; only the host talks to the broker — invariant 25 strengthened to "the guest holds no node at all" |
| Cost | none per effect | one round trip per synchronous-class effect; epoch-class ops (reads, clock, rand) can be batched or served from a host-supplied snapshot (deterministic under `--seed`/`--clock`) |
| Foreign code | must run in the guest, needing the library and its syscalls | runs in a foreign worker *of the host* as today (blast radius already bounded), or in a guest that hosts the library — profile-selectable |

B is chosen. It is the only option where adding a backend cannot weaken authority, because no
backend ever re-encodes authority. Its cost — a round trip per effect — is measured before the
first profile ships (test plan §7), and it is the cost every WASM-host and enclave design already
pays.

## 3. The provider model

Six things are kept apart, each with one owner:

| Concern | Owner | What it is |
|---|---|---|
| **Authority** | `delulu-check` + broker | what the program may do (unchanged) |
| **Policy** | new module in the CLI, pure function | `derive(authority, grant, profile) → SandboxPolicy { level, limits, ttl, channel_rules, secrets_rule }` — deterministic, byte-stable JSON, part of the run envelope |
| **Host capability** | probe module | what this host can enforce: user namespaces, Landlock ABI, seccomp, cgroup v2 write access, KVM, a VMM binary and image, Job Object, restricted tokens, AppContainer, Seatbelt — reported by `doctor` and by the refusal message |
| **Backend (provider)** | trait `Launcher` | `spawn(guest_image, program_bytes, policy) → Guest { channel, pid_or_vm, limits }`, `kill()`, `wait()`, `evidence()`; implementations: `process` (Linux/Windows/macOS), `microvm` (Linux+KVM), `external` (operator command / remote) |
| **Runtime** | `delulu-runtime` guest mode | the interpreter with an `EffectSink` seam: primitives call the sink; the in-process sink is today's code; the channel sink serializes the request |
| **Audit** | broker audit log + run envelope | lifecycle records (`sandbox-launch`, `sandbox-limit`, `sandbox-kill`, `sandbox-death`, `channel-violation`), every brokered effect as today |

**The seam that must survive future providers is the channel protocol**, not the launcher:
`delulu-sandbox-channel/1`, length-prefixed canonical CBOR frames (the existing `broker_ipc`
framing, bounded per frame — ADAPTER-LINE-1's lesson), request kinds mirroring the primitive
table (`read_text`, `append_text`, `list_dir`, `get`, `now_ms`, `rand_int`, `println`, `readline`,
`secret_verify`, `actuator_command`, `sensor_read`, `compute_dispatch`, `plugin_load` …), each
carrying the opaque capability handle, the arguments, and the guest's span for the audit record.
A backend is any transport that carries those frames with the required properties: ordered,
bounded, authenticated by construction (a private pipe, a vsock port, or a TLS session bound to a
lease). A future Kubernetes, remote or confidential provider adds a launcher and a transport and
changes nothing in policy, runtime or audit.

**Refusal is the default in every seam:** a request kind the host does not know is a
`channel-violation` (kill the guest, audit, DL); a frame over the bound is a violation; a handle
the guest was never given is a violation; the provider reporting "launched" without the channel
connecting within the deadline is a launch failure, never a fallback to in-process.

## 4. Isolation levels — evaluated, not assumed

| Level | Mechanism | Security boundary | Attack surface | Startup / perf | Platform | Protects against | Does NOT protect against | Host needs | Untrusted code | Physical AI | Multi-tenant |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **L0 `none`** | in-process (today) | the language + custody | the whole `delulu` process | 0 | all | authority overreach by *well-typed* code | compiler/runtime bugs, foreign code, same-user reach | none | no | dev/sim only | no |
| **L1 `process`** | child guest runtime, channel over a private pipe; Linux: user+mount+pid+net namespaces (bwrap-style, no binary needed via `unshare`/`clone`), read-only rootfs view with only the guest binary, Landlock (deny all fs; net: no NIC), seccomp allowlist (`seccompiler`), rlimits + cgroup v2 where writable, `PR_SET_PDEATHSIG`; Windows: restricted token + Job Object (memory, CPU, process count, kill-on-close) [+ AppContainer where feasible]; macOS: Seatbelt profile denying fs/net except the pipe | the OS kernel's process isolation | host kernel syscalls reachable through the filter; the channel parser | ~5–30 ms (a process spawn plus namespace setup; to be measured) | Linux, Windows, macOS | foreign-code blast radius; accidental/hostile fs and net use by a bug in the runtime; **same-user reach when combined with identity separation** | kernel exploits; side channels; on Windows without a separate account, resources with null security descriptors; on macOS the deprecation risk of Seatbelt | unprivileged namespaces (Linux), a kernel with Landlock (probe), nothing on Windows/macOS | yes, for accidental and moderately hostile code | yes (the adapter stays host-side; the control program is contained) | no (same OS user) unless the launcher uses a distinct account |
| **L2 `microvm`** | Firecracker or Cloud Hypervisor subprocess under the jailer; guest = pinned kernel + static `delulu` guest runtime as `/init`; **vsock only** (no NIC, no block, no virtio-fs); program bytes over the channel; memory/vCPU from the VMM; wall-clock from the host | hardware virtualization (KVM) | the VMM's vsock device and the channel parser; KVM | ≤125 ms boot (Firecracker's number), <5 MiB overhead; snapshot restore later | Linux x86_64/aarch64 + KVM | kernel exploits in the guest; everything L1 does | hypervisor/KVM bugs; side channels (disable SMT for tenants); DoS of host CPU without cgroups | `/dev/kvm`, a VMM binary, the guest image, a jailer uid | yes, hostile | yes | yes, with a uid per VM and the jailer |
| **L3 `external`** | an operator-supplied launcher runs the guest runtime inside *their* environment (Docker+gVisor, Kata, Docker Sandboxes, a cloud sandbox, a k8s pod, ssh) and exposes the channel over stdio or a socket | whatever the operator's environment provides — **reported, not asserted** | the operator's environment | theirs | wherever they run | whatever their boundary does | DeluluLang cannot know; the level is labelled `external` and the guarantees field says `unknown` unless attested (L4) | the launcher command | as strong as the environment | via cloud brokers | theirs |
| **L4 `attested`** (future) | L2/L3 whose guest presents an attestation (Nitro PCRs, SEV-SNP/TDX report) of the pinned image before the host delegates any lease | hardware root of trust | the attestation verifier | + attestation round trip | EC2 Nitro, SNP/TDX hosts | a modified guest image obtaining a lease | everything below the hardware | specific hardware | yes | yes | yes |

**What actually makes sense for DeluluLang:** L0 stays (development, byte-identical default);
L1 is the **minimum useful secure sandbox** and the one that ships on all three OSes; L2 is the
production tier for hostile code on Linux hosts; L3 is how every other isolation technology plugs
in without DeluluLang re-implementing it; L4 is the principled end state and waits for hardware
and evidence. An "application-kernel" (gVisor) tier is **not** built by DeluluLang — it is an L3
launcher (`docker run --runtime=runsc …`), because gVisor is Linux-only, is not a library, and
DeluluLang would add nothing by wrapping it. A "hardened container" tier is likewise L3.

**Profiles** bundle a level with limits and defaults; they are names for policies, not new
permissions: `dev` (= L0, today's behaviour), `contained` (= L1, moderate limits, no prompts),
`hostile-agent` (= strongest available of L2/L1 **with refusal if L2 is requested and absent**,
tight limits, secrets never in guest, audit everything, `--no-prompt`), `external:<name>` (= L3).
A profile can be named on the command line or in `delulu.toml` under a `[sandbox]` table
(`profile`, `limits`, `require_level`), reviewable and diffable like the authority ceiling; the
command line wins; `require_level` refuses a weaker host.

## 5. User experience — designed from the existing CLI

The flag already exists: `delulu run … --isolation none|process|microvm`. It stays, with its
meaning **strengthened** — `process` will mean "the whole program in an L1 guest", which is a
change in behaviour for a flag that today only moves foreign code, so it is versioned and
announced (the current narrow behaviour becomes `--foreign-isolation process`, which already
exists). Additions:

```
delulu run app.delulu --grant … --sandbox                       # = --sandbox-profile contained
delulu run app.delulu --grant … --sandbox-profile hostile-agent  # refuses if the level is absent
delulu run app.delulu --grant … --isolation microvm             # unchanged: DL1408 where absent
delulu run app.delulu --grant … --sandbox-backend external:<cmd> # L3
delulu run app.delulu --grant … --limits cpu_ms=…,mem_mb=…,wall_ms=…,pids=…,disk_mb=…,ttl_ms=…
delulu sandbox probe [--json]      # what this host can enforce, per level, with the reason it cannot
delulu sandbox policy <file> [--profile P] [--json]   # the derived policy, without running
delulu sandbox status [--json]     # live guests: id, level, limits, remaining, ttl, state
delulu sandbox kill <id>           # e-stop for a guest (audit-recorded)
delulu explain E-SANDBOX           # the honesty caveats verbatim, like E-PLUGIN
delulu doctor                      # gains a `sandbox` section (§7)
```

`--json` on `run` gains an additive `sandbox` object (schema stays 1; fields additive):

```json
"sandbox": {
  "backend": "process", "level": 1, "profile": "contained",
  "requested_level": 1, "granted_level": 1,
  "policy": { "channel": "delulu-sandbox-channel/1", "fs": "brokered", "net": "brokered", "secrets": "host-only" },
  "limits": { "cpu_ms": 60000, "mem_mb": 512, "wall_ms": 120000, "pids": 64, "disk_mb": 0 },
  "remaining": { "cpu_ms": 58210, "mem_mb": 497, "wall_ms": 117004 },
  "ttl_ms": 600000,
  "host_guarantees": ["user-namespaces", "landlock-abi-6", "seccomp", "cgroup-v2"],
  "denied": [ { "kind": "read_text", "path": "../outside", "code": "DL0904", "why": "escapes the granted scope" } ],
  "state": "exited", "exit": 0, "guest_death": null,
  "audit": { "launch_seq": 12, "records": 9 }
}
```

An agent can therefore discover, from one envelope: backend, level, filesystem and network
posture (always *brokered*), the tools it may call (the effects in its row), capabilities and
effects (the existing `authority` object), limits and TTL, what was denied and why (the same DL
codes as today, with the guest span), remaining resources, the host's guarantees, and the guest's
state. `delulu sandbox probe --json` answers the same questions *before* running.

**Diagnostics** (numbers to be allocated by a ruling; the DL14xx range is custody's):
`DL1408` keeps its meaning and **gains the promised repair** (`requires_human: true`, the
fallback command); new codes for *sandbox policy refused* (profile needs a level the host lacks —
distinct from 1408's platform case), *resource limit exceeded, guest killed* (never an
authority-widening repair — the plugin `DL1506` attribution rule applies), *guest died*,
*channel violation*, *guest image integrity failure*. Each ships with an accepting and a rejecting
conformance witness like every other code.

## 6. Integration with the rest of the system

- **Authority.** Unchanged. The policy consumes the report; the guest cannot exceed it because
  the host performs every effect under the same checks as today. New static fact: the run
  envelope's `sandbox.policy` is derivable from `authority --json` plus the profile, and a test
  pins that derivation as a pure function.
- **Guard.** Unchanged in meaning: permits are checked at the host proxy; `guard request/approve`
  work for guests exactly as for in-process programs; `sandbox kill <id>` is the e-stop for a
  guest and is audit-recorded like `grants revoke`.
- **Broker and leases.** The guest never holds a lease; the host redeems `--lease` and holds the
  node. Invariant 25 ("a guest sees one node, never the socket") becomes "a guest sees no node";
  the audit record of every brokered effect carries the guest id.
- **Revocation.** Host-side, as today; a revoked node fails the next brokered request; a
  `sandbox kill` follows a root revocation of the guest's node (policy: kill on revoke).
- **Effects and capabilities.** The row is the tool list; capability values in the guest are
  handles; `Cap` opacity is preserved; `narrow` is a host request that returns a narrower handle.
- **Plugins (P2).** A plugin loaded by a sandboxed program is loaded **by the host proxy** on the
  guest's behalf (`plugin_load` request): the Stage 6 load sequence runs host-side; Verified
  exports execute *in the guest* (they are DIR re-checked by the host and shipped over the
  channel as bytes), Contained exports execute *host-side* under the WASM limits and answer
  calls over the channel. Either way the plugin's grant is ⊑ the program's node, so the sandbox
  policy computed for the program already bounds every plugin it can load — **sandboxing needs
  nothing plugin-specific, and plugins need nothing sandbox-specific beyond the channel request.**
- **Actors.** Actor worker threads live in the guest; sends are in-guest; only effects cross.
- **LSP / MCP (P4).** Analysis-only surfaces never launch a guest; `delulu.authority` stays
  effector-free. An MCP `sandbox_probe` read-only tool is allowed; `run` stays CLI-only.
- **Atlas.** Gains an optional `--sandbox [profile]` view relating program → authority → derived
  policy → limits, so a reviewer sees "what this program's sandbox would deny" beside "what the
  program can do"; the `atlas/1` schema grows additively.
- **Survey.** New crate(s) and the guest-runtime module appear automatically; the cross-language
  coupling to the image build scripts must be **named in comments** or the map will not know
  (the `lsp.rs` lesson).
- **Doctor.** A `sandbox` section that reports only what can change a decision (§7).
- **WASM.** Unchanged; the WASM engine remains an in-process engine option; it is *not* the
  guest's engine (the fragment cannot carry the language).
- **Deployment.** `DEPLOYMENT.md`'s Tier 2 gains the mechanical form: an L1 launcher that runs
  the guest under a *different account* where the OS allows an unprivileged process to do so
  (Windows restricted token / AppContainer; Linux user namespaces map the guest to an unprivileged
  uid), and a documented operator recipe where it does not.
- **Physical AI (P8).** The control program runs in a guest; the adapter and the envelope check
  stay host-side; invariant 52 is untouched — the hardware chain never depends on the guest.

## 7. What `delulu doctor` reports — only checks that can change a decision

| Line | Verdict | Why it matters |
|---|---|---|
| `sandbox backends` | which of L1/L2/L3 are available here, each with the first missing prerequisite | tells an operator which profile will refuse |
| `user namespaces` (Linux) | usable by an unprivileged process / disabled by sysctl | L1 on Linux depends on it |
| `landlock` (Linux) | ABI version / absent | which fs and net denials are kernel-enforced |
| `seccomp` (Linux) | available / absent | syscall allowlist enforceable |
| `cgroup v2` (Linux) | writable for this user / read-only | whether memory and pid limits are kernel-enforced or best-effort |
| `kvm` (Linux) | `/dev/kvm` present and openable / absent | L2 possible at all |
| `vmm` (Linux) | Firecracker or Cloud Hypervisor found, version / absent | L2 launcher |
| `guest image` | present, hashes match the pinned manifest / absent / tampered | L2 integrity; tampered is a **problem** |
| `job object / restricted token` (Windows) | applicable / not | L1 on Windows |
| `appcontainer` (Windows) | profile creatable / not | stronger L1 on Windows |
| `seatbelt` (macOS) | `sandbox-exec` present and a probe profile applies / not | L1 on macOS |
| `identity separation` | the launcher can run the guest as a different principal / same user | whether Tier 2 is mechanical here — the category-7 line |
| `network in guest` | none by construction (L1/L2) / unknown (L3) | the DNS/metadata class |

Each line is a probe that actually attempts the thing (create a namespace, apply a Landlock
ruleset to a throwaway child, open `/dev/kvm`, apply a job limit), never a version string; a
probe that cannot fail is not a check. No line is added for something that changes no decision.

## 8. The microVM, designed from scratch

| Question | Design | Why |
|---|---|---|
| VM contents | a pinned Linux kernel (Firecracker guest config, 6.1 series, the minimum: virtio-vsock, no net, no block if initramfs suffices) and an **initramfs** holding one static, Python-less `delulu` built as the guest runtime (`__guest` hidden subcommand, like `__foreign-worker`) as `/init`, plus `/dev` and `/proc` | smallest TCB; no shell, no package manager, nothing to pivot to |
| Guest kernel | pinned by hash in a manifest the CLI carries; built reproducibly from a script in the repository (named in a code comment for the Survey) | image integrity is checkable; updates are deliberate |
| Root filesystem | the initramfs, read-only; no block device | nothing persists; nothing to mount |
| Image construction | `scripts/` builds kernel + initramfs on Linux, records hashes in `guest-image.json`; CI builds it on a Linux runner and attaches it as an artifact; distribution ships it as an optional, separately-checksummed download (P5) | GPL kernel distribution is a licensing act — owner decision (D-NE-27) |
| Program delivery | the checked source (or DIR) and its blake3, the grant, the policy and the seed/clock over the channel after the guest's `hello` | no disk, no injection path; the guest verifies the hash it is told |
| File injection / mounts | none | brokered reads/writes; sibling paths not merely invisible — nonexistent |
| Networking | no NIC; vsock only | no DNS, no metadata, no egress by construction |
| Default posture | everything denied; only the channel exists | — |
| Broker communication | host-side only; the guest never receives an address | invariant 25 strengthened |
| Authority acquisition | the host redeems the lease or applies the grant; the guest receives handles | — |
| Revocation | host-side; kill-on-revoke policy | — |
| Secrets | never enter the guest; `verify` brokered; `map` refused under L2 by default (§8.3) | — |
| stdout/stderr | brokered `println`/`print`; guest kernel console disabled in production images (Firecracker's own advice) | no side channel through the serial console |
| Exit codes | the guest reports its result over the channel; the host maps it to the CLI's 0/1/2/3 exactly as in-process | contract unchanged |
| Timeouts | host watchdog kills the VM at `wall_ms`; VMM CPU quota via cgroups; the guest cannot extend | — |
| Resource quotas | vCPU count, memory from the VMM; pids inside the guest are irrelevant (one process); disk none | — |
| Snapshots / restore | **not in the first version**; later a warm pool restored *once each* with VMGenID reseeding (Firecracker's rule) | boot is already ≤125 ms; snapshots add the entropy-duplication class |
| Cleanup | the jailer's chroot per VM; the host removes the vsock socket and the jail on exit or death; a sweep on start removes orphans | — |
| Crash handling | VMM exit → `guest_death` with the VMM's exit status; the host audits it and returns a DL, never a success | — |
| Guest death verification | the host waits on the VMM pid; on `kill` it confirms the pid is gone and the socket closed before reporting | "killed" means verified dead |
| State isolation | one VM per run; nothing shared; per-VM uid via the jailer | — |
| Image integrity | hashes in the manifest checked before launch; mismatch is a **problem** in `doctor` and a refusal in `run` | — |
| Kernel/rootfs pinning | manifest with hashes and source commit; bumping is a deliberate act with the reproducibility re-check `rust-toolchain.toml` already prescribes | — |
| Update strategy | new manifest, new artifact, old one refused after a deprecation window recorded in `CHANGELOG.md` | — |
| Escape detection | the host observes only the channel; any frame outside the protocol, any unexpected vsock connection, any VMM exit is audited; host `nft` rules (operator recipe) log anything from a TAP device — there is none, so any traffic is an alarm | — |
| Forensic evidence | the audit chain carries launch, limits, kills, deaths, violations with the VMM's exit codes; the jail directory is preserved on violation (operator flag) | — |

### 8.1 Why not the WASM engine in the guest (a spec change)

Stage 5 §6 mandates the WASM engine in the guest. The backend compiles 6 of 19 entry programs; a
microVM tier that refuses most programs with `DL1201` would be a tier in name. The guest runs the
interpreter. The WASM engine's *containment* value (a second enforcement floor) is retained
in-process for `.dwx` runs and for Contained plugins; the microVM provides the boundary the WASM
floor was standing in for. This is a normative change to §6 and needs a **ruling** (a build-order
deviation, not an RFC: it changes no language semantics).

### 8.2 Why not Hyperlight now

It is the only Rust, kernel-less, Windows-capable micro-VM and its host-function model is exactly
§2.1's B. It is pre-1.0, its WASM guest is "not production-grade", and a WASM guest would inherit
the fragment limitation. Recorded as the Windows L2 candidate to re-evaluate when (a) it stabilizes
and (b) the WASM backend covers the language or a native guest build of the interpreter exists.

### 8.3 The `Secret.map` tension

`Secret.map` hands a pure closure the plaintext. Under "secrets never enter the guest", `map` cannot
run in the guest. Options: (i) refuse `map` under L2/`hostile-agent` (a runtime DL with an honest
message; the program is otherwise unchanged); (ii) execute the closure host-side in a nested
interpreter over the plaintext (the closure is pure by typing; the host already runs the same
interpreter) — sound but a new host-side execution path; (iii) allow plaintext into the guest
under a profile flag, labelled. Recommendation: (i) first, (ii) as the follow-up once measured.
Owner ruling requested (D-NE-25) because it changes what a valid program does under a profile.

## 9. What must NOT be implemented yet, and why

- **Snapshots and warm pools** — until the boot path is measured and VMGenID handling is tested;
  the snapshot-reuse class is real and subtle.
- **A DeluluLang-owned container runtime or gVisor wrapper** — L3 covers it without new TCB.
- **Windows microVM** — no stable Rust path exists that runs the interpreter (Hyperlight is
  experimental and WASM-bound); L1 on Windows is the honest tier.
- **macOS L2 via libkrun/Virtualization.framework** — plausible later; needs a Mac to run
  anything (the project has none outside CI).
- **Attestation (L4)** — needs hardware and a relying-party design; the seam (a `guarantees`
  field and a "refuse to delegate without attestation" policy hook) is designed, not built.
- **Guest-side foreign libraries** — foreign code under L2 would need the library inside the
  image; keep foreign workers host-side (bounded blast radius as today) until images can carry
  operator-supplied libraries with their own hashes.
- **Effect batching optimizations** — after the round-trip cost is measured, not before.
- **Any silent fallback** — a profile that cannot be honoured is refused; this is a rule, not a
  deferral.
