# DeluluLang — Stage 10 Implementation Specification

**Version:** 1.x ("Industrial") — the production-readiness stage beyond v1.0.
**Status:** Committed. Unlike Stages 1–9, Stage 10 is a **program of parallel tracks**, each
independently shippable in a 1.x minor, each RFC-visible, none changing v1.0 semantics except
where an activation was explicitly reserved (attributes, `Actuate`, threads-in-WASM).
**Depends on:** Stage 9 (v1.0 released; stability contract in force; measurement baseline
published).
**Governing documents:** `CONSTITUTION.md` (§5.11 execution modes, §7 robotics, §9 honesty),
`SOUNDNESS_AUDIT.md`.

---

## 0. Scope and goal — what "production-ready DeluluLang" concretely means

Production-ready is a checklist, not a vibe. DeluluLang is production-ready when **all** of the
following are true, each verified by a named criterion in §8:

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

**In scope:** the five tracks below (§2–§6), LTS machinery (§7), DL19xx diagnostics.
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

## 5. Track D — The embodied/robotics profile (`Actuate` activates)

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

## 6. Track E — Ecosystem

Registry growth mechanics (curated "authority showcase" list — packages notable for *minimal*
authority), third-party adapter/catalog support channels, `for-agents.md` versioned as an API,
and the deprecation of nothing (v1 stability holding is itself the deliverable).

## 7. Diagnostics (fresh range DL19xx)

| Code | Meaning | Repair |
|---|---|---|
| DL1901 | unknown attribute | remove/typo-fix — exact |
| DL1902 | mailbox overflow with drop policy in abort mode (runtime) | none — telemetry attached |
| DL1903 | dependency version has a published security advisory | upgrade — exact; `--deny-advisories` gates CI |
| DL1904 | actuator command refused by envelope (runtime telemetry class) | none — envelope shown |
| DL1905 | hw grant requested for artifact hash ≠ sim-approved hash | re-approve — `requires_human: true` |
| DL1906 | `@jit`/native emission requested without `exec.native` grant | none — `authority_widening` note in explanation |

## 8. Acceptance criteria

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
7. Every claim in Stage-10 marketing/release prose traces to one of these criteria — honesty
   review sign-off, same as Stage 9.

## 9. Honesty and threat-model caveats (carry into docs verbatim)

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

---

*This closes the committed sequence: Stages 1–9 build and prove the language; Stage 10 makes it
an industrial tool with physical stakes. Everything beyond lives in RFCs — proposed, argued,
measured, and honest, like everything above. Be delulu; ship the proof.* 🐦‍🔥
