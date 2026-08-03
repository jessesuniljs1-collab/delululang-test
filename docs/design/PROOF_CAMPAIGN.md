# The Proof Campaign (P17) — from *tested* to *proven*

**Status: OPEN. Started 2026-08-03.** This document is the ledger for a campaign whose standard is
not "the tests pass" but:

> If this project claims something, that claim must withstand scrutiny from programming language
> researchers, formal methods researchers, compiler engineers, cryptographers, operating systems
> engineers, distributed systems researchers, security auditors and mathematicians.

The predecessor campaign (`HARDENING_CAMPAIGN.md`) attacked the implementation and found real
defects, the worst of them C88 — an escapable effect row. This campaign attacks something different:
**the claims themselves.** A defect is a program that behaves wrongly. A false claim is a sentence
that a reader relies on and that is not true. This project has shipped both, and the second kind is
harder to find because the tests cannot see it.

---

## 1. The organizing principle: every guarantee in exactly one category

Nothing is allowed to sit in a grey area. Every guarantee the project offers must be assigned to
exactly one of:

| # | Category | What it means |
|---|---|---|
| 1 | **Mathematically proven** | A paper proof exists and has been refereed. |
| 2 | **Machine checked** | A proof assistant checks it (Lean/Coq/Agda). |
| 3 | **Model checked** | A model checker explores the state space (TLA+/Alloy). |
| 4 | **Property tested** | Generated inputs, not hand-picked examples. |
| 5 | **Differentially verified** | Two independent implementations agree. |
| 6 | **Fuzz verified** | Adversarial input generation finds nothing. |
| 7 | **Outside the proof boundary** | Explicitly *not* guaranteed, and named as such. |

A claim with no category is a claim to be deleted or demoted until it earns one.

---

## 2. The verification toolchain (provisioned and smoke-tested, 2026-08-03)

Claiming a method is available is itself a claim, so each was executed before being relied on:

| Tool | Status | Evidence |
|---|---|---|
| **Z3** 5.0.0 (pip `z3-solver`) | working | solved a known-`unsat` instance |
| **TLA+ / TLC** v1.7.4 + Java 21 | working | model-checked a 3-state spec, no error |
| **cargo-fuzz** 0.13.2 (WSL nightly) | installed | not yet exercised |
| **cargo-deny** 0.20.2 | installed | not yet exercised |
| **Miri** (nightly component) | installed | not yet exercised |
| **cargo-audit** | **FAILED to build** | superseded — `cargo deny check advisories` reads the same RustSec DB |
| Lean / Coq / Alloy | **absent** | mechanization (category 2) is therefore **not yet available** |

**Honest consequence:** categories 3 and 4 are reachable today. Category 2 is not, until a proof
assistant is installed. `DELULU_CORE.md` §9 has recorded a ~500-line Lean/Coq formalization as open
future work since Stage 2, and it remains open. Nothing in this document upgrades it.

---

## 3. Findings

Each finding was **observed by execution**, not argued. The committed witness is
`crates/delulu-broker/tests/order_laws.rs`, which enumerates *every* subset of a path universe
rather than checking hand-picked pairs. Three of its tests are `#[ignore]`d because they currently
fail: they are committed as evidence of an open defect, and run with
`cargo test -p delulu-broker --test order_laws -- --ignored`.

### P17-F1 — `⊑` is a preorder, not a partial order (206 counterexamples)

`authority.rs` calls itself "the ⊑ attenuation **lattice**". A lattice presupposes a partial order,
which presupposes antisymmetry. Antisymmetry does not hold:

```
A = {"./data"}   B = {"data"}      A ⊑ B  and  B ⊑ A  but  A ≠ B
```

`path::resolve` normalizes before comparing, so `./data`, `data`, `./data/` and `.\data` all resolve
to the same segment list — while `Authority` derives `PartialEq` **structurally**. Mutually
attenuating, structurally unequal.

**Severity: naming/structure, not escalation.** The correct description is a **preorder whose
poset reflection is a meet-semilattice**. Saying "lattice" is imprecise in the file that calls
itself the mathematical heart of custody.

### P17-F2 — `⊓` is not symmetric (414 counterexamples)

`authority.rs:145` states, as a documented fact a reader may rely on:

> *"note `⊓` is symmetric so the value is identical either way"*

It is not.

```
A = {"./data"}   B = {"data"}      A ⊓ B = {"./data"}      B ⊓ A = {"data"}
```

`intersect_path_sets` is `if desc(x,y) { insert x } else if desc(y,x) { insert y }`. When two
spellings denote the same path, **both** branches are true and the first wins — so the surviving
spelling is whichever argument came first. The existing unit test `meet_is_symmetric` passes because
its single hand-picked pair never exercises aliasing.

**Consequence:** the DL0802 repair value depends on argument order.

### P17-F3 — `⊑`-equivalent authorities hash differently (206 counterexamples)

`Authority::to_json` is the canonical form embedded in **hash-chained audit records** and covered by
**certificate signatures**. Two authorities that are mutually `⊑` — the same authority, differently
spelled — serialize to different bytes, and therefore to different hashes.

```
{"./data"} -> ...,"fs.read":["./data"],...
{"data"}   -> ...,"fs.read":["data"],...
```

**Consequence:** two brokers, or one broker at two times, that spell a path differently produce
different audit hashes for the same logical grant. Federation audit reconciliation is the surface
this most endangers, and the project already records reconciliation as fragile.

**Fix is format-affecting.** Canonicalizing the stored spelling would change the bytes of every
existing audit record and signature — the same compatibility constraint already documented for the
omitted `device` key. This therefore belongs in an RFC, not in a hardening patch, and is left
**open** rather than quietly changed.

