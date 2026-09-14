# The Mathematics of DeluluLang

**Every mathematical structure this language uses: what it is, where it is used, why it was chosen,
and — the question that actually matters — how strong the result is.**

"How strong" is not a feeling. Every claim below is assigned to **exactly one** of seven categories,
and nothing is allowed to sit between them:

| # | Category | What it means |
|---|---|---|
| 1 | **Mathematically proven** | A paper proof exists and has been refereed. |
| 2 | **Machine checked** | A proof assistant checks it (Lean/Coq/Agda). |
| 3 | **Model checked** | A model checker explores the state space (TLA+/Alloy). |
| 4 | **Property tested** | Generated or exhaustively enumerated inputs, not hand-picked examples. |
| 5 | **Differentially verified** | Two independent implementations agree. |
| 6 | **Fuzz verified** | Adversarial input generation finds nothing. |
| 7 | **Outside the proof boundary** | Explicitly *not* guaranteed, and named as such. |

A claim with no category is a claim to be deleted or demoted. **Category 2 is non-empty only for
the higher-order fragment** — the full type system is still unmechanized, and §12 says so plainly
rather than rounding up.

Findings referenced as `F1`–`F4`, `IF-1`, `P17-*` are recorded with witnesses in
[`design/PROOF_CAMPAIGN.md`](design/PROOF_CAMPAIGN.md).

---

## 1. The authority order — a preorder on spellings, a lattice on canonical forms

**What.** `authority.rs:150-165` defines `child ⊑ parent` as one conjunction over nine dimensions:
the effect set, seven scope dimensions, and `device`. Each must be narrower-or-equal.

**Why this shape.** A *conjunction* means a widening in any single dimension fails the whole check.
There is no averaging, no scoring, no "mostly narrower". That is the only shape for which "the
repair can never widen" is checkable in one place.

**How strong — and the correction this campaign forced.** The file calls this "the ⊑ attenuation
**lattice**". It is not one, on two counts:

- **`⊑` is a PREORDER, not a partial order.** Antisymmetry fails: `./data` and `data` are distinct
  `String`s that `path::resolve` maps to one path, so they attenuate each other while comparing
  unequal (`Authority` derives `PartialEq` structurally). **206 counterexamples**, F1.
- Its **poset reflection** — the quotient by mutual `⊑` — *is* a meet-semilattice, and that is the
  structure the security argument actually needs.

**Where the defect lives matters more than the defect.** Every dimension is antisymmetric *on its
own representation* — proved in Z3. So F1 is **not a property of the algebra**; it is a property of
the path dimension's **encoding**. Fixing it is a canonicalization change, not an algebra change.

Two consequences of the same encoding:

- **`⊓` is not symmetric on representations** (F2, **414 counterexamples**). `intersect_path_sets`
  (`path.rs:94-110`) tests `if desc(x,y) … else if desc(y,x)`, so when two spellings denote one path
  the *first argument's* spelling survives. `A⊓B = {./data}` where `B⊓A = {data}`.
- **`⊑`-equivalent authorities hash differently** (F3). That spelling reaches
  `Authority::to_json` — the canonical form inside hash-chained audit records and under certificate
  signatures — so one logical grant hashes two ways.

**None of these is an authority escalation.** The load-bearing law holds:

> **`⊓` is a genuine GREATEST lower bound and never widens.**

### The repair, 2026-08-06 — and what "canonicalization" turned out to mean

All three are now **closed** by choosing one representative per `⊑`-equivalence class and storing
*that*, applied at the custody boundary (`Broker::issue`, `attenuate`, `attenuate_core`). The
quotient and the representation then coincide, so the poset reflection *is* the representation and
the word "lattice" becomes accurate rather than aspirational.

**The canonical form is an ANTICHAIN of canonical spellings — two collapses, not one.** The
paragraph above (and the Z3 localisation it rests on) named only the first, and that estimate was
wrong in a way worth keeping visible:

1. **Spelling.** `./data`, `data`, `./data/`, `.\data` are one path under four names.
   `path::canonicalize` renders the resolved segments back to a single spelling.
2. **Set redundancy.** `{"./data", "./data/sub"} ≡ {"./data"}`, because `./data/sub` is already
   inside `./data` and contributes nothing to the covered region. Normalizing spellings does **not**
   remove this, so `⊑` stays a preorder if you stop at (1). `path::canonicalize_set` drops every
   element that lies within another, leaving the `⊑`-maximal ones.

The Z3 model could not have found (2): it abstracts each dimension as a set over an opaque element
type with an uninterpreted `within`, so it cannot express one *element* of a set subsuming another.
The counterexample came from the enumerator. **A proof about an abstraction is exactly as strong as
the abstraction's ability to state the property.**

