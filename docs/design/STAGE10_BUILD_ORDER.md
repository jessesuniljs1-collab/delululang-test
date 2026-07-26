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
`heartbeat_ms` is a per-device human decision. (**Refined by D43a**, which found what "touch" had to
mean on the *stepped* clock: a REFUSED command correctly never beat the lease, but it also never
advanced simulated time, so a program whose every command was refused held its device forever in
simulation while losing it on the wall clock. This sentence was accurate about beats throughout; the
gap was in the clock, not in the beat rule.) (d) **The watchdog is a thread that owes the
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

**D20 — The simulator's dead-man now ticks on a logical clock (finishing D19e's deferral): a
demonstration replays identically because its timing is a function of the command sequence, not of
interpreter speed.** D19e named this and left it as future work; the owner asked for it finished.
Ruled in five parts. (a) **A new clock mode, scoped to the simulator and opt-in.** `device.rs`
gains `ClockMode::{Wall, Stepped { step_us }}`. `Wall` is unchanged real time — every hardware
profile and every existing dead-man/e-stop test takes it byte-for-byte (the 17 device unit tests
pass untouched, because `DeviceBroker::new`/`with_authority_watch` still construct `Wall`).
`Stepped` advances a simulated-microsecond counter by a fixed step on each device interaction and
sweeps expiry SYNCHRONOUSLY at that interaction. It is reachable only through the new `with_config`
constructor, which the CLI resolves solely for `--broker-profile sim`; `--sim-step` on any other
profile is refused (exit 2), because a hardware dead-man is a real-time promise and must never be
quietly stepped. (b) **The honest limit is in the code and a test, not just the prose.** A stepped
clock advances only on interaction, so a program that STOPS interacting stops the clock and its
lease does not expire — `stepped_mode_does_not_model_the_wedged_program_only_a_wall_clock_can_catch`
asserts exactly that. The wedged-controller guarantee (a looping or blocked program losing its
actuator in real time) is `Wall`'s alone, and the module docs and this ruling say so rather than let
"deterministic sim" imply a safety property it does not carry. (c) **One arithmetic, two clocks.**
Heartbeat-before-TTL lives in a single `due()` helper shared by the wall-clock watchdog and the
stepped sweep, so the two clocks cannot drift in how they decide a lease has died; the watchdog
SKIPS heartbeat/TTL under `Stepped` (the sweep owns it) but still runs the operator e-stop probe, so
a Guard-driven revoke reaches the device in either mode. `overdue_us` and the engage latency become
simulated under `Stepped` (0 engage, exact overdue), so even the trace's numbers reproduce.
(d) **Proven by the property that motivated it.** The satellite demo under `--sim-step 50` is
byte-identical across a debug build and a release build — stdout AND the effect trace — despite the
debug interpreter costing ~230 ms/cycle against release's ~37 ms; `hga_commanded × step` is constant
across steps 25/50/100/200 and the HGA is revoked exactly one step past its TTL, so TTL enforcement
is exact, not approximate (the wide slew is refused interp-side before it reaches the broker, so a
cycle advances the clock by its two accepted interactions). `satellite_demo.rs`, `run-demo.sh` and
`RECORD.md` adopt `--sim-step 50` and the recording is regenerated from a real run.
(e) **`delulu authority` and the Guard are untouched.** This changes only the device broker's lease
*clock*; the authority/effect computation, the grant tree, the ⊑ lattice, the audit chain, and the
Guard's custody validation are not in the change and their suites are unaffected — scope confirmed by
the owner mid-build. Coverage unchanged (no new diagnostics, no anchors); three new device unit
tests witness the stepped clock's determinism, its simulated heartbeat, and its honest limit.
**Portability:** `device.rs` carries no `#[cfg]` and no OS call — pure `std` (`AtomicU64`, `Instant`,
`Mutex`) — so the code Windows and Linux both run green is byte-for-byte the macOS path, and the
stepped clock is arithmetic rather than wall-clock. macOS stays analyzed-not-run (no Apple hardware,
per `CROSS_PLATFORM_VERIFICATION.md`), a status D20 neither improves nor worsens.

**D21 — D12e is discharged: `Scopes` gains a `device` dimension, so a delegation can say "you may
slew ±5°" and not merely "you may actuate". Shipped as RFC 0001 phase F1, ahead of that RFC's
comment period — a process deviation recorded here rather than hidden.** D12e deferred this with a
stated condition, not an indefinite "later": extending the `⊑` lattice and the wire protocol is
"the most safety-critical lattice in the system", and doing it "late in a demonstrations phase would
buy a shallow version of the one thing that must not be shallow." That condition is now met — this
is not a demonstrations phase, and the work carries its own witnesses. Ruled in seven parts.

(a) **The subset relation is the opposite of the obvious guess, and that is the whole risk.** An
envelope's dimension list is a **whitelist**: `envelope_check` walks the *command's* fields and
refuses any field the envelope does not name ("the envelope cannot vouch for what it never
bounded", `value.rs`). So **more dimensions is WIDER** — each one admits a command shape that was
refused before — and a child naming a dimension its parent lacks is a widening, refused. A
reasonable implementer reading "constraints" instead of "whitelist" would have inverted this and
produced a silent widening. It is stated in the module docs next to the code that enforces it, and
`dropping_a_dimension_narrows_and_adding_one_widens` pins both directions.

(b) **Every other term tightens in the direction that costs the holder authority.** `rate_hz`: a
bounded parent may not delegate to an unbounded child. `heartbeat_ms`: child `≤` parent, because a
*smaller* heartbeat is stricter. `ttl_ms`: child `≤` parent. `fail`: exact match only —
`hold`/`coast`/`safe-park` have no safety order, since which is safer is device-dependent, so there
is no minimum to take and conservative is sound. The meet preserves `ttl_ms >= heartbeat_ms`
automatically (the side owning the smaller ttl has its own heartbeat below it); that is proved in a
comment and checked anyway, dropping the device if it were ever violated.

(c) **The latent trap that was actually there, and is now closed.** `Op::Actuate` has been sending
the device path to the broker on **every actuator command since 10g** (`interp.rs` passes
`env.device` as the check arg), where `validate.rs` accepted it via `_ => true` — an arm whose
comment still read *"reserved: no scope argument to validate"*, written before 10g activated the
op. This was **not a live fail-open**: `Scopes` had no device dimension to check against, and the
numeric envelope is genuinely enforced runtime-side. It was a rule waiting to die in a fall-through,
of exactly the shape this project has been bitten by three times. The wildcard is now an
**exhaustive match**, so a future `Op` forces a decision instead of defaulting to allow.

(d) **The fix is pinned by a witness that was OBSERVED to fail against the old code**, not merely
asserted to. `Op::Actuate` was temporarily reverted to `true`, and
`actuate_on_a_device_the_node_was_never_granted_is_refused` and
`the_actuate_effect_alone_no_longer_commands_every_device` both failed; the third new test
(attenuation) correctly still passed, being a different path. A regression test nobody has watched
fail is a test nobody knows is connected.

(e) **Two parsers for one grammar, pinned mechanically rather than culturally.** `delulu-broker`
depends on neither `delulu-runtime` nor vice versa (crate ruling 1), so the canonical grant STRING
is the contract and each side parses it independently.
`device_grant_strings_round_trip_between_the_runtime_and_broker_parsers` renders every shape the
grammar admits, parses it with both, and compares field by field — including that the runtime can
re-read the broker's canonical rendering. Drift fails a test instead of silently costing a bound.
**New refusal at parse: non-finite bounds.** `"NaN".parse::<f64>()` succeeds in Rust, and a NaN
bound would both defeat every comparison and break the reflexivity `DeviceScope`'s `Eq` promises —
the refusal and that `impl` are one decision, not two.

(f) **A lease now confers physical authority — only the authority the delegating side bounded.**
`grants_from_lease` builds actuators from the node's own device scopes, so `run --lease` flies the
corridor it was delegated. The old blanket refusal splits: `--grant actuator=` is still refused, but
the reason changed from "nobody could bound this" to "you do not get to bound it *yourself*", and
`--grant sensor=` keeps the original reason unchanged because `Scopes` still has **no sensor
dimension** (a sensor read is `Read` under a sensor scope). The addendum §2.2 UAS two-grant
lost-link pattern is now witnessed end to end in `device_delegation_cli.rs`: a mission grant and a
strictly-attenuated lost-link grant over the same device, the same program flying both, the wide
deflection refused under the narrow grant, and a *widened* second grant refused at delegation time
(DL0802) rather than at use. **No new diagnostic code**: an ungranted device is `OutOfScope` in
dimension `device` → DL0904, and a widening is DL0802 carrying the never-widening intersection.

(g) **The process deviation, stated plainly.** `rfcs/README.md` requires an RFC with a **≥ 14-day**
comment period for any change to "effect/authority behaviour", and says the period "does not shrink
because a release is near". Adding a dimension to the `⊑` lattice is squarely that, and
`STAGE10_AUTONOMY_ADDENDUM.md` §2.2 explicitly called D12e **RFC-gated**. RFC 0001 exists and
scopes this as phase F1, but it is a **draft with no sponsor and no comment period served** — an
AI-authored RFC may not name its own sponsor. This work shipped on the owner's direct instruction,
which is authority over this repository but is *not* the same thing as the comment period, and the
project's own rule is that a deadline is not a reason to skip the part where people disagree with
you. Recorded as a deviation, not reframed as compliance. What would make it right: a named sponsor
signs RFC 0001, the period runs, and **if the RFC is amended or rejected, F1 changes with it** —
the code is not grandfathered by having landed first. Byte-compatibility was preserved deliberately
so that reversal stays cheap: `device` is omitted from `Authority::to_json` when empty, so every
authority without a device scope hashes exactly as before in the audit chain.

**Verified (both runnable platforms, sequentially and isolated).** Windows: `cargo test --workspace`
89 suites / 0 failed, clippy **65/0** — the exact pre-F1 baseline, so ~600 new lines added zero
warnings — coverage 100%, `--check-reference` in sync, `fmt --check` 0 would change (7 examples, 10
book samples), python-less build clean. Linux (WSL, ext4, isolated target dir): 89 suites / 0
failed, clippy **66/0** (its own baseline), every gate exit 0. macOS remains analyzed-not-run: the
new module is pure `std` with no `#[cfg]` and no OS call, so the code both platforms run green is
the macOS path — which is an argument, not a run, and is not counted as one.

**D22 — Broker federation ships (RFC 0001 F2–F5): a grant tree that spans machines, closing
addendum §2.5. The broker still never opens a network socket, and two real defects were found and
closed on the way.** §2.5 called federation "a prerequisite for any real deployment in this
addendum's domains" and RFC-gated it; RFC 0001 is now sponsored (Jesse Sunil, comment period
2026-07-22 → 2026-08-05) and F2–F5 were built **during** that open period at the sponsor's
direction — a smaller deviation than D21(g)'s but still one, recorded in the RFC, `rfcs/README.md`,
and here. Ruled in nine parts.

(a) **The credential is an artifact, not a connection.** Federation is mediated by three signed,
self-contained documents the operator's *existing* link carries — grant certificate (`dlcert1`),
contact receipt (`dlrcpt1`), audit bundle (`dlbundle1`). No async runtime, no listener, and
`broker_transport.rs`'s "OS-authenticated same user" guarantee is left exactly as it was, because
nothing was added to that transport. DeluluLang does not own the radio, and this is why it does not
have to.

