# Stage 10 Playbook — "Industrial" (JIT policy, autonomy, compute, PQC, cloud, LTS)

**Companion to:** `docs/design/STAGE10_SPECIFICATION.md` (normative) and, for Track D domain scope,
`docs/design/STAGE10_AUTONOMY_ADDENDUM.md`. This file is *how to build it*.
**Depends on:** Stage 9 (v1.0 released; stability contract in force; measurement baseline published).
*Status note (updated 2026-07-20): **v1.0.0 is RELEASED** (both rc.1 blockers closed — D22 and the
D9 drill record — gate re-run green). The dependency is satisfied; Stage 10 build work may begin.
Spec Rev 2 (owner-directed) widened the program to eight tracks.*

> **The one-sentence goal:** make DeluluLang *production-ready* — a checklist (P1–P10), not a vibe:
> competitive-with-C performance, concurrency at scale, grant-gated native code, physical-stakes
> autonomy with dead-man safety across vehicles/aircraft/satellites/robots, vendor-neutral
> heterogeneous compute, hybrid post-quantum signing, authority-planned cloud deploys, LTS/CVE
> operations, and a living ecosystem. Unlike Stages 1–9, Stage 10 is a **program of parallel
> tracks (A–H)**, each independently shippable in a 1.x minor, **none changing v1.0 semantics**
> except where an activation was explicitly reserved (attributes, `Actuate`, threads-in-WASM).

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

### Track D-domains — the autonomy generalization (§5.6, addendum) — *Rev 2*

**DD1 — device-class catalog.** Energy envelopes (batteries/BMS: charge/discharge amps, cell-temp
ceiling, SoC floor), safety-mechanism supervision (invariant 52: the hardware chain must work with
DeluluLang dead; software e-stop = subtree revoke with a *published* latency budget), MCU
flash-and-talk (signed firmware + DL1905 hash gate; talking is `Read`/`Write` under device
scopes). Build on the D1 envelope machinery — no new mechanism, new envelope vocabularies.
**DD2 — domain profiles.** Vehicles (mission layer above the ADS), aircraft/UAS (lost-link is a
**pre-declared attenuated grant**, delegated before takeoff, engaged by lease death), satellites
(**the contact window is a lease TTL**; ground segment holds root; between contacts the craft runs
pre-attenuated autonomy grants; anomalies attenuate, never widen). Each profile is a broker
profile + adapter class + envelope vocabulary — the addendum is normative. **The cross-machine
grant tree the satellite and fleet profiles imply is broker federation — a named, RFC-gated gap
(addendum §2.5), not an existing mechanism**; Stage 5's broker is local-only and no DD sentence
may imply otherwise.
**DD3 — the satellite demonstration.** Deterministic seeded sim, **both broker roles in one
simulated host over a simulated link** (grant semantics witnessed, federation transport not —
addendum §2.5): lease dies at loss-of-signal → autonomy grant engages → re-contact re-delegates;
recorded, reproducible. *Test (criterion 10):* the demo reproduces from a clean checkout; the
addendum's honest boundaries pass line-by-line review.

### Track F — Heterogeneous compute (§7)

**F1 — the interface + CPU reference adapter.** One vendor-neutral adapter interface (enumerate /
allocate-within-envelope / submit / await / telemetry) and a deterministic in-tree CPU adapter so
conformance runs on every CI box with zero hardware. `Cap[Compute]`, envelope enforcement
host-side before submission (over-envelope → DL1907, **the command dies, not the process**; an
adapter that cannot attest independent below-adapter enforcement → grant refused, DL1911,
human-gated waiver).
**F2 — the honesty wiring.** Dispatch carries the existing `ForeignCall` effect — no new effect;
`delulu authority` shows `foreign: compute/<device> [kernels…]` under the outside-the-proof
separator. Kernels are hashed, signed **data** (no closure ever crosses — write the laundering
tests first, kitchen rule).
**F3 — one hardware adapter.** CUDA or Vulkan-compute behind the same interface, Verified-class,
signed — or the deferral published (invariant-45-style honesty). *Test (criterion 7):* conformance
via the CPU adapter per-commit; DL1907 witnessed and measured; unsigned kernel refused.

### Track G — Post-quantum cryptography (§8)

