<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.7 Imports — access is not authority

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.imports.access-is-not-authority`

Importing a module grants the ability to CALL its functions, never the authority those functions need. The caller must still hold and pass the capability.

- **Enforced by:** `DL0501`
- **Coverage:** covered
- **Note:** The rule that makes dependency review tractable: a malicious import cannot act on its own, only ask.

## `ref.rule.imports.resolution-is-unambiguous`

An import resolves to exactly one module; local/dependency collisions and import cycles are refused.

- **Enforced by:** `DL0303`, `DL0304`, `DL1006`
- **Coverage:** covered

