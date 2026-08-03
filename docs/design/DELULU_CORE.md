# Delulu Core — the formal core calculus (v0.2)

**Status:** Normative companion to `STAGE2_SPECIFICATION.md` §7.1. This document states the core
calculus and the theorems that justify the phrase "authority cannot escape the type." **Honesty
clause (binding, per Constitution §9):** the proofs here are **paper-level sketches**, not a
mechanized development. The theorems are stated precisely so they *can* be mechanized; a machine-
checked proof (Coq/Lean/Agda) is committed future work and is **not** claimed to exist yet. The
implementation's soundness is additionally guarded by the laundering suite
(`crates/delulu-check/tests/laundering.rs`), the conformance corpus, and the executable
`trace ⊆ row` witness (`--assert-trace`), which are evidence, not proof.

Delulu Core is intentionally **smaller** than the surface language: it models exactly the
machinery on which the authority guarantee rests, and nothing else. What it omits, it omits on
purpose (§6), and each omission corresponds to a Stage-1 restriction that already exists in the
implementation (no exceptions, no continuations, no mutable module state, no reflection).

---

## 1. Syntax

Types `τ`, rows `ρ`, terms `e`. `R` ranges over resource kinds (`FsRead`, `Console`, …); `ℓ` over
effect labels (`Read`, `Write`, `Net`, `Clock`, `Rand`, `Declassify`).

```
τ  ::=  Unit | Bool | Int | Str
     |  τ →ρ τ                     function carrying its effect row (rows never erase)
     |  Cap R                      an unforgeable capability token for resource kind R
     |  Secret τ                   a secret wrapper
     |  τ × τ | τ + τ              products and sums (records/variants, Result/Option)
     |  α                          type variable

ρ  ::=  { ℓ̄ }                      a closed effect row (a finite set of labels)
     |  { ℓ̄ | ε }                  an open row with tail row-variable ε

e  ::=  x | c | λx:τ. e | e e | let x = e in e
     |  (e, e) | π₁ e | π₂ e | inl e | inr e | case e of inl x ⇒ e | inr y ⇒ e
     |  op_ℓ[R](e, ē)              a primitive operation on a capability (the ONLY effect source)
     |  wrapₛ e | verify(e, e) | expose(e, e)     secret intro/compare/eliminate
     |  κ                          a runtime capability token value (no surface syntax; store-only)
```

Design invariant made syntactic: **a primitive operation `op_ℓ[R]` takes a `Cap R` as its first
argument.** There is no other term that carries an effect label. Capability tokens `κ` have **no
surface syntax** — they exist only in the runtime store and enter a program solely through the
root parameter of `main` (§4). This is the calculus-level statement of "capabilities are
unforgeable."

---

## 2. Effect rows

Rows are finite sets of labels with an optional tail variable. Row equality is set equality on the
closed part plus tail-variable identity — **there is no row subtyping in the calculus** (audit
rule R-3). Union `ρ₁ ⊔ ρ₂` is defined on closed rows as set union; on open rows it is computed by
the unification of §5. Containment `ρ₁ ⊆ ρ₂` is used at exactly one place: the function-body
subsumption check (T-Abs). This mirrors the implementation's `unify.rs` (equality, with binding)
and `check.rs` (subsumption only at declaration sites).

The **primitive signature environment** `Σ` is fixed and finite: it assigns each
`op_ℓ[R]` its argument types, result type, and the single label `ℓ` it emits. `Σ` is the
calculus-level image of the runtime primitive table (`prim.rs`) and the compile-time table
(`check.rs::method_sig`); Theorem 3 (agreement) is what forces those two tables to coincide.

---

## 3. Typing judgment

`Γ ⊢ e : τ ! ρ` — under context `Γ`, term `e` has type `τ` and may perform at most the effects in
`ρ`. Selected rules (the ones that carry the guarantee):

