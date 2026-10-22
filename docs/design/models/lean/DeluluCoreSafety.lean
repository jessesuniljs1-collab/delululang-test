/-
  Delulu Core v0.4 — Progress-or-fault, Preservation, Effect Soundness and No-forgery, MACHINE CHECKED.

  RW 5.1. `DELULU_CORE.md` §1–§4 and the three theorems of §7, for the calculus's monomorphic, closed-row
  fragment: functions carrying their effect rows, `let`, products, sums, capability tokens in a store, primitive
  operations on a capability (`E-Op` and `E-Refuse`, the fault), a higher-order primitive that invokes its function
  argument (`T-HOp`/`E-HOp`), and secrets (`wrap`, `verify`, `expose`). What is NOT here, so the result is not read
  wider than it is: type variables and open rows (§5's unification), and the internals of `Σ`. Each primitive's result
  on a permitted argument is assumed well typed with an empty row (`SigOk`), which is what §2 asks of `Σ`: the image of
  the runtime's primitive table and the checker's, made to agree. Attenuation IS here: `narrow` mints a token at the end
  of the store, of its parent's kind and under its parent's scope, or faults, and `attenuation_monotone` proves the
  No-forgery corollary's scope clause (audit R-7) over an abstract scope order with the one law attenuation needs.
  `E-HOp` is modelled by its simplest instance, a primitive whose unfolding applies the callback once; the higher-order
  file beside this one, `DeluluCore.lean`, treats the construct on its own.

  `verify`'s label is a parameter (`Sig.verifyLabel`): none, as v0.3's `T-Verify` wrote it, or `Declassify`, as v0.4
  writes it and the implementation has typed and traced it since campaign finding P17-IF1. Every theorem holds for either.

  Check it yourself rather than taking this comment's word:

      lean DeluluCoreSafety.lean     -- silence before the axioms block means every declaration type-checked

  Lean reports a `sorry` only as a warning ("declaration uses `sorry`") and still exits 0, so CI's `formal` job fails on
  any `sorry` in its output. The `#print axioms` block at the bottom lists what each theorem rests on: Lean's standard
  `propext`, `Classical.choice` and `Quot.sound`, and no `sorryAx`. (`DeluluCore.lean` rests on none at all.) And the
  proofs have teeth: with `T-Op`'s label or `T-HOp`'s latent row taken out of the rules, or the law that a narrowed
  scope is under its parent's weakened to `le s s`, this file fails to check. CI requires each of the three to fail.
-/

namespace DeluluSafety

/-- Effect labels (`Read`, `Write`, `Net`, …) and resource kinds (`FsRead`, `Console`, …, `Declassify`). Only their
    equality is ever used, so naturals stand for both. -/
abbrev Label := Nat
abbrev Kind := Nat

/-- A closed effect row: a finite set of labels, as a list; only membership is used (§2). -/
abbrev Row := List Label

/-- Row containment `ρ₁ ⊆ ρ₂`. -/
def Row.sub (a b : Row) : Prop := ∀ x, x ∈ a → x ∈ b

/-- Types (§1), without type variables. -/
inductive Ty where
  | unit
  | bool
  | int
  | str
  | arr    : Ty → Row → Ty → Ty   -- τ →ρ τ: a function carrying its effect row
  | cap    : Kind → Ty             -- Cap R
  | secret : Ty → Ty               -- Secret τ
  | prod   : Ty → Ty → Ty
  | sum    : Ty → Ty → Ty
  deriving DecidableEq, Repr

/-- Terms (§1), with de Bruijn variables: `var 0` is the nearest binder. -/
inductive Tm where
  | var    : Nat → Tm
  | unit   : Tm
  | bool   : Bool → Tm
  | int    : Int → Tm
  | str    : String → Tm
  | lam    : Ty → Row → Tm → Tm      -- λx:τ. e, with the latent row its arrow type carries
  | app    : Tm → Tm → Tm
  | letin  : Tm → Tm → Tm            -- let x = e₁ in e₂
  | pair   : Tm → Tm → Tm
  | fst    : Tm → Tm
  | snd    : Tm → Tm
  | inl    : Tm → Tm
  | inr    : Tm → Tm
  | case   : Tm → Tm → Tm → Tm       -- case e of inl x ⇒ e₁ | inr y ⇒ e₂
  | op     : Nat → Tm → Tm → Tm      -- op_ℓ[R](e₀, e₁): the capability, then the arguments (one; a tuple for ē)
  | hop    : Tm → Tm → Tm            -- hop(e₀, e₁): a primitive that invokes its function argument
  | wrap   : Tm → Tm                 -- wrapₛ e
  | verify : Tm → Tm → Tm
  | expose : Tm → Tm → Tm            -- expose(e, d)
  | narrow : Tm → Tm → Tm            -- c.narrow(e): a capability for a part of c's scope (attenuation)
  | tok    : Nat → Tm                -- κ: a capability token; no surface syntax, it lives in the store
  deriving DecidableEq, Repr

/-- The primitive signature environment `Σ` (§2), fixed and finite, with what the operational semantics needs of it:
    each operation's resource kind, label, argument and result types, the scope check `permits` (decidable), and its
    result `δ` on a permitted argument. `verifyLabel` is the label `verify` emits: `none` as §3's `T-Verify` writes it,
    `some declassifyLabel` as the implementation types and traces it since campaign finding P17-IF1. Every theorem
    below holds for either. -/
structure Sig where
  kind            : Nat → Kind
  label           : Nat → Label
  argTy           : Nat → Ty
  resTy           : Nat → Ty
  permits         : Nat → Tm → Bool      -- a token's scope, then the argument
  δ               : Nat → Tm → Tm
  declassifyKind  : Kind
  declassifyLabel : Label
  verifyLabel     : Option Label
  /-- Attenuation (`narrow`): the argument it takes, whether a scope admits it (a spelling that escapes it is refused,
      `DL0904`), and the narrowed scope; and the scope order `⊑`, a preorder, with the one law attenuation needs: a
      narrowed scope is under its parent's. -/
  narrowArgTy     : Ty
  narrowOk        : Nat → Tm → Bool
  narrowScope     : Nat → Tm → Nat
  le              : Nat → Nat → Prop
  le_refl         : ∀ s, le s s
  le_trans        : ∀ a b c, le a b → le b c → le a c
  narrow_le       : ∀ s v, le (narrowScope s v) s

/-- The capability store `σ` (§4): each token's resource kind and scope; a token is its index. Attenuation adds a token
    at the end, so a fresh one always exists and an old one never moves. -/
abbrev Store := List (Kind × Nat)

/-- Values. -/
inductive Value : Tm → Prop where
  | unit : Value .unit
  | bool (b : Bool) : Value (.bool b)
  | int (n : Int) : Value (.int n)
  | str (s : String) : Value (.str s)
  | lam (τ : Ty) (ρ : Row) (e : Tm) : Value (.lam τ ρ e)
  | pair {v₁ v₂ : Tm} : Value v₁ → Value v₂ → Value (.pair v₁ v₂)
  | inl {v : Tm} : Value v → Value (.inl v)
  | inr {v : Tm} : Value v → Value (.inr v)
  | wrap {v : Tm} : Value v → Value (.wrap v)
  | tok (κ : Nat) : Value (.tok κ)

/-- The typing judgment `Γ ⊢ e : τ ! ρ` (§3), under a store that types the tokens. Rows are joined by `++`; the one
    subsumption site is `T-Abs`. -/
inductive HasType (S : Sig) (σ : Store) : List Ty → Tm → Ty → Row → Prop where
  | var {Γ n τ} : Γ[n]? = some τ → HasType S σ Γ (.var n) τ []
  | unit {Γ} : HasType S σ Γ .unit .unit []
  | bool {Γ b} : HasType S σ Γ (.bool b) .bool []
  | int {Γ n} : HasType S σ Γ (.int n) .int []
  | str {Γ s} : HasType S σ Γ (.str s) .str []
  | lam {Γ τ₁ τ₂ ρb ρ e} :
      HasType S σ (τ₁ :: Γ) e τ₂ ρb → Row.sub ρb ρ → HasType S σ Γ (.lam τ₁ ρ e) (.arr τ₁ ρ τ₂) []
  | app {Γ e₁ e₂ τ₁ τ₂ ρf ρ₁ ρ₂} :
      HasType S σ Γ e₁ (.arr τ₁ ρf τ₂) ρ₁ → HasType S σ Γ e₂ τ₁ ρ₂ → HasType S σ Γ (.app e₁ e₂) τ₂ (ρ₁ ++ ρ₂ ++ ρf)
  | letin {Γ e₁ e₂ τ₁ τ₂ ρ₁ ρ₂} :
      HasType S σ Γ e₁ τ₁ ρ₁ → HasType S σ (τ₁ :: Γ) e₂ τ₂ ρ₂ → HasType S σ Γ (.letin e₁ e₂) τ₂ (ρ₁ ++ ρ₂)
  | pair {Γ e₁ e₂ τ₁ τ₂ ρ₁ ρ₂} :
      HasType S σ Γ e₁ τ₁ ρ₁ → HasType S σ Γ e₂ τ₂ ρ₂ → HasType S σ Γ (.pair e₁ e₂) (.prod τ₁ τ₂) (ρ₁ ++ ρ₂)
  | fst {Γ e τ₁ τ₂ ρ} : HasType S σ Γ e (.prod τ₁ τ₂) ρ → HasType S σ Γ (.fst e) τ₁ ρ
  | snd {Γ e τ₁ τ₂ ρ} : HasType S σ Γ e (.prod τ₁ τ₂) ρ → HasType S σ Γ (.snd e) τ₂ ρ
  | inl {Γ e τ₁ τ₂ ρ} : HasType S σ Γ e τ₁ ρ → HasType S σ Γ (.inl e) (.sum τ₁ τ₂) ρ
  | inr {Γ e τ₁ τ₂ ρ} : HasType S σ Γ e τ₂ ρ → HasType S σ Γ (.inr e) (.sum τ₁ τ₂) ρ
  | case {Γ e e₁ e₂ τ₁ τ₂ τ ρ ρ₁ ρ₂} :
      HasType S σ Γ e (.sum τ₁ τ₂) ρ → HasType S σ (τ₁ :: Γ) e₁ τ ρ₁ → HasType S σ (τ₂ :: Γ) e₂ τ ρ₂ →
      HasType S σ Γ (.case e e₁ e₂) τ (ρ ++ ρ₁ ++ ρ₂)
  /-- `T-Op`: the ONLY rule that introduces an operation's label, and it demands a `Cap R` (Fact B). -/
  | op {Γ o e₀ e₁ ρ₀ ρ₁} :
      HasType S σ Γ e₀ (.cap (S.kind o)) ρ₀ → HasType S σ Γ e₁ (S.argTy o) ρ₁ →
      HasType S σ Γ (.op o e₀ e₁) (S.resTy o) (S.label o :: (ρ₀ ++ ρ₁))
  /-- `T-HOp` (v0.3): the callback's LATENT row joins here — the rule whose absence was C88. -/
  | hop {Γ e₀ e₁ τ₁ τ₂ ρf ρ₀ ρ₁} :
      HasType S σ Γ e₀ (.arr τ₁ ρf τ₂) ρ₀ → HasType S σ Γ e₁ τ₁ ρ₁ → HasType S σ Γ (.hop e₀ e₁) τ₂ (ρf ++ ρ₀ ++ ρ₁)
  | wrap {Γ e τ ρ} : HasType S σ Γ e τ ρ → HasType S σ Γ (.wrap e) (.secret τ) ρ
  | verify {Γ e₁ e₂ ρ₁ ρ₂} :
      HasType S σ Γ e₁ (.secret .str) ρ₁ → HasType S σ Γ e₂ (.secret .str) ρ₂ →
      HasType S σ Γ (.verify e₁ e₂) .bool (S.verifyLabel.toList ++ ρ₁ ++ ρ₂)
  /-- `T-Expose`: the only eliminator that yields a secret's contents; capability-gated, it emits `Declassify` (R-2). -/
  | expose {Γ e d τ ρe ρd} :
      HasType S σ Γ e (.secret τ) ρe → HasType S σ Γ d (.cap S.declassifyKind) ρd →
      HasType S σ Γ (.expose e d) τ (S.declassifyLabel :: (ρe ++ ρd))
  /-- Attenuation is pure: minting a narrower capability performs nothing, as minting from `Root` does not (§4). -/
  | narrow {Γ e₀ e₁ R ρ₀ ρ₁} :
      HasType S σ Γ e₀ (.cap R) ρ₀ → HasType S σ Γ e₁ S.narrowArgTy ρ₁ → HasType S σ Γ (.narrow e₀ e₁) (.cap R) (ρ₀ ++ ρ₁)
  /-- A token is typed by the store: a capability exists only where `σ` holds it (unforgeability, §1). -/
  | tok {Γ κ R s} : σ[κ]? = some (R, s) → HasType S σ Γ (.tok κ) (.cap R) []

/-! ### Rows -/

theorem Row.sub_refl (a : Row) : Row.sub a a := fun _ h => h

theorem Row.sub_trans {a b c : Row} (h₁ : Row.sub a b) (h₂ : Row.sub b c) : Row.sub a c :=
  fun x hx => h₂ x (h₁ x hx)

theorem Row.nil_sub (a : Row) : Row.sub [] a := fun _ h => nomatch h

/-! ### Weakening at the outer end and substitution of a closed value -/

/-- A derivation stays one when more types are added at the OUTER end of the context: no index below the old length
    moves. Used to place a closed value under binders. -/
theorem weaken_end {S σ Γ e τ ρ} (h : HasType S σ Γ e τ ρ) (Δ : List Ty) : HasType S σ (Γ ++ Δ) e τ ρ := by
  induction h with
  | var hn =>
    apply HasType.var
    rw [List.getElem?_append_left]
    · exact hn
    · exact (List.getElem?_eq_some_iff.mp hn).1
  | unit => exact .unit
  | bool => exact .bool
  | int => exact .int
  | str => exact .str
  | lam _ hs ih => exact .lam ih hs
  | app _ _ ih₁ ih₂ => exact .app ih₁ ih₂
  | letin _ _ ih₁ ih₂ => exact .letin ih₁ ih₂
  | pair _ _ ih₁ ih₂ => exact .pair ih₁ ih₂
  | fst _ ih => exact .fst ih
  | snd _ ih => exact .snd ih
  | inl _ ih => exact .inl ih
  | inr _ ih => exact .inr ih
  | case _ _ _ ih ih₁ ih₂ => exact .case ih ih₁ ih₂
  | op _ _ ih₀ ih₁ => exact .op ih₀ ih₁
  | hop _ _ ih₀ ih₁ => exact .hop ih₀ ih₁
  | wrap _ ih => exact .wrap ih
  | verify _ _ ih₁ ih₂ => exact .verify ih₁ ih₂
  | expose _ _ ih₁ ih₂ => exact .expose ih₁ ih₂
  | narrow _ _ ih₀ ih₁ => exact .narrow ih₀ ih₁
  | tok hκ => exact .tok hκ

/-- `subst k v e`: `e` with the variable `k` replaced by the CLOSED value `v` and every variable above `k` moved down
    one. Call-by-value reduction of a closed program substitutes only closed values, so `v` is never shifted. -/
def subst (k : Nat) (v : Tm) : Tm → Tm
  | .var n => if n < k then .var n else if n = k then v else .var (n - 1)
  | .unit => .unit
  | .bool b => .bool b
  | .int n => .int n
  | .str s => .str s
  | .lam τ ρ e => .lam τ ρ (subst (k + 1) v e)
  | .app e₁ e₂ => .app (subst k v e₁) (subst k v e₂)
  | .letin e₁ e₂ => .letin (subst k v e₁) (subst (k + 1) v e₂)
  | .pair e₁ e₂ => .pair (subst k v e₁) (subst k v e₂)
  | .fst e => .fst (subst k v e)
  | .snd e => .snd (subst k v e)
  | .inl e => .inl (subst k v e)
  | .inr e => .inr (subst k v e)
  | .case e e₁ e₂ => .case (subst k v e) (subst (k + 1) v e₁) (subst (k + 1) v e₂)
  | .op o e₀ e₁ => .op o (subst k v e₀) (subst k v e₁)
  | .hop e₀ e₁ => .hop (subst k v e₀) (subst k v e₁)
  | .wrap e => .wrap (subst k v e)
  | .verify e₁ e₂ => .verify (subst k v e₁) (subst k v e₂)
  | .expose e d => .expose (subst k v e) (subst k v d)
  | .narrow e₀ e₁ => .narrow (subst k v e₀) (subst k v e₁)
  | .tok κ => .tok κ

/-- The substitution lemma: types AND rows are preserved — the row of `e` does not change when a closed value, whose
    row is empty, stands for a variable. -/
theorem subst_typing {S σ Γ Δ e τ ρ τ₁ v} (h : HasType S σ (Γ ++ τ₁ :: Δ) e τ ρ) (hv : HasType S σ [] v τ₁ []) :
    HasType S σ (Γ ++ Δ) (subst Γ.length v e) τ ρ := by
  generalize hc : Γ ++ τ₁ :: Δ = Γ' at h
  induction h generalizing Γ with
  | @var Γ' n τ hn =>
    subst hc
    simp only [subst]
    by_cases hlt : n < Γ.length
    · simp only [hlt, if_true]
      apply HasType.var
      rw [List.getElem?_append_left hlt] at hn
      rw [List.getElem?_append_left hlt]
      exact hn
    · by_cases heq : n = Γ.length
      · subst heq
        simp only [Nat.lt_irrefl, if_false, if_true]
        rw [List.getElem?_append_right (Nat.le_refl _)] at hn
        simp at hn
        subst hn
        have := weaken_end hv (Γ ++ Δ)
        simpa using this
      · have hgt : Γ.length < n := by omega
        simp only [hlt, heq, if_false]
        apply HasType.var
        rw [List.getElem?_append_right (by omega)] at hn
        rw [List.getElem?_append_right (by omega)]
        have hk : n - Γ.length = (n - 1 - Γ.length) + 1 := by omega
        rw [hk] at hn
        simpa using hn
  | unit => exact .unit
  | bool => exact .bool
  | int => exact .int
  | str => exact .str
  | lam _ hs ih =>
    subst hc
    exact .lam (ih (Γ := _ :: Γ) rfl) hs
  | app _ _ ih₁ ih₂ => subst hc; exact .app (ih₁ rfl) (ih₂ rfl)
  | letin _ _ ih₁ ih₂ => subst hc; exact .letin (ih₁ rfl) (ih₂ (Γ := _ :: Γ) rfl)
  | pair _ _ ih₁ ih₂ => subst hc; exact .pair (ih₁ rfl) (ih₂ rfl)
  | fst _ ih => subst hc; exact .fst (ih rfl)
  | snd _ ih => subst hc; exact .snd (ih rfl)
  | inl _ ih => subst hc; exact .inl (ih rfl)
  | inr _ ih => subst hc; exact .inr (ih rfl)
  | case _ _ _ ih ih₁ ih₂ =>
    subst hc
    exact .case (ih rfl) (ih₁ (Γ := _ :: Γ) rfl) (ih₂ (Γ := _ :: Γ) rfl)
  | op _ _ ih₀ ih₁ => subst hc; exact .op (ih₀ rfl) (ih₁ rfl)
  | hop _ _ ih₀ ih₁ => subst hc; exact .hop (ih₀ rfl) (ih₁ rfl)
  | wrap _ ih => subst hc; exact .wrap (ih rfl)
  | verify _ _ ih₁ ih₂ => subst hc; exact .verify (ih₁ rfl) (ih₂ rfl)
  | expose _ _ ih₁ ih₂ => subst hc; exact .expose (ih₁ rfl) (ih₂ rfl)
  | narrow _ _ ih₀ ih₁ => subst hc; exact .narrow (ih₀ rfl) (ih₁ rfl)
  | tok hκ => exact .tok hκ

/-- Substitution into a body typed under one binder, in the empty context. -/
theorem subst0_typing {S σ e τ ρ τ₁ v} (h : HasType S σ [τ₁] e τ ρ) (hv : HasType S σ [] v τ₁ []) :
    HasType S σ [] (subst 0 v e) τ ρ :=
  subst_typing (Γ := []) (Δ := []) h hv

/-! ### The store only grows -/

/-- A derivation under a store stays one under any extension of it: a token keeps its index, kind and scope. -/
theorem weaken_store {S σ Γ e τ ρ} (h : HasType S σ Γ e τ ρ) (δ : Store) : HasType S (σ ++ δ) Γ e τ ρ := by
  induction h with
  | var hn => exact .var hn
  | unit => exact .unit
  | bool => exact .bool
  | int => exact .int
  | str => exact .str
  | lam _ hs ih => exact .lam ih hs
  | app _ _ ih₁ ih₂ => exact .app ih₁ ih₂
  | letin _ _ ih₁ ih₂ => exact .letin ih₁ ih₂
  | pair _ _ ih₁ ih₂ => exact .pair ih₁ ih₂
  | fst _ ih => exact .fst ih
  | snd _ ih => exact .snd ih
  | inl _ ih => exact .inl ih
  | inr _ ih => exact .inr ih
  | case _ _ _ ih ih₁ ih₂ => exact .case ih ih₁ ih₂
  | op _ _ ih₀ ih₁ => exact .op ih₀ ih₁
  | hop _ _ ih₀ ih₁ => exact .hop ih₀ ih₁
  | wrap _ ih => exact .wrap ih
  | verify _ _ ih₁ ih₂ => exact .verify ih₁ ih₂
  | expose _ _ ih₁ ih₂ => exact .expose ih₁ ih₂
  | narrow _ _ ih₀ ih₁ => exact .narrow ih₀ ih₁
  | tok hκ =>
    apply HasType.tok
    rw [List.getElem?_append_left (List.getElem?_eq_some_iff.mp hκ).1]
    exact hκ

/-! ### Operational semantics (§4) -/

/-- A configuration's next state: a term, or `fault` — a run that ended refused (`E-Refuse`), which does not reduce. -/
inductive Res where
  | ok : Tm → Res
  | fault : Res

/-- One call-by-value evaluation frame `E[·]`, left to right. -/
inductive Frame where
  | app1 (e₂ : Tm)
  | app2 (v₁ : Tm)
  | let1 (e₂ : Tm)
  | pair1 (e₂ : Tm)
  | pair2 (v₁ : Tm)
  | fst
  | snd
  | inl
  | inr
  | case1 (e₁ e₂ : Tm)
  | op0 (o : Nat) (e₁ : Tm)
  | op1 (o : Nat) (v₀ : Tm)
  | hop0 (e₁ : Tm)
  | hop1 (v₀ : Tm)
  | wrap
  | verify1 (e₂ : Tm)
  | verify2 (v₁ : Tm)
  | expose1 (d : Tm)
  | expose2 (v : Tm)
  | narrow0 (e₁ : Tm)
  | narrow1 (v₀ : Tm)

/-- The term with `e` in the frame's hole. -/
def Frame.plug : Frame → Tm → Tm
  | .app1 e₂, e => .app e e₂
  | .app2 v₁, e => .app v₁ e
  | .let1 e₂, e => .letin e e₂
  | .pair1 e₂, e => .pair e e₂
  | .pair2 v₁, e => .pair v₁ e
  | .fst, e => .fst e
  | .snd, e => .snd e
  | .inl, e => .inl e
  | .inr, e => .inr e
  | .case1 e₁ e₂, e => .case e e₁ e₂
  | .op0 o e₁, e => .op o e e₁
  | .op1 o v₀, e => .op o v₀ e
  | .hop0 e₁, e => .hop e e₁
  | .hop1 v₀, e => .hop v₀ e
  | .wrap, e => .wrap e
  | .verify1 e₂, e => .verify e e₂
  | .verify2 v₁, e => .verify v₁ e
  | .expose1 d, e => .expose e d
  | .expose2 v, e => .expose v e
  | .narrow0 e₁, e => .narrow e e₁
  | .narrow1 v₀, e => .narrow v₀ e

/-- A frame's left neighbours are values (call-by-value, left to right). -/
def Frame.wf : Frame → Prop
  | .app2 v | .pair2 v | .op1 _ v | .hop1 v | .verify2 v | .expose2 v | .narrow1 v => Value v
  | _ => True

/-- `⟨σ ; e⟩ ⟶ ⟨σ' ; r⟩`, optionally emitting a label (§4). Only attenuation changes the store, and only by adding a
    token at its end. -/
inductive Step (S : Sig) : Store → Tm → Option Label → Store → Res → Prop where
  | appβ {σ τ ρ e v} : Value v → Step S σ (.app (.lam τ ρ e) v) none σ (.ok (subst 0 v e))
  | letβ {σ v e} : Value v → Step S σ (.letin v e) none σ (.ok (subst 0 v e))
  | fstβ {σ v₁ v₂} : Value v₁ → Value v₂ → Step S σ (.fst (.pair v₁ v₂)) none σ (.ok v₁)
  | sndβ {σ v₁ v₂} : Value v₁ → Value v₂ → Step S σ (.snd (.pair v₁ v₂)) none σ (.ok v₂)
  | caseL {σ v e₁ e₂} : Value v → Step S σ (.case (.inl v) e₁ e₂) none σ (.ok (subst 0 v e₁))
  | caseR {σ v e₁ e₂} : Value v → Step S σ (.case (.inr v) e₁ e₂) none σ (.ok (subst 0 v e₂))
  /-- `E-Op`: fires only on a token the store holds, of the operation's kind, whose scope permits the argument. -/
  | opOk {σ o κ s v} : σ[κ]? = some (S.kind o, s) → Value v → S.permits s v = true →
      Step S σ (.op o (.tok κ) v) (some (S.label o)) σ (.ok (S.δ o v))
  /-- `E-Refuse` (v0.3): a present capability whose scope does not cover the argument — the operation is not performed,
      the run faults, and the trace still records the attempt. -/
  | opRefuse {σ o κ s v} : σ[κ]? = some (S.kind o, s) → Value v → S.permits s v = false →
      Step S σ (.op o (.tok κ) v) (some (S.label o)) σ .fault
  /-- `E-HOp` (v0.3), for the primitive whose unfolding `δ_hop(v_f, v)` is the callback applied once: it emits nothing
      itself; the callback's labels are emitted by the application as it reduces. -/
  | hopβ {σ f v} : Value f → Value v → Step S σ (.hop f v) none σ (.ok (.app f v))
  | verifyβ {σ v₁ v₂} : Value v₁ → Value v₂ →
      Step S σ (.verify (.wrap v₁) (.wrap v₂)) S.verifyLabel σ (.ok (.bool (decide (v₁ = v₂))))
  | exposeβ {σ v κ s} : Value v → σ[κ]? = some (S.declassifyKind, s) →
      Step S σ (.expose (.wrap v) (.tok κ)) (some S.declassifyLabel) σ (.ok v)
  /-- Attenuation: a new token, of the parent's kind, at the end of the store, its scope the parent's narrowed — the one
      way reduction creates a token (§4). Pure: no label. -/
  | narrowOk {σ κ R s v} : σ[κ]? = some (R, s) → Value v → S.narrowOk s v = true →
      Step S σ (.narrow (.tok κ) v) none (σ ++ [(R, S.narrowScope s v)]) (.ok (.tok σ.length))
  /-- A narrowing the parent's scope does not admit (a spelling that escapes it, `DL0904`) is refused: a fault, no token. -/
  | narrowRefuse {σ κ R s v} : σ[κ]? = some (R, s) → Value v → S.narrowOk s v = false →
      Step S σ (.narrow (.tok κ) v) none σ .fault
  /-- Congruence: a subterm's step is the whole term's. -/
  | ctx {σ σ' : Store} {F : Frame} {e l e'} : F.wf → Step S σ e l σ' (.ok e') → Step S σ (F.plug e) l σ' (.ok (F.plug e'))
  /-- `E-Fault` (v0.3): nothing in the calculus can catch a fault, so it is the whole configuration's. -/
  | ctxF {σ σ' : Store} {F : Frame} {e l} : F.wf → Step S σ e l σ' .fault → Step S σ (F.plug e) l σ' .fault

/-- A step only extends the store. -/
theorem step_extends {S σ e l σ' r} (h : Step S σ e l σ' r) : ∃ δ, σ' = σ ++ δ := by
  induction h with
  | narrowOk => exact ⟨_, rfl⟩
  | ctx _ _ ih => exact ih
  | ctxF _ _ ih => exact ih
  | _ => exact ⟨[], by simp⟩

/-- What the theorems ask of `Σ`: an operation's result on a permitted, well-typed argument is a closed term of its
    result type with an empty row — the operation's own label is accounted for by `T-Op`, and nothing else is emitted. -/
def SigOk (S : Sig) : Prop :=
  ∀ σ o s v, Value v → HasType S σ [] v (S.argTy o) [] → S.permits s v = true → HasType S σ [] (S.δ o v) (S.resTy o) []

/-! ### Values -/

/-- A value's row is empty: building a value performs nothing. -/
theorem value_row {S σ Γ v τ ρ} (hv : Value v) (h : HasType S σ Γ v τ ρ) : ρ = [] := by
  induction hv generalizing τ ρ with
  | unit => cases h; rfl
  | bool => cases h; rfl
  | int => cases h; rfl
  | str => cases h; rfl
  | lam => cases h; rfl
  | pair _ _ ih₁ ih₂ => cases h with | pair h₁ h₂ => rw [ih₁ h₁, ih₂ h₂]; rfl
  | inl _ ih => cases h with | inl h => exact ih h
  | inr _ ih => cases h with | inr h => exact ih h
  | wrap _ ih => cases h with | wrap h => exact ih h
  | tok => cases h; rfl

/-- Canonical forms, for closed values. -/
theorem canon_arr {S σ v τ₁ ρf τ₂ ρ} (hv : Value v) (h : HasType S σ [] v (.arr τ₁ ρf τ₂) ρ) :
    ∃ e, v = .lam τ₁ ρf e := by
  cases hv <;> cases h; exact ⟨_, rfl⟩

theorem canon_cap {S σ v R ρ} (hv : Value v) (h : HasType S σ [] v (.cap R) ρ) : ∃ κ s, v = .tok κ ∧ σ[κ]? = some (R, s) := by
  cases hv <;> cases h; exact ⟨_, _, rfl, ‹_›⟩

theorem canon_secret {S σ v τ ρ} (hv : Value v) (h : HasType S σ [] v (.secret τ) ρ) : ∃ w, v = .wrap w ∧ Value w := by
  cases hv <;> cases h; exact ⟨_, rfl, ‹_›⟩

theorem canon_prod {S σ v τ₁ τ₂ ρ} (hv : Value v) (h : HasType S σ [] v (.prod τ₁ τ₂) ρ) :
    ∃ v₁ v₂, v = .pair v₁ v₂ ∧ Value v₁ ∧ Value v₂ := by
  cases hv <;> cases h; exact ⟨_, _, rfl, ‹_›, ‹_›⟩

theorem canon_sum {S σ v τ₁ τ₂ ρ} (hv : Value v) (h : HasType S σ [] v (.sum τ₁ τ₂) ρ) :
    ∃ w, Value w ∧ (v = .inl w ∨ v = .inr w) := by
  cases hv <;> cases h
  · exact ⟨_, ‹_›, .inl rfl⟩
  · exact ⟨_, ‹_›, .inr rfl⟩

/-! ### A frame's typing comes apart and goes back together -/

/-- Row bookkeeping: containment goals over `++` and `::`, closed by `grind` once `Row.sub` is unfolded. -/
macro "rowsub" : tactic => `(tactic| (simp only [Row.sub] at *; grind))

/-- The term in a frame's hole is typed, its row is inside the whole's, and any term of the hole's type with a smaller
    row, under any extension of the store, can take its place with the whole's type at a smaller row. Every congruence
    case of Preservation is this. -/
theorem frame_typing {S σ F e τ ρ} (h : HasType S σ [] (Frame.plug F e) τ ρ) :
    ∃ τh ρh, HasType S σ [] e τh ρh ∧ Row.sub ρh ρ ∧
      ∀ δ e' ρh', HasType S (σ ++ δ) [] e' τh ρh' → Row.sub ρh' ρh →
        ∃ ρ', HasType S (σ ++ δ) [] (Frame.plug F e') τ ρ' ∧ Row.sub ρ' ρ := by
  cases F <;> simp only [Frame.plug] at h ⊢
  case app1 => cases h with
    | app h₁ h₂ => exact ⟨_, _, h₁, by rowsub, fun δ _ _ h' hs => ⟨_, .app h' (weaken_store h₂ δ), by rowsub⟩⟩
  case app2 => cases h with
    | app h₁ h₂ => exact ⟨_, _, h₂, by rowsub, fun δ _ _ h' hs => ⟨_, .app (weaken_store h₁ δ) h', by rowsub⟩⟩
  case let1 => cases h with
    | letin h₁ h₂ => exact ⟨_, _, h₁, by rowsub, fun δ _ _ h' hs => ⟨_, .letin h' (weaken_store h₂ δ), by rowsub⟩⟩
  case pair1 => cases h with
    | pair h₁ h₂ => exact ⟨_, _, h₁, by rowsub, fun δ _ _ h' hs => ⟨_, .pair h' (weaken_store h₂ δ), by rowsub⟩⟩
  case pair2 => cases h with
    | pair h₁ h₂ => exact ⟨_, _, h₂, by rowsub, fun δ _ _ h' hs => ⟨_, .pair (weaken_store h₁ δ) h', by rowsub⟩⟩
  case fst => cases h with
    | fst h₁ => exact ⟨_, _, h₁, Row.sub_refl _, fun _ _ _ h' hs => ⟨_, .fst h', hs⟩⟩
  case snd => cases h with
    | snd h₁ => exact ⟨_, _, h₁, Row.sub_refl _, fun _ _ _ h' hs => ⟨_, .snd h', hs⟩⟩
  case inl => cases h with
    | inl h₁ => exact ⟨_, _, h₁, Row.sub_refl _, fun _ _ _ h' hs => ⟨_, .inl h', hs⟩⟩
  case inr => cases h with
    | inr h₁ => exact ⟨_, _, h₁, Row.sub_refl _, fun _ _ _ h' hs => ⟨_, .inr h', hs⟩⟩
  case case1 => cases h with
    | case h₀ h₁ h₂ =>
      exact ⟨_, _, h₀, by rowsub, fun δ _ _ h' hs => ⟨_, .case h' (weaken_store h₁ δ) (weaken_store h₂ δ), by rowsub⟩⟩
  case op0 => cases h with
    | op h₀ h₁ => exact ⟨_, _, h₀, by rowsub, fun δ _ _ h' hs => ⟨_, .op h' (weaken_store h₁ δ), by rowsub⟩⟩
  case op1 => cases h with
    | op h₀ h₁ => exact ⟨_, _, h₁, by rowsub, fun δ _ _ h' hs => ⟨_, .op (weaken_store h₀ δ) h', by rowsub⟩⟩
  case hop0 => cases h with
    | hop h₀ h₁ => exact ⟨_, _, h₀, by rowsub, fun δ _ _ h' hs => ⟨_, .hop h' (weaken_store h₁ δ), by rowsub⟩⟩
  case hop1 => cases h with
    | hop h₀ h₁ => exact ⟨_, _, h₁, by rowsub, fun δ _ _ h' hs => ⟨_, .hop (weaken_store h₀ δ) h', by rowsub⟩⟩
  case wrap => cases h with
    | wrap h₁ => exact ⟨_, _, h₁, Row.sub_refl _, fun _ _ _ h' hs => ⟨_, .wrap h', hs⟩⟩
  case verify1 => cases h with
    | verify h₁ h₂ => exact ⟨_, _, h₁, by rowsub, fun δ _ _ h' hs => ⟨_, .verify h' (weaken_store h₂ δ), by rowsub⟩⟩
  case verify2 => cases h with
    | verify h₁ h₂ => exact ⟨_, _, h₂, by rowsub, fun δ _ _ h' hs => ⟨_, .verify (weaken_store h₁ δ) h', by rowsub⟩⟩
  case expose1 => cases h with
    | expose h₁ h₂ => exact ⟨_, _, h₁, by rowsub, fun δ _ _ h' hs => ⟨_, .expose h' (weaken_store h₂ δ), by rowsub⟩⟩
  case expose2 => cases h with
    | expose h₁ h₂ => exact ⟨_, _, h₂, by rowsub, fun δ _ _ h' hs => ⟨_, .expose (weaken_store h₁ δ) h', by rowsub⟩⟩
  case narrow0 => cases h with
    | narrow h₀ h₁ => exact ⟨_, _, h₀, by rowsub, fun δ _ _ h' hs => ⟨_, .narrow h' (weaken_store h₁ δ), by rowsub⟩⟩
  case narrow1 => cases h with
    | narrow h₀ h₁ => exact ⟨_, _, h₁, by rowsub, fun δ _ _ h' hs => ⟨_, .narrow (weaken_store h₀ δ) h', by rowsub⟩⟩

/-! ### Theorem 2 — Preservation, types and rows -/

/-- **Theorem 2.** A step from a well-typed closed term emits only a label its row holds, and what it steps to has the
    same type, under the store the step leaves, at a row inside the old one; a step to a fault emits only a label its
    row holds. -/
theorem preservation {S σ e l σ' r τ ρ} (hS : SigOk S) (hst : Step S σ e l σ' r) (h : HasType S σ [] e τ ρ) :
    (∀ x, l = some x → x ∈ ρ) ∧ (∀ e', r = .ok e' → ∃ ρ', HasType S σ' [] e' τ ρ' ∧ Row.sub ρ' ρ) := by
  induction hst generalizing τ ρ with
  | appβ hv =>
    cases h with
    | app h₁ h₂ =>
      cases h₁ with
      | lam hb hs =>
        have := value_row hv h₂; subst this
        refine ⟨by simp, fun e' he' => ?_⟩
        cases he'
        exact ⟨_, subst0_typing hb h₂, by rowsub⟩
  | letβ hv =>
    cases h with
    | letin h₁ h₂ =>
      have := value_row hv h₁; subst this
      refine ⟨by simp, fun e' he' => ?_⟩
      cases he'
      exact ⟨_, subst0_typing h₂ h₁, by rowsub⟩
  | fstβ =>
    cases h with
    | fst h =>
      cases h with
      | pair h₁ h₂ =>
        refine ⟨by simp, fun e' he' => ?_⟩
        cases he'
        exact ⟨_, h₁, by rowsub⟩
  | sndβ =>
    cases h with
    | snd h =>
      cases h with
      | pair h₁ h₂ =>
        refine ⟨by simp, fun e' he' => ?_⟩
        cases he'
        exact ⟨_, h₂, by rowsub⟩
  | caseL hv =>
    cases h with
    | case h₀ h₁ h₂ =>
      cases h₀ with
      | inl hw =>
        have := value_row hv hw; subst this
        refine ⟨by simp, fun e' he' => ?_⟩
        cases he'
        exact ⟨_, subst0_typing h₁ hw, by rowsub⟩
  | caseR hv =>
    cases h with
    | case h₀ h₁ h₂ =>
      cases h₀ with
      | inr hw =>
        have := value_row hv hw; subst this
        refine ⟨by simp, fun e' he' => ?_⟩
        cases he'
        exact ⟨_, subst0_typing h₂ hw, by rowsub⟩
  | opOk _ hv hp =>
    cases h with
    | op h₀ h₁ =>
      have := value_row hv h₁; subst this
      refine ⟨by simp, fun e' he' => ?_⟩
      cases he'
      exact ⟨_, hS _ _ _ _ hv h₁ hp, Row.nil_sub _⟩
  | opRefuse =>
    cases h with
    | op => exact ⟨by simp, fun e' he' => nomatch he'⟩
  | hopβ hf hv =>
    cases h with
    | hop h₀ h₁ =>
      have := value_row hf h₀; subst this
      have := value_row hv h₁; subst this
      refine ⟨by simp, fun e' he' => ?_⟩
      cases he'
      exact ⟨_, .app h₀ h₁, by rowsub⟩
  | verifyβ =>
    cases h with
    | verify h₁ h₂ =>
      refine ⟨fun x hx => by simp [hx], fun e' he' => ?_⟩
      cases he'
      exact ⟨_, .bool, Row.nil_sub _⟩
  | exposeβ hv =>
    cases h with
    | expose h₁ h₂ =>
      cases h₁ with
      | wrap hw =>
        have := value_row hv hw; subst this
        refine ⟨by simp, fun e' he' => ?_⟩
        cases he'
        exact ⟨_, hw, Row.nil_sub _⟩
  | @narrowOk κ R s v hκ =>
    cases h with
    | narrow h₀ h₁ =>
      cases h₀ with
      | tok hκ' =>
        rw [hκ] at hκ'
        simp only [Option.some.injEq, Prod.mk.injEq] at hκ'
        obtain ⟨rfl, rfl⟩ := hκ'
        refine ⟨by simp, fun e' he' => ?_⟩
        cases he'
        exact ⟨_, .tok (s := S.narrowScope s v) (by simp), Row.nil_sub _⟩
  | narrowRefuse =>
    cases h with
    | narrow => exact ⟨by simp, fun e' he' => nomatch he'⟩
  | ctx _ hstep ih =>
    obtain ⟨τh, ρh, he, hsub, back⟩ := frame_typing h
    obtain ⟨hl, hok⟩ := ih he
    refine ⟨fun x hx => hsub x (hl x hx), fun e' he' => ?_⟩
    cases he'
    obtain ⟨ρh', he', hsub'⟩ := hok _ rfl
    obtain ⟨δ, rfl⟩ := step_extends hstep
    exact back _ _ _ he' hsub'
  | ctxF _ _ ih =>
    obtain ⟨τh, ρh, he, hsub, _⟩ := frame_typing h
    obtain ⟨hl, _⟩ := ih he
    exact ⟨fun x hx => hsub x (hl x hx), fun e' he' => nomatch he'⟩

/-! ### Theorem 1 — Progress-or-fault -/

theorem step_in {S σ e l σ' r} (F : Frame) (hF : F.wf) (hs : Step S σ e l σ' r) :
    ∃ l σ' r, Step S σ (F.plug e) l σ' r := by
  cases r with
  | ok e' => exact ⟨_, _, _, .ctx hF hs⟩
  | fault => exact ⟨_, _, _, .ctxF hF hs⟩

/-- **Theorem 1.** A well-typed closed term is a value or takes a step — to a term, or to a fault. None is stuck: a
    missing capability is a type error, and a present one whose scope does not cover the argument is `E-Refuse`. -/
theorem progress {S σ e τ ρ} (h : HasType S σ [] e τ ρ) : Value e ∨ ∃ l σ' r, Step S σ e l σ' r := by
  generalize hΓ : ([] : List Ty) = Γ at h
  induction h with
  | var hn => subst hΓ; simp at hn
  | unit => exact .inl .unit
  | bool => exact .inl (.bool _)
  | int => exact .inl (.int _)
  | str => exact .inl (.str _)
  | lam => exact .inl (.lam _ _ _)
  | app h₁ _ ih₁ ih₂ =>
    subst hΓ; right
    rcases ih₁ rfl with hv₁ | ⟨l, σ', r, hs⟩
    · rcases ih₂ rfl with hv₂ | ⟨l, σ', r, hs⟩
      · obtain ⟨e, rfl⟩ := canon_arr hv₁ h₁
        exact ⟨_, _, _, .appβ hv₂⟩
      · exact step_in (.app2 _) hv₁ hs
    · exact step_in (.app1 _) trivial hs
  | letin _ _ ih₁ _ =>
    subst hΓ; right
    rcases ih₁ rfl with hv₁ | ⟨l, σ', r, hs⟩
    · exact ⟨_, _, _, .letβ hv₁⟩
    · exact step_in (.let1 _) trivial hs
  | pair _ _ ih₁ ih₂ =>
    subst hΓ
    rcases ih₁ rfl with hv₁ | ⟨l, σ', r, hs⟩
    · rcases ih₂ rfl with hv₂ | ⟨l, σ', r, hs⟩
      · exact .inl (.pair hv₁ hv₂)
      · exact .inr (step_in (.pair2 _) hv₁ hs)
    · exact .inr (step_in (.pair1 _) trivial hs)
  | fst h₁ ih =>
    subst hΓ; right
    rcases ih rfl with hv | ⟨l, σ', r, hs⟩
    · obtain ⟨v₁, v₂, rfl, hv₁, hv₂⟩ := canon_prod hv h₁
      exact ⟨_, _, _, .fstβ hv₁ hv₂⟩
    · exact step_in .fst trivial hs
  | snd h₁ ih =>
    subst hΓ; right
    rcases ih rfl with hv | ⟨l, σ', r, hs⟩
    · obtain ⟨v₁, v₂, rfl, hv₁, hv₂⟩ := canon_prod hv h₁
      exact ⟨_, _, _, .sndβ hv₁ hv₂⟩
    · exact step_in .snd trivial hs
  | inl _ ih =>
    subst hΓ
    rcases ih rfl with hv | ⟨l, σ', r, hs⟩
    · exact .inl (.inl hv)
    · exact .inr (step_in .inl trivial hs)
  | inr _ ih =>
    subst hΓ
    rcases ih rfl with hv | ⟨l, σ', r, hs⟩
    · exact .inl (.inr hv)
    · exact .inr (step_in .inr trivial hs)
  | case h₀ _ _ ih _ _ =>
    subst hΓ; right
    rcases ih rfl with hv | ⟨l, σ', r, hs⟩
    · obtain ⟨w, hw, rfl | rfl⟩ := canon_sum hv h₀
      · exact ⟨_, _, _, .caseL hw⟩
      · exact ⟨_, _, _, .caseR hw⟩
    · exact step_in (.case1 _ _) trivial hs
  | @op _ o e₀ e₁ _ _ h₀ _ ih₀ ih₁ =>
    subst hΓ; right
    rcases ih₀ rfl with hv₀ | ⟨l, σ', r, hs⟩
    · rcases ih₁ rfl with hv₁ | ⟨l, σ', r, hs⟩
      · obtain ⟨κ, s, rfl, hκ⟩ := canon_cap hv₀ h₀
        cases hp : S.permits s e₁ with
        | true => exact ⟨_, _, _, .opOk hκ hv₁ hp⟩
        | false => exact ⟨_, _, _, .opRefuse hκ hv₁ hp⟩
      · exact step_in (.op1 _ _) hv₀ hs
    · exact step_in (.op0 _ _) trivial hs
  | hop _ _ ih₀ ih₁ =>
    subst hΓ; right
    rcases ih₀ rfl with hv₀ | ⟨l, σ', r, hs⟩
    · rcases ih₁ rfl with hv₁ | ⟨l, σ', r, hs⟩
      · exact ⟨_, _, _, .hopβ hv₀ hv₁⟩
      · exact step_in (.hop1 _) hv₀ hs
    · exact step_in (.hop0 _) trivial hs
  | wrap _ ih =>
    subst hΓ
    rcases ih rfl with hv | ⟨l, σ', r, hs⟩
    · exact .inl (.wrap hv)
    · exact .inr (step_in .wrap trivial hs)
  | verify h₁ h₂ ih₁ ih₂ =>
    subst hΓ; right
    rcases ih₁ rfl with hv₁ | ⟨l, σ', r, hs⟩
    · rcases ih₂ rfl with hv₂ | ⟨l, σ', r, hs⟩
      · obtain ⟨w₁, rfl, hw₁⟩ := canon_secret hv₁ h₁
        obtain ⟨w₂, rfl, hw₂⟩ := canon_secret hv₂ h₂
        exact ⟨_, _, _, .verifyβ hw₁ hw₂⟩
      · exact step_in (.verify2 _) hv₁ hs
    · exact step_in (.verify1 _) trivial hs
  | expose h₁ h₂ ih₁ ih₂ =>
    subst hΓ; right
    rcases ih₁ rfl with hv₁ | ⟨l, σ', r, hs⟩
    · rcases ih₂ rfl with hv₂ | ⟨l, σ', r, hs⟩
      · obtain ⟨w, rfl, hw⟩ := canon_secret hv₁ h₁
        obtain ⟨κ, s, rfl, hκ⟩ := canon_cap hv₂ h₂
        exact ⟨_, _, _, .exposeβ hw hκ⟩
      · exact step_in (.expose2 _) hv₁ hs
    · exact step_in (.expose1 _) trivial hs
  | @narrow _ e₀ e₁ _ _ _ h₀ _ ih₀ ih₁ =>
    subst hΓ; right
    rcases ih₀ rfl with hv₀ | ⟨l, σ', r, hs⟩
    · rcases ih₁ rfl with hv₁ | ⟨l, σ', r, hs⟩
      · obtain ⟨κ, s, rfl, hκ⟩ := canon_cap hv₀ h₀
        cases hp : S.narrowOk s e₁ with
        | true => exact ⟨_, _, _, .narrowOk hκ hv₁ hp⟩
        | false => exact ⟨_, _, _, .narrowRefuse hκ hv₁ hp⟩
      · exact step_in (.narrow1 _) hv₀ hs
    · exact step_in (.narrow0 _) trivial hs
  | tok => exact .inl (.tok _)

/-! ### Theorem 3 — Effect soundness, over every finite prefix of a run -/

/-- `⟨σ ; e⟩ ⟶* ⟨σ' ; r⟩`, with the labels emitted on the way: any number of steps to a term (a value, or any
    configuration part-way — every finite prefix of a run, including one that never ends), or to a fault. -/
