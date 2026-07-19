<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.10 Async — an effect, not a second system

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.async.is-an-effect-not-a-colour`

Asynchrony is expressed in the effect row, not by a parallel universe of `async` function types. There is no function-colouring split.

- **Enforced by:** `DL0106`
- **Coverage:** covered
- **Note:** `async` and `await` are reserved words with no grammar — declaring one is refused, so a second calling convention cannot be introduced by user code. Concurrency is reached through actors (§5.9), whose sends are already asynchronous.

