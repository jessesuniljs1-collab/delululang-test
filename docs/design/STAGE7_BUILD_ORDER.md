# Stage 7 Build Order — "Concurrent" (operational companion)

**Status:** OPEN — head chef cooking directly this stage (Opus 4.8 sous-chefs drafted per-dish
as needed). Opened 2026-07-18.
**Normative:** `STAGE7_SPECIFICATION.md` (the *what*). **Binding how-to:**
`docs/playbooks/STAGE7_PLAYBOOK.md` (phases 7a–7j, traps 1–9). **Precedence:** spec > playbook >
this document. This document records *operational* rulings: gate order, house rules, deviations,
and the close-out evidence table.

---

## 1. Gates

Phases 7a–7j from the playbook, grouped into commit gates. Every phase commits at green
(build + full workspace suite); a gate closes only when its named witnesses are seen green
by the head chef.

| Gate | Phases | What must be true at close |
|---|---|---|
| G1 | 7a, 7b | Grammar/AST parses round-trip; DL1608 + `fmt --migrate 0.7` works; alias/sendable/default-rcap tables cell-for-cell green |
| G2 | 7c, 7d | Viewpoint matrix + write rule on ordinary records; consume flow analysis (DL1602); recover env restriction (DL1605) |
| G3 | 7e, 7f | Actor decls checked (DL1601/1604/1606/1607); `Async` active; T-Spawn/T-Send; DL0501 across the actor boundary; `delulu why` chain |
| G4 | 7g | Native scheduler: ping-pong witness (criterion 1 — see deviation 2), quiescence, per-sender FIFO, iso move + refcount==1 debug check |
| G5 | 7h, 7i | WASM cooperative parity (criterion 6); `--debug-rcaps`; causal trace + `--assert-trace`; TSAN lane (criterion 7, Linux) |
| G6 | 7j + close-out | Rcap property-test ship-gate (criterion 8 — a counterexample BLOCKS the stage); `std.actors.Promise[T]` (criterion 9); failures/poisoning; all 11 criteria witnessed; §11 caveats verbatim in docs |

## 2. House rules (carried from Stage 6 + Stage-7 specifics)

1. **Never push to GitHub.** Commit locally, often.
2. **Zero pre-existing tests edited** — except where a phase *intends* a surface change
   (there should be none: Stage 7 layers on; trap 7). Any pre-existing test that breaks is a
   stop-and-examine event, not an edit-the-test event.
3. **Do not change plugins or the broker** (playbook trap 7). Actors *use* Stage-5/6 machinery.
   If a Stage-7 change seems to require editing `delulu-broker`, Stage-5 broker code, or Stage-6
   plugin semantics, stop and re-derive.
4. **Machine channels are never styled**; `--json` output for non-actor programs stays
   byte-identical to v0.6.
5. **New dependencies:** none expected. One pre-authorized exception: `crossbeam-deque` for the
   7g scheduler *if* a hand-rolled shared-runqueue pool cannot meet criterion 1 cleanly (see
   deviation 3 — the decision is recorded when 7g lands, either way).
6. **Refusal honesty:** anything v0.7 does not enforce says so (WASM = cooperative single-thread,
   labeled in output; liveness never claimed — spec §11 verbatim into docs).
7. **THE KITCHEN RULE** (standing, from Stage 6): for every rule, write the "what if the checker
   couldn't tell" case as a named test *before* claiming the rule holds. Security rules die in
   the skip branch. Stage-7 skip branches to hunt: rcap-of-inferred-type undetermined at a send
   site; consume through aliasing the flow analysis didn't see; recover capturing through a
   closure; viewpoint adaptation on a `Var` not yet resolved; sendability of a row-polymorphic
   closure; default-rcap rule on a type family the rule's match didn't list.
8. **Rows and rcaps never bleed** (playbook trap 1): sendability is aliasing, rows are effects;
   both checked at send sites, separately; a `val` closure may be `!{Write}`.
9. **Pony's tables are ground truth** (trap 2): faithful implementation + property validation,
   no redesign. Criterion 8 arbitrates; a counterexample means *our code* is wrong.
10. **DL1610 is a compiler-bug signal**, never "we catch races at runtime" (trap 3). The
    guarantee is static.

## 3. Deviations ledger

Numbered, argued, ruled. A deviation ships only with its ruling recorded here.

1. **AST shape for rcaps** — spec §2.1 sketches `TypeExprR { rcap, core }` as a wrapper struct.
   Ruling: the *mechanism* (an optional rcap prefix at type positions, plain Stage-1 types
   inside) is normative; the *struct shape* is notation. Implemented as whatever shape touches
   the fewest existing sites while representing every position the spec itself uses (incl.
   `val fn(val T) -> …` — §8 puts an rcap on a fn-type parameter, so fn-type params must carry
   one). **Finalized in 7a:** a `TypeExpr::Rcap { rcap, inner }` wrapper *variant* (not a
   wrapper struct) — every type position gets rcap capability with zero churn on existing
   structures, and the compiler's exhaustiveness check found every consumer that needed an
   arm. Actor members parse into `ActorDecl { fields: Vec<ActorField>, ctor: CtorDecl,
   behaviors, fns }` (the sketch's `FieldDecl` is `ActorField` to avoid colliding with the
   record-field `FieldDef`). Two placement notes, ruled: DL1606 (behavior return type) and the
   DL1602 field-consume variant are emitted by the **parser** — the spec constrains the codes
   and repairs, not the emitting stage, and the parser owns the best spans. `recover` with a
   lift target outside {iso, val} is DL1607 at parse.
