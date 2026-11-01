---
name: check-passes-with-a-warning
description: "Three ways a check passed what it should not (2026-10-22): Lean's sorry is a warning with exit 0; a crate's tests built alone get different features than CI's workspace build; a renamed test can be a dangling conformance witness"
metadata:
  type: feedback
---

1. **Lean reports `sorry` as a warning and exits 0.** A planted `theorem planted : False := by sorry` checked "clean"; CI's
   `formal` step read only the exit code. The step now greps the output for `sorry` (D-V2-175, PR #49), and two rule
   mutants must fail to check.
2. **`cargo test -p delulu-wasm` alone builds `delulu-runtime` without `net`** (`no-client`), and CI's workspace build has
   it on, because `delulu` turns it on and cargo unifies features (`connect`). A test that asserted `no-client` was green on
   the laptop and red on CI (PR #48, run 39825874508).
3. **A test I renamed was a conformance witness** (`tests/conformance/witnesses.toml`, entrenched); only `cargo test -p
   delulu-conform` sees a dangling one.

**Why:** each was green where it was checked and wrong where it mattered.

**How to apply:** for any checker, ask what it reports as a warning and still passes. Assert what holds in both builds
(crate alone and workspace). After renaming or removing a test, grep `witnesses.toml` and run `delulu-conform`.
HANDOFF §11.5 has the repo copy. Related: [[mutant-run-per-target]], [[delulu-clean-checkout]].