inductive Steps (S : Sig) : Store → Tm → List Label → Store → Res → Prop where
  | refl {σ e} : Steps S σ e [] σ (.ok e)
  | step {σ e l σ₁ e' tr σ' r} : Step S σ e l σ₁ (.ok e') → Steps S σ₁ e' tr σ' r → Steps S σ e (l.toList ++ tr) σ' r
  | fault {σ e l σ'} : Step S σ e l σ' .fault → Steps S σ e l.toList σ' .fault

/-- **Theorem 3 (Effect soundness — "authority cannot escape the type").** Every label a run of a well-typed closed term
    emits is in the term's row, whether the run ends in a value, a fault, or is cut anywhere part-way. -/
theorem effect_soundness {S σ e τ ρ tr σ' r} (hS : SigOk S) (h : HasType S σ [] e τ ρ) (hs : Steps S σ e tr σ' r) :
    ∀ x, x ∈ tr → x ∈ ρ := by
  induction hs generalizing ρ with
  | refl => intro x hx; nomatch hx
  | step hst _ ih =>
    obtain ⟨hl, hok⟩ := preservation hS hst h
    obtain ⟨ρ', h', hsub⟩ := hok _ rfl
    intro x hx
    rcases List.mem_append.mp hx with hx | hx
    · exact hl x (by simpa using hx)
    · exact hsub x (ih h' x hx)
  | fault hst =>
    obtain ⟨hl, _⟩ := preservation hS hst h
    intro x hx
    exact hl x (by simpa using hx)

/-- Type safety along a run: every configuration a well-typed closed term reaches is well typed, under the store the run
    has built, at a smaller row, and is a value or steps — no run gets stuck. -/
theorem safety {S σ e τ ρ tr σ' e'} (hS : SigOk S) (h : HasType S σ [] e τ ρ) (hs : Steps S σ e tr σ' (.ok e')) :
    (∃ ρ', HasType S σ' [] e' τ ρ' ∧ Row.sub ρ' ρ) ∧ (Value e' ∨ ∃ l σ'' r, Step S σ' e' l σ'' r) := by
  have typed : ∃ ρ', HasType S σ' [] e' τ ρ' ∧ Row.sub ρ' ρ := by
    generalize hr : Res.ok e' = r at hs
    induction hs generalizing ρ with
    | refl => cases hr; exact ⟨_, h, Row.sub_refl _⟩
    | step hst _ ih =>
      obtain ⟨_, hok⟩ := preservation hS hst h
      obtain ⟨ρ₁, h₁, hsub₁⟩ := hok _ rfl
      obtain ⟨ρ₂, h₂, hsub₂⟩ := ih h₁ hr
      exact ⟨ρ₂, h₂, Row.sub_trans hsub₂ hsub₁⟩
    | fault => cases hr
  obtain ⟨ρ', h', hsub⟩ := typed
  exact ⟨⟨ρ', h', hsub⟩, progress h'⟩

/-! ### No forgery, and attenuation is monotone -/

/-- The capability tokens occurring in a term. -/
def Tm.toks : Tm → List Nat
  | .var _ | .unit | .bool _ | .int _ | .str _ => []
  | .lam _ _ e | .fst e | .snd e | .inl e | .inr e | .wrap e => e.toks
  | .app a b | .letin a b | .pair a b | .op _ a b | .hop a b | .verify a b | .expose a b | .narrow a b =>
    a.toks ++ b.toks
  | .case a b c => a.toks ++ b.toks ++ c.toks
  | .tok κ => [κ]

/-- A well-typed term names only tokens the store holds — the typing rule for a token is the store's word. -/
theorem typed_toks {S σ Γ e τ ρ} (h : HasType S σ Γ e τ ρ) : ∀ κ, κ ∈ e.toks → ∃ R s, σ[κ]? = some (R, s) := by
  induction h with
  | tok hκ => intro κ hk; simp [Tm.toks] at hk; subst hk; exact ⟨_, _, hκ⟩
  | var | unit | bool | int | str => intro κ hk; simp [Tm.toks] at hk
  | lam _ _ ih | fst _ ih | snd _ ih | inl _ ih | inr _ ih | wrap _ ih =>
    intro κ hk; exact ih κ (by simpa [Tm.toks] using hk)
  | app _ _ ih₁ ih₂ | letin _ _ ih₁ ih₂ | pair _ _ ih₁ ih₂ | op _ _ ih₁ ih₂ | hop _ _ ih₁ ih₂ | verify _ _ ih₁ ih₂
    | expose _ _ ih₁ ih₂ | narrow _ _ ih₁ ih₂ =>
    intro κ hk
    simp only [Tm.toks, List.mem_append] at hk
    rcases hk with hk | hk
    · exact ih₁ κ hk
    · exact ih₂ κ hk
  | case _ _ _ ih ih₁ ih₂ =>
    intro κ hk
    simp only [Tm.toks, List.mem_append] at hk
    rcases hk with (hk | hk) | hk
    · exact ih κ hk
    · exact ih₁ κ hk
    · exact ih₂ κ hk

/-- Every token of `σ` descends from one of `σ₀`: the same kind, a scope under its scope. -/
def Descends (S : Sig) (σ₀ σ : Store) : Prop :=
  ∀ (κ : Nat) (R : Kind) (s : Nat), σ[κ]? = some (R, s) → ∃ (κ₀ : Nat) (s₀ : Nat), σ₀[κ₀]? = some (R, s₀) ∧ S.le s s₀

theorem descends_refl (S : Sig) (σ : Store) : Descends S σ σ := fun κ _ s h => ⟨κ, s, h, S.le_refl s⟩

/-- A step keeps every token descended from the root store: the only token it can add is a narrowing, of its parent's
    kind and under its parent's scope (audit R-7). -/
theorem step_descends {S σ₀ σ e l σ' r} (hst : Step S σ e l σ' r) (hd : Descends S σ₀ σ) : Descends S σ₀ σ' := by
  induction hst with
  | @narrowOk κ R s v hκ =>
    unfold Descends at hd ⊢
    intro κ' R' s' h'
    by_cases hlt : κ' < σ.length
    · rw [List.getElem?_append_left hlt] at h'
      exact hd κ' R' s' h'
    · rw [List.getElem?_append_right (by omega)] at h'
      by_cases heq : κ' = σ.length
      · subst heq
        simp at h'
        obtain ⟨rfl, rfl⟩ := h'
        obtain ⟨κ₀, s₀, h₀, hle⟩ := hd κ R s hκ
        exact ⟨κ₀, s₀, h₀, S.le_trans _ _ _ (S.narrow_le s v) hle⟩
      · have : κ' - σ.length ≠ 0 := by omega
        simp [this] at h'
  | ctx _ _ ih => exact ih
  | ctxF _ _ ih => exact ih
  | _ => exact hd

/-- **Attenuation is monotone (audit R-7).** Every token in any store a run builds descends from the root store: the same
    kind as a root token, and a scope `⊑` its scope. -/
theorem attenuation_monotone {S σ₀ e tr σ' r} (hs : Steps S σ₀ e tr σ' r) : Descends S σ₀ σ' := by
  suffices ∀ σ, Descends S σ₀ σ → ∀ {e tr σ' r}, Steps S σ e tr σ' r → Descends S σ₀ σ' from
    this σ₀ (descends_refl S σ₀) hs
  intro σ hd e tr σ' r hs
  induction hs with
  | refl => exact hd
  | step hst _ ih => exact ih (step_descends hst hd)
  | fault hst => exact step_descends hst hd

/-- **Corollary (No-forgery).** In any configuration a well-typed program reaches from the root store, every token in
    the term is one the store holds, and it descends from a root token: the same kind, a scope `⊑` that token's. So the
    program's authority is bounded by `σ_root` for scopes, as Theorem 3 bounds it by `row(main)` for kinds. -/
theorem no_forgery {S σ₀ e τ ρ tr σ' e'} (hS : SigOk S) (h : HasType S σ₀ [] e τ ρ) (hs : Steps S σ₀ e tr σ' (.ok e')) :
    ∀ κ, κ ∈ e'.toks →
      ∃ (R : Kind) (s : Nat) (κ₀ : Nat) (s₀ : Nat), σ'[κ]? = some (R, s) ∧ σ₀[κ₀]? = some (R, s₀) ∧ S.le s s₀ := by
  obtain ⟨⟨_, h', _⟩, _⟩ := safety hS h hs
  intro κ hk
  obtain ⟨R, s, hκ⟩ := typed_toks h' κ hk
  obtain ⟨κ₀, s₀, h₀, hle⟩ := attenuation_monotone hs κ R s hκ
  exact ⟨R, s, κ₀, s₀, hκ, h₀, hle⟩

/-! ### The axioms each theorem rests on -/

#print axioms subst_typing
#print axioms preservation
#print axioms progress
#print axioms effect_soundness
#print axioms safety
#print axioms attenuation_monotone
#print axioms no_forgery

end DeluluSafety
