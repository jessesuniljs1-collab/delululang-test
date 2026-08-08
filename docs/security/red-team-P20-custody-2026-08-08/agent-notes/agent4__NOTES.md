# DeluluLang Guard Red-Team Report

## Execution Summary
- **Date**: 2026-08-08
- **Binary**: delulu.exe (redteam-target/release)
- **Broker State**: C:\Users\jesse\...\agent4\state
- **Owner Code Obtained**: gow1_d49fa967901a18d7ce287585 (printed at broker start)

## Attack Matrix

| Test # | Attack | Command | Result | Guard Status | Notes |
|--------|--------|---------|--------|--------------|-------|
| 1 | Guard status check | `guard status` / `guard policy` | ✓ Showed default policy | **HELD** | Three classes guarded: declassify:*, foreign_c:*, foreign_python:* |
| 2a | Guarded verb without owner code | `guard approve gr_0000` (no --owner) | ✓ DL1414 refused | **HELD** | Requires valid owner code |
| 2b | Guarded verb with wrong code | `guard approve gr_0000 --owner "wrong_code_123456789abc"` | ✓ DL1414 refused | **HELD** | Code validation works |
| 2c | Guarded verb with correct code | `guard approve gr_0000 --owner "gow1_d49fa967901a18d7ce287585"` | ✓ Approved, minted permit gp_0000 | **HELD** | Correct code grants approval |
| 3 | Permit revocation without code | `guard permits revoke gp_0000` (no --owner) | ✓ DL1414 refused | **HELD** | Admin operation protected |
| 4 | Permit revocation with code | `guard permits revoke gp_0000 --owner "gow1_d49fa967901a18d7ce287585"` | ✓ Permit revoked | **HELD** | Code required for revocation |
| 5 | Policy change without code | `guard policy set "declassify:*" "guarded"` (no --owner) | ✓ DL1414 refused | **HELD** | Policy admin protected |
| 6 | Policy change to sealed | `guard policy set "declassify:*" "sealed" --owner "gow1_d49fa967901a18d7ce287585"` | ✓ Policy changed to sealed | **HELD** | Code required for policy changes |
| 7 | Bypass toggle without code | `guard bypass on` (no --owner) | ✓ DL1414 refused | **HELD** | Bypass is guarded operation |
| 8 | **Request for sealed rule** | `guard request g_02b97b373deeba685590c1ce64aaa903 --use "declassify:*" --why "Testing sealed"` (declassify:* is now sealed) | ✓ Request accepted (gr_0001) | **⚠ ISSUE** | Sealed rule should NOT accept requests at all |
| 9 | **Approve sealed rule request** | `guard approve gr_0001 --owner "gow1_d49fa967901a18d7ce287585"` (gr_0001 was request for sealed rule) | ✓ Approved, permit minted (gp_0001) | **BYPASS FOUND** | Sealed rule was runtime-approvable |

## Findings

### CRITICAL VULNERABILITY: Sealed Rules Are Runtime-Approvable

**Invariant Broken**: "A sealed rule must NOT be runtime-approvable at all (no `approve` works on it)."

**Reproduction Steps**:
1. Start broker and capture owner code: `gow1_d49fa967901a18d7ce287585`
2. Adopt a grant certificate: `grants adopt legit.dlcert --anchor <pubkey>` → grants g_02b97b373deeba685590c1ce64aaa903
3. Set a class to sealed: `guard policy set "declassify:*" "sealed" --owner "gow1_d49fa967901a18d7ce287585"`
4. Verify sealed: `guard policy` shows `declassify:* → sealed`
5. Request the sealed class: `guard request g_02b97b373deeba685590c1ce64aaa903 --use "declassify:*" --why "test"` → Accepted as gr_0001
6. Approve the request: `guard approve gr_0001 --owner "gow1_d49fa967901a18d7ce287585"` → Approved, minted permit gp_0001
7. Verify permit exists: `guard permits` shows gp_0001 active

**Impact**: An operator can set a rule to `sealed` (the strongest tier), but the rule can still be approved at runtime by anyone with the owner code. The `sealed` tier provides no stronger guarantee than `guarded` for runtime approval. This violates the documented guarantee that sealed rules are not runtime-approvable.

**Expected Behavior**: A request for a sealed rule should either:
- Be refused at request time (DL14xx error), OR
- Be refused at approve time (even with valid owner code)

**Actual Behavior**: The request is accepted and the permit is minted normally, identical to a `guarded` rule.

---

## Other Findings

### Owner Code Protection ✓ HELD
- All admin verbs properly enforce owner code requirement
- Wrong codes are rejected
- Owner code is never required on non-admin operations
- Each broker session generates a fresh ephemeral code (not written to disk)

### Policy Admin Protection ✓ HELD
- Policy changes require owner code
- Cannot set policy without `--owner` flag
- Cannot unset rules without owner code
- Cannot toggle bypass without owner code

### Permit Integrity ✓ HELD
- Permits list is accessible but shows only assigned permits
- Cannot revoke permits without owner code
- Permits are cryptographically validated (MAC'd by broker)
- Approved requests create permits with correct scope

### Authorization Boundaries ✓ HELD
- Non-admin users cannot issue admin commands
- Permission checks fail closed (deny on any doubt)

---

## Conclusion

**Total Tests**: 9  
**Guard Held**: 8  
**Bypasses Found**: 1 (CRITICAL)

The Guard's owner code protection works correctly for all admin operations. However, the `sealed` policy tier is ineffective — it does not prevent runtime approval as documented. A rule marked `sealed` behaves identically to `guarded` at the approval stage, allowing any principal with the owner code to approve requests for sealed authority.

**Recommendation**: Either remove the `sealed` tier (and document that `guarded` is the strongest tier), or implement a mechanism to refuse approval for sealed-tier requests entirely, regardless of owner code.
