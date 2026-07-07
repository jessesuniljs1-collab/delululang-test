# Stage 7 Playbook — "Concurrent" (actors, reference capabilities, async)

**Companion to:** `docs/design/STAGE7_SPECIFICATION.md` (normative). This file is *how to build it*.
**Depends on:** Stage 6 complete (actors interact with plugins and the broker and **must not change
either**). This is the **deepest type-system extension** in the whole sequence — the only stage that
adds a second checking axis (reference capabilities) alongside the existing effect-row axis.

> **The one-sentence goal:** compile-time data-race freedom via the **actor model with Pony-style
> reference capabilities**, with asynchrony expressed as the `Async` effect in the *same* rows — one
> system, no second coloring mechanism, no `await`. After this stage "no actor accesses another
> actor's mutable state" (Constitution §5.15.2) is a *statically proven* fact.

---

## 0. Orientation — read these before writing anything

- Spec §3 (rcap semantics) — the **deny-property definitions are normative; the tables are their
  consequences.** You will implement the tables, but you must property-test them against the deny
  definitions (criterion 8) or you have not shipped the guarantee.
- Spec §1 invariants 33–37 (no shared mutable state; behaviors atomic; causal row soundness;
  caps unforgeable across actors; keyword migration).
- Spec §4 (the typing-rule delta: T-Actor/Behavior/Ctor/Spawn/Send/Consume/Recover/SyncMethod).
- `CONSTITUTION.md` §5.9, §5.10 (why one concurrency system, why no `await`).
- **Pony's reference-capability papers** — the rcap system is adopted from Pony's production-proven
  design; do not reinvent the lattice, adopt it. The alias table, viewpoint-adaptation matrix, and
  deny properties in spec §3 are the Pony system; treat them as ground truth.

### The two hardest ideas, stated plainly

1. **Reference capabilities are a second axis, orthogonal to effect rows.** A value has *both* an
   rcap (`iso val ref box tag trn` — who may alias/mutate/send it) and, if it is a function, a row
   (what effects it performs). They are checked independently and both are checked at send sites. A
   `val` closure can still be `!{Write}`. Do not let the two axes bleed into each other.
2. **Sendability is the whole game.** Everything crossing an actor boundary must be `iso` (consumed,
   uniqueness proven), `val` (deeply immutable), or `tag` (opaque identity). Data-race freedom is a
   *corollary* of sendability + the deny properties — if the rcap checker is right, races are
   impossible with no runtime cost.

---

## 1. Where the work lands

- **`delulu-syntax`:** the grammar/AST delta (spec §2, §2.1) — `actor` decls, `be`/`new`,
  `spawn`/`consume`/`recover`, the `rcap` prefix on types, and the `@`-free keyword migration for
  `consume`/`recover`. This is a real parser extension (the first big one since Stage 1).
- **`delulu-check`:** the rcap checker (a new sub-module — `rcaps.rs`) implementing alias,
  sendability, viewpoint adaptation, `consume` flow-analysis, `recover` environment restriction, and
  the T-Actor/Behavior/Send/Spawn rules. **The effect-row checker is extended, not replaced** — the
  causal-soundness argument (invariant 35) layers actor turns on top of the existing T-CapOp story.
- **`delulu-runtime`:** the actor scheduler (work-stealing, native), per-actor heaps, message
  passing by move/share, quiescence, the `--debug-rcaps` owner-tag race checker.
- **`delulu-wasm`:** the single-threaded cooperative scheduler (v0.7 honesty — identical semantics,
  no parallelism), keeping two-engine parity on *semantics* (which never promised timing).
- **`Async`** joins the core effect set (a one-line-ish change in the effect enum — but it ripples
  through every row rule, `delulu why`, and manifests).

---

## 2. Phase plan (each phase: build green → test green → update spec §"status" → commit)

Order chosen so the **rcap checker is proven on ordinary (non-actor) code first**, then actors are
layered on a checker already known to be sound.

### Phase 7a — keyword migration + grammar/AST for rcaps and actors (parse only)
Add `consume`/`recover` as true keywords with **DL1608 + exact rename repair** and `delulu fmt
--migrate 0.7` (invariant 37 — this must land *first* so the migration story is testable before the
features that need the keywords). Activate `actor`/`spawn`/the six rcaps from the reserved list;
`be`/`new`/`self` contextual **inside actor bodies only**. Parse the full §2 grammar into the §2.1
AST (`Rcap`, `TypeExprR`, `ActorDecl`, `BehaviorDecl`, `Spawn`/`Consume`/`Recover`). No checking yet.
*Test:* parse round-trip of actor decls, rcap-prefixed types, spawn/consume/recover; a pre-0.7
program using `consume` as an identifier → DL1608 + migrate fixes it.

