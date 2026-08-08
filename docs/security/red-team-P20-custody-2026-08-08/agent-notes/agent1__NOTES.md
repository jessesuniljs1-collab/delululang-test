# Red-team notes — agent1 (specialty: certificates & adoption)

Binary: `delulu.exe` (v1.0.0). State dir: `agent1\state`. Anchor pubkey (real):
`31feca86a431ae340059831278232c011daefc57dfd2dfd38a2b8e45e1304035`. Vehicle pubkey:
`44649c760bf59d76d066dd15f23a185958b1d05c0dbf708c36a602816dc8cd37`. `legit.dlcert` fingerprint:
`c6968234819f8539191a009236da4b0596720eacb2f895fd9690b0ff18b8c368`.

## Summary table

| # | Attack | Result | Code / evidence |
|---|--------|--------|------------------|
| 1 | Adopt `legit.dlcert` with `--anchor` = vehicle.pub (wrong anchor) | **HELD** | DL1415, exit 1: "issuer `31feca86…` is not a configured trust anchor" |
| 2 | Adopt `legit.dlcert` with no `--anchor` at all | **HELD** | usage error, exit 2: "no trust anchors… Pass --anchor \<hex\>" |
| 3 | Mint self-anchored cert (issuer=vehicle, `parent: anchor`, signed by `vehicle.key`, effects `Actuate,Net`), adopt with the **real** anchor | **HELD** | mint succeeds (offline minting is unrestricted, as expected); adopt → DL1415, exit 1: "issuer `44649c76…` is not a configured trust anchor" |
| 4 | Mint a **widened** child of `legit.dlcert` (`Actuate,Net`, slew −90..90) via `--parent-cert legit.dlcert`, then adopt `[legit, child]` | **HELD** | refused at **mint time**: DL1416, exit 2: "this certificate asks for more than --parent-cert holds… requested: effects={Net,Actuate} device=[…-90..90…] most it can carry: effects={Actuate} device=[…-45..45…]". No child file was ever produced, so the follow-on adopt had nothing to submit. |
| 5 | Hand-edit `legit.dlcert`'s `authority:` line (add `Net`, widen slew to −90..90), keep the original `sig:`, adopt as `tampered.dlcert` | **HELD** | DL1415, exit 1: "certificate 0's signature does not verify over its own bytes" |
| 6 | **Revocation evasion**: adopt `legit.dlcert` → revoke it → mint a same-bounds (non-widened) extension `ext.dlcert` via `--parent-cert legit.dlcert` → adopt `[legit.dlcert, ext.dlcert]` | **BROKE** | Re-adoption **succeeded** (exit 0) and produced a **new live node** carrying the identical `{Actuate}` / `sat0/hga slew −45..45` authority that had just been revoked. See full reproduction below. |

**5 of 6 held. Attack 6 (revocation evasion) broke the invariant** — this is the finding.

---

## Full reproduction of the break (Attack 6)

```
$anchor  = 31feca86a431ae340059831278232c011daefc57dfd2dfd38a2b8e45e1304035
$vehicle = 44649c760bf59d76d066dd15f23a185958b1d05c0dbf708c36a602816dc8cd37

# 1. Adopt the legitimate cert
delulu grants adopt bundle\legit.dlcert --anchor $anchor --json
# -> node g_2c688250cec84981329a13e0ac7614e3, state=live,
#    fingerprint c6968234819f8539191a009236da4b0596720eacb2f895fd9690b0ff18b8c368
#    effects={Actuate} device=[sat0/hga:slew_deg=-45..45,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park]

# 2. Revoke it
delulu grants revoke g_2c688250cec84981329a13e0ac7614e3 --json
# -> {"newly_revoked":["g_2c688250cec84981329a13e0ac7614e3"], "by_seq":3, ...}
# `grants tree` now correctly shows: g_2c688250cec84981329a13e0ac7614e3 [revoked@3] ...

# 3. Mint a pass-through extension — SAME effects, SAME device bounds as legit.dlcert
#    (attenuation is respected: nothing wider is requested)
delulu grants certify --subject $vehicle --effects Actuate \
  --device "sat0/hga:slew_deg=-45..45,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park" \
  --ttl 20m --key bundle\vehicle.key --parent-cert bundle\legit.dlcert --out ext.dlcert
# -> mints fine (fingerprint 585b174186a39a71df22fb744e8e3e0f7cd8b82055f4e9587f7011d1bbd50210)

# 4. Re-adopt the chain [legit.dlcert, ext.dlcert] against the REAL anchor
delulu grants adopt bundle\legit.dlcert ext.dlcert --anchor $anchor --json
# -> exit 0:
# {"command":"grants adopt","fingerprint":"585b1741...","node":"g_9b0f928c5de5b3d3897ecbbc0ff1ed48", ...}

# 5. Confirm
delulu grants tree --json
# -> g_2c688250cec84981329a13e0ac7614e3 [revoked@3] effects={Actuate} device=[...-45..45...]
# -> g_9b0f928c5de5b3d3897ecbbc0ff1ed48 [live]     effects={Actuate} device=[...-45..45...]   <-- LIVE, same authority
```

