<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.2 Effect system — effect rows in every function type

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.effects.declared-row`

A function may perform only the effects its row declares. Performing an undeclared effect is a compile error.

- **Enforced by:** `DL0501`
- **Coverage:** covered
- **Note:** This is the identity claim's load-bearing rule.

## `ref.rule.effects.rows-propagate-through-calls`

A caller's row must contain every callee's row. Effects compose upward through the call graph with no escape hatch.

- **Enforced by:** `DL0501`
- **Coverage:** covered
- **Note:** There is no `unsafe`, no dynamic dispatch that erases a row, and no reflection.

## `ref.rule.effects.declared-but-unperformed-warns`

Declaring an effect a function never performs is a warning, not an error: over-declaring is safe but dishonest about what the code does.

- **Enforced by:** `DL0502`
- **Coverage:** covered
- **Note:** A warning rather than an error, because a widened row is a *smaller* claim of trustworthiness — it never lets a program do more than it says.

## `ref.rule.effects.row-bindings-never-merge`

A row variable never union-merges two conflicting bindings, and a row expression carries at most one row-variable tail.

- **Enforced by:** `DL0504`
- **Coverage:** covered
- **Note:** Audit rule R-3's operational form. Rewritten at the 1.0 gate (Stage 9 ruling D22): the earlier statement claimed "at most one row variable per signature", an arity rule the checker does not have — a multi-row-variable signature is legal, and the probe on record shows it cannot launder (row honesty is enforced per variable by DL0501, and a multi-variable row TERM is refused at resolution, DL0306). DL0503, which named the arity rule, was retired rather than frozen unreachable.

## `ref.rule.effects.builtin-callbacks-compose`

A higher-order builtin (`List.map`, …) has the row of the callback it is given: a pure builtin cannot launder an effectful function into a pure context.

- **Enforced by:** `DL0501`
- **Coverage:** covered
- **Note:** Audit rule R-4 — the classic laundering hole, closed by making builtins row-polymorphic rather than pure.

## `ref.rule.effects.generic-var-is-type-or-row`

A generic variable is a type variable or a row variable, never both.

- **Enforced by:** `DL0410`
- **Coverage:** covered