### Phase 7b — the rcap lattice: alias + sendability, unit-tested in isolation
Implement `alias(κ)` (spec §3 alias table) and `sendable(κ)` (`iso`/`val`/`tag`) as pure functions
with exhaustive unit tests **before** wiring them into the checker. Implement the **default-rcap
rule** (spec §2 "Default rcaps when omitted") — this is ergonomically load-bearing and easy to get
subtly wrong (Int/Str/Cap/Secret/Plugin/actor-refs/immutable-composites → `val`; other
records/lists/closures → `ref`; PyObj/ForeignPtr → `ref` pinned; actor-type-as-type → always `tag`).
*Test:* the alias table cell-for-cell; the default rule for every type family; `sendable` for all six.

### Phase 7c — viewpoint adaptation + field read/write rules (non-actor code)
Implement the viewpoint-adaptation matrix `o ▷ f` (spec §3 table) for field reads, and the write
rule (receiver ∈ {iso, trn, ref}; assigned value storable at the field rcap via the alias table
unless consumed; write via box/val/tag → DL1604). Test on ordinary records — no actors yet. This is
where most rcap bugs live; isolate it.
*Test (criterion 4 subset):* every matrix cell; write through `box` → DL1604; field access on `tag`
→ DL1604.

### Phase 7d — `consume` flow analysis + `recover`
`consume x` yields x's full rcap and **kills the binding** (flow-sensitive definite-unassignment; any
later use → DL1602). Locals/params only in v0.7 (field-consume deferred; distinct DL1602 message).
`recover κ { e }` (κ ∈ {iso, val}, default iso) checks `e` in an environment where only `val`/`tag`/
consumed-`iso` outer bindings are visible (else DL1605); lifts the result to κ.
*Test (criterion 3):* consume then use → DL1602; recover building a mutable list, lifted to iso;
capture of an outer `ref` inside recover → DL1605.

### Phase 7e — actor declarations + T-Actor/Ctor/Behavior/SyncMethod (checking)
Check `actor` decls: fields (var fields are legal — actor-owned isolated state, *not* the banned
module-level ambient state), exactly one `new`, `be` behaviors, `fn` methods. **T-Behavior:** body
checked with `self : ref ActorType`; row checked like T-Fn; **every parameter type must be sendable
or DL1601**; return type annotation → DL1606. **T-SyncMethod:** `fn` methods callable only from
`self` (external call on an actor ref → DL1604 — outsiders hold `tag`; messages are the only
cross-actor interface).
*Test:* an actor with a `ref` behavior param → DL1601; a behavior with a return type → DL1606; an
external `fn` call on an actor reference → DL1604.

### Phase 7f — `Async` effect + T-Spawn / T-Send (rows meet actors)
Activate `Async` in the core effect set (obeys every row rule; `delulu why Async` works). **T-Spawn:**
`spawn A(ā)` → `tag A`, row `{Async} ∪ row(A.new)`. **T-Send:** `a.beh(ā)` → `Unit`, row `{Async} ∪
row(beh)`; **iso arguments must be `consume`d** (or fresh recover results) — unconsumed iso → DL1601
with the exact consume repair. This is where invariant 35 (causal row soundness) becomes real: a
behavior that is `!{Write}` sent from a `!{Async}`-only function → **DL0501** (must be `!{Async,
Write}`).
*Test (criterion 5):* the causal-row chain — `delulu why Write` shows `main → spawn/send → behavior →
console.println` across the actor boundary; a send-site row missing the behavior's effect → DL0501.

### Phase 7g — the native actor scheduler + heaps + message passing
Work-stealing thread pool (default = physical cores, `--actors-threads N`); per-actor MPSC mailbox
(unbounded in v0.7 — backpressure is Stage 10); run-to-completion turns; per-sender-pair FIFO causal
ordering (no global order). Per-actor `Rc` heaps; `val` promoted to a shared `Arc` deep-frozen heap;
`iso` sends **move** the object graph (static uniqueness → pointer handoff); `tag` sends copy the
handle. Quiescence exit (main returns *and* all mailboxes empty *and* no turn running).
*Test (criterion 1):* ping-pong 1M messages, quiescence exit, deterministic counts, ≥2× throughput
at 4 threads vs 1 (smoke-level perf bar).

