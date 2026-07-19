<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.4 Secrets and information flow

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.secrets.no-implicit-flow`

`Secret[T]` never coerces to `T`. A secret cannot flow into a position expecting a plain value.

- **Enforced by:** `DL0602`
- **Coverage:** covered

## `ref.rule.secrets.expose-requires-declassify`

`expose` is the only unwrap, it requires `Cap[Declassify]`, and it carries the `Declassify` effect — so declassification is visible in the authority report.

- **Enforced by:** `DL0501`
- **Coverage:** covered
- **Note:** Audit rule R-2: declassification is an effect, which is what stops a secret from leaving silently.

## `ref.rule.secrets.map-must-be-pure`

`Secret.map` requires a pure function, and its result stays `Secret`.

- **Enforced by:** `DL0603`
- **Coverage:** covered
- **Note:** An effectful mapper could exfiltrate the plaintext without ever calling `expose`.

## `ref.rule.secrets.opaque-has-no-observers`

An opaque value has no stringification, no serialization, and no structural equality — only constant-time `verify`.

- **Enforced by:** `DL0604`, `DL0605`
- **Coverage:** covered
- **Note:** Audit rule R-5. Structural equality would leak the contents a byte at a time.

## `ref.rule.secrets.never-cross-into-wasm`

Secret contents never enter the WASM guest.

- **Enforced by:** `DL1205`
- **Coverage:** covered