**The theorem now proved** (`the_order_is_antisymmetric_on_canonical_representatives`):
`A ⊑ B ∧ B ⊑ A ⟹ canon(A) = canon(B)`. Take `x ∈ canon(A)`; from `A ⊑ B` get `y ∈ canon(B)` with
`x ⊑ y`, from `B ⊑ A` get `z ∈ canon(A)` with `y ⊑ z`. Then `x ⊑ z` inside the antichain `canon(A)`
forces `x = z`, so `x ⊑ y ⊑ x`, so `x` and `y` have equal resolved segments and equal canonical
spellings. Hence `x ∈ canon(B)`; symmetrically `canon(B) ⊆ canon(A)`. ∎

**Safety of the repair.** Canonicalization changes spelling, never meaning:
`resolve(canonicalize(p)) = resolve(p)` exhaustively over an adversarial corpus, and
`canonicalizing_an_authority_changes_no_containment_decision` checks every ordered pair in both the
fully-canonical and the mixed child-canonical/parent-raw shape. One trap had to be designed around:
a relative component that *looks* like a drive (`./C:`) must not be rendered bare, or it would
re-resolve as the whole of drive `C:` — a widening. Relative canonical forms therefore always carry
a `./` prefix.

| Claim | Category | Evidence |
|---|---|---|
| `⊓` is the GLB; no widening; reflexive; transitive — **all nine dimensions** | **1 + 4** | **Proved in Z3**, 17 obligations (`design/models/authority_algebra.py`), and enumerated exhaustively over real path strings (`delulu-broker/tests/order_laws.rs`) |
| `⊑` is antisymmetric **on raw spellings** | **7 — FALSE, and permanently so** | F1. Not fixable in the comparison: `⊑` is defined through the non-injective `resolve`. Pinned by `raw_spellings_remain_a_preorder_which_is_why_canonicalization_is_required` |
| `⊑` is antisymmetric **on canonical representatives** | **4 — TRUE**, with a written proof | `the_order_is_antisymmetric_on_canonical_representatives`, exhaustive over the universe |
| `⊓` is symmetric | **4 — TRUE** (was FALSE, F2) | The meet emits canonical representatives, so argument order cannot decide the result |
| `⊑`-equivalent authorities hash identically | **4 — TRUE** (was FALSE, F3) | `equivalent_authorities_serialize_identically`; canonicalized at the custody boundary before hashing |

---

## 2. The path dimension — unions of subtrees

**What.** A path scope is a set of paths denoting the **union of their subtrees**.
`is_descendant_or_equal` (`path.rs:76-87`) is component-wise prefix comparison after lexical
normalization, and `all_within` (`path.rs:114-122`) is a **cover** relation: every child path lies
under *some* parent path.

**Why.** Unions of subtrees are closed under intersection — `subtree(p) ∩ subtree(q)` is
`subtree(deeper)` when one is a prefix of the other and `∅` otherwise. That closure is exactly why
`intersect_path_sets` computes a true GLB rather than an approximation.

**Why purely lexical, never touching the disk.** `path.rs:4-10` is explicit: `⊑` compares two grant
*specifications*, and that comparison must be deterministic, machine-independent, and defined for
paths that do not exist. (The *enforcement* path is different and does consult the filesystem —
campaign finding C84 — because a lexical check cannot see a symlink.)

**And the enforcement path had a case even the filesystem could not settle for it.** C84 made the
runtime resolve links before comparing, but a write must be able to *create* its file, so the resolver
fell back to canonicalizing the nearest existing ancestor and re-appending the rest — on the stated
grounds that what remains are "plain names the OS has not yet been asked to interpret". That is false
for a **dangling** symlink, where `canonicalize` fails exactly as it does for an absent name; the
link's own name was re-appended, the check passed, and the open then followed it out of the grant
(`SYMLINK-DANGLE-1`, 2026-08-10, fixed). This is not a defect in the *order* — the mathematics above
is untouched — but it is the sharpest available reminder that **`⊑` bounds what a grant may say, and
never what the filesystem will do with a path.** Those are two different guarantees, and only the
first is what this document proves.

**How strong:** category **4**, exhaustively enumerated over every subset of a path universe
deliberately containing several spellings of one path. That enumeration is what found F1–F3.

---

## 3. Effect rows — finite label sets with one tail variable

**What.** A row is a finite set of effect labels with an optional tail variable:
`ρ ::= {ℓ̄} | {ℓ̄ | ε}` (`design/DELULU_CORE.md` §2). Rows unify **by equality**, with binding for the
tail variable — **there is no row subtyping** (audit rule R-3), and subsumption exists at exactly one
site: the function-body check (T-Abs).

