<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.11 Compilation — high-level surface, tiered backend

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.compilation.engines-agree`

The interpreter and the WASM backend produce the same observable behavior for any program both support; an unsupported construct falls back honestly rather than silently differing.

- **Enforced by:** `DL1201`, `DL1206`
- **Coverage:** covered
- **Note:** Parity is checked differentially, and a disagreement is compiler-bug class.

## `ref.rule.compilation.artifacts-carry-their-authority`

A compiled artifact carries its authority section; a missing, tampered, or unsupported one is refused rather than run.

- **Enforced by:** `DL1202`, `DL1204`
- **Coverage:** covered

