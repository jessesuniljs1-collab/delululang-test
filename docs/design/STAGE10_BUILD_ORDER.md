# Stage 10 Build Order — "Industrial" (tracks A–H)

**Status: COOKING** (opened 2026-07-20, on v1.0.0 `198bf44`). **Governing documents
(precedence):** `STAGE10_SPECIFICATION.md` Rev 2 (normative) + `STAGE10_AUTONOMY_ADDENDUM.md`
(normative for Track D domains) > `docs/playbooks/STAGE10_PLAYBOOK.md` (method) > this build
order (operational rulings). **Depends on:** v1.0.0 RELEASED (satisfied — Stage 9 close-out).

The stability contract is IN FORCE from the first commit of this stage: everything here is
additive — codes add-only in the DL19xx budget, grammar changes only where 1.0 explicitly
reserved the activation (attributes, `Actuate`, threads-in-WASM), machine channels byte-identical
for programs a phase does not touch. `release_requires_full_coverage` is an active hard gate:
**a phase that adds an anchor ships its witnesses in the same commit or does not ship.**

## 1. House rules (carried, permanent)

1. Never push to GitHub. Commit locally, per phase, at green.
2. The forbidden word never appears in any repo/product surface; scrub before close-outs.
3. Every rule ships with named witnesses; the skip branch ("what if the checker couldn't tell")
   is written BEFORE the rule is claimed.
4. Machine channels (`--json`) are never styled and stay byte-identical for untouched programs.
5. Cryptography is never hand-rolled (Stage-6 house rule 5; spec §8.2 binds Track G to it).
6. Docs move with code; the spec's status section logs per phase.
7. The commit trailer names the model ACTUALLY RUNNING (Stage-9 D21 lesson, permanent).
8. New dependencies require a ruling here. Dependency austerity is the default.

## 2. Rulings ledger (this stage's namespace; Stage-9 rulings are cited as "S9-D<n>")

**D1 — Phase order: gates before engines, policy before power, sim before hardware.**
Unlike Stages 1–9 the spec's tracks are parallel, so the order is this kitchen's to rule. RULED:
(a) **10a = Track A2 (attributes)** — the smallest reserved activation goes first because it
establishes the invariant-45 witness pattern (the suite passes identically with and without any
hint) that every later execution mode is judged by. (b) **10b = Track A3 (JIT policy)** — the
`exec.native` gate lands while no JIT exists to gate: a power that arrives after its leash cannot
have an ungated day one. (c) **10c/10d = Track B** (mailboxes+telemetry, then cycle collection) —
debt before novelty. (d) **10e–10g = Track D** (`Actuate` model → dead-man+sim → demonstrations)
— the highest-stakes track gets the most room, in sim only. (e) **10h = Track F**, **10i = Track
G**, **10j = Track H**, **10k = Track C**. (f) **Track A1/A4 last** (10l) — they depend on
external maturity (Cranelift tier config, Wasmtime threads) and deferral is a first-class outcome
(spec §2.4). Tracks may still land out of order if a dependency surfaces; any reorder is ruled
here first.

**D2 — Version discipline: the workspace stays `1.0.0` until a 1.x minor passes its own gate.**
Stage 10 ships as 1.x minors (spec header). RULED: no `-dev` pre-versions and no speculative
bumps — the version flips to `1.1.0` in the commit where the first minor's acceptance evidence is
complete, and the checklist pattern from 1.0 (a gate that can say no) is reused per minor.

**D3 — Track E cannot be self-catered, and will not be faked.**
Criterion 6 needs independently-authored packages, a third-party locale, and independent harness
adoption. RULED: nothing this kitchen writes counts toward "independent"; the criterion is
recorded PENDING-ADOPTION and the mechanisms (showcase list, channels) ship without claiming the
numbers. A close-out that needs external humans says so.

**D4 — External-maturity tracks defer honestly.**
A1's optimizing tier and A4's threads depend on what wasmtime/cranelift actually offer when 10l
opens. RULED: the phase opens with an investigation gate; "not production-ready, deferred, here
is why" is a PASSING outcome for the phase (mode honesty beats mode count), and criterion 3
carries the deferral note verbatim.

**D5 — Track G's dependencies: vetted PQC crates, network permitting; otherwise the track waits.**
House rule 5 forbids hand-rolling ML-DSA/ML-KEM. RULED: when 10i opens, the kitchen attempts to
take vetted RustCrypto implementations as dependencies with the vetting recorded here (crate,
version, KAT provenance). If this machine cannot reach a registry to add them, **the track waits
and says so** — an offline kitchen does not hand-roll lattice cryptography to hit a milestone.

**D6 — 10b ships the human-policy gate; the broker-lattice dimension lands WITH the first native
tier (10l), never after it.** A broker dimension with no enforceable operation behind it would be
dead policy data whose firing the audit chain could never witness. RULED: `--grant exec.native`
(embedded grants) + the manifest declaration + DL1906 + the authority request-stamp are 10b; the
`⊑`-checked broker dimension is a 10l entry gate — it must merge BEFORE the tier itself in that
phase, so no tier ever exists ungated. Two sub-rulings: (a) `--grant-manifest` deliberately does
NOT confer `exec.native` — the red-tier grant is named explicitly at the prompt or not at all;
(b) a `--lease` run derives `exec_native: false` unconditionally until the broker dimension
exists — fail closed, stated in code.

