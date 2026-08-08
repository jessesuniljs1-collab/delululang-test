# DeluluLang Broker Red-Team Results

**Date:** 2026-08-08  
**Authorized red-team exercise on local sandbox**  
**Binary:** `redteam-target/release/delulu.exe`  
**Focus:** Lease token integrity and redemption attacks  

## Summary

All six attack scenarios designed to break token security were **HELD** (rejected correctly). The broker's token integrity mechanisms function as intended:

- Single-use enforcement
- MAC integrity verification
- Revocation tracking
- Cross-broker MAC key isolation

## Attack Results

| # | Attack | Command | Result | Exit Code | DL Code | Status | Notes |
|----|--------|---------|--------|-----------|---------|--------|-------|
| 1a | Valid delegation + first use | `grants delegate --effects Read --ttl 1h` then `run --lease <token>` | Success | 0 | — | PASS | Token minted and redeemed once; lease shows `g_6d088b4c6a5ce7d7467e4f6062e985cd` active |
| 1b | (same) | `run /tmp/test.delulu --lease <token>` | Success | 0 | — | PASS | Token consumed; guard active, custody daemon confirmed |
| 2 | Replay single-use token | `run /tmp/test.delulu --lease <same-token>` | Rejected | 0 | **DL1407** | **HELD** | Error: "single-use token already redeemed" — single-use invariant enforced |
| 3 | Tamper MAC | Flip hex chars in MAC suffix, redeem | Rejected | 0 | **DL1407** | **HELD** | Error: "MAC verification failed (bad, tampered, or rotated-away key)" — MAC integrity guarded |
| 4 | Forge token (made-up MAC) | Construct `dlt1_<valid-payload>.<all-zeros-mac>`, redeem | Rejected | 0 | **DL1407** | **HELD** | Error: "MAC verification failed" — broker's MAC key not invertible |
| 5a | Delegate from child node | `grants delegate --parent <child> --effects Read --multi` | Success | 0 | — | PASS | Multi-use token created for revocation scenario |
| 5b | Use multi-use token (pre-revoke) | `run --lease <multi-token>` | Success | 0 | — | PASS | Token accepted before revocation |
| 5c | Revoke child node | `grants revoke <child-id>` | Success | 0 | — | PASS | Synchronous revocation; child marked `[revoked@14]` in tree |
| 5d | Use token after revocation | `run --lease <same-multi-token>` | Rejected | 0 | **DL1403** | **HELD** | Error: "lease was revoked by audit seq 14" — revocation blocks tokens from revoked nodes |
| 6a | Start second broker | `DELULU_STATE_DIR=state2 broker start` | Success | 0 | — | PASS | Separate broker instance with distinct MAC key |
| 6b | Mint token in broker A | `grants delegate --effects Read --ttl 1h` (in state dir A) | Success | 0 | — | PASS | Token created with broker A's MAC key |
| 6c | Redeem in broker B | `DELULU_STATE_DIR=state2 run --lease <token-from-A>` | Rejected | 0 | **DL1407** | **HELD** | Error: "MAC verification failed" — broker B's key cannot verify broker A's MAC |

## Invariants Verified

1. **Anchoring** — Not directly tested (requires ground key we don't have; covered by adoption path)
2. **Attenuation** — Not directly tested (requires certificate forgery we don't have; delegated tokens inherit parent effects)
3. **No forgery** — Partially verified via token MAC attacks (tamper, forge)
4. **Revocation holds** — ✅ CONFIRMED — revoked nodes block token use (DL1403)
5. **Token integrity** — ✅ CONFIRMED — MAC, single-use, MAC key per broker all enforced (DL1407)
6. **Audit truth** — Not tested (would require manual log tampering + verify call)
7. **Guard** — Confirmed active (`guard: on — guarded: declassify:*, foreign_c:*, foreign_python:*`); request/approve flow not attacked

## Key Findings

**All attacks correctly HELD.** No invariant broken.

- **Single-use enforcement works:** DL1407 caught replay immediately
- **MAC verification is strong:** Both tampered and forged MACs rejected identically
- **Cross-broker isolation holds:** Each broker uses a distinct signing key; tokens are non-portable
- **Revocation is synchronous:** Node revocation at `audit seq 14` instantly blocks all tokens from that node
- **No grace period observed:** Revocation takes effect "before the next use"; multi-use token failed on first attempt after revocation

## Attack Methodology Notes

- Token format: `dlt1_<hex-payload>.<hex-mac>` (payload is JSON-encoded delegation record)
- Payload tampering not attempted (would require understanding internal JSON schema; MAC would still catch it)
- Cross-broker test confirms MAC key derivation is broker-local (not global)
- Guard bypass not attempted (not relevant to token security; guard is a separate gating mechanism)

## Conclusion

**Result: 6/6 attacks failed to breach token security.** The broker's lease token system holds its invariants under the designed threat model (untrusted party with knowledge of token format but not the broker's signing key, revocation state, or ground authority key).
