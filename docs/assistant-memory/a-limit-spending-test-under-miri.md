---
name: a-limit-spending-test-under-miri
description: "A test that spends a production limit spends it under Miri too, and miri-slow never runs on a PR — the pigeonhole test spent the checker's 10M-cell limit, 3.3 h under Miri, and timed out the next two full runs (2026-10-22)"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-22T15:32:12.722Z
---

`a_match_the_search_cannot_finish_is_refused_not_passed` (PR #34, MATCH-EXHAUST-1) spent the checker's 10,000,000-cell
exhaustiveness limit: under a second natively, 11,873 s under Miri (~1 ms a cell). PR CI never runs `miri-slow`, so it
landed green; then `miri-slow (delulu-check)` hit its 240 min on launch/2.0.0's everything run 39678647200 and master's
nightly 39746314492 (57 min the day before). Fix (PR #42): `EXHAUSTIVE_BUDGET = if cfg!(miri) { 10_000 } else { 10_000_000 }`
(precedent: `audit.rs` `LOCK_WAIT`), test sized to the compiled arm (Miri: 4 holes refused, 3 proved).

**Why:** a cancelled `jobs=everything` blocks the launch branch's evidence, and the cause surfaces a day late.

**How to apply:** before landing a test that spends a bound or a big loop, multiply its native work by Miri's cost, or
dispatch `ci.yml -f jobs=miri-slow` at the branch. Locate a Miri timeout by the gaps between the job log's `test … ok`
lines (fetch with `gh api --allow-escape-sequences …/jobs/<id>/logs`). Verify a Miri arm natively by forcing
`cfg!(miri)` to `true` in a scratch copy (run + mutant), then restore. Related: [[mutant-run-per-target]],
[[delulu-p18-uncertainty]], [[release-program-progress]].