### P17-F4 — Row unification is order-dependent and has no principal types

Two programs that differ only by **swapping two parameters**:

```delulu
module order_a                                    // REJECTED: DL0504

fn g[e](p: fn() -> Unit ! {Read | e}, q: fn() -> Unit ! e) -> Unit ! {Read | e} {
    p()
    q()
}

fn main(root: Root) ! {Read} {
    let fs = root.fs_read("./")
    g(fn() -> Unit ! {Read} { let _ = fs.read_text("a") },
      fn() -> Unit ! {Read} { let _ = fs.read_text("b") })
}
```

```delulu
module order_b                                    // checks clean — only the parameters moved

fn g[e](q: fn() -> Unit ! e, p: fn() -> Unit ! {Read | e}) -> Unit ! {Read | e} {
    q()
    p()
}

fn main(root: Root) ! {Read} {
    let fs = root.fs_read("./")
    g(fn() -> Unit ! {Read} { let _ = fs.read_text("b") },
      fn() -> Unit ! {Read} { let _ = fs.read_text("a") })
}
```

Reproduce with `delulu check order_a.delulu` and `delulu check order_b.delulu`.

Both are called with two `!{Read}` callbacks. The constraints are `{Read} ∪ e = {Read}` and
`e = {Read}`, which are **simultaneously satisfiable** by `e := {Read}`. The second program is
itself the proof that this is admissible: it binds `e := {Read}` first and then accepts
`{Read | e}` — so the system does treat `{Read | {Read}}` as `{Read}`, idempotent set semantics,
confirmed by its own behaviour rather than assumed. The first program is rejected only because
unification binds `e := {}` greedily from the first parameter and never backtracks.

**This is NOT a criticism of R-3b, which is correct.** `SOUNDNESS_AUDIT.md` §R-3b and
`STAGE1_SPECIFICATION.md:504` deliberately require that conflicting row-variable bindings **fail
with DL0504 and are never union-merged**, because union-merging would silently widen a row. That
rule is right and must stay. The gap is upstream of it: greedy binding at an *ambiguous* constraint
manufactures a spurious conflict, which R-3b then correctly reports. The diagnostic is accurate
about what the solver found; the solver only found it because of parameter order.

**Severity: completeness and predictability, not soundness.** Rejecting a well-typed program is
fail-closed — no authority escapes. But for a language whose stated audience is AI agents generating
code, "reordering two parameters decides whether your program compiles" is a real defect, and the
absence of principal types is currently **undocumented**: a search of every specification and
reference document for principality or inference-completeness returns nothing. Either the solver
should defer ambiguous row bindings until the constraint set is complete, or the specification
should state plainly that inference is order-dependent and incomplete. Silence is the one option
that is not acceptable.

---

## 4. Confirmed NON-findings

Stating what survived attack matters as much as stating what did not.

- **The meet never widens, and it is the *greatest* lower bound.** Verified exhaustively over every
  subset pair of the universe, not spot-checked. This is the law attenuation actually rests on: a
  computed repair can never hand back more authority than either input. **It holds.** F1–F3 are
  about naming, determinism and serialization — **none of them is an authority escalation.**
- **The element relation is reflexive and transitive**, and the set-level order is reflexive and
  transitive. Verified exhaustively; also **proved symbolically in Z3** over an abstract partial
  order, so the result is not an artifact of the chosen universe.
- **Deferred-callback effect attribution (`Promise.then`) — UNPROVEN, not a finding.** A witness was
  built (`pr.then(cb)` armed in a function declaring `{Async, Write}`, triggered from one declaring
  only `{Async}`). It **checks clean**, and `why Write` attributes `main → arm — Write`. That
  attribution is defensible: the closure is created in `arm`, which legitimately holds the effect.
  The program **does not execute** (`DL0907: spawn without an actor system attached`), so no runtime
  violation was observed and none is claimed. Settling this requires an actor-system entry point and
  remains open.

---

## 5. What this campaign has NOT yet done

Named so that no reader mistakes a plan for a result:

- **No mechanized proof exists.** No proof assistant is installed. `DELULU_CORE.md` §7's theorems
  remain paper-level sketches, exactly as §9 says.
- **The broker state machine is not yet model-checked.** TLC is provisioned and proven to run; no
  DeluluLang specification has been written for it.
- **No property-based program generation yet.** The language is still tested with hand-written
  examples plus the conformance corpus; `order_laws.rs` is the first exhaustive-enumeration test in
  the repository.
- **Fuzzing, Miri and sanitizers are provisioned but unexercised.**
- **The other seven audit domains are unstarted** — effect rows/type theory, capability algebra,
  broker state machine, cryptography, information flow, concurrency/distributed, and the theorem
  sketches. Eight specialist agents were dispatched on 2026-08-03 and **all eight were killed by an
  account session limit during their reading phase**, returning no conclusions. Their surviving
  witness programs were recovered from disk and two of them produced F4 and the unproven
  `Promise.then` lead above. This is recorded because the work was commissioned, was not completed,
  and must not be assumed done.

---

## 6. Method note — why exhaustive enumeration replaced spot-checks

`authority.rs`'s own test carried the comment *"Property spot-check"* over a single pair. A
spot-check cannot distinguish "this law holds" from "this law holds for the pair I thought of."
Every law in F1–F3 was already covered by a passing hand-written test; all three failed the moment
the input space contained something a human would not have written — two spellings of the same path.
The lesson generalizes and is the campaign's working rule:

> **Do not write examples. Generate inputs.** A test whose inputs a human chose can only find
> defects that human anticipated.