```
(T-Var)   Γ, x:τ ⊢ x : τ ! {}                 (T-Const)  Γ ⊢ c : ty(c) ! {}

(T-Abs)   Γ, x:τ₁ ⊢ e : τ₂ ! ρ_body     ρ_body ⊆ ρ
          ---------------------------------------------      (subsumption site — the ONLY one)
          Γ ⊢ λx:τ₁. e : (τ₁ →ρ τ₂) ! {}      -- building a closure is pure; the row is in the type

(T-App)   Γ ⊢ e₁ : (τ₁ →ρ_f τ₂) ! ρ₁     Γ ⊢ e₂ : τ₁ ! ρ₂
          --------------------------------------------------
          Γ ⊢ e₁ e₂ : τ₂ ! (ρ₁ ⊔ ρ₂ ⊔ ρ_f)   -- the callee's latent row joins here

(T-Op)    Σ(op_ℓ[R]) = (Cap R, τ̄) → τ, emits ℓ     Γ ⊢ e₀ : Cap R ! ρ₀     Γ ⊢ ēᵢ : τᵢ ! ρᵢ
          -----------------------------------------------------------------------------------
          Γ ⊢ op_ℓ[R](e₀, ē) : τ ! ({ℓ} ⊔ ρ₀ ⊔ ⊔ᵢ ρᵢ)      -- the ONLY rule that introduces a label

(T-Sec)   Γ ⊢ e : τ ! ρ                        (T-Verify) Γ ⊢ e₁,e₂ : Secret Str ! ρ₁,ρ₂
          -------------------------                        -----------------------------------
          Γ ⊢ wrapₛ e : Secret τ ! ρ                       Γ ⊢ verify(e₁,e₂) : Bool ! (ρ₁ ⊔ ρ₂)

(T-Expose) Γ ⊢ e : Secret τ ! ρ_e     Γ ⊢ d : Cap Declassify ! ρ_d
          --------------------------------------------------------------
          Γ ⊢ expose(e, d) : τ ! ({Declassify} ⊔ ρ_e ⊔ ρ_d)   -- expose emits Declassify (R-2)
```

Two structural facts follow immediately from the rule shapes and are used throughout:

- **(Fact A — labels come only from T-Op/T-Expose.)** No rule other than `T-Op` and `T-Expose`
  adds a concrete label to a row; every other rule only unions the rows of subterms. Hence a label
  `ℓ` in a derivation's conclusion is traceable to a `T-Op` (or `T-Expose`) node in the tree.
- **(Fact B — a label demands a capability.)** The `T-Op`/`T-Expose` premises require a subterm of
  type `Cap R` (with `R` the resource kind for `ℓ`; `Declassify` for expose). By the shapes of the
  typing rules, a closed term of type `Cap R` reduces (Theorem 1) to a token `κ` for `R`.

**Opacity (R-5), as a syntactic side condition.** The eliminators that would leak a `Secret`/`Cap`
— string conversion, structural equality, serialization — are simply **not in the grammar**. The
only eliminators for `Secret τ` are `verify` (Bool, constant-time by fiat in the operational
semantics) and `expose` (capability-gated, Declassify-emitting). This is why "a secret cannot be
stringified" is not a theorem to prove but an absence to observe.

---

## 4. Operational semantics

A configuration is `⟨σ ; e⟩` where `σ` is a **capability store** mapping tokens `κ` to their
resource kind and scope. Reduction `⟨σ ; e⟩ ⟶ ⟨σ' ; e'⟩ | tr` optionally emits a trace label.
The only label-emitting rule:

```
(E-Op)   κ ∈ dom(σ)     σ(κ) = R     (scope of κ permits the arguments)
         --------------------------------------------------------------
         ⟨σ ; op_ℓ[R](κ, v̄)⟩  ⟶  ⟨σ' ; v⟩   emitting  ℓ
```

`E-Op` **requires `κ ∈ dom(σ)`**: an operation can only fire on a token actually present in the
store. Tokens are never created by reduction except by attenuation (`narrow`, which produces a
token of *no greater* scope) and never forged. The initial configuration is
`⟨σ_root ; main κ_root⟩`, where `σ_root` contains exactly the tokens the human/broker granted and
`κ_root` is the sole root capability — the calculus image of "root enters only at `main`."

---

## 5. Row unification (the R-3 / R-3b fragment)

Unification is by equality with a single mechanism for the tail variable: `{ℓ̄ | ε}` unifies with a
row `ρ` containing at least `ℓ̄` by binding `ε` to the remainder; a tail variable already bound to a
closed row that is then required to equal a *different* closed row is a **conflict**, not a union
(R-3b). The calculus permits at most one tail variable per signature (R-3 / DL0503), which makes
substitution sound in every position **regardless of variance**, because it is equality, not
inequality. This is exactly `unify.rs::unify_row`; the calculus adds nothing the implementation
does not already do.

