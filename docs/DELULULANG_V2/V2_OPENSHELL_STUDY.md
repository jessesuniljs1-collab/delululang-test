# NVIDIA OpenShell, studied — what DeluluLang takes, what it does not, and the phases that build it

**Status:** study complete and design written 2026-09-28, on the owner's commission of the same day
(*"I want this to be studied and incorporated not just as copy but as real engineering for the sandbox
currently we are working on"*). **Nothing in §4–§6 is built yet**; every item there carries the phase
slice that builds it and the witness that will prove it. The decisions are D-V2-52 to D-V2-55
(`V2_DECISION_LOG.md`). Where this file says what OpenShell does, it reports OpenShell's own
documentation and source at the version named in §0 — it is not a claim DeluluLang has measured.

---

## 0. What was read

| Source | Version, date | What |
|---|---|---|
| NVIDIA OpenShell documentation | `latest` = v0.1.2 and `dev`, fetched as Markdown 2026-09-28 | all 57 pages the `llms.txt` index lists: about (overview, architecture, installation, support matrix), gateways (overview, authentication, configuration, container deployment), workspaces, sandboxes (overview, templates, runtimes), providers (overview, profiles, AWS, Google), policies (overview, network rules, manage, prover, advisor, default policy, schema), inference, extensibility (overview, supervisor middleware ×3, gateway interceptors, drivers, isolation backends), observability (logs, logging, OCSF export, telemetry), Kubernetes ×6, SDKs ×5 and API errors, security best practices, image verification, the 0.1.0 upgrade guide, tutorials ×5 |
| The seven architecture diagrams | the SVG sources in `docs/images/` of the repository | read label by label: system architecture, sandbox enforcement flow, Sandbox Protocol, sandbox authentication, isolation backend, extension points, the Kubernetes runtime |
| `github.com/NVIDIA/OpenShell` | commit `36b0386` (2026-09-28), a 38-crate Rust workspace | `architecture/` (README, sandbox, security-policy, compute-runtimes, sandbox-limits, gateway), `rfc/` 0001, 0002, 0005, 0012, and the source of `openshell-isolation-interface` (the typestate contract), `openshell-binary-identity`, `openshell-prover` (the Z3 model, its README) and `openshell-supervisor-network` |
| NVIDIA's announcement | 2026-09-28, "Open Agent Safety Platform" | OpenShell as the open runtime; **NVIDIA Sentry**, an out-of-band watchdog on BlueField-4 DPUs that "can quarantine agents that attempt to move outside their boundaries in milliseconds"; partners named (Anthropic's Claude Managed Agents among them) |
| NVIDIA's agent-safety solutions page | 2026-09-28 | the same platform described as "runtime governance, continuous threat detection, and hardware-isolated policy enforcement" |

**Terms (D-V2-52).** OpenShell is licensed Apache-2.0. **Nothing of it is copied into this
repository** — no code, no schema file, no documentation text, no diagram — and none of its crates
becomes a dependency: DeluluLang's licence and `NOTICE` are the owner's, and importing Apache-2.0
material would change what `NOTICE` must say. Every idea below is re-engineered in DeluluLang's own
terms. Where DeluluLang interoperates (PS-E-05), it *emits* documents in OpenShell's published policy
format — its own output — and names the product only to identify that format. A CI workflow that
exercises OpenShell downloads a pinned, checksum-verified release when it runs and ships nothing.

## 1. OpenShell in one page

OpenShell runs **arbitrary agent binaries** — Claude Code, Codex, OpenCode, Copilot CLI — inside a
sandbox, and governs what they may touch. Four roles, read from the system diagram:

- **Gateway** (control plane): the API, authentication (mTLS, OIDC, edge JWT), durable state
  (sandboxes, policy revisions, providers, settings), workspaces with platform/workspace roles, the
  **policy prover**, and the **compute driver** (Docker, Podman, Kubernetes, microVM; Windows MXC
  announced) that places and fences each sandbox.