(b) **A lease token could not have been stretched to do this**, and the reason is structural rather
than a matter of effort: `Token` carries **no authority bytes** (only a reference into the minting
broker's `HashMap`) and is MAC'd with a **symmetric** key, so any party able to verify is able to
mint. Sharing that key between ground and vehicle is the first thing anyone proposes and the worst
option available — one captured vehicle key would mint ground authority.

(c) **Federation introduces no new authority mathematics.** A chain is verified by running the
existing `attenuation_check` at every hop. There is deliberately no second `⊑` implementation; a
second one is a second place for the rule to die. This is the property to protect through review.

(d) **Cryptography is injected, never re-implemented.** `delulu-broker` depends only on
`delulu-diag`/`delulu-check` (crate ruling 1), so it declares a `SignatureVerifier` trait and the
`delulu` crate supplies ed25519 from the runtime's adopted `ed25519-dalek` — the same idiom as
`IdSource`/`ClockSource`. `verify()` returns **the signer's public key rather than a bool**, because
"this signature is valid" and "the named issuer signed this" are different claims and conflating
them accepts a validly-signed forgery. Post-quantum inherits `pqc.rs`'s DL1910 refusal rather than
carving an exception: an `ml-dsa-65` certificate is refused, never treated as unsigned.

(e) **The sharpest skip branch in the design, implemented and tested.** An unknown scope dimension
or unknown effect **refuses the whole certificate** (DL1418). `docs/for-agents.md` tells consumers
to *ignore unknown fields* — correct for a reporting surface, catastrophic for an authority: a
dimension a verifier cannot see is one it cannot enforce, so ignoring it silently **widens** the
grant. The two rules must be stated together wherever either is stated, or one will be applied to
the other's domain. Likewise an algorithm this build cannot verify is refused rather than treated as
unsigned — verifying is a trust decision, so refusing to verify must refuse the trust.

(f) **Revocation cannot cross a partition; expiry can — so expiry is the mechanism.** A certificate
may carry `uplink_ttl_ms`, and the holder runs only that long without a signed contact receipt. A
30-day mission certificate with a one-hour uplink term is revocable in an hour instead of
un-revocable for 30 days. Receipts are bound to one certificate fingerprint (so a receipt for a
harmless grant cannot be moved onto a powerful one), must be signed by an **anchor** (so a vehicle
cannot renew its own lease), may never push a lease past the certificate window (proof of contact is
not a grant of authority), and extend **monotonically** (so replaying a stale receipt is a harmless
no-op rather than a way to strip a vehicle of authority — shortening is `revoke`'s job).

(g) **TWO REAL DEFECTS, both found by asking whether the mechanism could be defeated.**

  1. **Replay could undo a revocation.** Nothing stopped adopting the same certificate twice, so
     `grants revoke` on an adopted node — the only tool an operator has while the link is up — was
     defeated by re-presenting the credential. A chain may now be adopted **once per broker
     lifetime**, which is exactly the scope of the revocation it protects (the tree is in memory, so
     a restart clears both together and legitimate post-restart recovery still works).
  2. **A subtree could outlive its root's lease.** `attenuate_core` checked that a parent was live
     at creation and bounded a child's authority by `⊑` — but never bounded the child's *deadline*,
     and `effective_state` judged a single node. A holder could therefore delegate itself a child
     with `ttl_millis: None` and keep commanding after its own lease died. Locally that was a latent
     wrong; under federation it is fatal, because the uplink lease is the only bound that survives a
     partition and **the party it bounds is precisely the party that can mint children**. Fixed by
     inheriting expiry (`effective_state_inherited` walks to the root), routed through every
     enforcement *and* reporting path so an operator is shown what the broker would decide.
     Inheriting at read time rather than clamping at write time is deliberate: a contact receipt
     extends the root and the whole subtree must come with it.

  Both fixes are pinned by witnesses **observed to fail** against the old code, not merely asserted
  to. For (2), `node_view` was temporarily reverted to per-node expiry and exactly the two new tests
  failed while the other twenty-two passed.

(h) **Audit chains are cross-linked, never merged — because merging is impossible, not merely
undesirable.** A chain is `blake3(prev_hash ‖ record)` over one broker's monotone `seq`; splicing
two would invalidate every hash after the splice, so a "merged" log would be a lie or a rewrite, and
this is the one artifact whose value is that it is neither. The receiver writes one `reconcile`
record naming the bundle's digest, range, and the sender's head — **the mechanism already in the
tree**, where a new day file's first record carries the previous file's last hash. **A tampered
bundle is an INCIDENT, not a denial** (exit 1, recorded, withdraws nothing): the log is
observability, not enforcement, and making reconciliation gate operation would quietly convert it
into an enforcement input. `seq` is per-broker, so a combined timeline is not a global order and the
success line says so.

(i) **What this does NOT close, so the closure is not read wider than it is.** Both brokers run on
one machine and the "link" is a filesystem copy — there is no radio, no latency, and no partition
except one the tests create by letting time pass. **No hardware adapter ships in-tree**, so every
device is still simulated. Multi-hop depth > 1 is expressible and exercised at depth 2, no further.
Sensors still have no scope dimension. Certification is unchanged and remains **none** (addendum §3),
and the WCET/hard-real-time refusals stand. Federation makes a real deployment *possible* to
design; it does not make one *done*.

**Diagnostics:** DL1415 (chain does not verify to an anchor), DL1416 (a hop is not `⊑`, carrying the
intersection), DL1417 (outside the validity window), DL1418 (unsupported algorithm/dimension, or
malformed). Each has an explain body and **both** an accepting and a rejecting conformance witness —
the `--coverage` gate refused them until they did, exactly as the RFC predicted. **No code was added
for the uplink lease or for bundle failure**: the uplink lease *is* the node TTL, so an expired
uplink is the ordinary DL1402 swept by machinery that already exists, and a bad bundle is the
existing DL1405. The RFC had penciled in DL1419/DL1420; not adding them is the better answer, since
a parallel expiry path would have been a second place for liveness to be wrong.

**Verified (both runnable platforms, sequentially and isolated).** Windows: `cargo test --workspace`
90 suites / 0 failed, 0 build warnings, clippy **65/0** — the exact pre-federation baseline across
roughly 2,600 added lines — coverage 100%, `--check-reference` in sync, `fmt --check` 0 would
change, python-less build clean. Linux (WSL, ext4, isolated target dir): recorded in
`CROSS_PLATFORM_VERIFICATION.md`. macOS remains analyzed-not-run: `cert.rs` and `device_scope.rs`
carry no `#[cfg]` and no OS call, so the code both platforms run green is the macOS path — an
argument, not an execution, and not counted as one.

**D23 — The first real hardware adapter: `Profile::Hw` stops being a gate with nothing behind it.
An operator-supplied SUBPROCESS, not the signed plugin spec §5.4 anticipated — and the difference
is stated rather than blurred.** Ruled in six parts.

(a) **A subprocess is the right first adapter, and the reasoning is not "it was easier".** The
property that matters for custody is not "did a servo move" but **does the command leave
DeluluLang's guarantee**. A subprocess crosses exactly that boundary: the bytes go to code this
project did not write, cannot type-check, and must not trust. Every architectural question worth
answering — envelope enforcement before dispatch, a hung driver not wedging the control loop, a
lying driver unable to widen anything — is fully in play and testable without a laboratory. It is
also how drivers actually attach in the field (a serial bridge, a CAN gateway, a ROS node, a vendor
SDK shim); writing serial framing into the runtime would have picked one bus and one vendor, while a
process boundary picks none.

(b) **This is NOT what spec §5.4 described, and the gap is a real one.** §5.4 says hardware adapters
are Verified-class Stage-6 plugins with `require_signed: true`. That would buy **supply-chain
assurance** — you would know who wrote the driver, and the loader would refuse an unsigned one.
`--adapter-cmd` buys **isolation and reach** instead: a separate process that cannot corrupt the
runtime and can be any program on the machine. It carries **no signature check whatever**. The two
are complementary, not substitutes, and no material may describe this as satisfying §5.4. What
DL1905 approves is the **artifact** — the DeluluLang program's exact bytes — and it says nothing
about the driver. The operator chooses the driver by typing the flag, and the operator is inside the
trust boundary (spec §10). The signed-plugin path remains unbuilt and remains named.

(c) **The ordering is the whole point, and it is proven by evidence DeluluLang cannot see.** The
envelope is checked host-side, against the grant, **before one byte reaches the driver**. A driver
can therefore refuse *more* — a hard stop, a thermal limit, a fault — and can never permit more,
whatever it replies. D11e observed that host-side and adapter-side checks lived in one process,
making the ordering "structural rehearsal"; with a subprocess the split is real. The test that
establishes it reads **the driver's own log**: a program commanding 12° (in envelope) and 999° (out)
produces exactly **one** line in that log. From inside DeluluLang "refused before dispatch" and
"dispatched and rejected" look identical; the log is the only place the difference is visible.

(d) **Four fail-closed rules, each with a witness.** A reply that is not exactly `OK` is a protocol
error, never acceptance (`"ok"`, `"OKAY"`, `"ACK"`, `"true"`, `"1"`, `""` all tested). A silent
driver times out on a deadline rather than wedging the loop — blocking stdio has no portable
deadline, so a reader thread feeds an `mpsc` channel and the exchange uses `recv_timeout`: std only,
no async runtime, the same thread-plus-channel shape the dead-man watchdog already uses. **A failed
adapter stays failed** (poisoned), because once framing is in doubt a late reply would be read as
the answer to the *next* command — which is how a robot executes yesterday's instruction. A garbled
or non-finite sensor reading is an error, never a `None` and never a number: invariant 50 means a
broken driver must not be mistaken for an unplugged sensor, since those call for different
responses.

(e) **A `hw:` profile with no driver REFUSES.** Not a silent no-op: a program told "COMMANDED" while
the machine never moved is the worst failure mode available here. Likewise a driver that will not
start fails the run **before `main`**, rather than letting the program discover the machine is
unreachable partway through a motion. And no driver is spawned at all until DL1905 has passed, so a
hardware process is never started for bytes a human did not sign off on — tested by asserting the
driver's log does not exist.

(f) **What has NOT changed.** No driver for any real device ships in-tree, and every demonstration
in this repository still commands the in-tree simulator. `STAGE10_AUTONOMY_ADDENDUM.md` §4's "no
hardware ships in Stage 10" stands, as does §3's certification claim of **none**. This ships the
socket a driver plugs into, not the driver — and running the adapter against a shell script proves
the socket works, not that anything physical moved. Invariant 52 is untouched: the hardware safety
chain must still function with DeluluLang absent, and what a driver does with a *permitted* command
remains below the boundary.

**Verified (Windows):** 92 suites / 0 failed, 0 build warnings; clippy **65/0** — still the exact
baseline; coverage 100%; `--check-reference` in sync; fmt 0-change; python-less clean. A pre-existing
test (`a_matching_signoff_passes_the_gate_and_then_stops_for_want_of_an_adapter`) pinned the old "no
adapter ships in this build" wall and correctly failed; it was updated to pin the *new* honest wall
(`--adapter-cmd` is missing) rather than weakened — the gate is still seen to pass, and the run still
refuses rather than pretending.

**D24 — Two unbounded loops in the Stage-1 parser are closed, and the guard that both were missing
now exists in exactly one place. The fix is for the class; the instances were symptoms.** Opens the
hardening campaign (`HARDENING_CAMPAIGN.md` C1). Ruled in five parts.

(a) **Both defects are real, were observed, and are reachable from code a person would type.**
`match flag { true => n = 1 … }` — an arm body is an expression and assignment is a statement, so
`parse_expr` stopped at `=` and nothing consumed it. The arm loop's condition was unchanged, so it
ran again, pushing one more `Arm` each pass: **CPU pegged and resident memory 650 MB → 1.16 GB in
three seconds**, sampled live, until the machine ran out. Separately, an `import` after the first
item spins with **no** allocation — `parse_item` returns `None` without consuming and `recover_item`
deliberately stops *at* `import` — so it pegs a core silently and forever, which is harder to
notice than the one that eats the machine. Both reproduced at `rc=124` before the fix and terminate
with a correct diagnostic after it.

(b) **This is the skip-branch lesson, not a novel hazard.** The parser's author knew this failure
mode precisely and defended against it **four separate times** with the same hand-written idiom
(`let before = self.pos; …; if self.pos == before { self.bump(); }`) — in the foreign-block loop,
the actor-body loop, the statement-block loop, and inside `recover_item`. The two loops that lacked
it are exactly the two that hung. A rule that is known, written down, and applied by hand is a rule
that will be omitted somewhere; the omission is the defect, not the ignorance.

(c) **RULED: the guard is structural from here.** `Parser::parse_until` is now the only loop over a
closing delimiter in the parser, and every brace-delimited list — statements, actor members,
foreign functions, match arms, and the module's item list — goes through it. Progress is guaranteed
by construction: a step that consumes nothing has one token consumed on its behalf, so iteration
count is bounded by token count. It is no longer possible to *forget* the guard, because there is
no longer a hand-written list loop to copy from.

(d) **The enforcing witness fails rather than hangs.** A regression test for a hang is close to
useless: reintroduce the bug and the test wedges CI instead of reporting. So the load-bearing
witness is structural — `every_delimited_list_loop_goes_through_the_progress_guard` scans the
parser source and asserts exactly one such loop exists. It earned its place on first run by
catching an occurrence inside a comment that a manual `grep` over the same file had missed.

(e) **Both diagnostics were sharpened, because both defects were reached by plausible code.**
Generic messages are what made these expensive to diagnose: "expected `=>`, found `=`" points three
tokens past the mistake, and "expected an item" is true of a perfectly well-formed `import` line.
They now state the actual rule — an arm body is an expression; `import` belongs before the first
item — and the arm case carries an exact repair (`brace-match-arm-assignment`) that wraps the
assignment in a block. **No diagnostic code was added or changed**: DL0201 and DL0208 keep their
identities, so the machine surface is untouched and the stability contract is not engaged. Prose is
explicitly not the contract (`for-agents.md`), which is what makes this improvement free.

**D25 — The front door is rebuilt, and writing the guide found two real defects the whole test
suite could not: a user function silently losing to a builtin, and a named function that
type-checks as a value and cannot be called.** Hardening campaign P1 (`HARDENING_CAMPAIGN.md`
C4–C6, C8, C10–C13). Ruled in six parts.

(a) **The documentation was the primary defect surface, and the evidence is not an opinion.** The
P0 sweep wrote one honest program per domain using only what the Book, samples, examples, and
reference teach. Most did not compile, every failure looked like a language limitation, and **none
of them was** — sum types are `type T = A | B(X)`, `List.get` returns `Option[T]`, a behavior that
sends declares `! {Async}`, `Net` is an effect while `Http` is a resource kind. Rewritten against
the real surface the same programs check clean and run. The project had tested its documentation
for **accuracy** and never for **sufficiency**: every claim in it is true, and a developer cannot
get from them to a working program.

(b) **RULED: a teaching sample is not documentation until a gate runs it.** `book.rs` already
holds the right principle — "a tutorial whose examples do not compile teaches people something
false" — and implements half of it. Checking proves a program is *well-typed*; it says nothing
about whether it *works*, and the gap is not hypothetical: the reference row-polymorphism sample
checks clean and cannot run, and additionally has no `fn main`, so running it was never possible.
`crates/delulu/tests/examples_run.rs` adds the missing half over `examples/` and `examples/guide/`:
every file checks clean, and every file with a `main` is run and must not fail with **DL0907**, the
code the runtime raises when the checker let something through. The assertion is deliberately
narrow — an ungranted capability or a missing file is a legitimate outcome and the gate says
nothing about those.

(c) **C13 is a checker/runtime divergence and is the most serious defect of the campaign so far.**
`apply(double, 21)` type-checks — row polymorphism exists so that it can — and faults at runtime
with "unbound name `double`". A lambda in that position always worked; only a *named* top-level
function was missing from `eval_var`. `SOUNDNESS_AUDIT.md` §C examines function values and closes
the channel correctly: the analysis was right about the types and the interpreter simply had no
case. A named function now evaluates to a closure over the globals — exactly the environment
`call_fn` builds for a direct call — so calling by value and calling by name are one computation.
Verified by removing the fix and observing the new gate fail by name, together with all three unit
witnesses, then restoring it.

(d) **C11's fix refuses rather than shadows, and dislodged a latent host panic.** A user
`fn parse_int` was accepted and every call to it silently resolved to the builtin, so the only
symptom was a type error at a *call site* naming a type the author never wrote. Builtins are
intercepted before user scope, so the declaration is now refused with **DL0302 — an existing code**,
which keeps the machine surface and the stability contract untouched. Refusing beats letting the
user's definition win: otherwise a call would mean different things in different modules. Out with
it came `check_fn`'s `.expect("fn in table")` — the invariant "every `Item::Fn` is in the table"
was held by nothing but resolve never skipping registration, and the first skip turned a bad
program into a process crash. A host panic is never an acceptable answer to a bad program (the
Stage-9 D15 lesson, restated).

(e) **The README told a new reader the project was an unbuilt skeleton.** On a v1.0.0 tree with
Stage 10 closed and 23 rulings it said *"Stage 1 ('Skeleton') — under construction"* and
instructed `rustup default stable` against a pin that exists to make builds reproducible. It now
states the real status, the platforms actually verified, that **macOS has never been executed**,
that **no release binary or public repository exists** so you build from source, and that **there
is no LICENSE** — so, by default copyright, nobody else may legally use any of this. That last one
is recorded and **deliberately not fixed**: choosing a licence is a legal commitment belonging to
the copyright holder alone, and it is the single hardest blocker to the campaign's own objective.

(f) **DL0703 now names the flag.** Zero ambient authority means the first program anyone writes
fails until a human grants the console — correct, and the entire point. But a refusal that does not
say what to type teaches nothing, so all eleven refusal messages name the exact grant
(`--grant console`, `--grant fs.read=./data`, `--grant secret:NAME=VALUE`, and the rest, echoing
the actual path or device). Prose only: codes, spans, and the `--json` envelope are unchanged.

**D26 — DeluluLang refuses Trojan Source: raw bidirectional control characters in source are an
error (DL0107). The language whose purpose includes reviewing AI-written code will not accept
source whose rendering can be inverted against its meaning.** Hardening campaign P2
(`HARDENING_CAMPAIGN.md` C3). Ruled in four parts.

(a) **This is the highest-value Stage-1 security property, not a lint.** A file with a U+202E
override in a comment checked clean and could be made to *render* as the opposite of what it *runs*
— the Trojan Source attack (CVE-2021-42574). For most languages that is a review nuisance; for one
whose flagship use case is a human (or an AI) reviewing code another AI wrote, defeating review is
the entire attack. So it is refused outright, matching Rust's deny-by-default, not warned about.

(b) **RULED: DL0107 is authorized in the DL01xx lexer range** (this §5 requires a ruling for any
new code; the D22 federation codes DL1415–DL1418 set the precedent that a new code lands in the
range of the *subsystem* it belongs to, not mechanically in DL19xx). A bidi-control refusal is a
lexical property, so DL0107 — the next free slot after DL0106 — is the consistent home. The three
retired numbers (DL0503/DL0702/DL0906) remain retired; nothing is reused.

(c) **The design is one scan of raw source, ahead of tokenizing.** The rule cannot die in a
per-token branch that forgot it (the skip-branch discipline), and because it reads raw bytes the
`\u{202e}` escape — ASCII in source, visible in review — is untouched, keeping the legitimate
string-data case open. Raw right-to-left *letters* are never refused: the target is reordering
*control* characters, and breaking Arabic or Hebrew string data would be its own discrimination,
against the constitution's no-discrimination stance. All four properties are witnessed, and the
reject file was seen to check clean on the pre-fix binary before it refused after.

(d) **Stability.** A program carrying a raw bidi control is *touched* by this phase, so its channel
changing is within the contract; every program without one lexes byte-identically (the scan finds
nothing, emits nothing), so the machine surface is unchanged for all non-attack input. This is the
same judgement Rust made shipping the mitigation in a point release: refusing an attack is not a
breaking change to any program a user should have been relying on.

**D27 — DeluluLang is licensed: Apache-2.0 for the code, a trademark policy for the name, with
Jesse Sunil permanently recorded as creator. This is the one ruling in the campaign made by the
owner, not the kitchen.** Hardening campaign P1 finding C9. Owner-approved 2026-07-24.

(a) **Licensing was owner-reserved and was treated that way.** The campaign's standing rule is that
licensing, philosophy, governance, the public specification, and backward compatibility are never
decided autonomously. C9 (no LICENSE → default copyright → nobody may legally use the project) was
therefore presented as a recommendation and **held** until Jesse chose, even though it was the
single largest adoption blocker. Recorded so the discipline is legible: the hardest blocker was left
open on purpose rather than resolved without authority.

(b) **RULED by the owner: Apache-2.0, and derivatives take a different name.** The two-tool
structure — permissive code licence for the code, trademark policy for the name — is established
practice (Rust, Python, Mozilla), so no legal language was invented. Apache-2.0 over MIT for two
concrete reasons Jesse's goals required: its **§4(d) NOTICE mechanism** makes the creator
attribution legally sticky through redistribution (MIT cannot force that), and its **explicit patent
grant** matters for a language aimed at robotics and autonomous systems. Files: `LICENSE` (verbatim
Apache text), `NOTICE`, `TRADEMARK.md` (different-name rule), `GOVERNANCE.md`; plus
`license`/`authors` on all twelve crates and the project's own licence added to the SBOM component.

(c) **The name is protected without restricting the code.** The Apache licence covers the code and
grants no rights in the name; `TRADEMARK.md` governs the name. Anyone may use, modify, sell, and
fork DeluluLang; a modified or derivative *language* must ship under a different name and not present
itself as the original. This is exactly the constitution's no-discrimination stance — every user,
human or AI, has the same freedom — with the one narrow protection Jesse reserved: that no one can be
misled about what the original DeluluLang is or who created it.

(d) **The genuine ambiguity was escalated, not guessed.** One sentence in the brief could have meant
"rename derivatives" or "keep the name"; the honest reading (rename, consistent with name-protection)
was recommended but flagged as the owner's call, and Jesse confirmed rename. A licensing decision
resolved by an assistant's guess is precisely the kind of thing this ruling exists to prevent.

**D28 — The supply chain could lie about secrets: the semver-authority law and `authority --diff`
were blind to secret-scope widening. Closed, surgically, without a lockfile format break. A related
deeper hole (the pin and self-declaration layers) is presented to the owner, not auto-fixed,
because it changes acceptance behavior.** Hardening campaign P3 (`HARDENING_CAMPAIGN.md` C18/C19).
Ruled in four parts.

(a) **The hole was real and against invariant 10.** Reading a secret adds no effect and no
capability kind — only a name — and the lock entry never recorded secret names. So `authority_widened`
and `authority --diff` saw byte-identical authority across a version that quietly added a secret
read, and `delulu lock` waved the patch bump through. Constitution invariant 10 lists *scopes* among
what may not widen silently, and a secret name is a scope; Stage 2 is the stage that exists to make
that true for the supply chain. Verified by two witnesses observed to fail against the pre-fix code.

(b) **RULED: fix the widening detection, not the hash.** The lock entry gains a `secrets` field and
the semver-authority law + `authority --diff` treat a new secret as a widening. The `authority_hash`
is deliberately left over effects+kinds — its documented meaning — because folding secrets in would
change every existing hash and fail any committed lockfile with DL1002 on the next build, and
**backward compatibility is owner-reserved**. The security property closes anyway: a same-version
secret change moves the `content_hash` and is caught by DL1010. Choosing the smaller change that
still closes the hole is the point.

(c) **The deeper layer (C19) is owner-reserved and was NOT auto-fixed.** `check_self_authority`
(DL1009) does not require a package to declare the secrets it reads, and `check_pins` (DL1001) does
not constrain a dependency's secrets against the consumer's pin — the same blindness at the first
review gate. The fix is small and breaks nothing in-tree, but it changes what the **checker
accepts** (a package reading an undeclared secret would begin to error), which is a
backward-compatibility change the owner reserved. It is presented with full analysis and held —
the licensing discipline (D27) applied to a language-behavior change. Recommendation on record: make
it, because it completes invariant 10 for secrets.

(d) **This is hardening, not redefinition.** Nothing about the authority lattice, the `⊑` relation,
or the Guard changed. A dimension the model already contained (secrets, present in `PackageAuthority`
and in `AuthoritySpec`) is now *checked* where it was computed-but-dropped. That is exactly the
campaign's harden-never-redefine rule.

**D29 — The two execution engines disagreed on faults, and a WASM trap flooded the terminal with a
16,000-line backtrace. Engine parity (invariant 15) is now true for fault codes, and no trap ever
dumps a backtrace.** Hardening campaign P4 (`HARDENING_CAMPAIGN.md` C20). Ruled in three parts.

(a) **The divergence was real and the differential fuzz was structurally blind to it.** The same
faulting program gave `DL0902`/`DL0901`/`DL0905` on the interpreter and a generic `DL0904` — plus,
for deep recursion, **16,326 lines** of guest backtrace — on the WASM engine. Invariant 15 promised
byte-identical stderr and identical exit codes; both were false for faults. The 50k-program
differential harness missed it because it counts any `(Err, Err)` as agreement without comparing the
faults. Root cause: the WASM run captured `e.to_string()` on the wasmtime error, which both appends
the full backtrace and discards the structured trap.

(b) **RULED: map the structured trap to the interpreter's code; never emit a backtrace.**
`delulu_wasm::clean_trap` downcasts to `wasmtime::Trap` and maps the deterministic traps
(`IntegerDivisionByZero → DL0902`, `IntegerOverflow`/overflow-`unreachable` → DL0901, `StackOverflow
→ DL0905`, out-of-bounds → DL0903), returning a single clean line; unknown traps keep only their
first line. The CLI's exit-code mapper reads the embedded code, so both engines now report the same
code and exit for the same fault. One residual is documented, not hidden: `%`-by-zero and overflow
both trap via `unreachable` and are indistinguishable from the trap alone, so `%`-by-zero is DL0901
on WASM where the interpreter says DL0902.

(c) **Invariant 15 was over-stated and is now precise.** "Byte-identical stderr" cannot hold when
one engine faults inside the guest with no source span; the honest and enforceable contract is
identical **stdout**, identical **exit codes**, and agreeing fault **codes** — which is what tooling
and agents match on. The spec now says exactly that. This is tightening a guarantee to what is true
and testable, not weakening it: the previous wording was a claim the code never kept.

A separate finding surfaced writing the witness and is recorded as C21 (OPEN), not fixed here: the
interpreter's `MAX_DEPTH = 10_000` guard overflows the *host* stack below ~20 MiB, so a small-stack
embedding crashes before DL0905 fires. That is the interpreter's stack discipline, not engine
parity, and wants its own pass.

**D30 — No declaration may shadow a builtin type name or a core effect name. Sixteen type names and
ten effect names were shadowable, and every shadow was silently inert.** Hardening campaign P5
(`HARDENING_CAMPAIGN.md` C23).

(a) **How it was found, and what it was not.** The Stage-4 marshallability fence is an allowlist
matched by NAME, so the attack is to make something else answer to one of those names:
`type Int = Secret[Str]` plus a foreign signature naming `Int` **checked clean**. It is not
exploitable — both engines lower foreign signatures by name through one shared path
(`lower_foreign_sig`), the call site types `Int` as the builtin, and a real `Secret[Str]` value is
still refused DL0602 — verified by running it. Invariant 20 holds.

(b) **What was actually broken was review integrity, and that is not a lesser property here.** All 16
builtin type names (`Int … Contained`, including `Root`, `Cap`, `Secret`, `Plugin`) and all 10 core
effect names were accepted as user declarations and then had no effect anywhere, because `lower_type`
and `lower_row` match builtins before user scope. `effect Write` is the authority-bearing case: the
author believed they had declared a private effect while every `! {Write}` still meant the one that
reaches the filesystem. A source file could say `type Cap = Int` and mislead every later reader
without ever failing — in a language whose premise is that authority is legible from source.

(c) **RULED: refuse at the definition site, DL0302, on all three declaration-table paths.** Same code
and same reasoning as C11's prelude-builtin refusal (S-D25): refuse rather than pick a winner,
because either winner makes one name mean two things depending on where it is read. `resolve.rs`,
`program.rs`, and `deps.rs` each enforce it, and the package path is separately witnessed — a rule
that holds on two paths out of three holds nowhere. The name lists are single constants
(`PRELUDE_TYPES`, `CORE_EFFECT_NAMES`) beside the matchers they mirror, walked by the tests.

**Compatibility, stated plainly:** this refuses programs that previously compiled. Every such program
contained a declaration that did nothing, so no working behaviour changes — but a tree containing one
now fails, which is a real (and intended) break.

**D31 — A foreign signature still refuses type aliases, and now says so usefully.** Hardening
campaign P5 (`HARDENING_CAMPAIGN.md` C24). The fence is **deliberately** name-based and stays that
way: both engines share one lowering that cannot see module aliases, so expanding an alias in the
checker and not in the marshaller is how ABI confusion starts — the runtime would have marshalled
`FKind::Unit` for `Meters`, silently substituting a value. Ruled: keep the refusal, name the alias's
target, and attach an Exact repair writing it; re-classify an alias expanding to a function type from
DL1301 to **DL1302**, since R-6a is about what the type means. The resolver walk is **bounded at 32
hops** because `type A = (A)` is accepted (C16) and an unbounded walk would have introduced a hung
compiler as part of the fix.

**D32 — The authority report states the credential-exposure conclusion it already had the facts
for.** Hardening campaign P5 (`HARDENING_CAMPAIGN.md` C25), discharging the commission's requirement
that DeluluLang *tell* users when code exposes credentials. A program that declassifies `API_KEY` and
hands it to `msvcrt.puts` was run end to end; the secret was printed by the C function, with every
gate behaving correctly (row declared, manifest permitted, human granted). The gap was that the
report read before granting listed `Declassify`, the secret name, and the foreign lib on three
separate lines and never joined them.

Ruled: a gated `exposure:` line, derived entirely from facts already computed. It reports
**capability, not behaviour**; it names the safe case ("no egress in its row") as well as the unsafe
one; and it changes **nothing** about what the language permits — no new refusal, no change to any
grant relation, no new authority concept, so the Authority guardrail is untouched. The `--json`
channel is deliberately unchanged: it already carries `effects`, `secrets`, and `foreign_calls`, so
an agent could always derive this and only the human could not. Reports for programs without
`Declassify` are byte-identical, including the pinned Stage-3 report.

**D33 — A build that checked nothing no longer reports success, and a directory named where a file
belongs gets a sentence instead of an OS error code.** Hardening campaign P5 (C26, C27). A package
whose sources sat beside `delulu.toml` rather than under `src/` printed
`built clean (1 package(s), 0 module(s))` and exited 0; it now refuses on the posture the deferred-git
gate already used — a check that could not run must not report success — and the closing
`0 error(s)` line (which read as success beside a nonzero exit, and affected the pre-existing git and
advisory refusals too) now states the reason instead of counting errors that were never the problem.
`delulu run <dir>` reported the raw OS error (`Access is denied. (os error 5)` on Windows, `Is a
directory` on Linux — misleading, and differently misleading per platform); one fix in the shared
`load` helper covers every file-taking command and points at `delulu build`.

**D34 — The manifest ceiling and the dependency pin now bound SECRETS, not just effects.** Hardening
campaign P5 (`HARDENING_CAMPAIGN.md` C19), **owner-approved 2026-07-25** after being held as a
backward-compatibility decision.

The same blindness as D28, one layer earlier and more consequential, because the pin is the *first*
review gate. `root.secret("X")` contributes no effect and no capability kind — only a name — so:
`check_self_authority` (DL1009) let a package read any secret while declaring none, and
`scope_violations` (DL1001) constrained a dependency's effects, `net`, and `fs` but never its secrets,
so a consumer who pinned *which* secrets a dependency may read was not actually constrained. The pin
was decoration. `AuthoritySpec` already carried the field and `package_authority` already computed the
value; nothing read either.

Ruled: enforce computed `secrets ⊆ manifest.secrets` (DL1009) and `dep.secrets ⊆ pin.secrets`
(DL1001). The pin check follows the **same empty-means-unconstrained convention as its siblings** —
inventing a stricter default for secrets alone would be surprising, and would be a second, unruled
compatibility change riding along. Secret names compare exactly; unlike paths there is no prefix
relation between them. This completes invariant 10 ("scopes") for the secret dimension.

**Compatibility, stated plainly:** a package that read an undeclared secret, or a pin that omitted a
dependency's secrets, previously checked clean and now errors. Nothing in-tree read a secret from a
*package* manifest, so the in-tree cost was zero, but downstream trees will see new errors — which is
the point of the rule.

**D35 — Surface-syntax morphs are built. A program's keywords may be Chinese, emoji, or a short-alias
AI profile, and it is the same program.** Hardening campaign C22, built on the owner's instruction
2026-07-25. `docs/design/SYNTAX_MORPH_SPEC.md` had been normative and unimplemented since Stage 8,
with a header claiming otherwise (corrected first, in commit `5aea47d`; this ruling covers the build).

(a) **The spec's own law had a hole, and it is the part worth remembering.** §1 required a morph to be
bijective, single-token, and free of alias-vs-alias collisions. That permits `let = "fn"`: bijective,
one token, renders and round-trips perfectly — and a file written in it uses the word `fn` to mean
`let`. For a language whose premise is that a human or an AI can *review* code another AI wrote, a
surface that lies to the reviewer is the same class of attack as the bidi controls D26 refuses. Ruled:
**an alias may not be another keyword's canonical spelling (DL1711)**, added to the spec as rule 1a.
Bidi controls inside an alias are refused on identical reasoning (DL1712).

(b) **RULED: the parser does not resolve the pragma.** Resolving `//! morph: X` means reading a file
off disk, and a parser that acquires filesystem authority from a comment in its own input is **ambient
authority inside the compiler** — the exact thing this language exists to eliminate. `delulu-syntax`
accepts an already-loaded `Morph` and never touches the filesystem; the CLI, which already holds that
authority, does the lookup. This is why `morph_file.rs` lives in the `delulu` crate and not beside
the law it loads.

(c) **RULED: exactly two conversion points, and canonical byte offsets for spans.**
`lexer::lex_with_morph` is the only place a non-canonical surface becomes tokens; the CLI's file
loader normalizes a pragma-bearing file at the edge. Parser, checker, DIR, hashes, both engines, and
every report see canonical and cannot distinguish surfaces — verified by asserting that a
Chinese-keyword program's authority report is byte-identical to its canonical form's. Spans are
canonical byte offsets exactly as §1 requires; rendering never adds or removes a line, so lines are
exact, while a column within a converted line can shift by the keyword-length difference and the
quoted snippet shows the canonical spelling. That trade is documented rather than hidden.

(d) **Scope shipped is enumerated, not implied.** Built: the law and validation, both rendering
directions, the lexer hook, the TOML format and search path, `delulu morph list|info|check|render`,
pragma-aware `check`/`run`/`authority`/`fmt`, DL1710–DL1714, and two working morphs
(`morphs/zh-CN-keywords.toml`, `morphs/compact-ai.toml`). **Not built:** `.dpx` plugin delivery,
per-reader LSP view morphs, `fmt --to-morph` flags (`morph render` does that job), the `[style] morph`
repo policy key, and morph-aware **package** builds — a package's `src/` must be canonical, which is
what the spec itself recommends for shared projects.

Evidence it is real rather than plumbed: a program whose keywords are Chinese prints
`fib(10) = 55` through `delulu run`, its string literals and identifiers untouched, and converting
back yields the original bytes. Emoji, Cyrillic, Greek, and mixed-script morphs round-trip in the
property tests — the alias rule denies *structural* hazards rather than allowing a list of scripts,
because the point of the feature is that the surface belongs to whoever reads it. No token-savings
number is claimed anywhere; savings are tokenizer-specific (Constitution §5.11).

**D36 — A lease token for a dead grant no longer redeems, and the audit read path no longer presents
a broken chain as authentic.** Hardening campaign P6 (`HARDENING_CAMPAIGN.md` C29, C30). Both findings
are the same defect wearing two coats: the machinery to answer the question existed, and the code that
needed the answer did not ask.

(a) **C29 — `redeem` never asked whether the node was alive.** It verified the MAC over the whole
payload, confirmed the node existed, and checked the token's own `exp_millis`. A token for a REVOKED
node redeemed `Ok`; so did one under a revoked ancestor, and one under an *expired* ancestor whose
token carried no deadline of its own. Only the case where the token's deadline happened to mirror the
node's TTL was refused, incidentally rather than by design.

Not privilege escalation — `validate` re-reads effective state per operation, so a redeemed dead node
authorizes nothing — but for a system whose product is accountability the consequences are the point:
the redemption wrote an audit record reading `decision: "allow"` for a grant an operator had killed,
and `set_holder_peer` stamped the redeemer's own text onto the revoked node. RULED: call
`effective_state_inherited` after the MAC check and before the nonce is burned or any state written,
so a refused redemption mutates nothing. Revoked → DL1403, expired → DL1402, reusing the codes the
enforcement path already uses.

(b) **C30 — the audit READ path verified nothing.** `audit verify` recomputes every hash and link and
refuses at the failing seq (DL1405); `tail`/`query` called none of it. A record whose `decision` was
flipped displayed the forged value with no warning, and a record corrupted into non-JSON **vanished
from the listing** — 1, 2, 4 shown, 3 gone, no gap marker. An entry can be erased from the record of
what happened by corrupting one line.

RULED: verify on read, then **still display** the records — an operator investigating a tampered log
is exactly who most needs to read it — behind a warning that they must not be trusted and that
anything corrupted beyond parsing is missing entirely. Nonzero exit so a script cannot treat a corrupt
read as clean; `--json` always carries `chain_verified`, plus `chain_error` when false, so a machine
never infers integrity from an absent field. The log stays **observability, not enforcement**; what
changed is that the observation is honest about itself.

**D37 — The Guard gains a `device` class, and its op→axis map is now exhaustive.** Hardening campaign
P6 (`HARDENING_CAMPAIGN.md` C31).

`Scopes` has eight dimensions; the Guard enumerated seven. `tier_for_mint` walked a fixed
`[…; 7]` array and `use_axis_class` ended in `_ => None`, so when RFC 0001 F1 added `device` neither
grew and **neither could fail to compile**. `Op::Actuate` — active, round-tripping per command — was
born with no axis of its own.

Stated precisely, because overstating it would be wrong: actuation was **still gated**, through the
cross-cutting `effect:Actuate` rule, at mint and at use. What was missing is per-item granularity.
`device` was the only authority axis without it — an operator could write `net:api.example.com` or
`secret:DB_PASSWORD`, but for devices only "all actuation" or "none". On the one axis that moves
physical hardware, a thruster could not be sealed while a status LED stayed at `warn`.

RULED: add `GuardClass::Device`, gating on the device NAME (the envelope stays bounded by `⊑`, not by
policy patterns); extend the mint walk to it; map `Op::Actuate` to it. **No default rule** — adding
one would change behaviour for existing device holders, and the tier physical actuation deserves is an
operator's decision, not a library's. And the durable half: `use_axis_class` is now **exhaustive**,
naming the ops that genuinely have no scope dimension, so the next `Op` cannot be born ungated in
silence — the build breaks until a person answers "what gates it?"

**D38 — The `--json` contract is enforced at one choke point, and diagnostic output is bounded.**
Hardening campaign (`HARDENING_CAMPAIGN.md` C2, C32, C33), commissioned as a crash hunt across the
compiler and the CLI with an explicit framing correction from the owner: **no discrimination between
the surfaces** — a human may drive the CLI and an agent may drive the compiler, so a machine-contract
break and an unreadable wall of stderr are the same class of defect, not one each.

(a) **C2 — every `--json` command must emit one object, and on failure most emitted none.**
`docs/for-agents.md` states the promise; a usage or I/O error broke it on essentially every
subcommand, printing a human sentence to stderr and exiting nonzero with zero bytes on stdout.
RULED: enforce it in `cli::run`, wrapping the whole dispatch, **not** at the ~161 `return 2` sites — a
rule enforced per site is a rule the next site forgets. The fallback envelope carries the documented
fields, sets `summary.errors = 1` so the documented pass test (`summary.errors == 0`) stays correct,
and **invents no DL code**: the registry is a stable contract and a usage error is not a language
diagnostic, so `diagnostics` stays empty and the reason goes in an additive `error` object.

The gate tests **exactly one** object, not at least one, which is how it caught the mirror defect
twice — `delulu test` and later `deploy` both already printed a report, so the fallback made two.

(b) **C32 — a 10 KB file produced 76 MB of stderr.** Every diagnostic quoted its entire source line,
and a 5000-deep field chain is one 10 KB line with ~5000 errors against it, each printing that line
twice (text and underline). Measured 76,518,387 bytes in 14.2 s. Same class as D29's guest backtrace:
past some volume, output is no longer a diagnostic but a denial of service against its reader.
RULED: bound the **human** channel twice — a 160-character window around the span (char-indexed, `...`
on the elided side, caret arithmetic corrected; lines under the limit render byte-identically) and a
50-diagnostic cap with a note stating exactly how many were withheld. `--json` stays uncapped, because
it is a contract to report every diagnostic. Result: 23,530 bytes in 0.125 s.

(c) **C33 — `deploy` and `fleet` worked and `--help` listed neither.** That is why the first sweep
missed them, and why `deploy`'s double-emit survived. Both are documented now, and a gate asserts every
dispatched subcommand appears in `--help`: an undocumented command is a command nothing sweeps.

**Also verified and worth recording as a negative result:** twenty-six hostile programs (2000-deep
parens, 800-deep generic types, 20 000-term expressions, a 200 KB literal, a 100 000-character
identifier, 6000 functions, a 3000-field record, unterminated literals, NUL bytes, empty and BOM-only
files) and the full CLI × malformed-argument sweep produced **no panic, no hang, and no signal death**
on either platform. The front end is robust; that deserves saying as plainly as the defects do.

**D39 — A plugin ceiling may not declare authority the plugin model cannot confer.** Hardening
campaign P7 (`HARDENING_CAMPAIGN.md` C34).

`Grant::to_authority` and `PluginArtifact::ceiling` hard-code `device`, `foreign_c`, and
`foreign_python` to empty — deliberately: a plugin is not a thing that commands a machine or binds a
native library. But a manifest that *declared* one of them had the declaration silently dropped, so the
artifact loaded clean while advertising a ceiling it did not have. Verified by running it: a manifest
declaring a device envelope and `foreign_c: ["libm"]` produced empty scopes, kept all three effects,
and returned `Ok` from step 1.

Not exploitable — the drop is toward *less* authority, and `cap_slice` gives an unlisted effect no host
import at all. RULED anyway, on C23's reasoning and SECURITY.md §3.1: a declaration that is accepted
and then means nothing misleads review without ever failing, and dropping it silently is the one option
that is both safe and dishonest. DL1508 at step 1, naming the dimension; an **empty** list stays legal
because it claims nothing.

**Deliberately not ruled:** a ceiling may still name `Actuate` or `ForeignCall`. `cap_slice` marks
those effects as having no Contained host import "in v0.6" — forward work — so refusing them today
would prejudge it. Their inertness is witnessed instead.

**Verified and unchanged, recorded so it is not re-derived:** one path to a loaded plugin's authority
(`to_authority` → `step3_ceiling`'s `⊑` over all nine dimensions → `step4_holder`, the same value
throughout); the class is never inferred or substituted and Verified never falls back to Contained
(DL1504); an invalid signature refuses **unconditionally**, before `require_signed` is consulted, with
unsigned-but-required a distinct code (DL1511 vs DL1510); a reload mints a fresh node so an
unload/reload authority swap is impossible; and the `.dpx` reader allocates only on bytes actually
present, bounds its ULEB shift, and strictly advances. The one hostile shape its existing bit-flip
sweep cannot reach — a crafted 5-byte ULEB declaring 0xFFFF_FFFF — is now witnessed too.

**D40 — Root authority may not narrow silently across an actor boundary.** Hardening campaign P8
(`HARDENING_CAMPAIGN.md` C35).

`RootMsg` is a hand-written enumeration of the dimensions a `Root` carries to an actor. Diffed against
`RootVal`, exactly one is missing: **`computes`** (phase 10h). Phase 10e's `actuators`/`sensors` cross,
and `ComputeEnvelope` is plain data of the same shape as `ActuatorEnvelope`, so there was no obstacle —
10h simply did not extend the list. An actor holding a Root slice therefore loses compute authority, and
nothing said so.

RULED in two parts, because the two halves are different kinds of question:

(a) **The silence is a defect and is fixed.** The conversion site names the omission, and a gate reads
both struct definitions out of the source and fails if any `RootVal` dimension neither crosses nor
appears in an explicit `WITHHELD_FROM_ACTORS` list — telling the maintainer to *decide*, not to append.
It fails in the other direction too, so a stale "withheld" claim cannot outlive the fact. Rust has no
reflection and the lists live in different files; `delulu-conform` already scans compiler source for
exactly this reason.

(b) **Whether `computes` SHOULD cross is not the kitchen's call.** Carrying it widens what an actor may
do — a capability decision, not a hardening fix. The restrictive reading stands until the owner decides.
Fail-closed is the safe default and this ruling keeps it.

**The pattern this is the third instance of, named so it stops recurring:** a hand-maintained list of
authority dimensions falls behind `Scopes` and nothing notices — C31's fixed `[…; 7]` guard array, C34's
dropped plugin dimensions, and now C35's actor boundary. Every such list needs a gate.

**D41 — The formatter preserves comment paragraphs, and the mermaid graph declares its own scope.**
Hardening campaign P9 (`HARDENING_CAMPAIGN.md` C15, C36).

(a) **C15 — `fmt` merged comment paragraphs, and neither law could see it.** Two paragraphs separated by
a blank line came out as one block. The identity law's comment projection is each comment's
`(text, own_line)` in order; a merge changes none of those. Only the *spacing between* comments was
lost — the part carrying the author's structure. RULED: track the source line each own-line comment ends
on and emit one blank when the next starts more than a line later. Runs of blank lines still collapse to
one (canonical formatting), and a **trailing** comment does not end a paragraph — otherwise the
formatter invents blank lines on top of item separation. The transferable lesson is recorded in the
ledger: **when a projection is chosen to prove a property, ask what the projection cannot see.**

(b) **C36 — `--format mermaid` under-reported the graph without saying so.** 3 nodes and 1 edge for a
graph with 12 and 23: packages and modules only, no functions, effects, capabilities, or call edges.
The scope is correct and *is* documented — one line in an addendum — but a mermaid diagram is pasted
into READMEs and agent context, permanently separated from that documentation, and the conclusion
available to its reader is "this program has no effects". For a graph whose purpose is that authority be
legible, that is the worst wrong reading it could invite. RULED: the artifact describes itself (two `%%`
comments naming the scope, the exclusions, and which format shows the rest) and `--help` says it too.
The HTML renderer already did exactly this — full graph, collapsed above a cap *with a visible notice* —
so the pattern existed in the same file and one renderer had not adopted it.

**Verified and unchanged:** locale invariance proved mechanically (byte-identical `--json` across both
shipped locales while human prose changes); the Atlas's four query verbs answer correctly, `digest` is
byte-stable across runs, and a program with check errors is refused with DL1780 *alongside* its
underlying diagnostic; the LSP survives empty input, non-JSON, an unknown method, a truncated frame
declaring `Content-Length: 99999`, and a hover on a nonexistent file with no panic, hang, or signal.

**D42 — The coverage law checks that a witness EXERCISES its anchor; unsigned and badly-signed are
different codes; and DL0907 describes the class it actually covers.** Hardening campaign P10
(`HARDENING_CAMPAIGN.md` C37, C38, C14). Stage 9 decides whether anyone can trust a build they did not
make, so this pass attacked the CLAIMS.

(a) **C37 — invariant 42 proved existence, not exercise.** A witness pointing at a nonexistent or
`#[ignore]`d test is caught (both verified by breaking them). But repointing DL1710's *rejecting* witness
at a real, active, unrelated test left the gate reporting **100% coverage** while nothing produced that
code. The law proved "a named, non-ignored test exists", not "that test exercises the anchor".

RULED: a **rejecting** witness's body must name the code it witnesses. A static scanner cannot run a test
and observe its diagnostics; naming the code is the strongest property available from that vantage point,
and **108 of 109** rejecting witnesses already satisfied it. Scoped to rejecting witnesses on purpose —
an accepting witness proves a code does *not* fire, and the eighty anchors witnessed by
`accepting_programs_check_clean` would never name one. The single legitimate exception (DL1907, whose
refusal surfaces as a catchable `ComputeErr` value rather than a DL-coded diagnostic) is an explicit
reasoned entry, not a weakened rule — the `WITHHELD_FROM_ACTORS` pattern from D40.

(b) **C38 — the detached path violated a rule this project had already made.** An absent signature and an
invalid one both reported DL1705, "signature verification failed" — untrue for an unsigned artifact, since
nothing was verified. Stage-6 deviation 8 already ruled that badly-signed and unsigned are *different
faults*; its own test asserts the phrase, and the plugin path implements it with DL1510 vs DL1511. Only
the detached path never followed it. RULED: use **DL1511** for unsigned, whose meaning already is "carries
no signature", and generalize its registry entry from plugins to every artifact kind rather than mint a
new number. The distinction is the load-bearing one: unsigned is a policy question, a signature that fails
to verify is an attack indicator.

Everything else on that path already refused correctly: a signature over a different artifact, truncated
to 95 of 96 bytes, a flipped key byte, a flipped signature byte, and `--require-hybrid` against a
classical-only signature (DL1908, its own code). No fail-open.

(c) **C14 — DL0907 described one condition and is raised for a dozen.** Titled "match reached no arm",
and raised for an unbound name, an assignment to one, a field assignment on a non-record, an index
assignment on a non-list, `?` on a non-Result, an unknown function, an unknown test name, an actor turn
with no address, and a foreign value at an actor boundary. A reader who hit it for an unbound name and ran
`delulu explain DL0907` — which the diagnostic invites — was told something false about their own program.
RULED: the code names the CLASS ("an internal invariant the checker should have guaranteed was violated"),
the message names the condition, and the `match` case stays as the canonical example.

**Verified and unchanged: the SBOM is accurate** — 17 direct dependencies declared, 17 listed, zero drift
either way, every version matching what the lockfile resolves for that direct declaration, and its own
note explains why the transitive omission is stated rather than discovered. The D19 fix held. Recorded
with a caveat about method: a first pass nearly mis-reported two versions as drift by comparing against a
name→version map, when `wasm-encoder` and `getrandom` each appear at three versions in the lockfile and
the SBOM correctly names the one bound by the direct declaration. The tool was right; the analysis was
wrong.

**Not ruled, deliberately: `type A = B` is ambiguous in the normative grammar** and the parser
resolves it silently toward a single-variant sum, so `fn g() -> Meters { Int }` checks clean and no
alias to a bare type name can be written at all. Choosing the disambiguation rule changes which
programs are accepted and is a **public-specification decision reserved to the owner** — recorded as
C28 with both coherent options and a recommendation, and documented against actual behaviour in
`STAGE1_SPECIFICATION.md` so the ambiguity is at least resolved on paper.

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
figure above was a fast-build snapshot. See D19. **D20** then finished D19e's deferral: the sim
dead-man ticks on a logical clock (`--sim-step`), so the satellite demo replays byte-identically
across debug and release; the wall-clock dead-man (the real-time guarantee) is unchanged.

**D43 — A simulation that cannot run out of time, and an envelope grammar read two ways.**
Hardening campaign P11 (`HARDENING_CAMPAIGN.md` C39–C45), the last per-stage pass and the one with
physical stakes. Seven sub-rulings; the first two are the ones that moved a machine.

(a) **C39 — the stepped clock did not charge a refused command.** `--sim-step` (D20) advances simulated
time one step per *device interaction*, and the interpreter refuses an out-of-envelope command before the
broker is reached (10e: the command dies, never the process). Two correct decisions composed into this: a
program whose every command was refused **froze simulated time** and held its device forever, while the
identical program and grant on the wall clock lost it to the watchdog — witnessed, `beat overdue by
657 µs` against no revocation at all at 1000× the heartbeat, six times over.

RULED: a refused attempt advances the logical clock and sweeps expiry
(`DeviceBroker::note_refused_attempt`), and if that sweep kills the lease, the lease is the reported
fact — "you no longer hold this device" outranks "your setpoint was out of range", the ordering
`DeviceBroker::command` already documented for the accepted path. **The dead-man itself is untouched**:
`due()` still decides when a lease dies, the wall-clock watchdog is unchanged, and a refused command
still does not BEAT a lease. This is the simulator being made to owe the dead-man the time the wall clock
owes it for free. It matters because **DL1905 refuses hardware without an approved simulation of those
exact bytes** — so the environment that authorizes hardware could not rehearse the revocation hardware
would produce, for exactly the fault class (every setpoint out of range, a units bug being the ordinary
cause) that a dead-man exists to answer.

**A second defect fell to the same ordering change, on the WALL clock, and it is the more broadly
important one.** Consulting the lease before the envelope on the refusal path means a program that has
already lost its device and then sends an out-of-envelope command is told it lost the DEVICE. Before, it
was told its setpoint was out of range — defeating the distinction 10f deliberately built (D11b:
`Envelope` and `LeaseRevoked` are separate variants because "you clamp a bad setpoint and retry, and you
STOP when you no longer hold the machine"). A controller told `Envelope` clamps and retries against a
machine it does not hold. Witnessed on the wall clock with no `--sim-step`: `after: REVOKED` now,
`after: REFUSED` before, with the run summary reporting the revocation either way. This consequence was
not predicted when the fix was designed and the first attempt to demonstrate it failed — six rapid
refusals finish before the watchdog ticks — so the witness settled it, not the reasoning.

Whether a refused command SHOULD prove liveness is a separate question and is **owner-reserved** (C46).
The dead-man's documented remit is "silence, not malice"; a malfunctioning controller is not silent but
is malfunctioning. Both readings are defensible, which is why an implementation detail must not settle
it.

(b) **C40 — a term stated twice was resolved silently, and the two parsers resolved it oppositely.**
`authority.rs` already ruled this shape one level up: `Scopes::device` is keyed by device because "two
envelopes for the same device would be an ambiguity the enforcement path would have to resolve, and
resolving it silently is how a widening gets in." Applied per device, never per term. The broker kept the
LAST occurrence (`BTreeMap::insert`), the runtime the FIRST (`Vec::push` + first-match), so
`angle_deg=-30..95,angle_deg=-1..1` meant `[-1,1]` to the recorded authority and `[-30,95]` to the code
that moves the machine. The dangerous edit is the safe-looking one: **appending a tighter bound recorded
a tightening it did not apply.**

RULED: both parsers refuse a repeated term, dimensions and fixed terms alike. Nothing legitimate states a
bound twice. Multiple `--grant actuator=` flags for one device were checked separately and are correctly
MET (the tighter envelope wins in either order) — the defect was inside one envelope string only.

(c) **C41 — non-finite bounds.** The broker has refused them since D12e and documents the refusal as what
makes `DeviceScope`'s `impl Eq` sound; neither runtime parser checked, and the machine is moved from the
runtime side. `angle_deg=-inf..inf` commanded 12° successfully. `kernel_ms=0..inf` is the sharper case:
that term is mandatory with the stated reason "a kernel with no time budget can occupy the device
forever", and `0..inf` satisfies the requirement while being the unbounded budget it exists to prevent.
RULED: refused in both runtime parsers, same wording as the broker. (`NaN..NaN` was already harmless —
every comparison against NaN is false — but by accident, not by design.)

(d) **C42 — the fail-state vocabulary.** `fail` was free text on the broker side and a closed enum on the
runtime side, so `fail=hodl`, `fail=`, `fail=safe_park` and `fail=Hold` all produced a grant no program
could mint — fail-closed, but discovered when a robot tried to move rather than at delegation. RULED: one
canonical list, `device_scope::FAIL_STATES`, in the LOWER crate (`delulu-runtime` depends on
`delulu-broker`, not the reverse), so there is one list rather than two that can drift.

(e) **C43 — the law that could see none of (b), (c) or (d).** The existing pin read "every envelope the
runtime can parse must render to a string the broker parses back to the SAME authority": one-directional,
over four hand-picked good specs. This is D42(a)'s defect in another subsystem — **a law that proves less
than it claims** — and the same shape, checking agreement only where both sides say yes. RULED: a
bidirectional law over a corpus including the hostile shapes (`runtime_ok == broker_ok`, the corpus's own
expected verdict, and full field agreement where both accept). Each of C40/C41/C42 was observed failing
it in the correct direction.

(f) **C44 — the hint scan's catch-all, gated.** `module_requests_native` drives both the authority
report's `native-emission` line and DL1906, and ends in `_ => false`. Correct today — those three AST
structs are the only ones with an `attrs` field — but a catch-all cannot fail to compile, and
`@ignore`/`@slow` on tests are the obvious future additions. The **fourth** instance of the recurring
pattern (D37/D39/D40): a hand-maintained list of authority-bearing things falling behind
the type that defines them. RULED: a gate reads `ast.rs` for structs declaring `pub attrs:` and fails in
both directions, instructing the maintainer to DECIDE rather than append.

(g) **C45 — an approval that did not carry its own scope.** `deploy plan` compares the environment
profile's EFFECT ceiling and nothing else; `deploy.rs`'s header always said so, but the verdict said
"within `<env>`'s authority ceiling" and `--json` said `"approved": true` with no scope. Nine dimensions
exist; this compares one. Same reasoning as D41's mermaid fix: the artifact travels away from the docs
that qualify it. RULED: `EFFECT ceiling` in the verdict, plus `compared`/`not_compared` on BOTH surfaces
— an agent reading `--json` gets what a human is told, per the no-discrimination rule. The verdict stays
the last human line, because `deploy_plan_adversarial.rs` contracts that and CI logs rely on it.

**Verified-and-held in Stage 10 (do not re-derive):** DL1909 cannot be bypassed by an empty,
unparseable, mis-sectioned (`[authorities]`), mis-keyed (`effect =`) or wrong-typed (`effects = "Clock"`)
profile — every one yields a ceiling of no effects at all and refuses every service; an unreadable
profile is a plain exit-2 error, never an approval. The `@jit` leash holds: DL1906 warns, the hint is
ignored, `native-emission` appears in both the human report and `--json`, and a lease can never confer it
(`exec_native: false` is hard-coded on the lease path). Certificate authority parsing refuses an unknown
authority key, effect name or scope dimension by refusing the certificate WHOLE, with the reason stated
in the code: "an authority dimension a verifier cannot see is one it cannot enforce". A long forged chain
is not a DoS — verification is sequential and dies at the first unanchored or unsigned hop. Single
adoption is keyed per CERTIFICATE, so a subordinate broker may adopt two roots and each is separately
bounded, revocable and audited; whether it SHOULD is federation policy, not a defect.

**D44 — What size exposed: a grammar that could not describe its own formatter, a quadratic field
lookup, a crash the crash-gate could not see, and a review surface that never read the manifest.**
Hardening campaign P12 (`HARDENING_CAMPAIGN.md` C47–C51). Four sub-rulings closed, two questions
raised.

(a) **C47 — the normative grammar disagreed with the parser in BOTH directions.** Every
comma-separated bracketed list requires a trailing comma when it spans lines (`match` arms excepted),
and §3.0 said the opposite everywhere: `[ "," ]` where the parser demands one, and no trailing comma
listed at all for `params`, call args, record literals and `list_lit` — which the parser accepts and
which **`delulu fmt` emits**. An independent implementation written from §3 alone would have rejected
every formatted file containing a wide list.

RULED: the SPECIFICATION was wrong, and is corrected. §3.0 states the newline rule normatively — it
cannot be derived from an EBNF with no `NEWLINE` terminal — and the four productions carry the
`[ "," ]` the parser has always accepted. Whether the parser *should* require the comma on multi-line
lists is a language-surface question (it is stricter than most languages, and the diagnostic does not
teach the fix) and is **owner-reserved** as C47b.

(b) **C48 — record field lookup was quadratic, behind a `.clone()`.** `field_type` cloned the whole
type definition on every field access, so a function reading N fields of an N-field record deep-copied
N² field entries: 632 ms to check one 2000-field record, against 173 ms for a 40,046-line file five
times its size. Isolation separated the conflated variables — declaration alone flat, accesses alone
flat, an N-term `+` chain flat, only the product exploding. The clone existed solely to release the
borrow on `self.table` before `lower_type` takes `&mut self`.

RULED: clone the one field's type expression and the generics, never the definition. **632 ms → ~42 ms
at N=2000 (15×)**, 4× input now costing ~2.9× instead of 11×. The residual `find()` scan keeps the
cost O(fields × accesses) with a small constant; it is **published with its measured curve** rather
than implied away, and a name→index map is named as the next step.

(c) **C49 — an empty `delulu.toml` panicked, and the no-panic gate was blind to it.** Five of twelve
manifest shapes crashed `build`/`check` while `lock` and `authority` diagnosed all of them (C23/D30's
"two paths out of four" again). **The crash came from a fix made earlier in this same campaign**:
C26/D33's "no `.delulu` modules found under `<dir>`" note reaches for the root package's directory,
and an unreadable manifest fails resolution before a root package exists. DL1004 was computed
correctly every time — the tool crashed while being helpful about something else.

RULED: `Workspace::root_pkg()` returns `Option`, so the crash cannot be reintroduced without the
compiler forcing the empty case to be considered. And the gate is fixed, which matters more: `main.rs`
runs the CLI on a 512 MiB-stack worker thread (so `MAX_DEPTH` fires before the native stack does) and
maps a worker panic to exit **2** — deliberate, documented, and correct, since 2 is "internal". But
`json_contract.rs` keyed its no-panic sweep on `code == 101`, so it could not see any crash in the
path where all the work happens. Both sweeps now detect the panic message itself. **A repair needs its
own skip-branch analysis; "what if there is nothing to name?" is one.**

(d) **C50 — the review surface never opened the manifest.** `delulu authority <dir>` printed a
confident report, `diagnostics: []`, `summary: {errors: 0}` and exit 0 for a package `check` refuses
with DL1004, and the report was byte-identical to one for a well-formed manifest. `summary.errors: 0`
was a false claim in a machine-readable field, on both surfaces, on the one command whose product is
"what this program can do to your system".

RULED: a PRESENT manifest is parsed and its diagnostics are unioned in. ABSENT stays legal —
`authority` accepts a plain directory of modules (C26/D33) — and present-but-unreadable is refused,
the same shape as Stage-6 deviation 8's present-but-invalid signature. Noted and NOT ruled: `authority`
does not evaluate the manifest CEILING either, so a package violating its own declared authority still
reports clean there (DL1009 is `check`'s). That is defensible division of labour, but it is the review
surface, and it is flagged for an owner rather than decided.

(e) **C51 — `authority` cannot review a package that has dependencies.** OPEN. It uses the
single-package loader while `build`, `check`, `lock` and `authority --diff` resolve the graph, so
every monorepo member fails with DL0303 while `build` on the same package succeeds. The fix is
identified (resolve the workspace on this path too) and deliberately **not applied here**: it changes
what the authority report CONTAINS for a whole class of packages, and that report is a published
contract surface. Carried to P13.

**Verified-and-held at scale (do not re-derive).** On a 40,046-line / 478 KB file, release, warm:
`check` 173 ms; `atlas` 290–426 ms in every format with `digest` byte-stable across runs; `fmt` on
30,009 lines 1,292 ms with its output still checking clean. Peak memory never exceeded 52 MB anywhere
in the corpus. **D38's diagnostic cap holds exactly as designed**: 5,000 real errors render in 155 ms /
14 KB with an honest note naming the 4,950 withheld, while `--json` stays uncapped at 5,000
diagnostics in one object with a truthful `summary`. Monorepos hold: 50-deep dependency chains and
200-package diamonds check, build and lock cleanly, and **lockfiles are byte-identical across repeated
writes** at every size — the semver-authority law's determinism survives depth.

**D45 — The review surface can review a real package, and a lockfile can no longer lie about one.**
Hardening campaign P13 (`HARDENING_CAMPAIGN.md` C51, C52). Two sub-rulings, both on the supply-chain
surface, plus a large negative result.

(a) **C51 — `authority` ran the wrong loader.** `delulu authority <dir>` used the single-package
loader while `build`, `check`, `lock` and `authority --diff` all resolve the dependency graph, so
every package with a dependency was refused with DL0303 while `build` on the same directory
succeeded. The one command whose product is "what can this do to my system" could not answer for a
monorepo member — and the supply-chain question is exactly that case.

RULED: **the presence of a `delulu.toml` selects the loader.** With a manifest, resolve the whole
graph as the siblings do; without one, keep the single-package loader. That branch is not
decoration — `resolve_workspace` requires a manifest and reports DL1004 without one, so routing
everything through it would have refused a plain directory of modules, which C26/D33 made legal and
C50 deliberately preserved. The obvious fix would have traded C51 for that regression.

Verified before shipping, as the phase required: a no-dependency package's report is **byte-identical**
before and after on both surfaces, for a simple package and for one exercising multiple modules,
secrets and `Net`. One improvement rides along: a library package with no `fn main` reported
`Authority of \`package\`` — a placeholder — and now uses the root package's name, which is available
once the graph is resolved.

(b) **C52 — `build --locked` verified a lockfile's hashes but not its claims.** `verify_locked`
recomputed `content_hash` and `authority_hash` from reality and compared them to the stored hashes,
and never looked at `effects`, `cap_kinds`, `secrets` or the scope lists — the fields a human opens a
lockfile to read. A lockfile could claim a dependency has no effects and no capabilities while that
dependency genuinely performs `Net`, and the locked build printed "built clean". `authority --diff` on
the very same file reported `+ effects Net` and `verdict: WIDENING`: **the interactive review command
caught what the automated CI gate did not.**

RULED, four checks, each restating a rule this project had already made:
- The recorded authority fields are compared against the computed authority (DL1002). Written as a
  **destructuring** `let LockEntry { … }` so a field added to the type cannot compile until someone
  decides whether it belongs — the C31/C34/C35/C44 pattern answered structurally instead of with a
  fifth hand-maintained list. `accepted_by` is excluded **by name and with a reason**: it is an
  operator's recorded decision, not re-derivable from source.
- The recorded `version` is compared against the package's own (DL1002). `--locked` means "refuse any
  resolution not already pinned", and a stale version was never pinned.
- A **duplicated** entry is refused, not resolved (DL1011) — the C40 rule, third application.
- An **unreadable lock format version** pins nothing (DL1011) rather than being interpreted as
  version 1. Same rule as an unverifiable signature algorithm (DL1908), and DL1011 is the honest code
  because "nothing is pinned" is precisely what it already names — `Lockfile::parse` documents the
  same posture for a garbled file. **No new diagnostic code was needed.**

A fifteen-case semantic attack matrix went from 15 accepted to 2, with the untampered control
building throughout. **Framed precisely: this is a review-integrity defect, not an authority
escalation** — the manifest pin bounds a dependency independently of the lockfile, and the
hash-protected tampering was already caught on every shape tried.

Two residuals, named: a lock entry for a package not in the resolved graph is still accepted (it is
never examined and refusing it could break a legitimate superset lockfile), and `accepted_by` can be
edited freely — **verified to confer nothing**, since it is written by the `--accept-authority` flow
and never read to make a decision.

**Negative result worth recording so it is not re-run blind.** 561 fuzz invocations across four
parsers — `.delulu` source, `delulu.toml`, `delulu.lock`, morph TOML — under truncation at twelve
offsets, byte flips, deletions, inflations, injections (NUL, BOM, `1e400`, 200-deep bracket runs,
oversized integers) and self-duplication, swept through `check`/`fmt`/`atlas`/`build`/`lock`/
`authority`/`morph`: **no panics, no hangs.** Crashes were detected by the panic MESSAGE, not by exit
code — without D44c's lesson this sweep would have been as blind as the one it replaced.

⚠ **Method warning.** The first run of the lockfile attack used plain `delulu build` and reported all
fifteen tamperings accepted — a false catastrophe. `build` does not consult the lockfile; `--locked`
does. Before reporting a surface as unprotected, confirm the command under test is the one making the
guarantee.

**D46 — The four owner-reserved questions, decided.** Jesse gave explicit authority to settle them
("If there is a problem fix them chef, no need to wait for my approval or permission", 2026-07-26),
naming C28, C35, C46 and C47b. They had been carried open across several phases precisely because
each changes something reserved to the owner — the public grammar, what an actor may hold, a
safety policy, and the language surface. Each is decided below with its reasoning, because a ruling
whose argument is not written down is just a preference.

(a) **C28 — `type A = B` is an ALIAS.** The right-hand side was read as a single-variant sum whenever
it was a bare identifier, and every consequence was silent: `type Meters = Int` declared a constructor
named `Int`, so `fn g() -> Meters { Int }` type-checked; a mistyped value reported `expected 'T9'`;
and **no alias to a bare type name could be written at all**, since `type Meters = (Int)` —
parenthesised — was the only spelling reaching the alias production.

RULED: a variant list is signalled **syntactically and only** by `(` or `|` after the identifier.
`type E = A | B` and `type P = Data(Int)` are sums; `type Meters = Int` is an alias; a single
field-less variant is `type U = Nothing()`. The rule does not consult name resolution, so the grammar
stays context-free — what makes a right-hand side a sum is a token, not whether some identifier
happens to name an existing type. Every language with this syntax means "alias", which is what a
reader means by it. **A search of the tree found zero single-variant field-less sums**, so nothing in
this repository changes meaning.

(b) **C35 — `computes` crosses an actor boundary.** `RootMsg` is a hand-enumerated copy of `RootVal`'s
dimensions and phase 10h did not extend it, so a `Root` slice silently lost its compute grants at the
boundary. D40 gated the silence; whether the dimension SHOULD cross was left open.

RULED: it crosses. The argument that settles it is the asymmetry — **`actuators` and `sensors` already
cross, and actuation moves physical machines.** Refusing the strictly less consequential dimension
while permitting the more consequential one was an omission, not a safety position; the comment that
justified withholding even read "fail closed, like the actuator list", on the line above where the
actuator list crosses. `ComputeEnvelope` is plain data and `Send` by construction, the envelope BOUNDS
its holder rather than empowering them, and the broker still re-checks every dispatch against the
grant. `WITHHELD_FROM_ACTORS` is now **empty** — every `RootVal` dimension crosses — and the drift
gate stays, because "nothing is withheld" is a claim that has to keep being true.

(c) **C46 — a refused command does not prove liveness.** D43a made the stepped and wall clocks agree
about a refused command costing time; whether a controller whose every setpoint is out of range should
KEEP its machine was left to the owner.

RULED toward the stricter reading, which is also what the code already did. A controller emitting only
refused commands is not silent, but it **is** malfunctioning, and taking a machine away from a
malfunctioning controller is what a dead-man exists for. The alternative would let a units bug hold an
actuator indefinitely while never moving it correctly. No behaviour changes; what changes is that this
is now a decision with a reason rather than an accident of implementation, and §5.2 says so.

(d) **C47b — a multi-line bracketed list needs no trailing comma.** It was required, which is stricter
than most languages, and the diagnostic a person met (`expected }`, caret after the last element,
while `}` sat on the next line) did not teach the fix.

RULED: all four spellings are accepted — one line or many, trailing comma or not — in every bracketed
list. **This was never a design decision.** §2.2 inserts a `Term` at a newline only when the previous
token can end a statement, and a comma cannot; so `a,\n)` always parsed and `a\n)` did not, purely
because that single terminator was never skipped before the closing bracket. The fix is one skip in
each of the nine list loops (`skip_terms_before_closer`), which is why the change is nine lines rather
than a grammar redesign. `match` arms already accepted both forms, so this also removes an
inconsistency between one list and every other.

Witnesses for (a), (b) and (d) were observed failing against the pre-fix code with their exact
payloads; (c) changes no behaviour and is carried by the existing dead-man tests. Full suite green on
both platforms at the phase's baselines.

**D47 — A type alias is validated where it is written, and a cyclic one can no longer crash the
compiler.** Hardening campaign P14 (`HARDENING_CAMPAIGN.md` C53, C54; reshaping C16). Two sub-rulings
and one published limit.

(a) **C54 — a used cyclic alias aborted the compiler with a stack overflow.** `lower_type` expands an
alias by recursing into its target, so a cycle is unbounded recursion. `type A = A` plus a single use
of `A` died with `has overflowed its stack`, exit `0xC00000FD` — and so did `type A = B; type B = A`,
`type A = List[A]`, `type A = iso A` and `type A = fn(A) -> Int`. A hard crash from three lines of
ordinary source, and for anything that compiles code it did not write — an editor, a CI runner, a
package registry — a denial of service.

**This reshapes C16, and the correction is worth stating plainly.** P2 recorded cyclic aliases as
*hygiene*, on the evidence that 5000-deep terminating chains resolve and that a secret cannot launder
through a cycle. Both of those findings hold. What that pass never tested was a cycle that is actually
USED — and the declaration alone is harmless precisely because nothing lowers it. The verdict was
right about what it measured and wrong about the class.

**Note what the crash was invisible to.** A stack overflow aborts the process without printing
`panicked at`, so the no-panic sweeps — which match that message, exactly as D44c made them — could not
see it. That is the third time a gate has been blind to the failure it exists to catch (D42a's coverage
law, D44c's exit-code sweep, this). The lesson is not about any one gate: **ask what signal a gate keys
on, and what failure produces a different signal.**

RULED: `Checker::check_type_aliases` runs before anything lowers a type, detects cycles in the alias
graph, and reports **DL0304** at each participating declaration, naming the chain (`A = B = C = A`) so
a multi-step cycle is followable. `lower_type`'s alias arm consults the resulting set and refuses to
expand a cyclic alias, which makes the crash structurally impossible rather than merely diagnosed.
Only alias→alias edges are considered, and that is what keeps the graph small: a reference to a record
or a sum terminates, because those are nominal and are never expanded — which is also why a recursive
`type Node { next: Option[Node] }` and a recursive `type Tree = Leaf | Branch(Tree)` remain legal, and
are tested as such.

**DL0304 was generalized rather than a new code minted.** Its title becomes "a cycle in the declaration
graph (imports, or type aliases)" and its explain body covers both, because the reason is identical in
both: resolution has to terminate. Same discipline as DL1511 in D42b — reuse the code whose meaning
matches, keep the subsystem range meaningful (DL03xx is resolution), and do not strand agents keying on
numbers.

(b) **C53 — an unused alias target was never resolved.** `type Meters = Metres` — a typo — checked
clean, with DL0301 arriving only at a use site; in a library whose own code never uses the alias, that
diagnostic landed on a consumer who did not make the mistake. The C11/C23 family: a declaration
accepted and then silently inert.

RULED: the same pass lowers each alias target, so an unresolvable name is reported at the declaration.
It reuses the real resolver rather than duplicating its notion of which names exist — the alternative
would have been a second list of builtin type names, which is the drift shape this campaign has closed
four times. Forward references keep working because the pass runs after the whole module's type names
are registered, and that is tested alongside a 200-deep terminating chain.

(c) **Published limit, not a defect: runtime record field access is O(record width) per read.** The
interpreter was checked for C48's clone-per-access shape and does **not** have it — `Interp::field`
clones only the value it finds. It does scan linearly. Measured with total field reads held constant at
~200,000 and only the width varying: **165 → 300 → 1,570 → 5,071 µs per 1k reads** at 50 → 200 → 800 →
3,200 fields, i.e. linear in width.

RULED: publish the number, do not change the representation. For the widths real programs use — five to
twenty fields — a linear scan over a short `Vec` is the *faster* representation: no hashing, no
indirection, the record in cache. Removing the cost means either a per-instance map (paying memory on
every record, including the narrow ones that dominate) or resolving field indices statically through
the DIR side tables; the second is the right fix and is a real change, not a tidy-up. It is linear, not
quadratic, and the constant is small — the opposite of C48, which was accidentally quadratic *and*
allocated on every access. Recorded in `measurements/scale/RECORD.md` under the Constitution's own rule
that where DeluluLang loses, the table says so.

**D48 — The Authority and Guard cross-stage capstone.** Hardening campaign P15, the phase the
commission ranked highest. The full discharge is `docs/design/AUTHORITY_GUARD_CAPSTONE.md`; this ruling
records what changed in code and what was decided.

(a) **The authority SERIALIZATION seam had no gate, and now does** (closed in P15).
`Authority::to_json` (write) and `authority_from_json` (read) are two hand-enumerated lists of the same
eight dimensions on opposite sides of a certificate, an audit record and every `--json` report. The read
side already refuses an unrecognized dimension by rejecting the whole certificate — RFC §4.9.3's
load-bearing skip branch, tested. The **write** side had nothing: a ninth dimension added to `Scopes`
would simply not be emitted.

RULED: a round-trip gate that populates every dimension, asserts write→read is the identity, and
**destructures `Scopes`** so a new field makes the test fail to COMPILE until someone decides how it
serializes. Verified non-vacuous by deleting `foreign.python` from the writer and watching the gate name
exactly that dimension. This is the **sixth** instance of the recurring hand-maintained-list pattern
(C31/C34/C35/C44/C52 preceding it) and it is answered the same structural way.

Why it mattered although omission is fail-closed for the grant: the authority embedded in every
hash-chained AUDIT record would have under-reported what a holder actually held, and `render_compact`
feeds the same list into DL0802's repair text. C29/C30's class — not an escalation, a loss of the
accountability the system sells.

(b) **P6's open question — "revocation racing an in-flight operation" — is discharged, and the answer
is that there is no data race to find.** The broker daemon is single-threaded and serializes at
*request* granularity: one `accept`, one frame, one `handle`, one response. A `check` and a `revoke`
cannot interleave inside the broker. What remains is the **logical** window the project already
documents — revocation is effective before the next USE, never retroactively against an operation
already authorized — bounded by per-use re-checking for synchronous ops and by `--epoch-ms` for
epoch-class ops, and measured at 12.7 ms p50 / **39.7 ms worst** for operator-to-stopped. Mid-run
revocation is tested end to end WITH a control (`estop_cli.rs`). Nothing to fix; the debt is paid by
demonstrating the bound rather than by discovering a defect.

(c) **The 14 Guard surfaces are discharged individually, and three of them do not exist in v1.x.**
Saying so is the honest discharge — a surface that cannot be reached needs a reason, not a checkmark.
The **optimizer** is described in spec §2.1 and not implemented. The **native backend** does not exist
and is leashed (DL1906; and `exec_native: false` is hard-coded on the lease path, so the leash holds
across the federation boundary). **Distributed execution** is not a separate surface — the distributed
piece is broker federation plus a single-host actor runtime. The other eleven are guarded, each with
cited evidence, and two carry named gaps rather than clean passes: the adapter has **no signature
check** (D23), and **hardware has never been exercised** — no driver ships in-tree and no physical
device has ever been commanded.

(d) **Two failure shapes are promoted from incidents to design rules**, because each recurred often
enough that treating them as one-offs would be the actual defect:

1. *A hand-maintained list of authority-bearing things falls behind the type that defines it, and
   nothing notices.* Six instances (C31, C34, C35, C44, C52, and (a) above). The answer is never
   "remember to update the list" — it is a compiler-enforced pattern or a source-scanning gate. Where a
   dependency edge permits, the stronger answer is one list referenced by both sides
   (`device_scope::FAIL_STATES`, D43d).
2. *A gate is blind to the failure it exists to catch.* Three instances plus a near-miss: a coverage law
   that proved a witness existed rather than that it exercised its anchor (D42a); a no-panic sweep keyed
   on exit 101 while a worker-thread panic maps to exit 2 (D44c); a sweep matching `panicked at` against
   a stack overflow, which prints no such text (D47a); and a cross-parser law that checked agreement
   only where both sides said yes (D43e). **The rule: ask what SIGNAL a gate keys on, then ask what
   failure produces a different signal.**

**D49 — A diagnostic trace is bounded; an assertion trace is not.** Hardening campaign P16
(`HARDENING_CAMPAIGN.md` C56).

`TraceSink` held every record until the process exited, so memory grew with the number of effects
PERFORMED rather than with the program's live data: 100,000 console writes took peak working set from
6.7 MB to 70.1 MB — about 633 bytes retained per effect, with no ceiling. At a thousand effects a
second, an ordinary rate for the control loops Stage 10 exists to serve, that is roughly 2.3 GB per
hour. `--trace-effects` is opt-in and off by default, which is why this is a finding rather than an
emergency; what makes it a finding at all is *which* runs turn it on — a long-lived controller being
diagnosed in the field is precisely where hours of uptime meet a flag that never frees.

RULED: the sink takes its retention policy from the caller, and the two callers get different ones.

- `--trace-effects` alone gets `TraceSink::bounded(200_000)`: records are kept from the FRONT, the
  number withheld is counted, and the run prints a note naming it and pointing at the uncapped path.
- **`--assert-trace` is never capped.** It consumes the same records to prove that no effect outside
  the declared set occurred, so a dropped record could hide a violation — a fail-OPEN on a
  security-adjacent check, which is strictly worse than the memory it would save. The asymmetry is the
  ruling, not an implementation detail, and the test that pins it says so in its name.

Kept from the front rather than as a ring buffer, deliberately: a deterministic prefix plus an honest
count is reproducible evidence, where a ring buffer would let two runs of the same program disagree
about what happened. This is the same contract D38 gave the diagnostic flood, applied to the same
class of problem.

Measured after the fix: 500,000 effects bound the buffer at **134 MB** (from ~316 MB unbounded) and the
run reports `300000 further effect record(s) not traced`.

**D50 — Cross-platform re-verification and the campaign's close-out.** Hardening campaign P16.

The 2026-07-21 figures in `CROSS_PLATFORM_VERIFICATION.md` predated sixteen phases that changed the
parser, the checker, the broker, the lockfile verifier and the runtime, so they were re-taken from the
committed tree rather than assumed to have survived: **Windows 95 suites / 1289 passed / 0 failed /
clippy 65; Linux 95 / 1293 / 0 / clippy 66; coverage 100%; reference in sync.** Clippy is unchanged on
both platforms across roughly 4,000 added lines.

P1's front-door promise was re-tested the only way that means anything — a fresh `git clone` into an
empty directory, then the README's own commands verbatim. Build, `check`, `authority`, `run`, and the
DL0703 refusal when `--grant console` is omitted: all as documented.

RULED, and it is a statement rather than a change: **macOS has never been executed.** Not once, in any
phase. Every macOS cell reads "never run" rather than "untested" or "pending", because those words
invite a reader to assume someone tried. Nobody tried, there is no hardware, and the CI matrix that
names `macos-latest` has never executed because the repository is never pushed. A declared matrix is
not evidence, and nothing in this repository may describe DeluluLang as supported on three platforms.

The close-out honesty scrub: "quantum-proof"/"quantum-safe" appear 12 times and **every one is inside a
sentence prohibiting the term**; all nine `measurements/` records now state when they were taken (four
did not, including the one this campaign wrote, which had itself argued that a table should say when it
was taken); and the performance clause is intact, with two new losses published under it (C55, C56).

**D51 — The interpreter's depth bound is a contract with the host, not an undocumented requirement.**
Closes `HARDENING_CAMPAIGN.md` C21, open since P4.

The interpreter is a tree-walker: one DeluluLang call costs several native frames. `MAX_DEPTH = 10_000`
is the bound that raises DL0905 — but only if the native stack outlasts it. `delulu`'s `main.rs`
reserves 512 MiB for exactly that reason, so on the CLI deep recursion is a diagnostic. An **embedder**
gets no such thread: on Rust's ~2 MiB default the bound is never reached and the process dies of
`STATUS_STACK_OVERFLOW` instead — the host-crash class D15 fixed for the CLI, resurfacing for anyone
using `delulu-runtime` as a library.

RULED: the bound becomes part of the API. `DEFAULT_MAX_DEPTH` and `STACK_BYTES_PER_DEPTH` are public,
and `Interp::with_max_depth(n)` lets an embedder choose a bound their stack can actually hold. The
default is unchanged, so every existing entry point behaves exactly as before. The witness runs on a
deliberately small thread and proves the guard fires there rather than the stack giving way.

**The per-frame figure was measured, and the number C21 recorded is badly misleading.** Bracketed by
moving the bound on an 8 MiB thread until it broke:

| thread stack | bound | bytes/depth | result |
|---|---|---|---|
| 8 MiB | 500 | 16 KiB | `STATUS_STACK_OVERFLOW` |
| 8 MiB | 200 | 40 KiB | `STATUS_STACK_OVERFLOW` |
| 8 MiB | 100 | **80 KiB** | **DL0905, clean** |

Those are **debug** figures — which is what an embedder's own tests run, and therefore the case that
must not crash. Release is far cheaper: `main.rs`'s 512 MiB works out to about 52 KiB per unit and the
release CLI reports DL0905 cleanly at 100,000 calls, so release fits inside the published budget
automatically.

C21 recorded "10,000 frames need **more than 16 MiB**". That is true, and it reads as though 16 MiB
were nearly enough; the real debug cost is roughly an order of magnitude higher. **A first draft of
`STACK_BYTES_PER_DEPTH` took the recorded figure literally, derived 2 KiB per unit, and would have
advised an embedder into precisely the crash this contract exists to prevent.** The published constant
is now the measured 80 KiB, and a test asserts the published budget covers what `main.rs` actually
reserves — so the advice and the CLI cannot drift apart.

**D52 — A hardware driver's provenance is checked before it is spawned.** Closes the gap D23 named
("an operator-supplied SUBPROCESS with NO signature check").

The envelope bounds what a driver may be *asked* to do — enforced host-side before one byte reaches
it, proved from the driver's own log — and says nothing about where the driver came from. That gap was
recorded honestly and left open. It is now closed in the sense that is available: provenance, not
behaviour.

RULED, four branches, and the asymmetry is the ruling:

1. **A signature that is present and does not verify refuses the run, regardless of policy** (DL1510).
   This is Stage-6 deviation 8's rule applied to drivers, and it is the branch a "not required, so
   don't check" reading skips. A signature that fails to verify means these are not the bytes someone
   signed — tampering, wrong key, or truncation, and the run does not get to guess which.
2. **Absent is a policy question**, because requiring a signature everywhere would refuse every driver
   an operator builds locally. Allowed by default, disclosed loudly.
3. `--require-signed-adapter` turns absent into a refusal (DL1511).
4. **When the first token of `--adapter-cmd` is not a readable file, the run says it could not check.**
   An interpreter-hosted driver (`powershell -File drive.ps1`) names the *interpreter*, so verifying
   the first token would vouch for the wrong bytes entirely — and passing silently there would be
   worse than not checking at all, because it would look checked. Under the flag this refuses.

Reuses the existing detached-signature machinery and the existing codes; **no new diagnostic number
was minted**. Cryptography remains adopted, never hand-rolled.

**What this deliberately does not claim.** Spec §5.4 describes Verified-class signed plugins loaded
into the host; this is still an operator-supplied subprocess. Signing buys provenance — "an operator
with this key vouched for these bytes" — and does not bound what the driver does once running. The
envelope is what bounds that, and it is unchanged. The gap narrowed; it did not disappear, and
`AUTHORITY_GUARD_CAPSTONE.md` §2 row 8 says so.

**D53 — "Signed" is not "signed by anyone you trust", and D52 could not tell the difference.**
Extends D52 after Jesse asked, of the four-branch gate, whether it was right for a language meant
to last. It was not, and the reason was not one of the four branches — it was the question they all
answered.

`verify_detached` reads the public key **out of the first 32 bytes of the signature file it is
checking**. The `.sig` sits beside the driver. So an attacker who can overwrite `drive.exe` can
overwrite `drive.exe.sig` with one they signed a second ago, and every branch of D52 says yes: the
signature is present, it verifies, the gate prints `signature verifies (signer …)` and spawns the
driver. **Against that attacker `--require-signed-adapter` bought nothing.** It stopped accidents
and unsigned drivers, which is worth having and is not what a reader of the flag's name assumes.

The witness plays the attack out before fixing it
(`a_driver_resigned_by_an_attackers_key_verifies_and_is_still_refused_when_the_key_is_pinned`), so
the hole is on the record as observed behaviour and not only as a paragraph.

RULED:

1. **`--adapter-signer <hex>` pins the key.** A signature that verifies under any other key is
   **DL1510** — the "wrong-key" case that code's own text has always claimed to cover, and did not,
   because nothing ever compared the signer to anything. Pinning implies the signature is required:
   an operator who names the acceptable signer has not said "unsigned is fine".
2. **`--adapter-artifact <path>` names which bytes carry the provenance.** D52's branch 4 was right
   to refuse rather than verify an interpreter, but it left `--require-signed-adapter` **unusable
   for every script-hosted driver** — a control nobody can switch on is not a control. Separating
   "which command runs" from "which bytes were signed" makes the flag usable and keeps branch 4's
   honesty for the case where nothing was named.
3. **An unpinned verify must say what it does not mean.** The success line now states that it proves
   these bytes were signed by that key, not that the key is trusted, and prints the
   `--adapter-signer` invocation that would pin it. A bare "signature verifies" reads as an
   assurance; on a signature that travels beside the file, it is not one.

No new diagnostic number minted; no cryptography hand-rolled. Spec §10's "signatures authenticate
origin, not behavior — no trust policy" is unchanged as a statement about the *default*, and there
is now a way for an operator to state one for the case that moves machinery.

**What is still open, and named rather than implied closed: C60 — the verdict is printed, not
recorded.** A run leaves no durable, queryable evidence of which key signed the driver that drove the
machine. Both existing homes were examined and neither fits: the broker's audit chain is a no-op
without a sink (so it would record nothing in the default configuration, which is the configuration
that matters), and the DL1905 sign-off record is written by a **simulation**, before any adapter has
been chosen. That is a real gap in accountability and it wants an artifact this ruling is not
inventing at the tail of a pass. It is also the sharper form of the general point: the sign-off gate
binds the program's bytes and says nothing about the driver's.

**D54 — A literal that is not the value you wrote is refused, in both numeric columns.** Closes C17,
open since P2, and the framing that closed it is the one the finding did not have: this was never
about floats, it was about an **asymmetry**. The lexer has always refused an integer literal too
large for `Int` — "there is no automatic promotion, because a silent widening is a silent change of
meaning" — and accepted `1.0e400`, which becomes `inf`. Same defect, opposite answers, in two arms
of one function.

`f64::from_str` does not fail on a magnitude it cannot hold: it **saturates** to infinity and
**flushes** to zero. RULED: both are **DL0104**, on the existing code.

The underflow half was not in C17 and is the worse of the two. `1.0e-400` becomes `0.0` — and where
`inf` announces itself downstream, a silently-zeroed gain makes a control law quietly do nothing
while every value on the way looks ordinary. A literal the author *did* write as zero is still zero
(`0.0e-400` is accepted), and a **subnormal is accepted**: it loses precision but keeps its
magnitude, which is the property the rule is about.

Infinity remains reachable by computing it (`1.0 / 0.0`). What cannot be done is spelling it as a
finite number.

**The rule has one home, because it has two callers.** Writing the tier-4 corpus found that
DeluluLang had `parse_int` and no way at all to read a `Float` out of text — so a program could not
read a temperature from a file. `parse_float` is added, and it asks **the same function the lexer
asks** (`delulu_syntax::num::float_from_text`). Had it been written separately the obvious
implementation would have been `s.trim().parse::<f64>().ok()`, which answers `Some(inf)` for
`1.0e400` and `Some(inf)` for the *word* `inf` — a data file could then put infinity into a program
whose source is forbidden to write it, and the language would have had two float rules wearing one
name. That is C40's shape exactly (two parsers resolving one question in opposite directions), and
it is why the rule is a shared function rather than a copied five lines.

`parse_float` is additive: no existing program changes meaning, and the WASM backend refuses it with
the DL1201 it already gives `parse_int`, so no engine-parity debt is created.

**D55 — The capability corpus is evidence, and tier 4 was an empty directory.** Closes C7.

`tests/corpus/` held **seven** programs (the finding said eight; it counted `tier4-multimodule/`'s
`NOTE.md`, which is a note, and the correction is recorded rather than quietly applied), and the
tier for multi-module programs contained no program. The note deferred it to Stage 2 — "packages
land with `STAGE2_SPECIFICATION.md`" — and Stage 2 shipped, and nobody came back.

RULED: tier 4 is **four packages, seven modules, dependency depth three, with a diamond**, exercising
what only appears above single-file size — authority declared per package and joined across the
graph, a ceiling stated by the consumer rather than claimed by the dependency, a type that crosses
every boundary while carrying none, and both senses of "module". The conformance harness learned the
difference: a directory holding a `delulu.toml` is **built** through the loader `delulu build` uses,
and its files are no longer fed to `check_source` one at a time (which would have reported failures
that say nothing about the program). Three corpus programs are additionally **run**, with their
output asserted — `corpus_cli.rs` — because checking clean and working are different claims.

**Writing the corpus is what made it evidence, and it immediately found four things.** This is the
argument for a corpus in the first place, so they are listed rather than folded away:

- **No `parse_float`** — see D54. A language with a `Float` type could not read one from input.
- **C58** — a `pub fn` whose signature names a type the package does not **re-export** builds clean
  on its own and fails when consumed, with the error reported *inside the dependency's own source*
  (`dep:reading/src/reading.delulu`) saying `Sample` is "not a type" — in a file where `Sample` is
  perfectly in scope. The rule is right (`pub import` is explicit re-export, like Rust's `pub use`);
  the diagnostic blames the wrong line, in the wrong package, for the wrong reason. Same family as
  C53: a declaration accepted, inert, and paid for by someone else.
- **C59** — **a multi-package program cannot be run.** `delulu run` takes one `.delulu` file or a
  `.dwx`; `build` on a package emits `interface.json` and nothing executable. `kind = "bin"` is
  declarable and unexecutable, and this is true of `examples/greeter/` too — a two-module example
  that ships in this tree and can only be checked. The corpus tier says so in its own README instead
  of implying otherwise, and `corpus_cli.rs` says why its tier-4 assertions are compile-time ones.
  This is the largest capability gap the campaign has found and it is a feature, not a fix.
- **C61** — `let _ = expr` is refused (DL0201) although `_` is a valid **match** pattern. Discarding
  is still possible under any other name, so the restriction prevents nothing and costs the reader a
  worse one.

**D56 — C55's published reasoning was an extrapolation, and is now a measurement.** The named limit
claimed that "for the widths real programs use — five to twenty fields — a linear scan over a short
`Vec` is the faster representation". The table under it started at **fifty**. The claim about the
range that matters was reasoning past the smallest measured point, which is the shape this campaign
distrusts everywhere else.

Measured at the narrow end, total reads held constant at 200,000, release build:

| fields | 2 | 5 | 10 | 20 | 50 | 200 |
|---|---|---|---|---|---|---|
| µs per 1k reads | 444 | 265 | 209 | **188** | 223 | 415 |

The curve is **U-shaped**. Per-read cost is at its *minimum* between ten and twenty fields and rises
in both directions — at width 2 because loop overhead is amortised over two reads, at width 200
because the scan begins to dominate. In the five-to-twenty band the field lookup is not the cost;
interpreter overhead is. A per-instance hash map would add an allocation to every record to speed up
the part that is already free.

RULED: **C55 stays a published limit**, unchanged in substance and corrected in kind — the reasoning
is now data. The wide-region slope agrees with the original table across two different harnesses
(200/50 ≈ 1.86 here, 1.82 there), which is the cross-check that makes both trustworthy.

**D57 — A declared invariant with no gate is a comment.** Closes C62, which was found by accident:
staging this pass's documentation edits produced a 1,927-line diff on a file whose real change was
102 lines.

`.gitattributes` (D19d) declares `* text=auto eol=lf`, adopted on measured evidence — "472/472 LF,
zero CRLF" — and that evidence was correct on the day. Three days later `HARDENING_CAMPAIGN.md` was
created and thereafter rewritten by tooling on nearly every phase, and its committed blob held
**CRLF**. Exactly one file out of 512, for the whole campaign, in the document that itself lists *"a
gate is blind to the failure it exists to catch"* as one of the two rules the campaign produced.
Here the gate was not blind. It did not exist.

RULED: the file is renormalized, and `governance.rs::no_tracked_text_file_is_stored_with_crlf` reads
the **index** — the bytes about to be committed — and fails on any CR in a text blob. It skips what
git classifies as binary, which is what keeps the signed `.dwx`/`.sig` artifacts out of it: their
bytes are a signature's subject and nothing may normalize them. Observed failing against `HEAD`
before the renormalization was staged.

The renormalization is why this pass's diff on `HARDENING_CAMPAIGN.md` is large; the semantic change
is small and the rest is line endings. Said here so nobody has to work that out from the diff.

## 5. Diagnostics budget

DL1901–DL1911 as allocated in spec §10. No other new codes without a ruling here. The three
retired numbers (DL0503/DL0702/DL0906) are never reused (S9-D22).

Post-close-out additions, each ruled above and allocated in its subsystem's range (not mechanically
in DL19xx): **DL1415–DL1418** (broker/federation certs, D22); **DL0107** (lexer bidi-control
refusal, D26). The subsystem-range convention keeps a code's number meaningful — a reader seeing
`DL01xx` knows it is lexical without consulting a table.
