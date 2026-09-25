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
  ↓ sandbox policy          derive(authority, grant, profile) → SandboxPolicy           [designed]
  ↓ sandbox backend         a launcher and a channel: process, microvm, external        [designed; `microvm` is a probe that refuses]
  ↓ host effect channel     the guest asks; the host performs                           [designed; the WASM host and the foreign worker are the precedents, implemented]
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
directory instead. Linux under a subordinate uid is RW 4.21.]

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
controls*. [designed]

## 6. Isolation levels — honest labels for boundaries that exist

| Level | Boundary | Platform | State |
|---|---|---|---|
| **L0 `none`** | the language and custody, in-process | all | implemented (today's default) |
| **L1 `process`** | the host kernel: Landlock + seccomp + rlimits + a user cgroup (Linux; no dependence on user namespaces, which CI blocks); restricted token + Job Object (+ AppContainer) (Windows); Seatbelt (macOS); the guest reaches nothing but the channel | Linux, Windows, macOS | designed — today `process` isolates **foreign code only** (NE-16b); strengthened and announced in PS-A |
| **L2 `microvm`** | KVM + a VMM (Firecracker under its jailer, Cloud Hypervisor second); the interpreter as `/init` in a kernel-only image; vsock only, no NIC, no filesystem device | Linux + KVM | designed — today a probe that refuses everywhere (`DL1408`); PS-C |
| **L3 `external`** | an operator-supplied launcher (Docker + gVisor, Kata, a cloud sandbox, Kubernetes, ssh) carrying the channel over stdio; the level is labelled `external` and the guarantees `unknown` unless attested | wherever the operator runs | designed; PS-D |
| **L4 `attested`** | L2/L3 whose guest attests the pinned image before any lease is delegated | specific hardware | deferred; the seam is designed with a fake attester (PS-D-02) |

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
subprocess with no signature check and no real driver ships (RW 4.7); the signed Verified-class
adapter is P8. [partially implemented: envelopes, dead-man, e-stop and revocation are implemented
and measured against the simulator; the signed adapter is designed]

## 10. What is and is not claimed

Claimed today, with witnesses: zero ambient authority; attenuation-only delegation; transitive
revocation with a stated latency bound; the hash-chained, anchored audit log (modification,
reordering and truncation detected; an attacker who rewrites log and anchor is not); Guard permits
rather than bearer codes; filesystem containment through resolved paths (with the TOCTOU residual
named); the WASM in-process floor; the separate-OS-account boundary on POSIX.

Egress control is claimed since PS-B-02, with witnesses and falsified mutants
(`crates/delulu-runtime/src/egress/tests.rs`, `crates/delulu/tests/egress_cli.rs`): one host-side
client for L0 and guests; the allowlist; resolve-once-and-pin with every candidate address
classified; special-use refusal unless `net.special=`; redirects re-checked; the response bound;
certificate verification against the platform store; proxy variables ignored; no resolver reachable
by the client itself.

A resource bound on the main program is claimed since PS-B-01, with its witnesses and a falsified
mutant (`crates/delulu/tests/budget_cli.rs`): D-V2-25's 1 GiB and 5 minutes by default, enforced by a
host watchdog on every engine at a stated 25 ms resolution.

Not claimed until its witness is green and its mutant is red: any L1 or L2 containment; content
inspection of permitted traffic (a program may send
anything to a host it was granted — category 7, and the grant is the control);
identity separation provided by the toolchain; anything about an L3 environment; multi-tenancy on
one OS user (never). Kernel and hypervisor exploits and side channels are category 7 at every level.
