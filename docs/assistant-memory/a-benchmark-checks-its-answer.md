---
name: a-benchmark-checks-its-answer
description: "2026-10-19: Study C timed any lane that exited 0 and truncated to whole milliseconds; on a Linux VM a C process read 0 and dropped out of every ratio. Hold each lane to the reference's answer, time below the fastest lane's scale, and record the date and machine"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-19T15:00:00.000Z
---

A measurement harness must check each lane's ANSWER before it times it, and its unit must be finer than the fastest
lane on the machine it runs on.

**Why:** Study C (`crates/delulu-measure/src/study_c.rs`) timed a lane whenever its process exited 0 — its comment said
"a benchmark where the lanes do different work measures nothing", and nothing compared outputs — just after RW 3.4 grew the
WASM lane from one benchmark to four. And it used `as_millis()`: the v1.0 run on the laptop read C at 5–6 ms, so nobody saw
it; the first run on a Linux VM read every C process as 1 or 0 ms, and the benchmark at 0 vanished from both ratio tables
without a word. Found by the on-demand routine's fourth run re-measuring for the hot-path table; fixed with mutants
M513–M518 red. The re-measure moved the published range from 2.0×–60.5× to 8.9×–359.0× (interpreter) — not because the
interpreter got slower (its own times matched) but because a Linux C process starts in ~1 ms.

**How to apply:** before re-running any study on a new machine, read the harness for (a) what it does when a lane prints
another answer, (b) what its unit reads at the fastest lane there, (c) whether its record names the date and machine. Keep
the run being replaced whole (`measurements/study-c/runs/<date>/`), and update every document that quotes the old figure —
`grep` the number itself.
