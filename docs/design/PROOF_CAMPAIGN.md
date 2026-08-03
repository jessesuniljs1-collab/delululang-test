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
- ~~The broker state machine is not yet model-checked.~~ **DONE — see §6 below.** The grant tree is
  now model-checked; **leases, redemption, certificate adoption and federation are still not.**
- ~~No property-based program generation yet.~~ **DONE — see §7 below.**
- **Fuzzing, Miri and sanitizers are provisioned but unexercised.**
- **The other seven audit domains are unstarted** — effect rows/type theory, capability algebra,
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

### 🔶 P17-C1 — the audit chain does not detect TRUNCATION (OBSERVED)

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

**Severity is bounded and stated:** the audit directory sits under the operator's own state
directory, and this project already records that it provides no multi-tenancy or same-user
isolation. But a hash chain is sold as *tamper evidence*, and the attack it fails to detect is the
attractive one — you do not modify the record of what you did, you delete it. **The honest claim is
"detects modification and reordering", not "tamper-evident".** Closing it means anchoring the head
outside the log; `AuditBundle::verify` already takes an `expected_start` (`audit.rs:485`), which is
the same idea applied to a bundle's beginning. The live chain's end has no equivalent. Persistence-
format change → RFC, not a patch.

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

### 🔶 P17-B2 — expiry is judged against a WALL clock, so backwards time resurrects authority (OBSERVED)

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

Not claimed: that an attacker can set your clock (usually privileged). The finding is that the
guarantee **rests on clock monotonicity, an assumption the design never states**. Ruling D20 already
moved the simulator's dead-man onto a logical clock for this class of reason; broker expiry did not
get the same treatment.

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

## 13. Method note — why exhaustive enumeration replaced spot-checks

`authority.rs`'s own test carried the comment *"Property spot-check"* over a single pair. A
spot-check cannot distinguish "this law holds" from "this law holds for the pair I thought of."
Every law in F1–F3 was already covered by a passing hand-written test; all three failed the moment
the input space contained something a human would not have written — two spellings of the same path.
The lesson generalizes and is the campaign's working rule:

> **Do not write examples. Generate inputs.** A test whose inputs a human chose can only find
> defects that human anticipated.
