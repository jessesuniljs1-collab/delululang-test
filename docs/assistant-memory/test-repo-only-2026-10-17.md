---
name: test-repo-only-2026-10-17
description: "Jesse 2026-10-17 night — push to and use ONLY delululang-test; never push to DeluluLang/any other repo, tag, switch origin, move the routine or change visibility until he says so in words"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-17T18:49:50.182Z
---

Jesse, 2026-10-17 (night), verbatim: *"Only push to and use test repo until I tell u to change. Do not change repo,
unless told by me. Always push only to my repo"*.

**Why:** he wants to decide himself when development leaves the test repository; it outranks the 2026-10-17 commission's
launch steps (D-V2-153 §3 move to `DeluluLang` `main`; CHECKLIST-2.0 steps 4–10 — snapshot push, branch protection,
`RELEASES=on`, tag `v2.0.0`, publish, `origin` switch, routine repo change, `delululang-test` private).

**How to apply:** every push goes to `origin` = `github.com/jessesuniljs1-collab/delululang-test` (master, launch/2.0.0,
autosave/laptop). Never push to `DeluluLang` (still empty), never push a tag, never `git remote set-url`, never update
the routine's repository, never change a repository's visibility — until he says so in words. Keep `launch/2.0.0` current
by merging master INTO it; read its CI. Recorded in AGENTS.md push rule, HANDOFF §11.1, V2_DECISION_LOG (`9ad8e9c`).
Supersedes the launch parts of [[owner-release-delegation-2026-10-17]] and [[release-program-progress]]; see also
[[final-public-repo-gate]], [[testing-repo-autopush]].