- **Supervisor** (trusted, *outside* the workload's boundary): evaluates policy (an OPA engine),
  resolves DNS, opens every approved upstream connection, injects credentials, runs middleware, keeps
  the gateway session. "The supervisor is trusted and makes the decisions."
- **Sandbox** (`openshell-sandbox`, *inside* the boundary with the agent): owns the agent's process
  tree, identifies which executable made each request from trusted `/proc` data, intercepts TCP opens
  and DNS queries with seccomp user notification and hands them to the supervisor. It "never makes
  policy decisions" and "holds nothing worth stealing".
- **Agent**: the untrusted workload. Its only network path is the mediated channel.

**The boundary, as the enforcement and protocol diagrams draw it.** The workload is network-isolated
("deny all egress except to the supervisor"): Docker/Podman turn the workload container's networking
off, Kubernetes uses a NetworkPolicy, the microVM has no network device. Supervisor and sandbox share
**one mutually authenticated HTTP/2 connection** (a Unix socket, TCP, or vsock, chosen by the driver)
carrying a control stream (attach · confirm · start · exec · signals · health), a DNS stream, and one
stream per TCP connection with its own backpressure. **Fail closed:** the sandbox holds the launch
until the supervisor confirms, and *freezes the agent if the connection drops*; only the same
supervisor process may reconnect.

**The isolation-backend contract (RFC 0012, `contract.rs`).** A typestate: `attach` → `BoundBoundary`
→ `confirm` → `ConfirmedBoundary` → `ReadyBoundary::start_agent` → `RunningBoundary`. Each transition
consumes the previous state by value, so *the agent cannot be started on a boundary that was not
confirmed* — by construction, not by convention. `ConfirmedBoundary::try_new` checks four
backend-neutral properties (filesystem confinement, egress interception, request attribution,
privilege floor), each `enforced` with a named mechanism; four outer-fence guarantees
(default-deny egress, no unmanaged egress path, revocation verified, **controller loss fails
closed**) bound to the sandbox's **generation** with a digest of the native evidence; and that the
runtime's exit terminates the workload.

**Authentication (the authentication diagram).** The gateway alone signs. The compute driver hands the
supervisor a bootstrap credential; the gateway issues a JWT pair per sandbox *generation* — one for
supervisor→gateway calls (only what a supervisor needs), one for supervisor→sandbox calls, which the
sandbox verifies holding only the gateway's public key. A restart is a new generation with fresh
tokens and fresh TLS certificates; the old ones stop working.

**Policy.** Declarative YAML, five sections: `filesystem_policy` and `landlock` and `process` (static,
fixed at creation), `network_policies` and `network_middlewares` (dynamic, hot-reloadable). Deny by
default. A network rule is *binaries × endpoints*; an endpoint may add request inspection (`rest`
method/path/query, `websocket`, `graphql` operation/fields, `mcp` method/tool, `json-rpc`), with
`enforce` or `audit` (the default). Deny rules beat allow rules. Executables are identified by the
kernel's real path, a rule covers the listed binary and its descendants, and each authorizing path is
**pinned to the SHA-256 of the first executable observed there** — a later mismatch is denied.
Loopback, link-local and cloud-metadata destinations can never be authorized; private addresses only
by exact host or `allowed_ips`. The supervisor answers DNS itself with short-lived placeholder
addresses.

**Credentials ("providers").** The agent never sees a real credential: its environment holds
**opaque placeholders**, which the proxy resolves *only* in requests to the endpoints the provider's
profile binds — two independent checks, network policy and credential binding — and which fail
closed when unknown, expired or sent elsewhere (`credential_endpoint_mismatch`). Middleware sees
traffic after policy and *before* credentials are injected. Logs never carry secrets.

**The prover.** A Z3 model of a policy. A **boundary check** answers "does this candidate allow
anything the boundary does not?" with exactly four results — `within_boundary`, `exceeds_boundary`
(with one counterexample), `unsupported` (a shape the model cannot decide soundly, e.g. a candidate
path nested under a different boundary path, because a symlink could move it), `inconclusive` — and
the list of domains it covered. A **proposal risk check** compares reach with and without a proposed
rule and reports categorical findings: `link_local_reach`, `l7_bypass_credentialed`,
`credential_reach_expansion`, `capability_expansion`.

**The advisor.** When a request is denied, the denial tells the agent how to ask; the agent proposes
the narrowest rule through `policy.local`; the gateway re-validates it against the live policy, runs
the prover, and either waits for a human (the default) or auto-approves when the operator opted in
*and* the prover found nothing *and* no destination is flagged.

**Observability.** Every security-relevant event is an **OCSF v1.8.0** record (network, HTTP,
process, detection finding, configuration change, lifecycle), with a one-line shorthand for humans
and agents and an optional JSONL export for a SIEM.

**Sentry** (announced, hardware): out-of-band telemetry and enforcement on a DPU, able to quarantine
an agent in milliseconds, outside anything the agent's host could tamper with.

## 2. The difference that decides everything

**OpenShell governs programs it cannot read. DeluluLang governs programs it compiles.**

OpenShell's workload is an opaque binary, so its policy is a *separate artifact* describing what the
binary may do, enforced by watching the binary's syscalls and deciding each request at run time. It
must allow the binary to open sockets (to intercept them), must identify the binary by path and hash
(because the binary is the only identity it has), and must keep the policy in step with the agent by
hand, by proposals, and by a prover.

DeluluLang's workload is a program whose **authority is computed from its own text before it runs**
(`delulu authority`), whose effects are **in its type**, and which runs as a guest that **performs no
effects at all** — the host performs every effect over the channel, under custody, the Guard and
audit. Three consequences follow, and every design choice below comes from one of them:

1. **DeluluLang can forbid what OpenShell must allow.** A DeluluLang guest needs no socket, no DNS,
   no network syscall: its channel is connected before it locks itself down. Where OpenShell
   intercepts, DeluluLang can simply deny.
2. **DeluluLang's policy is derived, not written.** `derive(authority, grant, profile)` →
   `SandboxPolicy` (PS-A). The question OpenShell's prover asks of a *policy*, DeluluLang can ask of
   a *program*: "can anything this code does exceed this boundary?" — and point at the line.
3. **DeluluLang's order is proved once, not per policy.** `⊑` is proved an order in Z3 over ten
   dimensions (`MATHEMATICS.md` §1, the budget dimension §6b); every delegation is checked against
   it. A boundary check is a `⊑` query, not a new model.

The two are complementary, not rivals: OpenShell is the right wall around an agent written in any
language; DeluluLang is the language whose programs can say exactly what that wall must permit.
§4.5 turns that sentence into a feature.

## 3. The two designs side by side

State labels are this repository's: **implemented** (in the binary, witnessed), **designed**,
**absent**. File references are to DeluluLang.

| OpenShell | DeluluLang today | State | Where this study takes it |
|---|---|---|---|
| Supervisor decides; sandbox never decides | the host decides and performs every effect; the guest holds opaque handles (`guest.rs`, `channel`) | implemented, and stronger (the guest makes no request the host did not define) | — |
| Workload network-isolated; only the supervisor channel | L1: Landlock denies TCP bind/connect (ABI v4); AppContainer with no capabilities (Windows); Seatbelt (macOS); L2: vsock only, no NIC | implemented — but on Linux Landlock covers **TCP only**, so UDP, raw, netlink and Unix sockets are outside it, and the docs say so (`jail.rs::confine_filesystem`) | PS-E-03: deny socket creation outright after lock-down |
| Typestate: nothing runs until the boundary is confirmed | the guest reports its self-applied layers first (RW 4.23, `channel::SELF_APPLIED`); an external launcher's attestation is verified before the program is sent (PS-D-02) | implemented in pieces, by convention in `guest.rs`, not by construction; the report *describes* a missing layer and the run proceeds | PS-E-01 |
| Confirmation bound to a generation | the attestation is bound to a per-run nonce — at L3 with `--require-attestation` only | partial | PS-E-01: every run has a generation |
| Controller loss fails closed; runtime exit ends the workload | Linux `PR_SET_PDEATHSIG` (guest and VMM), Windows Job kill-on-close; macOS: a spinning orphan lives until its CPU limit (`jail.rs` notes it); an external launcher is started with no death signal and no jail (`launch_external`) | partial | PS-E-02 |
| seccomp denylist (io_uring, userfaultfd, memfd, pidfd, the mount API, namespace flags, socket families …), `RLIMIT_CORE=0`, `PR_SET_DUMPABLE=0`, `no_new_privs`, Landlock ABI ≥ 3 required | seccomp denies 18 syscalls (+fork/vfork on x86-64); `no_new_privs`, `RLIMIT_CORE=0`; Landlock requested at ABI v3 and reported; **the host is dumpable; the guest may read all of `/proc`** | partial | PS-E-03 (hypotheses H1–H6, each witnessed before it is called a defect) |
| Executable identity pinned by SHA-256, TOFU | the *program's* identity is its bytes (sign-off DL1905, manifest DL0701, `.dpx` re-proof, break-glass bound to one program); a hardware driver is verified and started as one file (ADAPTER-SPELL-1); **an external launcher is looked up on `PATH` at spawn and never hashed** | partial | PS-E-04 |
| Compute drivers: Docker, Podman, microVM, Kubernetes | L1 process, L2 Firecracker microVM, L3 `external:CMD` with a Docker + gVisor recipe (not tested) | implemented | PS-E-05: OpenShell as a tested L3 |
| Policy YAML, hand-written or proposed | `SandboxPolicy` derived from authority; `delulu sandbox policy --json`, hashed into the audit | implemented | PS-E-05: export it as an OpenShell policy |
| OCSF events, JSONL export | hash-chained, anchored audit (`delulu-broker/src/audit.rs`); no OCSF | implemented (audit); absent (OCSF) | PS-E-06 |
| Out-of-band watchdog (Sentry, DPU) | the device dead-man watchdog, e-stop, transitive revocation — host-side | implemented (in-band) | P8-04: an out-of-band monitor with revoke authority only |
| Prover boundary check, four results, coverage | `⊑` proved; delegation refused when wider (DL0802); `authority --grants` prints what a program needs; **no "is this program within that boundary?" command** | partial | P9-01 |
| Prover risk findings on a change | none | absent | P9-02 |
| Advisor: proposals, review, auto-approval on no findings | refusals carry structured repairs (DL codes); nothing files a request with the operator | absent | P9-03 |
| Credential placeholders bound to endpoints | `Secret[T]` is opaque (`map`, `verify` — broker-side since VERIFY-FABRICATED-1 — and `expose`); the guest holds no secret bytes; **no program can send a secret to an endpoint at all** (`http` has one method, `get(url)`) | absent | P9-04 |
| L7 rules: method, path, query, GraphQL, MCP tool | `net` grants are host-scoped; `http.get` only | absent | P9-05 (method and path); the rest named in §5 |
| Middleware (content inspection) | not claimed: "a program may send anything to a host it was granted" (category 7) | absent by design | §5 |
| Gateway, OIDC, workspaces, Kubernetes | the broker daemon, grant trees, `fleet.rs`; one OS user is never multi-tenant | different scope | §5 |

## 4. What DeluluLang takes — the design of each slice

Every slice follows the inner loop (`docs/CLOUD_ROUTINE.md` step 5): the Survey first, **the defect
or gap witnessed failing before it is fixed, every new test falsified by a mutant**, clippy, the suite
alone, the Survey regenerated last, the CI run read. A hypothesis below is not a finding until its
witness is red on the current binary; if the witness cannot be made red, the hypothesis is recorded as
refuted, with the evidence, and nothing is "fixed".

### 4.1 PS-E-01 — the boundary is confirmed before the program is sent, by construction

**Now.** `guest.rs` launches the guest, reads its `Confined { applied }` report (RW 4.23), verifies an
attestation where one is required (PS-D-02), then sends the program. The order is right; nothing in
the types makes it so, and a missing layer is *reported* (`posture`, `host_guarantees`,
`fully_enforced: false`) while the run proceeds — for every profile, including `hostile-agent`.

**Design.** Split the launch into states whose transitions consume their predecessor:
`Launched` → `confirm(…)` → `Confirmed` → `send_program(&Confirmed)` → `Running`. `confirm` is the only
constructor of `Confirmed`, and it builds one record:

- `generation`: a fresh nonce for **every** run at every level (today only attested L3 runs have one),
  written into the `sandbox-launch` audit record and the report, and bound into any attestation;
- five **properties**, each `established { mechanism, evidence }`, `absent { why }` or `unknown`:
  `filesystem_confinement`, `egress_confinement` (the guest reaches nothing but its channel),
  `privilege_floor` (`no_new_privs`; a restricted token / AppContainer; a subordinate uid; a VM),
  `host_loss_ends_guest` (PS-E-02), `resource_ceiling` (the budget's enforcement);
- the profile's **required set**: `hostile-agent` requires all five; `contained` (the default)
  requires filesystem, egress and resource; `dev` requires none; an unattested external launcher is
  `unknown` throughout, as PS-D-01 already reports it, and satisfies a requirement only through an
  attester's claim that names it.

A required property that is not `established` **refuses before a byte of the program is sent**, under
DL1408's rule ("an unsupported level refuses; it never silently downgrades"), naming the property,
the mechanism that was missing, and the ways out (the microVM, an external launcher, a host setting).
`host_guarantees` is derived from the confirmation — one source, not two. A new diagnostic code only
with a D-V2 record.

**Consequence named now:** `hostile-agent` on a Linux host that forbids user namespaces (Ubuntu's
default) stops running the guest as the operator's own user and refuses instead. That is the point:
code you do not trust should not run as you. The refusal names `--isolation microvm` and the one
sysctl.

**Witnesses.** (1) A guest whose confirmation lacks a required property receives no program bytes —
observed by a canary program whose first statement asks the host for a marker effect that never
appears in the audit. (2) `hostile-agent` on the arm64 CI job (no user namespaces) refuses; the x86-64
job (namespaces allowed) runs. (3) The report's properties equal what was measured, on all three
OSes. **Mutants:** `send_program` reachable without `Confirmed` (a second constructor); a required
property treated as established when `unknown`.

**Built so far (2026-09-28, routine run 2, D-V2-56).** Building it found the "Now" above too kind: the
host's first frame WAS the program, and the guest confined itself and reported only afterwards. The
channel is now `/3` — `Open` (the generation, no program) → the guest's `Confined` for that generation →
`Program` — and `crates/delulu/src/boundary.rs` holds the typestate (`Opened::confirm` the only
constructor of `Confirmed`, `Confirmed::send_program` the only writer of the program). Every run has a
generation, in the report and the launch and death records. Witness (1) is built in the form the channel
allows before the required sets exist: a launcher that never confirms receives no program byte
(`tests/sandbox_confirm_cli.rs`). **Second step (D-V2-57):** every run reports the five properties
(`sandbox.properties`), answered from its own posture; an L3 run's are `unknown`. Read against the jail
code, the `contained` set above would refuse every macOS run (reads open, no memory ceiling), so the
sets wait for each OS's reported answers. **Third step (D-V2-59):** CI's answers read — Linux x86-64,
arm64 and Windows all five, macOS two — `hostile-agent` requires all five and is refused in
`Opened::confirm`; `contained` requires none until macOS's gaps close. RW 4.31 closed the same run: an
L3 guest's own words are `guest_reported`, never host guarantees. Open: `contained`'s set, attesters'
claims as properties.

### 4.2 PS-E-02 — the guest ends with its host, on every backend, measured

**Now.** Linux guests and the microVM's VMM carry `PR_SET_PDEATHSIG = SIGKILL`; Windows guests live in
a Job Object with kill-on-close. **macOS has no death signal**: a guest that is computing and asking
for nothing never notices its host is gone, and lives until `RLIMIT_CPU` ends it (the default budget
is five minutes of CPU) — `jail.rs` documents this. **An external launcher** is spawned with neither
(`launch_external`): if the host dies, the launcher sees its standard input close, and what it does
next is its own.

**Design.** `host_loss_ends_guest` becomes a property with a mechanism per backend, and the gaps are
closed: on **macOS** the guest runs a watcher thread before the program starts — `kqueue`
`EVFILT_PROC`/`NOTE_EXIT` on the host's pid, `getppid() == 1` as its fallback — that `_exit`s at once;
on **Linux** the external launcher gets `PR_SET_PDEATHSIG` too, and on **Windows** it joins a Job
Object with kill-on-close. What the launcher itself started (a container) is the launcher's to end:
the report says "the launcher was ended with the host; what it started is the launcher's", unless an
attester claims more.

**Witnesses.** For each backend and OS: a guest that spins without asking; the host killed with
`SIGKILL` / `TerminateProcess`; the guest's pid gone within 1 s, measured from outside both. macOS
first, because it is the red one. **Mutant:** remove the watcher → the macOS witness goes red.

**Built so far (2026-09-28, routine run 2):** the Linux external launcher — `PR_SET_PDEATHSIG` in its
`pre_exec`, witnessed red first (a host killed with SIGKILL left its launcher running). **Then (2026-09-29,
routine run 3, D-V2-60) macOS**, witnessed red on a runner first — the guest and the launcher both outlived a
killed host — and closed by a watcher that runs OUTSIDE the guest rather than as a thread in it (a watcher in
the guest is the guest's own word): `kqueue` on a pipe only the host holds and on the guest's exit, claimed
only once armed. **And Windows** (the same run): the launcher joins a job whose only limit is kill-on-close,
red `37828ab`, `witness.yml` run `36527876892` — "the external launcher outlived its host by more than 3.0 s", green `1a63829`: `36528140459` (`sandbox_confirm_cli`, 7 passed — the launcher gone 20 ms after its host was killed), `36528149629` (`sandbox_external_cli`), `36528152057` (the jail unit tests), `36528154367` (`sandbox_run_cli`) — all success. **E-02 is complete.**

### 4.3 PS-E-03 — the guest's kernel surface, narrowed to what a guest that performs no effects needs

OpenShell's filter is broad because its agent must keep working; DeluluLang's guest has already
connected its channel when it locks itself down, so it can be denied far more. Six hypotheses, **none
yet witnessed**; each is tested by an *escaped guest* — a test-only guest mode (as PS-C's hostile
guests were) that, after lock-down, makes raw syscalls as a compromised interpreter would, and reports
what succeeded.

- **H1 — syscalls the filter does not name.** `memfd_create`, `io_uring_setup`/`enter`/`register`,
  `userfaultfd`, `pidfd_open`/`pidfd_getfd`/`pidfd_send_signal`, the new mount API (`fsopen`,
  `fsconfig`, `fsmount`, `fspick`, `move_mount`, `open_tree`, `mount_setattr`), `kexec_file_load`, and
  `clone`/`clone3` with `CLONE_NEWUSER` (`unshare` is denied; namespace creation by `clone` is not).
  Landlock already refuses mount-table changes to a restricted task, so some of these may prove
  redundant — the witness decides, and a redundant denial is still kept as depth only if it costs
  nothing. `clone3` cannot be filtered by argument; it is answered `ENOSYS` so libc falls back to
  `clone`, whose flags can.
- **H2 — sockets.** Landlock mediates TCP only. An escaped Linux guest may still create UDP (a DNS
  exfiltration channel if the host has a network), netlink, and Unix sockets — and a **same-uid** guest
  (Ubuntu's default, no subordinate uid) may `connect` to the operator's own Unix sockets: an SSH
  agent, a session bus. The guest needs no new socket after lock-down, so **`socket` is denied
  outright** (and `socketpair` too unless the witness shows the runtime needs it). This is stronger
  than a network namespace and needs no privilege.
- **H3 — `/proc`.** The Landlock ruleset grants reads beneath `/proc`. A same-uid guest may read
  `/proc/<pid>/environ` and `cmdline` of the operator's other processes — an API key in a shell's
  environment — because the read-mode ptrace check passes for the same uid and Yama restricts only
  attaching. **Design:** grant `/proc/self` (the directory the guest resolves to) and the few global
  files the runtime is witnessed to need, not `/proc`.
- **H4 — the host is dumpable.** While a same-uid guest runs, the host holds custody and secret
  bytes. **Design:** the host sets `PR_SET_DUMPABLE=0` on itself for the life of a sandboxed run, so no
  same-uid process — an escaped guest or anything else — can attach to it or read its memory or
  environment (OpenShell's "non-dumpable broker"). Witness: an escaped guest reads a sentinel from the
  host's `/proc/<pid>/environ` before, and cannot after (with H3 reverted, so H4 is witnessed alone).
- **H5 — the Landlock ABI.** The report states the effective ABI; `hostile-agent` requires ≥ 3
  (truncation mediated) through PS-E-01's required set; abstract Unix sockets are scoped at ABI 6
  where the kernel has it (H2 makes this depth, not the boundary).
- **H6 — the other two operating systems, the same question.** The macOS Seatbelt profile's
  `mach-lookup` allowances and the Windows AppContainer's reachable named objects are audited with the
  same escaped-guest harness: what could a compromised guest *reach*, not only *open*.

Each confirmed hypothesis is a finding with a name (`GUEST-SOCKET-1`, `GUEST-PROC-1`,
`HOST-DUMPABLE-1`, … — chosen when witnessed), a witness red on the old code, a fix, a mutant, and a
line in `V2_SECURITY_MODEL.md` §10.

**Built so far (2026-09-29, routine run 3, D-V2-61).** The escaped guest is a test-only child
(`jail::escaped_tests`) — the guest's own lock-down, then raw calls, against a free control. **H1, H2 and H3
were all red** on `1a63829`: GUEST-SYSCALL-1 (memfd, io_uring, userfaultfd, pidfd, fsopen, and a user
namespace through `clone`), GUEST-SOCKET-1 (UDP, netlink, Unix sockets, a connect to the operator's
socket — while the report said `network: only the channel`), GUEST-PROC-1 (another process's `environ`).
Building H3 found **H7, GUEST-DEV-1**: `/dev` granted whole let the guest open the operator's terminal.
All four closed — the filter refuses the calls, every new socket and `clone`'s namespace flags (`clone3`
ENOSYS); Landlock grants `/proc/self` and five devices — each with a mutant. **H4 the same run
(D-V2-62):** red as a non-root user — a same-user process read a serving host's `environ`
(HOST-DUMPABLE-1) — closed by `PR_SET_DUMPABLE = 0` once the guest is launched. **H8, found the same run
(D-V2-63):** the guest shares its host's session, so the operator's terminal is its controlling terminal,
and an escaped guest pushed keystrokes into it with `TIOCSTI` (GUEST-TIOCSTI-1) — closed in the filter,
compared on the command's low 32 bits. **H9, the same run (D-V2-64):** it could signal any process of the
same user — its host's process group, or `kill(-1)` (GUEST-SIGNAL-1); now it signals only itself. **H10
(D-V2-65):** and it could change their limits, priority, CPUs and scheduling (GUEST-PROCESS-1) — refused.
Open: H5, H6.

### 4.4 PS-E-04 — the external launcher is resolved once, hashed, and optionally pinned

**Now.** `launch_external` passes the command's first word to `Command::new`, which looks it up on
`PATH` at spawn; the report names the program, never its bytes. ADAPTER-SPELL-1 was the same shape for
hardware drivers, fixed by resolving once in DeluluLang (D-V2-50).

**Design.** Resolve the launcher to an absolute path in DeluluLang (the `resolve_driver` code, shared,
not copied); open it; hash the opened file; start **that** file (Linux: `fexecve` of the opened
descriptor; Windows: the resolved path with the file held open, deny-write). The report and the
`sandbox-launch` audit record carry `launcher: { path, digest }`; an attestation statement binds the
launcher's digest with the nonce (the statement's version bumps); `--launcher-digest HEX` pins it, and
a mismatch refuses before anything starts. The digest algorithm follows the repository's file hashes
(BLAKE3), recorded in the decision that builds it.

**Witnesses.** The digest in the report equals an independent hash of the file; a pinned run with a
swapped file refuses; a `PATH` entry planted ahead of the intended launcher is not what runs. **Mutant:**
hash one file, start another.

### 4.5 PS-E-05 — OpenShell as a tested L3 backend, and DeluluLang as its policy author

Two directions, one soundness rule: **what DeluluLang emits never allows more than the program's
authority and grants allow.**

**(a) `delulu sandbox policy <file> --format openshell`.** From the same `derive(authority, grant,
profile)` that PS-A built, emit an OpenShell policy (`version: 1`) for running `delulu run` inside an
OpenShell sandbox:

- `filesystem_policy`: `read_only` = the system paths plus each `fs.read` grant's root; `read_write`
  = each `fs.write` grant's root — each as the absolute, resolved path the CLI already stores (Ruling 2:
  resolve at the edges), because OpenShell accepts only absolute paths with no `..`;
  `include_workdir: false`; `landlock.compatibility: hard_requirement`;
- `process`: a non-root `run_as_user`/`run_as_group`;
- `network_policies`: one rule per granted host — the endpoint on 443, `protocol: rest`,
  `enforcement: enforce`, and an explicit rule allowing `GET` on every path — **not** the `read-only`
  preset, which also allows `HEAD` and `OPTIONS`, a method set wider than the program's authority (the
  language's `http` performs `GET` only — derived from the primitive table, so P9-05 widens the rule only
  when the language does) — and **one binary: the `delulu` executable's real path** in the image;
- a grant with no OpenShell equivalent (`net.special=` to a loopback or link-local address, which
  OpenShell never authorizes; a budget; a device) **refuses the export** by name, or — for dimensions
  OpenShell simply does not model (budgets map to `--cpu`/`--memory` at creation) — is listed under
  `unrepresented` in the JSON envelope and a comment block. Never dropped in silence, never widened.

**(b) The guest inside OpenShell (L3).** A recipe, in `docs/DEPLOYMENT.md`: a sandbox whose policy
has **no network rules at all** and a read-only filesystem, reached through
`--sandbox-backend external:<wrapper>` whose wrapper runs `openshell sandbox exec … -- delulu __guest
--stdio-pipes`, carrying the channel on the exec's standard streams. The level stays **3**, the
guarantees `unknown` unless attested (PS-D-02); DeluluLang never reports OpenShell's controls as its
own measurement.

**Witnesses — a manual CI workflow first (`openshell.yml`, `workflow_dispatch`, as `container.yml`
was):** install OpenShell's pinned release with its checksums; start a local gateway with the Docker
driver; build the repository's image (`Dockerfile`, RW 7.3); then (1) a granted program runs and an
ungranted one is refused by DeluluLang; (2) a `curl` inside the sandbox to a host the exported policy
does not list is denied by OpenShell (its `NET:OPEN … DENIED` log line read, not assumed); (3)
`openshell-prover check export.yaml --boundary allowed.yaml` answers `within_boundary`, and against a
boundary missing one granted host answers `exceeds_boundary` with that host as the counterexample —
**the falsifier**: two independent formal tools, one DeluluLang's, one NVIDIA's, agreeing on the same
program. A unit test in-tree holds the export's soundness rule over every example program
(`delulu examples`): each emitted endpoint and path traces to a grant.

### 4.6 PS-E-06 — the audit chain, exported as OCSF, still verifiable

`delulu audit export --format ocsf [--since SEQ]` writes JSON Lines, one OCSF v1.8.0 event per audit
record. Proposed mapping, each class UID checked against the published 1.8.0 schema before it ships:
an `http` effect → HTTP Activity (4002); a filesystem effect → File System Activity (1001);
`sandbox-launch` → Process Activity (1007); `delegate`/`revoke`/`grant` → an access-management class;
a refusal, a break-glass use, a special-use reach → Detection Finding (2004). Every event carries the
record's `seq`, `hash` and `prev_hash` under `unmapped.delulu`, so **the chain can be re-verified from
the export** — OCSF records alone cannot say whether one was removed; these can. Never secret bytes (the
audit holds none — the export test asserts it for every record kind), never a query string.
Witnesses: the export of a corpus run validates against the schema; deleting one line from the export
is detected by `delulu audit verify --ocsf`. Mutant: an export that drops `prev_hash`.

### 4.7 P8-04 — an out-of-band monitor (the shape of Sentry, in software)

A DPU is out of reach; its *shape* is not. A **monitor** is a separate principal — its own process, a
separate OS identity where the host allows one — holding exactly one authority: **revoke** over one
run's grant node. It reads the audit as it is appended (or PS-E-06's stream) and applies declarative
rules — N refusals within T, any envelope refusal on an actuator, any break-glass use, any special-use
reach — and **quarantines** by revoking: the run's e-stop and fail-state engage (implemented for
devices), the guest is ended. It cannot grant, cannot perform an effect, and trusts nothing the guest
says, only what the host recorded. Witness against the simulator: a guest that keeps probing refused
operations loses its actuator within a measured bound after the Nth refusal; the monitor, compromised
(a mutant that tries to grant), is refused by the broker.