---

## 6. What Delulu Core deliberately excludes (and why it is sound to)

Each exclusion is a Stage-1/2 language restriction already enforced, so the calculus faithfully
models the implemented language rather than an idealized superset:

- **No exceptions / no first-class continuations.** Non-local control that bypasses a row cannot
  exist if the construct does not exist (Stage-1 §5.8). `panic` is modeled as divergence carrying
  no label and no capability.
- **No mutable module state.** The store `σ` holds only capability tokens; there is no ambient
  mutable cell through which a capability could launder (Stage-1 §5.5, DL0305).
- **No reflection / eval / dynamic import.** `Σ` is fixed and finite; nothing enlarges the set of
  primitive operations at runtime.
- **Plugins and FFI are axioms, not terms.** A `Plugin[Contained]` export is modeled as an opaque
  operation whose row is its entire granted authority (audit R-1); an FFI call as an operation
  emitting `ForeignCall`. The calculus does **not** prove anything about the *internals* of such
  code — it proves the boundary types are respected, which is exactly the honesty boundary the
  Constitution draws (§5.12, §6). Stated here so no reader mistakes the theorems for a claim about
  foreign or opaque code.

---

## 7. Theorems

Let `labels(tr)` be the multiset of labels emitted along a reduction sequence, and `row(e)` the
row in `e`'s typing.

