<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.15 Runtime guarantees

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.runtime.faults-are-diagnostics`

A runtime fault — overflow, division by zero, index out of bounds, recursion depth — is a diagnostic with a code, never a host crash or silent wrap.

- **Enforced by:** `DL0901`, `DL0902`, `DL0903`, `DL0905`
- **Coverage:** accepting only

## `ref.rule.runtime.trace-is-within-the-row`

Every effect performed at runtime lies within the statically declared row. A violation is compiler-bug class.

- **Enforced by:** `DL1101`
- **Coverage:** accepting only
- **Note:** This is the static claim checked against reality: `--assert-trace` makes the soundness argument falsifiable at runtime rather than only on paper.

## `ref.rule.runtime.plugin-limits-terminate-and-revoke`

A plugin that exceeds its resource limits is terminated and its grant node revoked — dead, not wounded.

- **Enforced by:** `DL1506`
- **Coverage:** covered

