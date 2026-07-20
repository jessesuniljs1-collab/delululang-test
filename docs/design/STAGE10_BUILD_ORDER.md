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

**D11 — The dead-man (10f): what became mandatory, what became a variant, and where the beat
comes from.** Seven sub-rulings. (a) **`heartbeat_ms`, `ttl_ms` and `fail` are MANDATORY on every
actuator grant, and each omission is refused by name.** 10e's grant form accepted an envelope with
no lease at all, which is a device grant with no dead-man — the exact hazard invariant 47 exists
to remove. Defaulting them was rejected: "what this machine does when the software stops" is an
operator's decision, and a runtime that picks quietly has made it. The 10e witnesses were updated
in the same commit; that grant form is Stage-10 syntax and has never shipped in a release, so the
stability contract is untouched. A `ttl_ms < heartbeat_ms` grant is refused too — the lease would
expire before its first beat was ever due. (b) **`ActuateErr` gains `LeaseRevoked(Str)` rather
than reusing `Envelope(Str)`.** The two demand different reactions: you clamp a bad setpoint and
retry, and you STOP when you no longer hold the machine. Forcing that distinction through a reason
string would make every control program string-match its way to a safety decision. Named
`LeaseRevoked`, not `Revoked`, because `PluginErr` already declares `Revoked` and bare-constructor
resolution requires uniqueness. The prelude shape change is why **`PRIM_TABLE_VERSION` goes 2 → 3
and why that constant's documented scope widened** to the primitive table *and the prelude types
its signatures mention*: a DIR whose `match` was checked exhaustive against two variants is not
exhaustive against three, and a version watching only the table would have passed it silently.
(c) **The beat rides device activity.** Spec §5.2 says the runtime beats "while the holding
actor's turns are healthy"; this runtime beats a device's lease on every accepted operation
against that device. The consequence is documented rather than hidden: a control loop must touch
its device at least once per `heartbeat_ms`, which is the dead-man's contract and the reason
`heartbeat_ms` is a per-device human decision. (d) **The watchdog is a thread that owes the
program nothing.** A lease that expires only when the program asks whether it has expired is a
comment, not a dead-man; so the revoke decision runs on its own tick and fires whether or not the
interpreter executes another instruction. The safety half is witnessed too — a beaten lease is
never revoked — because a dead-man that fires under a healthy program teaches operators to
disable it. (e) **"Validated twice" is claimed only as far as it is true.** The interpreter checks
the command against the capability value's scope; the broker re-checks it against the envelope the
GRANT carried. Both live in one process today, so this is structural rehearsal for spec §5.1's
host/adapter split, NOT the independent defense-in-depth a hardware deployment gets — stated in
the module docs and the DL1904 explain rather than left for a reader to assume. What it does buy
is real and unit-witnessed: if the two copies ever disagree, the grant wins. (f) **The DL1905 gate
refuses when it cannot tell.** No sign-off record is not "nothing to check" — the skip branch says
no, and the passing branch is witnessed separately so the refusals prove something. A sign-off is
written only by a clean `sim` run: a faulted run approves nothing, and a null-adapter run cannot
sign at all, because approving an artifact whose simulation fell over would make the gate certify
what it exists to catch. The sim adapter has its own skip branch, also witnessed: a `#` mirror
sensor naming a device or dimension the simulator does not model reads `NoDevice` and never falls
through to the synthetic signal — a plausible float from a joint that does not exist is precisely
the invariant-50 failure, and it would look completely normal in the output. (g) **Deferred, with
reasons.** The Book's Track D chapter lands with 10g, where there is a demonstration to describe
rather than a syntax to recite; the dead-man latency record publishes Windows only (n=20) and says
so, because extrapolating a scheduler-sensitive number across platforms is inventing evidence; and
`delulu grants revoke` on an actuator subtree (spec §5.2's e-stop) exists in the broker API
(`DeviceBroker::revoke`, `RevokeCause::Operator`) but has no CLI surface until 10g needs one.

**D12 — The demonstrations (10g): what the e-stop turned out to require, and three defects the
demonstrations found.** Eight sub-rulings. (a) **`Op::Actuate` is activated, and an actuator grant
now puts `Actuate` in the node's authority.** Stage 5 reserved the variant with
`required_effect() == None`; it now requires `Effect::Actuate`, and `Cap[Actuator].command`
round-trips to the grant tree per command (spec §5.1's synchronous class; addendum §2.5 states
that model). Without this there is nothing for an operator to aim an e-stop at: a device grant left
no trace in the tree at all. **Sensor grants deliberately add nothing.** Minting `Read` for them
would hand the node a *file-reading* effect nobody granted — the fs scope would still be empty, but
an effect nobody asked for is exactly the widening the line exists to avoid; a sensor read is
`Read` with a SENSOR scope (§5.1), and that scope lives in the device broker. (b) **An `Actuate`
custody denial is a VALUE, not a fault — the only op with that asymmetry.** Every other denial at
that gate means the program asked for something it never held; this one can also mean an operator
hit e-stop a millisecond ago. A supervisor holding four arms must lose the revoked one and keep
parking the other three, so it returns `LeaseRevoked` and the process lives (10e's law, carried up
a layer). (c) **The e-stop aims at a per-device CHILD node, not the run's own.** This was ruled by
a failing test, not by design review: watching the run's node meant `grants revoke` took the
program's console down with the arm, so the supervisor could not report the loss. Spec §5.2 says
*subtree*, so each device now holds its own child carrying `{Actuate}` and nothing else. **Both
e-stops are kept** because they stop different amounts of machine: revoking the device stops that
device; revoking the parent stops everything transitively and ends the run. The tree says which is
which, and the blast radius of each is witnessed. (d) **A device's grant node must not outlive the
run that minted it.** Found by the measurement harness, which could not stop the arm: finished runs
were leaving `[live]` device nodes, so `grants list` offered an operator several arms and no way to
tell which — if any — a live process held. Revoking a ghost prints `ok: revoked 1 node(s)` and
stops nothing. **A successful-looking e-stop is worse than a missing one, because it ends the
search for the real one.** Runs now revoke their device nodes on exit. The nodes are **revoked, not
deleted** — the audit chain records what was held and when, and erasing it to tidy a listing would
trade evidence for cosmetics. The residual is stated, not fixed: a run's own `(process)` node still
outlives it (Stage 5 behaviour, shared with every run and relied on by `run --lease`); what no
longer outlives a run is a *device*. (e) **`run --lease` refuses a local `--grant actuator=`/
`sensor=` by name, and the reason is a named gap.** The lease path validated local grants by
enumerating the kinds it forbids — so every grant kind invented after that list was written fell
through the `else` and was silently DISCARDED by `grants_from_lease`. Device grants were exactly
that: the operator typed one, was told nothing, and the program died at the mint with `DL0703:
actuator was not granted`, a diagnostic that blames the program for the CLI having thrown the grant
away. **A refusal list is a skip branch wearing a disguise.** The honest refusal is not "you may
not" but "this cannot be delegated yet": `delulu_broker::Scopes` has dimensions for files, network,
secrets and foreign libraries and **none for a device**, so a delegating party can say "you may
actuate" but not "you may slew ±5°". For an arm on a bench that is a modelling detail; for a
spacecraft it is the crux, because the ground segment is meant to be the authority. Expressing an
envelope in a grant node means extending the `⊑` lattice and the wire protocol — the most
safety-critical lattice in the system — and doing that late in a demonstrations phase would buy a
shallow version of the one thing that must not be shallow. **Deferred, named, and enforced in the
meantime.** (f) **The revocation audit line's tail differs by cause.** `overdue_us` means something
different in each of the three: a missed beat is "beat overdue by N µs", a TTL expiry is "held N µs
past its ttl" (the beats were arriving perfectly — the *loan* ran out), and an operator revoke
carries the reason instead, because an e-stop reporting `overdue by 0 µs` reads like a heartbeat
that landed exactly on time. An audit trail that says why a machine stopped has to say the right
why. (g) **The demonstrations ship as committed programs with tests, not as scripts.** Criterion 4
asks that the demonstration "reproduce from a clean checkout"; a shell script nobody runs is a
script that used to work, so the demonstration's own `.delulu` files are executed by
`robotics_demo.rs` and `satellite_demo.rs` on every `cargo test`. Those tests assert the
demonstration's *claims* and never a latency — a test pinning a number would fail on a slow machine
and teach everyone to ignore it; the numbers are measured separately and published. The runner
scripts build their own binary, because an earlier draft preferred an existing `target/release`
build, found one predating the feature, and printed a confident page of zeros. `run-demo.sh` needs
only bash; the measurement harness `measure.py` also needs python3, and that split is deliberate —
the thing a reader runs to *see* the demonstration should carry no dependency the demonstration
does not. (h) **Deferred, with reasons.** Numbers are Windows-only (n=20) and say so;
`measurements/satellite-demo` publishes no latency table at all, because nothing in that scenario
is a latency claim — its content is authority semantics. The device-envelope-in-a-grant-node gap
(e) and addendum §2.5's broker-federation gap are both RFC-gated and both restated in the
recordings rather than left in the design docs where a reader of the demo would not meet them.

**D13 — Heterogeneous compute (10h): what "enforced" means term by term, and the one thing a grant
may never say.** Eight sub-rulings. (a) **`Cap[Compute]` is a new capability kind; dispatch carries
the EXISTING `ForeignCall` effect.** No new effect, no constitutional change (spec §7.1). Inventing
a `Dispatch` effect would imply DeluluLang says something about what a kernel computes; it does
not, and the authority report prints compute under the outside-the-proof separator so nobody has to
infer that. `PRIM_TABLE_VERSION` goes 3 → 4: 10h adds both table entries (`root.compute`,
`compute.dispatch`) and a prelude type (`ComputeErr`), which is exactly the case 10f widened that
constant's scope to cover. (b) **`ComputeErr`'s variants are all NEW names**
(`KernelEnvelope`/`UnknownKernel`/`NoAdapter`), never `Envelope`/`NoDevice`. Bare constructors
resolve to the unique sum declaring them, so reusing `ActuateErr`'s names would make BOTH sums
ambiguous and break every existing 10e/10f program that matches them bare — a stability-contract
break disguised as a naming convenience. `ComputeErr` is appended LAST in both prelude paths, or
every later type's id shifts under it. (c) **Attestation is a property of the ADAPTER, never a
claim in a grant.** A grant may `waive-attestation`; it may not assert one. A grant string that
could say `attest=yes` would make DL1911 a checkbox and invariant 50's "so double enforcement is
never silently single" a sentence rather than a mechanism. Consequently **the in-tree
`cpu-reference` adapter attests `false` and always will** — it runs in this process, so there is no
layer below it, and its envelope checks are the same code in the same address space as the thing
being bounded. DL1911's refusal is therefore the DEFAULT path for the only adapter that ships,
exercised on every CI run, and a test pins the flag so a future kitchen cannot quiet the diagnostic
by flipping it. The skip branch is closed the same way: an adapter this build cannot identify is
refused, because a gate that waves through what it cannot recognise refuses exactly the honest
adapters. (d) **"Enforced" is four different statements, and three are weaker than the word.**
`memory_bytes` is checked before submission; `kernel_ms` is checked AFTER the fact, on the
measurement, with the result discarded — the work has already happened; `queue_depth` is enforced
but **unreachable from a program**, because dispatch is synchronous (unit-tested with threads
instead, and stated); `power_w` is **carried and NOT enforced** — this adapter draws no measurable
power and cannot attribute any. That last one is mandatory in every grant so a real adapter
inherits the term, and calling it "enforced" in a summary would be precisely the silently-single
failure DL1911 exists to prevent one layer up. The ledger lives in code as
`compute::ENFORCEMENT_NOTE` so it cannot drift from a document nobody re-reads. (e) **Kernels are
data, enforced twice over.** At the type level `dispatch` takes the kernel NAME as a `Str`, so a
closure cannot be *spelled* as a kernel (DL0401 at the call site, not a runtime check). At the
artifact level kernels are files on disk with detached ed25519 signatures, verified BEFORE the
bytes are parsed — reading structure out of unauthenticated bytes is how a malformed-input bug
becomes a supply-chain one. Signed DeluluLang source, named as a kernel, is still refused: a valid
signature proves provenance, not eligibility. Dispatch resolves through the VERIFIED artifacts, not
the grant string, so a failed artifact is absent rather than merely reported. (f) **DL1912 and
DL1913 are separate codes**, following DL1510/DL1511 exactly: unsigned needs signing, invalid needs
investigating, and one message cannot honestly say both. Unlike plugins there is no `require_signed`
policy toggle — a kernel with no provenance is refused everywhere, always. (g) **The kernel name is
an alias the human chose; the ARTIFACT decides what runs**, exactly as `foreign.c=LIB:PATH` binds a
lib name to a binary. A grant may bind `reduce_sum` to an artifact declaring `reduce_max`, and it
computes the max. Documented by a test rather than left to be discovered, because reading it the
other way — assuming the name guarantees the behaviour — is the mistake worth preventing.
(h) **Deferred, with reasons.** **No hardware accelerator adapter ships and none was demonstrated**
(criterion 7's second item), because this machine has no GPU compute stack that could be exercised
honestly and an adapter that cannot be run is how a deferral becomes a claim — the invariant-45
pattern from 10b, with the note published in `measurements/compute/RECORD.md` rather than dropped.
The cost is named there too: **invariant 49 is tested against exactly one adapter**, so the
interface is plausible rather than proven, and no second implementation has ever been fitted to it.
Measured numbers are Windows-only (n=20) and say so; the manifest half of "kernels named in the
manifest" (a `[authority] compute.kernels` declaration mirroring `foreign.c`'s reviewable-intent
ceiling) is NOT built — the grant enumerates kernels today, and that gap is recorded here rather
than implied to exist.

*(Ledger grows as phases surface conflicts; nothing ships un-ruled.)*

## 3. Phase plan and gates

| Phase | Track | Deliverable | Gate (verified before commit) |
|---|---|---|---|
| 10a | A2 | **DONE** (2026-07-20) — Attribute grammar activation: `@aot`/`@interpret`/`@jit`/`@inline(...)` as hints; DL1901 on unknown attributes; fmt round-trips attributes | Invariant-45 twin witnessed (run output + authority byte-identical with and without hints); DL1901 registered + explained, exact removal repair, `authority_widening: false`; both new anchors witnessed same-commit, coverage **100%**; fmt canonical own-line form round-trips; suite **931/0/4**. One parse subtlety ruled in code: attributes swallow their line terminator (Go-style termination would otherwise orphan the decl) |
| 10b | A3 | **DONE** (2026-07-20) — `exec.native` grant + manifest declaration (`[authority] exec.native = true`), authority request-stamp, DL1906 | `@jit` without the grant → DL1906 **warning, program still runs** (D7: a hint may not change whether a program runs); granted run clean; machine `--json` channel never carries the warning; authority stamps `native_emission` ONLY when requested (skip branch = byte-stability witnessed); `--grant-manifest` does not confer it and a lease derives it false (D6, fail closed); explain carries the authority-widening + no-tier honesty notes; coverage **100%** (290 anchors), suite **936/0/4**. Broker-lattice dimension: 10l entry gate per D6 |
| 10c | B2/B3 | **DONE** (2026-07-20) — Bounded mailboxes (`actor A(mailbox = N)` + `[actors]` manifest defaults; `block` default / `drop-new` counted; DL1902 in abort mode) + `--trace-memory` mailbox telemetry | The B2 criterion witnessed: a 500:1-paced producer against a bound-8 consumer sustains with **peak depth ≤ 8 and zero loss** (`block_backpressure_sustains_...`); DL1902 forced deterministically (self-send storm, drop-new, abort); **the same-worker exemption witnessed by a test that deadlocks if it's wrong**; slot release is a Drop guard (no skip branch); drops never silent; CAS-exact bound; unconfigured actors unbounded (1.0 preserved). D8 rules the deferrals. Coverage **100%** (291 anchors), suite **941/0/4** |
| 10d | B1 | **DONE** (2026-07-20) — The cycle collector: mark-and-break between turns over a worker-wide registry (List/Record cells + closure-captured scopes); `--trace-memory` reports sweeps + cells collected | The leak corpus collected: 200 manufactured `l → Link(l) → l` cycles broken in one sweep, program output untouched; the safety half witnessed at BOTH levels (unit: a reachable cycle untouched, `Weak` proves real freeing; language: a state-held cycle survives churn); non-actor programs show no collector surface at all; **Study-C gate: interp geo-mean −1.0%, no regression** (D9a). Soundness argument + five sub-rulings in D9. Coverage 100%; suite **948/0/4** |
| 10e | D1 | **DONE** (2026-07-20) — `Actuate` activates: `root.actuator`/`root.sensor` mints, envelope scopes (`--grant "actuator=DEV:dim=lo..hi[,rate_hz=N]"`), `ActuateErr = Envelope(Str) \| NoDevice`, DL1904 telemetry, `PRIM_TABLE_VERSION` 1→2 | Refusal kills the command, never the process — every refusal test asserts **exit 0** with the error handled in-program; the skip branch witnessed directly (`a_dimension_the_envelope_never_bounded_is_refused_not_waved_through` — an unbounded dimension is refused BY NAME, not waved through), plus the non-numeric twin; DL1904 lands as `command.refused` **after** the attempt record, order asserted; device named in every trace record (an audit that can't say which actuator moved is not an audit); kind/scope split holds — wrong-device mint is DL0703 at the mint while zero-grant refuses at the pre-flight, both witnessed; invariant 50 witnessed (unbound sensor reads `NoDevice`, never a number). D10 rules the bump and the `rate_hz` gap. Coverage **100%** (296 anchors), suite **955/0/4** |
| 10f | D2/D4 | **DONE** (2026-07-20) — Dead-man leases (`heartbeat_ms`/`ttl_ms`/`fail` mandatory on every actuator grant; watchdog-thread revoke; `hold`/`coast`/`safe-park` fail-states), `ActuateErr::LeaseRevoked`, `--broker-profile sim` reference simulator (deterministic under `--seed`, mirror sensors close the loop), `rate_hz` enforced, DL1905 sim-to-hardware hash gate (`--signoff`/`--approved`), `PRIM_TABLE_VERSION` 2→3 | Missed heartbeat → revoke → fail-state witnessed at CLI level **with its control** (identical program + generous heartbeat keeps the device — without it, "revoked" proves only that the phase revokes things); TTL expiry witnessed against a perfectly-beaten lease; latency measured and published (`measurements/dead-man/RECORD.md`: overdue max 6.33 ms, sim engage max 17 µs, at `heartbeat_ms=25`, n=20, Windows, terms reported separately); sim replays identically under `--seed` and DIFFERS across seeds; **both DL1905 skip branches witnessed** — no sign-off record → refused, edited artifact → refused, matching sign-off → gate seen to PASS then the honest no-adapter wall; the sim's own skip branch witnessed (a mirror sensor of a device the simulator lacks reads `NoDevice`, never a synthetic number). D11 rules the mandatory terms, the new variant, the version bump, and three named deferrals. Coverage **100%** (297 anchors), suite **980/0/4** |
| 10g ✅ | D5/DD3 | The arm demonstration + the satellite scenario (both broker roles, one host, simulated link) | Criterion 4's four behaviors measured; criterion 10's satellite semantics witnessed; recordings state sim honestly (addendum §2.5 note verbatim) |
| 10h ✅ | F1/F2 | Vendor-neutral compute interface + in-tree CPU reference adapter; `Cap[Compute]`; DL1907/DL1911; kernels-are-data laundering tests | Criterion 7 minus the hardware adapter (F3 may defer per invariant-45-style honesty); dispatch carries `ForeignCall`; authority shows the outside-the-proof line |
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
