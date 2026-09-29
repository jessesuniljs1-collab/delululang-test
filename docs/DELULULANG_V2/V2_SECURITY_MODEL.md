# V2 security model — Authority, the Guard, custody, and the sandbox beneath them

**Status:** the active design, binding on every V2 phase. Every statement below carries one label —
**implemented** (in the binary today, with a witness), **measured**, **designed** (specified here,
not built), **partially implemented**, **unverified**, or **deferred** — and the labels are never
collapsed. The full design of the isolation layer, its threat model with the evidence category each
claim owes (T1–T15), and its test plan are in the archive
(`docs/archive/v1/NEXT_EVOLUTION_2026/SANDBOX_ARCHITECTURE.md`, `SANDBOX_THREAT_MODEL.md`,
`SANDBOX_TEST_PLAN.md`); this file is what V2 executes and how it is allowed to change.

---

## 1. The stack, and who owns each layer

```
principal                   who is asking — human, agent, robot, service: one kind, never branched on
  ↓ intent / program        the DeluluLang text (or DIR) — the only thing the compiler judges
  ↓ static effects          effect rows: what a function MAY do, in its type            [implemented]
  ↓ static authority        `delulu authority`: what the whole program can request     [implemented]
  ↓ operator grant / lease  `--grant`, `--lease`, the manifest ceiling; ⊑ bounds it     [implemented]
  ↓ Guard                   is this operation or delegation allowed? tiers, permits     [implemented]
  ↓ sandbox policy          derive(authority, grant, profile) → SandboxPolicy           [implemented (PS-A): `delulu sandbox policy`, the report's `policy_hash`]
  ↓ sandbox backend         a launcher and a channel: process, microvm, external        [process and microvm implemented (PS-A, PS-C); external implemented, its wall measured by nobody (PS-D-01)]
  ↓ host effect channel     the guest asks; the host performs                           [implemented (PS-A): `delulu-sandbox-channel/3`, the same host code for L1, L2 and L3]
  ↓ broker / custody        is this request backed by valid custody?                    [implemented]
  ↓ real effect             the host performs it, under containment                     [implemented]
  ↓ audit                   what happened, hash-chained, anchored                       [implemented]
```

Read downward as attenuation. Nothing lower may widen what was decided higher. The sandbox may
reduce environmental reach; it never creates authority the program did not have; a backend never
reinterprets Authority.

## 2. The core rule

The meanings of **Authority, the Guard, effects, capabilities, broker custody, revocation,
attenuation and audit are stable.** V2 may harden them, fix bugs in them, expand them and add
enforcement layers beneath them. V2 may not silently redefine their semantics. Concretely:

- no holder-kind branch anywhere — never `human = trusted, AI = untrusted`, never `robot = special`
  (Constitution §5.16; Stage-5 criterion 9 tests it);
- one semantic authority model for every backend: there is no "authority semantics for the process
  sandbox", "for the microVM", "for the cloud" — the backend changes the enforcement boundary, not
  the language's meaning;
- the sandbox is an enforcement layer underneath the Authority model, not a replacement for it and
  not a competing permission system;
- the Guard is not turned into a sandbox implementation; the sandbox is not turned into a policy
  engine.

The roles, in one line each: **Authority** — what can this program request? **Guard** — is this
operation or delegation allowed? **Sandbox** — what can this execution environment physically
reach? **Broker** — is this request backed by valid custody or lease authority? **Host** — perform the
effect. **Audit** — record what happened.

## 3. The same-user trust boundary (unchanged; category 7)

