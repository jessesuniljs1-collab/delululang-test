# DeluluLang — Stage 7 Implementation Specification

**Version:** 0.7 ("Concurrent"). **Status:** Committed — buildable directly from this document.
**Depends on:** Stage 6 complete (plugins and broker semantics must already be stable, because
actors interact with both and this stage must not change either).
**Governing documents:** `CONSTITUTION.md` (§5.9, §5.10), `SOUNDNESS_AUDIT.md` (the row-soundness
argument is *extended* here, not replaced).

---

## 0. Scope and goal

**Goal:** deepen the guarantee to concurrency — the **actor model with Pony-style reference
capabilities**, giving compile-time data-race freedom and per-actor authority slices, with
asynchrony as the `Async` effect in the same rows (one system, no second coloring mechanism).
After this stage, the runtime guarantee "no actor accesses another actor's mutable state"
(Constitution §5.15.2) is real.

**In scope (must ship):**
- Reference capabilities (rcaps): `iso val ref box tag trn` in the type grammar, with alias,
  sendability, and viewpoint-adaptation rules (§4).
- `actor` declarations: fields, `new` constructor, `be` behaviors, sync `fn` methods.
- `spawn`, behavior sends, `consume`, `recover`; the `Async` effect activates.
- Runtime: work-stealing actor scheduler (native engine), single-threaded cooperative scheduler
  (WASM engine, v0.7 — honesty in §7.3), per-actor heaps, message passing by move/share.
- `std.actors.Promise[T]`.
- Trace/`--assert-trace` extension to causal (cross-actor) effect attribution.
- New diagnostics: DL16xx.

**Non-goals:** `await`/suspension inside behaviors (post-1.0 RFC — behaviors are atomic,
Pony-style; two suspension models would break the one-system rule; `await` stays a reserved
word); actor supervision trees and restarts (post-1.0; failures follow §7.5); distributed actors
(post-1.0); multi-threaded WASM (Stage 10); per-actor broker nodes (post-1.0 — §7.6 note).

---

## 1. Invariants (carried + new)

All prior invariants hold. New:

33. **No shared mutable state across actors, statically.** Every value crossing an actor boundary
    (message argument, constructor argument) is *sendable*: `iso` (consumed — uniqueness proven),
    `val` (deeply immutable), or `tag` (opaque identity). There is no dynamic path around this
    (no unsafe cast, no reflection — unchanged from Stage 1).
34. **Behaviors are atomic.** A behavior runs to completion on its actor; an actor processes one
    message at a time; `fn` methods run only within the actor's own turn. No suspension exists.
35. **Causal row soundness (the Stage-1 property, extended).** Every runtime effect is performed
    inside some behavior/function whose static row contains it, and every behavior's row is
    contained in the row of every *send site* that targets it (T-Send). Whole-program authority
    remains `row(main)`: all actors are spawned, and all messages sent, transitively from `main`.
36. **Capabilities stay unforgeable across actors.** `Cap[R]`/`Secret[T]`/`Plugin[_]` are `val`
    (immutable handles, safely shareable); `PyObj` is `ref` and **actor-pinned** (CPython
    affinity; sending it is DL1601, using it from another actor is impossible by construction).
37. **Keyword migration is tooling, not breakage-by-surprise.** `consume` and `recover` become
    keywords (they were not in the Stage-1 reserved list — an acknowledged Stage-1 omission);
    pre-0.7 code using them as identifiers gets DL1608 with an exact rename repair, and
    `delulu fmt --migrate 0.7` applies it corpus-wide.

---

## 2. Lexical and grammar additions

New keywords: `consume`, `recover` (true keywords, migration per invariant 37). Activated from
the reserved list: `actor`, `spawn`, and the six rcaps. Contextual keywords **inside `actor`
bodies only**: `be`, `new`, `self` (top-level code may still use them as identifiers — no
migration needed for them).

```ebnf
item          = [ "pub" ] , ( fn_decl | type_decl | effect_decl | const_decl
                            | foreign_decl | actor_decl ) ;
actor_decl    = "actor" , IDENT , [ generics ] , "{" , { actor_member } , "}" ;
actor_member  = field_decl | ctor_decl | behavior_decl | fn_decl ;
field_decl    = ( "let" | "var" ) , IDENT , ":" , type , term ;
ctor_decl     = "new" , "(" , [ params ] , ")" , [ effect_row ] , block ;   (* exactly one per actor *)
behavior_decl = "be" , IDENT , "(" , [ params ] , ")" , [ effect_row ] , block ;
                (* behaviors have no return type: they yield Unit at the send site *)

type          = [ rcap ] , type_core ;
rcap          = "iso" | "trn" | "ref" | "val" | "box" | "tag" ;
type_core     = (* the entire Stage-1 type production, unchanged *) ;

expr_add      = "spawn" , path , "(" , [ expr , { "," , expr } ] , ")" ;
unary_add     = "consume" , IDENT
              | "recover" , [ rcap ] , block ;      (* default lift target: iso *)
```

