# Stage 10 Build Order — "Industrial" (tracks A–H)

**Status: CLOSED** (opened 2026-07-20 on v1.0.0 `198bf44`; closed 2026-07-20 at phase 10l,
`2d819a9`). All eleven phases 10a–10l built and committed; the §4 close-out table dispositions
criteria 1–11. **Governing documents
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

**D14 — The PQC dependency vetting (10i, discharging D5), and why post-quantum signing cannot ship
stable in this phase.** Five sub-rulings. (a) **The registry is reachable, so D5's "the track
waits" branch does NOT apply** — that branch exists for an offline kitchen, and this one is not.
The vetting was performed and is recorded here in full, because a dependency ruling whose evidence
lives only in a terminal that has since scrolled away is not a record.

**The vetting, as found on 2026-07-20:**

| | `ml-dsa` | `ml-kem` |
|---|---|---|
| Version adopted | **=0.1.1** (pinned exactly) | **=0.3.2** (pinned exactly) |
| Standard | FIPS 204 (final) | FIPS 203 |
| Repository | `RustCrypto/signatures` | `RustCrypto/KEMs` |
| Licence | Apache-2.0 OR MIT | Apache-2.0 OR MIT |
| `zeroize` support | yes (feature enabled) | yes (feature enabled) |
| Independent audit | **NONE — "has never been independently audited! USE AT YOUR OWN RISK!"** | **NONE — same warning, verbatim** |
| Vectors the crate itself tests against | Wycheproof | Wycheproof |
| Official NIST ACVP vectors shipped in the crate | **no** | **no** |

(b) **Adopted, but post-quantum signing does NOT reach stable in 10i, and the reason is stronger
than the spec's.** Invariant 51 says a PQC implementation reaches stable only after byte-exact
validation against the official NIST known-answer vectors. Two independent facts block that here,
and BOTH are published rather than one being allowed to stand in for the other: the official ACVP
vectors are not in hand (the crates validate against Wycheproof, which is a different corpus with
a different purpose), and **the implementations are unaudited by their own authors' statement.**
RULED: **KAT validation is necessary, not sufficient.** A known-answer test proves an
implementation computes the standard's answers; it says nothing about constant-time behaviour,
side channels, or conduct under adversarial input — which is what an audit finds. Reading invariant
51 as "KAT ⇒ stable" would let this project ship unaudited lattice code on a security-critical path
because it passed an arithmetic check. So the audit status is a SECOND, independent gate, and until
both clear, every post-quantum invocation without `--unstable` is **DL1910**. This is stricter than
the spec text and deliberately so; the safe direction is the only direction available here.
(c) **Versions are pinned exactly (`=0.1.1`, `=0.3.2`), not caret-ranged**, because `Cargo.lock` is
gitignored in this repo. Without a committed lock a caret range lets a fresh clone resolve a
DIFFERENT unaudited lattice implementation than the one this table describes, which would make the
vetting above a statement about bytes nobody is building. (d) **Recommendation, not yet acted on:
`Cargo.lock` should be committed.** Rust's convention of ignoring it applies to libraries; this
workspace ships a **binary** and a **signed release artifact**, and Stage 9's reproducibility claim
for that artifact quietly depends on dependency resolution nobody has pinned. Changing repo-wide
`.gitignore` policy is out of scope for a Track G phase and is flagged for the owner rather than
done unilaterally. (e) **The in-tree effort goes where §8.2 says it goes** — the envelope format,
the policy gates, and the tests — never into lattice arithmetic. House rule 5 outranks dependency
austerity; two adopted dependencies with a published audit gap is a better outcome than one
hand-rolled implementation with an unpublished one.