**Why equality rather than subsumption.** Covariant treatment of a row in *parameter* position is a
total soundness failure — a caller could pass a function with a wider row than the signature admits.
Equality-binding is sound in every position **regardless of variance**, which is why the design
deliberately buys restrictiveness with soundness. Conflicting bindings for one row variable are
**DL0504 and never a union** (R-3b): union-merging would silently widen.

**How strong.**

| Claim | Category | Evidence |
|---|---|---|
| Runtime trace ⊆ statically computed row | **4 + 6** | 250,000 generated programs; 151,670 executed with the trace checked (`delulu-fuzz`), 0 violations |
| The rules are complete for the surface language | **7** | Not established. §11 explains why. |
| Inference has principal types | **7 — FALSE** | **F4**: swapping two parameters decides whether a program compiles |

**F4 in one line:** `e := {Read}` satisfies both constraints, but unification binds `e := {}`
greedily from the first parameter and never backtracks, so R-3b then correctly reports a conflict
the *solver manufactured*. Fail-closed, so no authority escapes — but for an agent-facing language,
"reorder your parameters" is not something to leave undocumented.

**The fuzzing claim earned its category the hard way.** Until this campaign the generator emitted
four templates with no generics, no closures, no higher-order builtins and no control flow — so it
**could not express either of the two soundness holes this project has had**. The grammar *is* the
coverage. `delulu-fuzz/src/danger.rs` now generates those shapes, with tests asserting it really
does.

---

## 4. Secrets and declassification — a capability gate, NOT noninterference

**What it is.** `Secret[T]` is opaque: `str`, `==`, serialization and interpolation are refused
(DL0602/0604/0605). Two operations get information out, and **both carry the `Declassify` effect**:
`expose` (the whole value, and it needs `Cap[Declassify]`) and `verify` (one bit, constant-time).

**Why an effect and not just a capability.** Audit rule R-2: declassification must be *visible in
the row*, so `delulu authority` can report it before you run the program. That is the whole product.

**What it is NOT, stated plainly.** This is **not noninterference**, and the difference is not
academic:

- **There is no implicit-flow tracking.** `if` carries no pc-label. Branching on a secret-derived
  `Bool` and acting differently per branch moves information with nothing to stop it.
- **`verify` reveals one CHOSEN bit per call, and needs no capability.** Because `Secret.map` hands
  its closure the **plaintext** (gated only on *purity* — and purity is not confidentiality), the
  caller picks the predicate: `k.verify(k.map(fn(x) { g }))` tests the secret against any `g`.
  Iterated, that recovers the whole plaintext.

**This was found by breaking it.** Campaign finding **IF-1**: a program recovered an entire API key
while `delulu why Declassify` reported *"program cannot perform `Declassify`"*. The fix (closing
rule **R-2b**) makes `verify` carry `Declassify` in both halves of the primitive table, so such a
program is now **DL0501** unless it declares the effect.

**The fix buys visibility, not impossibility — and that distinction is the honest one.** A program
that *declares* `!{Declassify}` may still do it, and `authority` then tells you:
`exposure: API_KEY declassifiable -> files/console`. That *is* what R-2 promises.

| Claim | Category |
|---|---|
| A secret cannot be observed **directly** | **4** — twelve eliminators re-verified refused |
| Every declassification is **visible in the row** | **4** — `delulu-check/tests/secret_oracle.rs`, with controls |
| Noninterference | **7 — NOT PROVIDED.** No implicit-flow tracking exists |
| `verify` needs a declassify capability | **7 — it does not.** Open residue; closing it needs `Secret[Bool]`, which the runtime cannot represent |

---

## 5. The audit chain — blake3, prev-linked, domain-separated

**What.** Each record stores `prev_hash` and `hash = blake3(prev ‖ canonical_json(body))`
(`audit.rs:354-396`). Canonical JSON sorts keys recursively (`audit.rs:657-677`). Signatures are
**domain-separated**: `delulu-grant-v1` vs `delulu-receipt-v1` (`cert.rs:69-71`, `422-424`).

**Why domain separation.** Without a distinct context prefix, a signature over one object type can
be replayed as another. This project gets it right and **tests it** — a receipt signature cannot
validate as a grant, nor either as an artifact signature.

**How strong — and the limit that must not be blurred.**

| Claim | Category | Evidence |
|---|---|---|
| Detects **modification** of any record | **4** | `delulu-broker/tests/audit_truncation.rs` (control) |
| Detects **reordering** | **4** | `prev_hash` chain break |
| Signatures are domain-separated | **4** | `cert.rs` tests |
| Detects **truncation** | **4 — FIXED** | `ANCHOR.json` + `verify`; regression witness in `audit_truncation.rs` |
| Resists an attacker who rewrites BOTH log and anchor | **7 — it does not** | needs an EXTERNAL witness; pinned as a passing test |