**Default rcaps when omitted** (ergonomics rule, normative): `Int Float Bool Str Unit`,
`Cap[R]`, `Secret[T]`, `Plugin[_]`, actor references, and `Option/Result/List/records of only
such types` default to `val`; all other records/lists/closures default to `ref`; `PyObj` and
`ForeignPtr` default to `ref` (pinned). An actor type used as a type is always `tag` (writing
another rcap on an actor type is DL1607).

### 2.1 AST delta

```rust
pub enum Rcap { Iso, Trn, Ref, Val, Box_, Tag }
pub struct TypeExprR { pub rcap: Option<Rcap>, pub core: TypeExpr }   // rcap None = default rule
pub struct ActorDecl { pub public: bool, pub name: Ident, pub generics: Vec<Ident>,
    pub fields: Vec<FieldDecl>, pub ctor: CtorDecl,
    pub behaviors: Vec<BehaviorDecl>, pub fns: Vec<FnDecl>, pub id: NodeId, pub span: Span }
pub struct BehaviorDecl { pub name: Ident, pub params: Vec<Param>,
    pub row: Option<RowExpr>, pub body: Block, pub id: NodeId, pub span: Span }
pub enum Expr { /* … */ Spawn { actor: Path, args: Vec<Expr>, id: NodeId, span: Span },
    Consume { name: Ident, id: NodeId, span: Span },
    Recover { target: Option<Rcap>, body: Block, id: NodeId, span: Span } }
```

---

## 3. Reference-capability semantics

Adopted from Pony's production-proven system. The **deny-property definitions are normative**;
the tables are their consequences and must be property-validated (§9 criterion 8) before freeze.

| rcap | You may | Others (aliases) may | Sendable |
|---|---|---|---|
| `iso` | read+write | nothing (no other read or write alias exists) | yes, via `consume` |
| `trn` | read+write | local read (`box`) only; no other writer | no (convert first) |
| `ref` | read+write | local read+write | no |
| `val` | read | global read (deeply immutable forever) | yes |
| `box` | read | local read+write may exist elsewhere | no |
| `tag` | identity/send only | anything | yes |

**Alias table** (the rcap an alias of `x: κ` gets without `consume`):
`alias(iso)=tag · alias(trn)=box · alias(ref)=ref · alias(val)=val · alias(box)=box ·
alias(tag)=tag`.

**Consume:** `consume x` yields `x`'s full rcap and kills the binding (flow-sensitive definite-
unassignment; any later use is DL1602). `consume` of a field is not allowed in v0.7 (locals and
params only — Pony's field-consume subtleties are deferred; DL1602 variant message).

**Recover:** `recover κ { e }` (κ ∈ {iso, val}, default iso) type-checks `e` in an environment
where only `val`, `tag`, and consumed-`iso` outer bindings are visible (anything else referenced:
DL1605); the block's result is lifted to κ. This is how mutable construction becomes sendable.

**Viewpoint adaptation** (reading field of rcap f through receiver of rcap o, `o ▷ f`):

| o \ f | iso | trn | ref | val | box | tag |
|---|---|---|---|---|---|---|
| iso | iso | tag | tag | val | tag | tag |
| trn | iso | trn | box | val | box | tag |
| ref | iso | trn | ref | val | box | tag |
| val | val | val | val | val | val | tag |
| box | tag | box | box | val | box | tag |
| tag | — no field access (DL1604) — |

**Writes:** a field write requires receiver rcap ∈ {iso, trn, ref} and the assigned value's rcap
must be storable at the field's declared rcap (assignment uses the alias table unless consumed);
writing through `box`/`val`/`tag` is DL1604.

**Closures:** a closure's rcap is inferred from captures: all captures `val`/`tag` (or consumed
`iso`) ⇒ closure is `val` (sendable); otherwise `ref`. Explicit annotation allowed; a `val`
lambda capturing a `ref` is DL1603. Rows are orthogonal and unchanged (a `val` closure may still
be `!{Write}` — sendability and effects are different axes; both are checked at send sites).

---

## 4. Typing rules (delta — stated formally)

