<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.5 Modules — units of authority

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.modules.file-declares-its-module`

Every file begins with a `module` declaration.

- **Enforced by:** `DL0204`
- **Coverage:** covered

## `ref.rule.modules.no-mutable-module-state`

Module-level mutable state is forbidden: a module cannot hold a hidden channel between its functions.

- **Enforced by:** `DL0305`
- **Coverage:** covered
- **Note:** Shared mutable module state would be ambient authority in disguise.

## `ref.rule.modules.no-duplicate-definitions`

A name is defined at most once per module.

- **Enforced by:** `DL0302`
- **Coverage:** covered