**P17-C1, fixed.** Every check `verify` performs is **local to a link**, so deleting the last *k*
records left a chain in which every remaining link was still correct — `verify` returned `Ok`, just
shorter. Nothing anchored the head: `AuditLog::open` *recovered* it from the files, so the broker
resumed chaining from the truncated head and every later record was genuinely valid.

`ANCHOR.json` now records the head and the record count **outside the log**, refreshed on every
append; `verify` compares against it and `AuditLog::open` refuses to start on a disagreement.

**The claim is now "detects modification, reordering, and truncation" — but still NOT
"tamper-proof".** The anchor sits beside the log, so an attacker who deletes records can also
rewrite it. What is closed is accidental truncation and naive tampering; what remains open needs an
**external witness**, and that limit is pinned as a passing test rather than left in prose.

---

## 6. Device envelopes — interval containment

**What.** A device grant is a name, a fail-state, per-dimension closed intervals, an optional rate
limit, a heartbeat and a TTL. `within` (`device_scope.rs:208-232`) requires the same device and
fail-state, interval containment per dimension, and — the three that are easy to invert:

- a **smaller heartbeat is NARROWER** (you must check in more often),
- a **smaller TTL is NARROWER** (authority dies sooner),
- an **unbounded rate under a bounded parent is a WIDENING**, and is refused.

A child dimension the parent does not admit is a **widening**, refused rather than skipped — the
whitelist rule, written as `else { return false }` and not `else { continue }`.

**How strong:** category **1**. `within` and `meet` are **proved in Z3** to be reflexive,
transitive, a lower bound, and a genuine **GLB**, with the model faithful to all three asymmetries
(`design/models/authority_algebra.py`).

---

## 7. Time — a wall clock, ratcheted so it cannot go backwards

**What.** Expiry compares a node's absolute deadline against `SystemTime::now()` (`time.rs:14-25`),
folded up the ancestor chain by `effective_state_inherited` (`tree.rs:576-590`).

**Why inherited at read time.** `attenuate` bounds a child's authority by `⊑` but **not its
deadline**, so a holder could otherwise delegate itself a `ttl: None` child and outlive its own
lease. Inheriting at *read* time rather than clamping at *write* time is deliberate: a contact
receipt extends an adopted root's deadline and the whole subtree must come with it.

**How strong.**

| Claim | Category | Evidence |
|---|---|---|
| A node is live only if every ancestor is live | **3** | TLA+, and a teeth test reconstructs the pre-fix bug |
| Expiry is **permanent**, under any clock motion | **4 — FIXED** | the ratchet in `Broker::now`; witness in `clock_monotonicity.rs` |
| The clock is **accurate** | **7 — monotonic, not accurate** | a rewind still distorts measured intervals |

**P17-B2, fixed.** A grant deadlined at t=5,000 reported `Live` at 1,000, `Expired` at 9,000, and
**`Live` again at 2,000** — no revocation, no audit event, nothing recording that authority had
returned.

**Why it mattered for the stated users specifically.** On satellites, autonomous aircraft and robots
a backwards clock step is **routine, not adversarial**: GNSS acquisition after a cold start, an NTP
correction, an RTC read at power-on. And the uplink lease exists precisely to be *the bound that
survives a partition* — because revocation cannot cross one — and that bound is a wall-clock
deadline. The guarantee rested on clock monotonicity, an assumption the design never stated.

**The fix is a ratchet, not a monotonic clock.** `Broker::now` takes the running maximum of every
reading it has ever taken. `Instant` was not available: certificate `not_before`/`not_after` are
**signed absolute epoch-millis**, so the reading must stay wall-clock-comparable or a certificate
minted by the ground could not be evaluated here at all. The ratchet keeps it comparable while making
it non-decreasing — forward jumps advance it, backward jumps are clamped, **and what expired stays
expired**.

**Direction matters and is pinned as a test:** the ratchet can only ever *withhold* authority, never
grant it. Clamping upward can expire something early; it can never un-expire anything.

**Residue:** monotonicity is not *accuracy* — a clock set back and forward again still measures the
interval differently from wall time. And the ratchet is per-broker in-memory state, so a restart
begins afresh; that is sound only because the grant tree does not persist either (P17-B1), so every
node a restarted broker holds was created after the restart.

---

## 8. The broker state machine — model checked

