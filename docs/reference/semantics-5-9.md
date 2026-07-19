<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.9 Concurrency — actors with reference capabilities

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.actors.only-sendable-values-cross`

Only sendable values cross an actor boundary. An unconsumed `iso`, an aliased mutable reference, or a pinned foreign object may not be sent.

- **Enforced by:** `DL1601`, `DL1605`
- **Coverage:** covered
- **Note:** This is data-race freedom by typing rather than by lock discipline.

## `ref.rule.actors.consume-is-final`

After `consume`, the original binding is dead; using it is a compile error.

- **Enforced by:** `DL1602`
- **Coverage:** covered

## `ref.rule.actors.viewpoint-adaptation`

A reference capability's deny properties are enforced at every access: no write through `box`, no field read through `tag`, no synchronous call on `tag`.

- **Enforced by:** `DL1603`, `DL1604`, `DL1607`
- **Coverage:** covered

## `ref.rule.actors.behaviors-return-unit`

A behavior yields `Unit` at the send site; it cannot declare a return type.

- **Enforced by:** `DL1606`
- **Coverage:** covered
- **Note:** A behavior is asynchronous, so a return value would require a hidden await.

