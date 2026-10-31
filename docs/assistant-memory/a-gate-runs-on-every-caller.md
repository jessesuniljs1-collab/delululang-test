---
name: a-gate-runs-on-every-caller
description: "2026-10-31: a new gate in scripts/launch-gates.sh (criterion-2-cited) passed the code merge it was written on and would have failed the records-only citations merge that follows every launch run; test a gate against every call the procedure makes"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-31T14:00:00.000Z
---

A new gate is falsified against every call the procedure makes of it, not only the one it was written on.

**Why:** on-demand routine run 13 brought `launch/2.0.0` current with `master` and added `criterion-2-cited` to
`scripts/launch-gates.sh`, so that CHECKLIST-2.0's criterion 2 row — re-measured and written by hand on runs 8, 9, 11 and
13 — is read back. Its first form asked the row to cite the merged `master`'s own hash. It passed on the code merge, and
went red on cue for a base the row never cited. But every launch run makes a second merge: the citations merge, which
brings in the run's records-only closing commit, a hash the row never cites — the first form would have refused a
correct tree. Found before that merge, by listing the procedure's calls of the script. The gate now takes the row's
newest citation at the measured count and passes when criterion 2's inputs (`crates`, `tests`, `examples`,
`docs/reference`, `docs/book/samples`, `Cargo.toml`, `Cargo.lock`) are the same there as at the base.

**How to apply:** before landing a gate, list each place the procedure runs it (here: the code merge, the citations
merge) and run it on each shape, plus the defects it must catch (here: a base whose inputs changed, a planted count, a
removed citation — each exit 1).