**What.** Grant → delegate/attenuate → lease → redeem → validate → revoke → expire → adopt.
Modelled in TLA+ at `design/models/`, every action's guard citing the `file:line` it mirrors, and
taking the **weaker** guard where the code is ambiguous so the model can never be kinder than the
implementation.

**How strong:** category **3**.

| Model | States | Result |
|---|---|---|
| `Broker.tla` — the grant tree | **585,771 distinct** | no violation of attenuation, revoke-covers-subtree, no-resurrection, inherited expiry, audit append-only |
| `Custody.tla` — leases + certificates | **2,421 distinct** | no violation of single-use, no-dead-redemption, rotated-tokens-dead, revocation-survives-readoption |

**These models are shown to have teeth rather than asserted to — three times.** Remove a fix and TLC
reconstructs the corresponding **real historical bug**, none of which was described to it:

1. a `ttl: None` child outliving its parent's expired uplink lease (depth 4);
2. a certificate re-presented after a revocation, restoring killed authority (depth 4);
3. a token redeemed against a revoked grant, writing `decision: "allow"` into the audit chain
   (depth 5) — campaign finding C29.

**A model that has never caught anything is indistinguishable from one that cannot.**

**Bounds:** three nodes, two effects, clock ≤ 2. Bounded model checking holds *within* the bound and
is not a proof for all sizes. **Concurrency, partitions and clock skew are NOT modelled** — which
matters, because the uplink lease exists to bound behaviour during exactly a partition.

---

## 9. Actors — capability sharing, and why no `⊑` obligation exists

**What.** Actors are worker-owned and pinned at spawn. Every behaviour parameter must be **sendable**
— `iso`, `val` or `tag`, else DL1601 — and *undecidable sendability refuses*, because a guarantee
that cannot be established is not granted.

**Capabilities do cross message boundaries.** Verified by running: a bare `Cap[Console]`, a `val`
one, a `tag` one, a capability stashed in actor `var` state and fired later, and `val Root` — the
whole authority of the program — are all accepted and all run.

**Why that is sound.** You can only send a capability you already **hold**, and capabilities are
**unforgeable** (`CapVal` has no constructor from data; minting is broker-only). Message passing
therefore *shares* authority — **it cannot widen it**, so there is no `⊑` obligation to check. The
`⊑` check belongs where authority is *minted*, and that is where it is.

**And the accounting holds**, which is the part that had to be measured: `delulu authority` reports
`effects: Async, Write` and `capabilities: Console stdio` for every case, *including* the one where
an actor holds `Root` and derives a console inside the actor.

**Data races.** `actors.rs` contains **zero `unsafe`** — the topology earns it: actors never migrate,
`Value` is `Rc`-based and deliberately **not** `Send`, and messages cross only as `MsgValue`, an
owned `Send`-by-construction representation. Per-sender-pair FIFO falls out of `mpsc` rather than
being asserted. Category **1**, by construction.

---

## 10. The Survey — graph theory, and the one piece nobody had written down

**What.** `docs/survey/` is a **directed labelled multigraph** `G = (V, E)` — 1,019 nodes and 8,927
edges at the time of writing — where every edge carries a `file:line` provenance.

**The queries are reachability.** `impact <id>` and `rdeps` compute the **transitive closure** of the
edge relation from a node: "blast radius" is exactly the reachable set `R*(v)`, walked by BFS.

**Two totality properties, and `doctor` checks both:**

- **No edge dangles** — `∀(u,v) ∈ E : u, v ∈ V`. The endpoint map is total.
- **Every edge cites a line** — the provenance function is total.

**Why this is stronger than a hand-written map, which is the whole point.** The graph is *derived*
from the tree, so **staleness is decidable**: regenerate and compare. A test fails when it is behind.
A hand-maintained map has no such property, which is why `REPOSITORY_STRUCTURE.md` has drifted twice
in six weeks and the Survey has not drifted once.

**How strong:** category **4** — the totality properties are checked over the whole generated graph
on every run, not sampled.

---

## 11. The core calculus — paper sketches, two of them defective

**What.** `design/DELULU_CORE.md` states Delulu Core: a typing judgment `Γ ⊢ e : τ ! ρ`, an
operational semantics over a capability store, and three theorems — Progress, Preservation, and
**Effect Soundness** (`labels(tr) ⊆ ρ`, "authority cannot escape the type").

**How strong: category 1 at best, and two theorems need repair before mechanization.**

