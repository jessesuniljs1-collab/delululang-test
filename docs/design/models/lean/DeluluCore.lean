/-
  Delulu Core, the higher-order fragment — MACHINE CHECKED.

  Campaign P17-9. This is deliberately NOT a mechanization of `DELULU_CORE.md` §1–§7 as written.
  Campaign finding **P17-T1** established that doing so would prove the wrong theorem: the document
  contains ZERO occurrences of `higher-order`, `callback`, `invoke` or `map`, its `Σ` gives each
  primitive a single emitted label, and its `E-Op` reduces in one step. There is no construct for a
  primitive that INVOKES a function argument — which is precisely the construct that broke
  (finding C88: a callback reaching a higher-order builtin through a bare type parameter had its
  effect row silently dropped, and a dropped row is an empty row).

  So this file formalises the EXTENSION the calculus needs, and proves the two statements that
  matter about it:

    * `good_sound`  — with the corrected rule, every emitted label is in the declared row.
                      (Effect Soundness for this fragment.)
    * `bad_unsound` — with the rule the calculus actually states, there EXISTS a well-typed program
                      whose trace escapes its row. This is C88, mechanized.

  Both are proved, with no `sorry`. Check it yourself rather than taking this comment's word:

      lean DeluluCore.lean          -- silence means every declaration type-checked

  and the `#print axioms` block at the bottom is the definitive test — a proof that secretly used
  `sorry` would list `sorryAx` there. Verified with Lean 4.32.2.

  Scope, stated so the result is not read wider than it is: this is the higher-order fragment ONLY
  — no capabilities, no store, no secrets, no attenuation, no Progress/Preservation. It settles one
  question, which is the question that cost this project its worst soundness hole.
-/

namespace DeluluCore

/-- Effect labels. Two suffice to state the theorems; nothing depends on the number. -/
inductive Label where
  | write : Label
  | read  : Label
  deriving DecidableEq, Repr

/-- A row is a finite set of labels. Modelled as a list; only membership is ever used, so
    duplication and order are irrelevant — exactly the "rows are sets" of `DELULU_CORE.md` §2. -/
abbrev Row := List Label

/-- Row containment `ρ₁ ⊆ ρ₂`. -/
def subRow (r s : Row) : Prop := ∀ l : Label, l ∈ r → l ∈ s

theorem subRow_refl (r : Row) : subRow r r := fun _ h => h

theorem subRow_trans {r s t : Row} (h₁ : subRow r s) (h₂ : subRow s t) : subRow r t :=
  fun l hl => h₂ l (h₁ l hl)

theorem subRow_nil (r : Row) : subRow [] r := fun _ h => nomatch h

/-- Expressions. Just enough to express a primitive that invokes a function argument. -/
inductive Expr where
  /-- the unit value -/
  | unit : Expr
  /-- a closure carrying its LATENT row in its type — `DELULU_CORE.md` §1: "rows never erase" -/
  | lam  : Row → Expr → Expr
  /-- a primitive operation emitting exactly one label -/
  | op   : Label → Expr
  /-- a HIGHER-ORDER primitive: it invokes its function argument. The construct the calculus omits. -/
  | ho   : Expr → Expr
  deriving Repr

/-- The operational semantics, as a trace relation: `Emits e t` means evaluating `e` emits the
    labels `t`. Mirrors `E-Op`, plus the case the calculus has no rule for. -/
inductive Emits : Expr → Row → Prop where
  | unit : Emits Expr.unit []
  /-- Building a closure is PURE — `T-Abs`: "the row is in the type", not in the construction. -/
  | lam  {r : Row} {b : Expr} : Emits (Expr.lam r b) []
  /-- The only label-emitting rule, as in `E-Op`. -/
  | op   {l : Label} : Emits (Expr.op l) [l]
  /-- **The rule the calculus lacks:** the primitive INVOKES its argument, so the argument's
      labels are emitted. -/
  | ho   {rl : Row} {b : Expr} {t : Row} : Emits b t → Emits (Expr.ho (Expr.lam rl b)) t