`grants list --json` confirms the second node's `state` is `"live"` with `by_seq: null` (never revoked) — this
is not a propagation-delay artifact of the revoke call (whose own response says revocation takes effect
"synchronous class — before the next use" / within one epoch ≤ 50ms); the new node was created by an adopt
call that ran and returned **after** the revoke call had already completed and been confirmed in the tree.

### Why this works
The vehicle still legitimately possesses `legit.dlcert` (the operator-signed cert is not secret — the vehicle
was handed it) and `vehicle.key`. Revoking the **adopted node** clearly marks that node `[revoked@N]`, but
adopt-time validation of a *new* submission does not appear to check whether any certificate in the submitted
chain — here `legit.dlcert` itself, reused byte-for-byte — was already part of a chain that produced a
now-revoked node. The two adoptions got different identifiers (first keyed to `legit.dlcert`'s own fingerprint
`c6968234…`, second keyed to the new leaf `ext.dlcert`'s fingerprint `585b1741…`), so whatever revoked-set
tracking exists evidently keys on the leaf/node identity rather than on any ancestor certificate's fingerprint
appearing in a newly-submitted chain. No widening, no forgery, and no anchor substitution were needed —
attenuation (invariant 2), forgery-detection (invariant 3), and anchoring (invariant 1) all held throughout;
only revocation-persistence (invariant 4) failed, and it failed via exactly the "extend with a fresh
self-signed child and re-adopt `[legit, child]`" path the task brief called out as the key test.

**Note:** per the task brief this was expected to be *fixed* ("a recent fix retires the whole chain on
revoke"). On this build, that fix either is not present or does not cover this exact path (reusing the
identical parent cert file rather than a re-minted one). Recommend: track revoked state by certificate
fingerprint (not just adopted-node id) and refuse adoption of any chain containing a fingerprint that has
ever been part of a revoked node, or equivalently persist revocation against the `(issuer, subject, nonce)`
identity so any chain reusing that exact operator-signed cert is refused regardless of what is appended to it.

---

## Attacks that held — brief detail

**Attack 1 & 2 (anchoring):** The broker never treats an unconfigured or wrong pubkey as trustworthy just
because a cert parses cleanly. With no `--anchor` at all it refuses outright rather than silently trusting
nothing-in-particular. Straightforward invariant-1 hold.

**Attack 3 (self-anchored mint):** Minting is unrestricted offline (as expected — anyone can *write* cert
bytes), but the cert's cryptographic `issuer` is derived from the signing key, not from the `parent: anchor`
label. Signing with `vehicle.key` makes `issuer = vehicle.pub` no matter what the `parent` field claims, and
that issuer is checked against the configured anchor set at adopt time. Same DL1415 as attack 1.

**Attack 4 (widening):** Notable — this was refused by `grants certify` itself (DL1416) at mint time, before
adoption was even attempted, because `--parent-cert` was supplied and the tool computed the requested
authority exceeds what the parent cert can carry. This is a client-side guardrail in the honest CLI tool;
I did not have an independent raw ed25519-signing path to test whether the **broker/adopt side** would
separately re-derive and reject an over-wide chain if a non-standard tool produced a validly-signed,
correctly-chained, but over-wide child cert (i.e., I can't fully rule out that attenuation is enforced
client-side only). Flagging as a residual scope gap, not a confirmed break — the forgery test (attack 5)
shows the signature covers `authority`, so any such attempt would at minimum require a validly re-signed
cert, which the provided CLI refuses to produce when told the true parent.

**Attack 5 (forgery):** Editing the `authority:` text field while keeping the old `sig:` was caught
immediately and specifically: "certificate 0's signature does not verify over its own bytes" — confirms the
signature covers the authority/scopes field, not just subject/issuer/expiry.

---

## Scope / cleanup
- Broker for this session (`agent1\state`) was stopped (`delulu broker stop`) after attack 6.
- Files created in `agent1\`: `forged.dlcert`, `tampered.dlcert`, `ext.dlcert`, plus the now-populated
  `agent1\state\` (contains node `g_2c688250cec84981329a13e0ac7614e3` revoked, `g_9b0f928c5de5b3d3897ecbbc0ff1ed48`
  live — left in place as evidence of the break, not cleaned up).
- Nothing outside `agent1\` was touched.