> **Theorem 1 (Progress).** If `⟨σ ; e⟩` is well-typed (`∅ ⊢ e : τ ! ρ`, `σ` supplies every free
> token in `e` at the required kind) then either `e` is a value or `⟨σ ; e⟩ ⟶ ⟨σ' ; e'⟩`.
>
> *Sketch.* Standard structural induction on the typing derivation. The only non-standard case is
> `T-Op`: by Fact B the capability subterm reduces to a token `κ` of kind `R`, and well-formedness
> of `σ` (it supplies `e`'s free tokens) gives `κ ∈ dom(σ)`, so `E-Op` applies. No stuck state
> arises from a missing capability, because a missing capability makes the term ill-typed, not
> stuck.

> **Theorem 2 (Preservation, types and rows).** If `∅ ⊢ e : τ ! ρ` and `⟨σ ; e⟩ ⟶ ⟨σ' ; e'⟩ | tr`
> then `∅ ⊢ e' : τ ! ρ'` with `ρ' ⊔ labels(tr) ⊆ ρ`.
>
> *Sketch.* Induction on the reduction. The row component is the point of interest: every
> congruence step preserves or shrinks the row (subterm rows only union upward under T-App/T-Op,
> so a reduced subterm's row is `⊆` the original), and `E-Op` emits exactly the `ℓ` that `T-Op`
> already accounted for in `ρ`. Substitution (from β-reduction / `let`) preserves rows because
> lambda bodies carry their row in the arrow type (rows never erase). The single subsumption site
> (T-Abs) is where `ρ_body ⊆ ρ` is discharged; no other step introduces a `⊆`.

> **Theorem 3 (Effect soundness — "authority cannot escape the type").** If `∅ ⊢ e : τ ! ρ` and
> `⟨σ_root ; e⟩ ⟶* ⟨σ' ; v⟩` emitting trace `tr`, then `labels(tr) ⊆ ρ`. In particular, every
> emitted label is witnessed by a capability token that descends, by explicit passing or
> attenuation, from `σ_root`.
>
> *Sketch.* Immediate from Theorem 2 by induction on the reduction length: each step's emitted
> labels are absorbed into the standing row bound, and the row never grows. The capability-descent
> clause is by Fact B plus the store discipline of §4 (tokens are granted at the root or attenuated
> from existing tokens, never forged). This is the theorem the runtime `--assert-trace` flag checks
> *dynamically* on every traced run: `assert_trace(row(main), trace)` must be empty, and a
> non-empty result is DL1101, a compiler-bug-class failure.

> **Corollary (No-forgery).** In any reachable `⟨σ' ; e'⟩` from `⟨σ_root ; main κ_root⟩`, every
> token in `e'` has a scope `⊑` some token in `σ_root` (attenuation is monotone — audit R-7). Hence
> the whole-program authority is bounded by `row(main)` for kinds and by `σ_root` for scopes —
> exactly what `delulu authority` reports.

---

## 8. Relationship to the implementation (traceability)

| Calculus object | Implementation |
|---|---|
| `Σ` (primitive signatures) | `check.rs::method_sig` (compile-time) ⟷ `prim.rs` (runtime); Theorem 3 forces them to agree |
| Row equality + single tail var | `unify.rs::unify_row` (R-3, R-3b → DL0504) |
| Subsumption only at T-Abs | `check.rs` T-Fn/annotated-lambda boundary check (DL0501/DL0502) |
| `expose` emits `Declassify` | `check.rs` Secret.expose row (R-2) + `prim.rs` |
| Opacity as absent eliminators | `check.rs::is_opaque` → DL0604/DL0605 (R-5) |
| Store discipline / no forgery | `value.rs` `CapVal` has no constructor from data; broker-only minting |
| Theorem 3, dynamically | `trace.rs::assert_trace` + `--assert-trace` (DL1101) |

Every audit exploit (`SOUNDNESS_AUDIT.md` F-2…F-5) corresponds to a would-be violation of Fact A,
Fact B, or the opacity side condition, and appears as a rejection test in the laundering suite.
`F-1`/`F-6`/`R-7` concern plugins and are modeled here only as the boundary axioms of §6; their
runtime enforcement lands in Stage 6.

---

## 9. Mechanization — stated as future work, not as a present claim

> ### 🔴 BEFORE MECHANIZING, READ THIS — campaign finding P17-T1 (2026-08-04)
>
> **Mechanizing §1–§7 as written would NOT have caught C88**, the worst soundness hole this project
> has had. Search this document for `higher-order`, `callback`, `invoke` or `map`: there are **zero
> occurrences**. `Σ` gives each `op_ℓ[R]` argument types and a *single* emitted label; `E-Op` (§4)
> reduces in one step emitting exactly that label. **The calculus has no construct for a primitive
> that invokes a function argument.**
>
> `T-Op` computes its row as `{ℓ} ⊔ ρ₀ ⊔ ⊔ᵢ ρᵢ` — the op's label plus the rows of *evaluating* its
> arguments. For a lambda that is `{}`, because T-Abs makes closure construction pure. **A
> callback's latent row never enters the rule.** So Theorem 3 is provable and true *of this
> calculus*, while the implementation was unsound: the calculus is silent about the construct that
> failed. `SOUNDNESS_AUDIT.md` F-4/R-4 knows about higher-order builtins; §1–§7 does not.
>
> **A mechanization must therefore first EXTEND the calculus** with a higher-order primitive form —
> an `op` whose argument is a function it invokes, whose typing rule unions that function's latent
> row, and whose `E-Op` emits the callback's labels — or it will buy confidence in a model that
> excludes the only soundness hole this project has ever had.
>
> ### 🔶 Theorem 1 (Progress) is FALSE as stated — campaign finding P17-T2
>
> `E-Op` carries `(scope of κ permits the arguments)` as a **premise**. A capability that is present
> and well-typed but whose *scope* does not cover the argument makes `E-Op` inapplicable, and no
> other rule applies — so a well-typed closed term is **stuck**, which §7 Theorem 1 forbids.
> Observed: a program granted `fs.read=./data` reading `../outside.txt` checks clean and faults at
> run time with `DL0904`. This document contains no `fault` configuration at all.
>
> The sketch's justification covers a *missing* capability ("a missing capability makes the term
> ill-typed, not stuck") — not a *present* one with insufficient scope, which is the case the
> runtime actually raises. Types do not track scopes; scopes are runtime values. **The correct
> statement is progress-or-fault**, and the fault configuration must exist before Preservation can
> be stated over it.
>
> Both findings are recorded in `docs/design/PROOF_CAMPAIGN.md` §10.

A machine-checked development (the natural target is a ~500-line Lean or Coq formalization of §1–§7)
is **open, invited work** recorded in `CONTRIBUTING.md`. Until it exists, this document and the
test suites are the project's soundness evidence, and the Constitution's honesty clause forbids
describing the guarantee as "proven" in any stronger sense. What *is* true today: the theorems are
stated precisely enough to mechanize, the implementation is structured to match them
one-to-one (§8), and the dynamic witness (`--assert-trace`) has never reported a violation across
the conformance and fuzz corpora.
