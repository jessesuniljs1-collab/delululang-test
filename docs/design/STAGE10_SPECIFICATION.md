# DeluluLang — Stage 10 Implementation Specification

**Version:** 1.x ("Industrial") — the production-readiness stage beyond v1.0.
**Status:** Committed. Unlike Stages 1–9, Stage 10 is a **program of parallel tracks**, each
independently shippable in a 1.x minor, each RFC-visible, none changing v1.0 semantics except
where an activation was explicitly reserved (attributes, `Actuate`, threads-in-WASM).
**Revision 2 (2026-07-19, at the owner's direction):** scope extended from five tracks to eight.
New: **Track F** (heterogeneous compute — any vendor's GPU/TPU/NPU behind one authority model),
**Track G** (post-quantum cryptography — hybrid, KAT-gated, and never called "quantum-proof"),
**Track H** (cloud, fleets, and infrastructure — deployment is a grant). **Track D is generalized**
from the arm to the autonomy domains — road vehicles, aircraft and UAS, spacecraft and satellites,
robot fleets — with energy systems (batteries/BMS) and safety mechanisms (e-stop chains,
interlocks, watchdogs) as first-class device classes; domain depth lives in
`STAGE10_AUTONOMY_ADDENDUM.md`. The owner's charge, recorded: *DeluluLang is to be a key pillar of
a safe and secure autonomous future — the language in which the command layers of vehicles,
aircraft, satellites, and robots are written and modified, by humans and by AI, under authority
they cannot exceed.* Rev 2 turns that charge into tracks, invariants, and criteria; it changes no
v1.0 semantics and claims nothing built.
**Depends on:** Stage 9 (v1.0 released; stability contract in force; measurement baseline
published). *Status note (updated 2026-07-20): **v1.0.0 is RELEASED** — the rc.1 gate's two
blockers closed (D22 coverage 287/287 hard-gated; the D9 first-run drill recorded), gate re-run
green, artifact cut and signed. The dependency is satisfied; Stage 10 build work may begin. The
stability contract is now in force: everything Stage 10 adds is additive.*
**Governing documents:** `CONSTITUTION.md` (§5.11 execution modes, §5.12 interop, §7 robotics,
§9 honesty), `SOUNDNESS_AUDIT.md`, `STAGE10_AUTONOMY_ADDENDUM.md` (normative for Track D domain
scope).

---

## 0. Scope and goal — what "production-ready DeluluLang" concretely means

Production-ready is a checklist, not a vibe. DeluluLang is production-ready when **all** of the
following are true, each verified by a named criterion in §11:

P1. **Performance:** measured competitive-with-C on the published hot-path suite (geometric mean
    within 2.5× of C on compute kernels via the optimizing backend; the honest target from
    Constitution §5.11 — and if a workload misses, the published table says so).
P2. **Concurrency at scale:** multi-threaded execution on both engines; the Stage-7 leak and
    backpressure debts paid (cycle collection, bounded mailboxes).
P3. **Execution-mode policy:** `@aot`/`@interpret`/`@jit` hints active under the two inviolable
    rules (modes never weaken the sandbox; native-code emission is human-policy-gated).
P4. **Physical stakes:** the robotics/embodied profile (`Actuate`) shipped with its dead-man
    semantics, sim-to-real workflow, and a public simulated demonstration.
P5. **Operational maturity:** LTS releases, CVE process exercised for real, upgrade paths
    proven, 24-month-support guarantees in writing.
P6. **Ecosystem viability:** the registry carries independently-authored packages and plugins;
    at least one third-party catalog locale; the agent-harness page (`for-agents.md`) adopted by
    at least two independent harnesses.
P7. **Heterogeneous compute:** any vendor's accelerator — GPU, TPU, NPU — behind one vendor-neutral
    authority model: enumerated, envelope-bounded, and honestly labeled outside the proof (§7).
P8. **Post-quantum cryptography:** hybrid, KAT-validated, crypto-agile signatures and transport
    (§8) — the registry's trust built to outlive the arrival of a cryptographically relevant
    quantum computer (retroactive forgery for signatures; harvest-now-decrypt-later for
    transport).
P9. **Cloud and fleets:** a deployment's whole authority computed and approved before launch;
    fleet updates signed, staged, and hash-gated (§9).
P10. **Autonomy domains:** the autonomy generalization *specified* for vehicles, aircraft,
    satellites, and robot fleets with energy systems and safety chains as device classes; each
    domain's honest boundary reviewed line-by-line; the satellite domain *witnessed* in
    simulation; and the broker-federation gap named, not papered over (§5.6, criterion 10,
    `STAGE10_AUTONOMY_ADDENDUM.md` §2.5).

**In scope:** the eight tracks below (§2–§9), DL19xx diagnostics (§10), LTS machinery (§4).
**Non-goals (RFC-gated future, recorded so nobody claims them early):** distributed actors;
per-plugin microVMs; per-actor broker nodes; I/O-quota grant dimensions; effect handlers;
`await`; information-flow taint beyond `Secret`; certified/mechanized compiler.

---

## 1. Invariants (carried + new)

All prior invariants hold — Stage 10 is where they earn their keep. New:

45. **Mode-sandbox invariance, now executable.** For every execution mode (interp, AOT-WASM,
    optimized-native, JIT-tier), the *same* conformance and laundering suites pass with the
    *same* diagnostics and traces. A mode that cannot pass the full suite does not ship as a
    mode (it may ship as an experiment behind `--unstable`).
46. **Native-code emission is a grant.** No module JITs or runs natively-emitted code unless its
    grant carries `exec.native = true` — issued like any authority (broker node data, ⊑-checked,
    audited, revocable). Default for everything is off; the Stage-1 promise "only
    human-controlled policy may grant native code" becomes mechanism here.
47. **An actuator lease is a dead-man switch.** Every `Cap[Actuator]` lease has a mandatory TTL
    and heartbeat; a missed heartbeat revokes it (broker-side, synchronous class) — a hung or
    partitioned program *loses* physical authority by default rather than keeping it.
48. **Sim and real are the same program.** The sim-to-real transition changes only the broker
    profile and grants — a source or artifact diff between what was simulated and what is
    deployed is detectable (artifact hash comparison is part of the deploy flow, §5.4).
49. **No privileged vendor.** Device and cloud-provider access goes through adapters implementing
    vendor-neutral interfaces; no adapter carries semantics the interface cannot express. The day
    one vendor's chip or cloud needs privileged hooks is the day the authority model has a second
    class of citizen — refused by construction.
50. **A kernel is a foreign call with an envelope.** Compute dispatch is typed with the existing
    core `ForeignCall` effect — outside the proof, honestly labeled in `delulu authority` — and
    bounded by a device envelope (memory, kernel time, queue depth, power) enforced host-side
    and, where the adapter can enforce it, adapter-side; an adapter that cannot attest
    independent below-adapter enforcement is refused the grant (DL1911; waiver is human-gated),
    so double enforcement is never silently single. DeluluLang closures never become kernels;
    kernels are hashed, signed *data*.
51. **Hybrid or nothing; KAT or `--unstable`.** Post-quantum signatures and KEM ship only in
    hybrid with the classical algorithms v1.0 already trusts, and reach stable only after
    byte-exact validation against the official known-answer vectors. No product surface says
    "quantum-proof" (§8.3).
52. **The safety chain survives DeluluLang's death.** Hardware e-stops, interlocks, watchdogs, and
    BMS/firmware protection must function with the DeluluLang layer absent, hung, or compromised.
    DeluluLang supervises *above* that chain and revokes *toward* it; it never replaces it, and no
    deployment profile may route a hardware safety function through a DeluluLang program.
53. **No plan, no launch.** A deployment — cloud service, fleet update, or vehicle mission — runs
    only after its whole-deployment authority answer is computed and approved against its
    environment profile (§9.2). Deploy-time is compile-time for infrastructure.

---

## 2. Track A — Performance and execution modes

### 2.1 Optimizing backend

Cranelift-optimized Wasmtime tier plus a DIR-level optimizer (inlining across package
boundaries — legal because rows are declared and checked, so inlining cannot change authority;
monomorphization of generics; escape analysis to unbox locals). Continuous benchmarking: the
Stage-9 Study-C suite runs per-commit with regression gates (>3% geo-mean regression blocks
merge).

### 2.2 Attributes (grammar activation)

```ebnf
attribute = "@" , IDENT , [ "(" , STRING , ")" ] ;
fn_decl   = { attribute } , "fn" , … ;        (* also on actor_decl, module_decl *)
```

v1.x-defined attributes: `@aot`, `@interpret`, `@jit`, `@inline(never|always)` — **hints**, all
of them; the scheduler may ignore them; none changes semantics (invariant 45 is the test).
Unknown attributes: DL1901 (error — no silent vendor attribute space; extensions go through
RFCs).

### 2.3 JIT policy mechanics

`exec.native` in grants (invariant 46); manifest `[authority] exec.native = true` required for a
package to *request* it; `delulu authority` reports it as a distinct line (`native code
emission: granted/denied`). Untrusted agent code default: denied — the run prompt shows it in
red-tier wording. The JIT tier runs the same host-side scope checks (they were never in guest
code — Stage-3 architecture pays off here: there is nothing for a JIT to "optimize away").

### 2.4 Multi-threaded WASM engine

Wasmtime threads + shared-everything-GC per its maturity at implementation time; actor scheduler
ported; the Stage-7 TSAN/parity criteria re-run on this engine. If the proposal stack is not
production-ready when this track lands, the track *waits* and says so (mode honesty beats mode
count).

## 3. Track B — Memory and actor-runtime maturity

- **Per-actor cycle collection** (trial-deletion collector over the actor's Rc heap, runs
  between turns): closes the Stage-1/7 leak note. Leak test corpus (cyclic graphs, promise
  chains) goes from "documented leak" to "collected."
- **Bounded mailboxes:** `actor A(mailbox = 10_000)` manifest-side default + per-spawn override;
  overflow policy is explicit: `block` (default — backpressure by suspending the *sending turn*
  at the send site, which is safe because sends are the turn's last-resort suspension point and
  deadlock remains a documented non-guarantee) or `drop-new` (counted, reported). DL1902 on
  drop-in-`abort`-mode.
- Actor heap telemetry in traces (`--trace-memory`): per-actor bytes, collections, drops — the
  ops surface.

## 4. Track C — LTS and security operations at scale

- **Release trains:** 1.x minors every ~12 weeks; **LTS** designation every 4th minor with
  24-month security backports; the support matrix is a published table.
- CVE process: CNA registration or partner CNA; advisories tie to DL-coded detectors where
  applicable (a vulnerable-version *use* can be flagged by `delulu build` via the registry's
  advisory feed: DL1903 warning, `--deny-advisories` gate for CI).
- Broker/protocol/DIR major-version co-evolution policy: one page, published, with n−1 majors
  supported concurrently during LTS windows.

## 5. Track D — The embodied/autonomy profile (`Actuate` activates)

### 5.1 Model (Constitution §7, now mechanism)

New capability kinds `Actuator`, `Sensor`; new synchronous-class effect `Actuate` (reads from
sensors are `Read` with sensor scopes). An actuator capability's **scope is its envelope**:

```json
{ "kind": "Actuator",
  "scope": { "device": "arm0/elbow", "kind": "revolute",
             "envelope": { "torque_nm": [0, 2.5], "angle_deg": [-30, 95],
                           "velocity_dps": [0, 40], "duty_pct": [0, 60] },
             "rate_hz": 50, "heartbeat_ms": 200, "ttl_ms": 30000 } }
```

```delulu
fn extend(elbow: Cap[Actuator], deg: Float) -> Result[Unit, ActuateErr] ! {Actuate} {
  elbow.command(Move { angle_deg: deg, velocity_dps: 20.0 })
}
```

Every `command` is validated against the envelope **twice**: host-side (broker/backend, before
the wire) and device-adapter-side where the adapter supports it. Envelope violations are refused
as `ActuateErr::Envelope` (runtime DL09xx-class, plus DL1904 telemetry record) — the program
keeps running; the *command* dies, not the process (a control program that aborts on a bad
setpoint is itself a hazard).

### 5.2 Dead-man semantics (invariant 47)

Actuator leases: mandatory `heartbeat_ms` and `ttl_ms`; the runtime heartbeats automatically
while the holding actor's turns are healthy; a missed beat → broker revokes → the **device
adapter's mandated fail-state** engages (declared per device at grant time: `hold`, `coast`, or
`safe-park`). E-stop is `delulu grants revoke` on the actuator subtree — same mechanism as
everything else, now with a measured latency budget: revoke-to-fail-state ≤ heartbeat_ms +
adapter latency, published per adapter.

**What beats a lease, and what merely costs it time (normative; ruling D43a).** Only an operation the
broker *accepted* beats a lease. A refused command does not beat one — but it is still an
interaction, and under the stepped clock (`--sim-step`, ruling D20) it advances simulated time exactly
as an accepted one does. Without that, a program whose every command was refused froze simulated time
and held its device indefinitely, while the same program on the wall clock lost it: the two clocks
must agree about when a machine stops moving, or a simulation cannot rehearse the case that matters.
When the sweep triggered by a refused attempt is what kills the lease, the holder is told it lost the
device — that fact outranks the setpoint complaint that would otherwise have been reported.

The dead-man defends against **silence, not malice**, and this section does not claim otherwise: a
controller that keeps interacting keeps its lease however wrong its commands are, and operator
revocation (§5.2's e-stop) is the answer to a program that is alive and misbehaving.

### 5.2.1 The device grant grammar (normative; ruling D43b–d)

`DEVICE:dim=lo..hi[,dim=lo..hi…][,rate_hz=N],heartbeat_ms=N,ttl_ms=N,fail=STATE`

Two independent parsers read this one grammar — the broker's, which builds the authority that is
recorded, delegated, attenuated and audited, and the runtime's, which builds the capability value
enforced against each command. **They must accept and refuse exactly the same strings**, and that is a
mechanically enforced law, not an aspiration: an envelope legal to one and not the other is either a
grant no program can use or a bound no authority recorded. Three rules carry that:

1. **Every bound is a finite real interval.** `inf`, `-inf`, `infinity` and `NaN` all parse as `f64`
   in the host language and are all refused. An infinite bound admits every command while looking like
   a bound, and a NaN bound makes every comparison in the lattice meaningless.
2. **A term stated twice is refused, never resolved.** Not the first occurrence, not the last: an
   ambiguity about a physical bound has no safe resolution, because whichever is chosen, a reader of
   the other is wrong. This is the rule §3.1 already applies to two envelopes for one device, applied
   to two statements of one term.
3. **`fail` names one of exactly `hold`, `coast`, `safe-park`.** There is no default and no
   near-match: what a machine does when authority ends is not something a parser may guess.

### 5.3 The honest boundary (normative, verbatim in docs)

DeluluLang commands the **policy/command layer (1–100 Hz)**. Servo loops, torque control,
electrical protection, and hard real-time run in firmware/RTOS below the adapter — DeluluLang
sets *setpoints within envelopes*; it does not close 1–10 kHz loops, and no marketing sentence
may imply otherwise. The envelope is enforced above *and* below: even a compromised DeluluLang
stack cannot exceed what the adapter/firmware enforces independently — defense in depth at
physical stakes.

### 5.4 Sim-to-real workflow

`delulu run app.dwx --broker-profile sim` (reference simulator backend ships in-tree: kinematic
arm + sensor models, deterministic under `--seed`) vs `--broker-profile hw:<adapter>` (adapters
are Stage-6 plugins — Verified class required for adapters, `require_signed: true` mandatory).
The deploy flow records artifact hash at sim sign-off and refuses a differing hash at hw grant
time without explicit re-approval (invariant 48; DL1905).

### 5.5 The demonstration (public deliverable)

An agent (scripted LLM harness) live-edits a simulated arm's control program: correct edits take
effect at rate; an edit commanding 5 N·m against a 2.5 N·m envelope is refused per-command; a
harness that stops heartbeating loses the arm to `safe-park`; an operator e-stop revokes the
subtree. Recorded, reproducible (`measurements/robotics-demo/`), and honest about being
simulation.

### 5.6 Beyond the arm — the autonomy domains (Rev 2; addendum-governed)

The same four mechanisms — envelope-scoped capabilities, dead-man leases, declared fail-states,
and the sim-to-real hash gate — generalize from the arm to every domain where a software command
matters physically: **road vehicles, aircraft and UAS, spacecraft and satellites, and robot
fleets**, with **energy systems** (batteries/BMS, power distribution) and **safety mechanisms**
(e-stop chains, interlocks, watchdogs) as first-class device classes, and **microcontrollers and
accessories** as devices you flash and talk to (§7.3). Domain profiles — device classes, command
rates, fail-state vocabularies, link models (a satellite's contact window *is* a lease TTL), and
each domain's honest boundary — are specified in `STAGE10_AUTONOMY_ADDENDUM.md`, which is
normative for Track D domain scope. Two rules travel with every domain: the honest boundary
(§5.3) scales — DeluluLang is the command/mission layer above certified firmware, never the servo
loop, never the airworthy autopilot, never the BMS cell protection; and invariant 52 — the
hardware safety chain must not depend on DeluluLang existing. One gap is named rather than
papered over: the cross-machine broker federation the satellite and fleet profiles imply is
RFC-gated future work, not a claimed capability (addendum §2.5); Stage 5's broker is local-only,
and criterion 10's demonstration runs both broker roles in one simulated host and says so.

## 6. Track E — Ecosystem

Registry growth mechanics (curated "authority showcase" list — packages notable for *minimal*
authority), third-party adapter/catalog support channels, `for-agents.md` versioned as an API,
and the deprecation of nothing (v1 stability holding is itself the deliverable).

## 7. Track F — Heterogeneous compute (every chip, one authority model)

### 7.1 The model — a device is a capability; a kernel is a foreign call with an envelope (invariant 50)

New capability kind `Compute`. A `Cap[Compute]`'s **scope is its device envelope**:

```json
{ "kind": "Compute",
  "scope": { "device": "gpu0", "class": "gpu", "adapter": "vendor-x",
             "envelope": { "memory_bytes": 2147483648, "kernel_ms": [0, 50],
                           "queue_depth": 32, "power_w": [0, 120] },
             "formats": ["ptx-8", "spirv-1.6"] } }
```

Dispatching a kernel is typed as what it is: **foreign code** (Constitution §5.12). It carries the
existing core `ForeignCall` effect — no new effect, no constitutional change — and appears in
`delulu authority` under the outside-the-proof separator: `foreign: compute/gpu0 [kernels…]`.
DeluluLang verifies *reachability* (no dispatch without the capability, the manifest entry, and
`ForeignCall` in every row on the path) and enforces the *envelope* (memory ceiling, kernel time
budget, queue depth, power/duty where the adapter can enforce it) host-side before submission and
adapter-side where supported — the §5.1 double-enforcement rule, applied to silicon (over-envelope
dispatch → refused, DL1907, the command dies and not the process; an adapter that cannot attest
independent below-adapter enforcement is refused the grant, DL1911, human-gated waiver — so the
double claim is never silently single). What the kernel *computes* is
outside the proof, and the docs say so. A dedicated `Dispatch` effect distinguishing accelerator
dispatch from other foreign calls is RFC-gated future work, recorded here so nobody claims it
early.

**Kernels are data, never code from the row system** (the spirit of audit rule R-6a): a DeluluLang
closure never becomes a kernel; kernels arrive as opaque artifacts (PTX, SPIR-V, vendor blobs)
named in the manifest, hashed, and signed like any artifact. The host drives; devices get buffers.

### 7.2 Vendor neutrality (invariant 49)

Device access goes through **compute adapters** — Verified-class, `require_signed: true` Stage-6
plugins implementing one vendor-neutral interface: enumerate, allocate-within-envelope, submit,
await, telemetry. CUDA, ROCm, oneAPI, Metal, Vulkan-compute, and TPU runtimes each live behind an
adapter; **no vendor's adapter may have privileged semantics** — anything one adapter can express,
the interface must express. An in-tree **CPU reference adapter** (deterministic, no hardware
required) makes the interface conformance-testable on every CI run, with or without silicon.

### 7.3 Microcontrollers are devices, not accelerators (the honest taxonomy)

An MCU is not something you dispatch kernels to; it is a device you **flash and talk to**. MCU
support therefore lives in the embodied profile (Track D): firmware images are signed artifacts;
flashing is an actuation-class operation behind an explicit grant with the DL1905 hash gate;
message exchange with running firmware is `Read`/`Write` under device scopes. Anything beyond
this — compiling DeluluLang itself to bare-metal MCU targets — is RFC-gated future work, not a
Stage-10 deliverable, and no doc may imply otherwise.

## 8. Track G — Post-quantum cryptography (signatures that outlive the machines that made them)

### 8.1 Why now

A signed artifact is a claim addressed to the future, and "harvest now, decrypt later" is an
attack mounted from it: transport recorded today is broken the day a cryptographically relevant
quantum computer exists, and Shor's algorithm breaks the discrete-log problem under both X25519
and ed25519. The registry's artifacts and transport therefore move to post-quantum algorithms
*before* that day, not after.

### 8.2 The mechanism — hybrid, agile, honest (invariant 51)

- **Signatures:** ed25519 **and** ML-DSA-65 (FIPS 204), hybrid. The signature envelope carries
  both, with explicit algorithm identifiers (crypto-agility: the envelope names its algorithms so
  they can be replaced without a format break). Under hybrid-required policy **both must verify**;
  an artifact carrying classical-only, or an unknown algorithm id, → **DL1908**.
- **Transport:** ML-KEM-768 (FIPS 203) hybridized with X25519 for the registry channel — the
  session stays secure if *either* assumption holds.
- **Never PQ-only.** Lattice cryptanalysis is younger than curve cryptanalysis; hybrid means the
  guarantee is never weaker than what v1.0 already ships.
- **KAT or `--unstable`.** A PQC implementation ships as stable only after byte-exact validation
  against the official NIST known-answer vectors, recorded in the build order with vector
  provenance; until then, every invocation without `--unstable` → **DL1910**. An implementation
  that merely round-trips its own output has proven interoperability with itself, which is not a
  property anyone needs.
- **House rule 5 outranks dependency austerity: cryptography is never hand-rolled.** The recorded
  precedent is `ed25519-dalek` (v2) — a vetted implementation taken as a dependency, not an
  in-house one — and ML-DSA/ML-KEM are constant-time lattice code, the highest-risk category
  there is. The default is a vetted, KAT-validated implementation adopted under a build-order
  ruling that records the vetting; writing lattice cryptography in-tree is the extraordinary path
  and would itself need a ruling nobody should expect to win. The in-tree effort goes where it
  belongs: the envelope format, the policy gates, and the tests. Zeroization and constant-time
  discipline rules carry over from the Stage-8 signing integration unchanged.

### 8.3 The vocabulary rule (binding; §12)

The words **"quantum-proof"** and **"quantum-safe"** never appear in any product surface **as a
claim** — they may appear only inside prohibition or honesty-caveat sentences whose purpose is to
ban or correct the term (this section, invariant 51, the §12 caveats, and their mandated copies).
The honest term is **post-quantum**: standardized algorithms (FIPS 203/204) believed resistant to
known quantum attacks — a judgment about current cryptanalysis, not a proof. "Proof" claims about
cryptography are exactly the overclaim the Constitution's honesty clauses exist to prevent, and
the hybrid construction (§8.2) exists *because* the judgment is young.

## 9. Track H — Cloud, fleets, and infrastructure (deployment is a grant)

### 9.1 The model — no new semantics, the same authority story at datacenter scale

Cloud provider APIs are network resources: `Cap[Http]` scoped to provider endpoints, wrapped by
**provider adapters** (Verified-class plugins) that type the operations (create-instance,
put-object, …) so a program's cloud reach reads off its authority report like everything else. No
new effects; no privileged provider (invariant 49 applies to clouds exactly as to chips).

### 9.2 The deploy plan is an authority manifest (invariant 53)

`delulu deploy plan` computes, for a deployment (program + manifest + environment profile), the
**whole-deployment authority answer before anything runs**: effects, capability scopes, and
foreign holes, per service. Environment profiles (`envs/prod.toml`) declare the maximum authority
a deployment may hold there; a plan exceeding its profile → **DL1909** and the deploy refuses.
"What can this deployment do to my cloud account?" gets the same mechanical answer as "what can
this function do?" — before launch, not in the postmortem.

### 9.3 Fleets and updates

Fleet/OTA updates ride the existing machinery: artifacts signed (hybrid, once Track G lands),
staged rollout with health gates, and the approved-hash rule (DL1905) generalized — a fleet never
receives an artifact whose hash differs from what was approved, without explicit human
re-approval. Rollback artifacts are pinned at rollout start; an update that cannot be undone is an
outage with extra steps.

### 9.4 The honest boundary

DeluluLang bounds **its programs'** authority over cloud APIs. The provider's control plane, IAM,
billing, and hypervisor are layers it does not control and does not claim — a DeluluLang deploy
plan is least-privilege *input* to provider IAM, never a substitute for it. And this track
**deploys** programs; it does not distribute the actor runtime — distributed actors remain
RFC-gated (§0 non-goals).

## 10. Diagnostics (fresh range DL19xx)

| Code | Meaning | Repair |
|---|---|---|
| DL1901 | unknown attribute | remove/typo-fix — exact |
| DL1902 | mailbox overflow with drop policy in abort mode (runtime) | none — telemetry attached |
| DL1903 | dependency version has a published security advisory | upgrade — exact; `--deny-advisories` gates CI |
| DL1904 | actuator command refused by envelope (runtime telemetry class) | none — envelope shown |
| DL1905 | hw grant requested for artifact hash ≠ sim-approved hash | re-approve — `requires_human: true` |
| DL1906 | `@jit`/native emission requested without `exec.native` grant | none — `authority_widening` note in explanation |
| DL1907 | compute dispatch refused by device envelope (runtime telemetry class) | none — envelope shown |
| DL1908 | signature policy requires hybrid; artifact is classical-only or unknown algorithm | re-sign — exact |
| DL1909 | deploy plan authority exceeds environment profile | narrow the plan; widening the profile is flagged `authority_widening`, `requires_human: true` |
| DL1910 | unvalidated (pre-KAT) cryptography invoked without `--unstable` | none — validation status shown |
| DL1911 | device adapter cannot attest independent below-adapter envelope enforcement | grant refused; waiver is policy-explicit, `requires_human: true` |

## 11. Acceptance criteria

1. **P1:** published hot-path table: geo-mean ≤ 2.5× C on the compute-kernel suite under the
   optimizing backend, with per-benchmark numbers, both better and worse, published as-is.
2. **P3/45/46:** full conformance + laundering suites pass identically under all shipped modes;
   `@jit` without `exec.native` → DL1906; with the grant, the JIT tier's traces are identical to
   AOT's on the suite.
3. **P2:** cycle-collection corpus leak-free; bounded-mailbox backpressure demo sustains a
   10:1 producer/consumer rate mismatch at stable memory; multi-threaded WASM passes Stage-7
   criteria 1, 6, 7 (or the track is explicitly deferred with its honesty note published).
4. **P4:** the §5.5 demonstration reproduces from a clean checkout; envelope refusal, heartbeat
   loss → fail-state, and e-stop latency are all measured and within the adapter's published
   budget; the sim-vs-hw artifact-hash gate (DL1905) fires in the staged deploy test.
5. **P5:** one full LTS cycle exercised (a backported security fix released on the LTS train
   with advisory + DL1903 feed entry); the drill timeline published.
6. **P6:** ≥ 10 independently-authored registry packages, ≥ 1 third-party locale catalog,
   ≥ 2 independent agent harnesses consuming `for-agents.md` (referenced by their docs).
7. **P7:** the vendor-neutral compute interface passes conformance via the in-tree CPU reference
   adapter on every CI run; at least one hardware accelerator adapter demonstrated end-to-end (or
   the deferral published, invariant-45-style honesty); an over-envelope dispatch is refused
   (DL1907) with the refusal measured; an adapter that cannot attest below-adapter enforcement
   has its grant refused (DL1911), witnessed; the kernels-are-data law has laundering tests — no
   closure crosses, and an unsigned kernel artifact is refused.
8. **P8:** hybrid signing live for registry artifacts and the release pipeline; KAT validation
   recorded with vector provenance; a classical-only artifact under hybrid-required policy →
   DL1908, witnessed; the repo-wide scrub finds "quantum-proof"/"quantum-safe" only inside
   prohibition/honesty-caveat sentences (§8.3's rule), never as a claim.
9. **P9:** a reference deployment's whole-authority answer is computed, printed, and approved
   before launch in the staged test; a plan exceeding its environment profile → DL1909, witnessed;
   one fleet-update drill: staged rollout, health gate, approved-hash gate (DL1905), and rollback
   exercised.
10. **P10:** the autonomy addendum's per-domain honest boundaries pass line-by-line honesty
    review; a second simulated domain demonstration beyond the arm reproduces from a clean
    checkout — the satellite scenario: a contact-window lease expires at loss-of-signal, the
    pre-attenuated autonomy grant engages, ground re-contact re-delegates; all witnessed in sim
    and honest about being sim, with the recording stating that both broker roles run in one
    simulated host (the federation gap, addendum §2.5).
11. Every claim in Stage-10 marketing/release prose traces to one of these criteria — honesty
    review sign-off, same as Stage 9.

**Close-out disposition (2026-07-20): Stage 10 is CLOSED.** All eleven phases (10a–10l) are built
and committed; the per-criterion verdicts, evidence, and deferral rulings are in the build-order
close-out table (`docs/design/STAGE10_BUILD_ORDER.md` §4). Summary: six criteria MET; criterion 7
MET with its hardware-adapter clause deferred invariant-45-style; criterion 2 MET with the native
JIT/AOT-parity clause honestly N/A (no native tier ships); criterion 1 a DEFERRED-HONEST outcome
(the hot-path table published as-is, the ≤ 2.5× C target not met, D4); criterion 8 with its gates
met and hybrid-live standing on the D5 wait (PQC not yet stable, refused without `--unstable`);
criterion 5's mechanism drilled with its timed cycle PENDING-ADOPTION; criterion 6 PENDING-ADOPTION
(the mechanisms ship, the counts need a real ecosystem); criterion 11 signed off. Nothing failed
silently — every gap is a named, ruled, published deferral or an honest wait on the real world.

## 12. Honesty and threat-model caveats (carry into docs verbatim)

- Performance numbers are workload-specific; "competitive with C" means the published table,
  nothing broader. Where DeluluLang loses, the table says so.
- The JIT tier enlarges the attack surface; that is *why* it is grant-gated and off by default —
  and a grant is a risk decision, not a safety proof.
- Robotics: DeluluLang bounds the command layer; physical safety additionally depends on the
  adapter, firmware, and mechanical design — layers DeluluLang does not control and does not
  claim. The dead-man default fails *safe-as-declared*, which is only as safe as the declared
  fail-state.
- Backpressure prevents unbounded memory, not deadlock; liveness remains un-guaranteed
  (Stage-7 caveat, permanent).
- LTS windows bound *our* response time, not vulnerability existence.
- **Post-quantum, not "quantum-proof."** FIPS 203/204 algorithms are believed resistant to known
  quantum attacks — a judgment about current cryptanalysis, never a proof; the hybrid construction
  exists because the judgment is young.
- **GPU/TPU kernels are foreign code.** DeluluLang bounds their reachability, resources, and
  provenance — not their computation. The authority report says so, per device.
- **Autonomy domains: DeluluLang is the command/mission layer.** It claims no ISO 26262, DO-178C,
  ECSS, or any other certification; it produces *evidence a safety case can cite* (authority
  reports, envelopes, audit chains, deterministic sim), and certified layers below it remain in
  charge of physics. "Supports a safety case" and "is certified" are different sentences; only the
  first is ours.
- **Cloud: the provider's IAM, control plane, and hypervisor are trust anchors DeluluLang does not
  verify.** A deploy plan is least-privilege input to them, not a replacement for them.

---

---

## Implementation status

| Phase | Track | Status | Evidence |
|---|---|---|---|
| 10a | A2 | **BUILT** (2026-07-20) | Attributes active: `@aot`/`@interpret`/`@jit`/`@inline("never"\|"always")` on `fn`/`actor`/module header; DL1901 (exact removal repair, never widening) on unknown names, wrong shapes, and wrong placements; fmt round-trips the canonical own-line form. Witnesses: `attributes_cli.rs` (5 tests incl. the invariant-45 twin — run output and authority byte-identical with and without hints), fixtures `25_attributes_hints.delulu` + `DL1901_unknown_attribute.delulu`; coverage 100% with both new anchors; suite 931/0/4 |
| 10c | B2/B3 | **BUILT** (2026-07-20) | Bounded mailboxes: `actor A(mailbox = N)` (decl wins) + `[actors] mailbox/overflow` manifest defaults; `block` suspends the sending turn at the send site (CAS-exact bound; the B2 criterion witnessed with peak ≤ bound and zero loss), `drop-new` counts and reports every drop (DL1902 as an error only in abort mode); the same-worker structural exemption is documented, telemetry-visible, and witnessed by a test that deadlocks if it is wrong; unconfigured actors keep 1.0's unbounded behavior. `--trace-memory` ships the mailbox half of B3 (bound/peak/drops per actor); heap bytes + collections arrive with 10d. Deferrals + exemptions ruled in build-order D8. Coverage 100% (291 anchors); suite 941/0/4 |
| 10d | B1 | **BUILT** (2026-07-20) | The cycle collector: between turns, an actor's only live roots are its states (no locals, no continuations, immutable globals — the soundness argument is build-order D9), so trial deletion reduces to mark-and-break over a worker-wide registry of List/Record cells and closure-captured scopes. Registration only inside turns; the Study-C perf gate returned **geo-mean −1.0%** (no regression; ≤3% gate passes). The Stage-1/7 leak note is CLOSED: the corpus's manufactured cycles are collected (200/200 in one sweep, output untouched), the reachable-cycle safety half is witnessed at unit and language level with `Weak`-proven freeing, and telemetry reports honest cell counts, not invented bytes. Witnesses: `cycles.rs` (4 unit), `actors_cycles_cli.rs` (3 CLI). Suite 948/0/4 |
| 10e | D1 | **BUILT** (2026-07-20) | The physical boundary opens: `root.actuator(dev)`/`root.sensor(dev)` mint `Cap[Actuator]`/`Cap[Sensor]` by pure attenuation (ungranted device → DL0703 at the mint; zero grant → refused at the pre-flight, before a line runs), `actuator.command(rec) -> Result[Unit, ActuateErr] ! {Actuate}` validates the command record against the granted envelope, and `sensor.read() -> Result[Float, ActuateErr] ! {Read}` — reads are plain `Read`, deliberately not a new effect (§5.1). The envelope is **fail-closed in every branch**: non-record, non-numeric field, unbounded dimension, out-of-range, and NaN are all refused, the unbounded-dimension case by name (build-order D10b — an envelope must not grant what it forgot to mention). A refusal is a **value**, not a fault: DL1904 is telemetry (`command.refused` in the trace, after the attempt record), so the command dies and the process lives. Invariant 50 holds through the null adapter — an unbound sensor answers `NoDevice`, never a fabricated number. `PRIM_TABLE_VERSION` 1→2 (D10a: old DIRs refuse with DL1503 rather than pretend). `rate_hz` parses and is carried but is **not** enforced; it lands with 10f's dead-man leases (D10f). Witnesses: `actuate_cli.rs` (7 tests), fixture `26_actuate.delulu`. Coverage 100% (296 anchors); suite 955/0/4 |
| 10f | D2/D4 | **BUILT** (2026-07-20) | The dead-man closes the loop 10e opened. Every actuator grant now MUST carry `heartbeat_ms`, `ttl_ms` and `fail=hold\|coast\|safe-park` — each omission refused by name, because a device grant with no dead-man is the hazard invariant 47 exists to remove, and defaulting them would be the runtime making an operator's safety decision quietly (build-order D11a). The revoke decision runs on a **watchdog thread that owes the program nothing**: a control loop that stops driving its device loses it on schedule, and the fail-state engages whether or not the interpreter runs another instruction (D11d). Losing the device is its own error — `ActuateErr::LeaseRevoked(Str)`, separate from `Envelope`, because you clamp a bad setpoint and retry but you STOP when you no longer hold the machine (D11b); that prelude change is why `PRIM_TABLE_VERSION` goes 2→3 and why the constant's scope now explicitly covers the prelude types the primitive signatures mention. `rate_hz` is enforced, closing D10f's named gap. The reference simulator arrives behind `--broker-profile sim`: kinematic-only, deterministic under `--seed`, with `DEVICE#DIM` mirror sensors that read back what was commanded — and its own skip branch witnessed, since a mirror of a device the simulator lacks reads `NoDevice` rather than falling through to a plausible number (D11f). The sim-to-hardware gate is DL1905: `--broker-profile sim --signoff <record>` on a clean run, `--approved <record>` at `hw:<adapter>`; **no record at all is a refusal, not a pass**, and the matching-record branch is witnessed separately so the refusals prove something. Latency is measured and published, in separate terms, in `measurements/dead-man/RECORD.md`. "Validated twice" is claimed only as far as it is true (D11e): both checks live in one process today, so it is structural rehearsal for §5.1's host/adapter split, not independent defense-in-depth — what it does buy, unit-witnessed, is that the grant wins if the two copies disagree. Witnesses: `device.rs` (12 unit), `dead_man_cli.rs` (13 CLI, incl. the control case that makes the revocation test mean anything). Coverage 100% (297 anchors); suite 980/0/4 |
| 10g | D5/DD3 | **BUILT** (2026-07-20) | The demonstrations, and the e-stop they turned out to need. Stage 5's reserved `Op::Actuate` is **activated**: it requires `Effect::Actuate`, an actuator grant puts that effect in the node's authority, and every command round-trips to the grant tree — without which an operator has nothing to aim at (build-order D12a). Sensor grants deliberately add nothing, since minting `Read` would confer a file-reading effect nobody granted. `delulu grants revoke` is now a real **e-stop**: the device watchdog probes the tree per tick per device, and any non-live answer — revoked, expired, daemon silent, reply unrecognised — parks the machine, so a broker outage stops the arm (the direction to fail in). Each device holds its **own child node** carrying `{Actuate}` alone, ruled by a failing test: watching the run's node took the program's console down with the arm, and spec §5.2 says *subtree* (D12c). Both e-stops are kept and their blast radii witnessed — revoke the device to stop one machine, revoke the parent to stop everything and end the run. An `Actuate` custody denial is a **value**, not a fault (D12b). **Criterion 4:** `measurements/robotics-demo/` — the agent's three edits (nominal, over-torque, wedged) plus the operator e-stop and the DL1905 gate, reproducing from a clean checkout and re-run per commit by `robotics_demo.rs`. Measured (Windows, n=20): a daemon-custody command costs ~339 µs (worst 354 µs, a ~2.8 kHz ceiling against §5.3's 1–100 Hz claim), refusing costs the same as accepting, heartbeat loss engages `safe-park` ≈255 ms after the missed beat at `heartbeat_ms=250`, and operator-to-stopped is 12.7 ms at p50 / **39.7 ms worst**. **Criterion 10's demonstration half:** `measurements/satellite-demo/` — the contact window IS the lease `ttl_ms`, so LOS is that TTL expiring with nobody to send a message; the pre-attenuated autonomy grant engages by being the grant that did not end, the wide slew is refused on every cycle *including* after LOS, and re-contact is a NEW delegation because authority is re-issued, never revived. Controls pin both: the ungranted device dies at the mint (DL0703, not an expiry) and `missed-heartbeat` never appears. The addendum §2.5 federation note is carried **verbatim** — both broker roles run in one simulated host, witnessing grant semantics and not the transport. Three defects found and fixed: device nodes outlived their runs, so `grants list` offered ghosts an operator could "successfully" revoke while the machine kept moving (D12d); `run --lease` silently discarded `--grant actuator=`/`sensor=` because its refusal list enumerated only the grant kinds that existed when it was written (D12e); and a TTL expiry was journaled as an overdue heartbeat when the beats were arriving perfectly (D12f). One gap named rather than papered over: a grant node carries the authority to actuate but **cannot carry an envelope** (`Scopes` has no device dimension), so a ground segment cannot delegate a *bounded* one — RFC-gated, enforced meanwhile by refusing the combination. Witnesses: `device.rs` (17 unit), `estop_cli.rs` (5), `robotics_demo.rs` (6), `satellite_demo.rs` (4) |
| 10h | F1/F2 | **BUILT** (2026-07-20) | Heterogeneous compute, and a careful account of what "enforced" means. `Cap[Compute]` joins the kinds; `root.compute(dev)` is pure attenuation and `compute.dispatch(kernel, buffer)` carries the **existing** `ForeignCall` effect — no new effect, because inventing one would imply DeluluLang says something about what a kernel computes (D13a). `PRIM_TABLE_VERSION` 3→4: this phase adds both table entries and a prelude type, exactly the case 10f widened that constant to cover. `ComputeErr` takes all-new variant names (`KernelEnvelope`/`UnknownKernel`/`NoAdapter`) because reusing `ActuateErr`'s would make both sums ambiguous and break every 10e/10f program matching them bare (D13b). **Attestation is a property of the adapter, never a claim in a grant** — a grant may `waive-attestation`, never assert one, or DL1911 would be a checkbox (D13c). The in-tree `cpu-reference` adapter therefore attests **false** and always will: it runs in-process, so there is no layer below it, making DL1911's refusal the DEFAULT path for the only adapter that ships. An adapter this build cannot identify is refused too — the skip branch. **What is enforced, precisely** (D13d, and `compute::ENFORCEMENT_NOTE` in code): `memory_bytes` before submission; `kernel_ms` **after the fact**, on the measurement, result discarded; `queue_depth` enforced but **unreachable** from synchronous dispatch (unit-tested with threads); `power_w` **carried and NOT enforced** — mandatory in the grant so a real adapter inherits it, and not a bound anything checks. **Kernels are data**, enforced twice: `dispatch` takes a `Str`, so a closure cannot be spelled as a kernel (DL0401 at the call site), and artifacts are signed files verified BEFORE parsing — signed DeluluLang source is still refused, because a signature proves provenance, not eligibility (D13e). DL1913 unsigned / DL1912 invalid are separate codes with separate remedies, and unlike plugins there is no `require_signed` toggle: a kernel with no provenance is refused always (D13f). Dispatch resolves through VERIFIED artifacts, not the grant string. `delulu authority` prints `compute/<dev> [kernels…]` under the outside-the-proof separator. Measured (Windows, n=20, `measurements/compute/RECORD.md`): accepted dispatch ~2.3 µs, refused (DL1907) ~18.7 µs — refusing costs ~8× succeeding, the opposite of 10g's actuator path, recorded because an asymmetry favouring the failing path is invisible in a design doc. **Criterion 7's hardware-adapter half is DEFERRED and published invariant-45-style: no accelerator adapter ships and none was demonstrated, so invariant 49 is tested against exactly one adapter — a plausible interface, not a proven one.** The manifest half of "kernels named in the manifest" is not built; the grant enumerates them, and that gap is recorded rather than implied away (D13h). Witnesses: `compute_cli.rs` (17), `compute.rs` (6 unit), fixture `27_compute.delulu`. Coverage 100% (303 anchors, ratchet 297→303) |
| 10i | G | **BUILT** (2026-07-20) | Post-quantum signing, and the two gates that keep it honest while it is not yet stable. A self-describing `dlsig1` envelope (`crates/delulu-runtime/src/pqc.rs`): an `alg:` line names its algorithms, one hex part per algorithm — crypto-agility per spec §8.2, a new algorithm addable without a format break. A bare 96-byte v1.0 signature still parses and still verifies under the default policy (`legacy: true`); the stability contract does not bend for a new feature. Two skip branches closed at PARSE time, before any signature is checked: a declared algorithm with no bytes, and a part present but undeclared — a self-describing format that does not match its own declaration has already failed at being self-describing. **DL1908** fires for a classical-only artifact under hybrid-required policy in EITHER classical-only shape (the legacy blob or a `dlsig1` envelope naming only `ed25519`), and for an algorithm id this build cannot evaluate under EVERY policy — a verifier that shrugs at an unevaluable claim is the "when the checker cannot tell, it says yes" failure wearing a crypto-agility costume (D15b). **DL1910** gates BOTH signing and verifying without `--unstable`; verifying is the more important half, because verifying is invoking unvalidated cryptography to make a TRUST DECISION, the more dangerous direction. `delulu sign --hybrid --unstable` / `verify-sig --require-hybrid` are new, purely additive flags — every pre-10i call with neither flag takes the literal original code path; `cmd_verify_sig` now always routes through `pqc::verify`, which under the default policy calls the same `verify_detached` internally, so "byte-for-byte unchanged" is a checked fact (D15c). Plugin/package artifact verification (the `.dpx` in-band mechanism, architecturally separate from `pqc.rs`) was investigated and deliberately left untouched — layering hybrid policy there safely needs the same six-file threading `require_signed` already spans, a feature in its own right. **Official NIST ACVP known-answer vectors were obtained** from `usnistgov/ACVP-Server` (NIST's own repository) for ML-DSA-65 and ML-KEM-768, saved with per-file origin URL and git blob SHA1 under `measurements/pqc/vectors/`, independently re-verified by re-fetching and re-hashing rather than trusting a self-report; one vector was run through this project's own signing code and matched byte-for-byte (`nist_acvp_ml_dsa_65_keygen_seed_to_pk_matches`). This UPDATES ruling D14b's factual claim that the vectors were not in hand — they now are — but NOT D14b's conclusion: **KAT validation is necessary, not sufficient.** Both adopted crates remain, by their own authors' statement, never independently audited, so the independent second gate still holds and post-quantum signing does NOT reach stable this phase; every PQ operation still refuses without `--unstable` (D15e, and D14's text is not rewritten — the errata is the new ruling, per the D21 precedent). **First delegation to subagents in Stage 10**, explicitly owner-authorized mid-phase: the CLI wiring and the vector research were each produced by an independent Sonnet-5 subagent and independently re-verified before landing — every test re-run from a clean build, and the vector-research agent's central claim checked by re-fetching two saved files directly from GitHub and matching bytes and git blob SHA1s (D15f). Witnesses: `pqc.rs` (11 unit, incl. the NIST KAT test), `pqc_cli.rs` (6 CLI). Coverage 100% (305 anchors, ratchet 303→305) |
| 10j | H | **BUILT** (2026-07-20) | Cloud and fleets: a deploy plan is an authority manifest, a fleet update reuses the sim-to-hardware hash gate one level up. `delulu deploy plan --service NAME=DIR --env ENV.toml` parses the environment profile with the SAME `delulu_runtime::parse_manifest` a package's own `delulu.toml` uses — an environment profile is an authority manifest for a place a program runs, not a new format. Each service's authority is computed the same way `delulu authority <pkg-dir>` computes it (the existing `package_authority_value` closure, injected exactly as `cmd_publish` already receives it — no visibility widened). **DL1909** refuses the WHOLE plan, never partially, naming the exceeding service and effect by name (invariant 53: no plan, no launch); a plain `Diagnostic::error` matching DL1905's real precedent, since `Repair`'s fields carry byte-offset SOURCE edits and no span exists for a TOML ceiling (D16b). **`delulu fleet update` reuses DL1905, not a new code** — spec §9.3 names it explicitly as that rule generalized, and the reuse is real: the SAME `content_hash` and the SAME `Approval` record 10f built, with DL1905's explain text extended additively rather than duplicated (D16c). **`--previous` is unconditionally required on every invocation** — a design choice this phase's own demo caught for real: its first draft omitted the flag on three of four passes, and `run-demo.sh` genuinely failed exactly as the builder agent predicted before the three-line fix landed (D16d). The staged rollout (`run_rollout`) is a pure state machine — a health-check closure and a journal closure, no file/hash/CLI awareness — so "no member after a failure is ever staged" is asserted against the JOURNAL, not just the returned outcome (D16e). **First delegation to agent PAIRS rather than single agents**, per an explicitly broadened instruction: an adversarial test-writer built 14 tests against the fixed contract before the implementation existed (confirmed by its own read of `cli.rs` showing no `deploy` arm yet); a demo/record author built real fixtures with REAL content hashes (via the already-shipped `--signoff` machinery, never invented) and marked every not-yet-known value as an explicit placeholder. Every agent's work was independently re-verified — every test re-run from a clean build, the demo's reported contract mismatch reproduced firsthand before fixing it (D16f). Two small integration fixes, neither round-tripped through another agent: the `--previous` omission (three lines), and a rendering inconsistency between the two new commands — `deploy.rs` used the real `render_human` renderer for DL1909, `fleet.rs` had built a bespoke `eprintln!` for DL1905 believing the renderer was unreachable when it was already public; both now render identically (D16g). Measured (Windows, n=20): all four demo passes cluster at 67–104 ms with **no separable cost between accepting and refusing** — a differential attempt came back as pure noise, meaning this drill's own per-member cost is too small to clear `delulu`'s process-startup floor at these member counts, stated as a bound rather than stretched into false precision (`measurements/fleet-update/RECORD.md`). Named, not built: only EFFECTS are checked against the environment profile; capability scopes and foreign holes (also named in spec §9.2's full answer) are not, recorded in `deploy.rs`'s module doc rather than implied as covered. Witnesses: `deploy_cli.rs` (5), `deploy_plan_adversarial.rs` (14), `fleet_cli.rs` (10), 7+7 unit tests. Coverage 100% (306 anchors, ratchet 305→306) |
| 10k | C | **BUILT** (2026-07-20) | LTS and security operations. The registry gains an **advisory feed**: advisories stored per package as JSONL under `advisories/<name>` (the index's shape and discipline, one level over), served at `GET /advisories/<package>`, filed with `POST /advisory` or `delulu-registry advisory file` under a **package-scoped token** (yank's standing, reused — out-of-scope and unsigned filings refused DL1706, tested over HTTP), and dumped to a build-readable feed file by `advisory export`. `delulu build` consults a **local** `delulu.advisories.json` (synced out-of-band, read offline — the registry being down never breaks *or silences* a build; no new inter-crate dependency, client and server share only the JSON wire shape as they do for the index) and emits **DL1903** — a WARNING — when a resolved dependency is on a version an advisory names; **`--deny-advisories`** turns every match into an error, the CI gate, scoped to `build` so `check`'s output stays byte-stable. Matching is **exact version-string membership**, deliberately not semver ranges: a range predicate unevaluable against an odd version string is where a checker silently answers "not affected," the fail-open this detector exists to prevent (a half-record is counted malformed, never read as "matches nothing"). **The skip branch holds** (D17d): under `--deny-advisories` an absent, unreadable, or partly-unparseable feed is a build FAILURE — a gate that cannot find its evidence must not report a clean pass (the DL1905 missing-sign-off precedent) — while WITHOUT the gate an absent feed is silence; the gate-blocked refusal is a note + forced exit-1 mirroring `git_deferred`, not a DL1903 (which means specifically "a dependency is advised") and not a new code (the DL1901–1911 budget has no free slot). **Criterion 5 splits honestly** (D17g): the whole loop — advisory filed on the registry → feed exported → DL1903 warning → `--deny-advisories` CI failure → backported fix → clean gate → skip-branch refusal — is drilled end to end in `measurements/lts-cycle/` (6/6, registry as source of truth), while the **timed** LTS cycle (a real 12-week train, a real 24-month backport, a real CVE/CNA) is recorded PENDING-ADOPTION, never faked. Published: `docs/release/SUPPORT_MATRIX.md` (trains, LTS every 4th minor, 24-month windows) and `docs/design/VERSION_COEVOLUTION.md` (broker/protocol/DIR majors, n−1 concurrent during LTS). Built **solo by the head chef** (Opus 4.8), reverting 10i/10j's delegation. Witnesses: `advisories_cli.rs` (7), `advisories.rs` (3 unit), 5 registry advisory tests. Coverage 100% (307 anchors, ratchet 306→307); suite 1098/0/4 |
| 10b | A3 | **BUILT** (2026-07-20) | The gate before the engine: `--grant exec.native` (embedded; default-off everywhere, `--grant-manifest` never confers it, a lease derives it false — build-order D6), manifest declaration `[authority] exec.native = true` (reviewable request, never permission), authority report stamps `native_emission` only when `@jit` is present (hint-free reports byte-identical to 1.0), and DL1906 as a warning-class note that an ungranted hint was IGNORED — the program runs interpreted, sandbox intact (D7: invariant 45 means a hint may not change whether a program runs, so refusal was never an option). Witnesses: `jit_policy_cli.rs` (5 tests incl. the json-channel and skip-branch stability checks); the 10a twin test upgraded to its strong form (the stamp is the ONLY difference). Coverage 100% (290 anchors); suite 936/0/4. No native tier exists — that honesty is in the code, the explain, and the authority line itself |
| 10l | A1/A4 | **BUILT** (2026-07-20) | The last phase, and both remaining Track-A items land as **evidenced deferrals** — the passing outcome D4 defined ("'not production-ready, deferred, here is why' is a PASSING outcome; mode honesty beats mode count"). **A1 (optimizing tier):** the optimizing backend that ships in 1.x is the Cranelift-optimized Wasmtime tier, now **explicitly pinned** — `delulu_wasm::optimizing_engine()` sets `cranelift_opt_level(Speed)` rather than inheriting wasmtime's default, so the tier a `.dwx` runs under is a documented, drift-proof artifact instead of an accident of an upstream default; the four run-path engine sites route through it, and the plugin/limits engine keeps its own host-safety `Config` (§6f.2b). **Criterion 1 is NOT met and its hot-path table is published as-is — which criterion 1 explicitly asks for** (`measurements/study-c/HOT_PATH_TABLE.md`): the optimizing backend runs **1 of 6** compute kernels (`fib_recursive_24` at 2.0× C, a single startup-dominated point), the other five hit DL1201 because the 1.x WASM backend compiles a subset of the language, so a geo-mean over the suite is not computable; the interpreter (the default engine, running all six) is **2.0×–60.5× C**, and the C lane is startup-dominated so those ratios *understate* the true compute gap — v1.0 is **not competitive with C** on this suite, stated in those words (§5.11 forbids implying otherwise). The **DIR-level optimizer** (cross-package inlining, monomorphization, escape analysis) is deferred with rationale (build-order D18c); **authority-preservation across optimization holds structurally** — `delulu authority` is checker-computed and `.dwx`-embedded before any backend runs, so codegen-time inlining has nothing authority-relevant to change, and semantic parity is proven by the 50k two-engine differential, re-run at 3000 programs against the pinned engine and agreeing on every one. **A4 (multi-threaded WASM):** deferred with its honesty note (`docs/design/THREADED_WASM_DEFERRAL.md`) — criterion 3's own sanctioned path. Multi-threaded actor execution **already ships on the interpreter** (worker-owned scheduler, `--actors-threads`, TSAN-clean, Stage-7 criteria met there), so the capability is not pending; the WASM engine's scheduler is cooperative single-threaded by design (§6.5) with the same observable semantics, and porting it to a genuinely multi-threaded WASM engine needs the shared-everything-GC proposal stack, which was not a production-ready foundation at 1.x — the pinned wasmtime 27 run-path engine enables neither the wasm-threads nor the wasm-GC proposals. §2.4 sanctions the wait. **No new diagnostics, no new anchors** — coverage unchanged at 307, the pin is behavior-preserving (differential-witnessed), clippy baseline 65 unchanged. Built **solo** (Opus 4.8). Build-order D18. Suite 1098/0/4 |

---

*This closes the committed sequence: Stages 1–9 build and prove the language; Stage 10 makes it
an industrial tool with physical stakes. Everything beyond lives in RFCs — proposed, argued,
measured, and honest, like everything above. Be delulu; ship the proof.* 🐦‍🔥