**P17-T1 — the calculus does not model the construct that broke.** Searching the document for
`higher-order`, `callback`, `invoke`, `R-4` or `map` returns **zero occurrences**. `Σ` gives each
`op_ℓ[R]` argument types and a *single* emitted label; `E-Op` reduces in one step emitting exactly
that label. **There is no construct for a primitive that invokes a function argument.** `T-Op`
computes its row as `{ℓ} ⊔ ρ₀ ⊔ ⊔ᵢ ρᵢ` — the rows of *evaluating* the arguments, which for a lambda
is `{}` because T-Abs makes closure construction pure. **A callback's latent row never enters.**

So Theorem 3 is provable **and true of the calculus** while the implementation was unsound.
**Mechanizing §1–§7 as written would not have caught C88** — it would prove the wrong theorem.

**P17-T2 — Progress is FALSE as stated.** `E-Op` carries `(scope of κ permits the arguments)` as a
**premise**. A capability that is present and well-typed but whose *scope* does not cover the
argument makes `E-Op` inapplicable and no other rule applies — so a well-typed closed term is
**stuck**, which Progress forbids. Observed: a program granted `fs.read=./data` reading
`../outside.txt` **checks clean** and then faults `DL0904`. The document has **no fault
configuration at all**. The correct statement is **progress-or-fault**.

**P17-T3 — §6's faithfulness claim is weaker than stated.** §6 justifies its exclusions as making the
calculus faithful to the implemented language, and one exclusion is "no ambient mutable cell through
which a capability could launder". **Actor `var` state is exactly such a cell and can hold a
capability** — observed. The document mentions "actor" zero times.

None of these means the calculus is *wrong*. It means it is **silent**, and silence is the one thing
a soundness argument may not be about the construct that failed.

### The repair, machine checked

`design/models/lean/DeluluCore.lean` formalises the **extension** P17-T1 says the calculus needs —
a primitive that invokes its function argument — and proves two things in Lean 4.32.2:

