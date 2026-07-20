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

**D8 — Bounded mailboxes (10c): what shipped, what deferred, and the two exemptions that keep it
honest.** RULED: (a) The config surface is the actor declaration (`actor A(mailbox = N)`, additive
grammar, decl wins) plus the manifest default (`[actors] mailbox = N`, `overflow = "block" |
"drop-new"`); the spec's per-SPAWN override is DEFERRED to an RFC — spawn-site config is
expression-grammar growth with three-level precedence semantics, and no §11 criterion demands it.
(b) Unconfigured actors stay UNBOUNDED — the 1.0 behavior; bounding is opt-in, so no existing
program changes meaning. (c) **The same-worker exemption:** a `block` send from a worker to an
actor that worker owns can never wait — the only thread that could drain the mailbox is the one
that would be waiting. Structural self-deadlock, refused by construction; the bypass is visible
in telemetry (peak past the bound) and witnessed by a test that deadlocks in seconds if the
exemption is wrong. Cross-worker cycles of full mailboxes CAN still deadlock — spec §3's
documented non-guarantee: backpressure bounds memory, never liveness. (d) The mailbox-slot
release rides a Drop guard in the worker's Send arm, so no early-continue path (dead actor,
unknown behavior) can leak a slot — the skip branch closed by construction. (e) An unknown
`overflow` value warns and means `block` — a typo must be audible, not a silent policy change.
(f) B3 splits: 10c ships mailbox telemetry (`--trace-memory`: per-actor bound/peak/drops); heap
bytes and collection counts arrive with 10d's collector, where a heap walk exists. (g) DL1902 is
error-class ONLY in abort mode (abort mode is the statement that losing work is worse than
stopping); otherwise a drop is counted per actor and reported at exit, never silent.

**D9 — The cycle collector (10d): mark-and-break between turns, and why that is sound.** The
actor model pays for the collector's simplicity: **between turns, a worker's only live roots are
its actors' states** — locals died with the turn, continuations do not exist (§5.8), and module
globals are immutable pure constants (§5.5) that can never come to reference a turn's
allocations. So trial deletion reduces to mark-and-break over a registry of the cells a cycle
can pass through: List and Record cells (the mutable back-edges) and closure-captured scopes (a
captured `var` can hold its own closure). RULED: (a) registration happens ONLY inside turns —
non-actor programs pay one predictable branch per allocation, and the Study-C gate result is the
receipt: **interp geo-mean −1.0% vs the committed baseline** (single re-run, minute-granularity
noise on the small benches; the ≤3% gate passes with the sign pointing the wrong way for a
regression). (b) The sweep triggers at 64 registered cells (the amortizer) and runs in the
worker loop with EVERY live state on that worker as a root — the registry is worker-wide, so the
root set must be too. (c) Telemetry reports CELL COUNTS, deliberately not bytes — a byte figure
without a real size walk would be an invented number; B3's "per-actor bytes" is re-scoped to
this honest form. (d) Primitive results register their top-level cell only; nested fresh cells
in prim results are either the top cell or clones of eval-site-registered cells, and the ruling
records that reasoning rather than leaving it implicit. (e) The rcap system itself resists
cycles — the corpus needed explicit `ref` annotations to build one, which is the type system
making the collector's job rare, and worth recording. (f) The collector's unit tests prove
actual freeing via `Weak` handles, and prove the safety half (a reachable cycle is NEVER
touched) — a collector that frees live data is worse than a leak.

