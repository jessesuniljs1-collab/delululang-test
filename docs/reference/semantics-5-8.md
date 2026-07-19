<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.8 Errors — results, not exceptions

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.errors.results-not-exceptions`

Failure is a `Result` value. There are no exceptions, no unwinding across frames, and no continuations.

- **Enforced by:** `DL0301`
- **Coverage:** covered
- **Note:** There is no throw/catch syntax and no continuation capture: `throw e` is simply an unknown name. Runtime faults (DL09xx) terminate the program with a diagnostic rather than transferring control, so no construct moves control non-locally.

## `ref.rule.errors.try-requires-result-context`

`?` is only valid on a `Result` inside a function that itself returns `Result`.

- **Enforced by:** `DL0409`
- **Coverage:** covered

## `ref.rule.errors.match-is-exhaustive`

A `match` must cover every case.

- **Enforced by:** `DL0407`
- **Coverage:** covered
- **Note:** Exhaustiveness is what makes handling a new error variant a compile error rather than a silent fallthrough.