- **T-Actor:** field types are checked; `var` fields are actor-internal state (legal — the
  no-module-`var` rule is about ambient state; actor state is owned, isolated, and reached only
  via the actor's own turn).
- **T-Behavior:** body checked with `self : ref ActorType`; declared row checked like T-Fn
  (`ε_body ⊆ declared`); **every parameter type must be sendable** — `iso`, `val`, or `tag` —
  else DL1601. Behaviors return `Unit` structurally (a return type annotation is DL1606).
- **T-Ctor:** same sendability rule as behaviors (construction is a send to the new actor).
- **T-Spawn:** `spawn A(ā)` where `ā` match `A.new`'s params ⇒ type `tag A`, row
  `{Async} ∪ row(A.new)`.
- **T-Send:** `a.beh(ā)` where `a : tag A` ⇒ type `Unit`, row `{Async} ∪ row(beh)`; `iso`
  arguments must appear as `consume x` (or fresh `recover` results) — an unconsumed `iso` is
  DL1601 with the exact `consume` repair.
- **T-Consume / T-Recover:** per §3.
- **T-SyncMethod:** `fn` methods in actors: receiver `ref self`; callable only from `self`
  (external calls on an actor reference are DL1604 — outsiders hold `tag`, and tag denies
  synchronous access; *messages are the only cross-actor interface*).

**Extended soundness statement (the audit's §D, clause 5):** effects arise only at T-CapOp inside
some frame; frames belong to turns; a turn executes a behavior whose row contains the frame's
effects (T-Behavior); every send site's row contains that behavior's row (T-Send); transitively,
`row(main)` bounds the program. Data-race freedom: sendability + deny properties ensure no two
actors ever hold read/write-incompatible aliases of one object — validated by property tests
(§9.8) and by the runtime debug checker (§7.4), mechanization deferred with Delulu Core.

---

## 5. Async is an effect — and only an effect

`Async` activates in the core effect set. It obeys every row rule (polymorphism, manifests,
`delulu why Async`). A function that sends messages is `!{Async, …}`; a pure function still
provably does nothing. There is **no second async system**: no futures runtime, no await, no
colored functions — `Promise[T]` (§8) is a library *actor*, not a language feature. **Rejected
(recorded):** stackless coroutines/`await` for v1.0 — two suspension semantics (actor turns and
coroutine points) would double the concurrency model and break Constitution §5.10's "one system."

---

## 6. Runtime

### 6.1 Scheduler (native engine)

Work-stealing thread pool (default = physical cores, `--actors-threads N`); each actor: MPSC
mailbox (unbounded in v0.7 — backpressure is post-1.0, noted in §10), run-to-completion turns,
FIFO per sender-pair causal ordering (no global ordering claim). Program exit: when `main`'s
turn tree quiesces — `main` returns *and* all mailboxes are empty *and* no turn is running
(Pony-style quiescence); `--on-quiesce report` prints surviving actor count for debugging.

### 6.2 Heaps and message passing

Per-actor `Rc` heaps (Stage-1 model, now one per actor). `val` values are promoted at
creation/`recover`-to-val into the **shared immutable heap** (`Arc`, deep-frozen). `iso` sends
**move** the object graph between actor heaps (static uniqueness makes this a pointer handoff;
debug builds verify refcount==1 across the moved graph — §7.4). `tag` sends copy the actor
handle. Rc cycles leak (Stage-1 note carried; per-actor cycle collection is Stage 10 work).

### 6.3 Effects, broker, trace

Capability ops from any actor share the process's grant/lease machinery unchanged (Stage 5
classes apply per-op regardless of thread). Trace records gain
`"actor": "A#17", "turn": 412, "cause": {"sender": "main#0", "send_span": {...}}`;
`--assert-trace` now checks: effect ∈ row(executing behavior) **and** effect ∈ row(send site's
static row) via the cause chain — the executable form of invariant 35.

### 6.4 Debug race checker

`--debug-rcaps` (debug builds; used by the fuzz harness): every heap object carries an owner tag;
read/write from a non-owner actor without `val` promotion aborts with a compiler-bug-class
report (DL1610). This is belt-and-braces *testing* of the static guarantee, never the guarantee.

### 6.5 WASM engine (v0.7 honesty)

Single-threaded cooperative scheduler: identical semantics (turn atomicity, ordering, quiescence,
traces), no parallelism. Conformance parity holds because the *semantics* never promised timing.
Multi-threaded WASM lands in Stage 10; `delulu authority`/`run` output labels the engine's
concurrency mode.

### 6.6 Failures

A panic in a behavior kills its actor (poisoned; subsequent sends are silently dropped, counted,
reported at exit — `--on-actor-death abort` opts into whole-program abort; default keeps the
system live). Supervision/restart is post-1.0.

---

## 7. Interactions with prior stages (all normative)

- **Plugins:** plugin function values are `val` (immutable handles; grant liveness is re-checked
  per call anyway); calling them inside behaviors is ordinary T-Call. Loading plugins inside
  actors is legal (`Load` in the behavior's row).
- **FFI:** foreign lib handles are `val`; calls are synchronous within a turn (a slow C call
  blocks one scheduler thread — documented; the foreign *worker* (Stage 5) already keeps the
  process safe). `PyObj` is `ref` + pinned (invariant 36): Python effectively lives on the actor
  that created `Cap[Python]` use-sites.
- **Secrets:** `Secret[T]` is `val`; `expose` remains synchronous-class through the broker from
  any actor.
- **Per-actor authority slices** are value-level in v1.0: pass attenuated caps at `spawn` (the
  Constitution's multi-agent story needs nothing more); per-actor *broker nodes* (revocable
  per-actor at the broker) are a post-1.0 RFC — noted so nobody claims it early.

---

## 8. Standard library additions

`std.actors`:

```delulu
actor Promise[T] {                       // T must be sendable (val/iso-carried), checked at use
  new()
  be fulfill(v: val T)                   // first fulfill wins; later ones dropped+counted
  be then(f: val fn(val T) -> Unit ! e)  // f runs as a turn of the Promise actor; row e joins then's send row
}
```

(`then`'s row-variable plumbing is the R-4 law applied to a stdlib actor — the send site of
`then` carries `{Async} ∪ e`.) Nothing else; timers/channels are post-1.0 RFCs.

---

## 9. Acceptance criteria

1. Ping-pong: two actors exchange 1M messages; quiescence exit; deterministic message counts;
   ≥ 2× throughput with 4 threads vs 1 (native engine, smoke-level perf bar only).
2. Sendability: sending a `ref List[Int]` → DL1601 with rcap explanation; `consume`d `iso` list
   crosses; sender's later use → DL1602.
3. `recover`: build a mutable list in a recover block, send as `iso`; capture of an outer `ref`
   inside recover → DL1605.
4. Viewpoint adaptation: write through `box` receiver → DL1604; `tag` field access → DL1604;
   the F-3-style rcap ascription tricks are rejected (invariant unification unchanged).
5. Causal rows: an actor whose behavior is `!{Write}` — the send site inside a `!{Async}` -only
   function is DL0501 (must be `!{Async, Write}`); `delulu why Write` shows the chain
   `main → spawn/send → behavior → console.println` across the actor boundary.
6. Trace assert: full actor conformance suite under `--assert-trace --debug-rcaps`, zero
   violations, on both engines (WASM = cooperative).
7. TSAN (native engine, Linux CI): the actor stress corpus runs clean under ThreadSanitizer.
8. **Rcap property tests:** generated alias/adaptation sequences validated against the deny
   properties (no two actors reach read/write-incompatible aliases; adaptation table cells
   consistent with alias table and deny definitions). Any counterexample blocks the stage.
9. `Promise[T].then` row plumbing works end-to-end (effectful callback surfaces in the caller's
   row; pure callback keeps it `{Async}` only).
10. Migration: a pre-0.7 corpus using `consume` as an identifier gets DL1608 + working
    `delulu fmt --migrate 0.7`; all prior conformance suites green post-migration.
11. `PyObj` send attempt → DL1601 with the pinning explanation.

## 10. Diagnostics (fresh range DL16xx)

| Code | Meaning | Repair |
|---|---|---|
| DL1601 | non-sendable value crossing actor boundary (incl. unconsumed iso; incl. PyObj pinning) | `consume` where applicable — exact; else `requires_human: true` |
| DL1602 | use after consume (incl. field-consume unsupported note) | none |
| DL1603 | alias violates rcap deny property (incl. val closure over ref capture) | none — explanation cites §3 tables |
| DL1604 | access denied by viewpoint/receiver rcap (write via box; field via tag; sync call on tag) | none |
| DL1605 | recover block references non-sendable outer binding | none |
| DL1606 | behavior declares a return type | delete it — exact |
| DL1607 | rcap invalid for this type (e.g., non-tag on actor type) | normalize — exact |
| DL1608 | identifier collides with v0.7 keyword | rename — exact; `fmt --migrate 0.7` |
| DL1610 | debug race-checker violation (compiler-bug class) | none — file a bug |

## 11. Honesty and threat-model caveats (carry into docs verbatim)

- Compile-time data-race freedom covers **DeluluLang code**; foreign code and Contained plugins
  are bounded by their Stage-4/5/6 layers, not by rcaps.
- **Deadlock, livelock, starvation, and mailbox exhaustion are not prevented** — the guarantee is
  race freedom, not liveness. Unbounded mailboxes can exhaust memory; backpressure is post-1.0.
- The rcap tables are adopted from Pony's proven design; our own property-test validation
  (criterion 8) is a ship-gate, and mechanized proof remains Delulu Core future work.
- WASM-engine concurrency is cooperative single-threaded in v0.7 — semantics identical,
  parallelism absent, labeled in output.
- Message *ordering* is per-sender-pair FIFO only; no global order, no delivery-time bounds.

*Stage 7 completes the semantic core: every function typed, every effect rowed, every actor
isolated, every authority sliced. Stages 8–9 make it a product; Stage 10 makes it an industry
tool.*
