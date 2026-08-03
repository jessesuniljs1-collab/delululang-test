# DeluluLang Stage-1 Soundness Audit

**Status:** Normative. Rules R-1 … R-8 below are law; they are patched into
`STAGE1_SPECIFICATION.md` and `CONSTITUTION.md` as of this audit. **Date:** 2026-07-05.

**The question under audit:** can any sequence of higher-order functions, stored function
values, plugin loading (`Plugin[Verified]` and `Plugin[Contained]`), or Result-chaining cause an
effect to be performed at runtime that does **not** appear in the whole-program effect row?

**Answer:** yes — five holes existed. Each is closed below with a named rule. With R-1 … R-8
adopted, the row is a sound over-approximation of runtime effects under the Stage-1 assumptions
(§A). The closing argument is in §C.

---

## A. Audit assumptions (the threat model of this audit)

1. Stage-1 execution: single-threaded, no FFI, no reflection/eval/dynamic import, no
   continuations, panics abort.
2. The compiler, runtime, and primitive table (§7.3 of the Stage-1 spec) are trusted — a wrong
   row in the table is a compiler bug, not a language soundness question (but R-4 makes the
   table's higher-order entries a tested law, because it is the likeliest bug site).
3. **The holder model (normative, folded in from the design commitments):** the *Delulu Authority
   holder* of a grant is the party the broker issued it to — human, LLM orchestrating sub-agents,
   agent, or any future AI, with identical rules for all. Define the attenuation order
   `G₁ ⊑ G₂` iff `effects(G₁) ⊆ effects(G₂)` and every scope of G₁ is narrower-or-equal
   (path is a descendant, hosts ⊆, secret names ⊆, numeric envelopes ≤). Every sub-grant must
   satisfy `sub ⊑ holder's grant` (R-7); no grantee can widen its grant or reach its grantor's.
4. "Effect performed at runtime" means: an invocation of a primitive-table operation (or, at
   Stage 6, a plugin/foreign call) actually executing.

---

## B. Findings

### F-1 · HOLE (serious) — `Plugin[Contained]` per-export rows are unsound

**Escape.** Containment for opaque WASM is module-granular: the WASI import set is derived from
the *grant*, not from which export is called. A manifest may declare `scan : … !{Read}` and
`report : … !{Net}`; if the grant includes both Read and Net (because `report` legitimately needs
Net), then calling `scan` runs code that *can* — and in the exploit, does — perform `Net`.

```delulu
module exfil
fn main(root: Root) ! {Load, Read} {          // no Net anywhere in the program's rows
  let host = root.plugin_host()
  // evil.wasm manifest: effects = ["Read","Net"]
  //   exports: scan : fn() -> Str ! {Read}   (advisory lie)
  let p = load[Contained](host, "evil.wasm", grant_read_and_net)?
  let scan = p.get[fn() -> Str ! {Read}]("scan")
  scan()    // runtime: module reads ./data AND posts it to evil.example.com — Net ∉ any row
}
```

Nothing verifies an opaque module's per-export behavior; only the module boundary is real.

**Closing rule R-1 (Contained rows are module rows).** For `Plugin[Contained]`, the type of
*every* export is assigned the row equal to the plugin's **entire granted authority**
(`effects(grant)`, which is ⊆ manifest effects, enforced at load). Per-export rows in a Contained
manifest are documentation only and never enter the type system. `p.get[F](name)` succeeds only
if `row(F) ⊇ effects(grant)`. (For `Plugin[Verified]`, per-export rows remain fully verified at
load — unchanged.)

**Cost.** Coarser rows for opaque plugins only. This is not a regression; it is the honest price
of containment-only trust, and it makes the Verified/Contained split *visible in the rows*, which
is exactly the constitution's honesty posture.

### F-2 · HOLE — declassification is invisible to the rows

**Escape.** `expose` was typed pure. A function that reveals secrets therefore checks as `!{}`,
appears in `pure_functions` of the authority report, and the whole-program row shows nothing:

```delulu
fn leak_gate(d: Cap[Declassify], s: Secret[Str]) -> Str {   // row !{} — “provably pure”
  s.expose(d)                                               // reveals the secret at runtime
}
```

No I/O happens here, so this is an authority-legibility hole rather than an I/O escape — but
"which functions can reveal secrets" is precisely what a reviewer must be able to read off types.

**Closing rule R-2 (Declassify is an effect).** `Secret.expose` carries a new core effect
`Declassify`: `s.expose(d: Cap[Declassify]) -> T ! {Declassify}`. It enters the primitive table,
the core effect set, and the authority report. `leak_gate` must now declare `! {Declassify}`.

**Cost.** One annotation on every declassifying path. That annotation is the feature.

### F-3 · HOLE (latent, implementation-fatal) — subsumption placement and variance

The Stage-1 text said rows unify by set equality, but also said "no subtyping except row width
subsumption," which invites an implementer to apply subsumption *during unification, in all
positions*. Covariant treatment of a **parameter-position** row admits this:

```delulu
fn twice(cb: fn() -> Unit ! {}) -> Unit ! {} { cb(); cb() }   // honest: pure given pure cb

fn main(root: Root) ! {} {                                    // whole-program row: {} !
  let out = root.console()
  let h: fn(fn() -> Unit ! {Write}) -> Unit ! {} = twice      // admitted iff params checked covariantly (WRONG)
  h(fn() -> Unit ! {Write} { out.println("boo") })            // runtime Write; no row anywhere says Write
}
```

`twice` calls its callback believing it pure; the ascription launders an effectful callback into
it; `Write` executes with `row(main) = {}`. Total soundness failure from one variance mistake.

**Closing rule R-3 (single subsumption site; invariant unification).** In v0.1 there is **no
subsumption inside unification**: rows unify by equality (with row-variable binding). Subsumption
(`ε_body ⊆ ε_declared`) exists at exactly two sites: the T-Fn declaration check and the
row-annotated-lambda check. Ascriptions like the `let h` above therefore fail (`{}` ≠ `{Write}`
in parameter position). Any post-v0.1 subtyping proposal must be variance-correct (contravariant
in parameter rows) and pass this audit's exploit as a rejection test.

**Cost.** Occasional eta-expansion (`fn(x: T) !{Write} { g(x) }`) to move between rows
explicitly. No expressiveness loss in the fragment that matters; large soundness margin.

**Corollary R-3b (no union-merge of row variables).** When one row variable receives conflicting
closed bindings (`e ~ {Read}` then `e ~ {Write}`), the checker must **fail** (new DL0504), never
"helpfully" merge to the union — union-merging is exactly the covariance mistake in disguise when
the variable also occurs in parameter position. Consequence checked in §C: **one row variable per
signature with equality binding admits no unsound merge; its only cost is expressiveness** (you
cannot call `compose(f !{Read}, g !{Write})` without eta-wrapping both to `!{Read, Write}`).

### F-4 · HOLE (one wrong table entry away) — higher-order builtins must carry callback rows

`List.map`, `List.filter`, `Result`/`Option` combinators execute their function argument. If any
primitive-table entry omits the callback's row variable — e.g. `map : fn(List[T], fn(T)->U ! e)
-> List[U] ! {}` instead of `! e` — then:

```delulu
fn main(root: Root) ! {} {                                    // pure!
  let out = root.console()
  [1, 2, 3].map(fn(x: Int) -> Int ! {Write} { out.println(str(x)); x })
}
```

runtime `Write`, static row `{}`. The language rules are fine; the table is the exposed surface.

**Closing rule R-4 (the builtin-callback law).** Every builtin that may invoke a function
argument must include that argument's row variable in its own row. Enforced twice: a **meta-test**
walks the primitive table and asserts every function-typed parameter's row variable occurs in the
entry's row; and each higher-order builtin has a conformance test in the laundering suite pairing
an effectful callback with a pure caller (must be DL0501). Same law applies at Stage 4/6 to any
FFI or plugin surface that accepts DeluluLang function values.

**Cost.** None.

> **R-4 WAS REOPENED AND RE-CLOSED — hardening finding C82…C88, 2026-08-03. Read this before
> relying on anything above.**
>
> Both enforcement clauses above were true and both were satisfied, and the rule was still
> fail-open for four months, because they constrain *the primitive table* and *a corpus of
> laundering programs* — and the defect was in neither. The **enforcement site** read:
>
> ```rust
> if is_higher_order_method(&rt, &name.name) {
>     if let Some((Type::Fn { row, .. }, _)) = arg_tys.first() { acc.add_row(&r); }
> }                                        // ^ no `else`: any other shape drops the row silently
> ```
>
> An argument whose type is a **bare type parameter** is not syntactically `Type::Fn`, so the arm
> did not match and the callback's row was discarded — and a discarded row is an empty row. Nine
> lines were enough:
>
> ```delulu
> fn go[T](xs: List[Int], f: T) -> Int { let ys = xs.map(f)  1 }
> fn main(root: Root) { let out = root.console()
>   let n = go([1], fn(x: Int) -> Int ! {Write} { out.println("EFFECT ESCAPED"); x }) }
> ```
>
> `delulu check` said **checked clean**; `delulu authority` said **"effects: (none — provably
> pure)"** and listed `go` and `main` under **pure fns**; `delulu why Write` said **"program cannot
> perform `Write`"**; and `delulu run --grant console` **printed**. That is verbatim the §D claim
> negated: a runtime effect in no static row on the stack. The same door reopened **R-2**, because
> `Secret.map` had the identical skip branch — a plaintext secret was printed with no `Declassify`
> effect and no `Cap[Declassify]` anywhere, from a function this report called pure.
>
> **Now closed** by making the unknown case *refuse* (DL0401) rather than assume purity, at both
> sites. The reasoning is R-3's own: rows unify by **equality** and there is no subsumption, so a
> row the checker cannot determine is not `{}` — it is unknown, and a builtin that will *invoke*
> the value may not assume the pure case.
>
> **The lesson, recorded because it generalizes:** every clause of this audit constrains a *rule*,
> and a rule can be correct while the *branch that fires when the checker cannot tell* is missing.
> An audit of the rules cannot find that. Only the question "what does this do when it does not
> know?" finds it — the project's skip-branch discipline, which existed and was not applied here.

### F-5 · HOLE — generic builtins launder `Secret` (and would launder `Cap`)

`str(x)` was specified generically. `Secret[T]` forbids coercion to `T`, but stringification *is*
the coercion:

```delulu
fn main(root: Root) ! {Write} {
  let out = root.console()
  let key = root.secret("API_KEY")
  out.println(str(key))        // str : fn(T) -> Str with T := Secret[Str] — laundered, then Net/Write-able
}
```

Structural `==` on secrets is the same class: an equality oracle without `verify`'s constant-time
guarantee (a timing side channel by construction).

**Closing rule R-5 (opaque types).** `Secret[T]`, `Cap[R]`, `Root`, `Plugin[_]` — and any record,
variant, or collection *containing* one (a compiler-computed `opaque` property, propagated
structurally) — are excluded from `str`, `==`/`!=`, and any current or future serialization.
New diagnostics DL0604 (stringify/serialize opaque), DL0605 (equality on opaque). The only
equality on secrets remains `Secret.verify` (constant-time, deliberate single-bit channel, already
documented as a designed leak in Constitution §5.4).

**Cost.** None meaningful; `verify` covers the legitimate case.

> ### R-2 AND R-5 WERE REOPENED — 2026-08-03, campaign finding P17-IF1 — **R-2 NOW RE-CLOSED**
>
> **The sentence above — "deliberate single-bit channel" — is wrong, and the error is not small.**
> The channel is not one bit. It is one bit *of a predicate the attacker chooses*, and it can be
> iterated to recover the entire plaintext.
>
> `Secret.map` hands its closure the **plaintext**, gated only on purity (DL0603). *Purity is not
> confidentiality*: a pure closure computes any predicate over the plaintext and encodes the answer
> into the returned `Secret[Str]`. `Secret.verify` then returns that answer as an **ordinary,
> untainted `Bool`** — the unsealing step — and `check_if` carries no pc-label, so the `Bool` drives
> an observable effect. `k.verify(k.map(fn(x) { g }))` is an equality oracle against **any** `g`.
>
> Observed, not argued: `delulu why Declassify` prints *"program cannot perform `Declassify`"* for a
> program that prints the secret in full. `check` clean, `authority` shows `effects: Write` only, no
> `Cap[Declassify]` anywhere, `--assert-trace` exit 0.
>
> **Why this audit could not have caught it.** R-2 and R-5 are both *satisfied* here. `map` keeps
> its result tainted, exactly as R-5 requires. `verify` is the constant-time comparison R-5 blesses.
> `expose` still carries `Declassify`, exactly as R-2 requires. **Every rule holds and the property
> fails**, because the defect is in the *composition* of two sound operations — and a rule-by-rule
> audit has no place to look for that. This is the same lesson as the R-4 reopening above, in a new
> shape: there, a rule was skipped; here, no rule was skipped and none was enough.
>
> **What survives.** The direct surface is genuinely closed and was re-verified across twelve
> eliminators — DL0602/0604/0605/0203 all fire correctly. R-5's opacity claim holds for every
> *direct* eliminator. What does not survive is the stronger reading, that a secret cannot reach an
> observer without declassification.
>
> **Closing rule R-2b (a secret-derived value carries `Declassify`).** `Secret.verify` now emits
> `Effect::Declassify` in **both** halves of the primitive table — `check.rs::method_sig` and
> `trace::effect_for` — which must agree or `--assert-trace` would report a runtime effect absent
> from the row. This is R-2 applied where it always should have been: `verify` returns a `Bool`
> *derived from secret data*, that is a declassification, so it carries the effect. Both oracles are
> now **DL0501**.
>
> **How the old belief was protected.** `effect_for_is_none_for_pure_operations` asserted
> `effect_for("Secret", "verify") == None` under a comment calling verification pure. The premise
> that made the hole possible was itself a passing test. It has been moved to the effectful list.
>
> **R-2: re-closed. R-5: partially — and the difference matters.** Declassification is visible
> again, which is what R-2 promises. It is *not* impossible: a program declaring `!{Declassify}` may
> still run the oracle, and `delulu authority` will report `effects: Declassify` plus
> `exposure: … declassifiable -> files/console`. That is the honest guarantee.
>
> **RESIDUE, OPEN.** `verify` declassifies without requiring `Cap[Declassify]`, while `expose`
> requires it. Closing that asymmetry means `verify` returning `Secret[Bool]`; the runtime cannot
> represent it today (`SecretVal` is String-only), so it is an RFC, not a patch. Until then: holding
> a secret grants one chosen bit per call without a declassify capability — but never without
> declaring the effect.
>
> Witness: `crates/delulu-check/tests/secret_oracle.rs`. Full write-up:
> `docs/design/PROOF_CAMPAIGN.md` §IF-1.

### F-6 · HOLE (design gap at the Stage-6 boundary) — callbacks into and re-entrancy through plugins

If a host passes a DeluluLang function value into a plugin export: a **Verified** plugin's export
row can and must mention the callback's row variable (same as R-4), and load-time re-checking
makes that real. A **Contained** plugin cannot be bounded per-call: an opaque module holding a
funcref could invoke the callback arbitrarily — repeatedly, re-entrantly, or (if handles persisted)
*after the export returned*, performing the callback's effects at moments no caller row accounts
for.

**Closing rules R-6.**
- **R-6a:** function-typed arguments (and function-typed fields inside arguments) to
  `Plugin[Contained]` exports are a compile error (new DL0803). Verified plugins accept them,
  with rows composed per R-4.
- **R-6b:** host values passed to any plugin live only for the synchronous duration of the export
  call; the runtime table entry is invalidated on return (no persistent handles across calls).
- **R-6c:** function values obtained from a plugin bind the load-time `GrantId`; every call
  re-checks it. Unload revokes the id (existing DL0801, fail-closed); **reload mints a fresh id
  and fresh function values** — a live reference can never be silently re-bound to a plugin with
  different authority. This closes the unload/reload authority-swap.
- **R-7 (monotone attenuation — the holder model, broker law):** any grant issued to a plugin,
  sub-agent, or child context must satisfy `sub ⊑ holder's grant` (§A.3); the broker rejects
  wider requests (new DL0802). Revocation is transitive: revoking a grant revokes every grant
  attenuated from it. This is what makes "the LLM holds authority; its parallel agents cannot
  exceed it" a mechanical property rather than a policy hope, identically for every kind of holder.

**Cost.** R-6a costs real expressiveness (no higher-order calls into opaque plugins) — accepted;
the alternative is unbounded effect timing in unverifiable code. Everything else costs nothing.

---

## C. Channels examined and verified CLOSED (no change needed)

- **Closures capturing capabilities.** Capture is pure; T-Lambda infers the row from the body, so
  a capability-capturing closure carries its effects in its *type*, and every later call site
  unions that row into its context (T-Call). Calling it in a context whose row lacks the effect is
  DL0501 at the *enclosing function*, always.
- **Function values in `var` locals / records / lists, retrieved and invoked.** Types (including
  rows) are fixed at declaration; assignment and element insertion are checked by invariant
  unification (R-3), so a stored `fn !{}` slot can never receive a `!{Write}` value, and retrieval
  yields exactly the stored row. Erasure cannot happen: rows are part of `Type::Fn` everywhere
  (Stage-1 invariant 3).
- **Mutable capture + reassignment tricks** (Y-combinator-style via `var f: fn(…) = …; f = …`):
  the reassignment is checked against `f`'s declared type — row swaps rejected.
- **Result-chaining.** `?` is pure control flow; combinators are higher-order builtins governed by
  R-4; error values carry no authority.
- **`Root` passed around.** Legal; minting caps is pure; *using* them still surfaces effects in
  every enclosing row. Root is opaque (R-5), unforgeable, and only ever enters at `main`.
- **Single row variable (v0.1).** Equality-binding unification (R-3b) means a row variable is a
  name for exactly one row — substitution is sound in all positions regardless of variance,
  because it is equality, not inequality. Expressiveness-limiting, never unsound.
- **Module-level state.** Already banned (DL0305); the ambient-laundering channel does not exist.

## D. The soundness argument for patched Stage 1

**Claim (design-level, to be mechanized as "Delulu Core" in Stage 2):** in a well-typed program
under rules T-* + R-1…R-7, every runtime effect instance is a primitive-table operation whose
effect label is contained in the static row of every function on the dynamic call stack, hence in
`row(main)`.

Sketch, by induction on evaluation:
1. **Preservation of types.** Values flow only through positions checked by invariant unification
   (R-3); no evaluation step changes a value's static type, and rows are never erased
   (invariant 3). So at any call, the callee value's static row is the row the checker saw.
2. **Row containment at calls.** T-Call puts `row(callee)` into the caller's inferred row; T-Fn
   forces inferred ⊆ declared; T-Lambda makes lambda rows exact. Inductively, each frame's static
   row contains the row of every call it can make.
3. **Effects originate only at T-CapOp** (table ops, including `expose` after R-2) — each labeled
   with its effect, which by (2) is in every enclosing row. Plugins: Verified exports are
   re-checked code (same induction); Contained exports have row = full grant (R-1), and the WASI
   floor confines the module to that grant, so even *unverified* behavior stays inside the typed
   row. Callback timing cannot escape rows by R-6.
4. **Capabilities cannot be forged** (no constructor, opaque per R-5, sub-grants monotone per
   R-7), so no effect can be performed without a chain of explicit passes from `main`'s `Root` —
   which is what makes `row(main)` + the grant the whole-program authority.

Assumptions carried (never to be dropped from documentation): compiler/runtime/table correctness,
single-threaded Stage 1, no FFI, sandbox and hardware trust per Constitution §5.14.

**Status of this argument, stated plainly.** It is a *design-level* argument, written by hand, and
it was **wrong in practice from Stage 1 until 2026-08-03** — not because a step of it is invalid,
but because step 2 ("T-Call puts `row(callee)` into the caller's inferred row") assumes the checker
*has* `row(callee)`, and at higher-order builtins it silently did not when the argument's type was
a bare type parameter (see the R-4 box above). The sketch is sound; the implementation had a hole
the sketch could not see, and no amount of re-reading the sketch would have found it.

This is the strongest available argument for why "Delulu Core" — the mechanized proof this document
has promised since Stage 2 — is **not optional and not yet done.** A hand proof of the rules cannot
tell you which branch the implementation forgot to write. Until that mechanization exists, the
honest claim is: *the rules are believed sound, the implementation is tested against them, and the
tests are the actual guarantee.*

## E. New test obligations (added to the laundering suite)

| Test | Expects |
|---|---|
| F-1 program | `p.get[fn() -> Str ! {Read}]` on Read+Net grant → load-time refusal; requires `!{Read, Net}` |
| F-2 program | `expose` without `Declassify` in row → DL0501; authority report lists declassifiers |
| F-3 program | `let h: fn(fn() -> Unit !{Write}) -> Unit !{} = twice` → row mismatch error |
| F-3b | conflicting bindings for one row var → DL0504, never union |
| F-4 meta-test | every higher-order table entry carries its callback row var |
| F-4 program | effectful lambda into `map` inside pure fn → DL0501 |
| F-5 programs | `str(secret)` → DL0604; `secret == secret` → DL0605; record containing Cap under `==` → DL0605 |
| F-6a program | lambda argument to Contained export → DL0803 |
| F-6c program | call through retained reference after unload → DL0801; after reload, old ref still DL0801 |
| R-7 program | plugin requesting grant ⊐ holder's grant → DL0802 |
