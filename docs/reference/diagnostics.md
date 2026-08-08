<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_diag::REGISTRY` (the diagnostic code registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# Diagnostics

Every diagnostic code, with its title and its conformance status.

A code's **rejecting** witness proves it fires when it should; its **accepting** witness proves it does *not* fire on valid input. A diagnostic that always fires is as broken as one that never fires, so invariant 42 requires both.

> **Coverage (invariant 42):** 151 of 151 anchors in this chapter have both an accepting and a rejecting conformance witness (100.0%). Items marked otherwise are **not stable** until witnessed — see `STAGE9_BUILD_ORDER.md` D10.

Codes marked other than `covered` are **not stable**: the classification of every such code — shadowed by a more general code, producible but untested, or unproducible by construction — is in `docs/design/STAGE9_BUILD_ORDER.md` D10.

| Code | Title | Anchor | Coverage |
|---|---|---|---|
| `DL0101` | unexpected character | `ref.diag.DL0101` | covered |
| `DL0102` | unterminated string literal | `ref.diag.DL0102` | covered |
| `DL0103` | invalid escape sequence | `ref.diag.DL0103` | covered |
| `DL0104` | invalid numeric literal | `ref.diag.DL0104` | covered |
| `DL0105` | unterminated block comment | `ref.diag.DL0105` | covered |
| `DL0106` | reserved word used as a declared name | `ref.diag.DL0106` | covered |
| `DL0107` | bidirectional control character in source | `ref.diag.DL0107` | covered |
| `DL0108` | line-break-like character in source (renders as a new line, does not act as one) | `ref.diag.DL0108` | covered |
| `DL0201` | expected a different token | `ref.diag.DL0201` | covered |
| `DL0202` | expected an expression | `ref.diag.DL0202` | covered |
| `DL0203` | expected a type | `ref.diag.DL0203` | covered |
| `DL0204` | file must begin with a `module` declaration | `ref.diag.DL0204` | covered |
| `DL0205` | expected a pattern | `ref.diag.DL0205` | covered |
| `DL0206` | comparison operators are non-associative | `ref.diag.DL0206` | covered |
| `DL0207` | invalid assignment target | `ref.diag.DL0207` | covered |
| `DL0208` | expected an item | `ref.diag.DL0208` | covered |
| `DL0209` | expected a statement terminator | `ref.diag.DL0209` | covered |
| `DL0210` | expression nests too deeply | `ref.diag.DL0210` | covered |
| `DL0211` | type nests too deeply | `ref.diag.DL0211` | covered |
| `DL0301` | unknown name | `ref.diag.DL0301` | covered |
| `DL0302` | duplicate definition | `ref.diag.DL0302` | covered |
| `DL0303` | unknown module in import | `ref.diag.DL0303` | covered |
| `DL0304` | a cycle in the declaration graph (imports, or type aliases) | `ref.diag.DL0304` | covered |
| `DL0305` | module-level mutable state is forbidden | `ref.diag.DL0305` | covered |
| `DL0306` | unknown effect name | `ref.diag.DL0306` | covered |
| `DL0307` | not a capability resource kind | `ref.diag.DL0307` | covered |
| `DL0401` | type mismatch | `ref.diag.DL0401` | covered |
| `DL0402` | mixed numeric types (no implicit coercion) | `ref.diag.DL0402` | covered |
| `DL0403` | wrong number of arguments | `ref.diag.DL0403` | covered |
| `DL0404` | not callable | `ref.diag.DL0404` | covered |
| `DL0405` | unknown field or method | `ref.diag.DL0405` | covered |
| `DL0406` | wrong number of type arguments | `ref.diag.DL0406` | covered |
| `DL0407` | non-exhaustive match | `ref.diag.DL0407` | covered |
| `DL0408` | condition must be Bool | `ref.diag.DL0408` | covered |
| `DL0409` | `?` requires Result in a Result-returning function | `ref.diag.DL0409` | covered |
| `DL0410` | generic variable used as both type and effect row | `ref.diag.DL0410` | covered |
| `DL0411` | `for` iterates something that is not a list | `ref.diag.DL0411` | covered |
| `DL0412` | `break` or `continue` outside a loop | `ref.diag.DL0412` | covered |
| `DL0501` | function performs an effect not declared in its row | `ref.diag.DL0501` | covered |
| `DL0502` | declared effect never performed | `ref.diag.DL0502` | covered |
| `DL0504` | conflicting bindings for row variable (rows never union-merge) | `ref.diag.DL0504` | covered |
| `DL0601` | capability type cannot be constructed or forged | `ref.diag.DL0601` | covered |
| `DL0602` | secret value cannot flow here (Secret[T] is not T) | `ref.diag.DL0602` | covered |
| `DL0603` | Secret.map requires a pure function | `ref.diag.DL0603` | covered |
| `DL0604` | opaque type cannot be stringified or serialized | `ref.diag.DL0604` | covered |
| `DL0605` | opaque type has no structural equality | `ref.diag.DL0605` | covered |
| `DL0701` | main's effect row exceeds the authority manifest | `ref.diag.DL0701` | covered |
| `DL0703` | root slice not granted | `ref.diag.DL0703` | covered |
| `DL0801` | call through revoked plugin reference | `ref.diag.DL0801` | covered |
| `DL0802` | grant exceeds holder's grant (attenuation violation) | `ref.diag.DL0802` | covered |
| `DL0803` | function-typed argument to Contained plugin export | `ref.diag.DL0803` | covered |
| `DL1001` | dependency authority exceeds its pin | `ref.diag.DL1001` | covered |
| `DL1002` | locked authority hash mismatch (same version, different authority) | `ref.diag.DL1002` | covered |
| `DL1003` | semver-authority violation: authority widened without a major version bump | `ref.diag.DL1003` | covered |
| `DL1004` | malformed package manifest | `ref.diag.DL1004` | covered |
| `DL1005` | package or re-export cycle | `ref.diag.DL1005` | covered |
| `DL1006` | import is ambiguous between a local module and a dependency | `ref.diag.DL1006` | covered |
| `DL1007` | git dependency without a pinned rev or tag | `ref.diag.DL1007` | covered |
| `DL1008` | version conflict for one package name in the graph | `ref.diag.DL1008` | covered |
| `DL1009` | package performs an effect not permitted by its own authority manifest | `ref.diag.DL1009` | covered |
| `DL1010` | content hash mismatch (source changed under a locked version) | `ref.diag.DL1010` | covered |
| `DL1011` | locked build requires resolution not present in delulu.lock | `ref.diag.DL1011` | covered |
| `DL1101` | effect-trace assertion violation (a runtime effect not in the static row) | `ref.diag.DL1101` | covered |
| `DL1102` | repair did not produce an accepting program (fuzz harness) | `ref.diag.DL1102` | covered |
| `DL1201` | construct not supported by the WASM backend (runs on the interpreter instead) | `ref.diag.DL1201` | covered |
| `DL1202` | artifact missing or invalid `delulu:authority` section | `ref.diag.DL1202` | covered |
| `DL1204` | delulu:cap interface version unsupported by this toolchain | `ref.diag.DL1204` | covered |
| `DL1205` | secret contents cannot enter the WASM guest (`--target wasm`) | `ref.diag.DL1205` | covered |
| `DL1206` | engine parity self-check failure (compiler-bug class) | `ref.diag.DL1206` | covered |
| `DL1301` | unmarshallable type in a foreign signature (incl. Secret/opaque) | `ref.diag.DL1301` | covered |
| `DL1302` | function-typed value crossing the foreign boundary (no callbacks — rule R-6a) | `ref.diag.DL1302` | covered |
| `DL1303` | foreign lib used without a manifest entry or runtime grant | `ref.diag.DL1303` | covered |
| `DL1304` | foreign symbol not found at bind time | `ref.diag.DL1304` | covered |
| `DL1305` | python import not in the granted allowlist | `ref.diag.DL1305` | covered |
| `DL1306` | foreign return failed shape validation (encoding or size) | `ref.diag.DL1306` | covered |
| `DL1307` | python runtime unavailable | `ref.diag.DL1307` | covered |
| `DL1308` | unsupported ABI string in a `foreign` block | `ref.diag.DL1308` | covered |
| `DL1401` | broker unreachable / protocol failure (fail closed) | `ref.diag.DL1401` | covered |
| `DL1402` | lease expired (TTL) | `ref.diag.DL1402` | covered |
| `DL1403` | lease revoked (carries the revoking audit seq) | `ref.diag.DL1403` | covered |
| `DL1405` | audit chain verification failure | `ref.diag.DL1405` | covered |
| `DL1406` | broker protocol version mismatch | `ref.diag.DL1406` | covered |
| `DL1407` | delegation token invalid or already redeemed | `ref.diag.DL1407` | covered |
| `DL1408` | isolation profile unavailable on this platform | `ref.diag.DL1408` | covered |
| `DL1409` | foreign worker died (process isolation) — the isolated worker crashed; the host survived | `ref.diag.DL1409` | covered |
| `DL1410` | guard refusal: guarded authority, no permit (names the exact guard request command) | `ref.diag.DL1410` | covered |
| `DL1411` | guard request pending (carries the request id) | `ref.diag.DL1411` | covered |
| `DL1412` | guard request denied (carries the principal's comment verbatim) | `ref.diag.DL1412` | covered |
| `DL1413` | guard sealed refusal — not runtime-approvable; bypass does not lift it | `ref.diag.DL1413` | covered |
| `DL1414` | guard owner code missing or invalid — admin verb refused | `ref.diag.DL1414` | covered |
| `DL1415` | grant certificate chain does not verify to a configured trust anchor | `ref.diag.DL1415` | covered |
| `DL1416` | grant certificate hop is not an attenuation of its issuer (carries the intersection) | `ref.diag.DL1416` | covered |
| `DL1417` | grant certificate is outside its validity window | `ref.diag.DL1417` | covered |
| `DL1418` | grant certificate refused: unsupported algorithm/dimension, or malformed | `ref.diag.DL1418` | covered |
| `DL1501` | manifest export signature does not match the plugin code (the manifest never overrides the code) | `ref.diag.DL1501` | covered |
| `DL1502` | requested grant exceeds the plugin's declared ceiling | `ref.diag.DL1502` | covered |
| `DL1503` | DIR version unsupported (rebuild the plugin) | `ref.diag.DL1503` | covered |
| `DL1504` | Verified re-check failed (plugin code unsound or stale) — never falls back to Contained | `ref.diag.DL1504` | covered |
| `DL1505` | Contained module imports outside its grant slice | `ref.diag.DL1505` | covered |
| `DL1506` | plugin resource limit exceeded (plugin terminated and its grant node revoked) | `ref.diag.DL1506` | covered |
| `DL1507` | plugin API version mismatch (rebuild the plugin) | `ref.diag.DL1507` | covered |
| `DL1508` | malformed or tampered `.dpx` container (not a valid plugin artifact) | `ref.diag.DL1508` | covered |
| `DL1509` | a Contained plugin export's signature must be concrete at the `get` site (R-6a is otherwise undecidable) | `ref.diag.DL1509` | covered |
| `DL1510` | plugin signature present but invalid (tampered content, wrong key, or malformed signature) | `ref.diag.DL1510` | covered |
| `DL1511` | artifact is unsigned — it carries no signature at all (a DIFFERENT fault from one that fails to verify, DL1510/DL1705) | `ref.diag.DL1511` | covered |
| `DL1601` | non-sendable value crossing an actor boundary (incl. unconsumed iso; incl. PyObj pinning) | `ref.diag.DL1601` | covered |
| `DL1602` | use of a binding after `consume` | `ref.diag.DL1602` | covered |
| `DL1603` | alias violates a reference-capability deny property (incl. a `val` closure over a `ref` capture) | `ref.diag.DL1603` | covered |
| `DL1604` | access denied by viewpoint/receiver capability (write via box; field via tag; sync call on tag) | `ref.diag.DL1604` | covered |
| `DL1605` | recover block references a non-sendable outer binding | `ref.diag.DL1605` | covered |
| `DL1606` | a behavior declares a return type (behaviors yield Unit at the send site) | `ref.diag.DL1606` | covered |
| `DL1607` | reference capability invalid for this type (e.g. non-tag on an actor type) | `ref.diag.DL1607` | covered |
| `DL1608` | identifier collides with a v0.7 keyword (`consume`/`recover`) | `ref.diag.DL1608` | covered |
| `DL1610` | debug race-checker violation (compiler-bug class — file a bug) | `ref.diag.DL1610` | covered |
| `DL1701` | LSP/workspace configuration error | `ref.diag.DL1701` | covered |
| `DL1702` | formatter identity/idempotence violation (compiler-bug class — file a bug) | `ref.diag.DL1702` | covered |
| `DL1703` | test authority exceeds the package test ceiling | `ref.diag.DL1703` | covered |
| `DL1704` | catalog invalid: unknown key/placeholder, bad meta, or a welcome-override attempt — entry falls back to en-US | `ref.diag.DL1704` | covered |
| `DL1705` | signature verification failed | `ref.diag.DL1705` | covered |
| `DL1706` | registry index line invalid / semver-authority conflict at publish | `ref.diag.DL1706` | covered |
| `DL1707` | assertion failed (a `test` assertion did not hold at runtime) | `ref.diag.DL1707` | covered |
| `DL1710` | morph is not bijective (a keyword renamed twice, or two keywords sharing one alias) | `ref.diag.DL1710` | covered |
| `DL1711` | morph alias is another keyword's canonical spelling (the surface would mislead) | `ref.diag.DL1711` | covered |
| `DL1712` | morph alias is not a single valid token | `ref.diag.DL1712` | covered |
| `DL1713` | morph renames something that is not a renameable keyword | `ref.diag.DL1713` | covered |
| `DL1714` | the requested morph is not available | `ref.diag.DL1714` | covered |
| `DL1715` | the program uses one of the target morph's aliases as a name | `ref.diag.DL1715` | covered |
| `DL1780` | atlas refused: the program has check errors — fix them first (no partial graph) | `ref.diag.DL1780` | covered |
| `DL1781` | custody overlay unavailable — the broker daemon is not reachable; atlas emitted without it | `ref.diag.DL1781` | covered |
| `DL1790` | invalid theme name or malformed theme.toml — using the `default` theme | `ref.diag.DL1790` | covered |
| `DL1801` | use of a deprecated feature (RFC-linked) | `ref.diag.DL1801` | covered |
| `DL1802` | package declares a newer language edition than this toolchain | `ref.diag.DL1802` | covered |
| `DL1901` | unknown attribute | `ref.diag.DL1901` | covered |
| `DL1902` | mailbox overflow dropped a message under `drop-new` in abort mode | `ref.diag.DL1902` | covered |
| `DL1903` | dependency version has a published security advisory | `ref.diag.DL1903` | covered |
| `DL1904` | actuator command refused by its envelope | `ref.diag.DL1904` | covered |
| `DL1905` | hardware actuation requested for an artifact that no simulation approved | `ref.diag.DL1905` | covered |
| `DL1906` | native-code emission requested without the `exec.native` grant | `ref.diag.DL1906` | covered |
| `DL1907` | compute dispatch refused by the device envelope | `ref.diag.DL1907` | covered |
| `DL1908` | signature policy requires hybrid; artifact is classical-only or names an unknown algorithm | `ref.diag.DL1908` | covered |
| `DL1909` | deploy plan authority exceeds environment profile | `ref.diag.DL1909` | covered |
| `DL1910` | unvalidated (pre-KAT, unaudited) cryptography invoked without `--unstable` | `ref.diag.DL1910` | covered |
| `DL1911` | compute adapter cannot attest independent below-adapter envelope enforcement | `ref.diag.DL1911` | covered |
| `DL1912` | kernel artifact is malformed, unreadable, or its signature does not verify | `ref.diag.DL1912` | covered |
| `DL1913` | kernel artifact is unsigned (kernels are always signed — spec §7.1) | `ref.diag.DL1913` | covered |
| `DL0901` | integer overflow | `ref.diag.DL0901` | covered |
| `DL0902` | division by zero | `ref.diag.DL0902` | covered |
| `DL0903` | index out of bounds | `ref.diag.DL0903` | covered |
| `DL0904` | capability scope violation | `ref.diag.DL0904` | covered |
| `DL0905` | recursion depth exceeded | `ref.diag.DL0905` | covered |
| `DL0907` | an internal invariant the checker should have guaranteed was violated (compiler-bug class) | `ref.diag.DL0907` | covered |

## Codes this compiler cannot emit

These `DLxxxx` are named somewhere in this repository but are **not** in the table above. Each carries a recorded disposition, so a code you cannot find is an answered question rather than a dead end — `delulu explain <code>` prints the reason, and a code in neither table is reported by the Survey as unexplained.

`specified-not-implemented` marks a **real gap**: a specification names the code and nothing raises it. Those are kept visible rather than closed by inventing a code or editing the specification to match.

| Code | Disposition | Why |
|---|---|---|
| `DL0503` | `retired` | It named an arity rule the checker does not have: a multi-row-variable signature is legal, row honesty is enforced per variable by DL0501, and a multi-variable row term is refused at resolution by DL0306. Retired rather than frozen unreachable, so the coverage law never has to carry a code nothing can emit (ruling D22). |
| `DL0702` | `retired` | One of three codes that could never fire, retired at the 1.0 coverage gate rather than frozen unreachable (ruling D22). See `crates/delulu-conform/src/tests.rs`. |
| `DL0906` | `retired` | One of three codes that could never fire, retired at the 1.0 coverage gate rather than frozen unreachable (ruling D22). See `crates/delulu-conform/src/tests.rs`. |
| `DL1012` | `specified-not-implemented` | `docs/design/STAGE2_SPECIFICATION.md` rule VIS-1 says that referencing an item which is not cross-package-visible is DL1012. No such code is allocated and nothing emits it. Recorded as an open gap rather than closed by inventing a code the checker does not raise or by editing the specification to match the implementation. |
| `DL1203` | `specified-not-implemented` | `docs/design/STAGE3_SPECIFICATION.md` tabulates DL1203 for an artifact hash/receipt conflict (a receipt exists but the hash differs) with `requires_human: true`. No such code is allocated. Recorded as an open gap; a tamper-shaped condition is worth keeping visible rather than silently dropping from the specification. |
| `DL1404` | `never-allocated` | The DL14xx custody range skips it deliberately (spec §8), and the comment above the range in this file says so. It is the original of the house rule the others below mirror. Do not invent one. |
| `DL1419` | `never-allocated` | RFC 0001 penciled DL1419/DL1420 in for uplink-lease expiry. Not adding them was the better answer: an expired uplink is the ordinary DL1402 and a bad bundle the existing DL1405, so a parallel expiry path would have been a second place for liveness to be wrong (`docs/design/STAGE10_BUILD_ORDER.md`). |
| `DL1420` | `never-allocated` | The other half of the DL1419/DL1420 pair RFC 0001 penciled in and this project deliberately did not allocate (`docs/design/STAGE10_BUILD_ORDER.md`). |
| `DL1609` | `never-allocated` | The Stage-7 actor table skips it, mirroring the DL1404 house rule. The comment above the DL16xx range in this file says so. Do not invent one. |
| `DL1708` | `never-allocated` | The morph loader was given its own DL1710–DL1714 sub-block instead of continuing the DL170x run, so that a reader seeing a DL171x knows immediately which subsystem raised it. This code and the one after it were both left unallocated. |
| `DL1709` | `never-allocated` | The other half of the pair skipped when the morph loader was given its own DL1710–DL1714 sub-block. See DL1708. |
| `DL1784` | `never-allocated` | Never allocated in the Surface addendum's DL178x block, mirroring the DL1404 house rule. The comment above the DL17xx range in this file says so. |
| `DL9999` | `sentinel` | Not a diagnostic at all. It is the deliberately-invalid code the registry guard's own test asserts is absent (`crates/delulu-conform/src/rules.rs`), and the one `cli_contract` hands to `delulu explain` to prove it refuses what it does not know. Both uses depend on it staying unrecognised, so it is recorded here and deliberately left unexplainable. |
