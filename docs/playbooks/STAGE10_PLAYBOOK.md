# Stage 10 Playbook — "Industrial" (JIT policy, robotics, LTS)

**Companion to:** `docs/design/STAGE10_SPECIFICATION.md` (normative). This file is *how to build it*.
**Depends on:** Stage 9 (v1.0 released; stability contract in force; measurement baseline published).

> **The one-sentence goal:** make DeluluLang *production-ready* — a checklist (P1–P6), not a vibe:
> competitive-with-C performance, concurrency at scale, grant-gated native code, physical-stakes
> robotics with dead-man safety, LTS/CVE operations, and a living ecosystem. Unlike Stages 1–9,
> Stage 10 is a **program of parallel tracks (A–E)**, each independently shippable in a 1.x minor,
> **none changing v1.0 semantics** except where an activation was explicitly reserved (attributes,
> `Actuate`, threads-in-WASM).

---

## 0. Orientation — the mindset shift

Stages 1–9 built and proved *the language*. Stage 10 is where the invariants **earn their keep** —
the payoff, not new theory. Two framing rules dominate every track:

- **Mode-sandbox invariance is executable (invariant 45).** Every execution mode — interpreter,
  AOT-WASM, optimized-native, JIT-tier — must pass the *same* conformance + laundering suites with
  the *same* diagnostics and traces. **A mode that cannot pass the full suite does not ship as a
  mode** (it may ship behind `--unstable`). This is the single acceptance bar that unifies Track A.
- **Native-code emission is a grant (invariant 46).** Nothing JITs or runs natively-emitted code
  unless its grant carries `exec.native = true` — issued like any authority (broker node data,
  `⊑`-checked, audited, revocable), default off everywhere. The Stage-1 promise "only
  human-controlled policy may grant native code" becomes *mechanism* here.

Because the tracks are independent, **a track that is not production-ready waits and says so** —
"mode honesty beats mode count" (spec §2.4). Ship order below is a suggestion; tracks may land in any
order as their dependencies (and the maturity of external pieces like Wasmtime threads) allow.

---

## 1. Track-by-track build plan

Each track is a mini-stage: build in phases, keep the full suite green under every shipped mode,
commit per phase, and log status in `STAGE10_SPECIFICATION.md`'s status section.

### Track A — Performance & execution modes (§2)

**A1 — the optimizing backend.** Cranelift-optimized Wasmtime tier + a DIR-level optimizer:
cross-package inlining (legal *because rows are declared and checked* — inlining cannot change
authority), generic monomorphization (the Stage-3 inliner generalized), escape analysis to unbox
locals. Wire continuous benchmarking (Study-C suite per-commit, >3% geo-mean regression blocks merge).
**A2 — attributes (grammar activation).** `@aot`/`@interpret`/`@jit`/`@inline(never|always)` — **all
hints**, none changes semantics; unknown attribute → DL1901 (no silent vendor attribute space —
extensions go through RFCs). *Test:* invariant 45 — the suite passes identically with and without any
hint.
**A3 — JIT policy mechanics.** `exec.native` in grants; manifest `[authority] exec.native = true`
required to *request* it; `delulu authority` reports `native code emission: granted/denied` as its own
line; untrusted-agent default = denied (red-tier run-prompt wording). `@jit`/native without the grant
→ **DL1906**. **The JIT runs the same host-side scope checks — they were never in guest code, so there
is nothing for a JIT to "optimize away"** (the Stage-3 architecture paying off). *Test (criterion 2):*
suite identical across modes; `@jit` sans grant → DL1906; with grant, JIT traces ≡ AOT traces.
**A4 — multi-threaded WASM engine.** Wasmtime threads + shared-everything-GC per its maturity;
port the actor scheduler; re-run Stage-7 TSAN/parity criteria on this engine. **If the proposal stack
is not production-ready, this track waits and publishes that.**

### Track B — Memory & actor-runtime maturity (§3)

