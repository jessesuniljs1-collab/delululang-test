<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.13 Portability — one portable target

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.portability.one-portable-target`

WASM is the portable target. A construct the backend cannot express is reported, and the program runs on the interpreter — never silently degraded.

- **Enforced by:** `DL1201`
- **Coverage:** covered

## `ref.rule.portability.isolation-labels-are-honest`

An isolation profile unavailable on the host is refused or labelled as a weaker fallback; the label always states what was actually used.

- **Enforced by:** `DL1408`
- **Coverage:** covered
- **Note:** Naming a profile you did not get is the failure mode this rule exists to prevent.