**G1 — envelope agility first.** Algorithm identifiers in the signature envelope and registry
transport *before* any new algorithm — the format must name its crypto so it can be replaced
without a break.
**G2 — ML-DSA-65 + ML-KEM-768, hybrid.** Signatures: ed25519 **and** ML-DSA-65, both must verify
under hybrid-required policy (classical-only → DL1908). Transport: ML-KEM-768 hybridized with
X25519. **Never PQ-only** (invariant 51). **House rule 5 applies: cryptography is never
hand-rolled** — take a vetted, KAT-validated implementation as a dependency under a build-order
ruling that records the vetting (the `ed25519-dalek` precedent); spend the in-tree effort on the
envelope format, the policy gates, and the tests.
**G3 — KAT validation.** Byte-exact against the official NIST vectors, provenance recorded in the
build order; until then everything sits behind `--unstable` (bare invocation → DL1910). *Test
(criterion 8):* hybrid live end-to-end; DL1908 witnessed; the scrub finds "quantum-proof" and
"quantum-safe" only inside prohibition/honesty-caveat sentences (§8.3's rule), never as a claim.

### Track H — Cloud, fleets, infrastructure (§9)

**H1 — provider adapters.** Cloud APIs = `Cap[Http]` scoped to provider endpoints + Verified-class
typed adapters; no privileged provider (invariant 49).
**H2 — `delulu deploy plan`.** The whole-deployment authority answer, computed and printed
**before launch**; environment profiles (`envs/*.toml`) as ceilings; plan > profile → DL1909, and
widening the profile is `authority_widening`, `requires_human: true` (invariant 53: no plan, no
launch).
**H3 — the fleet drill.** Staged rollout, health gates, DL1905 approved-hash gate generalized to
fleets, rollback pinned at rollout start. *Test (criterion 9):* the drill runs staged, gates fire,
rollback exercised.

---

## 2. The traps (highest-stakes stage — read twice)

1. **A mode that can't pass the full suite is not a mode.** Invariant 45 is non-negotiable. Do not
   ship a faster JIT tier that diverges on one diagnostic; fix it or gate it `--unstable`.
2. **Native code is off by default, everywhere, forever-until-granted.** `exec.native` is a human
   policy decision, `⊑`-checked and revocable like any authority. A grant is a *risk decision, not a
   safety proof* (spec §12). The JIT enlarges the attack surface — that is *why* it is grant-gated.
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
   (criterion 1, spec §12). No broader claim exists.
8. **Tracks are independent and honest about readiness.** Multi-threaded WASM (A4) waits on
   Wasmtime's threads/GC maturity; if it's not ready, publish the deferral. Mode honesty beats mode
   count.
9. **"Post-quantum," never "quantum-proof."** The banned words appear nowhere but their prohibition
   (spec §8.3) — cryptographic "proof" claims are the overclaim the honesty clauses exist to stop.
   And no PQC leaves `--unstable` without byte-exact official-KAT validation: self-round-tripping
   proves interoperability with yourself, which nobody needs.
10. **Hybrid or nothing.** A PQ-only signature or session is *weaker* than v1.0's guarantee the day
    a lattice break lands. Both algorithms verify, or the artifact fails (DL1908). This is invariant
    51 and it has no fast path.
11. **No vendor gets a private door.** Anything a CUDA adapter can express, the vendor-neutral
    interface must express (invariant 49) — the first privileged hook creates a second class of
    citizen in the authority model. The CPU reference adapter is the conformance floor precisely so
    no vendor's hardware is required to test the law.
12. **Kernels are data** (invariant 50). A DeluluLang closure never becomes a kernel; kernels are
    hashed, signed artifacts, and the laundering tests for that boundary are written *before* the
    feature is claimed (kitchen rule — the skip branch is where security rules die).
13. **The safety chain must survive DeluluLang's death** (invariant 52). If a design's e-stop works
    only while the DeluluLang process is healthy, the design is wrong — reject it in review no
    matter who proposes it. DeluluLang supervises above the chain; the chain answers to physics.
14. **Certification claims are defects.** No sentence anywhere may read as ISO 26262 / DO-178C /
    ECSS / IEC 61508 credit (addendum §3). "Produces evidence a safety case can cite" is the entire
    claim. WCET and hard real-time are refused permanently.
15. **No plan, no launch** (invariant 53). A deploy whose whole-authority answer wasn't computed
    and approved does not run — and widening an environment profile to make a plan fit is an
    `authority_widening`, human-gated decision, never a CI convenience.

---

## 3. Definition of done — the P1–P10 production checklist

Stage 10 (and thus the committed sequence) is done when spec §11 criteria 1–11 pass and the P1–P10
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
- **P7** (heterogeneous compute): CPU-reference conformance per-commit; a hardware adapter
  demonstrated or honestly deferred; DL1907 witnessed; kernels-are-data laundering tests green
  (criterion 7).
- **P8** (post-quantum): hybrid signing live; official-KAT validation recorded; DL1908 witnessed;
  the vocabulary scrub clean (criterion 8).
- **P9** (cloud/fleets): the deploy-plan gate live (DL1909 witnessed); one fleet drill with
  staged rollout, hash gate, and rollback (criterion 9).
- **P10** (autonomy domains): the addendum's boundaries pass line-by-line review; the satellite
  sim demo reproduces from a clean checkout; the other domains ship as reviewed *profiles* —
  specified and boundary-honest, not demonstrated — and the federation gap stays named
  (criterion 10, addendum §2.5).
- **Honesty sign-off** (criterion 11): every marketing/release sentence traces to one of these
  criteria — the same honesty review Stage 9 institutes.

Everything beyond P1–P10 lives in RFCs (spec §0 non-goals: distributed actors, per-plugin microVMs,
per-actor broker nodes, I/O-quota grants, effect handlers, `await`, taint beyond `Secret`, certified
compiler; plus the Rev-2 recorded deferrals: a dedicated `Dispatch` effect, bare-metal MCU
compilation) — proposed, argued, measured, honest.

*This closes the committed sequence: Stages 1–9 build and prove the language; Stage 10 makes it an
industrial tool with physical stakes. Be delulu; ship the proof.* 🐦‍🔥