**B1 — per-actor cycle collection.** A trial-deletion collector over each actor's Rc heap, run
*between turns*; closes the Stage-1/7 leak note. The leak corpus (cyclic graphs, promise chains) goes
from "documented leak" to "collected." *Test (criterion 3, part):* leak-free corpus.
**B2 — bounded mailboxes.** `actor A(mailbox = 10_000)` manifest default + per-spawn override;
overflow policy explicit: `block` (default — backpressure by suspending the *sending turn* at the send
site, safe because the send is the turn's last-resort suspension point; deadlock remains a documented
non-guarantee) or `drop-new` (counted, reported); DL1902 on drop-in-abort-mode. *Test:* a 10:1
producer/consumer mismatch sustains at stable memory.
**B3 — actor heap telemetry.** `--trace-memory`: per-actor bytes, collections, drops — the ops
surface.

### Track C — LTS & security operations at scale (§4)

**C1 — release trains + LTS.** 1.x minors every ~12 weeks; **LTS** every 4th minor with 24-month
security backports; publish the support matrix as a table.
**C2 — CVE process.** CNA (or partner CNA) registration; advisories tie to DL-coded detectors;
`delulu build` flags a vulnerable-version *use* via the registry advisory feed → **DL1903 warning**,
`--deny-advisories` gates CI. *Test (criterion 5):* run one full LTS cycle for real — a backported
security fix released on the LTS train with advisory + DL1903 feed entry; publish the drill timeline.
**C3 — version co-evolution policy.** Broker/protocol/DIR major-version policy: one published page,
n−1 majors supported concurrently during LTS windows.

### Track D — The embodied/robotics profile (`Actuate` activates) (§5) — *the highest-stakes track*

**D1 — the model.** New capability kinds `Actuator`/`Sensor`; new **synchronous-class** effect
`Actuate` (sensor reads are `Read` with sensor scopes). **An actuator capability's scope *is* its
envelope** (torque/angle/velocity/duty ranges, rate_hz, heartbeat_ms, ttl_ms — spec §5.1 JSON).
Every `command` is validated against the envelope **twice**: host-side (broker/backend, before the
wire) *and* device-adapter-side where supported. Envelope violation → `ActuateErr::Envelope` (runtime
DL09xx + **DL1904** telemetry) — **the command dies, not the process** (a control program that aborts
on a bad setpoint is itself a hazard).
**D2 — dead-man semantics (invariant 47).** Actuator leases have mandatory `heartbeat_ms`/`ttl_ms`;
the runtime auto-heartbeats while the holding actor's turns are healthy; a missed beat → broker
revokes → the adapter's **mandated fail-state** engages (`hold`/`coast`/`safe-park`, declared per
device at grant time). E-stop = `delulu grants revoke` on the actuator subtree (same mechanism as
everything else), with a **published revoke-to-fail-state latency budget** (≤ heartbeat_ms + adapter
latency).
**D3 — the honest boundary (verbatim in docs).** DeluluLang commands the **policy/command layer
(1–100 Hz)**. Servo loops, torque control, electrical protection, and hard real-time run in
firmware/RTOS *below the adapter* — DeluluLang sets *setpoints within envelopes*, does not close
1–10 kHz loops, and **no marketing sentence may imply otherwise.** The envelope is enforced above
*and* below (defense in depth): even a compromised DeluluLang stack cannot exceed what the
adapter/firmware enforces independently.
**D4 — sim-to-real (invariant 48).** `--broker-profile sim` (a deterministic reference simulator ships
in-tree: kinematic arm + sensor models, seeded) vs `--broker-profile hw:<adapter>` (adapters are
**Verified-class, `require_signed: true`** Stage-6 plugins). The deploy flow records the artifact hash
at sim sign-off and **refuses a differing hash at hw grant time without explicit re-approval** →
**DL1905**.
**D5 — the public demonstration (deliverable, `measurements/robotics-demo/`).** A scripted LLM harness
live-edits a simulated arm's control program: correct edits take effect at rate; a 5 N·m command
against a 2.5 N·m envelope is refused per-command; a harness that stops heartbeating loses the arm to
`safe-park`; an operator e-stop revokes the subtree. Recorded, reproducible from a clean checkout,
**honest about being simulation.** *Test (criterion 4):* all four behaviors measured within the
adapter's published budget; the sim-vs-hw hash gate (DL1905) fires in the staged deploy test.