**D7 — Invariant 45 binds semantics and authority FACTS; the authority report's request-stamp is
the hint made reviewable, and it is the ONLY difference a hint may make.** Surfaced by the suite
itself: 10a's twin test demanded byte-identical authority reports under hints, while spec §2.3
orders `delulu authority` to report the native-emission request — both are spec text. RULED: a
hint may never change effects, capabilities, secrets, scopes, outputs, or whether a program runs;
it MAY (and for `@jit`, must) appear as the conditional `native_emission` stamp, which exists
precisely so the request is reviewable before anyone grants it. The twin test now asserts the
strong form — remove the stamp and the reports must be identical — which is stricter than the
byte-equality it replaces, because it also pins WHAT the only difference is. DL1906 is
warning-class by the same law: a hint may not change whether a program runs.

*(Ledger grows as phases surface conflicts; nothing ships un-ruled.)*

## 3. Phase plan and gates

| Phase | Track | Deliverable | Gate (verified before commit) |
|---|---|---|---|
| 10a | A2 | **DONE** (2026-07-20) — Attribute grammar activation: `@aot`/`@interpret`/`@jit`/`@inline(...)` as hints; DL1901 on unknown attributes; fmt round-trips attributes | Invariant-45 twin witnessed (run output + authority byte-identical with and without hints); DL1901 registered + explained, exact removal repair, `authority_widening: false`; both new anchors witnessed same-commit, coverage **100%**; fmt canonical own-line form round-trips; suite **931/0/4**. One parse subtlety ruled in code: attributes swallow their line terminator (Go-style termination would otherwise orphan the decl) |
| 10b | A3 | **DONE** (2026-07-20) — `exec.native` grant + manifest declaration (`[authority] exec.native = true`), authority request-stamp, DL1906 | `@jit` without the grant → DL1906 **warning, program still runs** (D7: a hint may not change whether a program runs); granted run clean; machine `--json` channel never carries the warning; authority stamps `native_emission` ONLY when requested (skip branch = byte-stability witnessed); `--grant-manifest` does not confer it and a lease derives it false (D6, fail closed); explain carries the authority-widening + no-tier honesty notes; coverage **100%** (290 anchors), suite **936/0/4**. Broker-lattice dimension: 10l entry gate per D6 |
| 10c | B2/B3 | Bounded mailboxes (`block` default / `drop-new` counted, DL1902) + `--trace-memory` telemetry | 10:1 producer/consumer mismatch sustains at stable memory; DL1902 witnessed in abort mode; telemetry present; machine channels untouched elsewhere |
| 10d | B1 | Per-actor cycle collection (trial deletion between turns) | Leak corpus (cyclic graphs, promise chains) goes documented-leak → collected; no perf cliff on the Study-C suite (>3% geo-mean regression blocks) |
| 10e | D1 | `Actuate` activates: `Cap[Actuator]`/`Cap[Sensor]`, envelope scopes, double validation, `ActuateErr`, DL1904 | Envelope refusal kills the command, never the process — witnessed both ways; kind vs scope split holds (§5.3); coverage 100% incl. new anchors |
| 10f | D2/D4 | Dead-man leases (heartbeat/TTL, broker-side revoke, declared fail-states) + `--broker-profile sim` reference simulator (deterministic, seeded) | Missed heartbeat → revoke → fail-state, latency measured and published; sim deterministic under `--seed`; artifact-hash gate (DL1905) fires in the staged flow |
| 10g | D5/DD3 | The arm demonstration + the satellite scenario (both broker roles, one host, simulated link) | Criterion 4's four behaviors measured; criterion 10's satellite semantics witnessed; recordings state sim honestly (addendum §2.5 note verbatim) |
| 10h | F1/F2 | Vendor-neutral compute interface + in-tree CPU reference adapter; `Cap[Compute]`; DL1907/DL1911; kernels-are-data laundering tests | Criterion 7 minus the hardware adapter (F3 may defer per invariant-45-style honesty); dispatch carries `ForeignCall`; authority shows the outside-the-proof line |
| 10i | G | Crypto-agile envelopes → hybrid ML-DSA/ML-KEM per D5 → KAT validation; DL1908/DL1910 | Criterion 8 or the D5 wait, stated |
| 10j | H | `delulu deploy plan`, environment profiles, DL1909; fleet-update drill (staged, hash-gated, rollback) | Criterion 9 |
| 10k | C | Advisory feed + DL1903 + `--deny-advisories`; LTS/support-matrix pages; co-evolution policy | Criterion 5's machinery (the timed LTS cycle itself needs calendar time — recorded honestly) |
| 10l | A1/A4 | Optimizing tier + threads — or their honest deferrals (D4) | Criterion 1/3 or deferral notes published |

## 4. Close-out table (spec §11 — criteria 1–11)

Opens when the last phase closes. Until then, per-phase evidence accumulates in §3's gate column
and the spec's status log. Criterion 6 carries D3's PENDING-ADOPTION marker from day one.

## 5. Diagnostics budget

DL1901–DL1911 as allocated in spec §10. No other new codes without a ruling here. The three
retired numbers (DL0503/DL0702/DL0906) are never reused (S9-D22).