### Phase 7h — the WASM cooperative scheduler (parity on semantics)
Single-threaded cooperative scheduler on the WASM engine: identical turn atomicity, ordering,
quiescence, and traces; no parallelism. Two-engine parity holds because the *semantics* never
promised timing. Label the engine's concurrency mode in `delulu authority`/`run` output.
*Test (criterion 6):* the actor conformance suite runs on both engines with identical observable
results (WASM = cooperative).

### Phase 7i — the debug race checker + causal trace + `--assert-trace`
`--debug-rcaps` (debug builds): every heap object carries an owner tag; a read/write from a non-owner
without `val` promotion aborts with a compiler-bug-class report (DL1610) — **belt-and-braces testing
of the static guarantee, never the guarantee itself.** Trace records gain `actor`/`turn`/`cause`
(sender + send span); `--assert-trace` checks effect ∈ row(executing behavior) **and** effect ∈
row(send site) via the cause chain (the executable form of invariant 35).
*Test (criterion 7):* the actor stress corpus runs clean under `--debug-rcaps`; on Linux CI the same
corpus runs clean under **ThreadSanitizer** (native engine).

### Phase 7j — the rcap property-test gate + `std.actors.Promise[T]` + failures
The **ship-gate** (criterion 8): generate random alias/adaptation sequences and validate them against
the deny properties — no two actors ever reach read/write-incompatible aliases; adaptation cells
consistent with the alias table and deny definitions. **Any counterexample blocks the stage.** Ship
`std.actors.Promise[T]` (a library *actor*, not a language feature — its `then` row plumbing is the
R-4 law applied to a stdlib actor). Behavior panic → poisons its actor (sends dropped+counted,
reported at exit); `--on-actor-death abort` opts into whole-program abort; supervision is post-1.0.

---

## 3. The traps

1. **Rcaps and rows are different axes — never conflate them.** Sendability is about aliasing;
   effects are about what runs. Both are checked at send sites, separately. A `val` closure that
   performs `Write` is fine (it is sendable *and* effectful).
2. **Adopt Pony's tables; do not "improve" them.** The alias table, adaptation matrix, and deny
   properties are proven. Your job is faithful implementation + property validation, not redesign.
   Criterion 8 is the gate — if your tables disagree with the deny definitions, they are wrong.
3. **The debug race checker is a test, not the guarantee.** DL1610 firing means the *static* checker
   has a hole — it is a compiler-bug signal, not a runtime safety net. Never let anyone frame it as
   "we catch races at runtime." Data-race freedom is static and zero-cost.
4. **Behaviors are atomic; there is no suspension.** No `await`, no coroutines inside behaviors (spec
   §5, rejected-and-recorded). `await` stays a reserved word. Two suspension models would break the
   one-system rule — this is a hard "no", not a "not yet for convenience."
5. **`Async` is only an effect.** No futures runtime, no colored functions. `Promise[T]` is a library
   actor. If you find yourself building a second scheduler for async, stop.
6. **`PyObj` is actor-pinned** (invariant 36): it is `ref` and sending it → DL1601 with the pinning
   explanation. CPython has affinity; Python effectively lives on the actor that created its use
   sites. Do not try to make PyObj sendable.
7. **Do not change plugins or the broker.** Actors *use* them (plugin values are `val`; `expose` is
   still synchronous-class through the broker from any actor). If a Stage-7 change requires editing
   Stage-5/6 code, re-examine — the stage must layer on, not modify.
8. **Migration is not breakage-by-surprise.** `consume`/`recover` becoming keywords gets DL1608 + an
   exact rename + `fmt --migrate 0.7` (invariant 37, criterion 10). Land this in Phase 7a.
9. **Liveness is not guaranteed and never claimed** (spec §11): deadlock, livelock, starvation, and
   mailbox exhaustion are out of scope. The guarantee is *race freedom*, full stop. Unbounded
   mailboxes can OOM (backpressure is Stage 10). Copy §11 caveats verbatim into docs.

---

## 4. Definition of done (map to spec §9 acceptance criteria)

Ship when all 11 criteria pass — the load-bearing ones are criterion 5 (causal rows across the actor
boundary — the effect system and the actor system are *one system*), criterion 6 (both-engine
parity), criterion 7 (TSAN clean), and **criterion 8 (rcap property tests — the data-race-freedom
ship-gate; a counterexample blocks the stage)**. Add `## Implementation status` to
`STAGE7_SPECIFICATION.md` per the Stage-3 §8a pattern. Interpreter/native semantics are the reference
the WASM cooperative scheduler must match.

*Stage 7 completes the semantic core: every function typed, every effect rowed, every actor isolated,
every authority sliced. Stages 8–9 make it a product; Stage 10 makes it an industry tool.*
