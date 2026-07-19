<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.3 Kind vs. scope — the honest granularity split

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.granularity.effect-kind-is-static`

The effect KIND (Read, Write, Net, Clock, Rand, Declassify, ForeignCall) is static and appears in the type. The SCOPE (which path, which host) is a runtime value carried by the capability.

- **Enforced by:** `DL0306`, `DL0307`
- **Coverage:** covered
- **Note:** The honest split: the type system proves *what kind* of thing a function can do; the capability's scope decides *which* resource. Claiming static path-level proof would be a lie the design refuses to tell.

## `ref.rule.granularity.scope-checked-at-use`

A capability's scope is enforced when the operation runs; escaping it is a runtime refusal, not a type error.

- **Enforced by:** `DL0904`, `DL0703`
- **Coverage:** covered

