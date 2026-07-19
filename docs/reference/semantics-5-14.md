<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_conform::rules::RULES` (enforcement cross-checked against the diagnostic registry).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# §5.14 Security — defense in depth

Each statement below is normative and carries a stable anchor id that conformance metadata can cite. **Enforced by** names the diagnostics that make the rule bite; a rule counts as covered only when *every* one of them has both an accepting and a rejecting witness.

## `ref.rule.security.fail-closed`

When custody cannot be established — broker unreachable, protocol mismatch, token invalid — the answer is refusal, never a default-allow.

- **Enforced by:** `DL1401`, `DL1406`, `DL1407`
- **Coverage:** covered
- **Note:** The skip branch is where security rules die; every 'cannot tell' path refuses.

## `ref.rule.security.revocation-is-immediate-and-final`

A revoked or expired grant fails at the next use and carries the revoking audit sequence; a reloaded plugin gets a fresh node, never the old one.

- **Enforced by:** `DL0801`, `DL1402`, `DL1403`
- **Coverage:** covered

## `ref.rule.security.the-human-holds-the-keys`

Guarded authority requires a human decision. A pending request blocks, a denial carries the principal's words verbatim, and a sealed refusal cannot be lifted by bypass.

- **Enforced by:** `DL1410`, `DL1411`, `DL1412`, `DL1413`, `DL1414`
- **Coverage:** covered

## `ref.rule.security.audit-chain-is-verifiable`

The audit log is a hash chain; a corrupted or reordered record fails verification.

- **Enforced by:** `DL1405`
- **Coverage:** covered

## `ref.rule.security.signatures-verify-locally`

Signature verification happens on the client. A present-but-invalid signature is a different, louder failure than an absent one.

- **Enforced by:** `DL1510`, `DL1511`, `DL1705`
- **Coverage:** covered
- **Note:** Distinguishing 'tampered' from 'unsigned' matters: they call for different responses.