- **`good_sound`** — with the corrected rule (the callback's latent row surfaces into the caller),
  every emitted label is in the declared row. Effect Soundness, for this fragment.
- **`bad_unsound`** — with the rule the calculus actually states, **there exists a well-typed
  program whose trace escapes its row**. That is C88, mechanized:
  `ho (lam [write] (op write))` types at row `[]` and emits `write`.

`#print axioms` reports **"does not depend on any axioms"** for both, and for
`c88_good_row_contains_write` — fully constructive, not even `propext`.

**Category 2, and scoped honestly:** this is the higher-order fragment *only*. It settles the one
question that cost this project its worst soundness hole, and it settles nothing else.

---

## 12. The ledger — which categories are actually non-empty

| # | Category | Status |
|---|---|---|
| 1 | Mathematically proven | **Non-empty.** The nine-dimension order (Z3, 17 obligations); device containment; actor data-race freedom by construction. |
| 2 | **Machine checked** | **NON-EMPTY, for the first time — but narrowly.** Lean 4.32.2 proves Effect Soundness for the **higher-order fragment**, and proves that the calculus as written admits a program whose trace escapes its row (`design/models/lean/DeluluCore.lean`). `#print axioms` reports **"does not depend on any axioms"** for all three theorems — not even `propext` or `Classical.choice`. **The full type system is still NOT mechanized**: no capabilities, no store, no secrets, no attenuation, no Progress/Preservation. |
| 3 | Model checked | **Non-empty.** 585,771 + 2,421 distinct states, with three teeth tests reconstructing three real bugs. |
| 4 | Property tested | **Non-empty.** 250,000 generated programs; exhaustive enumeration of the order laws; the Survey's totality properties. |
| 5 | Differentially verified | **Non-empty.** Interpreter vs WASM engine, ~1,000 lines of parity tests; fault parity is a tested law. |
| 6 | Fuzz verified | **Partial.** The differential harness runs; `cargo-fuzz` targets are installed but **not yet written**. |
| 7 | Outside the boundary | **Populated and named** — see below. |

### Explicitly outside the proof boundary

1. **Noninterference for secrets.** No implicit-flow tracking; `verify` reveals one chosen bit per
   call without a capability.
2. **Audit-chain truncation.** Detected *by the chain's own links*: **no** — every check `verify`
   performs is local to a link, so a shortened chain still verifies. Detected *at all*: **yes, since
   2026-08-05** — an external `ANCHOR.json` holds head and count. What stays outside the boundary is
   an attacker who rewrites the anchor too, and that limit is pinned by a *passing* test.
3. **Clock monotonicity.** Assumed, never stated, and false on the target platforms. **Since
   2026-08-05 the broker's reading is ratcheted** (running maximum), so a backwards step can only
   withhold authority, never resurrect it — the *host* clock is still not monotonic.
4. **Antisymmetry of `⊑` on raw spellings** — still FALSE and permanently so (F1), because `⊑` is
   defined through a non-injective resolution. **On canonical representatives it is now proved**,
   and the broker stores only those, so the structure it is claimed to have is the structure it has.
5. **Principal types.** Inference is order-dependent (F4).
6. **Foreign code.** A `ForeignCall` is a hole in the guarantee — enumerated, not eliminated.
7. **Multi-tenancy.** Not provided; separate OS accounts required.
8. **Concurrency, partitions and clock skew** in the broker model.
9. **macOS.** Runs, not yet green: 1,654 of 1,655 tests on CI's macOS runner (2026-09-14); the one
   failure is fixed, pending the next run.
10. **Side channels**, including timing.
11. **Root issuance against a same-OS-user adversary (DISC-1, 2026-08-08).** The Guard gates
    *delegated* grants; in the LEGACY default **root** creation is ungated and headless
    (`ReqBody::Issue`), so a same-uid process — the common AI-agent deployment — can mint a root and
    command a Guard-*sealed* resource. Proven executably. **Opt-in strict mode is now shipped**
    (`broker start --require-anchored-roots`): a root may then enter only via an anchor-verified
    certificate, and the invariant *"no root exists in strict mode unless justified by a chain
    verifying against the pinned anchor"* is **category 4 (property/differentially tested + falsified)**
    — `strict_mode_no_root_without_a_chain_verifying_against_the_pinned_anchor`,
    `strict_mode_refuses_unsigned_issue_over_the_wire`. But the SECURITY of that invariant is
    **category 7**: against a same-uid adversary **no local secret** (owner code, TTY, env var, readable
    file, `root_policy.json`) is a boundary — only a separate OS account or an out-of-band anchor key
    is, and the code cannot guarantee that custody. *"The code verifies the signature"* ≠ *"secure
    against same-user compromise."* Constitution §5.16 law 4 and spec §112 still claim a "never
    programmatic / never headless" control that is **not** what is enforced; do not cite it as
    guaranteed. Threat model, architecture, evidence, and the migration analysis:
    `design/ROOT_ISSUANCE_TRUST_BOUNDARY.md`.

    **Update 2026-08-10 — the category does not move, but two things about it changed.** Strict mode
    had never actually been *runnable*: three defects (a certificate's relative filesystem scope
    matched nothing; the refusal surfaced as `DL1401 broker unreachable` because a hand-written code
    list had drifted by exactly this diagnostic; the scope flags were absent from `--help`) made the
    documented path fail end to end. All three are fixed and the whole path is verified. An invariant
    nobody can execute is not category 4 in any useful sense, so this raises the *evidence* to what it
    already claimed. Additionally: (a) every broker start now writes its **effective** mode into the
    hash-chained audit log, so a same-uid downgrade must leave permanent evidence or break
    `audit verify` — **detection**, which is what remains available when prevention is not; and (b)
    `delulu doctor` reports the mode, the anchor-key custody, and whether the filesystem enforces
    owner-only permissions at all. **The guarantee is still category 7** — a deployment property of
    the OS, not of this code — but it is now a deployment property that can be *checked*, which is a
    different thing from one that can only be *read about*. Recipe: `DEPLOYMENT.md`.
12. **Broker availability against a same-OS-user adversary (IPC-1 / DEADMAN-1, 2026-08-08).** The
    broker's single-connection serve loop and the dead-man watchdog's authority probe now BOUND their
    reads (`Connection::set_read_timeout`), so a stalled client can no longer hang the daemon
    indefinitely, and a hung broker can no longer stall the dead-man on Unix — the automatic heartbeat
    park is now fail-closed independent of broker responsiveness (**category 4**, pinned + falsified by
    `request_timed_fails_closed_on_a_broker_that_accepts_but_never_answers`). What stays **category 7**:
    full availability against a same-uid adversary is not guaranteed — it can still churn/dribble
    connections or simply `kill` the daemon (it shares the OS user). The *indefinite* hang is closed;
    saturation DoS by a co-resident same-uid process is a deployment property (run untrusted agents as a
    separate OS user). See `security/red-team-surfaces-2026-08-08/`.
13. **The separate-OS-account boundary — now tested, and its filesystem dependency (P21, 2026-08-08).**
    Items 11 and 12 both rest their residual on one claim — *"a separate OS account is the boundary"* —
    which had only ever been asserted (every prior test ran same-uid on one account). P21 **demonstrates
    it against the live binary with a real second uid** under WSL: a broker run by `user` (uid 1002)
    with its state dir on a POSIX filesystem denies a separate account (`attacker`, uid 1003) on all
    five vectors — directory traversal, `broker.key` read, IPC custody op (fail-closed `DL1401`, no
    local-state fallback), raw socket connect, and `kill` — while the same-uid control succeeds on all
    four. This is a **reproducible red-team differential** (the P21 scripts), not an in-suite property
    test and not a proof; it moves the claim from *asserted* to *verified-by-demonstration*. The
    guarantee itself stays **category 7**, because it is a property of the operating system and the
    filesystem, not of delulu's code. P21 also shows the sharp edge of that dependency: on a non-POSIX
    mount (9p/DrvFs under WSL; by the same mechanism NFS-without-mapping, SMB, exFAT/FAT)
    `chmod 0600/0700` is a **silent no-op** — a separate account read a "0600" file and the broker's
    `broker.key` — and delulu could not tell, because every `set_permissions` discarded its result.
    That is no longer silent — and no longer merely a warning: before writing a private key (`keygen`)
    or starting the broker, delulu probes the target directory with a throwaway file and **refuses,
    fail-closed**, when the filesystem does not enforce owner-only permissions, so the secret is never
    written (`crates/delulu/src/signing.rs`, `crates/delulu/src/brokerd.rs`); a post-write warning
    (`crates/delulu/src/broker_transport.rs`) backstops the `keygen` override. The broker also cannot
    bind its `AF_UNIX` socket on 9p at all (`ENOTSUP`). Evidence and both transcripts:
    `security/red-team-p21-crossaccount-2026-08-08/`.
14. **The filesystem check-then-open race (`CONTAIN-TOCTOU-1`, 2026-08-10).** Containment resolves a
    path and the operation that follows re-opens it **by name**, so a writer acting between those two
    moments can substitute a symlink. The confined program cannot do this through the primitive table
    — no operation there creates a link — so it needs a *second* writer: a same-uid process (already
    items 11–12) or anyone able to write into the granted directory, which is why a grant aimed at a
    shared location such as `/tmp` is a different proposition from one aimed at a private directory.
    Closing it means checking the opened **handle** (`O_NOFOLLOW`/`openat2`, `FILE_FLAG_OPEN_REPARSE_POINT`),
    which would make containment platform-dependent — the one property this project refuses — so it is
    named here rather than fixed. **Deployment rule: grant scopes that point at directories only the
    program's own user can write.**
15. **What `⊑` proves, and what it does not.** Worth stating in this list because two campaign findings
    landed on the seam. The order bounds what a grant may **say**; the filesystem decides what a path
    **does**. `SYMLINK-DANGLE-1` (a dangling link the resolver could not settle) and `GUARD-SPELL-1` (a
    guard rule compared against a *resolved absolute* path while the operator had written a relative
    one) were both defects in that second half, and neither touched the algebra §1–§2 proves. A proof
    about the specification lattice is not a proof about path resolution, and this document should not
    be read as offering one.

---

## 13. The one-paragraph answer

DeluluLang's mathematics is **real but uneven, and now honestly labelled**. The authority order is a
preorder on raw spellings whose poset reflection is a meet-semilattice — and since the broker stores
only canonical representatives (canonical spellings, reduced to an antichain), the reflection and the
representation now coincide, so `⊑` is a genuine partial order on everything the system can build.
Its load-bearing law — *the meet is a greatest lower bound and never widens* — is **proved in Z3
across all nine dimensions**. The broker's
state machine is **model checked**, and the models are demonstrated to have teeth by rediscovering
three bugs the project actually shipped. Effect soundness is **property- and fuzz-tested over 250,000
generated programs** whose grammar now contains the shapes that historically broke it. The Survey is
a derived graph whose staleness is decidable, which is why it does not rot.

The **higher-order fragment of the effect calculus is now machine checked in Lean**, with no axioms
at all — including a mechanized proof that the calculus *as previously written* was unsound for that
construct.

Against that: **the full type system still has no machine-checked proof** — only the higher-order
fragment has one — the core calculus remains silent about actors and mutable cells, and secrets are
protected against direct observation but not against a program that is trying.

**Two limits this paragraph used to overstate are now narrower, and §5 and §7 state them at their
true width.** Deletion of trailing audit records **is** detected, by a head-and-count anchor held
outside the log; what stays open is an attacker who rewrites the anchor as well, which needs an
external witness. And a backwards clock can **no longer** resurrect expired authority, because the
broker's reading is ratcheted to its running maximum; what stays open is that monotonicity is not
*accuracy* — a clock set back and forward again still mismeasures the interval. Both were left
reading as unfixed here after the fixes landed, which is the same drift this document exists to
prevent, appearing in its own summary.

**The design is mathematical. Several subsystems now have machine-checked, model-checked or
symbolically proved evidence — each bounded and labelled. The full type system's guarantee is still
the tests.** Anything stronger would be a lie of exactly the kind this project exists to avoid.