2. **Criterion-1 witness reading** — a single strictly-alternating ping-pong pair is a serial
   dependency chain; no scheduler on earth gets 2× from adding threads to it. Ruling: the
   witness runs N independent pairs (N ≥ threads×2) totalling 1M+ messages; deterministic
   per-pair counts; quiescence exit; ≥2× wall-clock at 4 threads vs 1 over the whole set. This
   reads the criterion's intent (scheduler actually parallelizes independent work) instead of
   its letter (physically impossible).
3. **Scheduler topology** — spec §6.1 says "work-stealing thread pool". True per-worker deques
   with stealing either means `crossbeam-deque` (new dep, pre-authorized in house rule 5) or
   hand-rolled Chase-Lev (rejected: subtle unsafe code is exactly what we don't hand-roll).
   Ruling deferred to 7g: implement the simplest topology that meets criterion 1 and TSAN;
   record the choice and, if it is a shared-runqueue pool rather than stealing deques, say so
   here honestly (semantics identical; stealing is a throughput optimization).
4. **Rcap tracking representation** — `Type` is serialized in Stage-6 DIR; embedding rcaps in
   `Type` forces a DIR version bump for a feature Verified plugins do not use (plugin exports
   are `val` by §7). Ruling: the rcap axis lives *beside* `Type` (binding entries, fn
   signatures, field metadata, expression side-tables), touching `unify.rs` not at all — the
   literal embodiment of "second axis, orthogonal". If the deny properties force rcaps into
   `Type` (e.g. fn-type param rcaps must survive unification), the DIR version bumps honestly
   (DL1503 "rebuild the plugin") and this ruling is amended, not silently violated.
5. **Keyword mechanics** — `consume`/`recover` become true tokens (invariant 37: DL1608 + exact
   rename + `fmt --migrate 0.7`). `actor`/`spawn` become true tokens with member-position
   preserved via the existing `keyword_lexeme` mechanism (so `x.spawn` field access stays legal,
   same as `py.import`). The six rcaps stay lexed as identifiers and are recognized
   *contextually in type position only* — they are RESERVED, so no user type can collide, and
   no expression-position breakage is possible. Activation ≠ tokenization; the reserved list
   already guarantees declaration sites are clean.
6. **`delulu fmt` scope** — no `fmt` subcommand exists pre-0.7. Invariant 37 requires only the
   migration form. Ruling: v0.7 ships `delulu fmt --migrate 0.7 <path>…` (token-stream rename of
   `consume`/`recover` identifiers to `consume_`/`recover_`, byte-preserving otherwise); a
   general formatter is NOT implied and NOT shipped (Stage 8 surface work may add one).
7. **`iso` send mechanics** — spec §6.2 says moves are "a pointer handoff". `Value` is `Rc`-based
   and not `Send`; the v0.7 boundary converts the (statically unique) graph into the receiving
   actor's heap by rebuild, with the debug lane asserting `Rc::strong_count == 1` across the
   moved graph (exactly the §7.4 check). Sender's binding is dead (consume), so rebuild vs
   handoff is observationally identical; handoff is a Stage-10 optimization note, recorded here.
8. **Sums in the default-rcap val set** — spec §2's list says "records of only such types"
   and does not name user sums. Ruled: sums of only-val components default `val`, mirroring
   records, by MECHANISM: `Value::Variant` holds its fields in an immutable `Rc<Vec<_>>` — a
   sum has no write surface at all in this runtime, so a sum of val components is deeply
   immutable in exactly the way a val record is. Defaulting it `ref` would be pure friction
   with no guarantee gained. (7b; witnessed in `rcaps::tests`.)

## 4. Close-out table (criteria → witnesses)

Filled in as phases land; the stage is BUILT only when every row names a green witness the
head chef has personally seen.

| # | Criterion (spec §9) | Witness | Seen |
|---|---|---|---|
| 1 | ping-pong 1M msgs, quiescence, deterministic, ≥2× @4 threads | — | — |
| 2 | `ref List[Int]` send → DL1601; consumed iso crosses; later use → DL1602 | — | — |
| 3 | recover mutable-build → iso send; outer `ref` capture → DL1605 | — | — |
| 4 | write via `box` → DL1604; `tag` field access → DL1604; F-3-style ascription rejected | — | — |
| 5 | causal rows: DL0501 at send site; `delulu why Write` crosses the actor boundary | — | — |
| 6 | conformance under `--assert-trace --debug-rcaps`, both engines, zero violations | — | — |
| 7 | TSAN clean on the actor stress corpus (native, Linux) | — | — |
| 8 | rcap property tests vs deny properties — **ship-gate** | — | — |
| 9 | `Promise[T].then` row plumbing end-to-end | — | — |
| 10 | DL1608 + `fmt --migrate 0.7`; prior suites green post-migration | — | — |
| 11 | `PyObj` send → DL1601 with pinning explanation | — | — |

## 5. Post-v0.7 RFC ledger (deferred with eyes open)

1. Mailbox backpressure (unbounded in v0.7; can OOM — §11 caveat, Stage 10).
2. Actor supervision/restart trees (v0.7: poison + count + report; `--on-actor-death abort`).
3. `await`/suspension inside behaviors — **rejected**, not deferred (Constitution §5.10 one-system
   rule); `await` stays reserved.
4. Distributed actors.
5. Per-actor broker nodes (v1.0 story is value-level attenuation at `spawn`; spec §7).
6. Multi-threaded WASM (Stage 10) + true stealing scheduler if deviation 3 lands shared-runqueue.
7. Field-consume (v0.7: locals/params only; DL1602 variant message).
8. Pointer-handoff iso sends (deviation 7).
