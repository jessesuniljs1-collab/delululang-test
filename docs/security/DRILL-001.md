# DRILL-001 — patch runbook rehearsal

**Date:** 2026-07-19. **Runbook:** `SECURITY.md` §4. **Criterion:** Stage 9 acceptance criterion 6.
**Outcome:** the runbook executed end to end in **5 minutes 28 seconds**, well inside every target.
**The drill's real value was not the timing.** It was what the staged vulnerability revealed about
the test suite, in §4.

This is a rehearsal against a **staged** vulnerability planted deliberately in a release-candidate
branch. No real vulnerability is described here.

---

## 1. The staged vulnerability

Planted in `crates/delulu/src/signing.rs`, in `cmd_verify_sig`:

```rust
Err(_) => {
    report_sig("verify-sig", &artifact, &SignatureStatus::Unsigned, json);
    return 0;   // was 1
}
```

An artifact with **no signature at all** printed `unsigned` and exited **0**.

**Why this shape was chosen.** It is the archetype the house rule exists for: the rule is enforced
correctly everywhere the checker has something to check, and fails open on the branch where it has
nothing. Every *substantive* verification path — valid signature, tampered bytes, mismatched pinned
key — remained correct. Only the "there is nothing here to verify" branch lied.

**Severity: High** (`SECURITY.md` §3) — verification subverted without breaking the type system. Any
caller gating on the exit code, which is every shell script and CI job, would accept an unsigned
artifact.

## 2. Timeline

| Phase | Time | Elapsed | What happened |
|---|---|---|---|
| T0 — receipt | 15:47:16 | — | Report received, RC branch `rc/1.0.0-drill` created |
| T1 — reproduce | 15:47:48 | 0:32 | Reproduced: unsigned artifact → exit 0. Severity assigned: High |
| T2 — blast radius | 15:49:45 | 2:29 | Full suite run **with the vulnerability in place** |
| T3 — failing test | 15:50:13 | 2:57 | Regression test written; confirmed it **fails** against the vuln |
| T4 — fix | 15:50:50 | 3:34 | Fix applied; targeted suite green |
| T5 — full verification | 15:52:32 | 5:16 | Full workspace suite: **876 passed, 0 failed**; coverage law re-run |
| T6 — signed release | 15:52:44 | 5:28 | Emergency artifact signed, signature verified, and re-verified as failing once the signature was removed |
| T7 — advisory | 15:52:44 | 5:28 | This document |

**Against the runbook's targets:** High severity targets a fix within 14 days. Elapsed: **5m28s**.

### 2.1 What the timing does and does not prove

It proves the *mechanics* work: the branch, the test-first discipline, the full-suite gate, the
signing path, and the advisory all connect without a missing step. It does **not** prove a real
incident would take five minutes. A real one includes triage under uncertainty, reproducing from
someone else's report, deciding on embargo, and coordinating with a reporter — none of which a
self-planted bug requires. Treat this as a proof that the pipeline has no gaps, not as a
service-level commitment.

## 3. Verification performed

- Regression test fails against the vulnerability, passes against the fix (both directions checked —
  a test that passes either way pins nothing).
- Full workspace suite after the fix: **876 passed, 0 failed, 5 ignored**.
- Conformance coverage law re-run: no regression.
- Emergency artifact signed and verified; then the signature was **removed** and verification
  correctly failed with exit 1 — the drill's own fix, exercised on a release artifact.

## 4. Retrospective — why the tests did not catch this

**This is the finding that mattered.**

With the vulnerability planted, the **entire 875-test suite passed**. Not one test failed. A
signature-verification bypass sat in the tree and every gate said green.

**Root cause:** every existing signing test asserted on the rendered *verdict* — the `unsigned` /
`valid` / `invalid` string, or the JSON envelope's fields. None asserted the **exit code** for the
missing-signature path. The verdict string was still correct; the vulnerability lived entirely in
the number the process returned, which is precisely the channel a scripted caller uses and a human
reading test output never sees.

There is a second, sharper reading. The suite tested the paths where verification *does something*
and skipped the path where it has nothing to do. That is the same class of gap this project has hit
before — the Stage-6 `DL0803` fail-open, and the Stage-9a primitive-table arity gate, which also
lived in the branch where the checker had nothing to check.

### 4.1 Actions taken

1. **`an_unsigned_artifact_fails_verification_by_exit_code`** added permanently to
   `crates/delulu/tests/signing_cli.rs`. It asserts the exit code *and* that the JSON envelope
   agrees with it — a machine surface that disagrees with the exit code is a bypass waiting to be
   found.
2. **The house rule is restated in `CONTRIBUTING.md` §1** in terms of this failure mode: if you
   cannot make your new gate fail on purpose, it is not a gate.

### 4.2 Actions recommended, not yet taken

Recorded honestly as open rather than quietly closed:

- **Audit every security-relevant CLI path for exit-code assertions.** `verify-sig` is fixed; the
  same "verdict asserted, exit code not" gap could exist in `plugin verify`, `audit verify`, and
  `build --locked`. Not audited in this drill.
- **Consider a lint or harness rule** that a test invoking the binary on a refusal path must assert
  the exit code. The bypass survived because asserting a message *feels* like asserting the
  behavior.

## 5. Cleanup

The staged vulnerability existed only on `rc/1.0.0-drill` and was never merged. What returns to
`master` is the regression test, this record, and the fix — which on `master` is a no-op, because
the vulnerability was never there. The test is not decorative: it now pins a property the suite
demonstrably did not hold anyone to.
