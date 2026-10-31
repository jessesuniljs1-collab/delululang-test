---
name: a-gate-counts-what-it-reads
description: "2026-10-31: `survey findings` exits 0 whatever it finds, and three launch merges read its error count by eye; scripts/launch-gates.sh counts it and fails a gate that did not run. And judge the 2.0.0 Breaking paragraph against v1.0.0's own code"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-31T09:00:00.000Z
---

A command that exits 0 whatever it prints is not a gate until something counts what it says — and a gate that did not
run is a failure, never a skip.

**Why:** release procedure step 1 (merging `master` into `launch/2.0.0`) asks for the Survey's findings at 0 errors.
`delulu-survey findings` prints `error: …` lines and exits 0 either way, so on-demand routine runs 8, 9 and 11 each read
the count by eye among a dozen hand-typed gate commands. On-demand routine run 11 wrote `scripts/launch-gates.sh`: it
counts the errors (red on a planted broken link), fails when `cargo-deny` is absent (exit 127), and ends `LAUNCH-GATES=ok`
only when every gate ran and passed (`partial-ok` for a named subset). The same run judged RW 4.79 for the 2.0.0
*Breaking* paragraph: whether a moved code is breaking depends on what 1.x did, not on the V2 state just before — and
v1.0.0's interpreter already asked the daemon's custody before the capability (`interp.rs:851` at the tag).

**How to apply:** before trusting a check in a script or a step, make it fail once (plant the defect) and read its exit
code, not its words. For a launch merge, `git fetch origin tag v1.0.0` and read `git show v1.0.0:PATH` for each section
that moves a code, an exit code, a JSON field or syntax.