### 4.8 P9-01 — `delulu authority <file> --within BOUNDARY`

The question OpenShell's prover asks of a policy, asked of a program. BOUNDARY is a grant set in the
`--grant` grammar (a file or flags). The result is one of exactly four, as a closed enum:
**within** (exit 0), **exceeds** (1, with a counterexample: the first requested scope the boundary
does not cover *and the source span of the call that requests it* — a line of code, which a policy
prover cannot give), **unsupported** (a dimension the program needs that the grant grammar cannot
bound, named), **inconclusive** (a resource limit reached); a bad invocation is 2, as everywhere in the
CLI. `--json` carries the result, the counterexample, and the **coverage** — the `⊑` dimensions
decided. It is the same `⊑` the lease tree enforces (`MATHEMATICS.md` §1) applied to
the static authority `delulu authority` already computes. Use: a supervising agent checks a
sub-agent's program before running it, then runs it with exactly the grants it needs. Witnesses: the
guide corpus, each program within its own `--grants` and exceeding each one-dimension narrowing, the
counterexample naming the right call; mutants: a dimension skipped (answers within wrongly), a
counterexample from the wrong span.

### 4.9 P9-02 — risk findings on a change of authority

`delulu grants diff OLD NEW` (grant sets or lease nodes) answers, with no severity grade, which
categorical findings the change introduces: `special_use_reach` (loopback, link-local, metadata or
private reach through `net.special=`), `effect_expansion` (a new effect kind), `scope_expansion` (a
wider scope in an existing kind), `device_reach_expansion`, `budget_expansion`, and, once P9-04 exists,
`secret_reach_expansion` (a secret usable at a destination it could not reach before). Each is a `⊑`
comparison per dimension — deterministic, reproducible, and the input P9-03 needs.