A process running as the operator's own OS user *is* the operator to the kernel: it can read the
broker's key, edit its policy, kill it, and mint root authority in the LEGACY default (DISC-1). No
code closes this; a separate OS account, container or VM does (`docs/DEPLOYMENT.md` Tier 2, verified
with a real second UID on POSIX). V2 does not pretend that sandboxing alone solves it. What V2 does:
**identity separation becomes a mechanism the toolchain provides where the OS allows an unprivileged
launcher one** — a restricted token or AppContainer on Windows, a mapped uid on Linux where user
namespaces are usable, a VM at L2 — and is reported as present or absent in `host_guarantees`
(PS-B-03). For autonomous deployments the intended shape is *human identity ≠ agent OS identity ≠
broker trust root*, where the deployment model justifies it. The boundary is always principal,
identity, authority, grant, sandbox, attestation and audit — never species. [BUILT on Windows,
PS-B-03: a `--sandbox` guest runs as a per-run AppContainer with no capabilities, measured against
T14 with a control (D-V2-34). macOS gives no second identity; its guest profile refuses the state
directory instead. BUILT on Linux, PS-B-03b, where the host allows user namespaces: the guest runs
as a subordinate uid with no groups or capabilities, T14 measured with a control (D-V2-37); where the
host forbids them (Ubuntu's default) the report says the guest is the same OS user.]

## 4. Resource authority

Today the main program has **no CPU, memory, wall-clock, disk or process bound on either engine**
(NE-22); only Contained plugins have fuel, memory and wall limits (`limits.rs`, Linux live engine).
V2 treats resources as **first-class authority dimensions, not launcher configuration**:

```
Authority = effects + resources + devices + network scope + other enforceable constraints
```

Budgets to investigate and implement where the phase allows: CPU, memory, wall clock, process count,
actor/concurrency count, filesystem quota, network bytes and requests, device use, GPU/device budget
where applicable. Rules: a budget appears in the derived `SandboxPolicy`, in `run --json`, in the
authority report where it is static, and in the audit record; a limit kill re-uses the `limits.rs`
attribution rule and never suggests widening; "never unlimited" is the plugin precedent. **The
mathematics is not changed without documenting and verifying it**: a dimension that can be given a
containment order (`child ⊑ parent` as an interval or count comparison, with a meet) joins the
Z3 model and the exhaustive enumeration; a limit that cannot yet be made a formal dimension stays an
explicit launcher control and is labelled as such, never smuggled into ⊑. [BUILT: PS-B-01 the
launcher control, PS-B-05 the dimension. Memory and processor time joined ⊑ as the tenth dimension
(`budget_scope.rs`, proved in the Z3 model's §2b, `MATHEMATICS.md` §6b); wall time stayed a launcher
control, because a program waiting on its input consumes nothing a delegator hands down.]

## 5. Execution modes and restriction control

Restrictions must be usable in practice and must never become a silent bypass.

**Sandbox control** (the CLI the architecture settles on; today's `--isolation none|process|microvm`
stays and is strengthened):

```
--sandbox                     = the default contained profile
--sandbox=off                 explicit, visible, audited
--sandbox-profile <name>      a named policy: dev, contained, hostile-agent, external:<name>
--isolation <level|backend>   the boundary requested
```

Invariants (each gets a witness and a mutant in PS-A): secure sandboxing is the normal posture where
appropriate and disabling it is explicit; a request for a stronger level **never silently downgrades**
and an unsupported level **refuses** (`DL1408`'s rule, generalized); **a program cannot disable its
own sandbox**; a sandboxed principal cannot grant itself a weaker level; break-glass operations are
initiated outside the sandbox by an authorized external principal. "microVM requested, but a
container was close enough" is never an answer.

**Authority control.** "Authority OFF" is not "the compiler stops caring" — authority is a semantic
property of the program. Instead, three execution modes, each observable in `run --json`, `doctor`
and the audit chain:

| Mode | What it is | Who can enter it |
|---|---|---|
| **STRICT / ENFORCED** | static authority + runtime custody + Guard + sandbox; the default | everyone |
| **AUDIT / DRY-RUN** | perform no effects; compute and report the required authority; show the would-be requests and the policy the run would get | an operator or an agent — it removes nothing |
| **PRIVILEGED / BREAK-GLASS** | explicit external operator action; a separate trust boundary; highly visible; fully audited; **never activated by program code** | an authorized external principal only |

The safest principle, and the one V2 implements: **restrictions can be relaxed by an authorized
external principal, but a program cannot relax its own authority.** Break-glass is an operational
escape hatch, not a change to the language's semantics, and it must look like one: a banner, a
`doctor` line, an audit record, a `--json` field. Program code cannot disable Authority, cannot
disable the Guard, and an agent cannot turn off its own security.

**Transitions that must be tested** (PS-A-10 / PS-B-06; each a witness plus a mutant): ON → OFF;
OFF → ON; strict → audit; audit → strict; break-glass → strict; failed and unsupported transitions;
a transition attempted during execution; after a delegation; after a plugin load; during a sandbox
escape attempt; whether a child process can inherit a weaker policy; whether a revoked authority can
become usable after a transition. **Prefer revocation, kill and restart over hot weakening of a
running sandbox** where that is the safer semantics. The design must keep three things distinct and
name which it is changing: *language semantics*, *execution enforcement*, *operator break-glass
controls*. [BUILT for the sandbox, PS-B-06 (D-V2-35): an operator may require the sandbox on a host,
and break-glass is an Ed25519-signed, single-use, one-program ticket, capped at a day, that relaxes
that requirement and nothing else — banner, run report, audit record (a use that cannot be recorded
does not happen), `doctor` and `sandbox status`. It is an execution-enforcement control; language
semantics and the Guard are never what it changes.]

## 6. Isolation levels — honest labels for boundaries that exist

| Level | Boundary | Platform | State |
|---|---|---|---|
| **L0 `none`** | the language and custody, in-process | all | implemented (today's default) |
| **L1 `process`** | the host kernel: Landlock + seccomp + rlimits + a user cgroup (Linux; no dependence on user namespaces, which CI blocks); restricted token + Job Object (+ AppContainer) (Windows); Seatbelt (macOS); the guest reaches nothing but the channel | Linux, Windows, macOS | implemented as `--sandbox` (PS-A, PS-B); `--isolation process` still isolates **foreign code only** (NE-16b), and says so |
| **L2 `microvm`** | KVM + a VMM (Firecracker under its jailer, Cloud Hypervisor second); the interpreter as `/init` in a kernel-only image; vsock only, no NIC, no filesystem device | Linux + KVM | implemented on x86_64 (PS-C, 2026-09-27) as `--isolation microvm`: Firecracker v1.17.0, a 6.18 kernel built from source with vsock and nothing else, the image checked on the copy that boots; run as root, **under Firecracker's jailer** (a uid of its own, a chroot, a reaper — D-V2-40) and refused as root without one; run as an ordinary user, the VMM is that user and the report says so; `DL1408` where it cannot be given |
| **L3 `external`** | an operator-supplied launcher (Docker + gVisor, Kata, a cloud sandbox, Kubernetes, ssh) carrying the channel over stdio; the level is labelled `external` and the guarantees `unknown` unless attested | wherever the operator runs | implemented as `--sandbox --sandbox-backend external:CMD` (PS-D-01, 2026-09-28, D-V2-46): the command, split on whitespace and run with no shell, carries the channel on its standard input and output (`__guest --stdio-pipes`) and is told the run's limits in its environment; the report says level 3, backend `external`, `fully_enforced: false` and **no host guarantee** (only what the guest measured of itself, RW 4.23), and names the launcher's program, never its arguments; authority is unchanged — the host decides and performs every effect; a Docker + gVisor recipe is documented in `DEPLOYMENT.md`, not shipped and not tested here; **attestable since PS-D-02** (2026-09-28, D-V2-48): `--require-attestation HEX` refuses to send the program until the launcher's attester has signed a statement over this run's nonce with the pinned key, and the report carries its claims as `sandbox.attestation` — the attester's, never merged into `host_guarantees`; the level stays 3 |
| **L4 `attested`** | L2/L3 whose guest attests the pinned image before any lease is delegated | specific hardware | deferred; the seam is designed with a fake attester (PS-D-02) |

**NVIDIA OpenShell as an L3 backend** is designed (PS-E-05, `V2_OPENSHELL_STUDY.md` §4.5): the guest
inside an OpenShell sandbox whose policy has no network rule, reached through `external:`; the level
stays 3 and its guarantees unknown unless attested. In the other direction, `delulu sandbox policy
--format openshell` will emit the OpenShell policy a program's authority implies — never wider.

**Machine-readable output** (the run report that `run --json --report-out <path>` writes, never the program's own standard
output, which the program could forge (D-V2-21); `delulu sandbox probe --json`
before running; `doctor` for the host) reports: requested level, actual level, backend, host
capabilities, guarantees, limitations, resource limits and what remains, network posture,
filesystem posture, identity posture, sandbox state, whether restrictions are fully enforced, and
whether break-glass is active. Never a guarantee that was not measured by an *attempt* (create the
ruleset, open `/dev/kvm`, apply the job limit) — a version string is not a probe. [designed;
PS-0-02/04/05, PS-A-07]

## 7. The microVM principle

The guest **performs no effects**: capabilities are opaque handles, the guest holds no secret bytes
and no broker address, and every effectful primitive is a bounded, versioned channel request the host
authorizes with today's custody, Guard, containment and audit code before performing it. The microVM
is the same guest behind a hypervisor, not a second design and not a second Authority
implementation. **The guest runs the interpreter, not the WASM engine** — an owner-approved V2
decision (D-NE-23; a ruled deviation from Stage 5 §6, because the WASM backend compiles 6 of 19 entry
programs and a tier that refuses most programs is a tier in name). **Before PS-C implements it, every
practical implication is verified against the actual code and backend capabilities**: the `EffectSink`
seam must exist and keep the local path byte-identical (the core-invariance snapshot), the channel
must be fuzzed, the image build must be reproducible, and a prerequisite the code proves missing stops
the phase rather than being papered over. [designed]

## 8. `SandboxPolicy` — a first-class formal object

Sandboxing is not a CLI flag. The policy is a pure function `derive(authority, grant, profile)`
producing a machine-readable object with: requested level, actual level, backend, filesystem posture,
network posture, secrets rule, devices, resources, identity, channel, guest, guarantees,
limitations, revocation state. It is byte-stable for identical inputs (pinned like the core-invariance
snapshot for the guide corpus), **explainable** (`delulu sandbox policy <file> --json`), **hashable**
(the hash goes into the `sandbox-launch` audit record), **auditable** and **diffable**. [designed;
PS-A-06]

## 9. Physical AI

A future actuator path is `program → Cap[Actuator] → Authority → Guard → Sandbox → host adapter →
safety envelope → device`. No raw device access because the caller is a robot; no "robot authority"
or "AGI authority" as a special category; the adapter and the dead-man watchdog stay host-side
(invariant 52) and the control program runs in a guest. Today the adapter is an operator-supplied
subprocess whose detached signature is checked before it is spawned (D52/D53, optionally pinned; the
file checked is the file started since ADAPTER-SPELL-1, 2026-09-28), and no real driver ships (RW
4.7); the signed Verified-class adapter is P8. [partially implemented: envelopes, dead-man, e-stop and revocation are implemented
and measured against the simulator; the signed adapter is designed]

## 9b. What the study of NVIDIA OpenShell changes (2026-09-28)

`V2_OPENSHELL_STUDY.md` compares the two designs line by line. The rules this model states are
unchanged; four of their enforcements get stronger, as phase PS-E (D-V2-53) and P9 (D-V2-55):

- **"Never a guarantee that was not measured" becomes a type** (PS-E-01): the program is sendable
  only from a `Confirmed` state built from five measured properties, bound to a per-run generation; a
  profile's required property that is absent refuses the run before the program is sent. *Built so far
  (D-V2-56): the order and the generation — `delulu-sandbox-channel/3` sends the program only through
  `boundary.rs`'s `Confirmed`; the properties and required sets are next.*
- **"The guest reaches nothing but the channel" is enforced beyond TCP** (PS-E-03, if its witness is
  red): Landlock mediates TCP only, so a guest that escaped the interpreter could still open UDP,
  netlink or Unix sockets on Linux; the guest needs none after lock-down, so `socket` is denied.
- **"The guest ends with its host" holds on every backend** (PS-E-02): Linux by the death signal (the
  external launcher too, since 2026-09-28), Windows by the job, macOS by a watcher outside the guest (the
  guest and the launcher, 2026-09-29, D-V2-60), and the Windows external launcher by a kill-on-close job
  (the same day). What a launcher starts outside itself stays the launcher's.
- **Authority checked at a boundary** (P9-01): the `⊑` the lease tree enforces, asked of a whole
  program before it runs, with the source line of anything that exceeds.

## 10. What is and is not claimed

Claimed today, with witnesses: zero ambient authority; attenuation-only delegation; transitive
revocation with a stated latency bound; the hash-chained, anchored audit log (modification,
reordering and truncation detected; an attacker who rewrites log and anchor is not); Guard permits
rather than bearer codes; filesystem containment through resolved paths, opened without following any
link since P5c (FS-RACE-1 closed the check-then-open race, CONTAIN-TOCTOU-1); the WASM in-process
floor; the separate-OS-account boundary on POSIX.

The sandbox is claimed level by level, each with its witnesses in `V2_LOG.md`, and every run's report
says what THAT host applied: **L1** (`--sandbox`) since PS-A and PS-B on Windows, Linux and macOS, as
the per-platform table in `docs/DEPLOYMENT.md` states it; **identity separation** since PS-B-03 (a
per-run AppContainer on Windows) and PS-B-03b (a subordinate uid on Linux, where the host allows user
namespaces); **L2** (`--isolation microvm`) since PS-C on Linux x86_64 with KVM, with the hostile-guest
red-team gates; the ceilings that stop a guest named from evidence (SANDBOX-STOP-1,
SANDBOX-CPU-LATE-1).

Egress control is claimed since PS-B-02, with witnesses and falsified mutants
(`crates/delulu-runtime/src/egress/tests.rs`, `crates/delulu/tests/egress_cli.rs`): one host-side
client for L0 and guests; the allowlist; resolve-once-and-pin with every candidate address
classified; special-use refusal unless `net.special=`; redirects re-checked; the response bound;
certificate verification against the platform store; proxy variables ignored; no resolver reachable
by the client itself.

The Linux guest's own layers are claimed against an ESCAPED guest since PS-E-03 (D-V2-61), with
witnesses and six falsified mutants (`crates/delulu/src/jail.rs`, `escaped_tests`): after it locks itself
down, a guest that has escaped its interpreter opens no socket of any family (GUEST-SOCKET-1), reads no
other process's `/proc` entry (GUEST-PROC-1) and no device but null, zero, full and the random ones
(GUEST-DEV-1), and makes none of the unnamed calls or a namespace through `clone` (GUEST-SYSCALL-1); and
a serving Linux host is closed to its own user — non-dumpable once its guest is launched
(HOST-DUMPABLE-1, D-V2-62); an escaped guest cannot type into the operator's terminal (GUEST-TIOCSTI-1,
D-V2-63); it signals no process but itself (GUEST-SIGNAL-1, D-V2-64) and changes no other process's
limits, priority, CPUs or scheduling (GUEST-PROCESS-1, D-V2-65). A report claims file writes denied on
Linux only where truncation is refused too (Landlock ABI ≥ 3, LANDLOCK-TRUNCATE-1, D-V2-66). Not yet: the Landlock ABI as a requirement (H5), macOS and Windows under the
same harness (H6).

An external (L3) launcher is claimed resolved once and named by its bytes since PS-E-04 (D-V2-69), with
witnesses and six falsified mutants (`crates/delulu/tests/sandbox_external_cli.rs`): a bare name is never a
file in the working directory (LAUNCHER-SPELL-1); the report's `launcher_blake3` is the file's; a pinned run
starts no other file; and on Linux a path swapped between the hash and the start is not what runs. Not
claimed: that window on macOS and Windows, or an in-place change by someone who may write the file.

A program's strings are claimed to reach the operator's terminal only escaped since TERMINAL-TEXT-1
(D-V2-70), with witnesses on five paths and six falsified mutants (`crates/delulu/tests/terminal_text_cli.rs`):
an assertion's values, a refusal's quoted path and suggested grant, a test's name, a quoted source line, and
a sandboxed guest's own fault line. Not claimed: the output of a guest that has escaped its interpreter,
which still reaches the terminal raw (RW 4.32), and a line break inside a diagnostic's message (RW 4.34).

A resource bound on the main program is claimed since PS-B-01, with its witnesses and a falsified
mutant (`crates/delulu/tests/budget_cli.rs`): D-V2-25's 1 GiB and 5 minutes by default, enforced by a
host watchdog on every engine at a stated 25 ms resolution.

Not claimed until its witness is green and its mutant is red: containment a host did not measure (a
level or guarantee a run's report does not list); content inspection of permitted traffic (a program may send
anything to a host it was granted — category 7, and the grant is the control);
identity separation where the host forbids it (macOS; Linux without user namespaces); anything about
an L3 environment — PS-D-01 reports it as unmeasured, and PS-D-02's attestation seam carries only
what the attester signs; anything about OpenShell's enforcement (designed as an L3 backend, PS-E-05, not yet run);
multi-tenancy on one OS user (never). Kernel and hypervisor exploits and side channels are category 7 at every level.