**D10 — The physical boundary (10e): what the envelope refuses, and what the bump costs.** Six
sub-rulings. (a) **`PRIM_TABLE_VERSION` goes 1 → 2, and that is the honest price of activation.**
Four primitives entered the table, so a DIR compiled against table 1 no longer describes this
runtime; it refuses with DL1503 rather than pretending the two tables agree. A version that never
moves is a version that means nothing. (b) **The envelope is fail-closed in every branch, and the
skip branch is the whole point.** A command must be a record; every field must be numeric; every
field must NAME a bounded dimension; and the value must lie in the inclusive range. The tempting
bug — check the dimensions the envelope knows and let the rest through — would mean an envelope
grants everything it forgot to mention, so an unbounded dimension is REFUSED, by name. NaN is
refused by construction (`!(x >= lo && x <= hi)` rather than a negated comparison chain, so the
unordered case falls to the refusing side). (c) **The refusal is a VALUE, never a fault.** A robot
that panics mid-motion is worse than one that declines a step and keeps its control loop alive, so
`command` returns `Result[Unit, ActuateErr]` and DL1904 is **telemetry** — a trace record with op
`command.refused`, following DL1305's denied-attempt pattern. Both records appear, attempt then
refusal: an attempt that was refused is still an attempt, and hiding it would hide intent. (d)
**Sensor reads are `Read`, not a new effect** — observation is observation (§5.1). The mint is
pure attenuation like every `root.X()`; the effect is in USING the capability. (e) **Invariant 50
holds through the null adapter**: with no simulator bound (10f's territory), `read()` returns
`NoDevice` and never a number. A control loop handed `0.0` by a sensor that isn't there will act
on it — an absent measurement must be *absent*, not plausible. (f) **`rate_hz` parses and is
carried but is NOT enforced**, and this is recorded as a gap rather than implied to work: rate
limiting without a dead-man lease is a comfort, not a control, and both arrive together in 10f.
Double validation likewise lands here only in its checker/runtime half — the broker half is 10f's,
and the explain text says out loud that neither replaces a hardware interlock.

*(Ledger grows as phases surface conflicts; nothing ships un-ruled.)*

## 3. Phase plan and gates

| Phase | Track | Deliverable | Gate (verified before commit) |
|---|---|---|---|
| 10a | A2 | **DONE** (2026-07-20) — Attribute grammar activation: `@aot`/`@interpret`/`@jit`/`@inline(...)` as hints; DL1901 on unknown attributes; fmt round-trips attributes | Invariant-45 twin witnessed (run output + authority byte-identical with and without hints); DL1901 registered + explained, exact removal repair, `authority_widening: false`; both new anchors witnessed same-commit, coverage **100%**; fmt canonical own-line form round-trips; suite **931/0/4**. One parse subtlety ruled in code: attributes swallow their line terminator (Go-style termination would otherwise orphan the decl) |
| 10b | A3 | **DONE** (2026-07-20) — `exec.native` grant + manifest declaration (`[authority] exec.native = true`), authority request-stamp, DL1906 | `@jit` without the grant → DL1906 **warning, program still runs** (D7: a hint may not change whether a program runs); granted run clean; machine `--json` channel never carries the warning; authority stamps `native_emission` ONLY when requested (skip branch = byte-stability witnessed); `--grant-manifest` does not confer it and a lease derives it false (D6, fail closed); explain carries the authority-widening + no-tier honesty notes; coverage **100%** (290 anchors), suite **936/0/4**. Broker-lattice dimension: 10l entry gate per D6 |
| 10c | B2/B3 | **DONE** (2026-07-20) — Bounded mailboxes (`actor A(mailbox = N)` + `[actors]` manifest defaults; `block` default / `drop-new` counted; DL1902 in abort mode) + `--trace-memory` mailbox telemetry | The B2 criterion witnessed: a 500:1-paced producer against a bound-8 consumer sustains with **peak depth ≤ 8 and zero loss** (`block_backpressure_sustains_...`); DL1902 forced deterministically (self-send storm, drop-new, abort); **the same-worker exemption witnessed by a test that deadlocks if it's wrong**; slot release is a Drop guard (no skip branch); drops never silent; CAS-exact bound; unconfigured actors unbounded (1.0 preserved). D8 rules the deferrals. Coverage **100%** (291 anchors), suite **941/0/4** |
| 10d | B1 | **DONE** (2026-07-20) — The cycle collector: mark-and-break between turns over a worker-wide registry (List/Record cells + closure-captured scopes); `--trace-memory` reports sweeps + cells collected | The leak corpus collected: 200 manufactured `l → Link(l) → l` cycles broken in one sweep, program output untouched; the safety half witnessed at BOTH levels (unit: a reachable cycle untouched, `Weak` proves real freeing; language: a state-held cycle survives churn); non-actor programs show no collector surface at all; **Study-C gate: interp geo-mean −1.0%, no regression** (D9a). Soundness argument + five sub-rulings in D9. Coverage 100%; suite **948/0/4** |
| 10e | D1 | **DONE** (2026-07-20) — `Actuate` activates: `root.actuator`/`root.sensor` mints, envelope scopes (`--grant "actuator=DEV:dim=lo..hi[,rate_hz=N]"`), `ActuateErr = Envelope(Str) \| NoDevice`, DL1904 telemetry, `PRIM_TABLE_VERSION` 1→2 | Refusal kills the command, never the process — every refusal test asserts **exit 0** with the error handled in-program; the skip branch witnessed directly (`a_dimension_the_envelope_never_bounded_is_refused_not_waved_through` — an unbounded dimension is refused BY NAME, not waved through), plus the non-numeric twin; DL1904 lands as `command.refused` **after** the attempt record, order asserted; device named in every trace record (an audit that can't say which actuator moved is not an audit); kind/scope split holds — wrong-device mint is DL0703 at the mint while zero-grant refuses at the pre-flight, both witnessed; invariant 50 witnessed (unbound sensor reads `NoDevice`, never a number). D10 rules the bump and the `rate_hz` gap. Coverage **100%** (296 anchors), suite **955/0/4** |
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