**D15 — Post-quantum signing (10i completion): the crypto-agile envelope, both gates live, real NIST
vectors obtained — and D14's factual predicate updated without rewriting it.** Six sub-rulings.
(a) **The envelope (`crates/delulu-runtime/src/pqc.rs`).** A self-describing `dlsig1` format —
`alg:` line naming its algorithms, one hex part per algorithm — so a new algorithm is addable
without a format break (crypto-agility, spec §8.2). A bare 96-byte blob (every artifact this
language has ever signed) still parses, marked `legacy: true`, and still verifies under the
default policy: the stability contract does not bend for a new feature. Two skip branches closed at
parse time, before any signature is checked: a declared algorithm with no bytes, and a part present
but undeclared. Neither is "verify what lines up" — a self-describing format that does not match
its own declaration has already failed at being self-describing, and the first of the two is
exactly how an envelope could claim a post-quantum guarantee it never carried.
(b) **Both policy gates are live and both are the honest direction.** DL1908 fires for a
classical-only artifact under hybrid-required policy — in EITHER classical-only shape, the legacy
blob and a `dlsig1` envelope naming only `ed25519`, because a check catching only the second shape
would wave through every artifact ever signed straight past the gate built to catch them. DL1908
ALSO fires for an algorithm id this build cannot evaluate, under EVERY policy, not only
hybrid-required — a verifier that shrugs at an unevaluable claim is the "when the checker cannot
tell, it says yes" failure wearing a crypto-agility costume. DL1910 gates BOTH signing and
verifying without `--unstable`, and verifying is the more important half to gate: verifying is
invoking unvalidated cryptography to make a trust decision, the more dangerous direction, not the
safer one. (c) **The CLI surface is additive by construction.** `sign --hybrid --unstable` and
`verify-sig --require-hybrid` are new, opt-in flags; every existing call with neither flag takes the
literal pre-10i code path — `cmd_verify_sig` now always routes through `pqc::verify`, but under the
default policy `pqc::verify` calls the same `verify_detached` internally, so "byte-for-byte
unchanged" is a checked fact, not an intention. Plugin/package artifact verification (`.dpx`'s
in-band `delulu:sig` section) was investigated and deliberately left untouched: it is an
architecturally separate 96-byte-fixed mechanism from `pqc.rs`'s envelope, and layering hybrid
policy onto it safely would mean threading a new field through the same six files
`require_signed` already spans — a feature in its own right, not a drop-in for this phase.
(d) **Official NIST ACVP known-answer vectors were obtained**, from `usnistgov/ACVP-Server`
(NIST's own repository, `gen-val/json-files/`) — real input→output pairs for ML-DSA-65 (keyGen,
sigGen, sigVer) and ML-KEM-768 (keyGen, encapDecap), saved under `measurements/pqc/vectors/` with
per-file origin URL and git blob SHA1, independently re-verified by fetching each file fresh and
hashing it again rather than trusting a self-report. One vector was run through this project's own
signing code (`ml_dsa_sign`'s key-derivation path, not a re-implementation of it) and matched
NIST's answer byte-for-byte — `crates/delulu-runtime/src/pqc.rs`'s
`nist_acvp_ml_dsa_65_keygen_seed_to_pk_matches`. Three things obtained but NOT run the same way,
named rather than hidden: `sk` (the pinned `ml-dsa` 0.1.1 exposes no public encoder for the raw
FIPS-204 secret-key layout, so nothing exists to compare NIST's `sk` field against); sigGen/sigVer
(the vectors hand you an already-expanded `sk` this crate cannot construct from, and sigVer's
vectors carry their own per-vector context while `pqc.rs` signs under one fixed domain-separator
context by design — running them would test the crate's raw API again, not `pqc.rs`); and ML-KEM
entirely (no function in `pqc.rs` calls `ml-kem` yet — Track G's KEM half is a pinned dependency
with no caller, recorded here rather than implied to exist). Full record, including every source
tried and every one that came back empty (NIST's CAVP page, both FIPS final-publication pages, the
PQC project page — all reachable, none carrying vector data): `measurements/pqc/KAT_RECORD.md`.
(e) **D14's factual predicate has changed; D14's conclusion has not — and D14's text is not
rewritten, per the D21 precedent (Stage 9): the errata is a new ruling, not an edit to the old
one.** D14b said "the official ACVP vectors are not in hand." As of this ruling they are, for both
algorithms this project depends on. D14b's actual conclusion — **KAT validation is necessary, not
sufficient** — is untouched: the vectors prove the implementation computes the standard's answers
on the cases checked; they say nothing about constant-time behaviour or conduct under adversarial
input, which is what an audit finds, and both adopted crates remain, by their own authors'
statement, never independently audited. So the second, independent gate D14b established still
holds on its own, and post-quantum signing does not reach stable in this phase regardless of the
vectors. DL1910 refuses every post-quantum operation without `--unstable`, unchanged.
(f) **Subagent attribution, recorded plainly.** Ruling D14's dependency vetting and this ruling's
integration were done by the head chef. The CLI wiring (sub-ruling c, the flags, `pqc_cli.rs`'s six
tests) and the vector research (sub-ruling d, the KAT record, the provenance-verified fetch, the
bonus KAT test) were each produced by an independent Sonnet 5 subagent — the first delegation of
this kind in Stage 10 (`docs/design/STAGE10_BUILD_ORDER.md`'s house rules do not forbid it; a
standing head-chef-only norm from Stage 9 was explicitly amended by the owner mid-phase). Every
line of both agents' output was independently re-verified before landing here: the CLI agent's
tests were re-run from a clean build rather than trusted from its report, and the vector-research
agent's central claim — that it reached NIST's actual repository rather than a substitute — was
checked by re-fetching two of the saved files directly from `github.com/usnistgov/ACVP-Server` and
confirming the bytes and git blob SHA1s matched, independently of anything the agent asserted about
its own work.

**D16 — Cloud and fleets (10j): a plan is an authority manifest, a fleet update is the same hash
gate one level up — and the first phase built by two agent PAIRS instead of two agents.** Seven
sub-rulings. (a) **The environment profile is not a new format.** `delulu deploy plan --service
NAME=DIR --env ENV.toml` parses `ENV.toml` with the SAME `delulu_runtime::parse_manifest` a
package's own `delulu.toml` already uses — an environment profile is an authority manifest for a
*place a program runs*, not a format that needed its own parser. Each service's authority is
computed the SAME way `delulu authority <pkg-dir>` computes it (the existing `pub(crate)
package_authority_value` closure, injected into `cmd_deploy` exactly the way `cmd_publish` already
receives it — no `cli.rs` visibility was widened; the closure-injection pattern this codebase
already had was the right tool, not a reason to invent one). (b) **DL1909 refuses the WHOLE plan,
never partially**, naming the exceeding service and effect by name — approving every service
except the one that widened would deploy something nobody checked against the profile, which
defeats computing the answer before anything runs (invariant 53). A plain `Diagnostic::error`,
matching DL1905's real precedent exactly: `Repair`'s `authority_widening`/`requires_human` fields
carry byte-offset edits into `.delulu` SOURCE, and there is no source span that would widen a TOML
ceiling — the "a human decides" idea lives in prose, as it already does for DL1905, not in a
struct field that does not fit. (c) **`delulu fleet update` reuses DL1905, not a new code** — spec
§9.3 names it explicitly as "the approved-hash rule (DL1905) generalized," and a fleet member's
sign-off and an actuator's are the identical question asked twice: did a human approve exactly
these bytes? A second code would be two names for one rule. The reuse is real, not just a shared
number: `fleet.rs` calls the SAME `delulu_broker::content_hash` and reads the SAME
`delulu_runtime::Approval` record 10f already built, and DL1905's `codes.rs` explanation was
extended additively (the device-case paragraph untouched, a new paragraph naming the fleet call
site) rather than duplicated. (d) **`--previous` is unconditionally required, on every invocation,
not only ones expected to fail** — the design choice this phase's own demonstration caught for
real: the first `run-demo.sh` draft omitted it on three of four passes, on the reasonable-looking
assumption that a rollback target is only needed when a rollback might happen. It is required
because "rollback artifacts are pinned at rollout start" (spec §9.3) is a statement about
*ordering* — deciding what to roll back to after a failure has already happened is deciding it too
late, and a target that is only sometimes supplied is 10f's `rate_hz` lesson again: a bound nobody
enforces is a comfort, not a control. (e) **The staged rollout is a pure state machine
(`run_rollout`), independent of any file, hash, or CLI flag** — it takes a health-check closure and
a journal closure and knows only members, so the safety property ("no member after a health
failure is ever staged") is tested by inspecting the JOURNAL, not the return value alone; a bug
that kept staging after a failure would still correctly return `RolledBack`, and only the journal
would catch it. (f) **First delegation to agent PAIRS, not single agents, per an explicit
broadened instruction** (10i delegated one agent per task; this phase's instruction was "multiple
agents for EACH task"). Two tasks, two agents apiece: a builder plus an independently-working
counterpart that could not create a file conflict with the builder because it never touched the
builder's files — an adversarial test-writer for `deploy plan` (wrote 14 tests against the fixed
contract BEFORE the implementation existed, confirmed by its own static read of `cli.rs` showing no
`deploy` arm yet, verified after landing as a check against a vacuous pass rather than tuned to
match) and a demo/record author for `fleet update` (built real fixtures — including REAL content
hashes obtained via the already-shipped `--signoff` machinery, never invented ones — against the
same fixed contract, correctly identified every value it could not yet know as an explicit
placeholder). Both `cli.rs` and `codes.rs` took one small, independent edit from each side of a
pair; both coexisted with zero real conflict, confirmed by reading the merged diff before either
pair's second agent even reported in. Every agent's work was independently re-verified before
landing — every test re-run from a clean build myself, both builders' claims about coexisting file
edits confirmed by reading the actual diffs, and the demo agent's reported contract mismatch
reproduced firsthand (`run-demo.sh` genuinely failed exactly as described) before the three-line
fix was applied. (g) **Two integration fixes, both small, neither round-tripped through another
agent.** The `--previous` omission in `run-demo.sh` (sub-ruling d) — the builder agent correctly
diagnosed the exact cause and correctly declined to edit a file that was not its own; the head chef
applied the three-line fix directly once both agents had reported. And a rendering inconsistency
between the two new commands' refusals: `deploy.rs` used the real `delulu_diag::render_human`
renderer for DL1909 (both `render_human` and `SourceMap` are genuinely public — no injection
needed), while `fleet.rs` had built a bespoke `eprintln!` for DL1905 believing the real renderer
was unreachable without widening `cli.rs`'s private `print_diagnostics`. It was already reachable;
`fleet.rs`'s refusal now renders through the identical `render_human(&d, &SourceMap::new())` call
`deploy.rs` uses, so the two sibling commands' refusals are visually consistent rather than each
inventing its own format — confirmed with the real fleet-update drill's own output, not just a
unit test, showing the renderer's `explain: delulu explain E-DL1905` footer for the first time.
Measured (Windows, n=20, whole-process wall clock): all four demo passes — a clean rollout, a
rollback, and both DL1905 refusals — cluster at 67–104 ms with **no separable cost between
accepting and refusing**, unlike 10h's compute dispatch (refusing ~8× the cost) or 10g's actuator
commands (refusing the same cost as accepting): a differential attempt (`--members 1` vs. `--members
50`) came back as pure noise (−450 µs to +430 µs, straddling zero), meaning whatever this drill's
own logic costs per member is too small to clear the floor `delulu`'s process startup already pays
at these `--members` counts — stated as a bound on what this measurement can show, not stretched
into a precision it does not have (`measurements/fleet-update/RECORD.md`). Named, not built:
capability scopes and foreign holes are not compared against the environment profile, only effects
— spec §9.2 describes the full authority answer as all three; this command checks the one that
`Manifest.effects`/`authority_of`'s existing plumbing already gave a clean, reusable path to,
and the narrower scope is stated in `deploy.rs`'s own module doc and the DL1909 explain text
rather than implied as covered.

**D17 — LTS and security operations (10k): the advisory feed is the index one level over, the gate
refuses when it cannot see, and criterion 5 splits into a built mechanism and a pending calendar.**
Built **solo by the head chef** (no subagents), reverting 10i/10j's delegation per the explicit
instruction "don't need to run multiple agents — you only do everything"; running model Opus 4.8, so
the trailer says Opus 4.8 (the environment states the running model, which settles the S9-D21
attribution concern for this phase). Seven sub-rulings. (a) **The advisory feed is not a new format,
and no new inter-crate dependency.** The registry stores advisories as JSONL under `advisories/<name>`
— one append-only file per package, one record per line — mirroring the index *exactly*, and serves
them at `GET /advisories/<package>`. The build reads a local `delulu.advisories.json` synced
out-of-band (`delulu-registry advisory export`). Client and server share only the JSON *wire* shape,
never a Rust type — `delulu` does **not** depend on `delulu-registry`, exactly as it does not for the
index (`read_local_line` returns a `Value` the client re-parses). Reading a local copy rather than
phoning the registry at build time is the "outage degrades to lockfiles" property one level up: the
registry being down never breaks a build, and never *silences* an advisory a build already synced.
(b) **Matching is exact version-string membership, deliberately NOT semver ranges.** An advisory
names affected version strings; a dependency is affected iff its resolved version is one of them.
A range predicate that could not be evaluated against some unusual version string would be a place
the checker silently answers "not affected" because it could not tell — precisely the fail-open this
detector exists to prevent. Exact membership is total: a version is in the list or it is not, and a
half-record (missing `id`/`package`/`affected`) is *counted as malformed*, never read as "matches
nothing." (c) **DL1903 is a WARNING by default, an ERROR only under `--deny-advisories`.** An
advisory is information; a toolchain that turned every advisory into a hard wall would only teach
people to reach for an ignore flag. `--deny-advisories` is the CI gate. Scoped to `command ==
"build"` (not `check`) because the supply-chain gate is a build concern and `check`'s output is
byte-stable and must not shift. (d) **THE SKIP BRANCH — a gate that cannot find its evidence
refuses.** Under `--deny-advisories`, a feed that is absent, unreadable, or carries an unparseable
record is a build *failure*, not a silent pass — the DL1905 missing-sign-off precedent applied to
the supply chain ([[skip-branch-verification-rule]]). The asymmetry is the whole point and is
load-bearing: **without** the gate an absent feed is silence (there is genuinely nothing known to
warn about); **with** the gate an absent feed is a refusal (you asked for a gate, and a gate with
nothing to check guarantees nothing). Both were witnessed, including the malformed-record case, and
the end-to-end skip branch is the 6th pass of the drill. (e) **The gate-blocked refusal is a note +
forced non-zero exit, not a DL1903 diagnostic — and that is a budget constraint resolved by
precedent, not a shortcut.** DL1903 means specifically "this dependency is on an advised version"; a
gate that could not read its feed has found no such dependency — it has found that it *cannot
answer*, a categorically different thing. Reusing DL1903 for "no feed" would blur the code's meaning;
minting a new code would exceed the DL1901–DL1911 budget (spec §10 — no free slot) for a condition
that is about the toolchain's evidence rather than the program. So it mirrors the existing
`git_deferred` refusal *exactly*: a note, and a forced exit 1, folded into the same `failed`
computation. (f) **Advisory filing is authorized by a package-scoped token — yank's standing,
reused — and the CNA nuance is named, not built.** A token files an advisory only for a package it is
scoped to (the identical `authorize(token, name)` yank takes), and an out-of-scope or unsigned filing
is refused (DL1706, tested over HTTP too). Real advisories are often filed by a third-party CNA or
security team, not the package owner; DeluluLang is **not** a CNA, registers no CVEs, and invents
none — spec §4 names "CNA registration or partner CNA" as *process*, and the owner/operator-token
model is the mechanism that fits the existing token system. The CNA-authority story is recorded as a
named boundary in `SUPPORT_MATRIX.md` and `measurements/lts-cycle/RECORD.md`, not implied as solved.
(g) **Criterion 5 splits honestly: the mechanism is built and drilled; the timed cycle is pending
calendar time.** The whole loop — release → advisory filed on the registry → feed exported →
DL1903 warning → `--deny-advisories` CI failure → backported fix → clean gate → skip-branch refusal
— is reproduced end to end in `measurements/lts-cycle/run-demo.sh` (6/6, registry as source of
truth). A *full* LTS cycle (a real 12-week train, a real 24-month backport aged in production, a real
CVE/CNA) cannot be compressed into a session and is recorded as **PENDING-ADOPTION** — the
invariant-45 / D3 posture — never faked; `DLSA-2026-0007` in the drill is a scratch identifier, not a
real advisory. Two documents published alongside: `docs/release/SUPPORT_MATRIX.md` (trains, LTS every
4th minor, 24-month windows, the schedule labelled plan-not-history since only 1.0 has shipped) and
`docs/design/VERSION_COEVOLUTION.md` (broker/protocol/DIR majors, n−1 concurrent during LTS windows,
grounded in the real `WIRE_VERSION`/`DIR_VERSION`/`PRIM_TABLE_VERSION` constants).

**D18 — The final phase (10l): both remaining Track-A items land as evidenced deferrals, and D4
makes that the passing outcome.** The two items left in Track A — A1's optimizing backend
(criterion 1) and A4's multi-threaded WASM engine (criterion 3) — both defer honestly, with their
evidence published, which is exactly what D4 ruled a passing result for this phase ("'not
production-ready, deferred, here is why' is a PASSING outcome; mode honesty beats mode count").
Built **solo** (Opus 4.8). Six sub-rulings. (a) **The optimizing backend that ships in 1.x is the
Cranelift-optimized Wasmtime tier, now explicitly pinned.** `delulu_wasm::optimizing_engine()` sets
`cranelift_opt_level(Speed)` **explicitly** — wasmtime 27's own default, so behaviorally identical
(the Stage-3 two-engine differential was re-run at 3000 programs against the pinned engine and agreed
on every one) — rather than inheriting the default, so the tier a `.dwx` runs under is a documented,
drift-proof artifact instead of an accident of an upstream default. The four run-path
`Engine::default()` sites route through it; the plugin/limits engine keeps its own `Config` (its
Windows host-safety settings are deliberately separate, per the 6f.2b comment). (b) **Criterion 1 is
NOT met, and the hot-path table is published as-is — which criterion 1 explicitly asks for.** The
optimizing (wasm) backend runs **1 of 6** compute kernels (`fib_recursive_24` at 2.0× C, a single
startup-dominated point); the other five hit DL1201 because the 1.x WASM backend compiles a
**subset** of the language. A geo-mean "on the compute-kernel suite under the optimizing backend" is
not computable over one sixth of the suite. The interpreter (the default engine, which runs all six)
is **2.0×–60.5× C**, and — the caveat cutting against us — the C lane is startup-dominated (its
spread exceeds its own minimum), so the ratios **understate** the true compute gap. v1.0 is **not
competitive with C** on this suite, stated in those words (constitution §5.11 forbids implying
otherwise). Published in `measurements/study-c/HOT_PATH_TABLE.md`, drawn from the **pinned** Study-C
results — not regenerated, since those figures are meta-test-pinned. (c) **The DIR-level optimizer is
deferred with rationale.** Cross-package inlining, monomorphization, and escape analysis — the piece
that would close the gap — is a substantial compiler needing (i) an authority-preservation *proof*
for cross-package inlining (legal in principle because rows are declared and checked, §2.1, but the
proof machinery is not built) and (ii) evidence the passes pay off (none exists). Rushing it into the
final phase would trade this stage's honesty for a mode count. **Authority-preservation across
optimization holds structurally regardless**: `delulu authority` is checker-computed before any
backend runs and embedded in the `.dwx` hash-bound to the code, so codegen-time inlining has nothing
authority-relevant to change; semantic parity is proven by the 50k two-engine differential. (d)
**A4's multi-threaded WASM engine is deferred with its honesty note — criterion 3's own sanctioned
path** ("or the track is explicitly deferred with its honesty note published"). Multi-threaded actor
execution **already ships on the interpreter** (worker-owned scheduler, `--actors-threads`,
TSAN-clean, the Stage-7 concurrency criteria met there), so the *capability* is not pending — the
deferral is about adding a *second* multi-threaded engine. The WASM engine's actor scheduler is
**cooperative single-threaded by design** (Stage 7 §6.5) with the same observable semantics. Porting
it to a genuinely multi-threaded WASM engine needs the **shared-everything-GC** proposal stack to
host actor heaps across threads; the pinned wasmtime 27 run-path engine enables neither the
wasm-threads nor the wasm-GC proposals, and that stack was not a production-ready foundation at 1.x.
§2.4 sanctions the wait ("mode honesty beats mode count"); the note is
`docs/design/THREADED_WASM_DEFERRAL.md`; it re-opens when the proposal stack matures, at which point
the Stage-7 criteria re-run on the new engine. (e) **No new diagnostics, no new anchors — coverage
unchanged at 307.** A deferral phase mints no codes, and the optimizing-tier pin is behavior-
preserving (differential-witnessed), so there is no witness churn and no floor move. Clippy baseline
unchanged. (f) **This closes the last phase; Stage 10 close-out (§4 below, spec §11) opens.**

**D19 — Post-close-out production-readiness pass (2026-07-21): cross-platform verification,
supply-chain honesty, the lockfile decision the owner reserved, and a criterion-10 acceptance test
that only passed on fast builds.** Stage 10 was CLOSED on 2026-07-20 (§4). This ruling records a
follow-on pass the owner requested — "test and verify everything, make it production-ready, across
Linux/Windows/macOS" — numbered into the same ledger rather than editing the sealed phases, per the
S9-D21 precedent (errata is a new ruling, never a rewrite). The full local CI-gate replay ran
natively on Windows and on Linux via WSL against an ext4 working-tree copy with an isolated target
dir; macOS remains static-analysis-only (no Apple hardware — stated, not implied). Built by Opus
4.8; the two decisions touching owner-reserved policy (c) and sealed acceptance evidence (e) were
put to the owner and taken by them, not made unilaterally. Five sub-rulings.

(a) **`broker_transport::state_hash` is gated `#[cfg(windows)]`.** The FNV-1a path hash names the
Windows named pipe (`\\.\pipe\delulu-broker-<hash>`); the Unix `imp` derives its socket path
directly and never calls it. Ungated it was a `dead_code` warning on Linux/macOS only —
grep-verified to have exactly one caller, inside `#[cfg(windows)] mod imp`, so the gate cannot break
another platform. Not a build failure (CI runs no `-D warnings` and no `clippy` gate) — hygiene, and
named as hygiene, not a fixed breakage. Linux `clippy --all-targets` drops 67→66 with zero
`state_hash` mentions; Windows holds at 65.

(b) **The SBOM listed a dependency it does not build and omitted two it does — and the test that
should have caught it was fool's-gold.** `docs/release/SBOM-1.0.json` named `wasm-encoder 0.252.0`
(a transitive copy pulled by wasmtime) while `delulu-wasm` declares `0.221` (resolved `0.221.3`) —
the entry now names the copy the workspace actually declares, the transitive copies noted. It
omitted `ml-dsa 0.1.1` and `ml-kem 0.3.2` entirely — both DIRECT dependencies of `delulu-runtime`
(D14c), both linked, exactly the "omission is worse than none because it will be trusted" case the
SBOM's own note warns of. Added. The regression: `release.rs::the_sbom_lists_the_real_dependencies`
asserted a hardcoded 6-crate subset that did not include the PQC crates added later, so the checker
could not see the omission it exists to catch — the skip-branch failure in a test, not a rule. The
required set now includes `wasm-encoder`, `ml-dsa`, `ml-kem` (house rule 3, applied to a test).

(c) **`Cargo.lock` is now committed, discharging D14d — the owner's decision, made.** D14d
recommended committing the lockfile ("this workspace ships a binary and a signed release artifact")
but flagged it "for the owner rather than done unilaterally," because it changes repo-wide
`.gitignore` policy. The owner chose to commit it. Two of the repo's own documents already ASSUMED a
committed lock and were therefore false: `docs/REPOSITORY_STRUCTURE.md` listed `Cargo.lock` as
"committed from Stage 2," and the SBOM note referenced "the committed Cargo.lock" — so this
reconciles the repo with what it already claimed rather than introducing a new policy. `.gitignore`
loses the `Cargo.lock` line (with an inline note that the tracking is deliberate); the structure doc
is corrected to the true date (tracked 2026-07-21, not "from Stage 2" — an aspirational comment that
was never true is not left standing). **The honesty boundary, stated:** the committed lock is the
CURRENT resolution and pins every future build; it was captured post-release and is NOT a
retroactive certificate for the already-signed 1.0.0 `.dwx`, signed 2026-07-20 before any lock was
tracked. The SBOM's direct-dep versions all appear in the committed lock (rustc is pinned, the PQC
crates are `=`-pinned, nothing was `cargo update`d between release and now), so the lock documents
that build rather than certifying it — a distinction the record keeps rather than blurs.

(d) **`.gitattributes` makes the LF invariant enforced instead of lucky.** `git ls-files --eol`
reported 472/472 tracked text files already stored LF, zero CRLF, with only the signed `.dwx`/`.sig`
as `-text` (git's own content auto-detection). Adding `* text=auto eol=lf` renormalizes nothing
(proven: the eol audit, and `git add --renormalize .` stages only `.gitattributes` itself) and
closes a real latent hazard the cross-platform audit surfaced — a CRLF reaching the lexer's raw
block-comment slice and leaking into `delulu fmt`'s "canonical" output, which the fmt gate requires
to be byte-identical across platforms. The signed artifacts are pinned `-text` so no future
`core.autocrlf` or git version can normalize the bytes a signature is a statement about.

(e) **Criterion 10's satellite test passed only on fast builds — the close-out figure rested on
that, and it is fixed now, not explained away.** `cargo test --workspace` (the primary CI gate)
builds unoptimized, and in a debug build one `fib(21)`-bearing cycle of `sat-pass.delulu` costs
~230 ms on the reference machine while the HGA grant set `heartbeat_ms=200` — so the beat, which
"rides device activity" (D11c: each command beats the lease), was DUE more often than a debug cycle
could send it, and the dead-man watchdog correctly revoked a compute-stalled controller as
`missed-heartbeat`, the exact mechanism `satellite_demo.rs` asserts must NOT be the cause (LOS must
be `ttl-expired`). Measured: release 37 ms/cycle → the demo behaves as designed (100/100 wheels,
clean TTL LOS); debug 228–232 ms/cycle → 1–2 wheels commands, everything else revoked. It even
inverts under load — a starved watchdog thread revokes LESS — which is why the same test passed in
the first (concurrent-load) run and failed in isolation. So the close-out's "criterion 10 MET /
1098-0-4" was recorded on a run where debug timing happened to keep pace; it was never robust.
**The fix is the raise-heartbeat option the owner chose, sized against the slow path:**
`heartbeat_ms=ttl_ms=1000` for the HGA (the beat window now clears the debug cycle ~4x; the HGA is
beaten every cycle so `since_beat` stays small and the lease ends on its TTL, in debug and release
alike; `ttl=1000 ms` stays well under even the fast release run so LOS still falls mid-pass) and
`heartbeat_ms=ttl_ms=600000` for the wheels (no compute gap can revoke the autonomy grant). Raising
the heartbeat forced the TTL up with it, because `ttl_ms >= heartbeat_ms` is a hard parse rule
(`value.rs`: a lease that expired before its first beat was due is refused) — the contact window is
now 1000 ms of simulated pass, semantically unchanged. Verified: `cargo test --test satellite_demo`
4/4, three runs; the raw program 6/6 deterministic in debug plus release; `run-demo.sh` and
`RECORD.md` updated to the same values with `RECORD.md`'s "Observed" block regenerated from a real
release run. **The deeper finding, named not fixed:** the dead-man watchdog uses WALL-CLOCK time
even under `--broker-profile sim`, so that profile is deterministic in its device readback (seeded)
but NOT in its lease timing — "the exact cycle at which LOS falls is machine-dependent," as
RECORD.md already said. Ticking the sim's watchdog on the sim's logical clock would decouple the
demo from interpreter speed entirely; it is a larger change to the lease/sim boundary, logged here
as future work, not smuggled into a test-timing patch.

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
| 10i ✅ | G | Crypto-agile envelopes → hybrid ML-DSA/ML-KEM per D5 → KAT validation; DL1908/DL1910 | Criterion 8 or the D5 wait, stated |
| 10j ✅ | H | `delulu deploy plan`, environment profiles, DL1909; fleet-update drill (staged, hash-gated, rollback) | Criterion 9 |
| 10k | C | **DONE** (2026-07-20) — Registry advisory feed (`advisories/<pkg>` JSONL + `GET /advisories/<pkg>` + `advisory file`/`export` CLI), `delulu build` DL1903 detector (warning) + `--deny-advisories` CI gate; `SUPPORT_MATRIX.md` (trains + LTS every 4th minor + 24-month windows) + `VERSION_COEVOLUTION.md` (n−1 majors during LTS) | Criterion 5's **mechanism** built and drilled end to end (`measurements/lts-cycle/`, 6/6, registry as source of truth); the **timed** LTS cycle recorded PENDING-ADOPTION (needs calendar time — no CVE/CNA invented). DL1903 warning-by-default / error-under-`--deny-advisories`, scoped to `build`; **the skip branch witnessed** — absent/unreadable/malformed feed under the gate refuses, never a silent pass (DL1905 precedent), while an absent feed *without* the gate is silence; exact version-string membership (no fail-open range parse); feed filed by a package-scoped token, CNA nuance named not built (D17); coverage **100%** (307 anchors, ratchet 306→307), clippy baseline 65 unchanged, suite **1098/0/4** |
| 10l | A1/A4 | **DONE** (2026-07-20) — The last phase: both remaining Track-A items land as **evidenced deferrals** (D4's passing outcome). A1: the optimizing backend that ships in 1.x is the Cranelift-optimized Wasmtime tier, now **explicitly pinned** (`delulu_wasm::optimizing_engine()` sets `cranelift_opt_level(Speed)` rather than inheriting wasmtime's default; the four run-path engine sites route through it). A4: the multi-threaded WASM engine is deferred with its published honesty note. Built **solo** (Opus 4.8) | **Criterion 1 NOT met, published as-is** — the hot-path table (`measurements/study-c/HOT_PATH_TABLE.md`) shows the optimizing backend runs **1 of 6** compute kernels (`fib_recursive_24` at 2.0× C; the other five hit DL1201), so a suite geo-mean is not computable; the interpreter is 2.0×–60.5× C and the C lane is startup-dominated so the ratios *understate* the gap; v1.0 is **not competitive with C**, stated in those words (§5.11). The DIR-level optimizer is deferred with rationale; **authority-preservation across optimization holds structurally** (authority is checker-computed and `.dwx`-embedded before any backend runs; semantic parity proven by the 50k two-engine differential, re-run at 3000 programs against the pinned engine — agreed on every one). **Criterion 3: multi-threaded WASM deferred**, its sanctioned path — multi-threaded actors already ship TSAN-clean on the interpreter; the WASM engine's scheduler is cooperative single-threaded by design; the shared-everything-GC stack needed to port it was not production-ready at 1.x and this project's engine enables neither wasm-threads nor wasm-GC (`docs/design/THREADED_WASM_DEFERRAL.md`). **No new diagnostics, no new anchors** — coverage unchanged at **307**, the pin is behavior-preserving (differential-witnessed), clippy baseline 65 unchanged, suite **1098/0/4**. D18 rules the phase and opens close-out |

## 4. Close-out table (spec §11 — criteria 1–11)

**Stage 10 CLOSED — 2026-07-20.** All eleven phases (10a–10l) built and committed; the acceptance
criteria are dispositioned below. Two criteria (5-timed, 6) carry **PENDING-ADOPTION** because they
need real calendar time or a real external ecosystem that cannot be manufactured without faking it
(D3, D17g); three land as **evidenced deferrals**, the passing outcome D4 defined ("'not
production-ready, deferred, here is why' is a PASSING outcome; mode honesty beats mode count"). No
criterion is failed-and-hidden; every gap is named, ruled, and pointed at its published note.

Verdict legend: **MET** — satisfied with witnesses in-tree. **MET / clause deferred** — core met, a
sub-clause deferred invariant-45-style with its honesty note published. **DEFERRED-HONEST (D4)** —
target not met, evidence published as-is (D4's passing outcome). **PENDING-ADOPTION** — the mechanism
ships and is drilled; the criterion's remaining half needs real-world calendar time or ecosystem,
recorded, never faked.

| # | Spec | Verdict | Evidence & disposition | Phase |
|---|---|---|---|---|
| 1 | P1 | **DEFERRED-HONEST (D4 / D18b)** | The hot-path table is published as-is (`measurements/study-c/HOT_PATH_TABLE.md`) — which criterion 1 explicitly asks for — but the geo-mean ≤ 2.5× C target is **not met**: the optimizing backend runs **1 of 6** kernels (`fib_recursive_24` @ 2.0× C; the other five hit DL1201), so a suite geo-mean is not computable, and the interpreter is 2.0×–60.5× C. v1.0 is **not competitive with C**, stated in those words (§5.11). DIR-level optimizer deferred with rationale; authority-preservation across optimization holds structurally. | 10l |
| 2 | P3/45/46 | **MET / native-tier clause N/A (D7)** | Conformance + laundering suites pass identically under every shipped mode (interpreter + WASM), proven at scale by the Stage-3 two-engine differential (50k programs). `@jit` without `exec.native` → DL1906, witnessed (`jit_policy_cli.rs`). The "JIT-tier traces identical to AOT's" clause is **vacuously honest**: no native/JIT tier ships (D7 — a hint may not change whether a program runs), and that honesty is in the code, the explain, and the authority line. | 10a, 10b |
| 3 | P2 | **MET** | Cycle-collection corpus leak-free (200/200 manufactured cycles broken in one sweep, output untouched, `Weak`-proven freeing — 10d). Bounded-mailbox backpressure sustains the rate mismatch at stable memory — witnessed at **500:1** (≥ the 10:1 asked), peak ≤ bound, zero loss (10c). Multi-threaded WASM **deferred with its honesty note** (`docs/design/THREADED_WASM_DEFERRAL.md`) — criterion 3's own sanctioned path; multi-threaded actors already ship TSAN-clean on the interpreter. | 10c, 10d, 10l |
| 4 | P4 | **MET** | The §5.5 arm demonstration reproduces from a clean checkout (`measurements/robotics-demo/`, re-run per commit). Envelope refusal, heartbeat-loss → fail-state (`safe-park` ≈255 ms), and e-stop latency (12.7 ms p50 / 39.7 ms worst) all measured within the adapter's published budget; the sim-vs-hw artifact-hash gate DL1905 fires in the staged test, both skip branches witnessed. | 10e, 10f, 10g |
| 5 | P5 | **Mechanism MET / timed cycle PENDING-ADOPTION (D17g)** | The whole LTS loop — advisory filed → feed exported → DL1903 warning → `--deny-advisories` CI failure → backported fix → clean gate → skip-branch refusal — is drilled end to end (`measurements/lts-cycle/`, 6/6, registry as source of truth). The **timed** cycle (a real 12-week train, 24-month backport, real CVE/CNA) is recorded PENDING-ADOPTION — it needs calendar time; no advisory or CNA id was invented. | 10k |
| 6 | P6 | **PENDING-ADOPTION (D3)** | Every mechanism ships and is exercised — the registry + advisory feed (10k), third-party catalog plugins (Stage 8), `for-agents.md` (Stage 8). The **counts** (≥ 10 independently-authored packages, ≥ 1 third-party locale catalog, ≥ 2 independent agent harnesses) require a real external ecosystem to form and cannot be manufactured without faking adoption. Carried as PENDING from day one. | Stage 8 + 10k |
| 7 | P7 | **MET / hardware-adapter clause deferred (invariant-45)** | The vendor-neutral compute interface passes conformance via the in-tree `cpu-reference` adapter on every CI run; over-envelope dispatch refused DL1907, **measured** (~18.7 µs, ~8× the accept path); a non-attesting adapter's grant refused DL1911, witnessed (cpu-reference attests **false** — the default path); kernels-are-data laundering tests hold (no closure crosses; unsigned artifact refused DL1913/DL1912). The **≥ 1 hardware accelerator adapter** half is **deferred, published invariant-45-style**: none ships, so invariant 49 is tested against exactly one adapter — a plausible interface, not a proven one (D13h). | 10h |
| 8 | P8 | **Gates MET / hybrid-live is the D5 wait; scrub PASSES** | Crypto-agile `dlsig1` envelope; a classical-only artifact under hybrid-required policy → **DL1908, witnessed**; DL1910 gates both sign and verify. KAT validation recorded with **NIST ACVP vector provenance** (`measurements/pqc/vectors/`, independently re-fetched and re-hashed; one vector run through this project's own signing code, byte-match). Hybrid signing is **not live-by-default**: PQC does not reach stable (both crates unaudited by their authors' own statement), so every PQ operation refuses without `--unstable` — the D5 wait, stated (D15e). The repo-wide scrub (run at close-out) finds "quantum-proof"/"quantum-safe" **only inside prohibition/caveat sentences** — 9 occurrences, none a claim (§8.3). | 10i |
| 9 | P9 | **MET** | A reference deployment's whole-authority answer is computed, printed, and approved before launch in the staged test; a plan exceeding its environment profile → **DL1909, witnessed** (refuses the whole plan, names the exceeding service + effect); one fleet-update drill exercises staged rollout, health gate, approved-hash gate (DL1905 reused per §9.3), and rollback — "nothing staged after a failure" asserted against the journal. | 10j |
| 10 | P10 | **MET** | The autonomy addendum's per-domain boundaries passed **line-by-line honesty review** (`docs/design/STAGE10_AUTONOMY_HONESTY_REVIEW.md`, 4 findings all fixed). The satellite scenario reproduces from a clean checkout (`measurements/satellite-demo/`): a contact-window lease expires at LOS, the pre-attenuated autonomy grant engages by outliving the pass, ground re-contact re-delegates — all witnessed in sim, honest about being sim, the addendum §2.5 federation note (both broker roles in one host) stated verbatim. | 10g |
| 11 | — | **SIGNED OFF** | No separate Stage-10 marketing/release prose was authored (v1.0 already shipped under Stage 9); the claims in this stage's docs — spec, build-order, measurements, Book ch. 16 — each trace to a criterion above and carry their honesty caveats (§12, verbatim). The banned-claims scrub passes (criterion 8); the performance claim says "not competitive with C" where that is true (criterion 1); every deferral is named and ruled. Honesty sign-off recorded, same discipline as Stage 9. | all |

**Net:** 6 MET, 1 MET-with-clause-deferred (7), 1 MET-with-native-clause-N/A (2), 1 DEFERRED-HONEST
(1), 1 gates-met-with-the-D5-wait (8), 1 mechanism-met-timed-cycle-pending (5), 1 PENDING-ADOPTION
(6), and criterion 11 signed off. Nothing failed silently; every gap is a named, ruled, published
deferral or an honest wait on the real world.

**Post-close-out (2026-07-21):** a production-readiness pass (**D19**) re-ran the gates natively on
Windows and on Linux (WSL), corrected three supply-chain-honesty defects in the SBOM, committed the
lockfile (discharging D14d), and fixed a criterion-10 timing fragility that had made the satellite
test pass only on fast (release) builds — it revoked its own leases `missed-heartbeat` in an
unoptimized `cargo test`. Criterion 10 remains **MET** and is now robust in debug; the "1098-0-4"
figure above was a fast-build snapshot. See D19.

## 5. Diagnostics budget

DL1901–DL1911 as allocated in spec §10. No other new codes without a ruling here. The three
retired numbers (DL0503/DL0702/DL0906) are never reused (S9-D22).
