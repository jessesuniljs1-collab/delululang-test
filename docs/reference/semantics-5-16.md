<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.16 The authority holder model

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.holder.grant-cannot-exceed-the-holder`

No grantee receives more authority than its grantor holds, at any depth of delegation.

- **Enforced by:** `DL0802`, `DL1502`
- **Coverage:** covered
- **Note:** Audit rule R-7. Composition is where attenuation systems usually leak, so the bound is checked at every level rather than only at the first.

## `ref.rule.holder.tests-hold-no-ambient-authority`

A test holds only the authority its declared row requires, bounded by the package's test ceiling; a pure test holds none.

- **Enforced by:** `DL1703`
- **Coverage:** covered
- **Note:** Invariant 41 — the test runner is a holder like any other, not a privileged context.

## `ref.rule.holder.plugin-manifest-never-overrides-code`

A plugin's manifest never widens what its code actually does; a mismatch is refused at build, and a failed re-check never falls back to a weaker class.

- **Enforced by:** `DL1501`, `DL1504`, `DL1509`
- **Coverage:** covered
- **Note:** Falling back to Contained on a failed Verified re-check would turn a broken proof into a silent downgrade.