/-- **Typing as `DELULU_CORE.md` states it.** `T-Op` computes its row as the union of the op's own
    label and the rows of *evaluating* its arguments. Evaluating a lambda is pure (`T-Abs`), so a
    callback contributes `[]`. The latent row never enters. -/
inductive TypedBad : Expr → Row → Prop where
  | unit : TypedBad Expr.unit []
  | lam  {r : Row} {b : Expr} : TypedBad (Expr.lam r b) []
  | op   {l : Label} : TypedBad (Expr.op l) [l]
  /-- the argument's row *as an expression* — which for a closure is empty -/
  | ho   {f : Expr} {rf : Row} : TypedBad f rf → TypedBad (Expr.ho f) rf

/-- **Typing with the correction.** A primitive that will invoke its argument carries that
    argument's LATENT row. This is audit rule R-4 made a typing rule. -/
inductive TypedGood : Expr → Row → Prop where
  | unit : TypedGood Expr.unit []
  /-- `T-Abs`: the body's row must fit the declared row — the one subsumption site. -/
  | lam  {rl rb : Row} {b : Expr} : TypedGood b rb → subRow rb rl → TypedGood (Expr.lam rl b) []
  | op   {l : Label} : TypedGood (Expr.op l) [l]
  /-- the latent row of the callback surfaces here -/
  | ho   {rl rb : Row} {b : Expr} :
      TypedGood b rb → subRow rb rl → TypedGood (Expr.ho (Expr.lam rl b)) rl

/-! ## Theorem 1 — Effect Soundness for the corrected rule -/

/-- **Every label a well-typed program emits is in its declared row.**
    `labels(tr) ⊆ ρ`, for the higher-order fragment. -/
theorem good_sound : ∀ {e : Expr} {r t : Row}, TypedGood e r → Emits e t → subRow t r := by
  intro e r t ht he
  induction he generalizing r with
  | unit => exact subRow_nil r
  | lam  => exact subRow_nil r
  | op   =>
      cases ht
      exact subRow_refl _
  | ho _ ih =>
      cases ht with
      | ho hbody hsub => exact subRow_trans (ih hbody) hsub

/-! ## Theorem 2 — C88, mechanized -/

/-- The witness: a higher-order primitive applied to a closure that writes.
    Under `TypedBad` this program's row is `[]`; it emits `write`. -/
def c88 : Expr := Expr.ho (Expr.lam [Label.write] (Expr.op Label.write))

theorem c88_typed_bad : TypedBad c88 [] := TypedBad.ho TypedBad.lam

theorem c88_emits : Emits c88 [Label.write] := Emits.ho Emits.op

/-- **The calculus as written admits a program whose trace escapes its row.**
    This is finding C88: `check` clean, `authority` reporting "(none — provably pure)", and a
    `Write` at run time. -/
theorem bad_unsound :
    ∃ (e : Expr) (r t : Row), TypedBad e r ∧ Emits e t ∧ ¬ subRow t r := by
  refine ⟨c88, [], [Label.write], c88_typed_bad, c88_emits, ?_⟩
  intro h
  -- `h` would place `write` in the empty row; membership in `[]` has no constructor.
  have hmem : Label.write ∈ ([] : Row) := h Label.write (List.Mem.head _)
  nomatch hmem

/-- And the corrected rule REFUSES to give that same program an empty row: the only row it admits
    is one containing `write`. Together with `good_sound`, this is the repair. -/
theorem c88_good_row_contains_write :
    ∀ {r : Row}, TypedGood c88 r → Label.write ∈ r := by
  intro r ht
  cases ht with
  | ho hbody hsub =>
      -- casing the body derivation refines the body's row to `[write]`
      cases hbody
      exact hsub Label.write (List.Mem.head _)

/-! ## Axiom audit — the check that cannot be talked around

    A theorem proved with `sorry` still type-checks; what it cannot do is hide from `#print axioms`,
    which reports `sorryAx`. Lean's own three axioms (`propext`, `Classical.choice`, `Quot.sound`)
    are the standard foundation and are expected; `sorryAx` is not. -/

#print axioms good_sound
#print axioms bad_unsound
#print axioms c88_good_row_contains_write

end DeluluCore