### Track E — Ecosystem (§6)

Registry growth mechanics (a curated **"authority showcase"** — packages notable for *minimal*
authority), third-party adapter/catalog channels, `for-agents.md` versioned as an API, and the
deprecation of nothing (v1 stability holding *is* the deliverable). *Test (criterion 6):* ≥10
independently-authored registry packages, ≥1 third-party locale catalog, ≥2 independent agent
harnesses consuming `for-agents.md`.

---

## 2. The traps (highest-stakes stage — read twice)

1. **A mode that can't pass the full suite is not a mode.** Invariant 45 is non-negotiable. Do not
   ship a faster JIT tier that diverges on one diagnostic; fix it or gate it `--unstable`.
2. **Native code is off by default, everywhere, forever-until-granted.** `exec.native` is a human
   policy decision, `⊑`-checked and revocable like any authority. A grant is a *risk decision, not a
   safety proof* (spec §9). The JIT enlarges the attack surface — that is *why* it is grant-gated.
3. **The robotics honesty boundary is a hard line, not a disclaimer.** DeluluLang commands 1–100 Hz
   setpoints within envelopes; it does **not** close kHz servo loops or provide hard real-time.
   Physical safety additionally depends on adapter/firmware/mechanical layers DeluluLang does not
   control and does not claim. Every robotics sentence in every doc traces to this boundary.
4. **The command dies, not the process.** Envelope violations refuse the *command* (ActuateErr +
   DL1904) and keep the control program running. A control program that aborts mid-motion is a
   hazard. This is the opposite of the usual "fail fast."
5. **Dead-man fails *safe-as-declared* — only as safe as the declared fail-state.** The default
   protects against hung/partitioned programs by *revoking physical authority*; but "safe-park"
   safety is the adapter's, not DeluluLang's. State this every time.
6. **Sim and real are the same program; the diff is detectable.** The artifact-hash gate (DL1905,
   invariant 48) is the mechanism — do not let a deploy substitute a different artifact than what was
   simulated without explicit human re-approval.
7. **Performance claims trace to the published table or they don't ship.** "Competitive with C" means
   the Study-C table, geo-mean ≤2.5× C on the compute-kernel suite, *with the losses published too*
   (criterion 1, spec §9). No broader claim exists.
8. **Tracks are independent and honest about readiness.** Multi-threaded WASM (A4) waits on
   Wasmtime's threads/GC maturity; if it's not ready, publish the deferral. Mode honesty beats mode
   count.

---

## 3. Definition of done — the P1–P6 production checklist

Stage 10 (and thus the committed sequence) is done when spec §8 criteria 1–7 pass and the P1–P6
checklist (spec §0) is *each verified by its named criterion*:

- **P1** (perf): published hot-path table, geo-mean ≤2.5× C, losses included (criterion 1).
- **P2** (concurrency at scale): cycle-collection leak-free + bounded-mailbox backpressure +
  multi-threaded WASM passing Stage-7 criteria *(or track deferred with published honesty note)*
  (criterion 3).
- **P3** (mode policy): all modes pass identically; native-code grant-gated (criterion 2).
- **P4** (physical stakes): the robotics demo reproduces; envelope/heartbeat/e-stop within budget;
  hash-gate fires (criterion 4).
- **P5** (operational maturity): one LTS cycle exercised for real with published timeline (criterion 5).
- **P6** (ecosystem): the registry/harness/locale thresholds met (criterion 6).
- **Honesty sign-off** (criterion 7): every marketing/release sentence traces to one of these
  criteria — the same honesty review Stage 9 institutes.

Everything beyond P1–P6 lives in RFCs (spec §0 non-goals: distributed actors, per-plugin microVMs,
per-actor broker nodes, I/O-quota grants, effect handlers, `await`, taint beyond `Secret`, certified
compiler) — proposed, argued, measured, honest.

*This closes the committed sequence: Stages 1–9 build and prove the language; Stage 10 makes it an
industrial tool with physical stakes. Be delulu; ship the proof.* 🐦‍🔥
