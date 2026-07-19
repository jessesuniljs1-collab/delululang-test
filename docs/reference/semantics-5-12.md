<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.12 Interop — first-class Python and C, honestly fenced

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.interop.foreign-needs-declared-authority`

A foreign library is usable only with a manifest entry and a runtime grant, and every foreign call carries the `ForeignCall` effect.

- **Enforced by:** `DL1303`, `DL0501`
- **Coverage:** covered
- **Note:** Foreign code is outside the proof, so the authority report discloses it under a separate heading rather than pretending it is proven.

## `ref.rule.interop.no-callbacks-across-the-boundary`

A function-typed value never crosses the foreign boundary: unverifiable code must not hold a re-entry point into verified code.

- **Enforced by:** `DL1302`, `DL0803`
- **Coverage:** covered
- **Note:** Audit rule R-6a.

## `ref.rule.interop.marshalling-is-an-allowlist`

Only allowlisted types marshal across a foreign signature; secrets and opaque types never do, and a return value that fails shape validation is a fault, not a coerced value.

- **Enforced by:** `DL1301`, `DL1306`
- **Coverage:** covered

## `ref.rule.interop.python-imports-are-allowlisted`

A Python import outside the granted allowlist is refused, and the denied attempt is recorded in the trace.

- **Enforced by:** `DL1305`
- **Coverage:** covered
- **Note:** Refusals are traced, so an attempt to reach beyond the grant is visible evidence rather than a silent no-op.

