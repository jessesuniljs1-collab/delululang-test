<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.1 Authority model — pure object-capability

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.authority.root-is-the-only-source`

All authority originates in the `Root` value passed to `main`. There is no ambient authority: no global, import, or constructor yields a capability.

- **Enforced by:** `DL0601`, `DL0301`
- **Coverage:** covered
- **Note:** A capability type has no literal syntax and no constructor; the only way to obtain one is to derive it from a `Root` you were handed.

## `ref.rule.authority.derivation-is-pure`

Deriving a capability from `Root` (`root.fs_read(path)`, `root.http(hosts)`, …) is a PURE operation carrying no effect. Holding authority is not using it.

- **Enforced by:** `DL0501`
- **Coverage:** covered
- **Note:** The effect appears when the capability is *used*, which is why a program that derives but never calls has an empty row.

## `ref.rule.authority.attenuation-is-monotone`

A derived capability is never wider than the one it came from. `narrow` may only shrink a scope; no operation widens one.

- **Enforced by:** `DL0802`
- **Coverage:** covered
- **Note:** The broker enforces the same law across process boundaries (audit rule R-7).

## `ref.rule.authority.no-forgery`

A capability value cannot be constructed, forged, cast, or deserialized into existence.

- **Enforced by:** `DL0601`, `DL0904`
- **Coverage:** covered
- **Note:** Statically there is no constructor; at the WASM boundary a forged handle is refused by the host (DL0904).