### 4.10 P9-03 — asking for authority: proposals, review, and approval bounded by `⊑`

When a run is refused for want of a grant, the refusal already carries a structured repair. P9-03 lets
that repair become a **request to the operator**, never a grant to itself: `delulu grants propose
--from <report.json>` files the narrowest grant set that admits exactly the refused request — computed
from the static authority of the refused call site, not from anything the program claims — with the
broker, as pending and audited; the operator answers `grants approve ID` or `grants reject ID
--reason …` (the reason returns to the agent, which may propose narrower). **Automatic approval exists
only as the operator's own standing delegation:** `grants auto-approve --within BOUNDARY` approves a
proposal only when it is `⊑` BOUNDARY *and* P9-02 finds nothing — a check the lease tree already
knows how to make, where OpenShell's is a heuristic list. An approved proposal is a new, narrow child
node; the program is **re-run**, never widened while running (V2 §5: revocation and restart over hot
weakening). The invariant V2 rests on survives: *a program cannot relax its own authority.*

### 4.11 P9-04 — secrets bound to endpoints, checked before the run and enforced at it

**The language change.** `http` gains a request with headers (and, with P9-05, methods), in which a
header value may be a `Secret[Str]`. A grant binds a secret's **audience**:
`--grant secret=GITHUB_TOKEN@api.github.com:443/repos/org/repo/**`. The guest never holds the bytes
(already the rule); the host's egress client resolves them from custody at send time, only if the
request's destination — and every redirect hop, re-checked as PS-B-02 already re-checks them — is
inside the audience; otherwise it refuses (`credential_endpoint_mismatch`'s DeluluLang twin) and the
secret is not sent. **The static half, which no runtime-only system has:** `Secret[T]` is already
opaque, so the checker can prove a secret flows only into a request's header position, and
`delulu authority` can report, per secret, every destination the program can send it to — before it
runs. A reviewer reads "`GITHUB_TOKEN` → `api.github.com` only" off the program. New primitive rows,
`PRIM_TABLE_VERSION` bumped, conformance anchors positive and negative, a D-V2 record.

### 4.12 P9-05 — network authority scoped to methods and paths, as a `⊑` dimension

`net` grants gain methods and path prefixes: `net=api.github.com:GET,POST/repos/org/repo/**`. As an
order: methods are a finite set under `⊆`; paths are restricted globs (`*` one segment, `**` only as
the final segment), whose containment is decidable by the syntax alone — and, like OpenShell's prover,
a shape whose containment would depend on the filesystem or on unrestricted globbing is refused, not
guessed. The meet exists; the dimension joins the Z3 model and the exhaustive enumeration
(`MATHEMATICS.md` §1), with mutants CI requires to fail. The runtime check lives in the egress client
the request already passes through.

## 5. What DeluluLang does not take, and why

- **Deny rules inside allow rules.** A capability system narrows by attenuation: a narrower grant is
  a smaller scope, not a larger one with a hole. OpenShell needs deny rules because its allow rules are
  broad globs over binaries it cannot read. If "everything under the repository except its webhooks"
  proves common, the answer is a set-difference scope with a decided `⊑`, designed as its own slice.
- **Hot-widening a running sandbox.** OpenShell reloads network policy live, including wider policy.
  V2's rule stands: revocation, kill and restart over hot weakening (`V2_SECURITY_MODEL.md` §5).
  P9-03's approved proposal takes effect on the next run.
- **Binary identity for the guest.** The guest *is* the interpreter; what it runs is identified by its
  bytes already (sign-off, manifest, `.dpx` re-proof). Only the external launcher — a binary DeluluLang
  does not compile — is pinned (PS-E-04).
- **`audit` as the default enforcement.** OpenShell's request rules default to logging violations. A
  DeluluLang grant is the control; AUDIT is a separate execution mode that performs no effects at all.
- **An HTTP endpoint inside the sandbox for proposals.** The guest has no network; a proposal travels
  as a report through the CLI and the broker (P9-03).
- **Content inspection (middleware).** Still not claimed (category 7). If it is ever built, its shape
  is P8-02's: a Verified-class `.dpx`, a pure function the host calls on the request *before* a secret
  is injected — never code that sees the secret.
- **A fleet control plane (gateway, OIDC, workspaces, Kubernetes, JWT pairs).** A language toolchain's
  concern is the program and the host that runs it. What transfers is already taken: generations
  (PS-E-01), a credential that names one run, a verifier that holds only a public key (PS-D-02's
  pinned attester key). A DeluluLang fleet surface, if the owner wants one, is its own phase.
- **L7 protocols beyond REST** (GraphQL operations, MCP tools, JSON-RPC methods, WebSocket messages).
  Named for later: the same `⊑` treatment as P9-05, once a DeluluLang program can speak them.

## 6. The phases, as they change

| Order | Phase | Slices | Why here |
|---|---|---|---|
| next | **PS-E — the boundary, confirmed** (D-V2-53) | E-01 confirmation by construction · E-02 host loss ends the guest · E-03 the kernel surface (H1–H6) · E-04 the launcher pinned · E-05 OpenShell as L3 and as a policy target · E-06 OCSF export | P8-01 puts control programs in guests; the boundary they stand on is confirmed first |
| then | **P8 — safe autonomy** (D-V2-51, D-V2-54) | P8-01 the control program in a guest (on E-01's confirmation) · P8-02 the Verified-class adapter · P8-03 the reference transport · **P8-04 the out-of-band monitor** | unchanged order; one slice added |
| then | **P9 — authority at the boundary** (D-V2-55) | P9-01 `--within` · P9-02 risk findings · P9-03 proposals · P9-04 endpoint-bound secrets · P9-05 method and path scopes | extends the authority model; each touches `⊑` or the primitive table, so each carries its proof or its conformance anchors |

A slice is sized for one routine run and ends with its CI run read green. A phase is complete when
`V2_PHASE_STATUS.md` names its commits and runs.

## 7. What this study does not claim

Nothing about OpenShell's enforcement has been measured by DeluluLang; §1 reports its documentation
and source at `36b0386` / v0.1.2. OpenShell is at 0.1.x and its 0.1.0 upgrade was breaking: PS-E-05
pins the version it tests and says so, and an interop test that breaks on an OpenShell release is
recorded as that, not as a DeluluLang regression. The hypotheses in §4.3 are hypotheses. Sentry is
hardware DeluluLang does not have; P8-04 borrows its shape, not its guarantees.
